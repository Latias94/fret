# ImUi Models Text Mode Test Split v1 TODO

Status: Closed
Last updated: 2026-05-04

## M1 - Mechanical Mode Split

- [x] Add `ecosystem/fret-imui/src/tests/models_text_modes.rs`.
- [x] Move read-only, select-all-on-focus, and password-mode tests into the new module.
- [x] Register `models_text_modes` from `ecosystem/fret-imui/src/tests/mod.rs`.
- [x] Remove the mode-only import from `models_text.rs`.
- [x] Run focused mode and broader text-model gates.

## Future Follow-Ons

- [ ] Split completion/history/undo command-policy tests into a dedicated module.
- [ ] Split textarea behavior into a dedicated module if multiline work resumes.
- [ ] Split focus/edit lifecycle tests separately from basic changed-signal tests.
