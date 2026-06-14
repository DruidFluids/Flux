//! Windows Firewall rule management for the remote sensor feed.
//!
//! Mirrors the C# installer: a single named inbound rule on TCP 5199 (private
//! profile). The Rust port hosts the feed from the widget rather than a service,
//! so the rule is added the first time the user enables the feed — one elevated
//! UAC prompt, after which Windows won't pop the raw "allow app" dialog on bind.

/// Firewall rule name — identical to the C# app so the two never double-up.
pub const RULE_NAME: &str = "fluidMonitor Remote Sensor";

/// Add (idempotently) the inbound allow rule for the feed. Runs an elevated
/// batch — `delete` then `add` — so a single UAC covers both. Best-effort: if
/// the user declines elevation, the normal per-bind firewall dialog still
/// appears as a fallback.
#[cfg(target_os = "windows")]
pub fn ensure_rule(port: u16) {
    let bat = std::env::temp_dir().join("fluidmonitor_fw_add.bat");
    let script = format!(
        "@echo off\r\n\
         netsh advfirewall firewall delete rule name=\"{RULE_NAME}\" >nul 2>&1\r\n\
         netsh advfirewall firewall add rule name=\"{RULE_NAME}\" dir=in action=allow \
         protocol=tcp localport={port} profile=private \
         description=\"fluidMonitor remote hardware sensor feed\"\r\n\
         del \"%~f0\"\r\n"
    );
    if std::fs::write(&bat, script).is_err() {
        return;
    }
    run_elevated("cmd.exe", &format!("/c \"{}\"", bat.display()));
}

/// Launch `file params` elevated (UAC) with a hidden window, fire-and-forget.
#[cfg(target_os = "windows")]
fn run_elevated(file: &str, params: &str) {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

    let to_w = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };
    let verb = to_w("runas");
    let file_w = to_w(file);
    let params_w = to_w(params);
    unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(file_w.as_ptr()),
            PCWSTR(params_w.as_ptr()),
            PCWSTR::null(),
            SW_HIDE,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_rule(_port: u16) {}
