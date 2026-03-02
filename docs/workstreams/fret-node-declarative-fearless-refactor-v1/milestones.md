# `fret-node` Declarative-First Fearless Refactor (v1) — Milestones

This file tracks “what done means” for each milestone. Keep criteria objective and checkable.

## Baseline scenarios and success metrics (applies to all milestones)

We evaluate progress against a small set of fixed, repeatable scenarios. Each milestone must
either:

- improve one metric without regressing the others, or
- document a deliberate tradeoff.

### Scenarios (stable, scriptable)

S1. Pan/zoom stress:

- Hold mouse drag to pan continuously for N seconds, then zoom in/out with wheel/pinch.
- Expectation: no pointer-capture leaks; no “stuck drag”; smooth-by-default caches avoid thrash.

S2. Selection + marquee:

- Click to select, shift-select, drag marquee, move selection, cancel mid-drag (PointerCancel).
- Expectation: state machine determinism; cancellation clears pressed/drag state.

S3. Large graph viewport culling:

- Load a large graph, keep viewport window fixed, and idle for N frames.
- Expectation: expensive render-data caches do not rebuild on steady-state frames.

### Metrics (observable)

- Correctness:
  - Pointer capture lifetime is correct (capture set/cleared; cancel delivered on capture switch).
  - No background focus/keyboard leakage when overlays are active (when applicable).
- Perf / cache health:
  - “Prepare” work (text/path/svg) does not thrash under pan/zoom and culling.
  - Cache entry counts remain bounded under a fixed scenario.
- UX smoothness:
  - Continuous pan/zoom does not trigger “jank spikes” from avoidable recomputation.

Implementation note:

- Prefer a `fretboard diag` scripted scenario for S1/S2, and a small perf/counter assertion for S3.

## M0 — Baseline + gates

Deliverables:

- A runnable minimal demo/harness for the node graph surface (native + web if applicable).
- A small regression gate for:
  - pointer capture / drag cancel correctness, and/or
  - pan/zoom mapping invariants, and/or
  - cache thrash counters (prepares/evictions) staying within a bound for a scripted scenario.

Done criteria:

- The current behavior has at least one reproducible, automated gate (test or diag script).
- The gate is stable enough to run in CI (no human-timing dependencies).

Evidence anchors (required):

- The chosen gate file(s) and the entrypoint that runs the scenario.
- The primary state machine code paths under test.

Suggested baseline artifacts (current v1 worktree):

- Minimal runnable entrypoint (native):
  - `cargo run -p fretboard -- dev native --bin node_graph_demo`
  - (feature wiring: `apps/fretboard/src/dev.rs` auto-enables `fret-demo/node-graph-demos` for `node_graph_demo`)
- Gate (Rust conformance):
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/escape_cancel_releases_pointer_capture_conformance.rs`
- Repro + bundle capture (diag script):
  - `tools/diag-scripts/node-graph/node-graph-pan-middle-escape-cancel.json`

Primary evidence anchors (why this is meaningful):

- Panning starts + capture is requested:
  - `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs` (`begin_panning`)
- Escape cancel clears gesture state and releases capture:
  - `ecosystem/fret-node/src/ui/canvas/widget/cancel.rs` (`handle_escape_cancel`)
- The scripted repro asserts panning state via viewport semantics `value`:
  - `ecosystem/fret-node/src/ui/canvas/widget/retained_widget.rs` (semantics `test_id=node_graph.canvas`, `value` includes `panning {bool}`)

## M1 — Declarative surface skeleton (paint-first)

Goal: build the declarative composition shell that can render and pan/zoom smoothly.

### M1a — Declarative entrypoint (compat retained)

Goal: provide a declarative-first authoring surface immediately, while keeping the current retained
canvas internal as a migration aid.

Deliverables:

- A declarative surface entrypoint in `fret-node` that:
  - returns `AnyElement`,
  - does not expose retained types in its public signature,
  - hosts the existing retained canvas/editor internally.
- A demo A/B switch so we can iterate without destabilizing the default retained demo path.

Done criteria:

- `apps/fret-examples/src/node_graph_demo.rs` can run in:
  - default retained mode (no env var),
  - declarative root mode (compat retained) (`FRET_NODE_GRAPH_DECLARATIVE=1`),
  - declarative root mode (paint-only skeleton) (`FRET_NODE_GRAPH_DECLARATIVE=paint`),
  with the same baseline interaction gates (Escape cancel / pointer capture) remaining valid.

Evidence anchors (required):

- Declarative compat surface entry:
  - `ecosystem/fret-node/src/ui/declarative/mod.rs` (`node_graph_surface_compat_retained`)
- Demo wiring:
  - `apps/fret-examples/src/node_graph_demo.rs` (`NodeGraphDemoDriver::new_from_env`, `render_root`)

Deliverables:

- Declarative surface entrypoint that composes:
  - `PointerRegion` input wiring,
  - `RenderTransform` (world mapping),
  - leaf `Canvas` paint pass for grid/background/edges.
- Externalized cache model(s) for expensive render data keyed by:
  - graph revision,
  - viewport/cull window,
  - effective scale factor.

Done criteria:

- Node graph can:
  - pan/zoom smoothly,
  - cull offscreen work,
  - avoid per-frame rebuild of heavy render data when inputs are unchanged.

Evidence anchors (required):

- The declarative surface entry function and its props type.
- The cache model(s) and their invalidation keys (rev/viewport/scale).
- A gate that shows “steady-state frames do not rebuild render data”.
  - Paint-only baseline gate (grid cache steady-state):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-steady-grid-cache.json`
  - Paint-only baseline gate (nodes cache steady-state):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-steady-nodes-cache.json`
  - Paint-only baseline gate (edges cache steady-state):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-steady-edges-cache.json`
  - Paint-only baseline gate (pan does not rebuild geometry):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-pan-does-not-rebuild-geometry.json`
  - Paint-only gate (keyboard zoom rebuilds geometry exactly once):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-keyboard-zoom-rebuilds-geometry.json`
  - Paint-only gate (diagnostics graph bump rebuilds geometry exactly once):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-diag-graph-bump-rebuilds-geometry.json`
  - Paint-only gate (hover + selection do not rebuild geometry caches):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-hover-and-select-do-not-rebuild-geometry.json`

## M2 — Interaction + portals in declarative form

Goal: move interaction policy and portal UI into declarative mechanisms without retained authoring.

Deliverables:

- Selection/marquee/drag workflows implemented via declarative input wiring + model reducers.
- “Portal” nodes (header/body) hosted as normal elements for the focused/visible subset.
- Portal subtree bounds harvested via `LayoutQueryRegion` into a local store (frame-lagged by contract),
  enabling future fit-view/selection unions without retained hit-testing.
- Overlay surfaces (context menus, rename, toolbars) implemented in ecosystem overlay policy.
- Paint-only baseline interaction gate(s) that prove selection/marquee state changes do not rebuild
  geometry caches.

Done criteria:

- Feature parity for the core editor interactions needed by downstream apps.
- No retained bridge required for the default UI path.

Evidence anchors (required):

- The declarative interaction reducers (selection/marquee/drag) and their gates.
- The portal composition code path for visible items only.

Initial paint-only interaction gates (v1 worktree):

- Marquee selection updates selection without rebuilding geometry render data:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-marquee-select-does-not-rebuild-geometry.json`
- PointerCancel clears marquee without committing selection:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-marquee-pointer-cancel-does-not-commit.json`
- Node drag preview does not rebuild geometry caches; commit rebuilds once:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-node-drag-preview-and-commit.json`
- Escape cancels node dragging without committing:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-node-drag-escape-cancel-does-not-commit.json`
- PointerCancel cancels node dragging without committing:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-node-drag-pointer-cancel-does-not-commit.json`
- PointerCancel clears panning without rebuilding geometry:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-pan-pointer-cancel-does-not-rebuild-geometry.json`

## M3 — Remove retained dependency from defaults (compat path only)

Goal: ensure the retained bridge path is no longer the default or required surface.

Deliverables:

- `fret-node` default features do not enable `fret-ui/unstable-retained-bridge`.
- Any retained implementation remains available only as an explicit opt-in compatibility feature.
- Documentation explains when (if ever) to opt into the compatibility path.

Done criteria:

- A downstream “ecosystem author” can adopt `fret-node` UI surfaces without touching retained APIs.
- Retained bridge usage is isolated, measurable, and delete-planned.

Compatibility hatch acceptance criteria:

- Retained is allowed only when:
  - a specific missing mechanism is documented (what declarative cannot express efficiently today),
  - the retained module is behind an explicit `compat-*` feature,
  - public API does not expose retained types,
  - at least one gate proves the hotspot and protects behavior/perf.
- Exit criteria must be stated up front:
  - what contract/caching change removes the need for retained,
  - what tests/diag scripts must remain green after removal.
