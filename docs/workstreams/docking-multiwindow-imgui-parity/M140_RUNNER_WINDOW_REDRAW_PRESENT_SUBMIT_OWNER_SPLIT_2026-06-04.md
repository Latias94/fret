# M140 Runner Window Redraw Present Submit Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time command submission and surface frame presentation now live in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present_submit.rs`. The split moves
`context.queue.submit(cmd_buffers)` and `frame.present()` out of `app_handler.rs` while preserving
ordering after screenshot command-buffer mutation and before scheduling presented-frame commit,
engine keepalive drop, screenshot finish, surface recovery, and hitch summary orchestration.

Marker summary: redraw present submit owner; queue submit; surface frame present;
app-handler present-submit dispatch.

Projection marker: redraw-time command submission and surface frame presentation.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_present_submit.rs` owns
  `WindowRedrawPresentSubmitInput` and `submit_window_redraw_present_frame`.
- The owner keeps field-level input for `WgpuContext`, `Vec<wgpu::CommandBuffer>`, and
  `WindowRedrawPresentTarget` so `WindowRuntime` ownership does not widen.
- The owner preserves `input.context.queue.submit(input.command_buffers)` and
  `input.present_target.into_frame_view()` followed by `frame.present()`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only the present-submit owner
  dispatch and continues to own text diagnostics, renderer perf/wgpu reports, screenshot begin/finish,
  scheduling presented-frame commit, engine keepalive drop, surface recovery, and hitch summary
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
- Broader workspace gates were not run because M140 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time command submission and surface frame presentation source-auditable in a named
owner while leaving `app_handler.rs` responsible for redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
