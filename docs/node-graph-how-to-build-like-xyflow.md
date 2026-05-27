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
   `NodeGraphController`, overlays, panels, presenters, and optional retained compatibility wiring.

The node editor UI is intended to be “editor-grade” (multi-window, docking, overlays), so UI add-ons
are hosted in **window space** (outside the pan/zoom render transform).

## Recommended (binding-first) integration

This is the closest match to “useReactFlow + built-ins”:

- Authoritative state lives in `runtime::store::NodeGraphStore`.
- App code usually starts with `NodeGraphSurfaceBinding::new(...)`, which creates the graph,
  view-state, editor-config, and store/controller mirrors as one app-facing bundle.
- The declarative surface renders through `node_graph_surface(cx, surface.surface_props())`.
- App actions should prefer the binding helpers (`dispatch_transaction*`, `set_viewport*`,
  `fit_view_nodes_in_bounds*`, `update_node*`, `update_edge*`, `undo*`, `redo*`) before dropping to
  an explicit controller.
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
Direct retained canvas authoring is compatibility/internal territory; new app code should keep the
declarative root surface as the taught default.

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

### Presenter (UI policy + derived labels)

Use a `NodeGraphPresenter` to control:

- node titles and port labels,
- context menu content and actions,
- edge label/marker hints and route hints,
- measured geometry integration (when using portal editors).

### `nodeTypes` / portal node renderers

Use `NodeGraphNodeTypes` with the declarative portal path to render per-node UIs (text inputs,
buttons, custom controls) while keeping the canvas itself policy-light.

### `edgeTypes` / custom edge paths

Use `NodeGraphEdgeTypes` to register custom edge path builders and keep hit-testing deterministic.

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

- Panels composition: `ui::NodeGraphPanel` (XyFlow `<Panel />` equivalent)
- Controls overlay: `ui::NodeGraphControlsOverlay`
- MiniMap overlay: `ui::NodeGraphMiniMapOverlay`
- Toolbars: `ui::NodeGraphNodeToolbar`, `ui::NodeGraphEdgeToolbar`

Stable contract:

- `docs/node-graph-addons-minimap-controls.md`

Keep add-ons bound to the controller/store surface where possible. Compatibility retained plumbing
may still host parts of the implementation, but guides and demos should not teach raw queue or
retained widget ownership as the normal downstream API.

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
