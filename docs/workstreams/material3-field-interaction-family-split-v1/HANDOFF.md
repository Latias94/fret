# Material3 Field Interaction Family Split v1 Handoff

Status: Closed
Last updated: 2026-05-31

## What Changed

- Added `material3_field_interactions.rs` with five field-family interaction tests.
- Moved three Autocomplete tests and two ExposedDropdown tests out of
  `material3_interaction_regressions.rs`.
- Reduced `material3_interaction_regressions.rs` to one residual plain TextInput test.

## What Remains

- The plain TextInput event regression should be audited for possible `fret-ui` mechanism-layer
  ownership.
- If that test moves, `material3_interaction_regressions.rs` can be deleted.

## Suggested Follow-Ons

- `material3-text-input-mechanism-test-ownership-v1`
