//! Installer visual style — mirrors Fluxid's built-in **"Dark (default)"**
//! theme so setup looks like the app it installs. Colors are copied from
//! `THEME_PRESETS[0]` in `fluid-widget/src/style.rs`:
//! bg `#1E1E22`, tile `#2A2A30`, accent `#00A8FF`, text `#E8E8EC`,
//! muted `#9A9AA8`.

use iced::widget::{button, container, text};
use iced::{Border, Color, Theme};

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

pub const BG: Color = rgb(0x1E, 0x1E, 0x22);
pub const TILE: Color = rgb(0x2A, 0x2A, 0x30);
pub const ACCENT: Color = rgb(0x00, 0xA8, 0xFF);
pub const TEXT: Color = rgb(0xE8, 0xE8, 0xEC);
pub const MUTED: Color = rgb(0x9A, 0x9A, 0xA8);
pub const DANGER: Color = rgb(0xFF, 0x6B, 0x6B);

/// The app theme: a custom dark palette built from the widget's defaults so
/// iced's stock widget styling (buttons, radios, checkboxes) already lands on
/// the Fluxid accent without per-widget overrides.
pub fn theme() -> Theme {
    Theme::custom(
        "Fluxid Dark".to_string(),
        iced::theme::Palette {
            background: BG,
            text: TEXT,
            primary: ACCENT,
            success: ACCENT,
            danger: DANGER,
        },
    )
}

/// Root window fill — the deep background with default text color.
pub fn root(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(BG.into()),
        text_color: Some(TEXT),
        ..container::Style::default()
    }
}

/// A raised panel (the options group), using the tile color + rounded corners.
pub fn card(_t: &Theme) -> container::Style {
    container::Style {
        background: Some(TILE.into()),
        border: Border {
            radius: 10.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub fn title(_t: &Theme) -> text::Style {
    text::Style { color: Some(ACCENT) }
}

pub fn body(_t: &Theme) -> text::Style {
    text::Style { color: Some(TEXT) }
}

pub fn muted(_t: &Theme) -> text::Style {
    text::Style { color: Some(MUTED) }
}

pub fn danger(_t: &Theme) -> text::Style {
    text::Style { color: Some(DANGER) }
}

/// Primary action button (Next / Install / Close) — accent fill, dark text.
pub fn primary(_t: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => rgb(0x33, 0xB8, 0xFF),
        button::Status::Disabled => Color { a: 0.35, ..ACCENT },
        button::Status::Active => ACCENT,
    };
    button::Style {
        background: Some(bg.into()),
        text_color: rgb(0x0C, 0x0C, 0x10),
        border: Border {
            radius: 7.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

/// Secondary button (Back) — subtle tile fill with light text.
pub fn secondary(_t: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => rgb(0x3A, 0x3A, 0x42),
        _ => TILE,
    };
    button::Style {
        background: Some(bg.into()),
        text_color: TEXT,
        border: Border {
            radius: 7.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}
