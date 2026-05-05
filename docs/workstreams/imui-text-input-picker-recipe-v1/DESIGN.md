# ImUi Text Input Picker Recipe v1

Status: Closed
Last updated: 2026-05-04

## Problem

The previous text-input completion/history slice added command routing for Tab and Up/Down, but app
authors still had to build the visible candidate UI by hand. Dear ImGui exposes completion/history
through mutable-buffer callbacks; Fret should instead provide a reusable recipe that composes
existing IMUI primitives and keeps data ownership in the app/editor layer.

## Target

- Add a model-backed input text picker helper for completion candidates.
- Add a history variant that reuses the same picker machinery with unfiltered history defaults.
- Render candidates in a non-modal popup anchored to the input field.
- Commit clicked candidates into the app-owned `Model<String>`.
- Return picked value/index and changed state.
- Keep runtime text input and `fret-imui` free of candidate storage and callback data structs.

## Ownership

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`: picker options and filtering policy.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`: recipe composition.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`: public IMUI helper methods.
- `ecosystem/fret-imui/src/tests/models_text.rs`: app-facing behavior proof.

## Must-Be-True Outcomes

- A user can render an input with visible completion candidates without writing popup/selectable
  boilerplate.
- Clicking a candidate updates the same text model used by the input.
- Completion hides exact-match suggestions by default so committed values do not immediately reopen
  the picker.
- History can show unfiltered entries even when the input is empty.
- The candidate list remains app-owned data; the helper does not store editor history or completion
  ranking internally.

## Non-Goals

- No mutable text-buffer callbacks.
- No runtime-owned candidate/history store.
- No active-descendant keyboard navigation or selection index in this slice.
- No multiline completion/history conflict handling.
