# Material3 Token Fallback Hardening v2 - Milestones

## M0 - Baseline

Complete when the lane has a source-backed inventory baseline and a bounded family slice.

Evidence:

- `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json`
- This workstream's `DESIGN.md`

## M1 - Shared Helper Slice

Complete when the chip-family common fallback behavior lives in one Material3 token helper and the
four chip token modules retain their public crate-local APIs.

Acceptance:

- No `crates/*` changes are needed.
- Recipes still call `chip_tokens`, `filter_chip_tokens`, `input_chip_tokens`, and
  `suggestion_chip_tokens`.
- The helper has unit tests for key fallback behavior.

## M2 - Inventory Proof

Complete when the inventory tool recognizes the helper as shared policy and the v2 artifact records
the count reduction for the chip-family modules.

## M3 - Verification And Closeout

Complete when focused gates pass, the lane is committed, and residual follow-ons are explicit.
