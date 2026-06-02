# Material3 Interaction Regression Harness Split v1 Handoff

Status: Closed
Last updated: 2026-05-31

## What Changed

- Added `ecosystem/fret-ui-material3/tests/material3_interaction_regressions.rs`.
- Moved 48 non-Radio tests out of `radio_alignment.rs`.
- Trimmed `radio_alignment.rs` imports and helpers to the three retained Radio tests.

## What Remains

- `material3_interaction_regressions.rs` is an intermediate owner. It should be split by component
  family in follow-ons.
- The plain TextInput regression should be audited separately for possible `fret-ui` mechanism
  ownership.

## Suggested Follow-Ons

- `material3-interaction-regression-family-split-v1`
- `material3-text-input-mechanism-test-ownership-v1`
