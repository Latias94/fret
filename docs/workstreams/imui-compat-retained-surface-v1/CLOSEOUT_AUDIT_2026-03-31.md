# Closeout Audit — 2026-03-31

This document closes `imui-compat-retained-surface-v1`.

Scope:

- reduce public/proof `imui` surfaces that still exposed retained-backed compatibility helpers,
- prefer the existing declarative compatibility surface where retained hosting still must survive,
- and delete compatibility facades that no longer have a justified first-party or workspace caller.

## Final decisions

### 1. The first-party node-graph proof survives, but it no longer teaches raw retained hosting

`apps/fret-examples/src/imui_node_graph_demo.rs` still exists as the sole first-party `imui`
compatibility proof for node graph.

What changed:

- it no longer imports `fret_ui::retained_bridge::*`,
- it no longer calls `fret_node::imui::retained_subtree_with(...)`,
- and it now hosts the proof through
  `fret_node::ui::declarative::node_graph_surface_compat_retained(...)`.

Conclusion:

- the proof remains visible,
- but the taught seam is now the declarative compatibility surface rather than a raw retained
  subtree helper.

### 2. `fret-node::imui` is deleted

The public `fret-node` compatibility facade was delete-ready and had no justified caller left once
the first-party proof moved to the declarative compatibility surface.

Deleted:

- `fret_node::imui::retained_subtree(...)`
- `fret_node::imui::retained_subtree_with(...)`
- crate feature `fret-node/imui`
- optional dependency on `fret-authoring`

What remains:

- `fret_node::ui::declarative::node_graph_surface_compat_retained(...)`
- `fret-node/compat-retained-canvas`

Conclusion:

- retained-backed node-graph compatibility survives only behind the declarative UI surface that
  hides retained internals from downstream `UiWriter` code.

### 3. `fret-plot::imui` is deleted

The public `fret-plot` immediate retained-canvas facade had no first-party or workspace caller and
no stronger reason to survive than inertia.

Deleted:

- `fret_plot::imui::line_plot_canvas(...)`
- `fret_plot::imui::line_plot_canvas_with(...)`
- crate feature `fret-plot/imui`
- optional dependency on `fret-authoring`

Conclusion:

- this lane does not keep a compatibility namespace alive merely to preserve an unused facade.

### 4. Legacy example feature wiring now names the real compatibility dependency

`apps/fret-examples/Cargo.toml` no longer enables `fret-node/imui` for legacy node-graph demos.
It now enables `fret-node/compat-retained-canvas`, which is the actual remaining compatibility
surface.

Conclusion:

- feature wiring now matches the real boundary instead of routing through a deleted facade name.

## Resulting public/proof surface after closeout

Inside this lane's scope, the surviving retained-backed compatibility story is now:

- one first-party proof demo:
  `apps/fret-examples/src/imui_node_graph_demo.rs`
- one explicit declarative compatibility surface:
  `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`

Deleted from this scope:

- `ecosystem/fret-node/src/imui.rs`
- `ecosystem/fret-plot/src/imui.rs`

## Out of scope after closeout

This closeout does **not** claim that all retained-backed surfaces are gone from the repo.

Still out of scope:

- broader retained widget/canvas internals,
- docking/chart/plot retained-bridge migrations outside `imui` public/proof surfaces,
- full deletion of `compat-retained-canvas`,
- full declarative replacement of all retained node-graph and plot internals.

## Evidence anchors

- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/Cargo.toml`
- `apps/fret-examples/src/node_graph_legacy_demo.rs`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/README.md`
- `ecosystem/fret-plot/src/lib.rs`
- `ecosystem/fret-plot/Cargo.toml`

## Closeout verdict

`imui-compat-retained-surface-v1` is complete.

The repo no longer teaches or exports raw public `imui` retained compatibility facades for node
graph and plot.
The only surviving node-graph compatibility proof is now routed through the declarative
compatibility surface that was already the intended delete-ready seam.
