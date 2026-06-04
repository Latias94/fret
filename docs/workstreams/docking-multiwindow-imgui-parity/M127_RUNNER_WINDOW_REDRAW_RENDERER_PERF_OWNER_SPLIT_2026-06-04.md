# M127 Runner Window Redraw Renderer Perf Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time renderer perf sample publication now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_renderer_perf.rs`. The split moves
`FRET_DIAG_RENDERER_PERF` gating, `Renderer::take_last_frame_perf_snapshot`,
`RendererPerfFrameSample` construction, `RendererPerfFrameStore` recording, and the
`driver.renderer_perf_sample` callback out of `app_handler.rs` while preserving redraw ordering
after scene rendering/text diagnostics and before WGPU diagnostics.

Marker summary: redraw renderer perf owner; renderer perf sample publication; app-handler dispatch only.

Ordering marker: redraw ordering after scene rendering/text diagnostics and before WGPU diagnostics.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_renderer_perf.rs` owns
  `maybe_publish_window_redraw_renderer_perf_sample`.
- The owner explicitly receives `app`, `driver`, `renderer`, `app_window`, `user`, `tick_id`, and
  `frame_id`, so it does not borrow the whole redraw state while `surface` remains mutably borrowed.
- It owns `FRET_DIAG_RENDERER_PERF`, `take_last_frame_perf_snapshot`,
  `RendererPerfFrameSample`, `RendererPerfFrameStore::default`, `store.record`, and
  `driver.renderer_perf_sample`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only redraw-time renderer
  perf dispatch after text diagnostics and before WGPU diagnostics.

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
- Broader workspace gates were not run because M127 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time renderer perf publication source-auditable in a named owner while leaving
`app_handler.rs` as dispatch plus redraw orchestration. It does not close `DW-P1-linux-003`; the
next true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
