# ImUi Models Text Filter Test Split v1

Status: Closed narrow test-architecture follow-on
Last updated: 2026-05-04

This lane records the second small split of the growing `fret-imui` text-model test surface. It
follows the picker split by isolating named/custom filter tests into their own module.

## Ownership

- `fret-imui` owns the proof tests and test module registration.
- `fret-ui-kit::imui` continues to own `InputTextFilters` and `InputTextCustomFilter` behavior.
- No runtime, kit policy, or public app-facing API is in scope.

## Must-Be-True Outcomes

- Named filter, numeric filter, and custom insertion-filter tests live in
  `models_text_filters.rs`.
- `models_text.rs` no longer carries filter-specific test bodies or filter-only imports.
- `tests/mod.rs` registers `models_text_filters`.
- Focused filter tests and the broader `models_text` filter both pass.

## Fixture Decision

The filter tests are closer to a future data matrix than the picker tests, but this slice keeps the
existing Rust harness because the current assertions still drive real model-backed text input
through the IMUI host. Convert to fixtures only when additional filter cases become repetitive
enough to justify stable `cases[].id` output.

## Non-Goals

- No change to `InputTextFilters`.
- No change to `InputTextCustomFilter`.
- No fixture schema in this slice.
- No decomposition of read-only, lifecycle, or textarea tests yet.
