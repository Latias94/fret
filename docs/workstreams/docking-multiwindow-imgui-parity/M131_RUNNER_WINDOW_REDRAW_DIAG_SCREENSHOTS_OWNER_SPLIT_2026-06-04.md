# M131 Runner Window Redraw Diag Screenshots Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time diagnostic screenshot capture and bundle screenshot readback now live in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_diag_screenshots.rs`. The split moves
feature-gated screenshot request polling, per-window diagnostic capture begin/finish, bundle
screenshot readback begin/finish, and capture failure logging out of `app_handler.rs` while
preserving the existing command-buffer enqueue, queue submit, frame present, and frame-id commit
ordering.

Marker summary: redraw diag screenshots owner; screenshot capture/readback lifecycle;
app-handler submit/present orchestration only.

Projection marker: screenshot capture/readback lifecycle.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_diag_screenshots.rs` owns
  `poll_window_redraw_diag_screenshot_requests`,
  `begin_window_redraw_diag_screenshot_capture`,
  `finish_window_redraw_diag_screenshot_capture`,
  `begin_window_redraw_bundle_screenshot_readback`, and
  `finish_window_redraw_bundle_screenshot_readback`.
- The owner receives `DiagScreenshotCapture`, `DiagBundleScreenshotCapture`, `AppWindowId`,
  `frame_view`, `wgpu::Device`, surface format/size, and the command-buffer vector by field-level
  parameter; it does not borrow the whole `WindowRuntime` or take over redraw orchestration.
- It owns `DiagScreenshotCapture::poll`, `begin_capture_for_window`, `finish_capture`,
  `DiagBundleScreenshotCapture::begin_readback`, `finish_and_write_bmp`, and the diagnostic
  capture failure warning.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only redraw-time dispatch
  into the owner plus `context.queue.submit`, `frame.present`, and frame-id commit ordering.
- About-to-wait screenshot polling in `diag_screenshots.rs` remains unchanged; M131 narrows only
  the redraw-time capture/readback path.

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
- Broader workspace gates were not run because M131 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time screenshot capture/readback lifecycle source-auditable in a named owner while
leaving `app_handler.rs` responsible for submit/present orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
