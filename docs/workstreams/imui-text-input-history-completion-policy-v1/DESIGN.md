# ImUi Text Input History Completion Policy v1

Status: Closed
Last updated: 2026-05-04

## Problem

Dear ImGui exposes `ImGuiInputTextFlags_CallbackCompletion` and
`ImGuiInputTextFlags_CallbackHistory` through a C++ callback that can inspect and mutate the active
text buffer.

That callback shape is the wrong contract for Fret's core runtime. `crates/fret-ui` owns the hard
text editing mechanism, while `fret-ui-kit::imui` owns immediate-mode authoring policy and command
routing. Completion/history is also usually app/editor domain behavior, not a runtime text engine
contract.

## Target

- Add opt-in command policy fields to single-line `InputTextOptions`.
- Dispatch completion on unmodified Tab while the field is focused.
- Dispatch history previous/next on unmodified Up/Down while the field is focused.
- Ignore IME composition and modified keys.
- Suppress repeat by default, with explicit repeat opt-ins.
- Keep `fret-imui` thin and avoid new `crates/fret-ui` public API.

## Ownership

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`: public IMUI policy options.
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`: key arbitration and command dispatch.
- `ecosystem/fret-imui/src/tests/models_text.rs`: app-facing immediate-mode proof.

## Non-Goals

- Dear ImGui-style mutable buffer callbacks.
- Completion popup rendering or history storage.
- Character filters.
- Undo/redo stack ownership.
- Multiline history callbacks. Dear ImGui explicitly treats multiline history as a key conflict.
