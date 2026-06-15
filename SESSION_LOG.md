# Fluxid (fluidmonitor-rs) — Session Log

## Session: 2026-06-15 (overnight autonomous polish)

### Starting State
Contrary to the overnight brief's assumption of a barebones/stub project, the
repo is a **mature, feature-complete product**:

- `cargo build` — **clean** (0 errors).
- `cargo clippy --workspace` — **clean** (0 warnings).
- `target\debug\fluxid.exe` — **launches**, borderless always-on-top window
  ("Fluxid Widget"), no panics on stderr. (Binary is `fluxid`, not
  `fluidmonitor`.)

Everything in the brief's Phases 1–7 already exists and is polished:

- **Tiles**: CPU/GPU/RAM/Network/Disk/Clock, live sysinfo data, monospace-ish
  layout, warn/flash, per-tile show/hide + field toggles, custom names.
- **Themes**: 57 built-in presets + 25 bundled game-franchise packs (Theme
  Store with install/remove, per-game folders).
- **Skins**: 16 built-ins (Default/Minimal/Sharp/Glass/Carbon/Neon/Cyberpunk/…)
  with glow, gradient, sheen, accent/header bars, texture overlays; plus
  user-installable JSON skins (range-clamped, data-only).
- **Settings window**: tabbed, fully themed custom controls (sliders, togglers,
  pick_lists, dark inputs), live hot-apply.
- **Tray**: icon + Settings/Show/Game Mode/Exit menu.
- **Game mode**, **warnings system** (flash + temp gradient), **global
  hotkeys**, **edge + window snapping**, **opacity**, **UI scale**,
  **run-at-startup**, **click-through**, **updates checker**, **remote
  monitoring** (fluid-remote crate, popouts per device), **optional PawnIO CPU
  temp driver** install flow.

Crates: fluid-core, fluid-sensor, fluid-ipc, fluid-remote, fluid-widget
(main bin `fluxid`), fluid-service, fluid-setup. ~8,000 lines of Rust.

iced 0.13, sysinfo 0.34, serde/serde_json, tray-icon, windows crate.

### Approach
Because the product is already polished and is a GUI widget that can't be fully
visually diffed through automation, this session targets **concrete, low-risk
correctness + polish improvements** verified by reasoning and `cargo build` /
`clippy`, not sweeping rewrites. Each change compiles before commit.

### Completed
- [x] Verified build / clippy / launch (gate zero passed on arrival).
- [x] **settings.rs**: atomic save (temp-file + rename) so a kill mid-write can't
  truncate `settings.json`; on a corrupt/unparseable config, back it up to
  `settings.json.bak` before resetting to defaults (was silently destroying the
  user's settings on the next save). Addresses Phase-6 "missing/corrupt config".
- [x] **fmt.rs**: NaN/Inf guard in `fmt_net`/`fmt_disk` (a NaN rate fell through
  every `<` comparison to the GB/s arm and printed "NaN").
- [x] **style.rs `parse_hex`**: malformed hex (non-hex digits) now falls back to
  the caller's default instead of collapsing to pure black (which could make
  theme text invisible); also guards non-ASCII input against a byte-slice panic.
- [x] **tile.rs**: `pct()` helper guards CPU/GPU/RAM percentage readouts against
  non-finite sensor values; `sub_header` clips long hardware/disk-model names to
  one line (`Wrapping::None`) instead of word-wrapping into the fixed-height tile.

All changes verified: `cargo clippy -p fluid-widget` clean, app relaunches OK.

### Findings deferred (real but higher-risk / lower-value — left for review)
- Width jitter when a byte-rate or VRAM value crosses the 10.0 boundary
  (`"9.9"`→`"10"` changes char count, shifting the content-sized value cell).
  Real but the fix is a layout change (fixed-width number column in
  `line_value`), so deferred rather than risk regressing alignment.

### Decisions Made
- DECISION: Treat this as a polish/bug-hunt session on a mature codebase rather
  than a from-scratch build, since Phases 1–7 already ship. Keep changes small,
  reviewable, and individually committed.

### Known Issues / TODO
- (to be filled as found)
