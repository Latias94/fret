# Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Owner and Reference Freeze

Exit criteria:

- Dear ImGui evidence is scoped to `ColorPicker4` / `AlphaBar` behavior.
- The owner remains `fret-ui-editor`.
- The implementation does not add runtime contracts or thicken `fret-imui`.

Status: Complete.

## M1 - AlphaBar Editing Slice

Exit criteria:

- AlphaBar appears only when alpha editing is visible.
- Pointer down and drag update the model alpha using clamped local x position.
- Draft hex and errors stay in sync with the model after alpha edits.

Status: Complete in `ecosystem/fret-ui-editor/src/controls/color_edit.rs`.

## M2 - Closeout Gate

Exit criteria:

- focused alpha-bar tests pass,
- full `fret-ui-editor` IMUI tests pass,
- and the lane records follow-on boundaries for HSV/picker depth.

Status: Complete.
