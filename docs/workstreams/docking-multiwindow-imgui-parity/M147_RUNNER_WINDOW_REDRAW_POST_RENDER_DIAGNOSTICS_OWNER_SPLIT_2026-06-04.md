# M147 Runner Window Redraw Post-Render Diagnostics Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time post-render diagnostics dispatch now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_post_render_diagnostics.rs`. The split
moves text diagnostics publishing, renderer perf sample dispatch, WGPU hub report dispatch, and
WGPU allocator report dispatch out of `app_handler.rs` while preserving render-scene command
recording, command-buffer assembly, screenshot capture, submit/finish ordering, diagnostics
environment gates, runtime behavior, and public effect surfaces.

Marker summary: redraw post-render diagnostics owner; text diagnostics dispatch; renderer perf and
wgpu reports; app-handler post-render diagnostics dispatch.

Evidence marker: diagnostics environment gates.

Projection marker: redraw-time post-render diagnostics before command buffer submission.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_post_render_diagnostics.rs` owns
  `WindowRedrawPostRenderDiagnosticsInput` and
  `publish_window_redraw_post_render_diagnostics`.
- The owner dispatches to `publish_window_redraw_text_diagnostics`,
  `maybe_publish_window_redraw_renderer_perf_sample`,
  `maybe_record_window_redraw_wgpu_hub_report`, and
  `maybe_record_window_redraw_wgpu_allocator_report`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only post-render diagnostics
  owner dispatch after `record_window_redraw_render_scene` and before command-buffer assembly.
- The underlying diagnostics modules still own their environment gates, sampling cadence, snapshot
  construction, report construction, and global-store writes.

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
- Broader workspace gates were not run because M147 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time post-render diagnostics dispatch source-auditable in a named owner while
leaving `app_handler.rs` responsible for redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
