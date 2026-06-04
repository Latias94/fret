# M133 Runner Window Redraw Surface Resize Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time pending surface resize fallback now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_surface_resize.rs`. The split moves
`pending_surface_resize` draining, fallback `resize_surface`, per-frame logical size quantization,
`last_delivered_window_resized` deduplication, `Event::WindowResized` delivery, and
`Event::WindowScaleFactorChanged` delivery out of `app_handler.rs` while preserving the existing
ordering after pre-render drains/diagnostic screenshot polling and before platform frame
preparation.

Marker summary: redraw surface resize owner; pending surface resize fallback; app-handler dispatch
only.

Projection marker: pending surface resize fallback.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_surface_resize.rs` owns
  `handle_window_redraw_pending_surface_resize`.
- The owner keeps using `WindowRuntime::pending_surface_resize`, `resize_surface`,
  `quantize_logical_px`, `last_delivered_window_resized`, `WindowMetricsService::scale_factor`,
  `Event::WindowResized`, and `Event::WindowScaleFactorChanged` in the same sequence as the
  previous redraw path.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only
  `self.handle_window_redraw_pending_surface_resize(app_window);` before platform frame
  preparation.
- `surface_lifecycle.rs` continues to own immediate surface resize event handling and GPU surface
  synchronization; M133 narrows only the redraw-time eventual-consistency fallback and high-level
  delivery path.

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
- Broader workspace gates were not run because M133 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time surface resize fallback source-auditable in a named owner while leaving
`app_handler.rs` responsible for redraw orchestration. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
