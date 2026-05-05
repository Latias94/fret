# ImUi Models Text Command Test Split v1 TODO

Status: Closed
Last updated: 2026-05-04

## M1 - Mechanical Command Split

- [x] Add `ecosystem/fret-imui/src/tests/models_text_commands.rs`.
- [x] Move completion, history, undo/redo, and repeat opt-in command tests into the new module.
- [x] Register `models_text_commands` from `ecosystem/fret-imui/src/tests/mod.rs`.
- [x] Keep command behavior and public APIs unchanged.
- [x] Run focused command and broader text-model gates.

## Future Follow-Ons

- [ ] Split textarea behavior into a dedicated module if multiline work resumes.
- [ ] Split focus/edit lifecycle tests separately from basic changed-signal tests.
- [ ] Consider fixtures only if command cases become a repetitive key/chord matrix.
