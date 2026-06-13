use fluid_core::settings::{AppSettings, Orientation, TempUnit};
use iced::widget::{button, checkbox, column, container, mouse_area, row, scrollable, slider, text, Space};
use iced::{Border, Element, Length};
use crate::style::Palette;
use crate::Message;

pub const ACCENT_PRESETS: [&str; 8] = [
    "#FF00A8FF", "#FF2BC8C8", "#FF3FB950", "#FF9D6FE0",
    "#FFE060A8", "#FFE05252", "#FFE08A3C", "#FFD4A93A",
];

const TILES: [&str; 6] = ["CPU", "GPU", "RAM", "Disk", "Network", "Clock"];

pub fn view<'a>(settings: &AppSettings, p: Palette, win_id: iced::window::Id, theme_name: String) -> Element<'a, Message> {
    let section = |label: &str| -> Element<'a, Message> {
        text(label.to_string()).size(11)
            .style(move |_| iced::widget::text::Style { color: Some(p.muted) })
            .into()
    };

    let pill = move |label: String, active: bool, msg: Message| {
        button(text(label).size(11))
            .padding([4, 10])
            .style(move |_: &iced::Theme, _: button::Status| button::Style {
                background: Some(iced::Background::Color(if active { p.accent } else { p.tile })),
                text_color: p.text,
                border: Border { radius: 4.0.into(), ..Border::default() },
                ..Default::default()
            })
            .on_press(msg)
    };

    // ── Theme cycler ──
    let theme_row = row![
        pill("\u{25C0}".into(), false, Message::ThemePrev),
        container(
            text(theme_name).size(12)
                .style(move |_| iced::widget::text::Style { color: Some(p.text) })
        ).width(Length::Fill).center_x(Length::Fill),
        pill("\u{25B6}".into(), false, Message::ThemeNext),
    ]
    .align_y(iced::Alignment::Center)
    .spacing(6);

    // ── Tile toggles ──
    let mut tiles_col = column![].spacing(4);
    for t in TILES {
        let visible = settings.visible_tiles.iter().any(|v| v == t);
        let name = t.to_string();
        tiles_col = tiles_col.push(
            checkbox(t.to_string(), visible)
                .size(14)
                .text_size(12)
                .on_toggle(move |on| Message::ToggleTile(name.clone(), on)),
        );
    }

    // ── Layout ──
    let orientation_row = row![
        pill("Vertical".into(), settings.orientation == Orientation::Vertical, Message::SetOrientation(Orientation::Vertical)),
        pill("Horizontal".into(), settings.orientation == Orientation::Horizontal, Message::SetOrientation(Orientation::Horizontal)),
    ].spacing(6);

    // ── Accent swatches ──
    let mut swatch_row = row![].spacing(5);
    for hex in ACCENT_PRESETS {
        let c = crate::style::swatch_color(hex);
        let selected = settings.theme_accent.eq_ignore_ascii_case(hex);
        let hex_owned = hex.to_string();
        swatch_row = swatch_row.push(
            button(Space::new(18, 18))
                .padding(0)
                .style(move |_, _| button::Style {
                    background: Some(iced::Background::Color(c)),
                    border: Border {
                        radius: 9.0.into(),
                        width: if selected { 2.0 } else { 0.0 },
                        color: p.text,
                    },
                    ..Default::default()
                })
                .on_press(Message::SetAccent(hex_owned.clone())),
        );
    }

    // ── Warnings ──
    let gpu_warn = settings.warn("GPU").cloned().unwrap_or_default();
    let cpu_warn = settings.warn("CPU").cloned().unwrap_or_default();

    let warnings_col = column![
        checkbox(format!("GPU temp warning ({:.0}\u{00B0})", gpu_warn.threshold), gpu_warn.enabled)
            .size(14).text_size(12)
            .on_toggle(|on| Message::SetWarnEnabled("GPU".into(), on)),
        slider(40.0..=100.0, gpu_warn.threshold as f32, |v| Message::SetWarnThreshold("GPU".into(), v)).step(1.0),
        row![
            checkbox("Flash".to_string(), gpu_warn.flash_enabled)
                .size(13).text_size(11)
                .on_toggle(|on| Message::SetWarnFlash("GPU".into(), on)),
            checkbox("Gradient".to_string(), gpu_warn.gradient_mode)
                .size(13).text_size(11)
                .on_toggle(|on| Message::SetWarnGradient("GPU".into(), on)),
        ].spacing(12),
        Space::with_height(4),
        checkbox(format!("CPU load warning ({:.0}%)", cpu_warn.threshold), cpu_warn.enabled)
            .size(14).text_size(12)
            .on_toggle(|on| Message::SetWarnEnabled("CPU".into(), on)),
        slider(50.0..=100.0, cpu_warn.threshold as f32, |v| Message::SetWarnThreshold("CPU".into(), v)).step(1.0),
        checkbox("Flash".to_string(), cpu_warn.flash_enabled)
            .size(13).text_size(11)
            .on_toggle(|on| Message::SetWarnFlash("CPU".into(), on)),
    ].spacing(4);

    let fahrenheit = settings.temperature_unit == TempUnit::Fahrenheit;

    let bottom = row![
        pill("Reset to Defaults".into(), false, Message::ResetDefaults),
        Space::with_width(Length::Fill),
        pill("Save and Close".into(), true, Message::SaveClose),
    ];

    let title_bar = mouse_area(
        container(
            text("Settings".to_string()).size(15)
                .style(move |_| iced::widget::text::Style { color: Some(p.text) })
        ).width(Length::Fill)
    )
    .on_press(Message::DragWindow(win_id));

    let body = column![
        section("THEME"),
        theme_row,
        Space::with_height(6),
        section("TILES"),
        tiles_col,
        Space::with_height(6),
        section("LAYOUT"),
        orientation_row,
        checkbox("Snap to edges".to_string(), settings.snap_to_edges)
            .size(14).text_size(12)
            .on_toggle(Message::SetSnap),
        Space::with_height(6),
        section("OPACITY"),
        slider(0.2..=1.0, settings.widget_opacity, Message::SetOpacity).step(0.05),
        Space::with_height(6),
        section("ACCENT"),
        swatch_row,
        Space::with_height(6),
        section("WARNINGS"),
        warnings_col,
        Space::with_height(6),
        checkbox("Fahrenheit".to_string(), fahrenheit)
            .size(14).text_size(12)
            .on_toggle(Message::SetFahrenheit),
    ]
    .spacing(5);

    let content = column![
        title_bar,
        Space::with_height(4),
        scrollable(body).height(Length::Fill),
        Space::with_height(6),
        bottom,
    ]
    .spacing(4);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(12)
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(p.bg)),
            border: Border { radius: 8.0.into(), ..Border::default() },
            ..Default::default()
        })
        .into()
}
