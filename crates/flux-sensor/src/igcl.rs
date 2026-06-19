#![cfg(windows)]
#![allow(dead_code)] // layout structs documented for offsets; probe reads raw bytes
//! Optional Intel GPU telemetry via IGCL (Intel Graphics Control Library,
//! `ControlLib.dll`) — used to read the live GPU **core clock** on Intel parts,
//! where D3DKMT reports 0 Hz at idle (so the tile would otherwise show "—").
//!
//! Fully optional and dynamically loaded: the DLL is absent on non-Intel systems
//! (load fails → no-op), and IGCL's Size/Version handshake means a wrong struct
//! layout fails cleanly rather than returning garbage. Any failure falls back to
//! the D3DKMT node-scan in `d3dkmt::read_clock_temp`.
//!
//! This first cut exposes only `probe()` (a diagnostic for `--gpu-debug`); it is
//! NOT yet wired into the live clock path. The probe even sweeps the telemetry
//! struct's `Size` field until the driver accepts it, so a single run on real
//! Intel hardware reveals both reachability and the exact struct size.

use std::ffi::c_void;
use std::fmt::Write as _;
use windows::core::{s, w};
use windows::Win32::Foundation::{FreeLibrary, HMODULE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

type ApiHandle = *mut c_void;
type DeviceHandle = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy)]
struct AppId {
    d1: u32,
    d2: u16,
    d3: u16,
    d4: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VersionInfo {
    major: u16,
    minor: u16,
}

#[repr(C)]
struct InitArgs {
    size: u32,
    version: u8,
    app_version: VersionInfo,
    flags: u32,
    supported_version: VersionInfo,
    application_uid: AppId,
}

// ctl_oc_telemetry_item_t: { bool bSupported; ctl_units_t units; ctl_data_type_t
// type; ctl_data_value_t value; }. With #[repr(C)] this is 24 bytes (the f64 value
// forces 8-byte alignment): b_supported@0, units@4, data_type@8, value@16.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct TelemetryItem {
    b_supported: u8,
    units: i32,
    data_type: i32,
    value: f64,
}

// FFI signatures. On x64 Windows there is a single calling convention, so
// `extern "system"` matches IGCL's CTL_APICALL regardless of cdecl/stdcall.
type FnInit = unsafe extern "system" fn(*mut InitArgs, *mut ApiHandle) -> i32;
type FnEnumerate = unsafe extern "system" fn(ApiHandle, *mut u32, *mut DeviceHandle) -> i32;
type FnTelemetry = unsafe extern "system" fn(DeviceHandle, *mut c_void) -> i32;
type FnClose = unsafe extern "system" fn(ApiHandle) -> i32;

/// Byte offsets within `ctl_power_telemetry_t` for the field we care about. The
/// header (Size u32 @0, Version u8 @4) is followed by 8-aligned telemetry items
/// starting at 8; `gpuCurrentClockFrequency` is the 4th item (8 + 3*24 = 80), and
/// its `value` (a double) sits 16 bytes into the item.
const CLOCK_ITEM_OFFSET: usize = 80;
const CLOCK_SUPPORTED_OFFSET: usize = CLOCK_ITEM_OFFSET; // b_supported @ item+0
const CLOCK_UNITS_OFFSET: usize = CLOCK_ITEM_OFFSET + 4;
const CLOCK_VALUE_OFFSET: usize = CLOCK_ITEM_OFFSET + 16; // value (f64) @ item+16

/// A nonzero application UID (IGCL rejects an all-zero UID).
const APP_UID: AppId = AppId {
    d1: 0xF100_D000,
    d2: 0x4C5F,
    d3: 0x11EE,
    d4: [0xB9, 0x62, 0x02, 0x42, 0xAC, 0x12, 0x00, 0x02],
};

/// Full IGCL diagnostic for `--gpu-debug`: load the DLL, init, enumerate devices,
/// and (sweeping the Size field) attempt `ctlPowerTelemetryGet`, reporting the GPU
/// core clock if it succeeds. Best-effort; never panics.
pub fn probe() -> String {
    let mut s = String::new();
    let _ = writeln!(s, "\n-- IGCL (Intel control library) --");
    unsafe { probe_inner(&mut s) };
    s
}

unsafe fn probe_inner(s: &mut String) {
    // 1. Load ControlLib.dll (present only with Intel's graphics driver/runtime).
    let hmod: HMODULE = match LoadLibraryW(w!("ControlLib.dll")) {
        Ok(h) if !h.is_invalid() => h,
        _ => {
            let _ = writeln!(s, "  ControlLib.dll: not found (no Intel control runtime — expected on non-Intel)");
            return;
        }
    };
    let _ = writeln!(s, "  ControlLib.dll: loaded");

    // 2. Resolve the entry points.
    let init: FnInit = match GetProcAddress(hmod, s!("ctlInit")) {
        Some(p) => std::mem::transmute(p),
        None => {
            let _ = writeln!(s, "  ctlInit: not exported");
            let _ = FreeLibrary(hmod);
            return;
        }
    };
    let enumerate: Option<FnEnumerate> =
        GetProcAddress(hmod, s!("ctlEnumerateDevices")).map(|p| std::mem::transmute(p));
    let telemetry: Option<FnTelemetry> =
        GetProcAddress(hmod, s!("ctlPowerTelemetryGet")).map(|p| std::mem::transmute(p));
    let close: Option<FnClose> =
        GetProcAddress(hmod, s!("ctlClose")).map(|p| std::mem::transmute(p));
    let _ = writeln!(
        s,
        "  exports: ctlInit=yes enumerate={} telemetry={} close={}",
        enumerate.is_some(),
        telemetry.is_some(),
        close.is_some()
    );

    // 3. ctlInit.
    let mut args = InitArgs {
        size: std::mem::size_of::<InitArgs>() as u32,
        version: 0,
        app_version: VersionInfo { major: 1, minor: 0 },
        flags: 0,
        supported_version: VersionInfo { major: 0, minor: 0 },
        application_uid: APP_UID,
    };
    let mut api: ApiHandle = std::ptr::null_mut();
    let r = init(&mut args, &mut api);
    let _ = writeln!(
        s,
        "  ctlInit: result={r} (0=ok)  supported v{}.{}",
        args.supported_version.major, args.supported_version.minor
    );
    if r != 0 || api.is_null() {
        let _ = FreeLibrary(hmod);
        return;
    }

    // 4. Enumerate devices.
    if let (Some(enumerate), Some(telemetry)) = (enumerate, telemetry) {
        let mut count: u32 = 0;
        let r = enumerate(api, &mut count, std::ptr::null_mut());
        let _ = writeln!(s, "  ctlEnumerateDevices: result={r}  count={count}");
        if r == 0 && count > 0 {
            let mut devices: Vec<DeviceHandle> = vec![std::ptr::null_mut(); count as usize];
            let r = enumerate(api, &mut count, devices.as_mut_ptr());
            if r == 0 {
                for (i, &dev) in devices.iter().enumerate() {
                    probe_device_telemetry(s, telemetry, i, dev);
                }
            } else {
                let _ = writeln!(s, "  ctlEnumerateDevices (fill): result={r}");
            }
        }
    }

    if let Some(close) = close {
        let _ = close(api);
    }
    let _ = FreeLibrary(hmod);
}

/// Sweep the telemetry struct's Size field until the driver accepts it, then read
/// the GPU core clock from the known offset. Reports the working size so the live
/// path can use it directly next iteration.
unsafe fn probe_device_telemetry(s: &mut String, telemetry: FnTelemetry, idx: usize, dev: DeviceHandle) {
    // 8-aligned scratch buffer big enough for any plausible ctl_power_telemetry_t.
    let mut buf = vec![0u64; 512]; // 4096 bytes
    let base = buf.as_mut_ptr() as *mut u8;

    for size in (256u32..=1400).step_by(4) {
        // Re-zero and set the Size/Version handshake fields.
        std::ptr::write_bytes(base, 0, buf.len() * 8);
        *(base as *mut u32) = size;
        *base.add(4) = 1u8; // Version
        let r = telemetry(dev, base as *mut c_void);
        if r == 0 {
            let supported = *base.add(CLOCK_SUPPORTED_OFFSET) != 0;
            let units = *(base.add(CLOCK_UNITS_OFFSET) as *const i32);
            let value = *(base.add(CLOCK_VALUE_OFFSET) as *const f64);
            let _ = writeln!(
                s,
                "  device {idx}: telemetry OK at Size={size}  clock.supported={supported} units={units} value={value:.1}",
            );
            return;
        }
    }
    // Nothing accepted — report the result for our own struct size as a hint.
    std::ptr::write_bytes(base, 0, buf.len() * 8);
    *(base as *mut u32) = 1024;
    *base.add(4) = 1u8;
    let r = telemetry(dev, base as *mut c_void);
    let _ = writeln!(s, "  device {idx}: no Size in 256..1400 accepted (last result={r})");
}
