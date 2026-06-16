//! Access to the embedded fluxid widget payload.
//!
//! `build.rs` produces `OUT_DIR/payload.bin` — either the real `fluxid.exe`
//! (packaged build) or an empty placeholder (plain `cargo build`).

/// The embedded `fluxid.exe`, or empty in a dev build (see `build.rs`).
pub const FLUXID_EXE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.bin"));

/// Whether a real payload was embedded. `false` in a dev build, in which case
/// the installer refuses to run the file-copy step.
pub fn is_bundled() -> bool {
    !FLUXID_EXE.is_empty()
}

/// Human-readable size of the embedded payload (for the UI).
pub fn size_mb() -> f32 {
    FLUXID_EXE.len() as f32 / (1024.0 * 1024.0)
}
