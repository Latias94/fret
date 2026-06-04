# M148 Runner Window Redraw Present Capture Commands Owner Split - 2026-06-05

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time present capture command assembly now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present_capture_commands.rs`. The split
moves engine/UI command-buffer assembly, diagnostic screenshot capture begin, bundle screenshot
readback begin, and bundle screenshot request-dir polling out of `app_handler.rs` while preserving
render-scene command recording, diagnostics capture internals, screenshot finish, present submit,
present finish, present error recovery, runtime behavior, and public effect surfaces.

Marker summary: redraw present capture commands owner; command buffer assembly; screenshot capture
begin; bundle screenshot readback begin; app-handler present-capture dispatch.

Projection marker: redraw-time present capture commands before submit.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_present_capture_commands.rs` owns
  `WindowRedrawPresentCaptureCommandsInput`, `WindowRedrawPresentCaptureCommands`, and
  `prepare_window_redraw_present_capture_commands`.
- The owner appends `ui_cmd` to engine command buffers, dispatches
  `begin_window_redraw_diag_screenshot_capture`, polls `DiagBundleScreenshotCapture::poll_request_dir`,
  and dispatches `begin_window_redraw_bundle_screenshot_readback`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only present-capture owner
  dispatch before `submit_window_redraw_present_frame`.
- `crates/fret-launch/src/runner/desktop/runner/window_redraw_diag_screenshots.rs` still owns
  capture/readback internals and finish-time screenshot handling.

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
- Broader workspace gates were not run because M148 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time present capture command assembly source-auditable in a named owner while
leaving `app_handler.rs` responsible for redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
