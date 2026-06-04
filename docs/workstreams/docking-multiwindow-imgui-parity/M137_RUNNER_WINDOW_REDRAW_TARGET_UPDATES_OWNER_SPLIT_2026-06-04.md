# M137 Runner Window Redraw Target Updates Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time render-target update application now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_target_updates.rs`. The split moves
`EngineFrameUpdate.target_updates` application, `RenderTargetUpdate::Update`,
`RenderTargetUpdate::Unregister`, `renderer.update_render_target(...)`, and
`renderer.unregister_render_target(...)` out of `app_handler.rs` while preserving ordering after
webview sync and before present orchestration.

Ordering marker: ordering after webview sync and before present orchestration.

Marker summary: redraw target updates owner; render-target update application; app-handler target updates dispatch.

Projection marker: redraw-time render-target update application before present.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_target_updates.rs` owns
  `apply_window_redraw_target_updates`.
- The owner keeps field-level input for `Renderer` and the `Vec<RenderTargetUpdate>` produced by the
  engine frame, so `WindowRuntime` ownership does not widen.
- The owner applies update/unregister deltas and preserves the existing unknown render-target error diagnostics.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the target-updates owner
  dispatch after webview sync and continues to own present orchestration, render-scene submission,
  frame presentation, diagnostics publication, and surface recovery.

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
- Broader workspace gates were not run because M137 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time render-target update application source-auditable in a named owner while
leaving `app_handler.rs` responsible for redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
