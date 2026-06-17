//! Shared data model for Flux: persisted settings and sensor snapshots.

pub mod settings;
pub mod sensor_data;
pub mod sensor_ipc;

pub use settings::AppSettings;
pub use sensor_data::SensorSnapshot;
