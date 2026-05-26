# `fret-node` Runtime/Store Contract Closure (v1) - Handoff

Status: active
Last updated: 2026-05-26

## Current State

The workstream has been opened from the `fret-node` audit findings. `FNRS-010` is complete.

Completed:

- `NodeGraphChanges::from_transaction` now covers node/edge metadata edits that were previously
  dropped.
- Cascaded edge removals from node/port deletion are now reported as edge changes.
- The mapping no longer has a catch-all silent drop; non-node/edge graph resources are explicit
  non-node/edge cases that require the committed `GraphTransaction` for full-fidelity controlled
  sync.
- Fresh validation:
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: passed, 41 tests.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

Primary remaining finding:

`NodeGraphLookups` may still become stale after incremental store dispatch for fields such as
hidden state and reconnectability. That is the next correctness layer.

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

Run `FNRS-020 - Make lookup cache updates exhaustive and stale-safe`.

Expected workflow:

1. Use `run-workstream-task` for `FNRS-020`.
2. Add failing tests proving `store.lookups()` is stale after dispatch for hidden and/or
   reconnectability edits.
3. Implement exhaustive incremental lookup updates or an explicit rebuild fallback.
4. Run fresh gates and update `EVIDENCE_AND_GATES.md`.
5. Mark `FNRS-020` complete in `TODO.md` only after reviewable evidence exists.

## Known Constraints

- Do not edit unrelated user changes.
- Do not use `git restore`, `git checkout --`, `git reset`, or `stash` to remove changes.
- Prefer `cargo nextest run` for Rust tests.
- Repository docs and code comments should stay English.

## Parallelism

Parallel workers are not recommended before FNRS-020 is complete, because it defines shared runtime
lookup semantics that later tasks depend on.

After FNRS-020:

- FNRS-040 UI mirror inventory can run in parallel with FNRS-050 feature/docs cleanup only if their
  file scopes stay disjoint.
