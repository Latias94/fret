# M120 Runner Window Pointer Button Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner `WindowEvent::PointerButton` handling now lives in
`crates/fret-launch/src/runner/desktop/runner/window_pointer_button.rs`. The split moves pointer
button platform event mapping, cursor screen-position fallback, macOS cursor transform calibration,
dock-drag pointer-capture synchronization, left mouse down/up tracking, cursor-based internal drag
drop delivery, DockFloating follow stop on left release, cross-window drag cancellation, dock-source
Up/Down rerouting, mapped event delivery, and effect draining out of `app_handler.rs` without
changing the winit event match, pointer-move handling, redraw behavior, or public effect surfaces.

Marker summary: pointer button event owner; left release drag cleanup; app-handler dispatch only.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_pointer_button.rs` owns
  `handle_window_pointer_button`.
- `handle_window_pointer_button` owns `WindowEvent::PointerButton` platform mapping,
  `MouseButton::Left` down/up tracking, `route_internal_drag_drop_from_cursor`,
  `stop_dock_tearoff_follow`, `clear_internal_drag_hover_if_needed`, `PointerEvent::Up` and
  `PointerEvent::Down` dock-source rerouting, and final effect draining.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the
  `WindowEvent::PointerButton` dispatch call.

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
- Broader workspace gates were not run because M120 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps the pointer-button left-release and dock-source reroute path source-auditable in a named
window pointer owner while leaving `app_handler.rs` as dispatch. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
