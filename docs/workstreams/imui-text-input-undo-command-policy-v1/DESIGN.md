# ImUi Text Input Undo Command Policy v1

Status: Closed
Last updated: 2026-05-04

## Problem

Dear ImGui owns the active `InputText` buffer and can therefore provide internal undo/redo unless
`ImGuiInputTextFlags_NoUndoRedo` is set. Fret text input does not currently own an internal undo
stack, and app/editor undo state is governed by higher-level command and transaction policy.

Copying Dear ImGui's active-buffer undo model into `crates/fret-ui` would move policy into the
runtime mechanism layer and would conflict with Fret's app-owned editing model.

## Target

- Add opt-in undo/redo command fields to single-line `InputTextOptions`.
- Dispatch undo on Ctrl+Z while the field is focused.
- Dispatch redo on Ctrl+Y and Ctrl+Shift+Z while the field is focused.
- Ignore IME composition, Alt, and Meta.
- Suppress repeated keydown by default, with an explicit repeat opt-in.
- Treat the default unset state as the Fret-native `NoUndoRedo` behavior.

## Ownership

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`: public IMUI policy options.
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`: key arbitration and command dispatch.
- `ecosystem/fret-imui/src/tests/models_text.rs`: app-facing immediate-mode proof.

## Must-Be-True Outcomes

- IMUI authors can route text-field undo/redo shortcuts into app-owned commands.
- Runtime text input does not grow an internal undo stack or mutable-buffer callback contract.
- Default `InputTextOptions` do not claim undo/redo ownership; leaving commands unset is equivalent
  to disabling built-in undo/redo for this Fret layer.
- Repeat behavior is opt-in and covered by the same policy command test as completion/history.

## Non-Goals

- No runtime-owned text undo stack.
- No Dear ImGui callback data struct.
- No editor transaction model changes.
- No completion/history popup rendering.
- No multiline-specific undo policy.
