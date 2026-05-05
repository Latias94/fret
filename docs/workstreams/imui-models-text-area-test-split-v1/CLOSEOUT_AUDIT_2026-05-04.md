# ImUi Models Text Area Test Split v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the fifth mechanical `models_text.rs` decomposition slice by isolating IMUI
multiline text-area coverage.

## What Shipped

- Added `ecosystem/fret-imui/src/tests/models_text_area.rs`.
- Moved textarea read-only, Tab policy, changed-signal, and lifecycle tests into the new module.
- Registered the new module from `ecosystem/fret-imui/src/tests/mod.rs`.
- Removed the `TextAreaOptions` import from `models_text.rs`.

## Proof

- `cargo nextest run -p fret-imui models_text_area --no-fail-fast` passes the focused textarea
  module.
- `cargo nextest run -p fret-imui models_text --no-fail-fast` passes the broader text-model filter,
  including the moved picker, filter, mode, command, and textarea modules that still match the
  filter name.

## Remaining Work

Start narrower follow-ons for:

- splitting remaining single-line focus/bounds/lifecycle tests,
- splitting push-id reorder stability separately from the basic changed-signal test,
- and introducing fixtures only if future multiline coverage becomes a repeated option matrix.
