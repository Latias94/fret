# ImUi Collection Inline Rename v1 - M1 App-Owned Inline Rename Slice

Date: 2026-04-23
Status: landed

## Landed slice

1. The collection proof now supports one app-owned inline rename slice.
2. F2 and the existing context-menu entry now start one app-owned inline rename editor for the active collection asset.
3. The inline editor uses `TextField` plus a proof-local focus handoff instead of widening `fret-ui-kit::imui`.
4. Committing rename still updates the visible label while preserving stable asset ids and collection order.
5. No new public `fret-ui-kit::imui` inline-edit or collection rename helper is admitted in this lane.

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_rename_surface.rs`
- `apps/fret-examples/src/lib.rs`

## Notes

This slice intentionally stays inside `imui_editor_proof_demo`.

The point is to prove that:

- inline collection rename pressure is still app-owned today,
- the existing editor-owned text-entry control is sufficient when embedded locally,
- and a better local architecture beats another premature shared helper.
