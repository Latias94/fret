# M1 App-Owned Draft Actions Slice - 2026-04-24

Status: landed

## Summary

`editor_notes_demo.rs` now carries one inspector-local draft action row:

- `Draft actions` is rendered inside the existing `InspectorPanel` / `PropertyGrid` surface.
- `Mark draft ready` updates app-owned outcome/status feedback.
- `Clear draft marker` updates app-owned outcome/status feedback.
- The row uses stable test ids for both buttons.
- The status copy is explicitly local inspector state only.

## Boundaries Kept

- No persistence or filesystem save command.
- No workspace dirty-close prompt.
- No command bus, clipboard, or menu integration.
- No `TextField` draft-buffer API widening.
- No `fret-ui-kit::imui`, `fret-imui`, `fret-authoring`, or `crates/fret-ui` API changes.

## Evidence

- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/src/lib.rs`
