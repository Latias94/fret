# Material3 Headless Golden Harness Split v1 TODO

Status: Closed
Last updated: 2026-05-31

Task IDs use `M3HGS-*`.

## Tasks

- [x] M3HGS-010: Open the harness split lane.
  - Scope: `docs/workstreams/material3-headless-golden-harness-split-v1`.
  - Expected result: lane records the boundary change and closeout gates.

- [x] M3HGS-020: Extract broad headless golden suites.
  - Scope: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` and
    `ecosystem/fret-ui-material3/tests/material3_headless_goldens.rs`.
  - Expected result: `material3_headless_*_suite_goldens_v1` tests live in a purpose-owned test
    binary, and `radio_alignment` no longer carries broad golden refresh churn.
  - Result: 21 broad headless suites were moved.

- [x] M3HGS-030: Preserve maintenance semantics.
  - Scope: ignored navigation and overlay broad golden suites.
  - Expected result: stale broad suites remain opt-in maintenance tests after migration.
  - Result: both ignored suites stayed ignored in `material3_headless_goldens`.

- [x] M3HGS-040: Verify and close.
  - Scope: focused nextest gates, check/clippy, catalog, layering, and diff hygiene.
  - Expected result: committed split with a clean worktree.

## Notes

- This is a harness ownership refactor, not a visual-golden refresh.
- The next cleanup should decide ownership for the remaining non-Radio interaction regressions in
  `radio_alignment.rs`.
