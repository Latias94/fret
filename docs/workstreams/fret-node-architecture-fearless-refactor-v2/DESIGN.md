# `fret-node` Architecture Fearless Refactor v2

Status: Complete
Last updated: 2026-05-27

Closeout: the six fearless refactor themes landed in `FNAR-020` through `FNAR-070`. The lane closes
with mutation, store, state, patch, canvas, and test seams deepened and verified by fresh package
and layering gates recorded in `EVIDENCE_AND_GATES.md`.

## Why This Lane Exists

`fret-node` has enough working surface area to expose a deeper architecture problem: mutation,
store ownership, controlled synchronization, editor policy, canvas mechanisms, and source-policy
tests are all coupled through historical compatibility paths. This lane exists to make the crate
future-facing rather than compatibility-centered.

The premise of this lane is explicit: historical compatibility is not a constraint. Code can be
deleted, public surfaces can be broken, and modules can be reshaped when that produces a smaller
and deeper long-term interface.

## Relevant Authority

- ADRs:
  - `docs/adr/0028-declarative-elements-and-element-state.md`
  - `docs/adr/0031-app-owned-models-and-leasing-updates.md`
  - `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
  - `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Existing docs:
  - `docs/node-graph-roadmap.md`
  - `docs/node-graph-xyflow-parity.md`
  - `docs/node-graph-controlled-mode.md`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/xyflow-interface-review.md`
- Related workstreams:
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1`
  - `docs/workstreams/fret-node-runtime-store-contract-closure-v1`
  - `docs/workstreams/fret-node-retained-canvas-mirror-cleanup-v1`
- Reference sources:
  - `repo-ref/xyflow`
  - `repo-ref/egui-snarl`

## Problem

The current crate works, but several modules are shallow:

- raw graph operations expose ordering and ownership invariants to callers,
- `NodeGraphStore` competes with UI mirrors instead of being the single source of truth,
- the persisted graph document contains editor policy and view-like facts,
- `NodeGraphChanges` is named like a full change stream but only carries node/edge deltas,
- generic canvas mechanisms live inside a domain UI package,
- large source-text policy tests freeze implementation shape instead of testing stable seams.

## Target State

When this workstream closes:

- graph mutation flows through one canonical mutation module that owns structural invariants,
  validation, inverse generation, diffing, and projection,
- `NodeGraphStore` is authoritative for document, view, editor config, internals revisions, and
  document replacement events,
- headless graph document state is split from editor policy state and derived UI state,
- the primary commit event is a full-fidelity patch stream, with XYFlow-style node/edge changes as
  an adapter,
- reusable canvas mechanisms move below `fret-node` where they can serve other editor-grade UI,
- retained compatibility and source-text policy scaffolding is deleted or reduced to narrow gates,
- tests exercise seams through compile, behavior, transaction, event, and diagnostics evidence.

## In Scope

- `ecosystem/fret-node/src/core`
- `ecosystem/fret-node/src/ops`
- `ecosystem/fret-node/src/runtime`
- `ecosystem/fret-node/src/io`
- `ecosystem/fret-node/src/ui`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `ecosystem/fret-canvas` only when a reusable canvas mechanism is extracted from `fret-node`
- node graph docs, roadmap, parity, controlled-mode, and workstream docs

## Out Of Scope

- shadcn or Material component surface work that is not required by node graph behavior,
- unrelated rendering backend changes,
- adding a collaboration backend,
- preserving deprecated public names solely for compatibility.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `fret-node` is a Domain UI Package, not Runtime Substrate. | High | `CONTEXT.md`, `docs/node-graph-roadmap.md` | Generic mechanisms must move to `fret-canvas` or lower layers. |
| Graph mutation is the highest-leverage first seam. | High | `ops/apply.rs`, `ops/diff.rs`, `runtime/changes.rs`, audit report | Store/UI refactors remain fragile without a canonical patch model. |
| Breaking compatibility is allowed for this lane. | High | User request on 2026-05-27 | Deprecated retained and mirror paths can be deleted instead of shimmed. |
| Existing tests are broad enough to catch accidental behavior loss, but too source-shape-heavy. | Medium | `surface_policy_tests.rs`, nextest baseline | Some gates must be replaced before deleting source-policy tests. |
| Some canvas extraction may touch `fret-canvas`. | Medium | XYFlow A-layer review and current `ui/canvas/*` ownership | The lane may need a narrow cross-crate task rather than a pure `fret-node` task. |

## Architecture Direction

The lane deepens modules in this order:

1. Mutation first: callers should not assemble raw structural operation sequences that require
   hidden ordering knowledge.
2. Store second: once mutations are canonical, the store can become authoritative without mirror
   drift.
3. State split third: document, editor policy, view state, and derived internals must have separate
   persistence and invalidation responsibilities.
4. Event stream fourth: full-fidelity patch events become primary; lossy compatibility views become
   adapters.
5. Canvas extraction fifth: generic mechanisms move below the Domain UI Package.
6. Test cleanup last: source-text policy tests shrink only after behavior seams have replacement
   evidence.

## Closeout Condition

This lane can close when:

- all six refactor themes have landed or been explicitly split into narrower follow-ons,
- targeted and package gates pass with fresh evidence,
- docs describe the shipped architecture,
- `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` agree,
- review and verification have no blocking findings.
