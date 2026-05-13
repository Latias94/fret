# M1 Text Undo Redo Alias

Status: Landed
Date: 2026-05-12

## Decision

The code editor now accepts `text.undo` and `text.redo` for editor-local text history while still
accepting the existing `edit.undo` and `edit.redo` command ids.

This aligns the editor with ADR 0127's text-surface local-history vocabulary without breaking the
existing focused command route. `edit.undo` / `edit.redo` remain accepted as transitional
window/document routing aliases until a later command metadata pass can split app-owned document
history from widget-local history more precisely.

## Must-Be-True Outcomes

- `text.undo` and `text.redo` operate on the editor-local `UndoHistory<CodeEditorTx>`.
- `edit.undo` and `edit.redo` continue to work for the existing focused route.
- Availability answers are symmetric across the text and edit undo/redo ids.
- Read-only editors block local undo/redo availability.
- Command execution logic is testable outside the render closure.

## Evidence Anchors

- Command dispatch helper: `ecosystem/fret-code-editor/src/editor/input/keyboard.rs`
- Render hook wiring: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Unit coverage: `ecosystem/fret-code-editor/src/editor/tests/keyboard_commands.rs`
- Boundary decision: `M1_COMMAND_KEYMAP_UNDO_BOUNDARY_2026-05-12.md`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor text_undo_redo --lib --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor command_availability --lib --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python -m json.tool docs/workstreams/code-editor-public-api-and-architecture-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
python tools/check_layering.py
```
