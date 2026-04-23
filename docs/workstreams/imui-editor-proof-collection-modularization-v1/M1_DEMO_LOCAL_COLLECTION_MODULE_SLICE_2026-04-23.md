# ImUi Editor Proof Collection Modularization v1 - M1 Demo-Local Collection Module Slice

Date: 2026-04-23
Status: landed

## Landed slice

1. The collection proof now lives in one demo-local `collection.rs` module under `imui_editor_proof_demo`.
2. The host file now routes collection rendering through `collection::render_collection_first_asset_browser_proof(ui)` and uses `collection::authoring_parity_collection_assets()` for the drag-chip seed set.
3. The module render entry intentionally binds to `KernelApp` authoring-surface helpers instead of widening the extracted function back to a generic `UiHost` seam.
4. Collection unit tests now live beside the module and the new modularization surface test freezes the host/module boundary explicitly.
5. Existing collection surface tests now read `collection.rs` for behavior anchors instead of pretending the host still owns the implementation inline.
6. No new public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API is admitted in this lane.

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/tests/imui_editor_collection_modularization_surface.rs`
- `apps/fret-examples/src/lib.rs`

## Notes

This slice intentionally stays demo-local.

The point is to prove that:

- collection maintenance pressure should first be solved inside the owning proof,
- modularization can improve reviewability without manufacturing a new framework seam,
- and the next default follow-on should now move back to product breadth instead of more file-size-driven cleanup.
