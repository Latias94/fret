# ImUi Text Input Undo Command Policy v1 Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. The Dear ImGui undo/redo shortcut gap is covered for Fret's IMUI layer as an app-owned
command policy, not as a runtime-owned text undo stack.

## What Shipped

- `InputTextOptions::undo_command`
- `InputTextOptions::redo_command`
- `InputTextOptions::undo_redo_command_repeat`
- Focused Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z command dispatch from single-line IMUI input text.
- Tests covering unsupported modifiers, default repeat suppression, and explicit repeat opt-in.

## Layering Decision

The implementation stays in `fret-ui-kit::imui` because undo/redo ownership is interaction policy.
`crates/fret-ui` remains a text editing mechanism layer and does not gain an internal undo stack for
this slice. `fret-imui` remains a thin app-facing proof surface.

## Evidence

- Dear ImGui references: `repo-ref/imgui/imgui.h`, `repo-ref/imgui/imgui_widgets.cpp`
- Policy options: `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- Key arbitration: `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- Regression gate: `ecosystem/fret-imui/src/tests/models_text.rs`

## Follow-On Policy

Do not reopen this lane for completion/history UI, deeper multiline editing, or editor transaction
integration. Those need separate policy or app/editor workstreams with their own repro and gates.
