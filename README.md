# Fluxid

A lightweight, good-looking system-monitor **widget** for the Windows desktop —
CPU, GPU, RAM, network, disk and a clock in a borderless always-on-top tile,
with themes, skins, warnings, game mode, global hotkeys and optional remote
monitoring. Built in Rust with [iced](https://iced.rs).

> Rewrite of the original C# fluidMonitor, in Rust for broad hardware coverage
> and a path to Linux/macOS. The widget renders on the GPU (wgpu) and polls
> sensors in-process, so it ships as a single executable with no service and no
> runtime dependency.

## Install

1. Download the latest **`fluidmonitor-setup-vX.Y.Z.exe`** from
   [Releases](https://github.com/DruidFluids/fluidmonitor-rs/releases).
2. (Optional but recommended) verify the download against its published
   `.sha256`:
   ```powershell
   Get-FileHash .\fluidmonitor-setup-vX.Y.Z.exe -Algorithm SHA256
   ```
3. Run it. The build is **unsigned**, so Windows SmartScreen shows a one-time
   “Windows protected your PC” prompt — click **More info → Run anyway**.
4. Follow the wizard: choose **Just me** (no admin) or **All users**, pick the
   optional desktop shortcut / startup / launch, and click **Install**.

The installer is a small self-contained custom installer that embeds the widget
— no separate download, no service, no .NET. It can also run **silently** for
scripted deployments:

```bat
fluidmonitor-setup.exe /S                 :: silent per-user install of everything
fluidmonitor-setup.exe --help             :: list every switch
```

**See [`docs/INSTALLER.md`](docs/INSTALLER.md)** for the full command-line
reference, install locations, the registry/shortcut layout, and uninstall
instructions.

### Uninstall

**Settings → Apps → Fluxid → Uninstall** (or Control Panel → Programs and
Features). Your settings in `%APPDATA%\Fluxid` are kept unless you uninstall
from the command line with `--remove-settings`.

## Build from source

Requires a recent stable Rust toolchain (Windows).

```powershell
# Run the widget directly
cargo run -p fluid-widget --release

# Build the distributable installer (widget + embedded payload + checksum)
powershell -ExecutionPolicy Bypass -File scripts\Build-Setup.ps1
# -> dist\fluidmonitor-setup-v<version>.exe
```

## Workspace layout

| Crate | What it is |
|-------|------------|
| `fluid-widget` | The widget app (binary `fluxid`). |
| `fluid-sensor` | Hardware sensor polling (sysinfo, NVML, optional PawnIO CPU temp). |
| `fluid-core` | Shared settings/types. |
| `fluid-ipc` / `fluid-remote` | Local IPC and remote-monitoring transport. |
| `fluid-setup` | The custom installer (binary `fluidmonitor-setup`). |
| `fluid-service` | Optional standalone sensor service. |

## License

MIT © Matt Hakes
