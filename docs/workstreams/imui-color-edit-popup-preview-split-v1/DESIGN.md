# ImUi Color Edit Popup Preview Split v1

Status: Closed narrow refactor follow-on
Last updated: 2026-05-05

This lane extracts shared color preview helpers from `color_edit/popup.rs` into
`color_edit/popup/preview.rs`. It keeps checkerboard and fill-layout details in one owner that can
be reused by the root swatch, preset swatches, and picker alpha previews.

## Ownership

- `popup.rs` owns popup overlay assembly and preset swatch composition.
- `popup/preview.rs` owns checkerboard preview grid, checkerboard cell colors, fill-preview layout,
  and color preview stack composition.
- `popup/picker.rs` owns HSV/SV/Hue/Alpha picker composition and imports preview helpers for
  AlphaBar rendering.
- `popup/numeric.rs` owns editable RGB/HSV numeric row composition and commit handling.

## Must-Be-True Outcomes

- Main swatch, preset swatches, and AlphaBar preview still share the same checkerboard-backed
  preview helpers.
- `checkerboard_cell_color` tests keep the same alternating colors.
- Source-policy tests point preview helper ownership at `popup/preview.rs`.

## Non-Goals

- No preview behavior changes.
- No public API changes.
- No swatch-row feature or picker feature changes.
