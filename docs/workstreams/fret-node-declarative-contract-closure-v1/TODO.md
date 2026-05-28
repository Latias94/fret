# `fret-node` Declarative Contract Closure v1 - TODO

Status: Active
Last updated: 2026-05-28

Task IDs use `FNDC` for node declarative contract closure.

## Cross-cutting Guardrails

- [ ] Keep `NodeGraphStore` as the graph commit authority.
- [ ] Do not reintroduce retained node graph public authoring or `compat-retained-canvas`.
- [ ] Prefer deleting stale retained guidance over compatibility shims.
- [ ] Keep headless runtime/store surfaces free of `fret-ui`.
- [ ] Add focused gates before shrinking source-policy or public surfaces.

## M0 - Scope And Evidence Freeze

- [x] FNDC-010 [owner=planner] [deps=none] [scope=docs/workstreams/fret-node-declarative-contract-closure-v1]
  Goal: Freeze the lane problem, target state, task ledger, evidence anchors, and first executable
  task.
  Validation:
  - `python3 -m json.tool docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
  - `python3 tools/check_workstream_catalog.py`
  Evidence:
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/DESIGN.md`
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/TODO.md`
  Handoff: DONE. Start implementation with `FNDC-020`.

## M1 - Retained Current-Fact Drift Closure

- [x] FNDC-020 [owner=codex] [deps=FNDC-010] [scope=docs/workstreams/standalone/xyflow-gap-analysis.md,docs/adr/0128-canvas-widgets-and-interactive-surfaces.md,docs/adr/0135-node-graph-canvas-middleware.md,docs/adr/IMPLEMENTATION_ALIGNMENT.md,ecosystem/fret-node/src/surface_policy_tests.rs]
  Goal: Rewrite stale retained `NodeGraphCanvas` current guidance around the shipped
  binding/controller/declarative surface and add a source-policy gate for standalone/ADR drift.
  Validation:
  - `cargo nextest run -p fret-node --no-default-features retained_node_graph_current_guidance_stays_declarative`
  - `python3 -m json.tool docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
  - `python3 tools/check_workstream_catalog.py`
  Review: DONE. Historical retained names remain only in historical/deleted context.
  Evidence:
  - `docs/workstreams/standalone/xyflow-gap-analysis.md`
  - `docs/adr/0135-node-graph-canvas-middleware.md`
  - `ecosystem/fret-node/src/surface_policy_tests.rs`
  Handoff: DONE. Continue with `FNDC-030` store dispatch commit-path consolidation.

## M2 - Store Dispatch Commit Path

- [x] FNDC-030 [owner=codex] [deps=FNDC-020] [scope=ecosystem/fret-node/src/runtime/store.rs,ecosystem/fret-node/src/runtime/tests.rs]
  Goal: Collapse duplicate profile/non-profile dispatch implementation into one internal commit
  path without changing public `NodeGraphStore` APIs.
  Validation:
  - `cargo nextest run -p fret-node store_dispatch store_middleware store_rejects`
  - `cargo nextest run -p fret-node --no-default-features`
  Review: DONE. The helper keeps middleware ordering, transaction normalization, history recording,
  and patch publication under one internal dispatch commit path.
  Evidence:
  - `ecosystem/fret-node/src/runtime/store.rs`
  - `ecosystem/fret-node/src/runtime/tests.rs`
  Handoff: DONE. Undo/redo profile/non-profile unification remains a possible follow-on; this task
  intentionally kept the public dispatch API stable and scoped the refactor to dispatch.

## M3 - Binding Mirror Ownership

- [x] FNDC-040 [owner=codex] [deps=FNDC-030] [scope=ecosystem/fret-node/src/ui/binding*.rs,ecosystem/fret-node/src/ui/controller*.rs,docs/node-graph-*.md]
  Goal: Reclassify, shrink, or isolate graph/view/editor-config mirrors so the public story remains
  store-first and downstream code does not treat mirrors as a second authority.
  Validation:
  - `cargo nextest run -p fret-node binding_surface controller_surface public_node_graph_guides`
  - `cargo nextest run -p fret-node`
  Review: DONE. The app model handles are now named and documented as store-derived projections for
  observation and explicit sync, not as a second graph authority.
  Evidence:
  - `ecosystem/fret-node/src/ui/binding.rs`
  - `ecosystem/fret-node/src/ui/binding_store_sync.rs`
  - `ecosystem/fret-node/src/ui/binding_viewport.rs`
  - `ecosystem/fret-node/src/ui/controller.rs`
  - `ecosystem/fret-node/src/surface_policy_tests.rs`
  - `docs/node-graph-how-to-build-like-xyflow.md`
  - `docs/node-graph-controlled-mode.md`
  Handoff: DONE. Continue with `FNDC-050` declarative interaction hook contract.

## M4 - Declarative Interaction Hook Contract

- [ ] FNDC-050 [owner=codex] [deps=FNDC-020,FNDC-030] [scope=docs/adr,ecosystem/fret-node/src/ui/declarative,ecosystem/fret-node/src/runtime]
  Goal: Replace the obsolete retained `NodeGraphCanvasMiddleware` direction with a declarative
  interaction hook contract that cannot bypass store commits.
  Validation:
  - focused hook contract test or compile gate
  - `cargo nextest run -p fret-node`
  Review: Reject any hook that mutates `Graph` directly or becomes a second store.
  Evidence:
  - new or updated ADR evidence
  - focused test path
  Handoff: Keep broad ReactFlow hook facade work as follow-on unless a minimal proof lands.

## M5 - Paint-only Orchestration Split

- [ ] FNDC-060 [owner=codex] [deps=FNDC-030] [scope=ecosystem/fret-node/src/ui/declarative/paint_only*,ecosystem/fret-canvas]
  Goal: Extract one meaningful pure frame/scene plan or record a negative audit that explains why
  the current paint-only orchestration should remain as-is.
  Validation:
  - `cargo nextest run -p fret-node node_graph_surface cache paint_only`
  - `cargo nextest run -p fret-canvas` if shared helpers move down
  Review: The extraction must reduce host-side coupling or generic duplication, not just move lines.
  Evidence:
  - `ecosystem/fret-node/src/ui/declarative/paint_only/*`
  - optional `ecosystem/fret-canvas/src/*`
  Handoff: Split large paint/cache work into narrower adapter lanes if needed.

## M6 - Closeout

- [ ] FNDC-070 [owner=planner] [deps=FNDC-020,FNDC-030,FNDC-040,FNDC-050,FNDC-060] [scope=docs/workstreams/fret-node-declarative-contract-closure-v1]
  Goal: Verify the lane, update evidence, and close or split remaining work.
  Validation:
  - `cargo fmt --check`
  - `cargo nextest run -p fret-node --no-default-features`
  - `cargo nextest run -p fret-node`
  - `cargo nextest run -p fret-canvas`
  - `python3 tools/check_layering.py`
  - `python3 tools/check_workstream_catalog.py`
  Review: Use `review-workstream` and `verify-rust-workstream` before closing.
  Evidence:
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
  Handoff: Close this lane only after broader ReactFlow/a11y follow-ons are explicit.
