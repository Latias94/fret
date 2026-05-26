# `fret-node` Runtime/Store Contract Closure (v1) - Handoff

Status: active
Last updated: 2026-05-26

## Current State

The workstream has been opened from the `fret-node` audit findings. `FNRS-010`, `FNRS-020`, and
`FNRS-030` are complete.

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
- Fresh validation:
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Primary remaining finding:

With runtime/store correctness closed, the next risk is UI state mirror drift in binding/controller
sync and retained/declarative compatibility surfaces.

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

Run `FNRS-040 - Reduce UI state mirrors after runtime/store gates are green`.

Expected workflow:

1. Use `run-workstream-task` for `FNRS-040`.
2. Start with an inventory of long-lived UI mirrors in `binding`, `controller_store_sync`, and
   retained/declarative canvas surfaces.
3. Pick one narrow mirror-removal or quarantine slice with a focused compatibility gate.
4. Run fresh gates and update `EVIDENCE_AND_GATES.md`.
5. Mark `FNRS-040` complete in `TODO.md` only after reviewable evidence exists.

## Known Constraints

- Do not edit unrelated user changes.
- Do not use `git restore`, `git checkout --`, `git reset`, or `stash` to remove changes.
- Prefer `cargo nextest run` for Rust tests.
- Repository docs and code comments should stay English.

## Parallelism

Parallel workers may start FNRS-050 inventory-only docs work if needed, but implementation work on
UI mirrors should stay serialized until the first FNRS-040 slice picks exact file scope.

After FNRS-020:

- FNRS-040 UI mirror inventory can run in parallel with FNRS-050 feature/docs cleanup only if their
  file scopes stay disjoint.
