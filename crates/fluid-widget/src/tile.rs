use fluid_core::sensor_data::*;
use fluid_core::settings::{AppSettings, TempUnit};
use iced::widget::{column, container, progress_bar, row, text, Space};
use iced::{Border, Element, Length};
use crate::style::Palette;
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

fn header<'a>(label: String, p: Palette) -> Element<'a, Message> {
    text(label).size(10)
        .style(move |_| iced::widget::text::Style { color: Some(p.muted) })
        .into()
}

fn value_text<'a>(s: String, size: u16, p: Palette) -> Element<'a, Message> {
    text(s).size(size)
        .style(move |_| iced::widget::text::Style { color: Some(p.text) })
        .into()
}

fn muted_text<'a>(s: String, size: u16, p: Palette) -> Element<'a, Message> {
    text(s).size(size)
        .style(move |_| iced::widget::text::Style { color: Some(p.muted) })
        .into()
}

fn usage_bar<'a>(percent: f32, p: Palette) -> Element<'a, Message> {
    progress_bar(0.0..=100.0, percent)
        .height(3)
        .style(move |_| iced::widget::progress_bar::Style {
            background: iced::Background::Color(p.bar_bg),
            bar: iced::Background::Color(p.accent),
            border: Border::default(),
        })
        .into()
}

pub fn cpu_tile<'a>(cpu: &CpuData, settings: &AppSettings, p: Palette) -> Element<'a, Message> {
    let name = if cpu.name.is_empty() { "CPU".to_string() } else { cpu.name.clone() };
    let temp = fmt_temp(cpu.temperature_c, settings);
    let clock = cpu.clock_mhz.map(|m| format!("{:.1} GHz", m / 1000.0)).unwrap_or_default();

    tile_container(column![
        header(name, p),
        row![
            value_text(format!("{:.0}%", cpu.usage_percent), 16, p),
            Space::with_width(Length::Fill),
            muted_text(format!("{} {}", clock, temp).trim().to_string(), 10, p),
        ].align_y(iced::Alignment::Center),
        usage_bar(cpu.usage_percent, p),
    ].spacing(3), p)
}

pub fn gpu_tile<'a>(gpu: &GpuData, settings: &AppSettings, p: Palette) -> Element<'a, Message> {
    let name = if gpu.name.is_empty() { "GPU".to_string() } else { gpu.name.clone() };
    let temp = fmt_temp(gpu.temperature_c, settings);

    tile_container(column![
        header(name, p),
        row![
            value_text(format!("{:.0}%", gpu.usage_percent), 16, p),
            Space::with_width(Length::Fill),
            muted_text(temp, 10, p),
        ].align_y(iced::Alignment::Center),
        usage_bar(gpu.usage_percent, p),
    ].spacing(3), p)
}

pub fn ram_tile<'a>(ram: &RamData, _settings: &AppSettings, p: Palette) -> Element<'a, Message> {
    let used_gb = ram.used_mb / 1024.0;
    let total_gb = ram.total_mb / 1024.0;

    tile_container(column![
        header("RAM".to_string(), p),
        row![
            value_text(format!("{:.1} GB", used_gb), 16, p),
            Space::with_width(Length::Fill),
            muted_text(format!("of {:.0} GB", total_gb), 10, p),
        ].align_y(iced::Alignment::Center),
        usage_bar(ram.usage_percent, p),
    ].spacing(3), p)
}

pub fn disk_tile<'a>(disk: &DiskData, _settings: &AppSettings, p: Palette) -> Element<'a, Message> {
    let primary = disk.drives.first();
    let (read, write, free) = primary
        .map(|d| (d.read_bytes_sec, d.write_bytes_sec, d.total_gb - d.used_gb))
        .unwrap_or((0, 0, 0.0));
    let percent = primary
        .map(|d| if d.total_gb > 0.0 { d.used_gb / d.total_gb * 100.0 } else { 0.0 })
        .unwrap_or(0.0);

    tile_container(column![
        header("Disk".to_string(), p),
        row![
            muted_text(format!("R: {}", format_bytes(read)), 10, p),
            muted_text(format!("W: {}", format_bytes(write)), 10, p),
            Space::with_width(Length::Fill),
            value_text(format!("{:.0} GB free", free), 11, p),
        ].spacing(8).align_y(iced::Alignment::Center),
        usage_bar(percent, p),
    ].spacing(3), p)
}

pub fn network_tile<'a>(net: &NetworkData, _settings: &AppSettings, p: Palette) -> Element<'a, Message> {
    let total_up: u64 = net.interfaces.iter().map(|i| i.upload_bytes_sec).sum();
    let total_down: u64 = net.interfaces.iter().map(|i| i.download_bytes_sec).sum();

    tile_container(column![
        header("Network".to_string(), p),
        row![
            value_text(format!("\u{2191} {}", format_bytes(total_up)), 11, p),
            value_text(format!("\u{2193} {}", format_bytes(total_down)), 11, p),
        ].spacing(14),
    ].spacing(3), p)
}

fn tile_container<'a>(content: impl Into<Element<'a, Message>>, p: Palette) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .padding([7, 10])
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(p.tile)),
            border: Border { radius: 6.0.into(), ..Border::default() },
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
