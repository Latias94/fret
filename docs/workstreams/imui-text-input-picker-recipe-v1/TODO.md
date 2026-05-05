# ImUi Text Input Picker Recipe v1 TODO

Status: Closed
Last updated: 2026-05-04

## Completed

- [x] Add `InputTextPickerFilter` and `InputTextPickerOptions`.
- [x] Add `InputTextPickerResponse`.
- [x] Add `input_text_completion_model(_with_options)`.
- [x] Add `input_text_history_model(_with_options)`.
- [x] Compose existing input, popup, and selectable primitives instead of adding runtime APIs.
- [x] Cover filtered completion popup commit behavior with a model test.
- [x] Cover unfiltered empty-history popup behavior with a model test.
- [x] Update roadmap, workstream catalog, todo tracker, and IMUI gap audit.

## Deferred Follow-Ons

- Active-descendant keyboard navigation and Enter-to-commit behavior.
- Editor-owned completion scoring, history storage, and command-driven selected-index mutation.
- Multiline-specific completion/history conflict policy.
