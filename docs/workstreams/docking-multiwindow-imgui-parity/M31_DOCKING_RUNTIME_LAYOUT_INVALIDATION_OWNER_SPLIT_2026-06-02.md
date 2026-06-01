# M31 Docking Runtime Layout Invalidation Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split for DockOp post-mutation layout invalidation used
by the multi-window parity lane. It keeps `DW-P1-linux-003` open because real Wayland compositor
acceptance still requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime viewport layout invalidation is now owned by private `runtime/layout_invalidation.rs`
without changing DockOp orchestration, graph mutation, request/fallback behavior, window-created
completion, before-close merge-back behavior, empty-window auto-close behavior, public runtime hook
paths, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps public runtime hook routing, graph mutation, request
  routing, window-created routing, before-close routing, and empty-window auto-close routing.
  `handle_dock_op(...)` delegates post-mutation viewport cleanup to the private owner.
- `ecosystem/fret-docking/src/runtime/layout_invalidation.rs` owns
  `invalidate_after_dock_op(...)`, `invalidate_windows(...)`,
  `DockInvalidationService::bump_windows(...)`,
  `clear_viewport_layout_for_window(...)`, per-window viewport layout cleanup, and whole-graph
  invalidation for tab/split fraction changes.
- `ecosystem/fret-docking/src/runtime/before_close.rs` and
  `ecosystem/fret-docking/src/runtime/window_created.rs` reuse the same private invalidation owner
  instead of keeping a runtime-shell helper.
- `tools/gate_docking_multiwindow_workstream_source.py` rejects layout invalidation bodies drifting
  back into `runtime.rs`.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking request_float_creates_window_and_window_created_moves_panel request_float_degrades_to_in_window_when_multi_window_is_disabled request_float_degrades_to_in_window_when_tear_off_is_disabled request_float_degrades_to_in_window_when_window_hover_detection_is_none request_float_is_idempotent_until_window_created window_created_updates_drag_source_window_for_active_dock_drag window_created_updates_drag_source_window_for_active_dock_tabs_drag redock_from_dock_floating_window_auto_closes_empty_os_window before_close_window_merges_dock_floating_panels_into_target_window request_float_canceled_by_close_panel_closes_created_window window_created_does_not_update_drag_source_when_canceled --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- Eleven request/fallback/window-created/before-close/auto-close regressions: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps DockOp post-mutation layout invalidation source-auditable in the new private owner while
preserving the existing create, fallback, cancellation, window-created, before-close, auto-close,
and public runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event is
still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
