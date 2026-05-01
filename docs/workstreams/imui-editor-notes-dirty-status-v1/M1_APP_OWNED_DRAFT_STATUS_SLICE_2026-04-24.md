# M1 App-Owned Draft Status Slice - 2026-04-24

Status: landed

## Summary

`editor_notes_demo.rs` now carries one inspector-local draft status row:

- `Draft status` is rendered inside the existing `InspectorPanel` / `PropertyGrid` surface.
- The row uses stable test id `editor-notes-demo.inspector.notes.draft-status`.
- The status is derived from the existing notes outcome plus committed line-count label.
- `Committed` reads as clean draft feedback.
- `Canceled` reads as preserved editor text feedback.
- Any other outcome reads as preserved-until-commit feedback.

## Boundaries Kept

- No workspace dirty-close prompt.
- No save/persistence command.
- No generic document-state model.
- No generic inspector, command, clipboard, or IMUI helper API.
- No multi-window runner behavior.

## Evidence

- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `tools/gate_imui_workstream_source.py`
