# M28 Docking Runtime Before-Close Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split for DockFloating before-close merge-back policy
used by the multi-window parity lane. It keeps `DW-P1-linux-003` open because real Wayland
compositor acceptance still requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Docking runtime DockFloating before-close merge-back is now owned by private
`runtime/before_close.rs` without changing DockOp orchestration, in-window fallback behavior,
create request emission, pending tear-off correlation/cancellation, window-created completion,
close-on-empty registry behavior, public runtime hook paths, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/runtime.rs` keeps public runtime hook routing, DockOp orchestration,
  in-window fallback dispatch, close-on-empty handling, and window-created routing.
  `handle_dock_before_close_window(...)` remains the public facade and delegates to the private
  owner.
- `ecosystem/fret-docking/src/runtime/before_close.rs` owns DockFloating registry removal,
  `window_root(closing_window)` existence check, `first_tabs_in_window(target_window)` target tab
  lookup, `DockOp::MergeWindowInto`, `clear_viewport_layout_for_window(closing_window)`,
  `clear_viewport_layout_for_window(target_window)`, and target-window invalidation.
- `ecosystem/fret-docking/src/runtime/tear_off.rs` remains the private owner for registry storage
  and pending tear-off state/correlation.
- `tools/gate_docking_multiwindow_workstream_source.py` rejects before-close merge bodies drifting
  back into `runtime.rs`.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking --lib -E 'test(before_close_window_merges_dock_floating_panels_into_target_window)' --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- Focused before-close merge-back regression: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `check_workstream_catalog.py`: pass.
- `WORKSTREAM.json` shape and `git diff --check`: pass.

## Verdict

This keeps DockFloating before-close merge-back source-auditable in the new private owner while
preserving the existing create, cancellation, fallback, window-created, and close-on-empty behavior.
It does not close `DW-P1-linux-003`; the next true closure event is still a dated real Linux Wayland
compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
