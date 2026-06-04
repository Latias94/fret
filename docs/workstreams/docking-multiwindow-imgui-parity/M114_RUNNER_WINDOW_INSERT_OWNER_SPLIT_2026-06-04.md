# M114 Runner Window Insert Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner window insertion/bootstrap now lives in
`crates/fret-launch/src/runner/desktop/runner/window_insert.rs` instead of the
`window_lifecycle.rs` create owner. The split preserves `insert_window`, `WindowRuntime`
construction, `SurfaceState::new_with_usage`, initial composited-alpha surface configuration through
`want_surface_composited_alpha_for_style`, `apply_window_metrics_event` bootstrap, surface config
diagnostic snapshots through `record_surface_config_snapshot`, environment updates through
`update_window_environment_for_window_ref`, `window_registry.insert`, z-order bootstrap,
`record_window_open` diagnostics, OS menu registration, `RunnerFrameDriveReason::SurfaceBootstrap`,
and `raf_windows.request`.

Marker summary: window insertion bootstrap; metrics bootstrap; redraw bootstrap; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_insert.rs` owns `insert_window`,
  `WindowRuntime` construction, optional surface setup, metrics bootstrap, window registry
  insertion, z-order bootstrap, menu registration, lifecycle diagnostics, and initial redraw/RAF
  bootstrap.
- `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` keeps OS window creation and
  create-request orchestration, then delegates insertion to `insert_window`.
- Existing call sites in `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` and
  `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` continue to call the same private
  runner method.

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
- Broader workspace gates were not run because M114 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner insertion/bootstrap source-auditable in a dedicated window insert owner
and leaves `window_lifecycle.rs` focused on OS window creation and create-request orchestration. It
does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland
compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
