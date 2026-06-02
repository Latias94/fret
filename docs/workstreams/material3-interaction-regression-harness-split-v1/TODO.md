# Material3 Interaction Regression Harness Split v1 TODO

Status: Closed
Last updated: 2026-05-31

Task IDs use `M3IRS-*`.

## Tasks

- [x] M3IRS-010: Open the interaction-regression harness split lane.
  - Scope: `docs/workstreams/material3-interaction-regression-harness-split-v1`.
  - Expected result: lane records the boundary change and closeout gates.

- [x] M3IRS-020: Move non-Radio interaction regressions.
  - Scope: `radio_alignment.rs` and `material3_interaction_regressions.rs`.
  - Expected result: only Radio-owned tests remain in `radio_alignment.rs`.
  - Result: 48 non-Radio tests moved to the new interaction-regression binary.

- [x] M3IRS-030: Tighten imports and helpers.
  - Scope: `radio_alignment.rs`.
  - Expected result: no stale helper/import warnings remain after the split.
  - Result: Radio imports now match the three retained tests.

- [x] M3IRS-040: Verify and close.
  - Scope: focused nextest gates, check/clippy, catalog, layering, diff hygiene.
  - Expected result: committed split with clean worktree.

## Notes

- This is an ownership split, not a behavior refactor.
- The next cleanup should split `material3_interaction_regressions.rs` into family-owned files.
