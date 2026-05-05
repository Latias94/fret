# ImUi Models Text Picker Test Split v1 TODO

Status: Closed
Last updated: 2026-05-04

## M1 - Mechanical Picker Split

- [x] Add `ecosystem/fret-imui/src/tests/models_text_picker.rs`.
- [x] Move completion/history picker tests into the new module.
- [x] Register `models_text_picker` from `ecosystem/fret-imui/src/tests/mod.rs`.
- [x] Remove picker-only imports from `models_text.rs`.
- [x] Run focused picker and broader text-model gates.

## Future Follow-Ons

- [ ] Split named/custom filter coverage out of `models_text.rs`; consider a small fixture harness
  if the cases stay data-shaped.
- [ ] Split lifecycle/read-only/password/command-policy tests by capability family once the next text
  policy lane needs edits.
- [ ] Continue decomposing large `fret-imui` test files such as `interaction.rs`, `floating.rs`, and
  `popup_hover.rs` only when a real refactor touches those areas.
