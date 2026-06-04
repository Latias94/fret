# M139 Runner Window Redraw Render Scene Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time render-scene command recording now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_render_scene.rs`. The split moves
`measure_redraw_phase(RedrawPhase::RenderScene, ...)`, `renderer.render_scene(...)`, and
`RenderSceneParams` construction out of `app_handler.rs` while preserving ordering after present
target preparation and before diagnostics publication, command submission, and `frame.present()`.

Ordering marker: ordering after present target preparation and before diagnostics publication.

Marker summary: redraw render scene owner; UI command buffer recording; app-handler render-scene dispatch.

Projection marker: redraw-time render-scene command recording before diagnostics and submit.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_render_scene.rs` owns
  `WindowRedrawRenderSceneInput` and `record_window_redraw_render_scene`.
- The owner keeps field-level input for `Renderer`, `WgpuContext`, `SurfaceState`, target view,
  scene, clear color, and scale factor so `WindowRuntime` ownership does not widen.
- The owner preserves `RedrawPhase::RenderScene`, `renderer.render_scene(...)`,
  `RenderSceneParams`, surface format/size reads, and UI command buffer return semantics.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the render-scene owner
  dispatch and continues to own text diagnostics publication, renderer perf/wgpu reports,
  screenshot capture/readback, command-buffer aggregation, queue submission, `frame.present()`,
  surface recovery, and hitch summary orchestration.

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
- Broader workspace gates were not run because M139 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time render-scene command recording source-auditable in a named owner while
leaving `app_handler.rs` responsible for redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
