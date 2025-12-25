# ADR 0044: Text Editing State Model and Core Commands

Status: Accepted

## Context

Fret targets editor-grade keyboard-driven UX. Text editing is a cross-cutting concern that impacts:

- shortcut arbitration (`KeyDown` vs `TextInput`),
- focus routing and key-repeat semantics,
- IME integration (cursor area feedback),
- clipboard and selection behavior.

If the text editing state model and command vocabulary are not locked early, downstream work (inspector fields,
search boxes, code editor, command palette, settings UI) tends to accumulate bespoke logic and forces rewrites.

References:

- Keyboard/IME split and suppression rules: `docs/adr/0012-keyboard-ime-and-text-input.md`
- Focus + command routing: `docs/adr/0020-focus-and-command-routing.md`
- Clipboard boundary: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Text system boundary (layout vs paint): `docs/adr/0006-text-system.md`

## Decision

### 1) Text editing state uses byte offsets (UTF-8) at char boundaries

The canonical internal representation for caret and selection is:

- `caret: usize` (byte offset into the UTF-8 buffer),
- `selection_anchor: usize` (byte offset; selection is the normalized range between anchor and caret).

Constraints:

- Offsets must always be clamped to valid UTF-8 **char boundaries**.
- Commands and events operate on these byte offsets.

Rationale:

- Byte offsets are stable for persistence and interop (clipboard, IME, external buffers).
- We can improve movement algorithms (grapheme clusters, emoji sequences, Unicode word breaks) later without
  changing the stored representation.

### 2) Core text editing commands are stable IDs

The `text.*` command namespace is reserved for focused text-editable widgets.

Baseline commands (single-line and multi-line widgets may share them):

- Selection / clipboard:
  - `text.select_all`
  - `text.copy`
  - `text.cut`
  - `text.paste`
- Navigation:
  - `text.move_left`, `text.move_right`
  - `text.move_word_left`, `text.move_word_right`
  - `text.move_home`, `text.move_end`
- Selection expansion:
  - `text.select_left`, `text.select_right`
  - `text.select_word_left`, `text.select_word_right`
  - `text.select_home`, `text.select_end`
- Deletion:
  - `text.delete_backward`, `text.delete_forward`
  - `text.delete_word_backward`, `text.delete_word_forward`

Semantics:

- Navigation commands collapse selection (set `selection_anchor = caret`).
- Selection expansion commands keep `selection_anchor` and move `caret`.
- Delete commands delete the selection if it is non-empty; otherwise delete by the unit implied by the command.
- `text.paste` requests clipboard text via effects (ADR 0041) and inserts at the caret, replacing selection.

### 3) Key repeat for editing is explicit and opt-in

Commands that represent continuous editing/navigation should be marked `repeatable` so platform key repeat
can re-dispatch them when held:

- all `text.move_*`, `text.select_*`, and `text.delete_*` commands.

This is a contract between the key dispatcher and command metadata (ADR 0020 / ADR 0023).

### 4) IME cursor area is anchored to the caret after layout/paint

Text-editable widgets must provide a caret rect in window logical coordinates so the platform backend can place
IME candidate windows near the caret:

- widget computes caret rect after layout/paint (based on current caret and preedit cursor),
- widget sends `Effect::ImeSetCursorArea` (ADR 0012),
- backend calls `window.set_ime_cursor_area(...)`.

## Consequences

- Text inputs, palettes, and inspector fields share a single command vocabulary and avoid bespoke key handling.
- Key-repeat behavior becomes predictable (only repeatable commands repeat).
- IME candidate windows can be positioned correctly for editor-grade UX.

## Implementation Notes (Current Workspace)

- TextInput baseline widget:
  - `crates/fret-ui/src/primitives/text.rs`
- Repeatable dispatch on key-repeat:
  - `crates/fret-ui/src/tree.rs`
- Demo command registration + bindings:
  - `crates/fret-demo/src/main.rs`
