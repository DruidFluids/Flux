use fluid_core::settings::AppSettings;
use iced::Color;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub tile: Color,
    pub accent: Color,
    pub text: Color,
    pub muted: Color,
    pub bar_bg: Color,
}

fn parse_hex(s: &str, fallback: Color) -> Color {
    let h = s.trim_start_matches('#');
    let (a, rgb) = match h.len() {
        8 => (
            u8::from_str_radix(&h[0..2], 16).unwrap_or(255),
            &h[2..],
        ),
        6 => (255, h),
        _ => return fallback,
    };
    let r = u8::from_str_radix(&rgb[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&rgb[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&rgb[4..6], 16).unwrap_or(0);
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}

impl Palette {
    pub fn from_settings(s: &AppSettings) -> Self {
        let bg = parse_hex(&s.theme_bg, Color::from_rgb(0.102, 0.102, 0.118));
        let tile = parse_hex(&s.theme_tile, Color::from_rgb(0.141, 0.141, 0.157));
        let accent = parse_hex(&s.theme_accent, Color::from_rgb(0.227, 0.561, 0.831));
        let text = parse_hex(&s.theme_text, Color::from_rgb(0.910, 0.910, 0.910));
        let muted = parse_hex(&s.theme_muted, Color::from_rgb(0.533, 0.533, 0.533));
        let op = s.widget_opacity.clamp(0.2, 1.0);
        Self {
            bg: Color { a: bg.a * op, ..bg },
            tile: Color { a: tile.a * op, ..tile },
            accent,
            text,
            muted,
            bar_bg: Color { a: 0.35 * op, ..muted },
        }
    }
}

pub fn swatch_color(hex: &str) -> Color {
    parse_hex(hex, Color::from_rgb(0.3, 0.3, 0.3))
}
