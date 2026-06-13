use fluid_core::settings::{AppSettings, Orientation, TempUnit};
use iced::widget::{button, checkbox, column, container, mouse_area, row, slider, text, Space};
use iced::{Border, Element, Length};
use crate::style::Palette;
use crate::Message;

pub const ACCENT_PRESETS: [(&str, &str); 8] = [
    ("Blue", "#FF3A8FD4"),
    ("Cyan", "#FF2BC8C8"),
    ("Green", "#FF3FB950"),
    ("Purple", "#FF9D6FE0"),
    ("Pink", "#FFE060A8"),
    ("Red", "#FFE05252"),
    ("Orange", "#FFE08A3C"),
    ("Gold", "#FFD4A93A"),
];

const TILES: [&str; 5] = ["CPU", "GPU", "RAM", "Disk", "Network"];

pub fn view<'a>(settings: &AppSettings, p: Palette, win_id: iced::window::Id) -> Element<'a, Message> {
    let section = |label: &str| -> Element<'a, Message> {
        text(label.to_string()).size(11)
            .style(move |_| iced::widget::text::Style { color: Some(p.muted) })
            .into()
    };

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

    let orient_btn = |label: &str, o: Orientation, active: bool| {
        let style = move |_: &iced::Theme, _: button::Status| button::Style {
            background: Some(iced::Background::Color(if active { p.accent } else { p.tile })),
            text_color: p.text,
            border: Border { radius: 4.0.into(), ..Border::default() },
            ..Default::default()
        };
        button(text(label.to_string()).size(11))
            .padding([4, 10])
            .style(style)
            .on_press(Message::SetOrientation(o))
    };
    let orientation_row = row![
        orient_btn("Vertical", Orientation::Vertical, settings.orientation == Orientation::Vertical),
        orient_btn("Horizontal", Orientation::Horizontal, settings.orientation == Orientation::Horizontal),
    ].spacing(6);

    let mut swatch_row = row![].spacing(5);
    for (_, hex) in ACCENT_PRESETS {
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

    let fahrenheit = settings.temperature_unit == TempUnit::Fahrenheit;

    let bottom = row![
        button(text("Reset to Defaults".to_string()).size(11))
            .padding([5, 10])
            .style(move |_, _| button::Style {
                background: Some(iced::Background::Color(p.tile)),
                text_color: p.text,
                border: Border { radius: 4.0.into(), ..Border::default() },
                ..Default::default()
            })
            .on_press(Message::ResetDefaults),
        Space::with_width(Length::Fill),
        button(text("Save and Close".to_string()).size(11))
            .padding([5, 10])
            .style(move |_, _| button::Style {
                background: Some(iced::Background::Color(p.accent)),
                text_color: p.text,
                border: Border { radius: 4.0.into(), ..Border::default() },
                ..Default::default()
            })
            .on_press(Message::SaveClose),
    ];

    // Draggable title bar
    let title_bar = mouse_area(
        container(
            text("Settings".to_string()).size(15)
                .style(move |_| iced::widget::text::Style { color: Some(p.text) })
        )
        .width(Length::Fill)
        .padding([0, 0])
    )
    .on_press(Message::DragWindow(win_id));

    let content = column![
        title_bar,
        Space::with_height(4),
        section("TILES"),
        tiles_col,
        Space::with_height(6),
        section("LAYOUT"),
        orientation_row,
        Space::with_height(6),
        section("OPACITY"),
        slider(0.2..=1.0, settings.widget_opacity, Message::SetOpacity).step(0.05),
        Space::with_height(6),
        section("ACCENT"),
        swatch_row,
        Space::with_height(6),
        checkbox("Fahrenheit".to_string(), fahrenheit)
            .size(14)
            .text_size(12)
            .on_toggle(Message::SetFahrenheit),
        Space::with_height(Length::Fill),
        bottom,
    ]
    .spacing(5);

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
