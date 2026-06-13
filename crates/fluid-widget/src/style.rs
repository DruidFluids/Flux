use iced::Color;

pub struct FluidTheme;

impl FluidTheme {
    pub const BG: Color = Color::from_rgb(0.071, 0.075, 0.106);
    pub const TILE: Color = Color::from_rgb(0.118, 0.125, 0.173);
    pub const ACCENT: Color = Color::from_rgb(0.302, 0.600, 1.000);
    pub const TEXT: Color = Color::from_rgb(0.910, 0.918, 0.937);
    pub const MUTED: Color = Color::from_rgb(0.560, 0.580, 0.640);
    pub const BAR_BG: Color = Color::from_rgb(0.180, 0.190, 0.250);
}
