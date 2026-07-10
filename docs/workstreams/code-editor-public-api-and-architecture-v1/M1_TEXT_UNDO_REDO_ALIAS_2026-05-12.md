# M1 Text Undo Redo Alias

Status: Superseded by the canonical edit-command surface (2026-07-10)
Date: 2026-05-12

## Decision

This historical slice introduced `text.undo` and `text.redo` as editor-local aliases while also
accepting `edit.undo` and `edit.redo`.

The alias design is now superseded. ADR 0044 and ADR 0127 define `edit.undo` / `edit.redo` as the
single user-facing identity; focus routing decides whether the editor-local history or the
document/window fallback handles it. The code editor no longer accepts `text.undo` / `text.redo`.

## Must-Be-True Outcomes

- `edit.undo` and `edit.redo` operate on the editor-local `UndoHistory<CodeEditorTx>` when the
  editor owns the focused route.
- `text.undo` and `text.redo` return `NotHandled`; no compatibility route remains.
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
cargo nextest run -p fret-code-editor edit_undo_redo --lib --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor command_availability --lib --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python -m json.tool docs/workstreams/code-editor-public-api-and-architecture-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
python tools/check_layering.py
```
