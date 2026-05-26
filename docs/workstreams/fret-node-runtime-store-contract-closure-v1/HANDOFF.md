# `fret-node` Runtime/Store Contract Closure (v1) - Handoff

Status: active
Last updated: 2026-05-26

## Current State

The workstream has been opened from the `fret-node` audit findings. `FNRS-010` through `FNRS-050`
are complete.

Completed:

- `NodeGraphChanges::from_transaction` now covers node/edge metadata edits that were previously
  dropped.
- Cascaded edge removals from node/port deletion are now reported as edge changes.
- The mapping no longer has a catch-all silent drop; non-node/edge graph resources are explicit
  non-node/edge cases that require the committed `GraphTransaction` for full-fidelity controlled
  sync.
- `NodeGraphLookups::apply_op` now updates hidden state, reconnectability, removed node ports,
  cascaded edge removal, and detached group parent state.
- Lookup apply no longer has a catch-all success arm; lookup-unaffected operations are explicit.
- `NodeGraphStore` dispatch/undo/redo paths now share common graph-state install/publish helpers.
- A dispatch coherency test proves graph state, changes, lookups, history, and subscribers observe
  the same committed metadata update.
- `NodeGraphSurfaceBinding` mirrors are quarantined behind private `NodeGraphSurfaceMirrors`.
- `UI_MIRROR_INVENTORY_2026-05-26.md` records remaining UI mirror ownership and risk.
- Feature/docs contract cleanup is complete:
  - `headless` is documented as a no-default build marker.
  - `fret-ui-kit` dependency reality is reflected in the roadmap.
  - large crate-root surface-policy scans live in `src/surface_policy_tests.rs`.
- Fresh validation:
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.
  - `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers new_binding_seeds_graph_view_and_store_models from_store_clones_initial_store_state_into_surface_models`: passed, 3 tests.
  - `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers first_party_node_graph_demos_stay_declarative_only`: passed, 2 tests.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo check -p fret-node --no-default-features --features headless`: passed.
  - `cargo check -p fret-node --features compat-retained-canvas`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  - `cargo clippy -p fret-node --features compat-retained-canvas --all-targets -- -D warnings`
    failed in unrelated `crates/fret-ui/src/tree/layout/clean_geometry.rs` lints before reaching
    `fret-node`.

Primary remaining finding:

Implementation slices are complete. The remaining task is closeout verification and deciding
whether any retained-canvas mirror cleanup should split into a follow-on workstream.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

Related background:

- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/`
- `docs/workstreams/crate-audits/fret-node.l0.md`
- `docs/node-graph-roadmap.md`

## Next Task

Run `FNRS-060 - Closeout verification and follow-on split`.

Expected workflow:

1. Use `verify-rust-workstream` for final closeout gates.
2. Run the closeout gate set from `EVIDENCE_AND_GATES.md`.
3. Record any skipped gate with reason.
4. Add a closeout audit or split follow-on list.
5. Mark `FNRS-060` complete and close the workstream if gates pass.

## Known Constraints

- Do not edit unrelated user changes.
- Do not use `git restore`, `git checkout --`, `git reset`, or `stash` to remove changes.
- Prefer `cargo nextest run` for Rust tests.
- Repository docs and code comments should stay English.

## Parallelism

Parallel workers are not needed for closeout.

After FNRS-020:

- FNRS-040 UI mirror inventory can run in parallel with FNRS-050 feature/docs cleanup only if their
  file scopes stay disjoint.
