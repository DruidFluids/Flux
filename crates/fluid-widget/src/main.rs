mod tile;
mod style;
mod fmt;
mod settings_panel;

use fluid_core::sensor_data::SensorSnapshot;
use fluid_core::settings::{AppSettings, Orientation, SnapPosition, TempUnit, WarnMetric};
use fluid_sensor::SensorPoller;
use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::{window, Border, Color, Element, Length, Point, Size, Subscription, Task, Theme};
use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};
use style::Palette;
use tile::WarnView;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};

const SETTINGS_SIZE: Size = Size::new(720.0, 900.0);
const SNAP_MARGIN: f32 = 20.0;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    iced::daemon("fluidMonitor", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run_with(App::new)
}

fn make_tray_icon() -> tray_icon::Icon {
    const SIZE: u32 = 32;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let corner = 6i32;
            let (xi, yi, s) = (x as i32, y as i32, SIZE as i32);
            let in_corner = (xi < corner && yi < corner && (corner - xi).pow(2) + (corner - yi).pow(2) > corner.pow(2))
                || (xi >= s - corner && yi < corner && (xi - (s - corner)).pow(2) + (corner - yi).pow(2) > corner.pow(2))
                || (xi < corner && yi >= s - corner && (corner - xi).pow(2) + (yi - (s - corner)).pow(2) > corner.pow(2))
                || (xi >= s - corner && yi >= s - corner && (xi - (s - corner)).pow(2) + (yi - (s - corner)).pow(2) > corner.pow(2));
            if in_corner { rgba.extend_from_slice(&[0,0,0,0]); }
            else { rgba.extend_from_slice(&[0,168,255,255]); }
        }
    }
    tray_icon::Icon::from_rgba(rgba, SIZE, SIZE).expect("tray icon")
}

#[cfg(target_os = "windows")]
fn work_area() -> Option<(f32, f32, f32, f32)> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS};
    let mut rect = RECT::default();
    unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, Some(&mut rect as *mut _ as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)).ok()?; }
    Some((rect.left as f32, rect.top as f32, rect.right as f32, rect.bottom as f32))
}
#[cfg(not(target_os = "windows"))]
fn work_area() -> Option<(f32, f32, f32, f32)> { None }

#[cfg(target_os = "windows")]
fn set_run_at_startup(on: bool) {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok((key, _)) = hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
        if on { if let Ok(exe) = std::env::current_exe() { let _ = key.set_value("fluidMonitor", &exe.to_string_lossy().to_string()); } }
        else { let _ = key.delete_value("fluidMonitor"); }
    }
}
#[cfg(not(target_os = "windows"))]
fn set_run_at_startup(_: bool) {}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WindowKind { Widget, Settings }

struct App {
    settings: AppSettings,
    snapshot: SensorSnapshot,
    poller: Option<SensorPoller>,
    windows: BTreeMap<window::Id, WindowKind>,
    warn_state: HashMap<String, (bool, Option<Color>)>,
    flash_on: bool,
    editing_color: Option<u8>,
    game_mode: bool,
    pending_snap: Option<(window::Id, Point, Instant)>,
    ignore_next_move: bool,
    _tray: TrayIcon,
    settings_id: tray_icon::menu::MenuId,
    show_id: tray_icon::menu::MenuId,
    game_id: tray_icon::menu::MenuId,
    exit_id: tray_icon::menu::MenuId,
}

#[derive(Debug, Clone)]
enum Message {
    SensorTick, TrayPoll, FlashTick,
    DragWindow(window::Id),
    WindowOpened(window::Id, WindowKind),
    WindowClosed(window::Id),
    WindowMoved(window::Id, Point),
    OpenSettings, HideWidget, SaveClose, ResetDefaults, Noop,
    ToggleTile(String, bool),
    SetOpacity(f32), SetOrientation(Orientation),
    SetAccent(String), SetFahrenheit(bool), SetSnap(bool),
    ThemePrev, ThemeNext, ThemeDice,
    SetWarnEnabled(String, bool), SetWarnThreshold(String, f32),
    SetWarnFlash(String, bool), SetWarnGradient(String, bool),
    SetHexColor(u8, String),
    SetTileWidth(f32), SetTileHeight(f32),
    SetPrimaryFontOffset(f32), SetSecondaryFontOffset(f32), SetIndicatorFontOffset(f32),
    SetMutedContrast(f32), SetInterval(f32),
    SetCpuName(String), SetGpuName(String),
    SetDisk(String), SetAdapter(String),
    SetAlwaysOnTop(bool), SetRunAtStartup(bool),
    SetUiScale(f32), SetClickThrough(bool), SetSnapWindows(bool),
    TrafficCycle,
    SetArrowSpacing(f32), SetArrowFontOffset(f32),
    SetDiskLabelSpacing(f32), SetDiskLabelFontOffset(f32),
        DiskLabelCycle,
    SkinPrev, SkinNext, SkinDice,
    SetSyncFonts(bool), SetRandomizeFonts(bool),
    SetUpdateMode(String),
    PresetSlotClick(u8),
    EditColor(u8),
    SetHotkey(String),
    SetRemoteEnabled(bool),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let settings = AppSettings::load().unwrap_or_default();
        let menu = Menu::new();
        let si = MenuItem::new("Settings", true, None);
        let wi = MenuItem::new("Show Widget", true, None);
        let gi = MenuItem::new("Game Mode", true, None);
        let ei = MenuItem::new("Exit", true, None);
        let (sid, wid, gid, eid) = (si.id().clone(), wi.id().clone(), gi.id().clone(), ei.id().clone());
        menu.append(&si).ok(); menu.append(&wi).ok(); menu.append(&gi).ok(); menu.append(&ei).ok();
        let tray = TrayIconBuilder::new().with_menu(Box::new(menu)).with_tooltip("fluidMonitor").with_icon(make_tray_icon()).build().expect("tray");
        let app = Self {
            settings, snapshot: SensorSnapshot::default(), poller: None,
            windows: BTreeMap::new(), warn_state: HashMap::new(),
            flash_on: false, editing_color: None, game_mode: false,
            pending_snap: None, ignore_next_move: false,
            _tray: tray, settings_id: sid, show_id: wid, game_id: gid, exit_id: eid,
        };
        let size = app.widget_size();
        let position = if app.settings.first_run_complete {
            window::Position::Specific(Point::new(app.settings.window_x as f32, app.settings.window_y as f32))
        } else { window::Position::Centered };
        let level = if app.settings.always_on_top { window::Level::AlwaysOnTop } else { window::Level::Normal };
        let (_id, open) = window::open(window::Settings {
            size, position, decorations: false, transparent: true, resizable: false, level, ..Default::default()
        });
        (app, open.map(|id| Message::WindowOpened(id, WindowKind::Widget)))
    }

    fn current_tiles(&self) -> Vec<String> {
        if self.game_mode { self.settings.game_mode_tiles.clone() }
        else { self.settings.tile_order.iter().filter(|t| self.settings.visible_tiles.contains(t)).cloned().collect() }
    }
    fn widget_size(&self) -> Size {
        let n = self.current_tiles().len().max(1) as f32;
        let sc = self.settings.ui_scale;
        let tw = self.settings.tile_width * sc;
        let th = self.settings.tile_height * sc;
        let orient = if self.game_mode { Orientation::Horizontal } else { self.settings.orientation.clone() };
        let sp = style::skin_style(&self.settings.active_skin).tile_spacing;
        match orient {
            Orientation::Horizontal => Size::new(16.0 + n * tw + (n - 1.0) * sp, 8.0 + 18.0 + 4.0 + th + 8.0),
            Orientation::Vertical => Size::new(tw + 16.0, 8.0 + 18.0 + 4.0 + n * th + (n - 1.0) * sp + 8.0),
        }
    }
    fn widget_window(&self) -> Option<window::Id> {
        self.windows.iter().find(|(_, k)| **k == WindowKind::Widget).map(|(id, _)| *id)
    }
    fn settings_window(&self) -> Option<window::Id> {
        self.windows.iter().find(|(_, k)| **k == WindowKind::Settings).map(|(id, _)| *id)
    }
    fn open_settings(&mut self) -> Task<Message> {
        if self.settings_window().is_some() { return Task::none(); }
        let pos = match (self.settings.settings_window_x, self.settings.settings_window_y) {
            (Some(x), Some(y)) => window::Position::Specific(Point::new(x as f32, y as f32)),
            _ => window::Position::Default,
        };
        let (_, t) = window::open(window::Settings {
            size: SETTINGS_SIZE, position: pos, decorations: false, transparent: true, resizable: false,
            level: window::Level::AlwaysOnTop, ..Default::default()
        });
        t.map(|id| Message::WindowOpened(id, WindowKind::Settings))
    }
    fn resize_widget(&self) -> Task<Message> {
        self.widget_window().map(|id| window::resize(id, self.widget_size())).unwrap_or(Task::none())
    }
    fn eval_warnings(&mut self) {
        self.warn_state.clear();
        for w in &self.settings.warnings {
            if !w.enabled { continue; }
            let (temp, load, used_gb): (Option<f32>, f32, f32) = match w.kind.as_str() {
                "CPU" => (self.snapshot.cpu.temperature_c, self.snapshot.cpu.usage_percent, 0.0),
                "GPU" => (self.snapshot.gpu.temperature_c, self.snapshot.gpu.usage_percent, 0.0),
                "RAM" => (None, self.snapshot.ram.usage_percent, self.snapshot.ram.used_mb / 1024.0),
                _ => continue,
            };
            let current: f64 = match w.metric {
                WarnMetric::Temperature => temp.unwrap_or(0.0) as f64,
                WarnMetric::Load => load as f64,
                WarnMetric::UsedGb => used_gb as f64,
                WarnMetric::Throughput => 0.0,
            };
            let exceeded = current >= w.threshold;
            let accent_override = if w.gradient_mode && w.metric == WarnMetric::Temperature {
                temp.and_then(|t| { let dist = w.threshold - t as f64; if dist <= 15.0 { Some(style::gradient_color(dist)) } else { None } })
            } else { None };
            self.warn_state.insert(w.kind.clone(), (exceeded && w.flash_enabled, accent_override));
        }
    }
    fn warn_view(&self, kind: &str) -> WarnView {
        match self.warn_state.get(kind) {
            Some(&(flash, ov)) => WarnView { flash: flash && self.flash_on, accent_override: ov },
            None => WarnView::default(),
        }
    }
    fn theme_name(&self) -> String {
        style::match_preset(&self.settings).map(|i| style::THEME_PRESETS[i].0.to_string()).unwrap_or("Custom".into())
    }
    fn disk_options(&self) -> Vec<String> {
        let mut v: Vec<String> = self.snapshot.disk.drives.iter().map(|d| d.mount.trim_end_matches('\\').to_string()).collect();
        v.sort(); v.dedup(); if v.is_empty() { v.push("C:".into()); } v
    }
    fn adapter_options(&self) -> Vec<String> {
        let mut v = vec!["All adapters".to_string()];
        let mut names: Vec<String> = self.snapshot.network.interfaces.iter().map(|i| i.name.clone()).collect();
        names.sort(); names.dedup(); v.extend(names); v
    }
    fn snap_position(&self, pos: Point) -> Option<Point> {
        let (l, t, r, b) = work_area()?;
        let sz = self.widget_size();
        let mut x = pos.x; let mut y = pos.y;
        if (x - l).abs() < SNAP_MARGIN { x = l; }
        if ((x + sz.width) - r).abs() < SNAP_MARGIN { x = r - sz.width; }
        if (y - t).abs() < SNAP_MARGIN { y = t; }
        if ((y + sz.height) - b).abs() < SNAP_MARGIN { y = b - sz.height; }
        if (x - pos.x).abs() > 0.5 || (y - pos.y).abs() > 0.5 { Some(Point::new(x, y)) } else { None }
    }
    fn game_corner(&self) -> Option<Point> {
        let (l, t, r, b) = work_area()?;
        let sz = self.widget_size();
        const M: f32 = 8.0;
        Some(match self.settings.game_mode_position {
            SnapPosition::TopLeft => Point::new(l + M, t + M),
            SnapPosition::TopRight => Point::new(r - sz.width - M, t + M),
            SnapPosition::BottomLeft => Point::new(l + M, b - sz.height - M),
            SnapPosition::BottomRight => Point::new(r - sz.width - M, b - sz.height - M),
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Noop => Task::none(),
            Message::SensorTick => {
                let poller = self.poller.get_or_insert_with(SensorPoller::new);
                self.snapshot = poller.poll(); self.eval_warnings(); Task::none()
            }
            Message::FlashTick => { self.flash_on = !self.flash_on; Task::none() }
            Message::TrayPoll => {
                let mut tasks: Vec<Task<Message>> = Vec::new();
                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == self.exit_id { return iced::exit(); }
                    if event.id == self.settings_id { tasks.push(self.open_settings()); }
                    if event.id == self.show_id { if let Some(id) = self.widget_window() { tasks.push(window::change_mode(id, window::Mode::Windowed)); } }
                    if event.id == self.game_id {
                        self.game_mode = !self.game_mode;
                        if let Some(id) = self.widget_window() {
                            tasks.push(window::resize(id, self.widget_size()));
                            if self.game_mode { if let Some(c) = self.game_corner() { self.ignore_next_move = true; tasks.push(window::move_to(id, c)); } }
                            else { self.ignore_next_move = true; tasks.push(window::move_to(id, Point::new(self.settings.window_x as f32, self.settings.window_y as f32))); }
                        }
                    }
                }
                if let Some((id, pos, when)) = self.pending_snap {
                    if when.elapsed() > Duration::from_millis(400) {
                        self.pending_snap = None;
                        if let Some(snapped) = self.snap_position(pos) {
                            self.ignore_next_move = true;
                            self.settings.window_x = snapped.x as f64; self.settings.window_y = snapped.y as f64;
                            let _ = self.settings.save(); tasks.push(window::move_to(id, snapped));
                        }
                    }
                }
                if tasks.is_empty() { Task::none() } else { Task::batch(tasks) }
            }
            Message::DragWindow(id) => window::drag(id),
            Message::WindowOpened(id, kind) => { self.windows.insert(id, kind); Task::none() }
            Message::WindowMoved(id, pos) => {
                match self.windows.get(&id) {
                    Some(&WindowKind::Widget) => {
                        if self.ignore_next_move { self.ignore_next_move = false; return Task::none(); }
                        if self.game_mode { return Task::none(); }
                        self.settings.window_x = pos.x as f64; self.settings.window_y = pos.y as f64;
                        self.settings.first_run_complete = true; let _ = self.settings.save();
                        if self.settings.snap_to_edges { self.pending_snap = Some((id, pos, Instant::now())); }
                    }
                    Some(&WindowKind::Settings) => {
                        self.settings.settings_window_x = Some(pos.x as f64);
                        self.settings.settings_window_y = Some(pos.y as f64); let _ = self.settings.save();
                    }
                    None => {}
                }
                Task::none()
            }
            Message::WindowClosed(id) => { self.windows.remove(&id); if self.widget_window().is_none() { return iced::exit(); } Task::none() }
            Message::OpenSettings => self.open_settings(),
            Message::HideWidget => self.widget_window().map(|id| window::change_mode(id, window::Mode::Hidden)).unwrap_or(Task::none()),
            Message::SaveClose => {
                let _ = self.settings.save();
                let close = self.settings_window().map(window::close).unwrap_or(Task::none());
                Task::batch([close, self.resize_widget()])
            }
            Message::ResetDefaults => {
                let keep = (self.settings.window_x, self.settings.window_y, self.settings.first_run_complete);
                self.settings = AppSettings::default();
                self.settings.window_x = keep.0; self.settings.window_y = keep.1; self.settings.first_run_complete = keep.2;
                self.resize_widget()
            }
            Message::ToggleTile(name, on) => {
                if on { if !self.settings.visible_tiles.contains(&name) { self.settings.visible_tiles.push(name.clone()); }
                    if !self.settings.tile_order.contains(&name) { self.settings.tile_order.push(name); }
                } else { self.settings.visible_tiles.retain(|t| t != &name); }
                self.resize_widget()
            }
            Message::SetOpacity(v) => { self.settings.widget_opacity = v; Task::none() }
            Message::SetOrientation(o) => { self.settings.orientation = o; self.resize_widget() }
            Message::SetAccent(hex) => { self.settings.theme_accent = hex; Task::none() }
            Message::SetFahrenheit(f) => { self.settings.temperature_unit = if f { TempUnit::Fahrenheit } else { TempUnit::Celsius }; Task::none() }
            Message::SetSnap(on) => { self.settings.snap_to_edges = on; Task::none() }
            Message::ThemePrev => {
                let n = style::THEME_PRESETS.len();
                let idx = style::match_preset(&self.settings).map(|i| (i + n - 1) % n).unwrap_or(n - 1);
                style::apply_preset(&mut self.settings, idx); Task::none()
            }
            Message::ThemeNext => {
                let n = style::THEME_PRESETS.len();
                let idx = style::match_preset(&self.settings).map(|i| (i + 1) % n).unwrap_or(0);
                style::apply_preset(&mut self.settings, idx); Task::none()
            }
            Message::ThemeDice => {
                let n = style::THEME_PRESETS.len();
                let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos() as usize).unwrap_or(0);
                let mut idx = nanos % n;
                if let Some(cur) = style::match_preset(&self.settings) { if idx == cur { idx = (idx + 1) % n; } }
                style::apply_preset(&mut self.settings, idx); Task::none()
            }
            Message::SetWarnEnabled(k, on) => { self.settings.warn_mut(&k).enabled = on; self.eval_warnings(); Task::none() }
            Message::SetWarnThreshold(k, v) => { self.settings.warn_mut(&k).threshold = v as f64; self.eval_warnings(); Task::none() }
            Message::SetWarnFlash(k, on) => { self.settings.warn_mut(&k).flash_enabled = on; self.eval_warnings(); Task::none() }
            Message::SetWarnGradient(k, on) => { self.settings.warn_mut(&k).gradient_mode = on; self.eval_warnings(); Task::none() }
            Message::EditColor(slot) => {
                self.editing_color = if self.editing_color == Some(slot) { None } else { Some(slot) };
                Task::none()
            }
            Message::SetHexColor(slot, v) => {
                match slot { 0 => self.settings.theme_bg = v, 1 => self.settings.theme_tile = v, 2 => self.settings.theme_accent = v, 3 => self.settings.theme_text = v, _ => self.settings.theme_muted = v }
                Task::none()
            }
            Message::SetTileWidth(v) => { self.settings.tile_width = v; self.resize_widget() }
            Message::SetTileHeight(v) => { self.settings.tile_height = v; self.resize_widget() }
            Message::SetPrimaryFontOffset(v) => { self.settings.primary_font_offset = v as i32; Task::none() }
            Message::SetSecondaryFontOffset(v) => { self.settings.secondary_font_offset = v as i32; Task::none() }
            Message::SetIndicatorFontOffset(v) => { self.settings.indicator_font_offset = v as i32; Task::none() }
            Message::SetMutedContrast(v) => { self.settings.muted_contrast = v; Task::none() }
            Message::SetInterval(v) => { self.settings.update_interval_ms = v as u64; Task::none() }
            Message::SetCpuName(v) => { self.settings.cpu_custom_name = v; Task::none() }
            Message::SetGpuName(v) => { self.settings.gpu_custom_name = v; Task::none() }
            Message::SetDisk(v) => { self.settings.selected_disk_mount = v; Task::none() }
            Message::SetAdapter(v) => { self.settings.network_adapter_name = if v == "All adapters" { String::new() } else { v }; Task::none() }
            Message::SetAlwaysOnTop(on) => {
                self.settings.always_on_top = on;
                self.widget_window().map(|id| window::change_level(id, if on { window::Level::AlwaysOnTop } else { window::Level::Normal })).unwrap_or(Task::none())
            }
            Message::SetRunAtStartup(on) => { self.settings.run_at_startup = on; set_run_at_startup(on); Task::none() }
            Message::SetUiScale(v) => { self.settings.ui_scale = v; self.resize_widget() }
            Message::SetClickThrough(on) => { self.settings.click_through = on; Task::none() }
            Message::SetSnapWindows(on) => { self.settings.snap_to_windows = on; Task::none() }
            Message::TrafficCycle => {
                let modes = ["Off", "Blink", "Fade", "Glow"];
                let cur = modes.iter().position(|m| *m == self.settings.network_traffic_indicator).unwrap_or(0);
                self.settings.network_traffic_indicator = modes[(cur + 1) % modes.len()].to_string();
                Task::none()
            }
            Message::SetArrowSpacing(v) => { self.settings.network_arrow_spacing = v; Task::none() }
            Message::SetArrowFontOffset(v) => { self.settings.arrow_font_offset = v as i32; Task::none() }
            Message::SetDiskLabelSpacing(v) => { self.settings.disk_label_spacing = v; Task::none() }
            Message::SetDiskLabelFontOffset(v) => { self.settings.disk_label_font_offset = v as i32; Task::none() }
            Message::SetHotkey(v) => { self.settings.click_through_hotkey = v; Task::none() }
                        Message::SetRemoteEnabled(on) => { self.settings.remote_enabled = on; Task::none() }
            Message::SkinPrev => {
                let skins = style::SKIN_NAMES;
                let cur = skins.iter().position(|s| *s == self.settings.active_skin).unwrap_or(0);
                self.settings.active_skin = skins[(cur + skins.len() - 1) % skins.len()].to_string();
                self.resize_widget()
            }
            Message::SkinNext => {
                let skins = style::SKIN_NAMES;
                let cur = skins.iter().position(|s| *s == self.settings.active_skin).unwrap_or(0);
                self.settings.active_skin = skins[(cur + 1) % skins.len()].to_string();
                self.resize_widget()
            }
            Message::SkinDice => {
                let skins = style::SKIN_NAMES;
                let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos() as usize).unwrap_or(0);
                let mut idx = nanos % skins.len();
                if let Some(cur) = skins.iter().position(|s| *s == self.settings.active_skin) { if idx == cur { idx = (idx + 1) % skins.len(); } }
                self.settings.active_skin = skins[idx].to_string();
                self.resize_widget()
            }
            Message::SetSyncFonts(on) => { self.settings.sync_fonts = on; Task::none() }
            Message::SetRandomizeFonts(on) => { self.settings.randomize_fonts_on_dice = on; Task::none() }
            Message::SetUpdateMode(mode) => {
                self.settings.update_check_mode = match mode.as_str() {
                    "Auto" => fluid_core::settings::UpdateMode::Auto,
                    "Off" => fluid_core::settings::UpdateMode::Off,
                    _ => fluid_core::settings::UpdateMode::Manual,
                };
                Task::none()
            }
            Message::PresetSlotClick(slot) => {
                let idx = slot as usize;
                if idx < self.settings.presets.len() {
                    // Load existing preset
                    let p = self.settings.presets[idx].clone();
                    self.settings.theme_bg = p.bg;
                    self.settings.theme_tile = p.tile;
                    self.settings.theme_accent = p.accent;
                    self.settings.theme_text = p.text;
                    self.settings.theme_muted = p.muted;
                    self.settings.active_skin = p.skin;
                } else {
                    // Save current to new slot
                    while self.settings.presets.len() <= idx {
                        self.settings.presets.push(fluid_core::settings::PresetSlot {
                            name: format!("Slot {}", self.settings.presets.len() + 1),
                            bg: self.settings.theme_bg.clone(),
                            tile: self.settings.theme_tile.clone(),
                            accent: self.settings.theme_accent.clone(),
                            text: self.settings.theme_text.clone(),
                            muted: self.settings.theme_muted.clone(),
                            skin: self.settings.active_skin.clone(),
                        });
                    }
                    let p = &mut self.settings.presets[idx];
                    p.bg = self.settings.theme_bg.clone();
                    p.tile = self.settings.theme_tile.clone();
                    p.accent = self.settings.theme_accent.clone();
                    p.text = self.settings.theme_text.clone();
                    p.muted = self.settings.theme_muted.clone();
                    p.skin = self.settings.active_skin.clone();
                }
                Task::none()
            }
            Message::DiskLabelCycle => {
                let styles = ["Letter", "Model", "Both", "None"];
                let cur = styles.iter().position(|s| *s == self.settings.disk_label_style).unwrap_or(0);
                self.settings.disk_label_style = styles[(cur + 1) % styles.len()].to_string();
                Task::none()
            }
        }
    }

    fn view(&self, id: window::Id) -> Element<'_, Message> {
        let opacity = if self.game_mode { self.settings.game_mode_opacity } else { self.settings.widget_opacity };
        let p = Palette::from_settings(&self.settings, opacity);
        match self.windows.get(&id) {
            Some(WindowKind::Settings) => settings_panel::view(&self.settings, p, id, self.theme_name(), self.disk_options(), self.adapter_options(), self.editing_color),
            _ => self.widget_view(id, p),
        }
    }

    fn widget_view(&self, id: window::Id, p: Palette) -> Element<'_, Message> {
        let mut tiles: Vec<Element<'_, Message>> = Vec::new();
        for name in self.current_tiles() {
            let w = self.warn_view(&name);
            let el = match name.as_str() {
                "CPU" => tile::cpu_tile(&self.snapshot.cpu, &self.settings, p, w),
                "GPU" => tile::gpu_tile(&self.snapshot.gpu, &self.settings, p, w),
                "RAM" => tile::ram_tile(&self.snapshot.ram, &self.settings, p, w),
                "Disk" => tile::disk_tile(&self.snapshot.disk, &self.settings, p, w),
                "Network" => tile::network_tile(&self.snapshot.network, &self.settings, p, w),
                "Clock" => tile::clock_tile(&self.settings, p, w),
                _ => continue,
            };
            tiles.push(el);
        }
        let orient = if self.game_mode { Orientation::Horizontal } else { self.settings.orientation.clone() };
        let body: Element<'_, Message> = match orient {
            Orientation::Vertical => {
                let skin = style::skin_style(&self.settings.active_skin);
                column(tiles).spacing(skin.tile_spacing).into()
            }
            Orientation::Horizontal => {
                let skin = style::skin_style(&self.settings.active_skin);
                row(tiles).spacing(skin.tile_spacing).into()
            }
        };
        let icon_btn = |label: &str, msg: Message| {
            button(text(label.to_string()).size(11).font(iced::Font::with_name("Segoe UI Symbol"))
                .style(move |_| iced::widget::text::Style { color: Some(p.muted) })
            ).padding(0).style(|_, _| button::Style { background: None, ..Default::default() }).on_press(msg)
        };
        let header = row![icon_btn("\u{2699}", Message::OpenSettings), Space::with_width(Length::Fill), icon_btn("\u{2715}", Message::HideWidget)].height(18);
        let skin = style::skin_style(&self.settings.active_skin);
        let widget_border = skin.border_color(&p);
        let root = container(column![header, Space::with_height(4), body])
            .width(Length::Fill).height(Length::Fill).padding(8)
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(p.bg)),
                border: Border { radius: skin.widget_radius.into(), width: skin.widget_border, color: widget_border }, ..Default::default()
            });
        mouse_area(root).on_press(Message::DragWindow(id)).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(self.settings.update_interval_ms.max(250))).map(|_| Message::SensorTick),
            iced::time::every(Duration::from_millis(200)).map(|_| Message::TrayPoll),
            iced::time::every(Duration::from_millis(600)).map(|_| Message::FlashTick),
            window::close_events().map(Message::WindowClosed),
            window::events().map(|(id, event)| match event {
                window::Event::Moved(pos) => Message::WindowMoved(id, pos),
                _ => Message::TrayPoll,
            }),
        ])
    }
    fn theme(&self, _id: window::Id) -> Theme { Theme::Dark }
}




