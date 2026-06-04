# M141 Runner Window Redraw Present Finish Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner successful redraw-time present finish now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present_finish.rs`. The split moves
`commit_presented_frame_for_window`, `drop(engine_keepalive)`,
`finish_window_redraw_diag_screenshot_capture`, and
`finish_window_redraw_bundle_screenshot_readback` out of `app_handler.rs` while preserving ordering
after command submission and surface frame presentation, and before the present phase returns `Ok`.
Surface recovery, out-of-memory handling, and hitch summary orchestration stay in `app_handler.rs`.

Marker summary: redraw present finish owner; frame-id commit; engine keepalive release;
diagnostic screenshot finish; app-handler present-finish dispatch.

Projection marker: redraw-time successful present finish after submit.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_present_finish.rs` owns
  `WindowRedrawPresentFinishInput` and `finish_window_redraw_present_frame`.
- The owner keeps field-level input for `App`, `FrameId`, `AppWindowId`,
  `Vec<EngineFrameKeepalive>`, `DiagBundleScreenshotCapture`, `wgpu::Device`,
  `WindowRedrawBundleScreenshotReadback`, and `wgpu::TextureFormat` so `WinitRunner` ownership does
  not widen.
- Under `diag-screenshots`, the owner also carries `DiagScreenshotCapture` and `InFlightCapture`
  through the finish boundary.
- The owner preserves `commit_presented_frame_for_window`, `drop(input.keepalive)`,
  `finish_window_redraw_diag_screenshot_capture`, and
  `finish_window_redraw_bundle_screenshot_readback`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the present-finish owner
  dispatch and continues to own present-target acquisition, render-scene recording, diagnostics
  publication, present-submit dispatch, surface recovery, and hitch summary orchestration.

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
- Broader workspace gates were not run because M141 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps successful present finish source-auditable in a named owner while leaving
`app_handler.rs` responsible for redraw orchestration and recovery. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
