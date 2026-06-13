use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme_bg: String,
    pub theme_tile: String,
    pub theme_accent: String,
    pub theme_text: String,
    pub theme_muted: String,
    pub active_skin: String,
    pub primary_font: Option<String>,
    pub secondary_font: Option<String>,
    pub indicator_font: Option<String>,
    pub font_size_offset: f32,

    pub orientation: Orientation,
    pub tile_order: Vec<String>,
    pub visible_tiles: Vec<String>,
    pub widget_opacity: f32,
    pub click_through: bool,

    pub window_x: f64,
    pub window_y: f64,
    pub settings_window_x: Option<f64>,
    pub settings_window_y: Option<f64>,
    pub snap_to_edges: bool,

    pub game_mode_enabled: bool,
    pub game_mode_hotkey: String,
    pub game_mode_position: SnapPosition,
    pub game_mode_opacity: f32,
    pub game_mode_tiles: Vec<String>,

    pub warnings: Vec<TileWarning>,

    pub remote_enabled: bool,
    pub remote_port: u16,
    pub remote_key: String,
    pub remote_devices: Vec<RemoteDevice>,

    pub update_check_mode: UpdateMode,
    pub last_update_check: Option<String>,
    pub presets: Vec<PresetSlot>,

    pub temperature_unit: TempUnit,
    pub start_minimized: bool,
    pub first_run_complete: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // C# dark defaults (ThemeApplier.cs)
            theme_bg: "#E61E1E22".into(),
            theme_tile: "#FF2A2A30".into(),
            theme_accent: "#FF00A8FF".into(),
            theme_text: "#FFE8E8EC".into(),
            theme_muted: "#FF9A9AA8".into(),
            active_skin: "Default".into(),
            primary_font: None,
            secondary_font: None,
            indicator_font: None,
            font_size_offset: 0.0,

            orientation: Orientation::Horizontal,
            tile_order: vec![
                "CPU".into(), "GPU".into(), "RAM".into(),
                "Disk".into(), "Network".into(), "Clock".into(),
            ],
            visible_tiles: vec![
                "CPU".into(), "GPU".into(), "RAM".into(),
                "Disk".into(), "Network".into(),
            ],
            widget_opacity: 1.0,
            click_through: false,

            window_x: 100.0,
            window_y: 100.0,
            settings_window_x: None,
            settings_window_y: None,
            snap_to_edges: true,

            game_mode_enabled: false,
            game_mode_hotkey: "Ctrl+G".into(),
            game_mode_position: SnapPosition::TopRight,
            game_mode_opacity: 0.8,
            game_mode_tiles: vec!["CPU".into(), "GPU".into(), "RAM".into()],

            warnings: vec![
                TileWarning { kind: "CPU".into(), enabled: false, metric: WarnMetric::Load,        threshold: 90.0, flash_enabled: true, flash_color: "#FFFF3333".into(), gradient_mode: false },
                TileWarning { kind: "GPU".into(), enabled: false, metric: WarnMetric::Temperature, threshold: 85.0, flash_enabled: true, flash_color: "#FFFF3333".into(), gradient_mode: true },
            ],

            remote_enabled: false,
            remote_port: 5199,
            remote_key: String::new(),
            remote_devices: Vec::new(),

            update_check_mode: UpdateMode::Manual,
            last_update_check: None,
            presets: Vec::new(),

            temperature_unit: TempUnit::Celsius,
            start_minimized: false,
            first_run_complete: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Orientation { Vertical, Horizontal }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SnapPosition { TopLeft, TopRight, BottomLeft, BottomRight }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateMode { Auto, Manual, Off }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TempUnit { Celsius, Fahrenheit }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum WarnMetric {
    #[default]
    Temperature,
    Load,
    UsedGb,
    Throughput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TileWarning {
    pub kind: String,
    pub enabled: bool,
    pub metric: WarnMetric,
    pub threshold: f64,
    pub flash_enabled: bool,
    pub flash_color: String,
    pub gradient_mode: bool,
}

impl Default for TileWarning {
    fn default() -> Self {
        Self {
            kind: String::new(),
            enabled: false,
            metric: WarnMetric::Temperature,
            threshold: 85.0,
            flash_enabled: true,
            flash_color: "#FFFF3333".into(),
            gradient_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDevice {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetSlot {
    pub name: String,
    pub bg: String,
    pub tile: String,
    pub accent: String,
    pub text: String,
    pub muted: String,
    pub skin: String,
}

impl AppSettings {
    pub fn warn(&self, kind: &str) -> Option<&TileWarning> {
        self.warnings.iter().find(|w| w.kind == kind)
    }

    pub fn warn_mut(&mut self, kind: &str) -> &mut TileWarning {
        if !self.warnings.iter().any(|w| w.kind == kind) {
            self.warnings.push(TileWarning { kind: kind.to_string(), ..Default::default() });
        }
        self.warnings.iter_mut().find(|w| w.kind == kind).unwrap()
    }

    pub fn config_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "fluidmonitor", "fluidMonitor")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let json = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&json)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }
}
