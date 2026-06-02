# M27 Docking Runtime Window-Created Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split for DockFloating window-created completion used
by the multi-window parity lane. It keeps `DW-P1-linux-003` open because real Wayland compositor
acceptance still requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime DockFloating window-created completion is now owned by private
`runtime/window_created.rs` without changing DockOp orchestration, in-window fallback behavior,
create request emission, pending tear-off correlation/cancellation, created-window
cancel-and-close behavior, active drag source remapping, close-on-empty registry behavior, public
runtime hook paths, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps public runtime hook routing, DockOp orchestration,
  in-window fallback dispatch, close-on-empty handling, and before-close merge behavior.
  `handle_dock_window_created(...)` remains the public facade and delegates to the private owner.
- `ecosystem/fret-docking/src/runtime/window_created.rs` owns
  `complete_for_create_request(request, now)`,
  `DockTearOffCompletion::CancelAndCloseWindow`, panel/tabs graph update via
  `float_panel_to_window` and `float_tabs_to_window`, active drag `source_window`/`current_window`
  remapping, invalidation, and DockFloating registry registration.
- `ecosystem/fret-docking/src/runtime/tear_off.rs` remains the private owner for registry storage
  and pending tear-off state/correlation.
- `tools/gate_docking_multiwindow_workstream_source.py` rejects window-created completion bodies
  drifting back into `runtime.rs`.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking --lib -E 'test(request_float_creates_window_and_window_created_moves_panel) or test(window_created_updates_drag_source_window_for_active_dock_drag) or test(window_created_updates_drag_source_window_for_active_dock_tabs_drag) or test(window_created_prefers_pending_pointer_id_over_drag_source_window_match) or test(window_created_does_not_update_drag_source_when_canceled)' --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- Five window-created and drag remap regressions: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps DockFloating window-created completion source-auditable in the new private owner while
preserving the existing create, cancellation, fallback, and close-on-empty behavior. It does not close
`DW-P1-linux-003`; the next true closure event is still a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
