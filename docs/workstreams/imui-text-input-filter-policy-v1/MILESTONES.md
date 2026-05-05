# ImUi Text Input Filter Policy v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Source Mapping

Dear ImGui reference:

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`

Outcome: named filters are separated from callback filters and treated as a bounded policy slice.

## M1 - Runtime Mechanism

Outcome: `TextInputProps` can carry a generic insert-filter closure, and the retained text-input
engine applies it to insertion paths without knowing about Dear ImGui flags.

Evidence:

- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/text/input/widget.rs`
- `crates/fret-ui/src/text/input/tests.rs`

## M2 - IMUI Policy

Outcome: `InputTextOptions::filters` exposes named filters through `fret-ui-kit::imui` and is wired
through the existing model-backed single-line input helper.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`

## M3 - Closeout

Outcome: the named-filter gap is closed, while callback filter behavior remains an explicit
follow-on rather than a hidden runtime contract expansion.
