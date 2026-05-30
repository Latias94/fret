# IMUI Demo Metrics Debug Action Metadata v1 - Milestones

Status: Closed
Last updated: 2026-05-30

## M1 - Metadata Contract

Status: Completed on 2026-05-30.

Exit criteria:

- CLI JSON exposes `action_metadata_doc` on the `demo-metrics-debug` route.
- Each route action exposes stable `id`, `category`, `primary`, and `requires_bundle` fields.
- DevTools GUI and MCP first-open resources surface the same metadata.
- Existing copyable action command text remains label-and-command only for easy shell use.
- Product-chain and first-open gates check the metadata.

## M2 - GUI Execution Readiness

Status: Completed on 2026-05-30 as a readiness projection.

Exit criteria:

- DevTools can decide whether an action is immediately runnable from metadata alone.
- Bundle-backed actions are visibly separate from demo/product-gate actions.
- Any actual execution control reuses existing diagnostics job infrastructure instead of inventing
  a route-local runner.

## M3 - Closeout or Split

Status: Completed on 2026-05-30.

Exit criteria:

- If metadata is sufficient, close this lane with a short audit.
- If a shared command palette or generalized action registry is required, start a narrower
  follow-on and keep this lane focused on the Demo/Metrics/Debug action metadata.
