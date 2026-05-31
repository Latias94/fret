# Material3 Interaction Regression Family Split v1 TODO

Status: Closed
Last updated: 2026-05-31

Task IDs use `M3IRF-*`.

## Tasks

- [x] M3IRF-010: Open the family split lane.
  - Scope: `docs/workstreams/material3-interaction-regression-family-split-v1`.
  - Expected result: lane records the first family-owned split and residual ownership boundary.

- [x] M3IRF-020: Move navigation interaction regressions.
  - Scope: `material3_interaction_regressions.rs` to
    `material3_navigation_interactions.rs`.
  - Result: 11 navigation tests now live in the navigation-owned binary.

- [x] M3IRF-030: Move overlay interaction regressions.
  - Scope: `material3_interaction_regressions.rs` to `material3_overlay_interactions.rs`.
  - Result: 12 overlay tests now live in the overlay-owned binary.

- [x] M3IRF-040: Move choice/action interaction regressions.
  - Scope: `material3_interaction_regressions.rs` to
    `material3_choice_action_interactions.rs`.
  - Result: 15 choice/action tests now live in the choice/action-owned binary.

- [x] M3IRF-050: Keep residual tests explicit.
  - Scope: `material3_interaction_regressions.rs`.
  - Result: 10 residual TextInput, TimePicker, Autocomplete, and ExposedDropdown tests remain for
    follow-on ownership audits.

- [x] M3IRF-060: Verify and close.
  - Scope: focused nextest gates, package check/clippy, catalog, layering, and diff hygiene.
  - Expected result: committed split with clean worktree.

## Notes

- This lane intentionally uses family-level binaries rather than per-component binaries because the
  moved tests share interaction harness setup inside each family.
- The residual file is not a dumping ground; it is the next audit queue.
