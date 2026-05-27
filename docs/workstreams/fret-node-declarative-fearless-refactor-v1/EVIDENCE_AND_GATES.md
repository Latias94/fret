# `fret-node` Fearless Refactor (v1) - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Current Focus

The current execution focus is the XYFlow consumer-surface proof: public docs, examples, and source
policy tests should teach `NodeGraphSurfaceBinding + node_graph_surface(...)` first, with
`NodeGraphController` as the lower-level imperative facade and retained canvas paths explicitly
bounded as compatibility/internal surfaces.

## Targeted Iteration Gates

```bash
cargo nextest run -p fret-node public_node_graph_guides_teach_binding_first_surface
```

This gate proves the public crate README and the XyFlow-style guide keep the binding-first teaching
surface and do not drift back to direct retained canvas authoring or stale graph/view/model triplets.

```bash
cargo nextest run -p fret-node --no-default-features runtime
```

This gate protects the headless runtime/change/store behavior while consumer docs reference
`NodeGraphStore`, controlled mode, and transaction-backed changes.

## Package And Boundary Gates

```bash
cargo check -p fret-node --no-default-features
cargo check -p fret-node --features compat-retained-canvas
python3 tools/check_layering.py
```

Use the no-default-features check when changing headless/runtime docs or exports. Use the
compat-retained check when touching retained compatibility boundaries. Use layering checks when
moving mechanisms across `fret-node`, `fret-canvas`, or core crates.

## Closeout Gates

```bash
cargo fmt --check
cargo nextest run -p fret-node
cargo check -p fret-node --features compat-retained-canvas --tests
```

Closeout should use narrower gates only when the workspace is blocked by unrelated failures, and the
closeout note must name those failures.

## Evidence Anchors

- `docs/node-graph-how-to-build-like-xyflow.md`
- `ecosystem/fret-node/README.md`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/README.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/design.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/todo.md`

Fresh verification is required before marking a task, Codex goal, or lane complete.
