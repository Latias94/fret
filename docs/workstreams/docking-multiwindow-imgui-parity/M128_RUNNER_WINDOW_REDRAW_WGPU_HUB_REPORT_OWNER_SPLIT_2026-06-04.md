# M128 Runner Window Redraw WGPU Hub Report Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time WGPU hub report publication now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_wgpu_report.rs`. The split moves
`FRET_DIAG_WGPU_REPORT` gating, report cadence parsing, `context.instance.generate_report`, hub
count projection, and `WgpuHubReportFrameStore` recording out of `app_handler.rs` while preserving
redraw ordering after renderer perf diagnostics and before WGPU allocator diagnostics.

Marker summary: redraw WGPU hub report owner; hub report count publication; app-handler dispatch only.

Projection marker: hub count projection.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_wgpu_report.rs` owns
  `maybe_record_window_redraw_wgpu_hub_report`.
- The owner explicitly receives `app`, `context`, `app_window`, `tick_id`, and `frame_id`, so it
  does not borrow the whole redraw state while `surface` remains mutably borrowed.
- It owns `FRET_DIAG_WGPU_REPORT`, `FRET_DIAG_WGPU_REPORT_EVERY_N_FRAMES`,
  `frame_id.is_multiple_of(every_n)`, `context.instance.generate_report`, `hub_report`,
  `WgpuHubReportCounts`, `WgpuHubReportFrameStore::default`, and `store.record`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only redraw-time WGPU hub
  report dispatch after renderer perf diagnostics and before allocator diagnostics.
- WGPU allocator report publication intentionally remains in `app_handler.rs` for a separate owner
  split because it has additional top-N/max-name configuration and macOS HAL sampling.

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
- Broader workspace gates were not run because M128 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time WGPU hub report publication source-auditable in a named owner while leaving
`app_handler.rs` as dispatch plus redraw orchestration. It does not close `DW-P1-linux-003`; the
next true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
