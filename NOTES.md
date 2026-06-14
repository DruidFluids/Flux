# Notes / pending decisions

Things that need a product decision from you — nothing is blocked, these are
trade-offs I didn't want to make unilaterally.

## CPU temperature accuracy (needs your call)
Accurate CPU **package/die** temperature on Windows requires a kernel-level
sensor driver (RAPL/MSR access). The old C# app used PawnIO for this. That
conflicts directly with the "zero security issues / no kernel driver" goal.

What the Rust port does now, in order of preference:
1. **sysinfo components** — only populated if a hardware-monitor driver is already feeding the OS.
2. **LibreHardwareMonitor / OpenHardwareMonitor WMI** — accurate CPU Package/core temp, *if you run one of those apps in the background* (no driver shipped by us). **Added today.**
3. **ACPI thermal zone (MSAcpi)** — coarse fallback. On your machine this reports ~17 °C, which is an ambient/chipset zone, **not** the CPU die. It now **rejects readings below 20 °C** (impossible for a CPU die) so the tile shows "—" instead of a misleading "17 °C". So: with nothing else available, CPU temp will read "—" until you run a hardware monitor (option a).

Options:
- **(a)** Run LibreHardwareMonitor in the background → fluidMonitor will read accurate temps automatically. Zero security cost. *(Recommended.)*
- **(b)** Accept that CPU temp may be inaccurate/absent without a helper.
- **(c)** Ship a signed kernel sensor driver → accurate, but a security/AV surface you've said you want to avoid.

Related limitation: **CPU clock** via sysinfo is the base/nominal clock on
Windows (e.g. a static 4300 MHz), not the live boosting frequency — there's no
clean driver-free live-clock API. CPU **usage** does update correctly.

## Settings UI redesign — DONE (tabs)
Rebuilt the Settings window as **tabs** (Tiles · Appearance · Behavior ·
Sensors · Remote · Updates) — one category at a time. The window resizes to
each tab so it stays compact, and there is **no scrollbar**. This diverges from
the C# single-pane layout deliberately, per your "less at once / no scrollbar /
not too big" feedback. If you'd prefer a different grouping or tab order, say so.

## Done recently
- Settings redesigned into compact tabs; no scrollbar; per-tab sizing.
- Secondary windows skip the taskbar (only the widget shows one entry).
- Light-theme readability fixed everywhere (field backgrounds + Alerts/colour-hex inputs + themed sliders).
- All window titles flush in the top-left corner.
- CPU temperature °C/°F moved to the top of Settings.
- Appearance changes (theme/skin/colours/fonts) persist immediately.
- Popup/sub-windows remember their last position.
- "colours" → "colors" in user-facing text.
