# ADR 0012: Keyboard, IME, and Text Input Model

Status: Accepted

## Context

Editor applications rely heavily on keyboard interaction:

- global shortcuts (command system),
- text input for property fields,
- code-editor-grade IME and composition,
- consistent behavior across Windows/macOS/Linux, and later wasm.

Keyboard events are a common source of rewrites if the boundary between:

- physical key presses,
- text input characters,
- IME preedit/commit,

is not defined early.

## Decision

### 1) Physical keys for shortcuts

Shortcut handling is based on **physical key codes** (layout-independent) plus modifier state.

This ensures stable shortcuts across keyboard layouts and platforms.

### 2) Text input and IME are separate data channels

Text entry uses:

- `TextInput` events for committed insertion text (characters),
- `ImeEvent` for preedit/composition state (enabled/disabled, preedit, commit).

Widgets that edit text consume these events and can render composition UI without requiring
platform APIs directly.

Locked P0 behavior:

- `Event::TextInput` **must not include control characters** (e.g. backspace, tab, newline).
- Editing/navigation keys (Backspace/Delete/Enter/Tab/Arrows) are handled via `KeyDown` + commands (ADR 0020 / ADR 0023),
  not by interpreting `TextInput` as an “everything input stream”.
- If a `KeyDown` is resolved to a command binding (keymap hit), any subsequent `TextInput` emitted for the same keystroke
  must be suppressed so shortcuts do not insert characters into focused text fields.

### 3) IME candidate window positioning is an explicit feedback path (cursor area)

Receiving `ImeEvent` is not sufficient for a production-grade editor UX.

On macOS and Windows (and some Linux IME stacks), the OS candidate window should be positioned near the
caret. With winit, this requires calling:

- `window.set_ime_cursor_area(position, size)`

Therefore the framework must define a feedback path from the UI runtime back to the platform window:

- Text-editing widgets must be able to report a **caret rect** in window coordinates after layout.
- The app/runner must forward that caret rect to the platform window via a platform effect.

Locked P0 behavior:

- Caret rect is expressed in **window logical coordinates** (DIP / logical px) consistent with ADR 0017.
- The platform runner may convert to physical units where needed by OS APIs.
- The caret rect must be updated whenever:
  - selection/caret moves,
  - layout changes (scrolling, wrapping, DPI scale changes),
  - preedit (composition) changes.

Effect shape (implemented in desktop runner):

- `Effect::ImeSetCursorArea { window, rect }`
- `Effect::ImeAllow { window, enabled }` (maps to winit `set_ime_allowed` policy)

### 4) Preedit is rendered inline (OS candidate window is not enough)

For editor UX parity with VSCode/Zed:

- Preedit text should be rendered inline in the text widget (underlined / marked range), driven by `ImeEvent::Preedit`.
- The OS candidate window still needs caret positioning via `set_ime_cursor_area`.

This avoids a “blind typing” feel when the candidate window is far from the text cursor or when the IME UI is minimal.

### 5) Command routing is focus-aware

Commands are routed via the focused node (and bubbling rules), with a global fallback keymap layer.

## Consequences

- Shortcut behavior is consistent across platforms.
- IME can evolve independently of the widget tree and renderer internals.
- Code editor widgets can be built on top of the same contract as property panels.

## Future Work

- Define the concrete `KeyCode` / physical key representation and mapping strategy per platform.
- Add composition caret/selection metadata for richer IME UI.
- Document wasm constraints (IME/clipboard differences, and lack of `set_ime_cursor_area` on web).

## Implementation Notes

Winit reference anchors (desktop):

- IME events: `repo-ref/winit/src/event.rs` (search `WindowEvent::Ime`)
- IME cursor positioning: `repo-ref/winit/src/window.rs` (search `set_ime_cursor_area`)

Effect naming note:

- `Effect::ImeSetCursorArea` / `Effect::ImeAllow` are implemented:
  - `crates/fret-app/src/app.rs`
  - `crates/fret-runner-winit-wgpu/src/runner.rs`

IME event ingestion note:

- Winit `WindowEvent::Ime` is mapped into `fret-core::Event::Ime` by the desktop runner.

Text input ingestion note:

- Desktop runner filters control characters out of `KeyboardInput.text` before emitting `fret-core::Event::TextInput`:
  - `crates/fret-runner-winit-wgpu/src/runner.rs`

Current MVP text widget note:

- A minimal single-line `TextInput` widget renders inline preedit and updates the IME cursor area:
  - `crates/fret-ui/src/primitives/text.rs`
