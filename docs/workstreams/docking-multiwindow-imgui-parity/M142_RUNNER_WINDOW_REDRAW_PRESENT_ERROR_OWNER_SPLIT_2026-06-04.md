# M142 Runner Window Redraw Present Error Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time present error and surface recovery handling now live in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present_error.rs`. The split moves the
`RenderError::SurfaceAcquireFailed` match, Lost/Outdated surface clearing, Timeout one-shot redraw
request, OutOfMemory shutdown/exit, generic surface-acquire no-op, and non-surface render-error log
out of `app_handler.rs` while preserving the successful-present path, renderdoc capture end,
present-finish dispatch, and hitch summary orchestration.

Marker summary: redraw present error owner; surface acquire recovery; timeout redraw retry;
out-of-memory exit; app-handler present-error dispatch.

Projection marker: redraw-time present error recovery after submit.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_present_error.rs` owns
  `handle_window_redraw_present_error` and
  `clear_window_surface_after_present_acquire_failure`.
- The owner keeps `RenderError`, `SurfaceAcquireError`, `RunnerFrameDriveReason`,
  `ActiveEventLoop`, `AppWindowId`, `WinitRunner`, and `WinitAppDriver` at the recovery boundary.
- The owner preserves `SurfaceRecoverLost`, `SurfaceRecoverOutdated`, `SurfaceRecoverTimeout`,
  `self.raf_windows.request(app_window)`, `self.dispatcher.shutdown()`, `event_loop.exit()`, and
  `error!(?err, "render error")`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only
  `handle_window_redraw_present_error` dispatch after renderdoc capture end and before hitch
  summary, with `scene_ops` cached before releasing the redraw state borrow.

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
- Broader workspace gates were not run because M142 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time present error recovery source-auditable in a named owner while leaving
`app_handler.rs` responsible for redraw orchestration and hitch reporting. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
