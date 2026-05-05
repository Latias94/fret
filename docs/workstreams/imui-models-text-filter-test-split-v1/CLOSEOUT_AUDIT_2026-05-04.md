# ImUi Models Text Filter Test Split v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the second mechanical `models_text.rs` decomposition slice by isolating IMUI
single-line input filter coverage.

## What Shipped

- Added `ecosystem/fret-imui/src/tests/models_text_filters.rs`.
- Moved named filter, numeric filter, and custom insertion-filter tests into the new module.
- Registered the new module from `ecosystem/fret-imui/src/tests/mod.rs`.
- Removed filter-only imports from `models_text.rs`.

## Proof

- `cargo nextest run -p fret-imui models_text_filters --no-fail-fast` passes the focused filter
  module.
- `cargo nextest run -p fret-imui models_text --no-fail-fast` passes the broader text-model filter,
  including the moved filter and picker modules that still match the filter name.

## Remaining Work

Start narrower follow-ons for:

- splitting read-only/password/select-all and command-policy tests by capability family,
- splitting textarea behavior when multiline work resumes,
- and converting filter cases to fixtures only if the suite becomes a repeated data matrix.
