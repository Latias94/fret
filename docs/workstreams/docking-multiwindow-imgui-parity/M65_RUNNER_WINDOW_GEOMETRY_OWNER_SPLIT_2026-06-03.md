# M65 Runner Window Geometry Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner window geometry/chrome request application now lives in
`crates/fret-launch/src/runner/desktop/runner/window_geometry.rs` instead of the general effect
dispatcher. The split preserves visible state, inner-size application, outer-position application,
raise/fronting behavior, and OS-native drag/resize requests.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod window_geometry;`.
- `crates/fret-launch/src/runner/desktop/runner/window_geometry.rs` owns:
  - `apply_window_visibility_request`,
  - `apply_window_inner_size_request`,
  - `apply_window_outer_position_request`,
  - `apply_window_raise_request`,
  - `begin_window_drag_request`,
  - `begin_window_resize_request`.
- `apply_window_inner_size_request` keeps resize convergence behavior by falling back to the current
  surface size when the platform applies a resize without returning a size, then syncing the surface
  and scheduling the resize redraw.
- `apply_window_outer_position_request` keeps the Windows absolute-physical conversion for
  deterministic multi-monitor scripted placement and keeps non-Windows logical positioning.
- `apply_window_raise_request` keeps platform fronting and the macOS deferred-front enqueue path.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic
  `WindowRequest` effect branches, but geometry/chrome branches now delegate directly to
  `window_geometry.rs` helpers.

## Commands Run

```powershell
cargo fmt --package fret-launch -- --check
cargo check -p fret-launch --lib
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt --package fret-launch -- --check`: pass.
- `cargo check -p fret-launch --lib`: pass.
- `cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps window geometry/chrome request policy source-auditable without changing runtime behavior. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
