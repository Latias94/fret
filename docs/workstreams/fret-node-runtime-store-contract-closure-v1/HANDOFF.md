# `fret-node` Runtime/Store Contract Closure (v1) - Handoff

Status: active
Last updated: 2026-05-26

## Current State

The workstream has been opened from the `fret-node` audit findings. `FNRS-010` and `FNRS-020` are
complete.

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
- Fresh validation:
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: passed, 45 tests.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Primary remaining finding:

The next risk is pipeline drift: dispatch, change emission, lookup updates, history, subscribers,
and controller/binding sync are still spread across repeated store paths.

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

Run `FNRS-030 - Harden store dispatch as the single runtime commit pipeline`.

Expected workflow:

1. Use `run-workstream-task` for `FNRS-030`.
2. Audit the repeated commit paths in `NodeGraphStore` (`dispatch_transaction`,
   `dispatch_transaction_with_profile`, `undo`, `undo_with_profile`, `redo`, `redo_with_profile`).
3. Add a focused dispatch-order/coherency test before refactoring.
4. Consolidate common commit finalization only if the test exposes meaningful duplication risk.
4. Run fresh gates and update `EVIDENCE_AND_GATES.md`.
5. Mark `FNRS-030` complete in `TODO.md` only after reviewable evidence exists.

## Known Constraints

- Do not edit unrelated user changes.
- Do not use `git restore`, `git checkout --`, `git reset`, or `stash` to remove changes.
- Prefer `cargo nextest run` for Rust tests.
- Repository docs and code comments should stay English.

## Parallelism

Parallel workers may start inventory-only work for FNRS-050, but implementation work should stay
serialized until FNRS-030 confirms the store commit pipeline shape.

After FNRS-020:

- FNRS-040 UI mirror inventory can run in parallel with FNRS-050 feature/docs cleanup only if their
  file scopes stay disjoint.
