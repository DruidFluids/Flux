//! Shared data model for Fluxid: persisted settings and sensor snapshots.

pub mod settings;
pub mod sensor_data;

pub use settings::AppSettings;
pub use sensor_data::SensorSnapshot;
