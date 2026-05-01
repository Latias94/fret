# ImUi Collection Command Package v1 - M1 App-Owned Duplicate Command Slice

Date: 2026-04-23
Status: landed slice

## Landed slice

1. The collection proof now lands the first app-owned command-package slice with `Primary+D` duplicate-selected.
2. The same duplicate command now routes through keyboard, the explicit button, and the collection context menu.
3. Duplicate results now reselect the copied set, preserve an active copied tile when possible, and publish app-owned command status feedback.
4. Unit tests plus a dedicated command-package surface test now freeze the duplicate-selected owner path explicitly.
5. No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/tests/imui_editor_collection_command_package_surface.rs`
- `tools/gate_imui_workstream_source.py`

## Notes

This is intentionally only the first command-package slice.

The lane stays active until we decide whether the current broader command package is already
coherent enough to close or whether one more narrow verb is still warranted.
