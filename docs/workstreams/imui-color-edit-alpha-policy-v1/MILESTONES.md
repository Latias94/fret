# Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Reference and Owner Freeze

Exit criteria:

- the lane stays a follow-on of the closed popup-depth slice,
- Dear ImGui evidence is limited to alpha-preserving RGB palette behavior,
- and no runtime or `fret-imui` widening is proposed.

Status: Complete.

## M1 - RGB-Only Alpha Preservation

Exit criteria:

- RGB-only hex commits preserve alpha,
- RGBA hex commits can change alpha only when alpha is visible,
- and RGB preset swatches preserve the current model alpha.

Status: Complete in `ecosystem/fret-ui-editor/src/controls/color_edit.rs`.

## M2 - Closeout Gate

Exit criteria:

- focused `fret-ui-editor` tests pass,
- the popup-stub guard remains green,
- and the lane is recorded as closed with a follow-on policy for larger picker work.

Status: Complete.
