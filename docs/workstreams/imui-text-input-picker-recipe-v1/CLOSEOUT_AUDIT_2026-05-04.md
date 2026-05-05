# ImUi Text Input Picker Recipe v1 Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. IMUI now has a reusable text-input picker recipe for the first visible completion/history
UI layer. It composes existing input, popup, and selectable primitives in `fret-ui-kit::imui` and
keeps candidate data app-owned.

## What Shipped

- `InputTextPickerFilter`
- `InputTextPickerOptions`
- `InputTextPickerResponse`
- `input_text_completion_model(_with_options)`
- `input_text_history_model(_with_options)`
- Regression tests for filtered completion commit and empty history display.

## Layering Decision

The recipe belongs in `fret-ui-kit::imui`. `crates/fret-ui` remains the text editing mechanism and
does not gain candidate storage, history storage, or Dear ImGui callback data. `fret-imui` remains
the app-facing proof surface.

## Evidence

- Options and public response: `ecosystem/fret-ui-kit/src/imui/options/controls.rs`,
  `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
- Recipe implementation: `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`
- Facade methods: `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- Tests: `ecosystem/fret-imui/src/tests/models_text.rs`

## Follow-On Policy

Do not reopen this lane for active-descendant keyboard navigation, editor-owned completion ranking,
history persistence, or multiline conflicts. Those need separate workstreams with their own gates.
