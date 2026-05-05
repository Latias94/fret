# ImUi Color Edit Popup Swatches Split v1

Status: Closed narrow refactor follow-on
Last updated: 2026-05-05

This lane extracts preset swatch row composition from `color_edit/popup.rs` into
`color_edit/popup/swatches.rs`. It keeps the popup overlay module focused on content assembly while
leaving preset activation policy in an internal editor-control owner.

## Ownership

- `popup.rs` owns popup overlay assembly and content ordering.
- `popup/swatches.rs` owns preset swatch row composition, selected-state border policy,
  alpha-preserving preset activation, draft sync, popup close, and swatch test-id derivation.
- `popup/preview.rs` owns checkerboard and color preview helpers used by swatches.
- `popup/picker.rs` and `popup/numeric.rs` keep their previous picker and numeric-row ownership.

## Must-Be-True Outcomes

- Preset swatches remain available through `ColorEditPopupOptions::presets`.
- Preset activation still preserves the current alpha channel, updates the hex draft, clears errors,
  and closes the popup.
- Source-policy tests point preset helper ownership at `popup/swatches.rs`.

## Non-Goals

- No preset behavior changes.
- No public API changes.
- No palette customization, history, eyedropper, or drag/drop feature work.
