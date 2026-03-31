# imui compatibility retained surface reduction v1 - TODO

Tracking doc: `docs/workstreams/imui-compat-retained-surface-v1/DESIGN.md`

Milestones: `docs/workstreams/imui-compat-retained-surface-v1/MILESTONES.md`

Baseline audit:
`docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`

Closeout audit:
`docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`

This board assumes a docs-first, delete-ready migration.
Compatibility aliases and unlabeled escape hatches are out of scope.

## M0 - Source-of-truth handoff

- [x] Create a new workstream directory with `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and a
      baseline audit.
- [x] Repoint top-level docs entrypoints from `imui-stack-fearless-refactor-v2/` to this new
      active follow-on.
- [x] Keep `imui-stack-fearless-refactor-v2/` as the closed closeout record instead of pretending
      it is still active.

## M1 - Baseline classification

- [x] Audit the first-party retained-backed `imui` proof surface in
      `apps/fret-examples/src/imui_node_graph_demo.rs`.
- [x] Audit the public retained-backed `fret-node::imui` surface.
- [x] Audit the declarative retained compatibility surface in
      `fret_node::ui::declarative::compat_retained`.
- [x] Audit the public retained-backed `fret-plot::imui` surface.
- [x] State which retained-backed surfaces are in scope for this lane and which remain out of scope.

Baseline result (2026-03-31):

- `imui_node_graph_demo` is the only current first-party `imui` example that intentionally hosts a
  retained subtree, and it already carries an explicit compatibility warning.
- `fret_node::imui` remains a public immediate retained-hosting facade with lower-level
  `retained_subtree(...)` / `retained_subtree_with(...)` entrypoints.
- `fret_node::ui::declarative::node_graph_surface_compat_retained(...)` already exists as a
  clearer declarative compatibility surface and is explicitly marked compatibility-only and
  delete-planned.
- `fret_plot::imui` remains a public retained-canvas immediate facade and therefore belongs in
  scope for explicit classification, even though the broader chart/plot bridge migration remains
  out of scope.

## M2 - Teaching-surface gates

- [x] Add a first-party source-policy gate that keeps retained-bridge authoring out of the normal
      `imui` example set.
- [x] Keep `imui_node_graph_demo` explicitly labeled as the compatibility-only retained proof.
- [x] Decide that `imui_node_graph_demo` remains the sole first-party proof in this lane, but only
      as a declarative compatibility proof instead of a raw retained-hosting teaching surface.

## M3 - Ecosystem compatibility labeling

- [x] Make `ecosystem/fret-node/src/imui.rs` explicitly compatibility-only in module docs.
- [x] Add a source-policy test that locks the `fret-node` compatibility labeling.
- [x] Make `ecosystem/fret-plot/src/imui.rs` explicitly compatibility-only in module docs.
- [x] Add a source-policy test that locks the `fret-plot` compatibility labeling.
- [x] Decide that `fret_plot::imui` should be deleted instead of renamed under a narrower
      compatibility namespace.

## M4 - Delete-ready decisions

- [x] Delete `fret_node::imui::retained_subtree(...)`.
- [x] Delete `fret_node::imui::retained_subtree_with(...)`.
- [x] Delete `fret_plot::imui::line_plot_canvas(...)`.
- [x] Delete `fret_plot::imui::line_plot_canvas_with(...)`.
- [x] Capture a final closeout audit that states what stays, what is deleted, and what proof
      surface remains.

Final result (2026-03-31):

- `imui_node_graph_demo` survives as the sole first-party proof in scope, but it now teaches
  `fret_node::ui::declarative::node_graph_surface_compat_retained(...)` instead of raw retained
  subtree hosting.
- `fret_node::imui` is deleted together with the `fret-node/imui` feature and `fret-authoring`
  dependency.
- `fret_plot::imui` is deleted together with the `fret-plot/imui` feature and `fret-authoring`
  dependency.
- `fret-node/compat-retained-canvas` remains the explicit compatibility feature for the surviving
  declarative node-graph seam.
