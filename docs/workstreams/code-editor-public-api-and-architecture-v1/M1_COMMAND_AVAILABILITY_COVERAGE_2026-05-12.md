# M1 Command Availability Coverage

Status: Landed
Date: 2026-05-12

## Decision

Extend the code editor focused command availability surface beyond `select_all`.

The editor now reports availability for the commands it already handles:

- `edit.undo`
- `edit.redo`
- `edit.select_all`
- `edit.copy`
- `edit.cut`
- `edit.paste`
- `text.move_word_left`
- `text.move_word_right`
- `text.select_word_left`
- `text.select_word_right`

Availability is a pure query and considers:

- enabled/focusable/selectable/editable interaction state,
- current selection,
- buffer emptiness,
- local undo/redo state,
- clipboard read/write platform capabilities.

Unknown commands remain `NotHandled` so workspace or app-level command owners can answer them.
The retired `text.undo/redo/copy/cut/paste/select_all` aliases are also `NotHandled`.

## Must-Be-True Outcomes

- Menu/palette/keymap availability no longer only sees `select_all` for the code editor.
- Read-only editors still expose non-mutating copy/navigation availability but block mutations.
- Clipboard commands reflect platform read/write capabilities.
- Undo/redo availability reflects the editor-local undo stack without promoting editor history into
  framework-global undo.
- Availability logic is testable without launching a UI tree.

## Evidence Anchors

- Availability implementation: `ecosystem/fret-code-editor/src/editor/input/keyboard.rs`
- Render hook wiring: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Unit coverage: `ecosystem/fret-code-editor/src/editor/tests/keyboard_commands.rs`
- Boundary decision: `M1_COMMAND_KEYMAP_UNDO_BOUNDARY_2026-05-12.md`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor command_availability --lib --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python -m json.tool docs/workstreams/code-editor-public-api-and-architecture-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
python tools/check_layering.py
```
