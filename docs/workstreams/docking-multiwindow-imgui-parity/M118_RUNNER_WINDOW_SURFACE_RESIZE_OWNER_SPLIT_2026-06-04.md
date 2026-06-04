# M118 Runner Window Surface Resize Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner `WindowEvent::SurfaceResized` handling now delegates to
`crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs`. The split moves immediate
surface resize synchronization, macOS hit-test refresh for active regions, surface resize redraw
requesting, and effect draining out of `app_handler.rs` without changing the winit event match,
redraw-time eventual-consistency resize fallback, public effect surfaces, or platform behavior.

Marker summary: surface resize event owner; immediate resize sync; app-handler dispatch only.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs` owns
  `handle_window_surface_resized`, which calls `sync_surface_resize_now`, refreshes macOS hit-test
  regions when enabled, calls `request_surface_resize_redraw`, and drains effects.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the
  `WindowEvent::SurfaceResized` dispatch call.
- Existing redraw-time `pending_surface_resize` handling remains in `app_handler.rs` as the
  eventual-consistency fallback for windows that queued a size before surface/context availability.

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
- Broader workspace gates were not run because M118 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps the immediate surface resize event path source-auditable in the surface lifecycle owner
while leaving `app_handler.rs` as dispatch plus redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
