# ImUi Collection Context Menu v1 - M1 App-Owned Context Menu Slice

Status: landed bounded slice
Date: 2026-04-23

## Goal

Land one collection-scope context-menu slice on the existing collection-first proof surface
without widening `fret-ui-kit::imui` or `crates/fret-ui`.

## Landed surface

1. The collection proof now supports one shared popup scope for app-owned quick actions.
2. Right-click on an unselected asset now replaces selection with that asset before opening the popup.
3. Right-click on collection background now opens the same popup without widening helper surface.
4. The popup reuses the existing delete helper instead of forking collection action policy.
5. The quick-actions popup stays explicit in the proof demo instead of becoming a new collection
   helper surface.
6. No new public `fret-ui-kit::imui` collection context-menu helper is admitted in this lane.

## Explicit rejects

1. No new shared collection context-menu helper is admitted in `fret-ui-kit::imui`.
2. No new generic key-owner or menu-policy surface is admitted here.
3. No select-all, rename, or broader command palette integration is admitted in this lane.
4. No lasso / freeform drag-rectangle policy is admitted in this lane.
5. No runtime contract changes were needed in `crates/fret-ui`.

## Gates tied to this slice

- `cargo nextest run -p fret-examples --test imui_editor_collection_context_menu_surface --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-examples --lib proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile --no-fail-fast`

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_context_menu_surface.rs`
- `tools/gate_imui_workstream_source.py`
