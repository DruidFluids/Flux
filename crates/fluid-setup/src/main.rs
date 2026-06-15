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
mod style;

use engine::{InstallOptions, Scope, UninstallOptions};

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if cli::has(&args, &["help", "h", "?"]) {
        show_help();
        std::process::exit(0);
    }
    if cli::has(&args, &["uninstall"]) {
        std::process::exit(run_uninstall_cli(&args));
    }
    // A silent/quiet switch on its own (no GUI) implies a headless install with
    // default options — the NSIS-style `/S` convention.
    if cli::has(&args, &["apply", "install"]) || cli::is_silent(&args) {
        std::process::exit(run_apply_cli(&args));
    }

    gui::run()
}

/// Argument parsing shared by every mode.
///
/// Every flag accepts `--flag`, `-flag` and `/flag` (case-insensitive); the few
/// that take a value use a following argument (`--scope all-users`). This keeps
/// one source of truth so each installer feature has exactly one switch.
mod cli {
    use super::Scope;

    /// Strip any leading `-`/`/` and lowercase, so `/S`, `-s`, `--silent` all
    /// normalise to a comparable token.
    fn norm(a: &str) -> String {
        a.trim_start_matches(['-', '/']).to_ascii_lowercase()
    }

    /// True if any arg matches one of `names` (already-normalised, lowercase).
    pub fn has(args: &[String], names: &[&str]) -> bool {
        args.iter().any(|a| {
            let n = norm(a);
            names.iter().any(|name| n == *name)
        })
    }

    /// The argument following the first occurrence of any of `names`.
    pub fn value<'a>(args: &'a [String], names: &[&str]) -> Option<&'a str> {
        let i = args.iter().position(|a| {
            let n = norm(a);
            names.iter().any(|name| n == *name)
        })?;
        args.get(i + 1).map(|s| s.as_str())
    }

    /// `/S`, `/q`, `--silent`, `--quiet` — suppress UI / message boxes.
    pub fn is_silent(args: &[String]) -> bool {
        has(args, &["s", "silent", "q", "quiet"])
    }

    pub fn scope(args: &[String]) -> Scope {
        value(args, &["scope"])
            .and_then(Scope::parse)
            .unwrap_or(Scope::PerUser)
    }
}

// ── Headless modes ──

/// Headless install. Used both directly (silent/scripted installs) and as the
/// elevated worker the GUI relaunches for an all-users install.
///
/// Default = install everything (desktop + startup + launch); opt out with
/// `--no-desktop` / `--no-startup` / `--no-launch`. The GUI always passes an
/// explicit set of flags so there is no ambiguity for the elevated worker — and
/// it passes `--no-launch` so the worker never starts the widget elevated.
fn run_apply_cli(args: &[String]) -> i32 {
    let opts = InstallOptions {
        scope: cli::scope(args),
        desktop_shortcut: !cli::has(args, &["no-desktop", "nodesktop"]),
        run_at_startup: !cli::has(args, &["no-startup", "nostartup"]),
        launch_after: !cli::has(args, &["no-launch", "nolaunch"]),
    };
    let silent = cli::is_silent(args);
    match engine::install(opts) {
        Ok(_) => 0,
        Err(e) => {
            if !silent {
                msgbox(&format!("Install failed:\n\n{e}"), "Fluxid Setup", true);
            }
            1
        }
    }
}

fn run_uninstall_cli(args: &[String]) -> i32 {
    let opts = UninstallOptions {
        scope: cli::scope(args),
        remove_settings: cli::has(args, &["remove-settings", "removesettings"]),
    };
    let silent = cli::is_silent(args);
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

const HELP_TEXT: &str = "\
Fluxid Setup — command-line switches

  (no switches)         Launch the graphical setup wizard.

Modes
  --install, --apply    Install without the wizard (headless).
  --uninstall           Uninstall (this is what Add/Remove Programs calls).
  /S, /q, --silent      Silent: no wizard and no message boxes. On its own,
                        runs a headless install with default options.
  --help, /?            Show this help.

Install options (default: install everything, per-user)
  --scope per-user      Install for the current user (no admin). Default.
  --scope all-users     Install for all users (prompts for administrator).
  --no-desktop          Do not create a desktop shortcut.
  --no-startup          Do not start Fluxid with Windows.
  --no-launch           Do not launch Fluxid when setup finishes.
  --all                 Enable every optional feature (the default).

Uninstall options
  --scope <type>        Match the scope Fluxid was installed with.
  --remove-settings     Also delete %APPDATA%\\Fluxid (settings/themes/skins).
  /S, --silent          Uninstall with no message boxes.

Every switch accepts --flag, -flag or /flag (case-insensitive).";

fn show_help() {
    // Console for dev/debug builds; a message box for the windowed release.
    println!("{HELP_TEXT}");
    #[cfg(all(windows, not(debug_assertions)))]
    msgbox(HELP_TEXT, "Fluxid Setup", false);
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
        let window = iced::window::Settings {
            size: iced::Size::new(520.0, 460.0),
            min_size: Some(iced::Size::new(460.0, 420.0)),
            icon: load_icon(),
            ..iced::window::Settings::default()
        };
        iced::application("Fluxid Setup", Wizard::update, Wizard::view)
            .theme(Wizard::theme)
            .window(window)
            .run_with(Wizard::new)
    }

    /// Decode the bundled Fluxid logo for the window / taskbar icon (same PNG
    /// the widget uses), so setup is visually branded as Fluxid.
    fn load_icon() -> Option<iced::window::Icon> {
        const PNG: &[u8] = include_bytes!("../assets/icon.png");
        let img = image::load_from_memory(PNG).ok()?.to_rgba8();
        let (w, h) = img.dimensions();
        iced::window::icon::from_rgba(img.into_raw(), w, h).ok()
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
                .style(style::root)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(28)
                .into()
        }

        fn welcome(&self) -> Element<'_, Message> {
            let payload_note: Element<'_, Message> = if payload::is_bundled() {
                text(format!("Package size: {:.1} MB", payload::size_mb()))
                    .size(12)
                    .style(style::muted)
                    .into()
            } else {
                text("⚠ Development build — no payload bundled; install is disabled.")
                    .size(12)
                    .style(style::danger)
                    .into()
            };
            column![
                text("Welcome to Fluxid").size(26).style(style::title),
                text(format!("Version {}", engine::VERSION))
                    .size(13)
                    .style(style::muted),
                Space::with_height(8),
                text("A lightweight system-monitor widget for your desktop.").size(14),
                text("This will install Fluxid and create shortcuts.").size(14),
                Space::with_height(12),
                payload_note,
                Space::with_height(Length::Fill),
                row![
                    Space::with_width(Length::Fill),
                    primary_button("Next", payload::is_bundled().then_some(Message::Next)),
                ],
            ]
            .spacing(6)
            .into()
        }

        fn options_page(&self) -> Element<'_, Message> {
            let location: Element<'_, Message> = match engine::install_dir(self.scope) {
                Ok(dir) => text(format!("Location: {}", dir.display()))
                    .size(12)
                    .style(style::muted)
                    .into(),
                Err(_) => Space::with_height(0).into(),
            };

            let elevation_note: Element<'_, Message> = if self.scope == Scope::AllUsers {
                text("You'll be asked to approve a Windows admin prompt.")
                    .size(12)
                    .style(style::muted)
                    .into()
            } else {
                Space::with_height(0).into()
            };

            let card = container(
                column![
                    text("Install for").size(15).style(style::body),
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
                    location,
                    elevation_note,
                    Space::with_height(6),
                    text("Options").size(15).style(style::body),
                    checkbox("Create a desktop shortcut", self.desktop)
                        .on_toggle(Message::ToggleDesktop),
                    checkbox("Start Fluxid when Windows starts", self.startup)
                        .on_toggle(Message::ToggleStartup),
                    checkbox("Launch Fluxid when setup finishes", self.launch)
                        .on_toggle(Message::ToggleLaunch),
                ]
                .spacing(8),
            )
            .style(style::card)
            .padding(18)
            .width(Length::Fill);

            column![
                text("Setup options").size(22).style(style::title),
                Space::with_height(10),
                card,
                Space::with_height(8),
                text("CPU temperature and remote monitoring can be enabled later in Fluxid's settings.")
                    .size(11)
                    .style(style::muted),
                Space::with_height(Length::Fill),
                row![
                    secondary_button("Back", Some(Message::Back)),
                    Space::with_width(Length::Fill),
                    primary_button("Install", Some(Message::StartInstall)),
                ],
            ]
            .spacing(4)
            .into()
        }

        fn installing(&self) -> Element<'_, Message> {
            column![
                text("Installing…").size(22).style(style::title),
                Space::with_height(12),
                text("Setting up Fluxid. This only takes a moment.")
                    .size(14)
                    .style(style::muted),
            ]
            .spacing(6)
            .into()
        }

        fn done(&self) -> Element<'_, Message> {
            let (heading, detail): (&str, Element<'_, Message>) = match &self.outcome {
                Some(o) if o.ok => {
                    let steps = o.steps.iter().fold(column![].spacing(4), |c, s| {
                        c.push(text(format!("✓  {s}")).size(13).style(style::body))
                    });
                    ("Setup complete", scrollable(steps).height(Length::Fill).into())
                }
                Some(o) => (
                    "Setup failed",
                    text(o.error.clone().unwrap_or_else(|| "Unknown error.".into()))
                        .size(14)
                        .style(style::danger)
                        .into(),
                ),
                None => ("Done", Space::with_height(0).into()),
            };
            column![
                text(heading).size(22).style(style::title),
                Space::with_height(12),
                detail,
                Space::with_height(Length::Fill),
                row![
                    Space::with_width(Length::Fill),
                    primary_button("Close", Some(Message::Finish)),
                ],
            ]
            .spacing(6)
            .into()
        }

        fn theme(&self) -> Theme {
            style::theme()
        }
    }

    fn primary_button(label: &str, msg: Option<Message>) -> Element<'_, Message> {
        styled_button(label, msg, style::primary)
    }

    fn secondary_button(label: &str, msg: Option<Message>) -> Element<'_, Message> {
        styled_button(label, msg, style::secondary)
    }

    fn styled_button(
        label: &str,
        msg: Option<Message>,
        f: impl Fn(&Theme, button::Status) -> button::Style + 'static,
    ) -> Element<'_, Message> {
        let b = button(text(label).size(14))
            .padding([8, 22])
            .width(Length::Shrink)
            .style(f);
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
        // Pass an explicit, fully-specified flag set (the worker defaults to
        // "install everything", so the unchecked options must be negated), and
        // always --no-launch so the elevated worker never starts the widget.
        let mut apply = vec![
            "--apply".to_string(),
            "--scope".to_string(),
            "all-users".to_string(),
            "--no-launch".to_string(),
        ];
        if !opts.desktop_shortcut {
            apply.push("--no-desktop".into());
        }
        if !opts.run_at_startup {
            apply.push("--no-startup".into());
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
