# Material3 TimePicker Interaction Family Split v1 TODO

Status: Closed
Last updated: 2026-05-31

Task IDs use `M3TPIF-*`.

## Tasks

- [x] M3TPIF-010: Open the TimePicker family split lane.
  - Scope: `docs/workstreams/material3-time-picker-interaction-family-split-v1`.
  - Expected result: lane records the TimePicker split and residual audit boundary.

- [x] M3TPIF-020: Move TimePicker interaction tests.
  - Scope: `material3_interaction_regressions.rs` to
    `material3_time_picker_interactions.rs`.
  - Result: 4 TimePicker tests now live in a TimePicker-owned binary.

- [x] M3TPIF-030: Move TimePicker-only helpers.
  - Scope: semantics invalid/label/live helpers.
  - Result: invalid/live helper functions moved with the TimePicker input validation tests.

- [x] M3TPIF-040: Tighten residual imports.
  - Scope: `material3_interaction_regressions.rs`.
  - Result: residual imports now match TextInput and field-family tests only.

- [x] M3TPIF-050: Verify and close.
  - Scope: focused nextest gates, package check/clippy, catalog, layering, and diff hygiene.
  - Expected result: committed split with clean worktree.

## Notes

- This split is intentionally narrow: it does not answer the field-family or plain TextInput
  ownership questions.
