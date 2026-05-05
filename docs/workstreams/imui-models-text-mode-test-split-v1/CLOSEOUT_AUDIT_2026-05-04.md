# ImUi Models Text Mode Test Split v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the third mechanical `models_text.rs` decomposition slice by isolating IMUI
single-line input mode coverage.

## What Shipped

- Added `ecosystem/fret-imui/src/tests/models_text_modes.rs`.
- Moved read-only, select-all-on-focus, and password-mode tests into the new module.
- Registered the new module from `ecosystem/fret-imui/src/tests/mod.rs`.
- Removed the `InputTextMode` import from `models_text.rs`.

## Proof

- `cargo nextest run -p fret-imui models_text_modes --no-fail-fast` passes the focused text mode
  module.
- `cargo nextest run -p fret-imui models_text --no-fail-fast` passes the broader text-model filter,
  including the moved picker, filter, and mode modules that still match the filter name.

## Remaining Work

Start narrower follow-ons for:

- splitting completion/history/undo command-policy tests by capability family,
- splitting textarea behavior when multiline work resumes,
- and splitting focus/edit lifecycle tests separately from basic changed-signal tests.
