# `fret-node` Declarative Contract Closure v1

Status: Active
Last updated: 2026-05-28

## Why This Lane Exists

The retained `NodeGraphCanvas` compatibility island is gone, and `fret-node` now teaches
binding/controller/declarative composition as the supported path. The next risk is quieter:
historical ADRs, standalone gap analysis, store dispatch code, binding mirrors, and declarative
surface orchestration can still pull future work back toward deleted retained vocabulary or duplicate
state paths.

This lane closes that gap before deeper feature work. The goal is not to preserve compatibility with
old retained APIs; the goal is to make the shipped declarative/store-first contract smaller, harder
to bypass, and easier to refactor.

## Relevant Authority

- ADRs:
  - `docs/adr/0028-declarative-elements-and-element-state.md`
  - `docs/adr/0031-app-owned-models-and-leasing-updates.md`
  - `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
  - `docs/adr/0135-node-graph-canvas-middleware.md`
  - `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- Existing docs:
  - `docs/node-graph-how-to-build-like-xyflow.md`
  - `docs/node-graph-xyflow-parity.md`
  - `docs/workstreams/standalone/xyflow-gap-analysis.md`
- Related workstreams:
  - `docs/workstreams/fret-node-architecture-fearless-refactor-v2/`
  - `docs/workstreams/fret-node-runtime-store-contract-closure-v1/`
  - `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/`

## Problem

The current code and tests prove the retained node graph canvas is removed, but parts of the
architecture record still describe it as the current turnkey surface. At the same time,
`NodeGraphStore` has duplicate dispatch/profile pipelines, `NodeGraphSurfaceBinding` still carries
graph/view/config mirrors beside the authoritative store, and the paint-only declarative surface has
large orchestration modules where planning and host side effects remain tightly coupled.

## Target State

- Standalone gap analysis and ADR records no longer teach deleted retained `NodeGraphCanvas`
  surfaces as current guidance.
- Source-policy tests cover the docs most likely to drift back toward retained node graph authoring.
- Store dispatch has one internal commit path for profile and non-profile transactions.
- Binding mirrors either become clearly advanced compatibility state or shrink behind a store-first
  projection boundary.
- Declarative UI interaction interception is designed as a new supported surface, not a revival of
  `NodeGraphCanvasMiddleware`.
- Paint-only orchestration is split around pure frame/scene plans where that materially reduces
  risk.

## In Scope

- `docs/workstreams/standalone/xyflow-gap-analysis.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `docs/adr/0135-node-graph-canvas-middleware.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/ui/binding*.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only*`
- `ecosystem/fret-canvas` only when a helper is proven domain-neutral

## Out Of Scope

- Reopening retained widget public authoring for node graphs.
- Reintroducing `compat-retained-canvas`.
- Building a complete ReactFlow hook facade in one task.
- Full semantic focus tree work for nodes, ports, minimap, and controls.
- Collaboration/CRDT backends or external persistence format changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Retained `NodeGraphCanvas` is deleted and should stay deleted. | High | `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/CLOSEOUT_AUDIT_2026-05-28.md`, `ecosystem/fret-node/src/surface_policy_tests.rs` | Stop and update ADR 0330 before any retained reintroduction. |
| Store dispatch duplication can be reduced without changing public APIs. | Medium | `ecosystem/fret-node/src/runtime/store.rs` has parallel profile/non-profile methods | Split a narrower proof task if profile ownership blocks a single helper. |
| Binding mirrors still exist for app/model integration, not because they are the true source. | Medium | `ecosystem/fret-node/src/ui/binding.rs`, runtime-store closure workstream | Preserve the advanced API but document or isolate it rather than deleting blindly. |
| A declarative interaction hook should be designed before implementation. | High | ADR 0135 retained middleware is obsolete; xyflow uses store/action middleware maps | First task should supersede stale ADR guidance and identify the correct new boundary. |

## Architecture Direction

Keep the long-term shape store-first and declarative-first:

1. Documentation and ADRs must describe the shipped contract before code refactors widen it.
2. Store mutation remains the one graph commit gate.
3. UI interaction hooks may intercept or propose commands, but committed graph edits still flow
   through `NodeGraphStore`.
4. Binding mirrors are treated as integration projections, not as another graph authority.
5. Pure geometry, cache, route, and edge helpers move to `fret-canvas` only when at least one
   non-node consumer or domain-neutral proof exists.

## Closeout Condition

This lane can close when:

- retained-current-fact drift is fixed and guarded by tests,
- the store dispatch duplication is reduced or explicitly split with evidence,
- binding mirror ownership is either shrunk or documented behind a stable advanced boundary,
- declarative interaction hook direction has a contract and at least one focused proof,
- paint-only orchestration has at least one meaningful pure-plan extraction or a recorded negative
  audit,
- final gates pass with fresh evidence,
- and follow-ons are split instead of silently expanding this lane.
