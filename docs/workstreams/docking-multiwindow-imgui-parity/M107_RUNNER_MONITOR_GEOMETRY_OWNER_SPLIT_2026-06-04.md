# M107 Runner Monitor Geometry Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner monitor geometry and outer-position settling helpers now live in
`crates/fret-launch/src/runner/desktop/runner/monitor_topology.rs` instead of the general
`window.rs` owner. The split preserves virtual desktop bounds lookup, physical monitor rect
collection, target monitor selection, visibility clamping, Windows work-area preference, mixed-DPI
scale lookup, and DockFloating outer-position settle behavior.

Marker summary: monitor geometry helpers; virtual desktop bounds; outer-position settle; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/monitor_topology.rs` owns `MonitorRectF64`,
  `RectF64`, `virtual_desktop_bounds`, `monitor_rects_physical`,
  `monitor_scale_factor_for_point`, `find_monitor_for_point`, `find_monitor_for_rect`,
  `clamp_window_outer_pos_to_monitor`, and `settle_window_outer_position`.
- `crates/fret-launch/src/runner/desktop/runner/window.rs` keeps window/client coordinate
  conversion, cursor-grab placement, z-order heuristics, and window helper tests, but no longer
  defines the monitor geometry helper bodies or monitor rect types.
- `crates/fret-launch/src/runner/desktop/runner/win32.rs` now imports `MonitorRectF64` from the
  monitor topology owner for Windows work-area projection.
- Existing call sites continue to use the same `WinitRunner` helper methods, so runtime behavior and
  public effect surfaces remain unchanged.

## Commands Run

```powershell
cargo fmt --package fret-launch -- --check
cargo check -p fret-launch --lib
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt --package fret-launch -- --check`: pass.
- `cargo check -p fret-launch --lib`: pass.
- `cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`:
  pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass, with the existing `WORKSTREAM.json` CRLF normalization warning.
- Broader workspace gates were not run because M107 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner monitor geometry and DockFloating outer-position settling
source-auditable in the monitor topology owner without changing runtime behavior. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
