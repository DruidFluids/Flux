//! Fluxid setup — a self-contained custom installer.
//!
//! Three modes, chosen by CLI args:
//! * (no args) → the iced wizard GUI.
//! * `--apply` → headless install engine (also the elevated worker the GUI
//!   spawns for an all-users install).
//! * `--uninstall` → headless uninstall engine; this exe is copied into the
//!   install dir and registered as the Add/Remove-Programs uninstall command.
//!
//! The widget (`fluxid.exe`) is embedded at build time (see `build.rs` /
//! `payload.rs`); there is no separate service and no runtime dependency, so
//! the installer's whole job is: copy the exe, make shortcuts, register the
//! uninstaller, apply the startup opt-in, and launch.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod engine;
mod payload;

use engine::{InstallOptions, Scope, UninstallOptions};

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--uninstall") {
        std::process::exit(run_uninstall_cli(&args));
    }
    if args.iter().any(|a| a == "--apply") {
        std::process::exit(run_apply_cli(&args));
    }

    gui::run()
}

// ── Headless modes ──

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn opt_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let i = args.iter().position(|a| a == name)?;
    args.get(i + 1).map(|s| s.as_str())
}

fn scope_from_args(args: &[String]) -> Scope {
    opt_value(args, "--scope")
        .and_then(Scope::parse)
        .unwrap_or(Scope::PerUser)
}

/// Headless install worker (the elevated process the GUI relaunches). Never
/// launches the widget itself — that would run it elevated; the GUI does it.
fn run_apply_cli(args: &[String]) -> i32 {
    let opts = InstallOptions {
        scope: scope_from_args(args),
        desktop_shortcut: flag(args, "--desktop"),
        run_at_startup: flag(args, "--startup"),
        launch_after: false,
    };
    match engine::install(opts) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

fn run_uninstall_cli(args: &[String]) -> i32 {
    let opts = UninstallOptions {
        scope: scope_from_args(args),
        remove_settings: flag(args, "--remove-settings"),
    };
    let silent = flag(args, "--silent");
    match engine::uninstall(opts) {
        Ok(_) => {
            if !silent {
                msgbox("Fluxid has been uninstalled.", "Fluxid", false);
            }
            0
        }
        Err(e) => {
            if !silent {
                msgbox(&format!("Uninstall failed:\n\n{e}"), "Fluxid", true);
            }
            1
        }
    }
}

#[cfg(windows)]
fn msgbox(text: &str, caption: &str, error: bool) {
    use windows::core::HSTRING;
    use windows::Win32::UI::WindowsAndMessaging::{
        MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
    };
    let icon = if error { MB_ICONERROR } else { MB_ICONINFORMATION };
    unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(text),
            &HSTRING::from(caption),
            MB_OK | icon,
        );
    }
}
#[cfg(not(windows))]
fn msgbox(_text: &str, _caption: &str, _error: bool) {}

// ───────────────────────────── GUI wizard ─────────────────────────────

mod gui {
    use super::*;
    use iced::widget::{
        button, checkbox, column, container, radio, row, scrollable, text, Space,
    };
    use iced::{Element, Length, Task, Theme};

    pub fn run() -> iced::Result {
        iced::application("Fluxid Setup", Wizard::update, Wizard::view)
            .theme(Wizard::theme)
            .window_size((520.0, 440.0))
            .run_with(Wizard::new)
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Next,
        Back,
        SetScope(Scope),
        ToggleDesktop(bool),
        ToggleStartup(bool),
        ToggleLaunch(bool),
        StartInstall,
        Installed(Outcome),
        Finish,
    }

    /// A Clone+Send result the async install Task hands back to the UI.
    #[derive(Debug, Clone)]
    pub struct Outcome {
        pub ok: bool,
        pub steps: Vec<String>,
        pub error: Option<String>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Page {
        Welcome,
        Options,
        Installing,
        Done,
    }

    struct Wizard {
        page: Page,
        scope: Scope,
        desktop: bool,
        startup: bool,
        launch: bool,
        outcome: Option<Outcome>,
    }

    impl Wizard {
        fn new() -> (Self, Task<Message>) {
            (
                Self {
                    page: Page::Welcome,
                    scope: Scope::PerUser,
                    desktop: true,
                    startup: true,
                    launch: true,
                    outcome: None,
                },
                Task::none(),
            )
        }

        fn options(&self) -> InstallOptions {
            InstallOptions {
                scope: self.scope,
                desktop_shortcut: self.desktop,
                run_at_startup: self.startup,
                launch_after: self.launch,
            }
        }

        fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::Next => {
                    self.page = Page::Options;
                    Task::none()
                }
                Message::Back => {
                    self.page = Page::Welcome;
                    Task::none()
                }
                Message::SetScope(s) => {
                    self.scope = s;
                    Task::none()
                }
                Message::ToggleDesktop(v) => {
                    self.desktop = v;
                    Task::none()
                }
                Message::ToggleStartup(v) => {
                    self.startup = v;
                    Task::none()
                }
                Message::ToggleLaunch(v) => {
                    self.launch = v;
                    Task::none()
                }
                Message::StartInstall => {
                    self.page = Page::Installing;
                    let opts = self.options();
                    Task::perform(run_install_async(opts), Message::Installed)
                }
                Message::Installed(outcome) => {
                    self.outcome = Some(outcome);
                    self.page = Page::Done;
                    Task::none()
                }
                Message::Finish => iced::exit(),
            }
        }

        fn view(&self) -> Element<'_, Message> {
            let body: Element<'_, Message> = match self.page {
                Page::Welcome => self.welcome(),
                Page::Options => self.options_page(),
                Page::Installing => self.installing(),
                Page::Done => self.done(),
            };
            container(body)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(28)
                .into()
        }

        fn welcome(&self) -> Element<'_, Message> {
            let payload_note: Element<'_, Message> = if payload::is_bundled() {
                text(format!("Package size: {:.1} MB", payload::size_mb()))
                    .size(12)
                    .into()
            } else {
                text("⚠ Development build — no payload bundled; install is disabled.")
                    .size(12)
                    .style(text::danger)
                    .into()
            };
            column![
                text("Welcome to Fluxid").size(26),
                text(format!("Version {}", engine::VERSION)).size(13),
                Space::with_height(8),
                text("A lightweight system-monitor widget for your desktop.").size(14),
                text("This will install Fluxid and create shortcuts.").size(14),
                Space::with_height(12),
                payload_note,
                Space::with_height(Length::Fill),
                row![
                    Space::with_width(Length::Fill),
                    nav_button("Next", payload::is_bundled().then_some(Message::Next)),
                ],
            ]
            .spacing(6)
            .into()
        }

        fn options_page(&self) -> Element<'_, Message> {
            let scope_choice = column![
                text("Install for").size(15),
                radio(
                    "Just me  (no admin required)",
                    Scope::PerUser,
                    Some(self.scope),
                    Message::SetScope,
                ),
                radio(
                    "All users  (requires administrator)",
                    Scope::AllUsers,
                    Some(self.scope),
                    Message::SetScope,
                ),
            ]
            .spacing(8);

            let location: Element<'_, Message> = match engine::install_dir(self.scope) {
                Ok(dir) => text(format!("Location: {}", dir.display())).size(12).into(),
                Err(_) => Space::with_height(0).into(),
            };

            let elevation_note: Element<'_, Message> = if self.scope == Scope::AllUsers {
                text("You'll be asked to approve a Windows admin prompt.")
                    .size(12)
                    .into()
            } else {
                Space::with_height(0).into()
            };

            let choices = column![
                text("Options").size(15),
                checkbox("Create a desktop shortcut", self.desktop)
                    .on_toggle(Message::ToggleDesktop),
                checkbox("Start Fluxid when Windows starts", self.startup)
                    .on_toggle(Message::ToggleStartup),
                checkbox("Launch Fluxid when setup finishes", self.launch)
                    .on_toggle(Message::ToggleLaunch),
            ]
            .spacing(8);

            column![
                text("Setup options").size(22),
                Space::with_height(10),
                scope_choice,
                location,
                elevation_note,
                Space::with_height(14),
                choices,
                Space::with_height(8),
                text("CPU temperature and remote monitoring can be enabled later in Fluxid's settings.")
                    .size(11),
                Space::with_height(Length::Fill),
                row![
                    nav_button("Back", Some(Message::Back)),
                    Space::with_width(Length::Fill),
                    nav_button("Install", Some(Message::StartInstall)),
                ],
            ]
            .spacing(4)
            .into()
        }

        fn installing(&self) -> Element<'_, Message> {
            column![
                text("Installing…").size(22),
                Space::with_height(12),
                text("Setting up Fluxid. This only takes a moment.").size(14),
            ]
            .spacing(6)
            .into()
        }

        fn done(&self) -> Element<'_, Message> {
            let (title, detail): (&str, Element<'_, Message>) = match &self.outcome {
                Some(o) if o.ok => {
                    let steps = o.steps.iter().fold(column![].spacing(4), |c, s| {
                        c.push(text(format!("✓  {s}")).size(13))
                    });
                    ("Setup complete", scrollable(steps).height(Length::Fill).into())
                }
                Some(o) => (
                    "Setup failed",
                    text(o.error.clone().unwrap_or_else(|| "Unknown error.".into()))
                        .size(14)
                        .style(text::danger)
                        .into(),
                ),
                None => ("Done", Space::with_height(0).into()),
            };
            column![
                text(title).size(22),
                Space::with_height(12),
                detail,
                Space::with_height(Length::Fill),
                row![
                    Space::with_width(Length::Fill),
                    nav_button("Close", Some(Message::Finish)),
                ],
            ]
            .spacing(6)
            .into()
        }

        fn theme(&self) -> Theme {
            Theme::Dark
        }
    }

    fn nav_button(label: &str, msg: Option<Message>) -> Element<'_, Message> {
        let b = button(text(label).size(14))
            .padding([8, 20])
            .width(Length::Shrink);
        match msg {
            Some(m) => b.on_press(m).into(),
            None => b.into(),
        }
    }

    /// Run the (blocking) install off the UI thread and normalise the result.
    async fn run_install_async(opts: InstallOptions) -> Outcome {
        let result =
            tokio::task::spawn_blocking(move || run_install_flow(opts)).await;
        match result {
            Ok(o) => o,
            Err(_) => Outcome {
                ok: false,
                steps: vec![],
                error: Some("Internal error during install.".into()),
            },
        }
    }

    /// Decide the in-process vs. elevated-worker path and produce an [`Outcome`].
    fn run_install_flow(opts: InstallOptions) -> Outcome {
        // Per-user (or already elevated) installs run in-process.
        if opts.scope == Scope::PerUser || engine::is_elevated() {
            return match engine::install(opts) {
                Ok(rep) => Outcome { ok: true, steps: rep.steps, error: None },
                Err(e) => Outcome {
                    ok: false,
                    steps: vec![],
                    error: Some(e.to_string()),
                },
            };
        }

        // All-users from an unelevated GUI: relaunch ourselves elevated to do
        // the privileged file/registry work, then launch the widget unelevated.
        let mut apply = vec![
            "--apply".to_string(),
            "--scope".to_string(),
            "all-users".to_string(),
        ];
        if opts.desktop_shortcut {
            apply.push("--desktop".into());
        }
        if opts.run_at_startup {
            apply.push("--startup".into());
        }

        match engine::relaunch_elevated_wait(&apply) {
            Ok(Some(0)) => {
                let mut steps =
                    vec!["Installed Fluxid (administrator)".to_string()];
                if opts.launch_after {
                    match engine::launch(opts.scope) {
                        Ok(()) => steps.push("Launched Fluxid".into()),
                        Err(e) => {
                            return Outcome {
                                ok: true,
                                steps,
                                error: Some(e.to_string()),
                            }
                        }
                    }
                }
                Outcome { ok: true, steps, error: None }
            }
            Ok(Some(code)) => Outcome {
                ok: false,
                steps: vec![],
                error: Some(format!("The installer exited with code {code}.")),
            },
            Ok(None) => Outcome {
                ok: false,
                steps: vec![],
                error: Some("Administrator approval was declined.".into()),
            },
            Err(e) => Outcome {
                ok: false,
                steps: vec![],
                error: Some(e.to_string()),
            },
        }
    }
}
