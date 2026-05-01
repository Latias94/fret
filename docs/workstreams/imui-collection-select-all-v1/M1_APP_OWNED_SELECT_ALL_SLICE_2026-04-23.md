# ImUi Collection Select-All v1 - M1 App-Owned Select-All Slice

Date: 2026-04-23
Status: landed

## Landed slice

1. The collection proof now supports one app-owned select-all shortcut slice.
2. Primary+A now selects all visible assets within the focused collection scope.
3. Select-all keeps the current active tile when possible instead of widening generic key-owner ownership.
4. The popup/menu surface stays unchanged in this lane.
5. No new public `fret-ui-kit::imui` collection select-all helper is admitted in this lane.

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_select_all_surface.rs`
- `tools/gate_imui_workstream_source.py`

## Notes

This slice intentionally stays inside `imui_editor_proof_demo`.

The point is to prove that:

- collection select-all pressure is still app-owned today,
- visible-order and active-tile policy can stay local to the product proof,
- and a better local architecture beats another premature generic helper.
