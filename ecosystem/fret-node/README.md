# `fret-node`

Node-graph foundation and integration surfaces for Fret editor workflows.

This crate provides headless graph model/schema building blocks plus optional UI integration
surfaces for node-graph editors.

## Status

Experimental learning project (not production-ready).

## Recommended usage (declarative-first)

New app code should prefer the declarative surface:

- `NodeGraphSurfaceBinding` (canonical app-facing bundle: graph + view state mirrors + controller)
- `node_graph_surface(...)` (declarative UI entry point)

Minimal composition pattern:

```rust
use fret_node::io::{NodeGraphEditorConfig, NodeGraphViewState};
use fret_node::Graph;
use fret_node::ui::{NodeGraphSurfaceBinding, node_graph_surface};

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

See `apps/fret-examples/src/node_graph_demo.rs` for a runnable example.

## Features

- `ui` / `fret-ui`: enable `crates/fret-ui` integration helpers (canvas widget, styling surfaces)
- `canvas-rstar`: opt into an R-tree spatial index backend for large graphs
- `app-integration`: optional `fret-app` helpers (commands/default bindings)
- `headless`: build headless-only graph model surfaces
- `compat-retained-canvas`: declarative compatibility surface that hosts the retained widget/editor
  stack internally (compatibility-only; delete-planned)

## Upstream references (non-normative)

Node-graph editors have a lot of established interaction vocabulary. These projects are useful for
design intent and parity targets:

- XyFlow (React Flow): https://github.com/xyflow/xyflow
- egui-snarl (Rust node graph editor): https://github.com/zakarumych/egui-snarl
