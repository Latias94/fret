# Material3 Slider Token Defaults v1 - Milestones

## M0 - Baseline

Complete when the lane records the inventory baseline and identifies the Slider default matrices to
extract.

## M1 - Helper Slice

Complete when Slider visual default matrices live in one private helper and `slider.rs` keeps its
current token API.

Acceptance:

- No `crates/*` changes are needed.
- `Slider` behavior remains unchanged.
- The helper has focused tests for representative state-layer, tick/stop, track, and handle
  defaults.

## M2 - Inventory Proof

Complete when the inventory tool recognizes the helper separately and a lane artifact records the
new Slider fallback/magic-constant counts.

## M3 - Verification And Closeout

Complete when focused gates pass, the lane is committed, and residual follow-ons are explicit.
