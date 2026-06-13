use crate::color::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePalette {
    pub bg: Color,
    pub tile: Color,
    pub accent: Color,
    pub text: Color,
    pub muted: Color,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self {
            bg: Color::from_hex("#1A1A1E").unwrap(),
            tile: Color::from_hex("#242428").unwrap(),
            accent: Color::from_hex("#3A8FD4").unwrap(),
            text: Color::from_hex("#E8E8E8").unwrap(),
            muted: Color::from_hex("#888888").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub palette: ThemePalette,
    pub skin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePack {
    pub franchise: String,
    pub version: u32,
    pub themes: Vec<PackTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackTheme {
    pub name: String,
    pub bg: String,
    pub tile: String,
    pub accent: String,
    pub text: String,
    pub muted: String,
    pub category: String,
}

pub struct BuiltInThemes;

impl BuiltInThemes {
    pub fn all() -> Vec<Theme> {
        vec![
            Self::theme("Default", "#1A1A1E", "#242428", "#3A8FD4", "#E8E8E8", "#888888", "Default"),
            Self::theme("Evergreen", "#0A120C", "#142018", "#58B848", "#D8E4D0", "#688860", "Aurora"),
            Self::theme("Sandstone", "#0E0A08", "#1E1610", "#C8A050", "#E0D8C8", "#806840", "Retro"),
            Self::theme("Deep Current", "#080C10", "#101820", "#4888C8", "#D0D8E4", "#586880", "Frosted"),
            Self::theme("Morning Dew", "#0A0E08", "#141E10", "#78B858", "#D8E4C8", "#688850", "Paper"),
            Self::theme("Hearthwood", "#100A08", "#201410", "#C87848", "#E4D8C8", "#886050", "Brutalist"),
            Self::theme("Terracotta", "#100808", "#1E1010", "#C86848", "#E4D0C8", "#886050", "Sharp"),
            Self::theme("Tidestone", "#080A0E", "#10141E", "#5888B8", "#D0D8E0", "#586878", "Holographic"),
            Self::theme("Forest Gold", "#0C0A06", "#1A1810", "#A8A048", "#E0DCC8", "#787440", "Ink"),
            Self::theme("Inlet", "#08080C", "#10101A", "#5868A8", "#D0D0E4", "#585870", "Aurora"),
            Self::theme("Canopy", "#080E08", "#101E10", "#48A848", "#C8E4C8", "#588858", "Frosted"),
            Self::theme("Sage", "#0A0C08", "#161A10", "#88A858", "#D8DCC8", "#687850", "Paper"),
            Self::theme("Clay Coast", "#0E0A08", "#1C1610", "#B88848", "#E0D4C0", "#887048", "Brutalist"),
            Self::theme("Dusk Harbor", "#0A0810", "#161020", "#8868B8", "#D8D0E4", "#706080", "Sharp"),
            Self::theme("Fern", "#080C06", "#101A0E", "#58A848", "#C8E0C8", "#588848", "Holographic"),
            Self::theme("Driftwood", "#0C0A08", "#1A1610", "#A89868", "#DCD8C8", "#787058", "Ink"),
            Self::theme("Glacier", "#0C0E10", "#1A1E22", "#78A8C0", "#D8DCE4", "#687880", "Frosted"),
            Self::theme("Amber Trail", "#0E0A08", "#1E1610", "#C8A050", "#E0D8C8", "#806840", "Retro"),
        ]
    }

    fn theme(name: &str, bg: &str, tile: &str, accent: &str, text: &str, muted: &str, skin: &str) -> Theme {
        Theme {
            name: name.into(),
            palette: ThemePalette {
                bg: Color::from_hex(bg).unwrap_or_default(),
                tile: Color::from_hex(tile).unwrap_or_default(),
                accent: Color::from_hex(accent).unwrap_or_default(),
                text: Color::from_hex(text).unwrap_or_default(),
                muted: Color::from_hex(muted).unwrap_or_default(),
            },
            skin: skin.into(),
        }
    }
}
