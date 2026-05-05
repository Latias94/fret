# ImUi Models Text Filter Test Split v1 TODO

Status: Closed
Last updated: 2026-05-04

## M1 - Mechanical Filter Split

- [x] Add `ecosystem/fret-imui/src/tests/models_text_filters.rs`.
- [x] Move named filter, numeric filter, and custom insertion-filter tests into the new module.
- [x] Register `models_text_filters` from `ecosystem/fret-imui/src/tests/mod.rs`.
- [x] Remove filter-only imports from `models_text.rs`.
- [x] Run focused filter and broader text-model gates.

## Future Follow-Ons

- [ ] Split read-only/password/select-all-on-focus tests by capability family when the next text
  policy lane needs edits.
- [ ] Split textarea behavior into a dedicated module if multiline work resumes.
- [ ] Convert filter cases into a fixture harness only if future additions become a true repeated
  case table.
