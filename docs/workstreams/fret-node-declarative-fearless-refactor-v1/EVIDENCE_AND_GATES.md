# `fret-node` Fearless Refactor (v1) - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Current Focus

FNDX-020 is the controlled-sync public-helper decision. Public sync should stay full-replace-first:
`replace_document(...)` for whole-document resets, `replace_graph(...)` for graph-only authoritative
sync, and explicit `GraphTransaction` values for reversible incremental edits. Diff-first
`replace_*_with_diff` helpers remain deferred until a concrete editor-grade workload proves they are
needed.

## Targeted Iteration Gates

```bash
cargo nextest run -p fret-node public_node_graph_guides_teach_binding_first_surface
```

This gate proves the public crate README and the XyFlow-style guide keep the binding-first teaching
surface and do not drift back to direct retained canvas authoring or stale graph/view/model triplets.

```bash
cargo nextest run -p fret-node controlled_sync_public_surface_stays_full_replace_first_until_workload_proves_diff_helper
```

This gate proves the controlled-mode docs keep the FNDX-020 decision explicit and the public
binding/controller sync surfaces have not grown a hidden diff-first replacement helper.

```bash
cargo nextest run -p fret-node controlled_graph_can_apply_store_changes_via_callbacks
```

This gate proves the controlled runtime path still supports app-owned graph state by applying store
`NodeChange` / `EdgeChange` callbacks with `apply_*_changes`.

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
- `docs/node-graph-controlled-mode.md`
- `ecosystem/fret-node/README.md`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `ecosystem/fret-node/src/runtime/tests.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/README.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/design.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/todo.md`

## Fresh Evidence - 2026-05-27

- `cargo nextest run -p fret-node controlled_sync_public_surface_stays_full_replace_first_until_workload_proves_diff_helper`: passed; proves controlled sync docs and public binding/controller sync sources stay full-replace-first and do not expose diff-first helpers.
- `cargo nextest run -p fret-node controlled_graph_can_apply_store_changes_via_callbacks`: passed; proves the current controlled callback/apply path still mirrors store changes into app-owned graph state.
- `cargo fmt --check`: passed; proves the Rust formatting gate is clean after the new source-policy test.
- Broader package/closeout gates were not rerun for FNDX-020 because this slice only changed docs,
  a source-policy test, and workstream notes; use the package/closeout gate list above before
  accepting broader lane closure.

Fresh verification is required before marking a task, Codex goal, or lane complete.
