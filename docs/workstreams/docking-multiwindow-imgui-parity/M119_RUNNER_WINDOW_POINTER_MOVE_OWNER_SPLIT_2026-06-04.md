# M119 Runner Window Pointer Move Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner `WindowEvent::PointerMoved` handling now lives in
`crates/fret-launch/src/runner/desktop/runner/window_pointer_move.rs`. The split moves pointer-move
platform event mapping, non-touch screen-position sampling, macOS cursor transform calibration,
DockFloating follow updates, external drag over-event co-delivery, cross-window dock-drag move
rerouting, dock-drag pointer-capture synchronization, internal drag hover routing, and effect
draining out of `app_handler.rs` without changing the winit event match, pointer button handling,
redraw behavior, or public effect surfaces.

Marker summary: pointer move event owner; dock drag move reroute; app-handler dispatch only.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_pointer_move.rs` owns
  `handle_window_pointer_moved`.
- `handle_window_pointer_moved` owns `WindowEvent::PointerMoved` platform mapping,
  `PointerSource::Touch` screen-sample suppression, `PhysicalPosition::new` screen coordinate
  projection, external drag `ExternalDragKind::OverFiles` delivery, cross-window dock-drag move
  rerouting, `sync_dock_drag_pointer_capture`, `route_internal_drag_hover_from_cursor`, and final
  effect draining.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the
  `WindowEvent::PointerMoved` dispatch call.

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
- Broader workspace gates were not run because M119 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps the pointer-move dock-drag and external-drag routing path source-auditable in a named
window pointer owner while leaving `app_handler.rs` as dispatch. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
