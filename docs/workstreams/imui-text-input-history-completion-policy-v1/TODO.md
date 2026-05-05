# ImUi Text Input History Completion Policy v1 TODO

Status: Closed
Last updated: 2026-05-04

## M0 - Source Mapping

- [x] Confirm Dear ImGui maps completion to Tab and history to Up/Down in `InputTextEx`.
- [x] Keep this slice out of `crates/fret-ui` because it is command policy, not text mechanism.

## M1 - IMUI Option Surface

- [x] Add `InputTextOptions::completion_command`.
- [x] Add `InputTextOptions::history_previous_command`.
- [x] Add `InputTextOptions::history_next_command`.
- [x] Add repeat opt-ins for completion and history commands.

## M2 - Key Policy

- [x] Dispatch completion on unmodified Tab.
- [x] Dispatch previous/next history commands on unmodified Up/Down.
- [x] Ignore modified keys and IME composition.
- [x] Suppress repeated keydown by default.

## M3 - Gates And Closeout

- [x] Add focused `fret-imui` model tests.
- [x] Update IMUI gap audit and workstream indexes.
- [x] Leave follow-on policy for filters, undo/redo command routing, and full completion/history UI
  recipes.
