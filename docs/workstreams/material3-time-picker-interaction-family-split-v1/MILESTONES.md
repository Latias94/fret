# Material3 TimePicker Interaction Family Split v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M1: TimePicker Ownership Split

Exit criteria:

- All four TimePicker tests are isolated in `material3_time_picker_interactions.rs`.
- TimePicker-only helper functions move with the new binary.
- `material3_interaction_regressions.rs` retains only TextInput and field-family residual tests.

Status: Complete.

## M2: Residual Hygiene

Exit criteria:

- Residual imports do not include TimePicker-only helpers or event utilities.
- Focused check proves both binaries compile independently.
- No production APIs or shared harness APIs are widened.

Status: Complete.

## M3: Closeout Evidence

Exit criteria:

- Focused nextest gates pass for the TimePicker binary and residual binary.
- Package-level check/clippy gates pass.
- Workstream catalog, layering, JSON, and diff hygiene gates pass.

Status: Complete.
