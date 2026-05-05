# ImUi Color Edit Model Split v1

Status: Closed narrow refactor follow-on
Last updated: 2026-05-05

This lane closes the architecture hazard left after the completed editor `ColorEdit` depth slices:
the control became a large mixed file where pure color math, parser/formatter policy, pointer
coordinate mapping, accessibility strings, and declarative UI composition lived together.

The refactor keeps public behavior unchanged. It splits pure, testable model helpers into
`controls::color_edit::model` while leaving the editor control surface, option structs, popup
composition, and event wiring in `color_edit.rs`.

## Ownership

- `fret-ui-editor` owns the editor `ColorEdit` component policy and model helpers.
- `controls::color_edit::model` owns pure color state helpers, numeric parsing/formatting,
  HSV/RGB conversion, coordinate normalization, sanitization, and a11y value text.
- `controls::color_edit.rs` owns public options, UI element composition, overlay wiring, swatch
  rendering, and interaction callbacks.
- `fret-imui`, `fret-ui-kit`, `crates/fret-ui`, and renderer/runtime contracts are not in scope.

## Assumptions

- Area: lane status
  - Assumption: the popup-options lane is closed, so this architecture cleanup should be a narrow
    follow-on rather than a reopened feature lane.
  - Evidence: `docs/workstreams/imui-color-edit-popup-options-v1/WORKSTREAM.json`.
  - Confidence: Confident.
  - Consequence if wrong: feature and architecture evidence would be mixed in one lane.

- Area: ownership
  - Assumption: color parsing/conversion helpers are editor policy/model details, not runtime
    mechanisms.
  - Evidence: helpers depend on `ColorEditPopupNumericInputs` and editor-facing numeric text policy.
  - Confidence: Confident.
  - Consequence if wrong: shared runtime APIs would be introduced before a second consumer exists.

- Area: proof
  - Assumption: behavior preservation can be proven with existing focused color-edit, source-policy,
    and adapter-smoke gates.
  - Evidence: those gates already cover popup visibility, numeric modes, HSV conversion, parsing,
    and public adapter reachability.
  - Confidence: Likely.
  - Consequence if wrong: add a fixture-driven color model conformance gate in a later follow-on.

## Must-Be-True Outcomes

- `color_edit.rs` is smaller and focused on public options plus UI composition.
- Pure color model helpers live in `color_edit/model.rs`.
- Existing tests move with the model where appropriate and continue covering the same behavior.
- Source-policy tests track the new model owner instead of pinning helper symbols to the old file.
- No public API, runtime contract, component behavior, or renderer path changes.

## Non-Goals

- No new `ColorEdit` features.
- No shared color crate or runtime-level color model.
- No Dear ImGui global `SetColorEditOptions()` state.
- No color history, palette customization, eyedropper behavior, drag/drop color payloads, or
  HueWheel fidelity.
- No broad reorganization of every editor control.
