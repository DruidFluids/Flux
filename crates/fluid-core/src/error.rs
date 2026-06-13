use thiserror::Error;

#[derive(Error, Debug)]
pub enum FluidError {
    #[error("Settings error: {0}")]
    Settings(String),

    #[error("Theme error: {0}")]
    Theme(String),

    #[error("Sensor error: {0}")]
    Sensor(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
