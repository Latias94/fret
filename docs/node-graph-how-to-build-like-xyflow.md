# How to build a node editor like XyFlow (with `fret-node`)

This guide is an **API-level map** for building an “editor-grade” node graph UI with `fret-node`,
following the same mental model many teams have from XyFlow / React Flow.

For the parity matrix and milestones, see:

- `docs/node-graph-xyflow-parity.md`
- `docs/workstreams/standalone/fret-node-xyflow-parity.md`

## Mental model: three layers

`fret-node` deliberately separates:

1) **Headless document + ops** (`core` / `ops`): serializable graph + deterministic transactions.
2) **Headless runtime ergonomics** (`runtime`): store, lookups, XyFlow-style apply/change helpers.
3) **UI integration** (`ui`): `NodeGraphSurfaceBinding`, `node_graph_surface(...)`,
   `NodeGraphController`, overlays, panels, presenters, and portal renderers.

The node editor UI is intended to be “editor-grade” (multi-window, docking, overlays), so UI add-ons
are hosted in **window space** (outside the pan/zoom render transform).

## Recommended (binding-first) integration

This is the closest match to “useReactFlow + built-ins”:

- Authoritative state lives in `runtime::store::NodeGraphStore`.
- App code usually starts with `NodeGraphSurfaceBinding::new(...)`, which creates the graph,
  view-state, editor-config, one authoritative store, and store-derived projection models as one
  app-facing bundle.
- The declarative surface renders through `node_graph_surface(cx, surface.surface_props())`.
- App actions should prefer the binding helpers (`dispatch_transaction*`, `set_viewport*`,
  `fit_view_nodes_in_bounds*`, `update_node*`, `update_edge*`, `undo*`, `redo*`) before dropping to
  an explicit controller.
- Tool-mode or shortcut interception belongs on `NodeGraphDeclarativeInteractionHook` through
  `NodeGraphSurfaceProps::interaction_hook`; hooks receive store snapshots and binding/controller
  commit helpers, not mutable graph ownership.
- When lower-level imperative ownership is useful, derive it explicitly with
  `NodeGraphController::new(surface.store_model())`.

Minimal composition pattern:

```rust
use fret_node::io::{NodeGraphEditorConfig, NodeGraphViewState};
use fret_node::ui::{NodeGraphSurfaceBinding, node_graph_surface};
use fret_node::Graph;

fn init(app: &mut fret::App, graph: Graph) -> NodeGraphSurfaceBinding {
    NodeGraphSurfaceBinding::new(
        app.models_mut(),
        graph,
        NodeGraphViewState::default(),
        NodeGraphEditorConfig::default(),
    )
}

fn view(cx: &mut fret_ui::ElementContext<'_, fret::App>, surface: &NodeGraphSurfaceBinding) {
    surface.observe(cx);
    node_graph_surface(cx, surface.surface_props());
}
```

Use this surface as the root of the editor composition, then add optional node portals, panels,
toolbars, minimap, controls, diagnostics, or domain actions through the binding/controller path.

Concrete example:

- `apps/fret-examples/src/node_graph_demo.rs`

Run it (desktop only):

```bash
cargo run -p fret-demo --features node-graph-demos --bin node_graph_demo
```

The demo includes a non-interactive help overlay (toggle: Cmd/Ctrl+H) and showcase toggles for
panel-vs-floating placement of MiniMap/Controls.

## Advanced (controlled mode) integration

Use this when an app already owns graph state (e.g. collaborative CRDTs, external persistence, or a
non-Rust host) and wants to apply deltas:

- generate a transaction (or a change set),
- apply it to your authoritative graph,
- feed the updated `Graph` back into the UI.

Entry points:

- `docs/node-graph-controlled-mode.md`
- `ecosystem/fret-node/examples/controlled_mode.rs`
- `runtime::apply` / `runtime::changes`

Use full document replacement (`NodeGraphSurfaceBinding::replace_document_action_host(...)`) for
reset-style synchronization. Diff-first controlled replacement remains a follow-on decision: keep it
transaction-explicit until a real workload proves the helper belongs in the public surface.

## Extension points

### Default declarative view policy

The default `node_graph_surface(...)` path deliberately exposes a narrower view-policy surface than
the full presenter trait:

- `NodeGraphNodeTypes` owns portal-rendered per-node UI.
- `NodeGraphSurfaceProps.edge_types` owns ReactFlow-style edge hint and custom paint-path policy.
- `NodeGraphSurfaceProps.skin` owns paint-only node/edge/port chrome; the default surface currently
  applies it to edge render hints.

Custom `NodeGraphPresenter` is not part of the default declarative surface. It remains an advanced
internal baseline until geometry, labels, context menus, and insertion/search policy are split into
separate default-path contracts.

### Presenter (advanced UI policy + derived labels)

Use a `NodeGraphPresenter` to control:

- node titles and port labels,
- context menu content and actions,
- edge label/marker hints and route hints,
- measured geometry integration (when using portal editors).

### `nodeTypes` / portal node renderers

Use `NodeGraphNodeTypes` with the declarative portal path to render per-node UIs (text inputs,
buttons, custom controls) while keeping the canvas itself policy-light.

### `edgeTypes` / custom edge paths

Use `NodeGraphEdgeTypes` through `NodeGraphSurfaceProps.edge_types` to register edge hint overrides
and custom paint-path builders. The default declarative surface uses custom paths for painting,
conservative paint culling, conservative spatial-index candidate rects, and exact path-distance hit
filtering for edge interaction candidates. It also uses the custom path midpoint for edge-center
anchors exposed through internals, and the declarative EdgeToolbar host consumes those anchors for
child placement. `EdgeRenderHint.label` now renders through the same screen-space child layer at
the edge-center anchor. For arbitrary non-interactive edge label children, use
`node_graph_surface_with_edge_label_renderer(...)` and `NodeGraphDeclarativeEdgeLabelRenderer`; the
renderer receives `NodeGraphEdgeLabelLayout` with the same screen-space anchor. Use
`node_graph_surface_with_renderers(...)` when combining custom node portal and edge-label
renderers. Edge-label renderers are hit-test transparent by default. For the first narrow
pointer-interactive control contract, return `NodeGraphEdgeLabelHitTestMode::ChildBounds` from
`edge_label_hit_test_mode(...)`; the default surface then routes pointer hit-testing only inside the
custom child bounds, while points outside those bounds fall through to the canvas surface.
Declarative EdgeToolbar remains the higher-level composition path for edge action clusters.

### Styling (theme tokens + UI-only chrome hints)

XyFlow uses a mix of global CSS variables and per-entity `node.style` / `edge.style` overrides.

In `fret-node`, styling is intentionally split:

- **Base tokens**: `NodeGraphStyle` (typed tokens derived from the app theme).
- **Paint-only per-entity chrome**: `NodeGraphSkin` (node/edge/port chrome hints).
- **Geometry-affecting overrides** (M2): a planned, UI-only, type-safe surface for per-node/per-edge
  layout knobs (kept out of serialized `Graph`).

Contract / guidance:

- `docs/node-graph-addons-theming.md`
- `docs/workstreams/fret-node-style-skinning-v2/README.md`

## Built-in add-ons (UI overlays)

- Panels, controls, minimap, and toolbar composition are crate-internal declarative overlay
  primitives today; public recipes should stay binding/controller-first instead of depending on
  retained widget names.

Stable contract:

- `docs/node-graph-addons-minimap-controls.md`

Keep add-ons bound to the controller/store surface where possible. Guides and demos should not teach
raw queue ownership or implementation-local surface state as the normal downstream API.

## Blackboard variables (symbols) and symbol references

`fret-node` treats graph-scoped symbols as first-class (`Graph.symbols`), and standardizes a
built-in symbol reference node kind:

- `fret.symbol_ref`

Contract (baseline):

- A symbol reference node's `Node.data` must be an object with a `symbol_id` string (UUID).
- The referenced `symbol_id` must exist in `Graph.symbols`.

Code pointers:

- Contract helpers: `core::symbol_ref` (`SYMBOL_REF_NODE_KIND`)
- Structural validation: `core::validate_graph_structural`
- Copy/paste fragment captures referenced symbols: `ops::GraphFragment`
