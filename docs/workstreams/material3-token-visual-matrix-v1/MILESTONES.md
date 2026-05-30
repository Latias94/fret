# Material3 Token Visual Matrix v1 - Milestones

Status: Active
Last updated: 2026-05-30

## M0 - Lane Opened

Exit criteria:

- Workstream docs exist.
- Initial matrix covers all 39 components from M3PV2.
- Source precedence and matrix dimensions are explicit.
- JSON/catalog gates pass.

## M1 - Inventory Report Ready

Status: Complete as of M3TVM-020.

Exit criteria:

- Generated or scripted inventory classifies token modules, injection functions, fallback chains,
  and magic visual constants.
- At least one report row maps each component to token owner modules and expected family packet.
- No component recipe is refactored before the inventory explains the target shape.

## M2 - Fixture Harness Ready

Exit criteria:

- A fixture-driven harness can validate at least color, alpha, shape, elevation, outline, and
  typography role outcomes.
- Button and TextField have the first fixture-backed rows.
- Fixture format is reviewable without reading a large Rust test body.

## M3 - Family Rows Closed

Exit criteria:

- Field, control/chip, navigation, and overlay/surface families have matrix rows with explicit
  gate state.
- Shared foundation refactors have at least two consumer proofs.
- No Material token policy leaks into `crates/*`.

## M4 - Closeout

Exit criteria:

- Matrix rows are no longer `inventory_seeded` without a follow-on reason.
- Obsolete fallback helpers and stale tests are deleted or explicitly retained as evidence.
- Residual breadth is split into new workstreams.
