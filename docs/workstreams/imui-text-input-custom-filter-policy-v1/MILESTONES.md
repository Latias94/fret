# ImUi Text Input Custom Filter Policy v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Reference

Dear ImGui reference:

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`

Outcome: Fret maps the useful callback-filter outcome to insertion filtering, not mutable-buffer
callbacks.

## M1 - Policy Surface

Outcome: `InputTextOptions::custom_filter` gives app authors a composable custom insertion filter.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`

## M2 - Wiring And Proof

Outcome: named filters run first and the custom filter sees the named-filtered text.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`

## M3 - Closeout

Outcome: custom input filtering is no longer an open IMUI text input parity gap; remaining text
input gaps are active-descendant picker keyboard navigation and deeper multiline behavior.
Undo/redo command routing is tracked by
`docs/workstreams/imui-text-input-undo-command-policy-v1/`, and the first visible picker recipe is
tracked by `docs/workstreams/imui-text-input-picker-recipe-v1/`.
