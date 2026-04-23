# ImUi Collection Zoom v1 - M1 App-Owned Zoom/Layout Slice

Date: 2026-04-23
Status: landed

## Landed slice

1. The collection proof now derives layout metrics from viewport width plus app-owned zoom state.
2. Primary+Wheel now adjusts tile extent without widening generic IMUI helper ownership.
3. Keyboard grid navigation now reads the derived layout columns instead of a frozen constant.
4. The zoom slice reuses the existing child-region scroll handle to keep hovered rows anchored while columns change.
5. No new public `fret-ui-kit::imui` collection zoom helper is admitted in this lane.

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_zoom_surface.rs`
- `apps/fret-examples/src/lib.rs`

## Notes

This slice intentionally stays inside `imui_editor_proof_demo`.

The point is to prove that:

- collection zoom/layout pressure is still app-owned today,
- the frozen proof-budget rule remains intact,
- and a better local architecture beats another premature generic helper.
