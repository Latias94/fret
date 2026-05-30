# `fret-node` Runtime/Store Contract Closure (v1) - Handoff

Status: closed
Last updated: 2026-05-27

## Current State

The workstream has been opened from the `fret-node` audit findings. `FNRS-010` through `FNRS-060`
are complete and the lane is closed.

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
- Fresh closeout validation:
  - `cargo fmt --check`: passed.
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo check -p fret-node --no-default-features --features headless`: passed.
  - `cargo check -p fret-node --features compat-retained-canvas`: passed.
  - `python3 tools/check_layering.py`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Follow-ons:

- Retained `NodeGraphCanvas` graph/view/editor-config mirror cleanup should be handled as a
  separate compatibility workstream or task slice.
- The broader compat clippy command
  `cargo clippy -p fret-node --features compat-retained-canvas --all-targets -- -D warnings`
  previously failed before reaching `fret-node` on unrelated existing `fret-ui` lints in
  `crates/fret-ui/src/tree/layout/clean_geometry.rs`.

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

No next task remains in this workstream. New work should start from a follow-on lane rather than
reopening this one.

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
