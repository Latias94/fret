# Workstream: `fret-node` Declarative-First Fearless Refactor (v1)

Status: Draft
Scope: `ecosystem/fret-node` (+ `ecosystem/fret-canvas` composition patterns)

## Intent

Make `ecosystem/fret-node` a **declarative-first** reference implementation for “editor-grade
canvas UI” in the ecosystem:

- keep the headless graph model portable and reusable,
- provide a declarative UI surface that composes with `crates/fret-ui` contracts,
- reach retained-grade performance via cache discipline and explicit invalidation,
- avoid leaking retained authoring (`UiTree`/`Widget`) into the long-term public API.

This workstream is explicitly *not* about “removing every retained line immediately”. It is about
making the **default** and **recommended** authoring surface declarative, while leaving a narrowly
scoped compatibility escape hatch when unavoidable (feature-gated and delete-planned).

## Recommendation (default declarative, compat retained)

We explicitly choose a “default declarative” posture:

- Default UI surfaces are declarative and do not require retained authoring.
- Retained widgets may remain as an **internal** implementation for narrow hotspots behind an
  explicit compatibility feature (opt-in), with exit criteria and a delete plan.

## Context (current state)

Today, `fret-node` is structurally “headless model + optional UI”. The UI/editor side uses a
retained canvas widget hosted inside the declarative runtime via the retained bridge:

- retained subtree hosting adapter: `ecosystem/fret-node/src/imui.rs`
- retained widget implementation: `ecosystem/fret-node/src/ui/canvas/widget/retained_widget.rs`
- retained bridge contract (feature-gated): `crates/fret-ui/src/retained_bridge.rs`

In parallel, the ecosystem already has declarative canvas composition patterns in `fret-canvas/ui`
that are closer to the intended long-term direction:

- declarative input wiring around a leaf `Canvas`: `ecosystem/fret-canvas/src/ui/canvas_surface.rs`
- world-layer composition with `RenderTransform` + cross-frame bounds: `ecosystem/fret-canvas/src/ui/world_layer.rs`

## Public API sketch (ecosystem authoring surface)

Goal: downstream ecosystem authors should be able to adopt node graph UI without touching retained
types (`UiTree`, `Widget`, `retained_bridge::*`) and without enabling `fret-ui/unstable-retained-bridge`.

Proposed public surfaces (illustrative, not locked yet):

- `fret_node::ui`:
  - `node_graph_surface(cx, NodeGraphSurfaceProps { graph, view_state, presenter, skin, ... }) -> AnyElement`
  - `NodeGraphSurfaceProps`:
    - headless model handles: `Model<Graph>`, `Model<ViewState>` (or equivalent)
    - view inputs: `PanZoom2D` (model), `cull_margin`, `effective_scale_factor`
    - domain hooks: callbacks for commands, transactions, drag payloads
    - policy knobs: keybindings/toggles live in ecosystem/app
  - `NodeGraphPresenter` / `NodeGraphSkin` remain the extension points for look and domain mapping.

Non-goals for public API:

- Do not expose `UiTree`, `Widget`, `NodeId` (runtime tree node ids), or retained bridge types.
- Do not require downstream crates to register resources manually; prefer hosted caches.

## Goals

1. **Public API becomes declarative-first**
   - Default `fret-node` UI surfaces should be usable without `fret-ui/unstable-retained-bridge`.
   - Public exports must not require downstream crates to implement `Widget` or call `UiTree`
     directly.
2. **Performance parity is achieved by design**
   - Avoid “recompute everything every frame” by externalizing caches and keying them by stable
     identity + revision + scale/viewport.
   - Prefer `CanvasPainter` hosted caches and/or shared ecosystem caches (ADR 0161 direction).
3. **Layering stays clean**
   - No new policy gets pushed into `crates/fret-ui`.
   - Gesture maps, tool modes, snapping, and domain transactions remain ecosystem/app-owned.
4. **Regression artifacts exist**
   - Add at least one focused gate per milestone (unit/integration and/or `fretboard diag` script)
     for interaction state machines and canvas perf risks.

## Non-goals

- Redesigning the node graph file formats, schema/migrations, or deterministic persistence rules.
- Introducing new “widget authoring primitives” as a stable extension surface.
- Solving every advanced editor feature (Blueprint-grade styling, full accessibility closure) in v1.

## Proposed target architecture (declarative-first)

### A) A single declarative “surface” component

Introduce a declarative surface entrypoint that owns composition, not policy:

- input wiring: `PointerRegion` hooks → update `Model` state and request redraws,
- view transforms: `RenderTransform` derived from `Model<PanZoom2D>`,
- world paint: leaf `Canvas` paints background + edges + bulk node chrome,
- world items: optional element subtrees for portals/interactive regions (bounded in count),
- overlays: explicit overlay roots / anchored placement at the ecosystem layer.

### B) Externalized caches and revisioned updates

To match retained-grade performance without retained authoring:

- keep the headless graph model as the source of truth,
- derive “render data” into `Model<…>` caches keyed by:
  - graph revision (or per-subsystem revision),
  - viewport / cull window,
  - effective scale factor (`dpi * zoom`),
- update caches only when relevant inputs change, not on every frame.

Cache best practices (checkable rules):

- **Stable identity**: cache keys are stable ids (node/edge ids + style keys), not transient pointers.
- **Revisioned invalidation**: caches observe a small set of “inputs that matter” (rev, viewport, scale).
- **Bounded retention**: caches are bounded by entry count and/or frame retention windows (ADR 0161).
- **Key hygiene**: avoid high-entropy keys (timestamps, random UUIDs) in paint loops.

### C) Explicit resource caching policy

Align caching semantics across declarative and retained implementations:

- declarative `CanvasPainter` hosted caches (ADR 0141),
- shared cache policy vocabulary for smooth-by-default bounded retention (ADR 0161),
- ecosystem-level retained caches remain available for internal compatibility paths.

### D) Compatibility: retained stays internal and feature-gated

If specific hotspots cannot be made efficient declaratively in v1:

- keep the retained canvas as an **internal implementation strategy** behind an explicit feature
  (compatibility), and
- ensure the default path and public API remain declarative-first.

The compatibility surface must be:

- opt-in,
- tightly scoped,
- delete-planned (tracked in milestones and TODOs).

Compatibility hatch criteria (when retained is allowed):

- A concrete missing contract is identified (what declarative cannot express efficiently today).
- A minimal compatibility module is introduced that does not leak retained types into public APIs.
- A gate exists that demonstrates the hotspot and protects behavior/perf.
- Exit criteria are written up front (what needs to change to remove retained).

## Contract dependencies (what we rely on)

This refactor assumes the following runtime contracts remain stable and sufficient:

- declarative authoring: `ElementContext` / `AnyElement` (`crates/fret-ui/src/lib.rs`)
- leaf custom draw: `Canvas` + `CanvasPainter` (ADR 0141 direction)
- input wiring: `PointerRegion` action hooks (`crates/fret-ui/src/action`)
- correct world mapping: `render_transform` semantics (ADR 0082)
- cross-frame geometry: `last_*bounds_for_element` / `LayoutQueryRegion` patterns

## Red lines (when to propose new runtime contracts)

If a milestone uncovers a declarative performance/ergonomics gap, do not “patch over” by exposing
retained widgets as a stable authoring surface.

Instead:

1) Attempt an ecosystem-layer solution (caches, budgets, keying, composition patterns).
2) If still blocked, write down the missing mechanism as a runtime contract proposal:
   - what the mechanism is,
   - which invariants it must satisfy,
   - which authoritative upstream reference motivates it,
   - why it cannot live in ecosystem policy.

Any hard-to-change runtime mechanism requires ADR alignment before being treated as stable.

## References

- Canvas surfaces and layering direction: `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- Canvas portals (staged): `ecosystem/fret-node/src/ui/portal.rs`
- Declarative canvas element + hosted painter: `docs/adr/0141-declarative-canvas-element-and-painter.md`
- Cache policy vocabulary for smooth-by-default: `docs/adr/0161-canvas-cache-policy-and-hosted-resource-caches.md`
