# M152 Runner Internal Drag Routing Owner Split - 2026-06-06

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner cross-window internal drag routing now lives in
`crates/fret-launch/src/runner/desktop/runner/internal_drag_routing.rs`. The split moves internal
drag pointer lookup, hover cancellation, runner-routed `InternalDrag` dispatch, cursor-based hover
selection, drop routing, under-moving-window diagnostics, and drag geometry diagnostics out of
`event_routing.rs` while preserving runtime behavior and public effect surfaces.

Marker summary: internal drag routing owner; cross-window docking hover routing; cursor-screen
position routing; under-moving-window diagnostics; runner-routed InternalDrag Enter/Over/Drop;
drop-to-source fallback; DockFloating follow-stop cleanup.

Projection marker: runner-routed InternalDrag Enter/Over/Drop for docking multi-window hand feel.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/internal_drag_routing.rs` owns
  `internal_drag_routing_pointer_id`, `clear_internal_drag_hover_if_needed`,
  `route_internal_drag_hover_from_cursor`, `route_internal_drag_drop_from_cursor`, and the private
  internal drag dispatch/geometry diagnostic helpers.
- `crates/fret-launch/src/runner/desktop/runner/event_routing.rs` keeps only ordinary window event
  delivery and platform completion dispatch.
- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares the new owner module.
- Source gates now anchor the `WindowClientOriginDiagnostics` / `local_pos_for_screen_pos`
  diagnostic markers in `internal_drag_routing.rs`, matching the new owner boundary.

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
python tools\check_layering.py
python tools\check_workstream_catalog.py
python tools\report_largest_files.py --top 30 --min-lines 800
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
- `python tools\check_layering.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `python tools\report_largest_files.py --top 30 --min-lines 800`: pass; neither
  `event_routing.rs` nor `internal_drag_routing.rs` appears in the top 30 >= 800-line report after
  the split.
- `git diff --check`: pass.
- Broader workspace gates were not run because M152 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package checks, targeted nextest, layering, source
  gates, and module-size guard cover this claim.

## Verdict

This keeps the multi-window docking hand-feel routing source-auditable in a named owner while
leaving `event_routing.rs` responsible for ordinary event delivery. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
