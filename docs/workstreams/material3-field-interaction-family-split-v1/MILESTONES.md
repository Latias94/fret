# Material3 Field Interaction Family Split v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M1: Field-Family Ownership Split

Exit criteria:

- All Autocomplete tests are isolated in `material3_field_interactions.rs`.
- All ExposedDropdown tests are isolated in `material3_field_interactions.rs`.
- `material3_interaction_regressions.rs` contains only the plain TextInput residual test.

Status: Complete.

## M2: Residual Hygiene

Exit criteria:

- Residual imports do not include field-family overlay, pointer, keyboard, or semantics helpers.
- Focused check proves both binaries compile independently.
- No production APIs or shared harness APIs are widened.

Status: Complete.

## M3: Closeout Evidence

Exit criteria:

- Focused nextest gates pass for the field-family binary and residual binary.
- Package-level check/clippy gates pass.
- Workstream catalog, layering, JSON, and diff hygiene gates pass.

Status: Complete.
