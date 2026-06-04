# M129 Runner Window Redraw WGPU Allocator Report Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner redraw-time WGPU allocator report publication now lives in
`crates/fret-launch/src/runner/desktop/runner/window_redraw_wgpu_allocator_report.rs`. The split
moves `FRET_DIAG_WGPU_ALLOCATOR_REPORT` gating, allocator report cadence parsing, top-N and
max-name-byte configuration, `context.device.generate_allocator_report`, macOS Metal allocated-size
sampling, and `WgpuAllocatorReportFrameStore` recording out of `app_handler.rs` while preserving
redraw ordering after WGPU hub diagnostics and before command-buffer submission.

Marker summary: redraw WGPU allocator report owner; allocator report sample publication;
app-handler dispatch only.

Projection marker: allocator report sample publication.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_redraw_wgpu_allocator_report.rs` owns
  `maybe_record_window_redraw_wgpu_allocator_report`.
- The owner explicitly receives `app`, `context`, `app_window`, `tick_id`, and `frame_id`, so it
  does not borrow the whole redraw state while `surface` remains mutably borrowed.
- It owns `FRET_DIAG_WGPU_ALLOCATOR_REPORT`,
  `FRET_DIAG_WGPU_ALLOCATOR_REPORT_EVERY_N_FRAMES`,
  `FRET_DIAG_WGPU_ALLOCATOR_REPORT_TOP_N`,
  `FRET_DIAG_WGPU_ALLOCATOR_REPORT_MAX_NAME_BYTES`, `frame_id.is_multiple_of(every_n)`,
  `context.device.generate_allocator_report`, `as_hal::<wgpu::hal::api::Metal>()`,
  `metal_current_allocated_size_bytes`, `WgpuAllocatorReportFrameStore::default`, and
  `store.record_sample`.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` keeps only redraw-time WGPU
  allocator report dispatch after hub report diagnostics and before command-buffer submission.
- WGPU hub report publication remains in `window_redraw_wgpu_report.rs`; M129 does not widen into
  general redraw orchestration or renderer/device diagnostics.

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
- Broader workspace gates were not run because M129 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps redraw-time WGPU allocator report publication source-auditable in a named owner while
leaving `app_handler.rs` as dispatch plus redraw orchestration. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
