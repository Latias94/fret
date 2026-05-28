# `fret-node` Declarative Contract Closure v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p fret-node --no-default-features retained_node_graph_current_guidance_stays_declarative
```

This proves the high-risk standalone/ADR guidance does not describe the deleted retained
`NodeGraphCanvas` surface as current app guidance.

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p fret-node --no-default-features retained_node_graph_current_guidance_stays_declarative
cargo nextest run -p fret-node store_dispatch store_middleware store_rejects
cargo nextest run -p fret-node binding_surface controller_surface public_node_graph_guides
```

### Package Gates

```bash
cargo nextest run -p fret-node --no-default-features
cargo nextest run -p fret-node
cargo nextest run -p fret-canvas
```

### Closeout Gates

```bash
cargo fmt --check
cargo nextest run -p fret-node --no-default-features
cargo nextest run -p fret-node
cargo nextest run -p fret-canvas
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
git diff --check
```

Use these narrower gates because the lane is scoped to `fret-node`, selected `fret-canvas`
mechanisms, ADRs, and workstream docs.

### Review Gate

Run `review-workstream` before accepting each task or lane completion. For implementation tasks,
run `verify-rust-workstream` before marking the task, Codex goal, or lane complete.

## Evidence Anchors

- `docs/workstreams/fret-node-declarative-contract-closure-v1/DESIGN.md`
- `docs/workstreams/fret-node-declarative-contract-closure-v1/TODO.md`
- `docs/workstreams/fret-node-declarative-contract-closure-v1/MILESTONES.md`
- `docs/workstreams/standalone/xyflow-gap-analysis.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `docs/adr/0135-node-graph-canvas-middleware.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/ui/binding.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`

## Fresh Evidence

### FNDC-010 Scope And Evidence Freeze - 2026-05-28

- Workstream docs created for the new follow-on lane.
- The lane explicitly does not reopen `fret-node-retained-exit-and-parity-seams-v1`.

### FNDC-020 Retained Current-Fact Drift Closure - 2026-05-28

- `docs/workstreams/standalone/xyflow-gap-analysis.md` now teaches
  `NodeGraphSurfaceBinding`, `node_graph_surface(...)`, and `NodeGraphController` as the current
  app-facing path.
- `docs/adr/0135-node-graph-canvas-middleware.md` is marked superseded and explicitly rejects
  reviving retained `NodeGraphCanvasMiddleware`.
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md` no longer lists node graph retained
  widget paths as current implementation examples.
- `ecosystem/fret-node/src/surface_policy_tests.rs` adds
  `retained_node_graph_current_guidance_stays_declarative` to lock this source-policy outcome.

Fresh gates:

- PASS `cargo nextest run -p fret-node --no-default-features retained_node_graph_current_guidance_stays_declarative`
- PASS `cargo nextest run -p fret-node --no-default-features` (132 tests)
- PASS `cargo fmt --check`
- PASS `python3 -m json.tool docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
- PASS `python3 tools/check_workstream_catalog.py`
- PASS `git diff --check`

### FNDC-030 Store Dispatch Commit Path - 2026-05-28

- `ecosystem/fret-node/src/runtime/store.rs` now routes `dispatch_transaction` and
  `dispatch_transaction_with_profile` through one private `dispatch_transaction_impl`.
- The shared path owns normalization, dispatch validation, `before_dispatch` middleware,
  scratch-graph application, committed-transaction validation, history recording,
  `after_dispatch` middleware, and `GraphCommitted` publication.
- `ecosystem/fret-node/src/runtime/tests.rs` adds
  `store_dispatch_with_external_profile_uses_same_commit_pipeline` to cover the external profile
  entrypoint against middleware ordering, history availability, patch payload, and event
  publication.

Fresh gates:

- PASS `cargo nextest run -p fret-node store_dispatch store_middleware store_rejects` (7 tests)
- PASS `cargo nextest run -p fret-node --no-default-features` (133 tests)
- PASS `cargo fmt --check`
- PASS `python3 -m json.tool docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
- PASS `python3 tools/check_workstream_catalog.py`
- PASS `git diff --check`

### FNDC-040 Binding Mirror Ownership - 2026-05-28

- `ecosystem/fret-node/src/ui/binding.rs` now names the graph/view/editor-config app model bundle
  `NodeGraphSurfaceProjections`, with public docs describing those handles as store-derived
  projection models for observation and explicit advanced sync.
- `ecosystem/fret-node/src/ui/binding_store_sync.rs` and
  `ecosystem/fret-node/src/ui/binding_viewport.rs` now route sync and viewport comments through the
  same projection terminology instead of implying external graph/view mirrors are authoritative.
- `ecosystem/fret-node/src/ui/controller.rs` and binding tests use projection terminology for
  graph/view/config model sync assertions.
- `ecosystem/fret-node/src/ui/binding.rs` adds
  `graph_projection_model_is_not_the_authoritative_store_graph`, proving direct projection-model
  edits do not mutate the authoritative store graph and are overwritten by `sync_from_store`.
- `ecosystem/fret-node/src/surface_policy_tests.rs` locks the internal binding surface terms and
  public guide guidance so downstream docs keep the store-first story.
- `docs/node-graph-how-to-build-like-xyflow.md` and `docs/node-graph-controlled-mode.md` now state
  that projection model handles are observation/sync targets, while mutations must flow through
  binding helpers, `NodeGraphController`, or `NodeGraphStore`.

Fresh gates:

- PASS `cargo nextest run -p fret-node view_projection_model graph_projection_model_is_not_the_authoritative_store_graph binding_surface controller_surface public_node_graph_guides` (12 tests)
- PASS `cargo nextest run -p fret-node binding_guides_keep_projection_models_out_of_the_authority_story graph_projection_model_is_not_the_authoritative_store_graph` (2 tests)
- PASS `cargo nextest run -p fret-node` (448 tests)
- PASS `cargo fmt --check`
- PASS `python3 -m json.tool docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
- PASS `python3 tools/check_workstream_catalog.py`
- PASS `git diff --check`
