# M149 Runner Window Redraw Present Owner Split - 2026-06-05

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time present-phase orchestration now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present.rs`. The split moves
surface frame acquisition, clear-color resolution, render-scene command recording, post-render
diagnostics, present capture command assembly, queue submission, surface frame presentation,
successful present finish, screenshot/readback finish dispatch, and `RedrawPhase::Present` timing out of
`app_handler.rs` while preserving redraw prepare, app render, engine frame record, webview sync,
render-target update application, RenderDoc capture bracketing, present error recovery, hitch
summary reporting, runtime behavior, and public effect surfaces.

Marker summary: redraw present-phase owner; surface acquire; clear-color dispatch; render-scene dispatch; post-render diagnostics dispatch; present capture commands dispatch; submit dispatch; present finish dispatch; present timing return; app-handler present owner dispatch.

Projection marker: redraw-time present-phase orchestration after render-target updates and before
present error recovery.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_present.rs` owns
  `WindowRedrawPresentInput` and `present_window_redraw_frame`.
- The owner measures `RedrawPhase::Present`, acquires/prepares the surface target, resolves the
  clear color, records the UI render-scene command buffer, publishes post-render diagnostics,
  prepares present capture commands, submits/presents the frame, and finishes successful present.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only redraw-time present owner
  dispatch plus present error handling, RenderDoc capture end, and hitch-summary orchestration.
- Existing child owners still own their focused internals:
  `window_redraw_present_target.rs`, `window_redraw_render_scene.rs`,
  `window_redraw_post_render_diagnostics.rs`, `window_redraw_present_capture_commands.rs`,
  `window_redraw_present_submit.rs`, and `window_redraw_present_finish.rs`.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo check -p fret-launch --features diag-screenshots --lib
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
- `cargo check -p fret-launch --features diag-screenshots --lib`: pass.
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
- Broader workspace gates were not run because M149 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package checks, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time present-phase orchestration source-auditable in a named owner while leaving
`app_handler.rs` responsible for winit redraw scheduling, event-loop recovery, and frame-level hitch
reporting. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
