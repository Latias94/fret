# Material3 Field Interaction Family Split v1 TODO

Status: Closed
Last updated: 2026-05-31

Task IDs use `M3FIF-*`.

## Tasks

- [x] M3FIF-010: Open the field-family split lane.
  - Scope: `docs/workstreams/material3-field-interaction-family-split-v1`.
  - Expected result: lane records the field-family split and final TextInput residual boundary.

- [x] M3FIF-020: Move field-family interaction tests.
  - Scope: `material3_interaction_regressions.rs` to `material3_field_interactions.rs`.
  - Result: 5 Autocomplete/ExposedDropdown tests now live in a field-family-owned binary.

- [x] M3FIF-030: Tighten residual imports.
  - Scope: `material3_interaction_regressions.rs`.
  - Result: residual imports now match the single plain TextInput test only.

- [x] M3FIF-040: Verify and close.
  - Scope: focused nextest gates, package check/clippy, catalog, layering, and diff hygiene.
  - Expected result: committed split with clean worktree.

## Notes

- This split leaves exactly one residual test on purpose: the plain TextInput mechanism ownership
  audit should not be mixed with Material3 field-family test ownership.
