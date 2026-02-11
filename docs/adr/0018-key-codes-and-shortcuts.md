# ADR 0018: Key Codes and Shortcut Normalization

Status: Accepted

## Context

Fret must support editor-grade keyboard interaction across Windows/macOS/Linux, and later wasm:

- reliable shortcuts (command system),
- consistent behavior across keyboard layouts,
- text input and IME composition for editors and property fields.

If “what is a key press?” is not defined early, the input model tends to be rewritten when:

- non-US layouts are used,
- macOS modifier conventions collide with Windows/Linux,
- IME composition and text widgets become real requirements.

References:

- Fret keyboard/IME separation principles:
  - `docs/adr/0012-keyboard-ime-and-text-input.md`
- winit keyboard model (physical key vs logical key):
  - https://docs.rs/winit/

## Decision

### 1) Physical key codes are the canonical representation for shortcuts

Fret defines a **physical key** representation for shortcut handling.

Properties:

- layout-independent (same physical key produces the same code regardless of layout),
- stable across platforms,
- suitable for keymap persistence and conflict detection.

Shortcut matching uses:

- `PhysicalKey` (Fret `KeyCode` / future `PhysicalKey`),
- `Modifiers` (shift/ctrl/alt/altgr/meta),
- `repeat` flag only for UI behaviors that explicitly opt into repeats.

### 2) Text input is not derived from key presses

Text editing widgets must not “invent” characters from key presses.

Committed text comes from:

- `Event::TextInput(String)` for text insertion,
- `Event::Ime(ImeEvent)` for composition state.

### 3) Platform normalization rules are explicit

Fret normalizes platform input into a common model:

- `Modifiers::meta` is the “command key” on macOS and the “Windows key” on Windows.
- Keymaps may define platform-specific bindings (e.g. `Cmd` vs `Ctrl`), but the event model is consistent.

### 4) Location-sensitive modifiers are deferred but reserved

For editor-grade shortcuts, left/right modifier distinction is occasionally needed (e.g. advanced keybindings).

Initial contract:

- `Modifiers` stays coarse (shift/ctrl/alt/altgr/meta). AltGr is semantically distinct from `ctrl+alt`
  for shortcut matching (see ADR 0043).

Reserved future extension:

- add optional left/right modifier location information without breaking `Event`/`KeyCode` semantics.

## Consequences

- Shortcuts remain stable across keyboard layouts (critical for professional editors).
- Text input and IME remain correct and portable (especially on macOS and for CJK users).
- Keymap persistence can target physical keys without depending on OS-level string names.

## Future Work

- Maintain alignment with `keyboard-types::Code` as the canonical physical key universe (ADR 0091).
- Define a canonical keymap file format (see ADR 0014) including platform-specific bindings.
- Add optional left/right modifier locations if needed by advanced users.

## Implementation Notes

Current implementation aligns `fret-core::KeyCode` with `keyboard-types::Code` (ADR 0091) and forwards winit physical
key codes directly:

- `crates/fret-core/src/input.rs`
- `crates/fret-runner-winit/src/lib.rs` (`map_physical_key`)
