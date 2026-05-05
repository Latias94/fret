# ImUi Color Edit Popup Numeric Split v1

Status: Closed narrow refactor follow-on
Last updated: 2026-05-05

This lane extracts the editable RGB/HSV numeric popup rows from `color_edit/popup.rs` into
`color_edit/popup/numeric.rs`. It keeps the popup module from accumulating every popup concern while
preserving the existing editor `ColorEdit` behavior.

## Ownership

- `popup.rs` owns popup overlay assembly, picker visuals, alpha bar, presets, and shared preview
  helpers.
- `popup/numeric.rs` owns editable RGB/HSV numeric row composition, validation error display,
  Enter/Escape commit behavior, and numeric field placeholders.
- `model.rs` still owns parsing, formatting, and color conversion.

## Must-Be-True Outcomes

- Editable popup numeric rows remain available through the same `ColorEditPopupNumericInputs`
  option surface.
- Invalid numeric input still reports the popup-local error line.
- Valid numeric commits update the color model and hex draft exactly as before.
- Source-policy tests point numeric helper ownership at `popup/numeric.rs`.

## Non-Goals

- No numeric behavior changes.
- No public API changes.
- No fixture format or shared color crate.
