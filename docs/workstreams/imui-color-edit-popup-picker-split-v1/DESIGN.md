# ImUi Color Edit Popup Picker Split v1

Status: Closed narrow refactor follow-on
Last updated: 2026-05-05

This lane extracts HSV/SV/Hue/Alpha picker composition from `color_edit/popup.rs` into
`color_edit/popup/picker.rs`. It keeps the popup assembly module from owning every picker detail
while preserving the existing editor `ColorEdit` behavior.

## Ownership

- `popup.rs` owns popup overlay assembly, preset swatches, and shared preview helpers.
- `popup/picker.rs` owns HSV picker composition, saturation/value grid and thumb overlay, HueBar,
  AlphaBar, gradient/thumb helpers, and picker-local pointer handlers.
- `popup/numeric.rs` owns editable RGB/HSV numeric row composition and commit handling.
- `model.rs` still owns parsing, formatting, color conversion, and picker coordinate math.

## Must-Be-True Outcomes

- The same `ColorEditPopupPicker::HsvHueBar` and `ColorEditPopupOptions::alpha_bar` surfaces still
  compose the visible picker controls.
- SV, HueBar, and AlphaBar pointer commits still update the color model, draft hex, and error state
  exactly as before.
- Focus and accessibility values for picker controls remain stable.
- Source-policy tests point picker helper ownership at `popup/picker.rs`.

## Non-Goals

- No picker behavior changes.
- No public API changes.
- No new color features, fixture format, or shared color crate.
