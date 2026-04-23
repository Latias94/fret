# ImUi Collection Rename v1 - M1 App-Owned Rename Slice

Date: 2026-04-23
Status: landed

## Landed slice

1. The collection proof now supports one app-owned rename slice.
2. F2 and the existing context-menu entry now open one app-owned rename modal for the active collection asset.
3. Committing rename updates the visible label while preserving stable asset ids and collection order.
4. The popup stays product-owned and uses the existing input/popup seams instead of widening `fret-ui-kit::imui`.
5. No new public `fret-ui-kit::imui` collection rename helper is admitted in this lane.

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_rename_surface.rs`
- `apps/fret-examples/src/lib.rs`

## Notes

This slice intentionally stays inside `imui_editor_proof_demo`.

The point is to prove that:

- collection rename pressure is still app-owned today,
- the existing popup/input seams already cover the needed product depth,
- and a better local architecture beats another premature generic helper.
