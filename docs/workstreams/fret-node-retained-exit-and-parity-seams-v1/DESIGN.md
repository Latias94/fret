# `fret-node` Retained Exit And Parity Seams (v1)

Status: Closed
Last updated: 2026-05-28

## Why This Lane Exists

The previous `fret-node` architecture lane deepened the graph, store, patch, canvas, and test seams,
but it intentionally left a follow-on question open: whether the retained canvas compatibility
island still earns its cost. It no longer does. New app code is binding-first/declarative-first, the
retained bridge is internal-only, and the remaining retained-only API/documentation keeps old
architecture vocabulary alive.

This lane removes that compatibility island, refreshes the public docs around the current surface,
pushes one more reusable canvas mechanism below `fret-node`, and closes one small XyFlow parity seam
around extension hooks/focus without leaking retained widget policy back into public APIs.

## Relevant Authority

- ADRs:
  - `docs/adr/0028-declarative-elements-and-element-state.md`
  - `docs/adr/0031-app-owned-models-and-leasing-updates.md`
  - `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- Existing docs:
  - `docs/node-graph-controlled-mode.md`
  - `docs/node-graph-how-to-build-like-xyflow.md`
  - `docs/node-graph-roadmap.md`
  - `docs/node-graph-xyflow-parity.md`
  - `docs/runtime-contract-matrix.md`
- Related workstreams:
  - `docs/workstreams/fret-node-architecture-fearless-refactor-v2/`
  - `docs/workstreams/fret-node-retained-canvas-mirror-cleanup-v1/`
  - `docs/workstreams/fret-node-runtime-store-contract-closure-v1/`

## Problem

`fret-node` still carries a feature-gated retained canvas implementation island, retained-context
adapter files, compatibility docs, and source-policy tests that describe the old shape. That keeps
the wrong extension point visible and makes further parity work harder to place: some generic canvas
mechanics remain in the node crate, while XyFlow-style hook/focus policy is mixed with retained
widget execution details.

## Target State

- `fret-node` no longer exposes or compiles a `compat-retained-canvas` feature.
- Old retained `NodeGraphCanvas` tutorials and API references are removed or rewritten around the
  binding/controller/declarative surfaces that downstream code should use.
- One additional domain-neutral canvas mechanism moves to `fret-canvas`, with `fret-node` retaining
  only graph-specific adapters/policy.
- XyFlow parity docs and tests identify one concrete extension/focus seam as current behavior rather
  than a retained-canvas TODO.
- Validation no longer depends on retained compatibility gates; final gates prove the headless,
  default, `fret-canvas`, formatting, clippy, and layering surfaces.

## In Scope

- Delete feature flags, modules, tests, docs, and policy guards whose only job is retained canvas
  compatibility.
- Rewrite node graph docs away from `NodeGraphCanvas::{new,with_store,with_callbacks}` as the public
  teaching surface.
- Extract a bounded generic canvas helper where the type vocabulary is not node-graph-specific.
- Add or tighten focused conformance tests for the chosen XyFlow extension/focus seam.
- Update ADR implementation evidence when behavior covered by ADR 0330 changes.

## Out Of Scope

- Rewriting all node graph UI into a final GPUI-style declarative element tree.
- Building a complete ReactFlow hook facade.
- Implementing full per-element accessibility trees for nodes, edges, ports, minimap, or controls.
- Preserving source compatibility for retained widget downstream authors.
- Running a full workspace test suite unless targeted gates expose cross-workspace risk.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The retained canvas feature is no longer a supported downstream API. | High | ADR 0330 frames retained widgets as internal/compat only; architecture v2 closeout recommends a new retained deletion lane. | Restore a smaller feature gate or split an ADR update before deleting code. |
| Default and no-default `fret-node` gates cover the supported public surface after retained deletion. | Medium | Prior lane passed `cargo nextest run -p fret-node` and `--no-default-features`; retained tests were explicitly feature-only. | Add replacement seam tests before removing retained-only coverage. |
| At least one generic helper remains extractable to `fret-canvas` without pulling graph types across the boundary. | Medium | `fret-node` still has viewport, threshold, route, culling, and interaction helpers beside extracted `fret-canvas` primitives. | Record a negative audit and keep the task as documentation/test cleanup only. |
| A narrow hook/focus parity slice can land without designing the final accessibility model. | Medium | Existing tests already cover keyboard focus cycling, focus cancel, callback hooks, and middleware. | Split full a11y semantics into a follow-on workstream. |

## Architecture Direction

The retained island is deleted rather than further hidden. `fret-node` keeps the node graph domain:
graph semantics, interaction policy, presenter hooks, and binding/controller APIs. `fret-canvas`
keeps reusable canvas mechanics such as viewport math, generic spatial/tile helpers, pan/zoom
constraints, and interaction primitives that can serve non-node canvases.

The public teaching surface is controller/store/declarative composition. Historical retained widget
constructors should not appear in new docs, tests, or examples except as removed-history notes.
Extension hooks should hang from graph callbacks, middleware, presenter contracts, or explicit
policy structs, not from retained widget contexts.

## Refactor Brief

- **Intent**: remove the last visible retained compatibility path so future node graph work can
  target binding-first/declarative composition without keeping duplicate retained runtime seams.
- **Scope**: `ecosystem/fret-node`, `ecosystem/fret-canvas`, node graph docs, ADR evidence, and this
  workstream.
- **Deletion plan**: remove `compat-retained-canvas`, retained-only context adapters, retained-only
  widget runtime files/tests, stale docs, and source-policy assertions that only guarded the compat
  island.
- **Boundary plan**: keep graph-specific behavior in `fret-node`; move reusable canvas mechanics to
  `fret-canvas`; keep `fret-ui` retained mechanisms internal and out of downstream docs.
- **Testing plan**: run focused retained-removal compile gates, `cargo nextest run -p fret-node
  --no-default-features`, `cargo nextest run -p fret-node`, `cargo nextest run -p fret-canvas`,
  `cargo fmt --check`, `cargo clippy` for touched packages, and `python3 tools/check_layering.py`.
- **Risk plan**: retained deletion may expose hidden test dependencies; recover by moving behavioral
  assertions to headless/default seams, not by preserving retained constructors.
- **Workflow plan**: durable `dev-flow/open-workstream` lane with vertical tasks; after each verified
  task, update TODO/evidence and commit only the lane's changes when appropriate.

## Closeout Condition

This lane can close when:

- retained canvas compatibility is gone or explicitly proven non-deletable with a replacement plan,
- public docs no longer teach the retained canvas API,
- the selected canvas extraction and hook/focus seam land with focused tests,
- final gates pass with fresh evidence,
- ADR/workstream evidence reflects the shipped behavior,
- and follow-on work is either split or explicitly deferred.

## Closeout Verdict

Closed on 2026-05-28. The retained compatibility island was deleted, public node graph docs were
rewritten around binding/controller/declarative composition, generic resize handle vocabulary moved
to `fret-canvas`, and `disableKeyboardA11y` now gates declarative active-descendant/a11y internals
with focused tests. Full semantic focus nodes, port-focus authoring, minimap focus, and a broader
ReactFlow hook facade remain explicit follow-on scope rather than hidden work in this lane.
