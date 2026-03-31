# Baseline Audit — 2026-03-31

This audit records the starting point for `imui-compat-retained-surface-v1`.

Goal:

- capture which retained-backed public/proof `imui` surfaces still exist after the v2 closeout,
- separate explicit compatibility surfaces from unlabeled inertia,
- and freeze the baseline before the next keep / relabel / delete decisions land.

## Audit inputs

Closeout / historical docs reviewed:

- `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/design.md`
- `docs/workstreams/crate-audits/fret-node.l0.md`

Implementation / proof anchors reviewed:

- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `ecosystem/fret-node/src/imui.rs`
- `ecosystem/fret-node/src/ui/declarative/mod.rs`
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
- `ecosystem/fret-node/README.md`
- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-plot/src/imui.rs`
- `ecosystem/fret-plot/Cargo.toml`

## Findings

### 1. First-party `imui` examples now have exactly one explicit retained-bridge proof surface

`apps/fret-examples/src/imui_node_graph_demo.rs` is currently the only first-party `imui` example
that still hosts a retained subtree.

This file already does two important things correctly:

- it labels itself as a retained-bridge IMUI demo,
- and it explicitly says it is compatibility-oriented and not the default downstream authoring path.

Conclusion:

- this demo is a valid proof anchor,
- but it should remain the exception rather than the normal immediate teaching surface.

### 2. `fret-node::imui` still exposes a raw retained-hosting public immediate facade

`ecosystem/fret-node/src/imui.rs` exports:

- `retained_subtree(...)`
- `retained_subtree_with(...)`

These are public `UiWriter` helpers that directly host retained subtrees through
`RetainedSubtreeProps`.

Conclusion:

- this is a real public compatibility surface,
- and its survival now needs an explicit keep / rename / delete decision rather than silent drift.

### 3. `fret-node` already has a better-labeled declarative compatibility surface

`ecosystem/fret-node/src/ui/declarative/compat_retained.rs` exports
`node_graph_surface_compat_retained(...)`.
Its doc comment already says:

- compatibility surface,
- allows declarative composition today,
- keeps retained authoring out of the downstream API surface,
- and is delete-planned.

Conclusion:

- for node graph, the repo already has a clearer compatibility story than raw immediate retained
  subtree helpers,
- so the raw `fret_node::imui` retained helpers should not be treated as obviously permanent.

### 4. `fret-plot::imui` is in scope because it is public and retained-backed

`ecosystem/fret-plot/src/imui.rs` exports:

- `line_plot_canvas_with(...)`
- `line_plot_canvas(...)`

These helpers host retained plot canvases inside a `UiWriter` surface.

Unlike `fret-node`, `fret-plot` currently depends on `fret-ui` with
`features = ["unstable-retained-bridge"]` directly in `Cargo.toml` rather than through a clearly
named compatibility feature alias.

Conclusion:

- `fret-plot::imui` belongs in this lane's scope as a public retained-backed immediate facade,
- even though broader plot/chart bridge removal remains outside this lane.

### 5. Wider retained-bridge migrations remain out of scope for this lane

This audit also observed retained-bridge usage in other places, including docking and chart
examples.
Those remain important, but they are not this lane's primary target because they are either:

- broader crate-level bridge migrations already tracked elsewhere,
- or non-`imui` public/proof surfaces.

Conclusion:

- keep this lane narrow,
- do not reopen the full retained-bridge exit allowlist from here.

## Decision from this audit

Treat the current repo state as follows:

- one first-party retained-backed `imui` proof demo survives intentionally,
- public retained-backed `imui` ecosystem facades in `fret-node` and `fret-plot` need explicit
  compatibility classification,
- and the next work should focus on containment, labeling, and delete-ready survival decisions
  rather than another broad stack reset.

## Immediate execution consequence

From this point forward:

1. point active `imui` docs entrypoints to this lane,
2. keep `imui_node_graph_demo` as the only explicit retained-backed first-party `imui` proof unless
   new evidence appears,
3. add source-policy gates that keep retained-bridge usage out of the normal first-party `imui`
   example set,
4. make retained-backed public `imui` modules explicitly compatibility-only,
5. defer the final keep / rename / delete decisions to a closeout audit rather than leaving them
   implicit.
