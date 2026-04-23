# ImUi Collection Keyboard Owner v1 - M1 App-Owned Keyboard Owner Slice

Status: landed bounded slice
Date: 2026-04-22

## Goal

Land one collection-scope keyboard-owner slice on the existing collection-first proof surface
without widening `fret-ui-kit::imui` or `crates/fret-ui`.

## Landed surface

1. `apps/fret-examples/src/imui_editor_proof_demo.rs` now keeps an explicit
   `ProofCollectionKeyboardState` model for the current active tile.
2. The collection scope now owns a focusable keyboard region locally in the proof demo.
3. `Arrow` / `Home` / `End` now move the active tile in visible collection order and replace the
   selected set app-locally.
4. `Shift+Arrow` / `Shift+Home` / `Shift+End` now extend the selected range from the current
   anchor app-locally.
5. `Escape` now clears the selected set while keeping the current keyboard location app-defined.
6. Background focus, background box-select, direct tile clicks, and selected-set drag/drop all
   stay explicit in the proof surface instead of becoming a new shared helper.

## Explicit rejects

1. No new public `fret-ui-kit::imui` collection keyboard-owner helper is admitted in this lane.
2. No new generic `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale facade is admitted here.
3. No lasso / freeform drag-rectangle policy is admitted in this lane.
4. No collection delete/select-all/action-command policy is admitted in this lane.
5. No runtime contract changes were needed in `crates/fret-ui`.

## Gates tied to this slice

- `cargo nextest run -p fret-examples --test imui_editor_collection_keyboard_owner_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_keyboard_owner_follow_on proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile proof_collection_keyboard_shift_navigation_extends_range_from_anchor proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile proof_collection_keyboard_ignores_primary_modifier_shortcuts --no-fail-fast`

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_keyboard_owner_surface.rs`
- `apps/fret-examples/src/lib.rs`
