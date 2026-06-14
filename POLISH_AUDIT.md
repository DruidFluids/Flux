# fluidMonitor-rs — Polish Audit

A multi-pass, file-by-file polish run: lint cleanup, dead-code pruning,
documentation, and correctness review. Each pass is documented below with
findings and resolutions.

---

## Pass 1 — machine lints + first manual read

### Tooling baseline
- `cargo clippy --workspace --all-targets` initially failed: a deny-level
  `clippy::never_loop` in `fluid-ipc` aborted the build graph, so `fluid-widget`
  (the largest crate) was never linted. Fixed first to unblock.

### Clippy findings & resolutions
| # | File | Lint | Resolution |
|---|------|------|------------|
| 1 | fluid-ipc/src/lib.rs:55 | `never_loop` (deny) | rewrote first-line read as `lines().next()` |
| 2 | fluid-sensor/src/lib.rs:78 | `unnecessary_map_or` | `map_or(true, …)` → `is_none_or` |
| 3 | fluid-sensor/src/lib.rs:243 | `new_without_default` | added `Default for SensorPoller` |
| 4 | fluid-remote/src/client.rs:14 | `large_enum_variant` | boxed `ClientEvent::Snapshot` |
| 5 | fluid-remote/src/lib.rs:27 | `large_enum_variant` | boxed `RemoteEvent::Snapshot` |
| 6 | fluid-setup/src/main.rs ×5 | `mismatched_lifetime_syntaxes` | `Element<Message>` → `Element<'_, Message>` |
| 7 | fluid-remote/tests/loopback.rs:42 | `field_reassign_with_default` | struct-init form |
| 8 | fluid-widget/src/main.rs | `dead_code` (`Message::ThemeDice`) | (see manual notes) |

### Manual findings & resolutions
| # | Location | Finding | Resolution |
|---|----------|---------|------------|
| M1 | fluid-core/src/{color,theme,error}.rs | **3 entirely dead modules** — `Color`, `ThemePalette`/`BuiltInThemes`/`ThemePack`, `FluidError` referenced only by their own `pub use` re-exports | deleted all three modules + re-exports |
| M2 | fluid-core/Cargo.toml | `iced` (wgpu!), `reqwest`, `thiserror` only used by the deleted modules | removed all three deps — fluid-core now pulls only serde/serde_json/anyhow/directories |
| M3 | fluid-core/src/sensor_data.rs | `cpu_temp_display` / `ram_usage_display` never called (tiles format inline) | pruned both methods + the empty impl block |
| M4 | fluid-widget/src/main.rs | `Message::ThemeDice` + handler orphaned (no sender; unified Die replaced it) | removed variant + handler |
| M5 | 14 source files | no `//!` module documentation | added concise module docs to every file |
| M6 | fmt.rs, settings_panel.rs, style.rs | leading UTF-8 **BOM** in source | stripped |

### Verified clean (no action needed)
- All `.unwrap()`/`.expect()` in non-test code are startup or invariant-safe
  (tray icon from const, mutex locks, `warn_mut` find-after-push, static names).
- No `#[allow(dead_code)]` hiding anything; compiler reports zero dead code.
- Remaining `TODO`s are legitimate platform gaps: macOS GPU/CPU-temp sensor
  stubs (degrade to `None`), and `fluid-setup`'s progress page (separate binary).

### Pass 1 result
- `cargo clippy --workspace --all-targets`: **0 warnings, 0 errors**
- `cargo build --workspace`: clean
- `cargo test --workspace`: 1 passed (loopback), rest have no tests
- Net: removed 3 modules, 3 dependencies, 2 dead methods, 1 dead message; added 14 module docs.

---

## Pass 2 — deduplication + deeper read ("polishing the polish")

### Findings & resolutions
| # | Location | Finding | Resolution |
|---|----------|---------|------------|
| P1 | settings_panel.rs ×2, popups.rs ×1 | three near-identical "InlineBtn" closures with **inconsistent** radius (4 vs 6) and padding (4,10 / 4,12 / 5,12) | extracted `style::inline_btn` as the single source of truth (radius 6, padding 5/12); locals are now one-line forwarders → zero call-site churn, consistent look |

### Reviewed, deliberately left as-is (with rationale)
- `fmt_net` vs `fmt_disk`: near-duplicate, but the KB precision differs on
  purpose (net shows `12.3 KB/s`, disk shows `12 KB/s`) to mirror the C# app.
- Status/accent colour literals (success greens `#3DC98A`/`#58C858`, danger reds
  `#CD5C5C`/`#C06060`, alert `#E06040`) are **not** unified: each mirrors a
  specific C# brush (`IndianRed`, etc.); faithfulness to the bible > internal
  de-dup.
- `popups::pill` vs the `settings_panel` `pill` closure differ materially
  (Segoe font + transparent/accent fill vs simple fill) — not duplicates.
- Dense one-statement-per-`;` formatting is the deliberate house style (no
  `rustfmt.toml`); **not** running `cargo fmt` — it would fight the author's
  layout and explode the diff for no behavioural gain.

### Pass 2 result
- `cargo clippy -p fluid-widget`: 0 warnings
- Visual regression: widget renders identically (all tiles, glow arrows, RAM
  speed) after Pass 1+2 changes.
