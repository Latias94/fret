# M32 Docking Runtime Test Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable test owner split for the docking runtime shell. It
keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5
runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime regression coverage is now owned by private `runtime/tests.rs` instead of an inline
`#[cfg(test)] mod tests { ... }` block in `runtime.rs`, without changing request, fallback,
window-created, before-close, auto-close, layout invalidation, public runtime hook behavior, or the
Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps the public runtime hook shell and declares
  `#[cfg(test)] mod tests;`.
- `ecosystem/fret-docking/src/runtime/tests.rs` owns the focused request/fallback/window-created/
  before-close/auto-close regression tests that were previously inline in `runtime.rs`.
- `tools/gate_docking_multiwindow_workstream_source.py` rejects runtime regression test bodies
  drifting back into `runtime.rs` and source-checks the new test owner.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking request_float_creates_window_and_window_created_moves_panel request_float_degrades_to_in_window_when_multi_window_is_disabled request_float_degrades_to_in_window_when_tear_off_is_disabled request_float_degrades_to_in_window_when_window_hover_detection_is_none request_float_is_idempotent_until_window_created window_created_updates_drag_source_window_for_active_dock_drag window_created_updates_drag_source_window_for_active_dock_tabs_drag window_created_prefers_pending_pointer_id_over_drag_source_window_match redock_from_dock_floating_window_auto_closes_empty_os_window before_close_window_merges_dock_floating_panels_into_target_window request_float_canceled_by_close_panel_closes_created_window window_created_does_not_update_drag_source_when_canceled --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
python tools\gate_imui_workstream_source.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- Twelve request/fallback/window-created/before-close/auto-close regressions: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `gate_imui_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps docking runtime regression coverage source-auditable in the new private test owner while
preserving the existing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure
event is still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
