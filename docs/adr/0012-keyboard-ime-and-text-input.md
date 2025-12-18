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

- `TextInput` events for committed text (characters),
- `ImeEvent` for preedit/composition state (enabled/disabled, preedit, commit).

Widgets that edit text consume these events and can render composition UI without requiring
platform APIs directly.

### 3) Command routing is focus-aware

Commands are routed via the focused node (and bubbling rules), with a global fallback keymap layer.

## Consequences

- Shortcut behavior is consistent across platforms.
- IME can evolve independently of the widget tree and renderer internals.
- Code editor widgets can be built on top of the same contract as property panels.

## Future Work

- Define the concrete `KeyCode` / physical key representation and mapping strategy per platform.
- Add composition caret/selection metadata for richer IME UI.
- Document wasm constraints (IME/clipboard differences).

