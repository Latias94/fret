# Material3 FAB Token Defaults v1 - Milestones

## M0 - Baseline

Complete when the lane records the inventory baseline and identifies the FAB default matrices to
extract.

## M1 - Helper Slice

Complete when FAB visual default matrices live in one private helper and `fab.rs` keeps its current
token API.

Acceptance:

- No `crates/*` changes are needed.
- `Fab` recipe behavior remains unchanged.
- The helper has focused tests for representative size, shape, spacing, and opacity defaults.

## M2 - Inventory Proof

Complete when the inventory tool recognizes the helper separately and a lane artifact records the
new FAB fallback/magic-constant counts.

## M3 - Verification And Closeout

Complete when focused gates pass, the lane is committed, and residual follow-ons are explicit.
