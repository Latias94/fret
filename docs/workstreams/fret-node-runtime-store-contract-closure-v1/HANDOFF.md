# `fret-node` Runtime/Store Contract Closure (v1) - Handoff

Status: active
Last updated: 2026-05-26

## Current State

The workstream has been opened from the `fret-node` audit findings. `FNRS-010`, `FNRS-020`,
`FNRS-030`, and `FNRS-040` are complete.

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
- Fresh validation:
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.
  - `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers new_binding_seeds_graph_view_and_store_models from_store_clones_initial_store_state_into_surface_models`: passed, 3 tests.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo check -p fret-node --features compat-retained-canvas`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  - `cargo clippy -p fret-node --features compat-retained-canvas --all-targets -- -D warnings`
    failed in unrelated `crates/fret-ui/src/tree/layout/clean_geometry.rs` lints before reaching
    `fret-node`.

Primary remaining finding:

Feature and documentation contracts still need closure: the `headless` feature wording is
misleading with defaults enabled, the `fret-ui-kit` dependency boundary needs a deliberate docs/code
decision, and large crate-root policy scans should move toward focused tests/audit helpers.

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

Run `FNRS-050 - Clean feature, dependency-boundary, and policy-test contracts`.

Expected workflow:

1. Use `run-workstream-task` for `FNRS-050`.
2. Start with the Cargo feature matrix in `ecosystem/fret-node/Cargo.toml`.
3. Decide whether to change feature names/code or document the current `headless` + defaults
   behavior explicitly.
4. Resolve the `fret-ui-kit` roadmap/dependency tension.
5. Move at least the most obvious large crate-root policy scan out of `src/lib.rs` if practical.
4. Run fresh gates and update `EVIDENCE_AND_GATES.md`.
6. Mark `FNRS-050` complete in `TODO.md` only after reviewable evidence exists.

## Known Constraints

- Do not edit unrelated user changes.
- Do not use `git restore`, `git checkout --`, `git reset`, or `stash` to remove changes.
- Prefer `cargo nextest run` for Rust tests.
- Repository docs and code comments should stay English.

## Parallelism

Parallel workers are not recommended for FNRS-050 unless docs-only and code/test movement scopes are
kept separate.

After FNRS-020:

- FNRS-040 UI mirror inventory can run in parallel with FNRS-050 feature/docs cleanup only if their
  file scopes stay disjoint.
