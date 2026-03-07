# Node Graph (fret-node) — Controlled Mode Guide

This guide documents the **controlled-mode** integration pattern for `ecosystem/fret-node` that
mirrors the React Flow / XyFlow mental model:

- the application owns the authoritative graph state, and
- the runtime emits `NodeChange` / `EdgeChange` events that the application applies.

In `fret-node`, the canonical document is `core::Graph` (hash maps) and undo/redo is expressed as
reversible `ops::GraphTransaction`. Controlled mode is therefore best understood as:

- **Events**: `runtime::changes::NodeGraphChanges` (`NodeChange` + `EdgeChange`)
- **Apply helpers**: `runtime::apply::{apply_node_changes, apply_edge_changes}`
- **Viewport/selection**: `io::NodeGraphViewState` and `runtime::events::ViewChange`

## When to use controlled mode

Use controlled mode when:

- your graph is owned by an external system (collaboration, authoritative server, ECS, domain engine),
- you want to treat `NodeGraphStore` as a *derived cache* / interaction layer, or
- you need to sync edits across multiple runtimes without giving any single store ownership.

If you are building a typical editor UI with a single graph instance, prefer the
**binding-first declarative** path: construct one store-backed `NodeGraphSurfaceBinding`, render
`node_graph_surface(...)`, and use the binding's `NodeGraphController` for imperative viewport or
query work. This keeps undo/redo, lookup caches, and editor interactions in the store/runtime
while teaching the same declarative surface that app code should copy.

## Building blocks (headless-safe)

- Change events:
  - `ecosystem/fret-node/src/runtime/changes.rs` (`NodeChange`, `EdgeChange`, `NodeGraphChanges`)
- Apply helpers:
  - `ecosystem/fret-node/src/runtime/apply.rs` (`apply_node_changes`, `apply_edge_changes`)
- Callback adapter:
  - `ecosystem/fret-node/src/runtime/callbacks.rs` (`NodeGraphCommitCallbacks`, `NodeGraphViewCallbacks`, `NodeGraphGestureCallbacks`, `NodeGraphCallbacks`, `install_callbacks`)
- Store (recommended runtime interaction surface):
  - `ecosystem/fret-node/src/runtime/store.rs` (`NodeGraphStore`)
- Controller + declarative surface:
  - `ecosystem/fret-node/src/ui/controller.rs` (`NodeGraphController`)
  - `ecosystem/fret-node/src/ui/binding.rs` (`NodeGraphSurfaceBinding`)
  - `ecosystem/fret-node/src/ui/declarative/mod.rs` (`NodeGraphSurfaceProps`, `node_graph_surface`)

## Pattern A - Binding-first declarative surface (recommended default)

- Create one `NodeGraphSurfaceBinding::new(models, graph, view_state)`.
- Render `node_graph_surface(cx, binding.surface_props())` for the default surface props.
- Route app-facing viewport/query/edit operations through `binding.controller()`.
- Use `binding.undo(...)` / `binding.redo(...)` when app code wants history actions without manually re-syncing graph/view mirrors.
- Optionally attach callbacks to the store (`install_callbacks`) when app-owned integration needs
  commit/view notifications.
  - Apps usually implement `NodeGraphCommitCallbacks` and optionally `NodeGraphViewCallbacks`.
  - Retained UI glue owns `NodeGraphGestureCallbacks` only when transient gesture lifecycle matters.

This is the default used by `apps/fret-examples/src/node_graph_demo.rs`.

## Pattern B — Controlled graph (application-owned)

High-level idea:

1) your app owns `Graph` (and usually `NodeGraphViewState`) as the source of truth,
2) you still run a `NodeGraphStore` for interaction and change derivation,
3) you apply the store’s changes to your app-owned graph via `apply_*_changes`,
4) when your app graph changes externally, you push it back into the store via `replace_graph`.

### Minimal wiring sketch

```rust
use fret_node::runtime::apply::{apply_edge_changes, apply_node_changes};
use fret_node::runtime::callbacks::{
    install_callbacks, NodeGraphCommitCallbacks, NodeGraphGestureCallbacks,
    NodeGraphViewCallbacks,
};

struct ControlledGraph {
    // Your app-owned graph state.
    graph: std::rc::Rc<std::cell::RefCell<fret_node::core::Graph>>,
}

impl NodeGraphCommitCallbacks for ControlledGraph {
    fn on_nodes_change(&mut self, changes: &[fret_node::runtime::changes::NodeChange]) {
        apply_node_changes(&mut self.graph.borrow_mut(), changes);
    }

    fn on_edges_change(&mut self, changes: &[fret_node::runtime::changes::EdgeChange]) {
        apply_edge_changes(&mut self.graph.borrow_mut(), changes);
    }
}

impl NodeGraphViewCallbacks for ControlledGraph {}

impl NodeGraphGestureCallbacks for ControlledGraph {}

// install_callbacks(&mut store, ControlledGraph { graph: ... });
```

### Notes and gotchas

- `apply_*_changes` is best-effort and order-preserving; it intentionally mirrors XyFlow’s
  “apply changes to your owned state” workflow.
- If you require full-fidelity, reversible edits, prefer applying committed transactions
  (`GraphTransaction`) via `ops::apply_transaction` instead of applying `NodeChange`/`EdgeChange`.
- Viewport/selection is modeled separately:
  - app-owned view state: `io::NodeGraphViewState`
  - change events: `runtime::events::ViewChange` via `NodeGraphViewCallbacks` (`on_view_change` / `on_viewport_change`)

## Runnable example

See `ecosystem/fret-node/examples/controlled_mode.rs`.

