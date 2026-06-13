mod tile;
mod style;

use fluid_core::sensor_data::SensorSnapshot;
use fluid_core::settings::AppSettings;
use fluid_sensor::SensorPoller;
use iced::widget::{column, container, mouse_area};
use iced::{window, Border, Element, Length, Size, Subscription, Task, Theme};
use std::time::Duration;
use style::FluidTheme;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    iced::application("fluidMonitor", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window(window::Settings {
            size: Size::new(240.0, 330.0),
            decorations: false,
            transparent: true,
            resizable: false,
            level: window::Level::AlwaysOnTop,
            ..Default::default()
        })
        .run_with(App::new)
}

struct App {
    settings: AppSettings,
    snapshot: SensorSnapshot,
    poller: Option<SensorPoller>,
}

#[derive(Debug, Clone)]
enum Message {
    SensorTick,
    DragWindow,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let settings = AppSettings::load().unwrap_or_default();
        (
            Self {
                settings,
                snapshot: SensorSnapshot::default(),
                poller: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SensorTick => {
                let poller = self.poller.get_or_insert_with(SensorPoller::new);
                self.snapshot = poller.poll();
                Task::none()
            }
            Message::DragWindow => {
                window::get_latest().and_then(window::drag)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tiles = column![
            tile::cpu_tile(&self.snapshot.cpu, &self.settings),
            tile::gpu_tile(&self.snapshot.gpu, &self.settings),
            tile::ram_tile(&self.snapshot.ram, &self.settings),
            tile::disk_tile(&self.snapshot.disk, &self.settings),
            tile::network_tile(&self.snapshot.network, &self.settings),
        ]
        .spacing(5);

        let root = container(tiles)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(FluidTheme::BG)),
                border: Border {
                    radius: 8.0.into(),
                    ..Border::default()
                },
                ..Default::default()
            });

        mouse_area(root)
            .on_press(Message::DragWindow)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(1)).map(|_| Message::SensorTick)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
