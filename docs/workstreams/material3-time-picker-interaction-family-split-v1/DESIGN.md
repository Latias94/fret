# Material3 TimePicker Interaction Family Split v1 Design

Status: Closed
Last updated: 2026-05-31

## Problem

After the first Material3 interaction-regression family split,
`material3_interaction_regressions.rs` still carried four TimePicker-owned tests beside field-family
and plain TextInput residuals. The TimePicker tests share component state, keyboard, pointer, input,
and invalid-semantics behavior, so keeping them in the residual owner made the remaining audit queue
less precise.

## Decision

Move the four TimePicker interaction tests into
`ecosystem/fret-ui-material3/tests/material3_time_picker_interactions.rs`.

Keep `material3_interaction_regressions.rs` only for the residual owner-boundary audits:

- plain TextInput event behavior;
- Autocomplete field-family behavior;
- ExposedDropdown field-family behavior.

## Boundaries

- This lane is a test-ownership refactor, not a TimePicker behavior change.
- TimePicker-specific invalid/live semantics helpers move with the TimePicker tests.
- No public Material3 API or runtime contract changes are introduced.

## Non-Goals

- Do not move Autocomplete or ExposedDropdown tests in this lane.
- Do not decide whether the plain TextInput regression belongs in `fret-ui`.
- Do not rewrite TimePicker interaction policy while moving tests.

## Follow-On Shape

The residual file now has two remaining decisions:

1. split Autocomplete and ExposedDropdown into a field-family interaction binary;
2. audit the plain TextInput test for mechanism-layer ownership.
