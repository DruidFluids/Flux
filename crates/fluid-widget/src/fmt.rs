//! Formatting helpers for sensor values (byte-rate humanizing, temperature
//! units, model-name shortening).

use fluid_core::settings::{AppSettings, TempUnit};

/// C# Shorten(): strip vendor prefixes, (R)/(TM), "<N>-Core", trailing Processor/CPU/Graphics
pub fn shorten(name: &str) -> String {
    if name.trim().is_empty() {
        return String::new();
    }
    let mut n = name.trim().to_string();
    for p in ["AMD ", "NVIDIA ", "Intel(R) ", "Intel "] {
        if n.to_lowercase().starts_with(&p.to_lowercase()) {
            n = n[p.len()..].to_string();
            break;
        }
    }
    n = n.replace("(R)", "").replace("(TM)", "").replace("(tm)", "");
    // strip " <N>-Core" token
    let words: Vec<&str> = n.split_whitespace().collect();
    let filtered: Vec<&str> = words.into_iter()
        .filter(|w| {
            let lower = w.to_lowercase();
            !(lower.ends_with("-core") && lower.trim_end_matches("-core").parse::<u32>().is_ok())
        })
        .collect();
    n = filtered.join(" ");
    for s in [" Processor", " CPU", " Graphics"] {
        if n.to_lowercase().ends_with(&s.to_lowercase()) {
            n = n[..n.len() - s.len()].to_string();
            break;
        }
    }
    n.trim().to_string()
}

/// (value, unit) pairs so tiles can render the unit in accent color.
pub fn fmt_net(bps: f64) -> (String, String) {
    if bps < 1024.0 {
        (format!("{:.0}", bps), "B/s".into())
    } else if bps < 1024.0 * 1024.0 {
        (format!("{:.1}", bps / 1024.0), "KB/s".into())
    } else if bps < 1024.0 * 1024.0 * 1024.0 {
        (format!("{:.1}", bps / 1024.0 / 1024.0), "MB/s".into())
    } else {
        (format!("{:.1}", bps / 1024.0 / 1024.0 / 1024.0), "GB/s".into())
    }
}

pub fn fmt_disk(bps: f64) -> (String, String) {
    if bps < 1024.0 {
        (format!("{:.0}", bps), "B/s".into())
    } else if bps < 1024.0 * 1024.0 {
        (format!("{:.0}", bps / 1024.0), "KB/s".into())
    } else if bps < 1024.0 * 1024.0 * 1024.0 {
        (format!("{:.1}", bps / 1024.0 / 1024.0), "MB/s".into())
    } else {
        (format!("{:.1}", bps / 1024.0 / 1024.0 / 1024.0), "GB/s".into())
    }
}

/// C# Temp(): em-dash when missing/<=0; (value, unit) otherwise
pub fn fmt_temp(temp_c: Option<f32>, settings: &AppSettings) -> Option<(String, String)> {
    let t = temp_c?;
    if t <= 0.0 {
        return None;
    }
    if settings.temperature_unit == TempUnit::Fahrenheit {
        Some((format!("{:.0}", t * 9.0 / 5.0 + 32.0), "\u{00B0}F".into()))
    } else {
        Some((format!("{:.0}", t), "\u{00B0}C".into()))
    }
}
