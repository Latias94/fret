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

## M1 — Declarative surface skeleton (paint-first)

Goal: build the declarative composition shell that can render and pan/zoom smoothly.

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

## M2 — Interaction + portals in declarative form

Goal: move interaction policy and portal UI into declarative mechanisms without retained authoring.

Deliverables:

- Selection/marquee/drag workflows implemented via declarative input wiring + model reducers.
- “Portal” nodes (header/body) hosted as normal elements for the focused/visible subset.
- Overlay surfaces (context menus, rename, toolbars) implemented in ecosystem overlay policy.

Done criteria:

- Feature parity for the core editor interactions needed by downstream apps.
- No retained bridge required for the default UI path.

Evidence anchors (required):

- The declarative interaction reducers (selection/marquee/drag) and their gates.
- The portal composition code path for visible items only.

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

