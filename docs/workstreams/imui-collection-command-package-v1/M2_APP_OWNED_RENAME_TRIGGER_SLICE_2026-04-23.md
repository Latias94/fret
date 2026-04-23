# ImUi Collection Command Package v1 - M2 App-Owned Rename Trigger Slice

Date: 2026-04-23
Status: landed slice

## Landed slice

1. The collection proof now routes the existing inline rename command through an explicit
   `Rename active asset` button in addition to `F2` and the collection context menu.
2. The explicit button stays app-owned inside
   `apps/fret-examples/src/imui_editor_proof_demo/collection.rs` and reuses the same proof-local
   inline rename session/status models instead of inventing a shared command helper.
3. Button and context-menu rename activation now share one demo-local app helper for the render
   path, while the keyboard owner path stays local to the collection proof.
4. No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/tests/imui_editor_collection_command_package_surface.rs`
- `apps/fret-examples/tests/imui_editor_collection_rename_surface.rs`
- `apps/fret-examples/src/lib.rs`

## Notes

This slice deliberately broadens trigger parity for an existing rename flow rather than adding a
new product command such as clipboard, reveal-in-finder, or platform-integrated open/reveal verbs.

The lane remains active until we decide whether duplicate + rename trigger breadth is coherent
enough to close or whether one more narrow app-owned verb is still warranted before the next
default non-multi-window priority shifts to the second proof surface.
