# ADR 0012: Keyboard, IME, and Text Input Model


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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

### 2.1) IME keystroke arbitration (preedit-first)

IME composition introduces a second consumer of keystrokes: the platform input method. To avoid
common failures (e.g. Tab/Enter/Escape being "stolen" by focus traversal or global shortcuts while
the IME is active), the framework defines a deterministic arbitration rule.

Definitions:

- "Composing" means the focused text input has an active IME preedit/composition state (preedit
  string is non-empty and/or the platform reports a marked/active composition range/cursor (some
  platforms report composition cursor updates even when the preedit string is empty).
- "Non-printing keys" include navigation and control keys: arrows, Backspace/Delete, Home/End,
  PageUp/PageDown, Escape, Enter (including NumpadEnter), Tab.
- Some IMEs also use printable keys (notably Space) during composition for candidate conversion; these
  must be treated as IME-sensitive while composing.

Locked P0 behavior:

1) When the focused widget is a text-editing widget and IME is enabled:
   - If composing, **IME gets first refusal** on `KeyDown` for:
      `Tab`, `Space`, `Enter`/`NumpadEnter`, `Escape`, arrows, `Backspace`, `Delete`, `Home`, `End`,
      `PageUp`, `PageDown`.
   - The UI runtime (text widget) may still handle these keys if the IME does not consume them,
     but **global shortcuts and focus traversal must not run first**.

2) Even when not composing, the platform may still require non-printing keys for IME UI (candidate
   navigation, dismissal). Therefore:
   - For non-printing keys without `Ctrl/Alt/Meta` modifiers, the platform/IME handler may be
     consulted before command routing (best-effort, platform dependent).
   - Keys with `Ctrl/Alt/Meta` are treated as shortcut candidates first (to avoid injecting
     control characters into text buffers on some platforms).

3) If a command binding is matched for a keystroke and the command is executed, any resulting
   committed text (`Event::TextInput`) for that same keystroke must be suppressed (see 2.0).

Rationale:

- This matches editor-grade expectations (VSCode/Zed-class behavior) where IME conversion and
  candidate navigation remain functional even when the host application has rich shortcut layers.
- It prevents the "Tab is eaten by the app" class of bugs observed in multiple Rust GUI stacks.

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
  - preedit (composition) text or preedit cursor changes.

Effect shape (implemented in desktop runner):

- `Effect::ImeSetCursorArea { window, rect }`
- `Effect::ImeAllow { window, enabled }` (maps to winit `set_ime_allowed` policy)

Emitter responsibility (P0):

- `Effect::ImeAllow` is owned by the UI runtime (`UiTree`): it is updated when focus changes and
  at paint time so programmatic focus updates also sync the platform IME state.
- Text-editing widgets must **not** emit `ImeAllow` every frame; they only emit `ImeSetCursorArea`
  when the caret rect changes.

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

- IME events: winit `WindowEvent::Ime` (`winit/src/event.rs`)
- IME cursor positioning: winit `Window::set_ime_cursor_area` (`winit/src/window.rs`)

Note: this repository does not currently vendor `winit` sources under `repo-ref/`. If you need to read the
exact version used by this workspace, use `Cargo.lock` + `cargo vendor` and inspect the vendored sources
under `vendor/` (or fetch the exact crate version from crates.io).

Effect naming note:

- `Effect::ImeSetCursorArea` / `Effect::ImeAllow` are implemented:
  - `crates/fret-app/src/app.rs`
  - `crates/fret-launch/src/runner/mod.rs`

IME event ingestion note:

- Winit `WindowEvent::Ime` is mapped into `fret-core::Event::Ime` by the desktop runner.

Text input ingestion note:

- Desktop runner filters control characters out of `KeyboardInput.text` before emitting `fret-core::Event::TextInput`:
  - `crates/fret-launch/src/runner/mod.rs`

Current MVP text widget note:

- A minimal single-line `TextInput` widget renders inline preedit and updates the IME cursor area:
  - `crates/fret-ui/src/text_input/mod.rs`
- Multiline text input experiments (still evolving; see ADR 0071):
  - `crates/fret-ui/src/text_area/mod.rs`

## Notes (Zed/GPUI reference, non-normative)

- GPUI exposes IME/text-input integration via an explicit `InputHandler` interface (modeled after
  `NSTextInputClient`), including UTF-16 selection/marked ranges and “bounds for range” queries
  used for candidate window positioning:
  `repo-ref/zed/crates/gpui/src/platform.rs` (`InputHandler`, `UTF16Selection`).
- GPUI updates IME candidate window positioning by querying selection bounds on the next frame and
  calling a platform window hook:
  `repo-ref/zed/crates/gpui/src/window.rs` (`invalidate_character_coordinates`).
