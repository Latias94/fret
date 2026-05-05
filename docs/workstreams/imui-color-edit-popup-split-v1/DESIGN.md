# ImUi Color Edit Popup Split v1

Status: Closed narrow refactor follow-on
Last updated: 2026-05-05

This lane follows the `ColorEdit` model split with the next architecture cleanup: move popup UI
composition out of the public control file. The control had already gained enough Dear ImGui-class
depth that keeping popup overlay assembly, HSV picker UI, alpha bar UI, numeric rows, swatches, and
pointer handlers in `color_edit.rs` made future color features risky to review.

The refactor keeps public behavior unchanged. `color_edit.rs` stays the public control and option
owner. `color_edit/model.rs` stays the pure model owner. `color_edit/popup.rs` now owns the popup
surface, picker widgets, preview helpers, and popup-local pointer handlers.

## Ownership

- `color_edit.rs` owns the public `ColorEdit` struct, option types, root swatch/input row, and
  overlay request entry point.
- `color_edit/model.rs` owns color parsing, formatting, HSV/RGB conversion, coordinate math, and
  sanitization.
- `color_edit/popup.rs` owns popup composition, HSV picker UI, numeric popup rows, alpha bar UI,
  preset swatches, checkerboard/gradient previews, and popup-local pointer handlers.
- `fret-imui`, `fret-ui-kit`, `crates/fret-ui`, and renderer/runtime contracts are not in scope.

## Must-Be-True Outcomes

- `color_edit.rs` is small enough to review as public control wiring.
- `popup.rs` contains popup UI composition and popup-local interaction helpers.
- `model.rs` remains free of declarative UI composition.
- Source-policy tests track all three internal owners instead of forcing helper symbols back into
  `color_edit.rs`.
- Existing color-edit behavior and public adapter reachability remain unchanged.

## Non-Goals

- No new `ColorEdit` features.
- No visual redesign.
- No public module/API exposure for `popup.rs`.
- No runtime, focus, overlay, text, or renderer contract changes.
- No HueWheel, color history, palette customization, eyedropper, or color drag/drop payload work.
