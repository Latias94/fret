# ImUi Collection Delete Action v1 - M1 App-Owned Delete Action Slice

Status: landed bounded slice
Date: 2026-04-22

## Goal

Land one collection-scope delete-selected slice on the existing collection-first proof surface
without widening `fret-ui-kit::imui` or `crates/fret-ui`.

## Landed surface

1. The collection proof now supports one app-owned delete-selected action slice.
2. `apps/fret-examples/src/imui_editor_proof_demo.rs` now keeps an explicit mutable collection
   asset model in addition to the selection and keyboard-owner models.
3. `Delete` / `Backspace` now remove the selected set from the stored asset model.
4. The explicit action button reuses the same delete helper instead of forking policy.
5. Remaining assets, selection, and keyboard active tile now reflow app-locally after deletion.
6. The delete-selected semantics stay reviewable in the proof surface instead of becoming a new
   shared helper or command facade.

## Explicit rejects

1. No new public `fret-ui-kit::imui` collection action helper is admitted in this lane.
2. No new generic key-owner or command-facade surface is admitted here.
3. No select-all, rename, or context-menu command breadth is admitted in this lane.
4. No lasso / freeform drag-rectangle policy is admitted in this lane.
5. No runtime contract changes were needed in `crates/fret-ui`.

## Gates tied to this slice

- `cargo nextest run -p fret-examples --test imui_editor_collection_delete_action_surface --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-examples --lib proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item proof_collection_delete_selection_picks_previous_visible_item_at_end --no-fail-fast`

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_delete_action_surface.rs`
- `tools/gate_imui_workstream_source.py`
