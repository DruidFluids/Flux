//! Shared channel between the elevated Flux sensor **service** and the
//! non-elevated **widget**.
//!
//! Reading the CPU die temperature via the PawnIO driver requires an elevated
//! process. To keep the widget itself non-elevated (so the in-app updater works
//! without UAC and it can sit in normal startup), an elevated Windows service
//! does the privileged read and publishes it here; the widget just reads it.
//!
//! Transport is a small JSON file under `%ProgramData%\Flux\`. ProgramData's
//! default ACL lets the SYSTEM-level service write it and every user read it,
//! so no custom security descriptor is needed (unlike a named pipe). Writes are
//! atomic (temp file + rename) so the widget never sees a torn value.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// What the service publishes for the widget to consume.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensorReadout {
    /// CPU package/die temperature in °C, if the service could read it.
    pub cpu_temp: Option<f32>,
    /// Unix seconds when this was written — used to reject stale data if the
    /// service has stopped.
    pub updated_unix: u64,
}

/// `%ProgramData%\Flux\sensors.json` — written by the service, read by the widget.
pub fn ipc_path() -> PathBuf {
    let base = std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    base.join("Flux").join("sensors.json")
}

/// Read the latest published readout, or `None` if absent/unparsable.
pub fn read() -> Option<SensorReadout> {
    let txt = std::fs::read_to_string(ipc_path()).ok()?;
    serde_json::from_str(&txt).ok()
}

/// Atomically publish a readout (temp file + rename). Best-effort.
pub fn write(readout: &SensorReadout) -> std::io::Result<()> {
    let path = ipc_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string(readout).unwrap_or_default())?;
    std::fs::rename(&tmp, &path)
}

/// Current Unix time in seconds.
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A readout is "fresh" (the service is alive) if written within this window.
pub const FRESH_SECS: u64 = 6;
