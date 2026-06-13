use fluid_core::sensor_data::*;
use fluid_core::settings::{AppSettings, TempUnit};
use iced::widget::{column, container, progress_bar, row, text, Space};
use iced::{Border, Element, Length};
use crate::style::FluidTheme;
use crate::Message;

fn fmt_temp(temp: Option<f32>, settings: &AppSettings) -> String {
    temp.map(|c| {
        if settings.temperature_unit == TempUnit::Fahrenheit {
            format!("{:.0}\u{00B0}F", c * 9.0 / 5.0 + 32.0)
        } else {
            format!("{:.0}\u{00B0}C", c)
        }
    }).unwrap_or_default()
}

fn header<'a>(label: String) -> Element<'a, Message> {
    text(label)
        .size(10)
        .style(|_| iced::widget::text::Style { color: Some(FluidTheme::MUTED) })
        .into()
}

fn value_text<'a>(s: String, size: u16) -> Element<'a, Message> {
    text(s)
        .size(size)
        .style(|_| iced::widget::text::Style { color: Some(FluidTheme::TEXT) })
        .into()
}

fn muted_text<'a>(s: String, size: u16) -> Element<'a, Message> {
    text(s)
        .size(size)
        .style(|_| iced::widget::text::Style { color: Some(FluidTheme::MUTED) })
        .into()
}

fn usage_bar<'a>(percent: f32) -> Element<'a, Message> {
    progress_bar(0.0..=100.0, percent)
        .height(3)
        .style(|_| iced::widget::progress_bar::Style {
            background: iced::Background::Color(FluidTheme::BAR_BG),
            bar: iced::Background::Color(FluidTheme::ACCENT),
            border: Border::default(),
        })
        .into()
}

pub fn cpu_tile<'a>(cpu: &CpuData, settings: &AppSettings) -> Element<'a, Message> {
    let name = if cpu.name.is_empty() { "CPU".to_string() } else { cpu.name.clone() };
    let temp = fmt_temp(cpu.temperature_c, settings);
    let clock = cpu.clock_mhz
        .map(|m| format!("{:.1} GHz", m / 1000.0))
        .unwrap_or_default();

    let content = column![
        header(name),
        row![
            value_text(format!("{:.0}%", cpu.usage_percent), 16),
            Space::with_width(Length::Fill),
            muted_text(format!("{} {}", clock, temp).trim().to_string(), 10),
        ].align_y(iced::Alignment::Center),
        usage_bar(cpu.usage_percent),
    ]
    .spacing(3);

    tile_container(content)
}

pub fn gpu_tile<'a>(gpu: &GpuData, settings: &AppSettings) -> Element<'a, Message> {
    let name = if gpu.name.is_empty() { "GPU".to_string() } else { gpu.name.clone() };
    let temp = fmt_temp(gpu.temperature_c, settings);

    let content = column![
        header(name),
        row![
            value_text(format!("{:.0}%", gpu.usage_percent), 16),
            Space::with_width(Length::Fill),
            muted_text(temp, 10),
        ].align_y(iced::Alignment::Center),
        usage_bar(gpu.usage_percent),
    ]
    .spacing(3);

    tile_container(content)
}

pub fn ram_tile<'a>(ram: &RamData, _settings: &AppSettings) -> Element<'a, Message> {
    let used_gb = ram.used_mb / 1024.0;
    let total_gb = ram.total_mb / 1024.0;

    let content = column![
        header("RAM".to_string()),
        row![
            value_text(format!("{:.1} GB", used_gb), 16),
            Space::with_width(Length::Fill),
            muted_text(format!("of {:.0} GB", total_gb), 10),
        ].align_y(iced::Alignment::Center),
        usage_bar(ram.usage_percent),
    ]
    .spacing(3);

    tile_container(content)
}

pub fn disk_tile<'a>(disk: &DiskData, _settings: &AppSettings) -> Element<'a, Message> {
    let primary = disk.drives.first();
    let (read, write, free) = primary
        .map(|d| (d.read_bytes_sec, d.write_bytes_sec, d.total_gb - d.used_gb))
        .unwrap_or((0, 0, 0.0));
    let percent = primary
        .map(|d| if d.total_gb > 0.0 { d.used_gb / d.total_gb * 100.0 } else { 0.0 })
        .unwrap_or(0.0);

    let content = column![
        header("Disk".to_string()),
        row![
            muted_text(format!("R: {}", format_bytes(read)), 10),
            muted_text(format!("W: {}", format_bytes(write)), 10),
            Space::with_width(Length::Fill),
            value_text(format!("{:.0} GB free", free), 11),
        ].spacing(8).align_y(iced::Alignment::Center),
        usage_bar(percent),
    ]
    .spacing(3);

    tile_container(content)
}

pub fn network_tile<'a>(net: &NetworkData, _settings: &AppSettings) -> Element<'a, Message> {
    let total_up: u64 = net.interfaces.iter().map(|i| i.upload_bytes_sec).sum();
    let total_down: u64 = net.interfaces.iter().map(|i| i.download_bytes_sec).sum();

    let content = column![
        header("Network".to_string()),
        row![
            value_text(format!("\u{2191} {}", format_bytes(total_up)), 11),
            value_text(format!("\u{2193} {}", format_bytes(total_down)), 11),
        ].spacing(14),
    ]
    .spacing(3);

    tile_container(content)
}

fn tile_container<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .padding([7, 10])
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(FluidTheme::TILE)),
            border: Border {
                radius: 6.0.into(),
                ..Border::default()
            },
            ..Default::default()
        })
        .into()
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB/s", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB/s", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB/s", bytes as f64 / 1024.0)
    } else {
        format!("{} B/s", bytes)
    }
}
