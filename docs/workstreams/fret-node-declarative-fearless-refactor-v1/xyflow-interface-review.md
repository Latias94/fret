# `fret-node` vs XYFlow — Interface and Architecture Review

Status: execution snapshot (last updated 2026-03-10)
Scope: `ecosystem/fret-node` public integration surfaces + internal interaction architecture

This note answers two questions:

1) How does `fret-node` compare to XYFlow’s mental model and API affordances?
2) What “fearless refactor” directions are worth landing next (and why)?

It is not a parity matrix. For the exhaustive checklist, use:

- `docs/node-graph-xyflow-parity.md`
- `docs/workstreams/fret-node-xyflow-parity.md`

## What we compare

XYFlow has two distinct targets:

- **A-layer**: `@xyflow/system` substrate (pan/zoom, drag, connect, resize, minimap math).
- **B-layer**: `@xyflow/react` runtime/store + component ecosystem (changes pipeline, instance
  helpers, callbacks).

`fret-node` splits the same concerns differently:

- **Headless document + ops** (`core`, `ops`, `rules`, `profile`): stable IDs, deterministic edits,
  typed connection planning, diagnostics.
- **Headless runtime ergonomics** (`runtime`): store, controlled mode, and an XYFlow-style change
  bridge (`runtime::{changes,apply}`).
- **UI integration** (`ui`): declarative surface + (compat) retained interaction engine.

## Where `fret-node` already aligns well (keep)

1) **Changes model bridge exists and is reversible**
   - XYFlow: `applyNodeChanges` / `applyEdgeChanges` utilities.
   - `fret-node`: `runtime::changes` maps `GraphTransaction` ⇄ `NodeChange`/`EdgeChange`, and
     `runtime::apply` applies change events in controlled mode while keeping undo/redo semantics.

2) **Instance/controller facade matches the “useReactFlow instance” intent**
   - XYFlow: imperative helpers (`setViewport`, `fitView`, query neighbors, etc.).
   - `fret-node`: `NodeGraphController` + `NodeGraphSurfaceBinding` own the same category of
     operations (viewport choreography, transaction-safe commits, query helpers).

3) **Mechanism vs policy separation is clearer than typical DOM-first stacks**
   - XYFlow encourages policy via components/CSS; `fret-node` can keep policy in `NodeGraphPresenter`
     + `NodeGraphSkin` + ecosystem overlays while keeping the headless substrate stable.

## Key deltas vs XYFlow (intentional)

1) **“Nodes as component subtrees” is not the center of gravity**
   - XYFlow’s default is DOM subtrees inside a pan/zoom container.
   - In Fret, the long-term home for the generic “world layer” mechanism is `ecosystem/fret-canvas`.
     `fret-node` should stay focused on editor-grade node-graph contracts and only use portals when
     they serve those contracts (inputs, overlays, diagnostics).

2) **Measured geometry is explicit, not “DOM layout happens”**
   - XYFlow exposes `updateNodeInternals` because measured handle bounds and node dimensions must be
     re-derived when subtree layout changes.
   - `fret-node` must keep the same outcome, but the mechanism should be explicit and deterministic
     (no hidden, frame-order-dependent invalidation).

3) **Multi-window/editor-grade expectations are first-class**
   - Overlays, docking, diagnostics, and non-trivial input arbitration are not “add-ons” in Fret.
     They are part of the target user experience, and the API design should reflect that.

## Fearless refactor opportunities (landable directions)

These are ordered by “risk reduced per line changed”.

1) **Keep one canonical app-facing integration story**
   - Recommended default: `NodeGraphSurfaceBinding` + `node_graph_surface(...)`.
   - Compatibility-only: retained widget/editor stack behind `compat-retained-canvas`.
   - Done criteria:
     - `ecosystem/fret-node/README.md` teaches declarative-first explicitly.
     - Demos and internal guides do not recommend raw retained authoring as the default.

2) **Push XYFlow A-layer mechanisms toward `fret-canvas`**
   - Goal: move world-layer mechanics (generic pan/zoom, generic hit-test utilities, generic
     spatial-index scaffolding) out of the node-editor domain.
   - Done criteria:
     - A clear boundary list in the workstream docs (“lives in `fret-canvas`” vs “lives in
       `fret-node`”).
     - One migrated mechanism with unchanged behavior and a conformance gate.

3) **Reduce interaction-state sprawl via typed “session” seams**
   - XYFlow keeps each interaction controller’s state localized (`XYDrag`, `XYHandle`, etc.).
   - `fret-node` can keep a single retained engine, but should keep each gesture’s state and cleanup
     logic in narrow modules with state-only tests (continued seam splits).
   - Done criteria:
     - New session seams include at least one state-only unit test.
     - Cancel/cleanup logic is centralized (no duplicated “clear 14 fields” blocks).

4) **Make “internals invalidation” a first-class contract (XYFlow `updateNodeInternals` outcome)**
   - The contract should describe *what* must be re-derived and *when* (handles, node rects, port
     anchors, spatial index), without leaking a specific retained/declarative implementation.
   - Done criteria:
     - One documented invalidation trigger surface (controller/store or measured-geometry store).
     - A conformance test that proves “pan-only does not rebuild geometry” while measured updates do.

5) **Unify B-layer middleware semantics across retained + declarative surfaces**
   - XYFlow commonly composes “onNodesChange/onEdgesChange middleware”.
   - `fret-node` already has a retained middleware chain; the declarative surface should keep a
     symmetric “change interception” seam so policy remains in ecosystem layers.
   - Done criteria:
     - One shared callback/middleware vocabulary for change events (transactions + derived changes).
     - One focused conformance test proving middleware order and cancellation behavior.

## Suggested gates while executing

- `cargo nextest run -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas --tests`
- `python3 tools/check_layering.py` (when moving code across crates)
- At least one `fretboard-dev diag` script for portal/overlay anchoring (when touching measured geometry)

## Evidence anchors (starting points)

- XYFlow:
  - `repo-ref/xyflow/packages/react/src/utils/changes.ts`
  - `repo-ref/xyflow/packages/system/src/*` (pan/zoom, drag, connect, resize)
- `fret-node`:
  - `ecosystem/fret-node/src/runtime/changes.rs`
  - `ecosystem/fret-node/src/runtime/apply.rs`
  - `ecosystem/fret-node/src/ui/binding.rs`
  - `ecosystem/fret-node/src/ui/controller.rs`
  - `docs/node-graph-xyflow-parity.md`
