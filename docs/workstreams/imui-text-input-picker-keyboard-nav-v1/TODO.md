# ImUi Text Input Picker Keyboard Navigation v1 TODO

Status: Closed
Last updated: 2026-05-04

## Completed

- [x] Add picker-owned active candidate state without moving candidate storage into runtime.
- [x] Add ArrowDown/ArrowUp active movement while keeping focus in the input field.
- [x] Add Enter/NumpadEnter commit of the active candidate.
- [x] Render the active candidate using the existing selected row visual/semantics path.
- [x] Preserve Enter submit behavior when the picker has no candidates.
- [x] Add focused IMUI tests for completion keyboard commit, history wrap, and no-candidate pass-through.
- [x] Record gates and closeout evidence.

## Follow-On Candidates

- Editor-owned completion ranking and history persistence.
- Richer active-descendant accessibility wiring once the picker needs a deeper a11y contract.
- Dismissed-query policy so history pickers can close and stay closed after commit.
- Multiline completion/history conflict handling.
