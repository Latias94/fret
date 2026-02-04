# How to build a node editor like XyFlow (with `fret-node`)

This guide is an **API-level map** for building an “editor-grade” node graph UI with `fret-node`,
following the same mental model many teams have from XyFlow / React Flow.

For the parity matrix and milestones, see:

- `docs/node-graph-xyflow-parity.md`
- `docs/workstreams/fret-node-xyflow-parity.md`

## Mental model: three layers

`fret-node` deliberately separates:

1) **Headless document + ops** (`core` / `ops`): serializable graph + deterministic transactions.
2) **Headless runtime ergonomics** (`runtime`): store, lookups, XyFlow-style apply/change helpers.
3) **UI integration** (`ui`): canvas widget, overlays, panels, presenter, retained bridge wiring.

The node editor UI is intended to be “editor-grade” (multi-window, docking, overlays), so UI add-ons
are hosted in **window space** (outside the pan/zoom render transform).

## Recommended (store-driven) integration

This is the closest match to “useReactFlow + built-ins”:

- Authoritative state lives in `runtime::store::NodeGraphStore`.
- The UI consumes:
  - `Model<Graph>` (for painting and hit-testing),
  - `Model<NodeGraphViewState>` (pan/zoom/selection),
  - optional `Model<NodeGraphStore>` (to keep view updates in sync for B-layer apps).

High-level composition pattern:

- `NodeGraphEditor` as a layering container
  - canvas node
  - portal host (optional)
  - overlays (toolbars, minimap, controls)
  - overlay host (context menus, rename dialogs, etc.)

Concrete example:

- `apps/fret-examples/src/node_graph_demo.rs`

Run it (desktop only):

```bash
cargo run -p fret-demo --features node-graph-demos --bin node_graph_demo
```

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

## Extension points

### Presenter (UI policy + derived labels)

Use a `NodeGraphPresenter` to control:

- node titles and port labels,
- context menu content and actions,
- edge label/marker hints and route hints,
- measured geometry integration (when using portal editors).

### `nodeTypes` / portal node renderers

Use `NodeGraphNodeTypes` + `NodeGraphPortalHost` to render per-node UIs (text inputs, buttons,
custom controls) while keeping the canvas itself policy-light.

### `edgeTypes` / custom edge paths

Use `NodeGraphEdgeTypes` to register custom edge path builders and keep hit-testing deterministic.

## Built-in add-ons (UI overlays)

- Panels composition: `ui::NodeGraphPanel` (XyFlow `<Panel />` equivalent)
- Controls overlay: `ui::NodeGraphControlsOverlay`
- MiniMap overlay: `ui::NodeGraphMiniMapOverlay`
- Toolbars: `ui::NodeGraphNodeToolbar`, `ui::NodeGraphEdgeToolbar`

Stable contract:

- `docs/node-graph-addons-minimap-controls.md`

