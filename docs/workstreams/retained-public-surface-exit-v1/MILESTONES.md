# Retained Public Surface Exit v1 - Milestones

Status: Active
Last updated: 2026-05-25

## M0 - ADR Accepted

Exit criteria:

- ADR 0330 exists and is accepted.
- ADR README and implementation alignment matrix reference it.

Status: Complete.

## M1 - Default Root Surface Shrunk

Exit criteria:

- `Widget`, `EventCx`, `CommandCx`, `CommandAvailabilityCx`, `LayoutCx`, `PrepaintCx`,
  `PaintCx`, and `SemanticsCx` are feature-gated.
- `Invalidation` and `CommandAvailability` remain available by default.
- Source-policy test proves the export shape.

Status: Complete.

## M2 - Node Compat Island Explicit

Exit criteria:

- `fret-node/compat-retained-canvas` enables `fret-ui/compat-retained-widgets`.
- Node retained canvas compatibility still compiles.
- Policy wording no longer calls retained root exports stable/default authoring API.

Status: Complete.

## M3 - Adapter Follow-On Assigned

Exit criteria:

- Node low-level adapter migration has an owner lane.
- This lane does not grow into a full node graph rewrite.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
