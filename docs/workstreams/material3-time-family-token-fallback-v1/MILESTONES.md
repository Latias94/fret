# Material3 Time Family Token Fallback v1 - Milestones

## M0 - Baseline

Complete when the lane records the current inventory and identifies the duplicated period selector
fallback policy in `time_picker` and `time_input`.

## M1 - Shared Helper Slice

Complete when period selector fallback logic lives in one Material3 token helper and both time token
modules keep their existing crate-local API.

Acceptance:

- No `crates/*` changes are needed.
- Recipes and visual fixtures continue to call the existing time token module functions.
- The helper has focused tests for divergent prefixes and shared state-layer fallback behavior.

## M2 - Inventory Proof

Complete when the inventory tool recognizes the helper as shared policy and a lane artifact records
the resulting fallback/magic-constant counts.

## M3 - Verification And Closeout

Complete when focused gates pass, the lane is committed, and residual follow-ons are explicit.
