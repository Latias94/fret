# M122 Runner Window Mapped Events Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner generic mapped window-event handling now lives in
`crates/fret-launch/src/runner/desktop/runner/window_mapped_events.rs`. The split moves the
catchall platform event mapping path out of `app_handler.rs` while preserving mapped-event
collection, wheel-event coalescing into `pending_wheel`, redraw requesting after coalesced wheel
input, RenderDoc F12 capture requests, Escape cancellation for active cross-window dock drags,
mapped event delivery, and effect draining.

The redraw-time pending wheel drain remains in `app_handler.rs`; M122 does not change redraw
scheduling or frame rendering behavior.

Marker summary: mapped window event owner; wheel coalescing catchall; renderdoc/escape handling;
app-handler dispatch only.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_mapped_events.rs` owns
  `handle_window_mapped_event`.
- `handle_window_mapped_event` owns catchall `WinitPlatform::handle_window_event` mapping.
- It owns wheel coalescing from mapped wheel events into `WindowRuntime::pending_wheel`.
- It owns RenderDoc F12 capture requests and Escape-based cross-window dock-drag cancellation.
- It owns final mapped-event delivery and effect draining for catchall window events.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the catchall dispatch
  call and the existing redraw-time pending wheel drain.

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
- Broader workspace gates were not run because M122 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps catchall mapped window-event handling source-auditable in a named owner while leaving
`app_handler.rs` as dispatch plus redraw orchestration. It does not close `DW-P1-linux-003`; the
next true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
