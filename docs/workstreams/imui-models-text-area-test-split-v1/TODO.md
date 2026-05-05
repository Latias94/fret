# ImUi Models Text Area Test Split v1 TODO

Status: Closed
Last updated: 2026-05-04

## M1 - Mechanical Textarea Split

- [x] Add `ecosystem/fret-imui/src/tests/models_text_area.rs`.
- [x] Move textarea read-only, Tab policy, changed-signal, and lifecycle tests into the new module.
- [x] Register `models_text_area` from `ecosystem/fret-imui/src/tests/mod.rs`.
- [x] Remove the textarea-only import from `models_text.rs`.
- [x] Run focused textarea and broader text-model gates.

## Future Follow-Ons

- [ ] Split remaining single-line focus/bounds/lifecycle tests into a dedicated module.
- [ ] Split push-id reorder stability separately from the basic changed-signal test if the identity
  lane resumes.
- [ ] Consider fixtures only if multiline cases become a repeated option matrix.
