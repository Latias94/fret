---
name: fret-text-input-and-ime
description: Text input, IME composition, and text-editing command routing in Fret. Use when implementing input/textarea/combobox text surfaces, debugging IME/caret issues, or adding gates to prevent regressions.
---

# Fret text input and IME

## When to use

- Building or refactoring `Input`, `Textarea`, `Combobox`, command palette search fields, or code-editor-like widgets.
- Debugging IME issues (composition breaks, Tab/Escape misbehaves, candidate window is misplaced).
- Fixing “shortcut inserts characters” or “typing triggers global commands” regressions.

## Core rules (ADR 0012 is the contract)

- **Shortcuts and text entry are separate channels**:
  - `KeyDown` is for commands/navigation/editing keys.
  - `TextInput` is for committed insertion text (no control characters).
  - `ImeEvent` carries preedit/composition state.
- **IME gets first refusal** on non-printing keys while composing (Tab/Enter/Escape/arrows/etc).
  - Global shortcuts and focus traversal must not steal these keys when composing.
- **Caret rect feedback is required** for correct candidate window placement.
  - The runner forwards caret geometry to the platform (`Effect::ImeSetCursorArea`).
- **If a key dispatch resolves to a command**, any `TextInput` for the same keystroke must be suppressed.

## Quick start (component-level)

### Use the shadcn-aligned Input surface

```rust
use fret_ui_shadcn::prelude::*;
use fret_ui_shadcn::input::Input;

pub fn search_box<H: UiHost>(cx: &mut ElementContext<'_, H>, query: Model<String>) -> AnyElement {
    Input::new(query)
        .placeholder("Search…")
        .a11y_label("Search")
        .into_element(cx)
}
```

### Combobox input: active-descendant semantics

- If focus stays in the input while navigating results, use `active_descendant` and `expanded` on the input.
- Keep global keybindings gated with `when: "focus.is_text_input == false"` unless they are explicitly editing commands.

## Workflow

1. Keep channels separate (ADR 0012): `KeyDown` for commands/editing keys, `TextInput` for committed text, `ImeEvent` for composition.
2. Ensure IME gets first refusal while composing (Tab/Enter/Escape/arrows must not be stolen).
3. Provide caret rect feedback so candidate windows place correctly (`Effect::ImeSetCursorArea`).
4. Gate global shortcuts on focus/composition state and add a regression artifact (script or unit test).

## Common pitfalls

- **Manually inserting characters from `KeyDown`** (breaks IME and international layouts).
- **Stealing Tab/Escape** for app navigation while composing (IME must win first).
- **Mixing UTF-8 byte indices with UTF-16 ranges**:
  - Platform text input queries typically use UTF-16 (`Utf16Range`).
- **Incorrect focus gating**: shortcuts should not fire while text input is focused unless the command is text-editing.

## Regression gates (recommended defaults)

- Add a `tools/diag-scripts/*.json` scenario that:
  - focuses an input, types, uses Backspace/Arrows, and asserts text + selection invariants via `test_id` targets.
- Add a unit test for any boundary-sensitive behavior (word boundaries, selection clamping, caret metrics).
- For IME-specific regressions, at minimum add a gate around the command/keymap `when` logic
  (so composing doesn’t accidentally trigger global shortcuts).

## References (start here)

- ADRs:
  - `docs/adr/0006-text-system.md`
  - `docs/adr/0012-keyboard-ime-and-text-input.md`
  - `docs/adr/0044-text-editing-state-and-commands.md`
  - `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
  - `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
  - `docs/adr/0195-web-ime-and-text-input-bridge-v1.md`
- Code entry points:
  - `crates/fret-runtime/src/platform_text_input.rs` (`PlatformTextInputQuery`, `Utf16Range`)
  - `crates/fret-runtime/src/window_text_input_snapshot.rs` (snapshots / platform bridge)
  - `crates/fret-ui/src/text_edit.rs` (editing model)
  - `crates/fret-ui/src/text_input/mod.rs` / `crates/fret-ui/src/text_area/mod.rs` (element internals)
  - `ecosystem/fret-ui-shadcn/src/input.rs` / `ecosystem/fret-ui-shadcn/src/textarea.rs`
  - `ecosystem/fret-ui-shadcn/src/combobox.rs` (active descendant + overlay patterns)

## Evidence anchors

- Contract: `docs/adr/0012-keyboard-ime-and-text-input.md`
- Editing commands: `docs/adr/0044-text-editing-state-and-commands.md`
- Geometry/caret metrics: `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
- Platform bridge: `crates/fret-runtime/src/platform_text_input.rs`

## Related skills

- `fret-commands-and-keymap` (shortcut gating and focus-aware routing)
- `fret-overlays-and-focus` (combobox/select overlays that must not break typing)
