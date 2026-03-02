# Workstream: `fret-node` Declarative-First Fearless Refactor (v1)

Status: In progress (last updated 2026-03-02)
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

## Progress snapshot

This is a living snapshot of what is already in place vs what remains.

- M0 (baseline + gates): **Present**
  - Minimal runnable demo: `cargo run -p fretboard -- dev native --bin node_graph_demo`
  - Diagnostics suite (paint-only): `fret-examples-node-graph-paint-only`
- M1 (declarative surface skeleton, paint-only): **Present**
  - Declarative paint-only surface (`FRET_NODE_GRAPH_DECLARATIVE=paint`) with hosted caches
  - Steady-state cache gates exist (grid/nodes/edges)
- M2 (interaction + portals, paint-only baselines): **Partially present**
  - Marquee/drag cancellation + portal bounds harvest + fit-view baselines are gated
  - Remaining: policy parity (selection/marquee reducers, overlays, richer portal hosting)
  - 2026-03-02: merged `main` and adapted paint-only to the `NodeGraphStyle { paint, geometry }`
    split (including the new `CanvasGeometry::build_with_presenter(..., overrides)` param)
- M3 (defaults + compatibility): **Present**
  - Retained is opt-in only: `fret-node/compat-retained-canvas`
  - Default features avoid `fret-ui/unstable-retained-bridge`

## How to run the paint-only gates

PowerShell (Windows-friendly):

```powershell
$env:FRET_DIAG='1'
cargo run -p fretboard -- diag suite fret-examples-node-graph-paint-only `
  --env FRET_NODE_GRAPH_DECLARATIVE=paint `
  --dir target/fret-diag-node-graph `
  --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos
```

Notes:

- `FRET_DIAG` is a reserved variable for `fretboard diag --launch` and should be set in the shell,
  not passed via `--env FRET_DIAG=...`.
- For GPU screenshots, pass `--env FRET_DIAG_GPU_SCREENSHOTS=1` and use a screenshot script (see
  `tools/diag-scripts/node-graph/node-graph-paint-only-wires-screenshot.json`).

## Recommendation (default declarative, compat retained)

We explicitly choose a “default declarative” posture:

- Default UI surfaces are declarative and do not require retained authoring.
- Retained widgets may remain as an **internal** implementation for narrow hotspots behind an
  explicit compatibility feature (opt-in), with exit criteria and a delete plan.

## Feature flags (public integration posture)

- `fret-node/fret-ui`: Enables the declarative-first UI integration surface (elements + canvas paint-only).
- `fret-node/compat-retained-canvas`: Opt-in compatibility surface for the legacy retained
  widget/editor stack. This enables `fret-ui/unstable-retained-bridge` transitively and is
  intentionally **not** part of `fret-node`'s default feature set.

## Ecosystem authoring guide (recommended)

This workstream aims to make downstream ecosystem authors productive without touching retained APIs.

### Recommended (declarative-first)

- Depend on `fret-node` with UI enabled, without the retained bridge:
  - `fret-node = { version = "0.1.0", features = ["fret-ui"] }`
- Compose the node graph surface as a normal declarative element:
  - paint-only milestone surface: `fret_node::ui::node_graph_surface_paint_only`
- Keep editor state in models:
  - graph: `Model<Graph>`
  - view state: `Model<NodeGraphViewState>`
  - derived caches: `Model<...>` keyed by (rev, viewport/cull, effective scale)

### Compatibility (retained escape hatch)

Enable this only when you have a concrete missing declarative mechanism and an exit plan:

- `fret-node = { version = "0.1.0", features = ["compat-retained-canvas"] }`
- Use retained-backed entrypoints internally:
  - declarative root hosting retained: `fret_node::ui::node_graph_surface_compat_retained`
  - `imui` subtree adapter: `fret_node::imui::*`

### API red lines

- Do not expose `UiTree`, `Widget`, or `fret_ui::retained_bridge::*` in downstream public APIs.
- Prefer caches + invalidation discipline over per-frame recomputation.

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

As a first migration step, this workstream adds a **declarative entrypoint** that still hosts the
current retained node-graph canvas as an internal subtree:

- compat declarative surface: `ecosystem/fret-node/src/ui/declarative/mod.rs`
- demo A/B switch: `apps/fret-examples/src/node_graph_demo.rs` (`FRET_NODE_GRAPH_DECLARATIVE=1`)

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
  - Note: `LayoutQueryRegion` bounds do not include absolutely positioned descendants; the query
    region itself should be the positioned box when harvesting overlay item bounds.

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

## M0 quickstart (baseline repro artifacts)

Run the existing node graph demo (Windows-friendly):

- `cargo run -p fretboard -- dev native --bin node_graph_demo`

Capture a baseline diagnostics bundle using the scripted repro:

- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-pan-middle-escape-cancel.json --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`

Notes:

- The diag script asserts `panning true` then `panning false` by inspecting the viewport semantics
  `value` string for `test_id=node_graph.canvas`.
- To opt into the declarative root for manual exploration:
  - Compat retained surface (current default for the declarative root):
    - PowerShell: `$env:FRET_NODE_GRAPH_DECLARATIVE=1`
  - Paint-only declarative skeleton (paint-first + semantic-zoom portals, no portal hit-testing yet):
    - PowerShell: `$env:FRET_NODE_GRAPH_DECLARATIVE=paint`

## M1 steady-state cache gate (paint-only baseline)

Run the paint-only steady-state cache gate and capture a diagnostics bundle:

Notes:

- Set `FRET_DIAG=1` in your shell (do not pass it via `--env`; it is reserved for tool-launched runs).
  - PowerShell: `$env:FRET_DIAG='1'`

- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-steady-grid-cache.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-steady-nodes-cache.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-steady-edges-cache.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-pan-does-not-rebuild-geometry.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-keyboard-zoom-rebuilds-geometry.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-diag-graph-bump-rebuilds-geometry.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-hover-and-select-do-not-rebuild-geometry.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-marquee-select-does-not-rebuild-geometry.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-marquee-pointer-cancel-does-not-commit.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-node-drag-preview-and-commit.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-node-drag-escape-cancel-does-not-commit.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-node-drag-pointer-cancel-does-not-commit.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-fit-view-to-portals-updates-view.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-hover-shows-portal-tooltip.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-hover-tooltip-falls-back-to-hover-anchor.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`
- `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-paint-only-pan-pointer-cancel-does-not-rebuild-geometry.json --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`

Run the promoted paint-only suite (IDs live in `tools/diag-scripts/index.json`):

- `cargo run -p fretboard -- diag suite fret-examples-node-graph-paint-only --env FRET_NODE_GRAPH_DECLARATIVE=paint --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`

Maintenance note:

- The suite members are defined via redirect stubs under
  `tools/diag-scripts/suites/fret-examples-node-graph-paint-only/`.
- `tools/diag-scripts/index.json` is generated; after changing suite membership, run:
  - `python tools/check_diag_scripts_registry.py --write`

## Diagnostics shortcuts (paint-only)

When running with `FRET_DIAG=1`, the paint-only surface provides a few deterministic shortcuts used
by scripted gates:

- `Ctrl+3`: bump graph revision (forces geometry cache rebuild exactly once)
- `Ctrl+4`: normalize graph + view for node-drag gates (single visible node centered)
- `Ctrl+5`: normalize graph + view for marquee gates (single visible node offset from center)
- `Ctrl+7`: enable portal hosting
- `Ctrl+8`: disable portal hosting (clears `PortalBoundsStore` so tooltip fallback paths can run)
- `Ctrl+9`: fit view to current portal bounds (consumes `PortalBoundsStore`)
