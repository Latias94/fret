# M121 Runner Window State Events Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner window state event handling now lives in
`crates/fret-launch/src/runner/desktop/runner/window_state_events.rs`. The split moves
`WindowEvent::ModifiersChanged`, `WindowEvent::ThemeChanged`, and `WindowEvent::Focused` handling
out of `app_handler.rs` while preserving modifier platform mapping, internal drag hover rerouting,
theme/environment refresh and redraw requesting, focus state updates, pressed-button reset on focus
loss, focus z-order bump, `Event::WindowFocusChanged` delivery, and macOS focus logging.

Marker summary: window state event owner; focus/environment refresh; app-handler dispatch only.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_state_events.rs` owns
  `handle_window_modifiers_changed`, `handle_window_theme_changed`, and
  `handle_window_focus_changed`.
- `handle_window_modifiers_changed` owns modifier platform mapping and internal drag hover
  rerouting.
- `handle_window_theme_changed` owns theme/environment refresh and redraw requesting.
- `handle_window_focus_changed` owns focus state, pressed button reset, z-order bump,
  `Event::WindowFocusChanged` delivery, and macOS focus logging.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the three winit state
  event dispatch calls.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo fmt --package fret-launch -- --check
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

- `cargo fmt --package fret-launch`: pass.
- `cargo check -p fret-launch --lib`: pass.
- `cargo fmt --package fret-launch -- --check`: pass.
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
- Broader workspace gates were not run because M121 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps modifiers/theme/focus state handling source-auditable in a named window state owner
while leaving `app_handler.rs` as dispatch. It does not close `DW-P1-linux-003`; the next true
closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
