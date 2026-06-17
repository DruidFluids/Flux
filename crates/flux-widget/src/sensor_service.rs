//! The elevated Flux **sensor service** (Windows only).
//!
//! `flux.exe --sensor-service` is registered as a Windows service. Running as
//! LocalSystem it can read the CPU die temperature via PawnIO (which requires
//! elevation) and publishes it through `flux_core::sensor_ipc` for the
//! non-elevated widget to read. This is the CAM/iCUE model: one privileged
//! service, a normal-user UI, no UAC after the one-time install.
//!
//! Install / uninstall touch the Service Control Manager and so must run
//! elevated — they're driven from `cpu_driver.rs` via an elevated
//! `flux.exe --install-sensor-service` / `--uninstall-sensor-service`.

use std::ffi::{OsStr, OsString};
use std::sync::mpsc;
use std::time::Duration;

use windows_service::service::{
    ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
    ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

const SERVICE_NAME: &str = "FluxSensorService";
const SERVICE_DISPLAY: &str = "Flux Sensor Service";
/// The argument that tells `flux.exe` to run as the service (the SCM launches it).
pub const SERVICE_ARG: &str = "--sensor-service";

windows_service::define_windows_service!(ffi_service_main, service_main);

/// SCM entry point: register the dispatcher and block until the service stops.
pub fn run() -> Result<(), String> {
    windows_service::service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .map_err(|e| e.to_string())
}

fn service_main(_args: Vec<OsString>) {
    // The SCM has no channel back here; on error it simply marks the service
    // failed. Nothing actionable to do but return.
    let _ = run_service();
}

fn run_service() -> Result<(), String> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

    let event_handler = move |control| match control {
        ServiceControl::Stop | ServiceControl::Shutdown => {
            let _ = shutdown_tx.send(());
            ServiceControlHandlerResult::NoError
        }
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        _ => ServiceControlHandlerResult::NotImplemented,
    };

    let status_handle =
        service_control_handler::register(SERVICE_NAME, event_handler).map_err(|e| e.to_string())?;

    let status = |state: ServiceState, accept: ServiceControlAccept| ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: state,
        controls_accepted: accept,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    };

    status_handle
        .set_service_status(status(
            ServiceState::Running,
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        ))
        .map_err(|e| e.to_string())?;

    // Poll PawnIO ~once a second and publish; stop promptly when asked.
    loop {
        let temp = flux_sensor::privileged_cpu_temp();
        let _ = flux_core::sensor_ipc::write(&flux_core::sensor_ipc::SensorReadout {
            cpu_temp: temp,
            updated_unix: flux_core::sensor_ipc::now_unix(),
        });
        if shutdown_rx.recv_timeout(Duration::from_secs(1)).is_ok() {
            break;
        }
    }

    status_handle
        .set_service_status(status(ServiceState::Stopped, ServiceControlAccept::empty()))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Register the service (auto-start) and start it. Must run elevated.
pub fn install() -> Result<(), String> {
    let manager = ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
    )
    .map_err(|e| e.to_string())?;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: exe,
        launch_arguments: vec![OsString::from(SERVICE_ARG)],
        dependencies: vec![],
        account_name: None, // LocalSystem
        account_password: None,
    };

    // Create, or reuse an existing registration after an in-place update.
    let service = match manager.create_service(
        &info,
        ServiceAccess::START | ServiceAccess::CHANGE_CONFIG | ServiceAccess::QUERY_STATUS,
    ) {
        Ok(s) => s,
        Err(_) => manager
            .open_service(
                SERVICE_NAME,
                ServiceAccess::START | ServiceAccess::QUERY_STATUS,
            )
            .map_err(|e| e.to_string())?,
    };
    let _ = service.set_description(
        "Publishes the CPU die temperature (via the PawnIO driver) to the Flux widget.",
    );
    // Start now if it isn't already running.
    let already = matches!(
        service.query_status().map(|s| s.current_state),
        Ok(ServiceState::Running) | Ok(ServiceState::StartPending)
    );
    if !already {
        service.start(&[] as &[&OsStr]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Stop and delete the service. Must run elevated. No-op if not installed.
pub fn uninstall() -> Result<(), String> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|e| e.to_string())?;
    let service = match manager.open_service(
        SERVICE_NAME,
        ServiceAccess::STOP | ServiceAccess::DELETE | ServiceAccess::QUERY_STATUS,
    ) {
        Ok(s) => s,
        Err(_) => return Ok(()), // not installed
    };
    if !matches!(
        service.query_status().map(|s| s.current_state),
        Ok(ServiceState::Stopped)
    ) {
        let _ = service.stop();
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(100));
            if matches!(
                service.query_status().map(|s| s.current_state),
                Ok(ServiceState::Stopped)
            ) {
                break;
            }
        }
    }
    service.delete().map_err(|e| e.to_string())
}

/// True when the service is registered and running.
pub fn is_running() -> bool {
    let Ok(manager) = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
    else {
        return false;
    };
    let Ok(service) = manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS) else {
        return false;
    };
    matches!(
        service.query_status().map(|s| s.current_state),
        Ok(ServiceState::Running) | Ok(ServiceState::StartPending)
    )
}
