use iced::widget::{button, column, container, text, toggler};
use iced::{Element, Length, Task, Theme};

fn main() -> iced::Result {
    iced::application("fluidMonitor setup", SetupWizard::update, SetupWizard::view)
        .theme(SetupWizard::theme)
        .window_size((480.0, 400.0))
        .run_with(SetupWizard::new)
}

struct SetupWizard {
    page: usize,
    opt_startup: bool,
    opt_remote: bool,
    opt_pawnio: bool,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    Next,
    Back,
    ToggleStartup(bool),
    ToggleRemote(bool),
    TogglePawnIO(bool),
    SetupComplete,
}

impl SetupWizard {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                page: 0,
                opt_startup: false,
                opt_remote: false,
                opt_pawnio: false,
                status: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Next => {
                if self.page < 3 {
                    self.page += 1;
                }
                if self.page == 2 {
                    self.status = "Registering service...".into();
                    // TODO: run setup tasks
                }
                Task::none()
            }
            Message::Back => {
                if self.page > 0 {
                    self.page -= 1;
                }
                Task::none()
            }
            Message::ToggleStartup(v) => { self.opt_startup = v; Task::none() }
            Message::ToggleRemote(v) => { self.opt_remote = v; Task::none() }
            Message::TogglePawnIO(v) => { self.opt_pawnio = v; Task::none() }
            Message::SetupComplete => {
                self.page = 3;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<Message> = match self.page {
            0 => self.welcome_page(),
            1 => self.options_page(),
            2 => self.progress_page(),
            3 => self.done_page(),
            _ => text("Unknown page").into(),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .center_x(Length::Fill)
            .into()
    }

    fn welcome_page(&self) -> Element<Message> {
        column![
            text("Welcome to fluidMonitor").size(20),
            text("v2.0.0").size(12),
            text("Let's set up a few things").size(14),
            button("Next").on_press(Message::Next),
        ]
        .spacing(12)
        .align_x(iced::Alignment::Center)
        .into()
    }

    fn options_page(&self) -> Element<Message> {
        column![
            text("Options").size(18),
            toggler(self.opt_startup)
                .label("Start with Windows")
                .on_toggle(Message::ToggleStartup),
            toggler(self.opt_remote)
                .label("Remote monitoring")
                .on_toggle(Message::ToggleRemote),
            toggler(self.opt_pawnio)
                .label("CPU temperature sensor")
                .on_toggle(Message::TogglePawnIO),
            button("Back").on_press(Message::Back),
            button("Set up").on_press(Message::Next),
        ]
        .spacing(12)
        .into()
    }

    fn progress_page(&self) -> Element<Message> {
        column![
            text("Setting up").size(18),
            text(&self.status).size(12),
        ]
        .spacing(12)
        .into()
    }

    fn done_page(&self) -> Element<Message> {
        column![
            text("You're all set").size(18),
            text("fluidMonitor is ready to use").size(14),
            button("Get started").on_press(Message::SetupComplete),
        ]
        .spacing(12)
        .align_x(iced::Alignment::Center)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
