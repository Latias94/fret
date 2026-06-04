# M117 Runner Window External Drag Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner external file drag window-event handling now lives in
`crates/fret-launch/src/runner/desktop/runner/window_external_drag.rs`. The split moves the
`WindowEvent::DragEntered`, `WindowEvent::DragMoved`, `WindowEvent::DragDropped`, and
`WindowEvent::DragLeft` state machine out of `app_handler.rs` while preserving token allocation and
reuse, path-cache updates, physical-to-logical pointer mapping, `ExternalDragKind::EnterFiles`,
`ExternalDragKind::OverFiles`, `ExternalDragKind::DropFiles`, `ExternalDragKind::Leave`,
payload-path publication, token release, event delivery, and effect draining.

The `PointerMoved` path in `app_handler.rs` still owns the normal pointer-event merge point and its
external-drag over-event co-delivery.

Marker summary: external file drag state machine; token/path cache; app-handler dispatch only.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_external_drag.rs` owns
  `handle_window_drag_entered`, `handle_window_drag_moved`, `handle_window_drag_dropped`, and
  `handle_window_drag_left`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` dispatches the four winit external
  drag arms to those helpers and keeps the pointer-move event merge path.
- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod window_external_drag;`.
- Existing public event surfaces and platform provider types remain unchanged.

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
- Broader workspace gates were not run because M117 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner external file drag state handling source-auditable in a named window event
owner and leaves `app_handler.rs` as dispatch plus pointer-event merge orchestration. It does not
close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
