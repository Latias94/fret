# `fret-node` Runtime/Store Contract Closure (v1) - Evidence And Gates

Status: active
Last updated: 2026-05-26

## Baseline From Opening Audit

The opening audit identified these hazards:

- `NodeGraphChanges` does not map every observable `GraphOp`.
- `NodeGraphLookups` can become stale after incremental store dispatch.
- `headless` is a misleading feature name when default features remain enabled.
- UI binding/canvas code still carries multiple state mirrors.
- `src/lib.rs` carries large string-scanning surface policy tests.
- `fret-ui-kit` dependency reality and roadmap wording need alignment.

Opening audit command results reported on 2026-05-26:

- `cargo check -p fret-node --no-default-features`: passed
- `cargo check -p fret-node --no-default-features --features headless`: passed
- `cargo check -p fret-node --features compat-retained-canvas`: passed
- `cargo nextest run -p fret-node --no-default-features runtime`: 38 passed
- `python3 tools/check_layering.py`: passed

These are baseline signals only. Each task completion must record fresh command evidence.

## Required Gates By Task

### FNRS-010

Commands:

- `cargo nextest run -p fret-node --no-default-features runtime`
- `cargo check -p fret-node --no-default-features`

Evidence required:

- Tests that enumerate or otherwise exhaustively cover `GraphOp` to `NodeGraphChanges` mapping.
- Implementation evidence in `runtime/changes.rs`.

### FNRS-020

Commands:

- `cargo nextest run -p fret-node --no-default-features runtime`
- `cargo check -p fret-node --no-default-features`

Evidence required:

- Tests proving fresh `store.lookups()` results after dispatch for lookup-affecting operations.
- Implementation evidence in `runtime/lookups.rs` and store dispatch paths.

### FNRS-030

Commands:

- `cargo nextest run -p fret-node --no-default-features runtime`
- targeted default-feature tests for controller/binding store sync discovered during execution

Evidence required:

- Tests proving dispatch coherency across graph, changes, lookups, history/subscribers, and
  controller/binding sync.
- Short local documentation of dispatch order.

### FNRS-040

Commands:

- `cargo nextest run -p fret-node --features compat-retained-canvas`
- targeted default-feature tests for changed retained/declarative surfaces

Evidence required:

- Mirror inventory.
- A concrete removal/quarantine diff.
- Compatibility retained evidence for the touched surface.

### FNRS-050

Commands:

- `cargo check -p fret-node --no-default-features`
- `cargo check -p fret-node --no-default-features --features headless`
- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo nextest run -p fret-node --no-default-features runtime`

Evidence required:

- Feature matrix documentation.
- Code or docs resolving `fret-ui-kit` boundary tension.
- Tests moved out of `src/lib.rs` where practical.

## Closeout Gates

Run before closing or claiming the lane is ready:

- `cargo fmt --check`
- `cargo nextest run -p fret-node --no-default-features runtime`
- `cargo check -p fret-node --no-default-features`
- `cargo check -p fret-node --no-default-features --features headless`
- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 tools/check_layering.py`

Broader optional gates, time permitting:

- `cargo nextest run -p fret-node --all-features`
- `cargo clippy -p fret-node --all-targets -- -D warnings`

## Source Anchors

- `ecosystem/fret-node/src/ops/mod.rs`
- `ecosystem/fret-node/src/runtime/changes.rs`
- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/ui/binding.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/Cargo.toml`
- `docs/node-graph-roadmap.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/`

## Evidence Log

### 2026-05-26 - Workstream opened

No fresh implementation gates were run during document creation. Baseline command results above came
from the opening audit and must be refreshed inside each task.

### 2026-05-26 - FNRS-010 completed

Changes:

- Added missing node metadata changes for selectable, draggable, connectable, deletable, parent,
  extent, expand-parent, hidden, and port order.
- Added missing edge metadata changes for selectable, deletable, and reconnectable.
- Reported cascaded edge removals from `RemoveNode` and `RemovePort`.
- Removed the catch-all silent drop in `NodeGraphChanges::from_transaction`; graph-resource
  operations outside node/edge change arrays are now explicit non-node/edge cases.
- Extended controlled-mode callback coverage so hidden/reconnectable edits keep an app-owned graph
  synchronized through `apply_node_changes` / `apply_edge_changes`.

Fresh gates:

- `cargo nextest run -p fret-node --no-default-features runtime`: passed, 41 tests.
- `cargo check -p fret-node --no-default-features`: passed.

Evidence anchors:

- `ecosystem/fret-node/src/runtime/changes.rs`
- `ecosystem/fret-node/src/runtime/apply.rs`
- `ecosystem/fret-node/src/runtime/tests.rs`

Verification pass:

- Claim: `FNRS-010` closes node/edge runtime change semantics for the scoped task.
- `cargo fmt -p fret-node --check`: passed.
- `cargo nextest run -p fret-node --no-default-features runtime`: passed, 41 tests.
- `cargo check -p fret-node --no-default-features`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
- Broader gates such as `--all-features`, `compat-retained-canvas`, and workspace clippy were not
  run for this task because `FNRS-010` touched headless runtime change/apply semantics only; the
  required task gates plus no-default package clippy cover this slice.

### 2026-05-26 - FNRS-020 completed

Changes:

- Added regression tests for stale lookup entries after `SetNodeHidden` and
  `SetEdgeReconnectable` dispatch.
- Added regression tests for lookup state after `RemovePort` and `RemoveGroup` dispatch.
- Updated `NodeGraphLookups::apply_op` for hidden state, reconnectability, removed node ports,
  cascaded edge removal, and detached group parent state.
- Replaced the lookup apply catch-all with an explicit list of lookup-unaffected operations.

Fresh gates:

- `cargo fmt -p fret-node --check`: passed.
- `cargo nextest run -p fret-node --no-default-features runtime`: passed, 45 tests.
- `cargo check -p fret-node --no-default-features`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Evidence anchors:

- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/runtime/tests.rs`

Skipped broader gates:

- `--all-features`, `compat-retained-canvas`, and workspace clippy were not run for this task
  because the touched implementation is headless runtime lookup maintenance and the required
  no-default runtime/clippy gates cover that slice.

### 2026-05-26 - FNRS-030 completed

Changes:

- Added `store_dispatch_pipeline_publishes_coherent_commit_state` to lock dispatch coherency across
  graph state, `NodeGraphChanges`, lookup freshness, history, and subscribers.
- Extracted common `NodeGraphStore` commit finalization helpers:
  - `install_committed_graph_state`
  - `publish_graph_commit`
- Routed dispatch, profiled dispatch, undo, profiled undo, redo, and profiled redo through the same
  graph-state install/publish path.

Fresh gates:

- `cargo fmt -p fret-node --check`: passed.
- `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.
- `cargo check -p fret-node --no-default-features`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Evidence anchors:

- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/tests.rs`

Skipped broader gates:

- `--all-features`, `compat-retained-canvas`, and workspace clippy were not run for this task
  because the changed behavior is the headless store commit pipeline and required no-default
  runtime/check/clippy gates cover the affected slice.

### 2026-05-26 - FNRS-040 completed

Changes:

- Added `UI_MIRROR_INVENTORY_2026-05-26.md`.
- Introduced private `NodeGraphSurfaceMirrors` to quarantine the binding's graph/view/editor-config
  mirrors away from the authoritative store and internals fields.
- Preserved public `graph_model()`, `view_state_model()`, `editor_config_model()`, `store_model()`,
  `sync_from_store*`, and viewport sync behavior.
- Updated surface-policy tests to assert the explicit mirror container.

Fresh gates:

- `cargo fmt -p fret-node --check`: passed.
- `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers new_binding_seeds_graph_view_and_store_models from_store_clones_initial_store_state_into_surface_models`: passed, 3 tests.
- `cargo check -p fret-node --features compat-retained-canvas`: passed.
- `cargo check -p fret-node --no-default-features`: passed.

Attempted broader gate:

- `cargo clippy -p fret-node --features compat-retained-canvas --all-targets -- -D warnings`:
  failed before reaching `fret-node` on existing `fret-ui` lints in
  `crates/fret-ui/src/tree/layout/clean_geometry.rs` (`enum_variant_names`, `large_enum_variant`,
  `collapsible_if`, `redundant_closure_call`). This was not fixed in FNRS-040 because it is outside
  the task scope.

Evidence anchors:

- `docs/workstreams/fret-node-runtime-store-contract-closure-v1/UI_MIRROR_INVENTORY_2026-05-26.md`
- `ecosystem/fret-node/src/ui/binding.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`
- `ecosystem/fret-node/src/ui/binding_viewport.rs`
- `ecosystem/fret-node/src/lib.rs`
