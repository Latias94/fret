# M134 Runner Window Redraw Frame Prepare Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time frame preparation now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_frame_prepare.rs`. The split moves
platform `prepare_frame`, surface-size-to-logical-bounds projection, logical pixel quantization,
scale-factor capture, and driver `gpu_frame_prepare` dispatch out of `app_handler.rs` while
preserving the existing ordering after pending surface resize handling and before render.

Marker summary: redraw frame prepare owner; platform frame preparation; app-handler prepare dispatch
only.

Projection marker: platform frame preparation.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_frame_prepare.rs` owns
  `WindowRedrawFramePrepareInput`, `prepare_window_redraw_frame`, and
  `window_redraw_frame_bounds`.
- The owner keeps field-level input for `App`, driver, user state, platform, window, GPU context,
  and renderer so `WindowRuntime` ownership does not widen.
- The owner keeps `measure_redraw_phase(RedrawPhase::Prepare, ...)`,
  `platform.prepare_frame`, `window.surface_size`, `quantize_logical_px`,
  `WindowRedrawFramePrepare { scale_factor, bounds }`, and `driver.gpu_frame_prepare` in the same
  frame-prepare sequence as the previous redraw path.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the frame-prepare owner
  dispatch and continues to own render, record, present, surface recovery, and hitch summary
  orchestration.

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
- Broader workspace gates were not run because M134 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time frame preparation source-auditable in a named owner while leaving
`app_handler.rs` responsible for redraw orchestration. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
