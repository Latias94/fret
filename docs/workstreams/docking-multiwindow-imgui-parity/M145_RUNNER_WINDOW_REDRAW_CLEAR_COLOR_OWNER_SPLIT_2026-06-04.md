# M145 Runner Window Redraw Clear Color Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time clear-color selection now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_clear_color.rs`. The split moves the
transparent-window clear-color decision out of `app_handler.rs` while preserving window style
diagnostics storage, present target acquisition, render-scene command recording, screenshot capture,
submit/finish ordering, runtime behavior, and public effect surfaces.

Marker summary: redraw clear color owner; visual transparent selection; app-handler clear-color dispatch.

Evidence marker: window style diagnostics storage; configured clear color fallback.

Projection marker: redraw-time clear color selection before render-scene recording.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_clear_color.rs` owns
  `resolve_window_redraw_clear_color`.
- The owner keeps the `RunnerWindowStyleDiagnosticsStore` lookup, `effective_snapshot(app_window)`,
  `visual_transparent`, transparent `ClearColor(wgpu::Color::TRANSPARENT)`, and configured clear color
  fallback at the redraw clear-color lifecycle boundary.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only clear-color owner
  dispatch before `record_window_redraw_render_scene`.
- `crates/fret-launch/src/runner/desktop/runner/window_redraw_render_scene.rs` still owns
  render-scene command recording with the already-selected clear color.

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
- Broader workspace gates were not run because M145 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time clear-color selection source-auditable in a named owner while leaving
`app_handler.rs` responsible for redraw orchestration. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
