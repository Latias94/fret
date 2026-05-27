# `fret-node` Runtime/Store Contract Closure (v1) - Closeout Audit

Date: 2026-05-27
Status: closed

## Closed Scope

This workstream closed the runtime/store contract hazards found in the `fret-node` audit:

- `GraphOp` to `NodeGraphChanges` now covers the scoped observable node and edge edit semantics.
- Cascaded edge removals from node and port deletion are reported through node/edge changes.
- Non-node/edge graph-resource operations are explicit non-change-array cases instead of being
  silently dropped.
- `NodeGraphLookups` stays fresh for hidden state, reconnectability, removed ports, cascaded edge
  removal, and detached group parent state after store dispatch.
- Store dispatch, profiled dispatch, undo, profiled undo, redo, and profiled redo now use shared
  graph-state install and publish helpers.
- `NodeGraphSurfaceBinding` long-lived mirrors are quarantined behind a private
  `NodeGraphSurfaceMirrors` owner.
- `headless`, default UI, and compatibility-retained feature contracts are documented and validated.
- Large crate-root surface policy scans moved into `surface_policy_tests.rs`.

## Fresh Closeout Gates

- `cargo fmt --check`: passed.
- `cargo fmt -p fret-node --check`: passed.
- `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.
- `cargo check -p fret-node --no-default-features`: passed.
- `cargo check -p fret-node --no-default-features --features headless`: passed.
- `cargo check -p fret-node --features compat-retained-canvas`: passed.
- `python3 tools/check_layering.py`: passed.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

## Follow-Ons

- Retained `NodeGraphCanvas` still owns graph/view/editor-config models for compatibility. Cleanup
  should be split into a retained-compatibility lane with its own tests.
- The broader compat clippy command
  `cargo clippy -p fret-node --features compat-retained-canvas --all-targets -- -D warnings`
  previously failed before reaching `fret-node` because of existing `fret-ui` lints in
  `crates/fret-ui/src/tree/layout/clean_geometry.rs`.
- Future public feature renames, if desired, should be handled as a separate public-contract task.

## Evidence Anchors

- `ecosystem/fret-node/src/runtime/changes.rs`
- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/tests.rs`
- `ecosystem/fret-node/src/ui/binding.rs`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `ecosystem/fret-node/Cargo.toml`
- `docs/node-graph-roadmap.md`
- `docs/workstreams/fret-node-runtime-store-contract-closure-v1/EVIDENCE_AND_GATES.md`
