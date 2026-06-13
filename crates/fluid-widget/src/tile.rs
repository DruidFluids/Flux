use fluid_core::sensor_data::*;
use fluid_core::settings::AppSettings;
use iced::widget::{column, container, row, text, Space};
use iced::{Border, Element, Length};
use crate::fmt;
use crate::style::Palette;
use crate::Message;

pub const TILE_W: f32 = 130.0;
pub const TILE_H: f32 = 110.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct WarnView {
    pub flash: bool,
    pub accent_override: Option<iced::Color>,
}

fn sub_header<'a>(label: String, p: Palette) -> Element<'a, Message> {
    text(label).size(11)
        .style(move |_| iced::widget::text::Style { color: Some(p.muted) })
        .into()
}

fn big<'a>(s: String, p: Palette) -> Element<'a, Message> {
    text(s).size(18)
        .style(move |_| iced::widget::text::Style { color: Some(p.text) })
        .into()
}

fn unit<'a>(s: String, accent: iced::Color) -> Element<'a, Message> {
    text(s).size(12)
        .style(move |_| iced::widget::text::Style { color: Some(accent) })
        .into()
}

fn small<'a>(s: String, p: Palette) -> Element<'a, Message> {
    text(s).size(11)
        .style(move |_| iced::widget::text::Style { color: Some(p.muted) })
        .into()
}

fn small_unit<'a>(s: String, accent: iced::Color) -> Element<'a, Message> {
    text(s).size(9)
        .style(move |_| iced::widget::text::Style { color: Some(accent) })
        .into()
}

fn line_label<'a>(s: String, p: Palette) -> Element<'a, Message> {
    text(s).size(14)
        .style(move |_| iced::widget::text::Style { color: Some(p.muted) })
        .into()
}

fn line_value<'a>(v: String, u: String, p: Palette, accent: iced::Color) -> Element<'a, Message> {
    row![
        text(v).size(14)
            .style(move |_| iced::widget::text::Style { color: Some(p.text) }),
        Space::with_width(3),
        text(u).size(9)
            .style(move |_| iced::widget::text::Style { color: Some(accent) }),
    ]
    .align_y(iced::Alignment::End)
    .into()
}

pub fn cpu_tile<'a>(cpu: &CpuData, settings: &AppSettings, p: Palette, w: WarnView) -> Element<'a, Message> {
    let accent = w.accent_override.unwrap_or(p.accent);
    let name = fmt::shorten(&cpu.name);
    let name = if name.is_empty() { "CPU".to_string() } else { name };

    // C#: "{temp}  {load}%" or "{load}%" when temp missing
    let mut primary = row![].spacing(4).align_y(iced::Alignment::End);
    if let Some((tv, tu)) = fmt::fmt_temp(cpu.temperature_c, settings) {
        primary = primary.push(big(tv, p)).push(unit(tu, accent)).push(Space::with_width(6));
    }
    primary = primary
        .push(big(format!("{:.0}", cpu.usage_percent), p))
        .push(unit("%".into(), accent));

    let secondary: Element<'a, Message> = match cpu.clock_mhz {
        Some(m) => row![
            small(format!("{:.0}", m), p),
            Space::with_width(3),
            small_unit("MHz".into(), accent),
        ].align_y(iced::Alignment::End).into(),
        None => Space::with_height(0).into(),
    };

    tile_container(column![
        sub_header(name, p),
        Space::with_height(Length::Fill),
        container(primary).width(Length::Fill).center_x(Length::Fill),
        Space::with_height(Length::Fill),
        container(secondary).width(Length::Fill).center_x(Length::Fill),
    ], p, w)
}

pub fn gpu_tile<'a>(gpu: &GpuData, settings: &AppSettings, p: Palette, w: WarnView) -> Element<'a, Message> {
    let accent = w.accent_override.unwrap_or(p.accent);
    let name = fmt::shorten(&gpu.name);
    let name = if name.is_empty() { "GPU".to_string() } else { name };

    let mut primary = row![].spacing(4).align_y(iced::Alignment::End);
    if let Some((tv, tu)) = fmt::fmt_temp(gpu.temperature_c, settings) {
        primary = primary.push(big(tv, p)).push(unit(tu, accent)).push(Space::with_width(6));
    }
    primary = primary
        .push(big(format!("{:.0}", gpu.usage_percent), p))
        .push(unit("%".into(), accent));

    // C#: "{MHz}\n{vu:0.0}/{vt:0.0} GB"
    let mut sec = column![].spacing(1).align_x(iced::Alignment::Center);
    if let Some(m) = gpu.clock_mhz {
        sec = sec.push(
            row![
                small(format!("{:.0}", m), p),
                Space::with_width(3),
                small_unit("MHz".into(), accent),
            ].align_y(iced::Alignment::End)
        );
    }
    if gpu.vram_used_mb > 0.0 && gpu.vram_total_mb > 0.0 {
        sec = sec.push(
            row![
                small(format!("{:.1}/{:.1}", gpu.vram_used_mb / 1024.0, gpu.vram_total_mb / 1024.0), p),
                Space::with_width(3),
                small_unit("GB".into(), accent),
            ].align_y(iced::Alignment::End)
        );
    }

    tile_container(column![
        sub_header(name, p),
        Space::with_height(Length::Fill),
        container(primary).width(Length::Fill).center_x(Length::Fill),
        Space::with_height(Length::Fill),
        container(sec).width(Length::Fill).center_x(Length::Fill),
    ], p, w)
}

pub fn ram_tile<'a>(ram: &RamData, _settings: &AppSettings, p: Palette, w: WarnView) -> Element<'a, Message> {
    let accent = w.accent_override.unwrap_or(p.accent);
    let used_gb = ram.used_mb / 1024.0;
    let total_gb = ram.total_mb / 1024.0;

    // C#: big "17.4 GB", secondary "27% of 64.0 GB"
    let primary = row![
        big(format!("{:.1}", used_gb), p),
        Space::with_width(4),
        unit("GB".into(), accent),
    ].align_y(iced::Alignment::End);

    let secondary = row![
        small(format!("{:.0}", ram.usage_percent), p),
        small_unit("%".into(), accent),
        small(format!(" of {:.1}", total_gb), p),
        Space::with_width(3),
        small_unit("GB".into(), accent),
    ].align_y(iced::Alignment::End);

    tile_container(column![
        sub_header("RAM".to_string(), p),
        Space::with_height(Length::Fill),
        container(primary).width(Length::Fill).center_x(Length::Fill),
        Space::with_height(Length::Fill),
        container(secondary).width(Length::Fill).center_x(Length::Fill),
    ], p, w)
}

pub fn disk_tile<'a>(disk: &DiskData, _settings: &AppSettings, p: Palette, w: WarnView) -> Element<'a, Message> {
    let accent = w.accent_override.unwrap_or(p.accent);
    let primary = disk.drives.first();
    let (read, write) = primary
        .map(|d| (d.read_bytes_sec as f64, d.write_bytes_sec as f64))
        .unwrap_or((0.0, 0.0));
    let mount = primary.map(|d| d.mount.trim_end_matches('\\').to_string()).unwrap_or_default();
    let (rv, ru) = fmt::fmt_disk(read);
    let (wv, wu) = fmt::fmt_disk(write);

    let lines = column![
        row![
            line_label("R:".into(), p),
            Space::with_width(6),
            line_value(rv, ru, p, accent),
        ].align_y(iced::Alignment::Center),
        row![
            line_label("W:".into(), p),
            Space::with_width(5),
            line_value(wv, wu, p, accent),
        ].align_y(iced::Alignment::Center),
    ]
    .spacing(4)
    .align_x(iced::Alignment::Start);

    tile_container(column![
        sub_header(mount, p),
        Space::with_height(Length::Fill),
        container(lines).width(Length::Fill).center_x(Length::Fill),
        Space::with_height(Length::Fill),
    ], p, w)
}

pub fn network_tile<'a>(net: &NetworkData, _settings: &AppSettings, p: Palette, w: WarnView) -> Element<'a, Message> {
    let accent = w.accent_override.unwrap_or(p.accent);
    let down: u64 = net.interfaces.iter().map(|i| i.download_bytes_sec).sum();
    let up: u64 = net.interfaces.iter().map(|i| i.upload_bytes_sec).sum();
    let (dv, du) = fmt::fmt_net(down as f64);
    let (uv, uu) = fmt::fmt_net(up as f64);

    let down_color = if down > 0 { accent } else { p.muted };
    let up_color = if up > 0 { accent } else { p.muted };

    let lines = column![
        row![
            text("\u{2193}".to_string()).size(15)
                .style(move |_| iced::widget::text::Style { color: Some(down_color) }),
            Space::with_width(8),
            line_value(dv, du, p, accent),
        ].align_y(iced::Alignment::Center),
        row![
            text("\u{2191}".to_string()).size(15)
                .style(move |_| iced::widget::text::Style { color: Some(up_color) }),
            Space::with_width(8),
            line_value(uv, uu, p, accent),
        ].align_y(iced::Alignment::Center),
    ]
    .spacing(4)
    .align_x(iced::Alignment::Start);

    tile_container(column![
        sub_header("All adapters".to_string(), p),
        Space::with_height(Length::Fill),
        container(lines).width(Length::Fill).center_x(Length::Fill),
        Space::with_height(Length::Fill),
    ], p, w)
}

pub fn clock_tile<'a>(_settings: &AppSettings, p: Palette, w: WarnView) -> Element<'a, Message> {
    let accent = w.accent_override.unwrap_or(p.accent);
    let now = chrono::Local::now();
    let time = now.format("%-I:%M").to_string();
    let ampm = now.format("%p").to_string().to_lowercase();
    let day = now.format("%-d").to_string();
    let day_n: u32 = day.parse().unwrap_or(1);
    let suffix = match day_n % 100 {
        11..=13 => "th",
        _ => match day_n % 10 { 1 => "st", 2 => "nd", 3 => "rd", _ => "th" },
    };
    let weekday = now.format("%A").to_string();
    let month = now.format("%B").to_string();

    let primary = row![
        big(time, p),
        Space::with_width(4),
        unit(ampm, accent),
    ].align_y(iced::Alignment::End);

    let secondary = column![
        small(format!("{},", weekday), p),
        row![
            small(format!("{} ", month), p),
            text(day).size(11)
                .style(move |_| iced::widget::text::Style { color: Some(accent) }),
            small(suffix.to_string(), p),
        ],
    ].align_x(iced::Alignment::Center);

    tile_container(column![
        Space::with_height(Length::Fill),
        container(primary).width(Length::Fill).center_x(Length::Fill),
        Space::with_height(4),
        container(secondary).width(Length::Fill).center_x(Length::Fill),
        Space::with_height(Length::Fill),
    ], p, w)
}

fn tile_container<'a>(content: impl Into<Element<'a, Message>>, p: Palette, w: WarnView) -> Element<'a, Message> {
    let bg = if w.flash {
        iced::Color::from_rgb(1.0, 0.2, 0.2)
    } else {
        p.tile
    };
    container(content)
        .width(Length::Fixed(TILE_W))
        .height(Length::Fixed(TILE_H))
        .padding([8, 10])
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { radius: 8.0.into(), ..Border::default() },
            ..Default::default()
        })
        .into()
}
