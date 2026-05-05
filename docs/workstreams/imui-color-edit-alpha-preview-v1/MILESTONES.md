# Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Reference and Owner Freeze

Exit criteria:

- Dear ImGui evidence is limited to transparent `ColorButton` preview behavior.
- The owner remains `fret-ui-editor`.
- No runtime, `fret-imui`, or generic draw-list widening is proposed.

Status: Complete.

## M1 - Checkerboard Preview Slice

Exit criteria:

- `ColorEdit` main swatches and preset swatches render through one checkerboard-backed preview
  helper.
- The current color remains the overlay color, so alpha is represented visually instead of being
  forced opaque.
- Checkerboard color alternation has focused unit coverage.

Status: Complete in `ecosystem/fret-ui-editor/src/controls/color_edit.rs`.

## M2 - Closeout Gate

Exit criteria:

- focused `fret-ui-editor` tests pass,
- the popup-stub guard remains green,
- and the lane is recorded as closed with a follow-on policy for larger picker work.

Status: Complete.
