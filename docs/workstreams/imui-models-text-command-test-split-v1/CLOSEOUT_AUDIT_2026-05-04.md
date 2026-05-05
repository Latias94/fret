# ImUi Models Text Command Test Split v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the fourth mechanical `models_text.rs` decomposition slice by isolating IMUI
single-line input command-policy coverage.

## What Shipped

- Added `ecosystem/fret-imui/src/tests/models_text_commands.rs`.
- Moved completion, history, undo/redo, and repeat opt-in command-policy tests into the new module.
- Registered the new module from `ecosystem/fret-imui/src/tests/mod.rs`.
- Left runtime command dispatch and public IMUI APIs unchanged.

## Proof

- `cargo nextest run -p fret-imui models_text_commands --no-fail-fast` passes the focused command
  module.
- `cargo nextest run -p fret-imui models_text --no-fail-fast` passes the broader text-model filter,
  including the moved picker, filter, mode, and command modules that still match the filter name.

## Remaining Work

Start narrower follow-ons for:

- splitting textarea behavior when multiline work resumes,
- splitting focus/edit lifecycle tests separately from basic changed-signal tests,
- and introducing fixtures only if future command coverage becomes a repetitive key/chord matrix.
