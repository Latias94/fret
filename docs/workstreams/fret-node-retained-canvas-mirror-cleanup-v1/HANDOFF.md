# `fret-node` Retained Canvas Mirror Cleanup (v1) - Handoff

Status: active
Last updated: 2026-05-27

## Current State

This follow-on lane is open from the closed runtime/store contract workstream. The previous lane is
closed and must stay closed.

Completed:

- NCM-010 scope and evidence freeze.
- NCM-020 retained canvas mirror owner quarantine.

Current task:

- NCM-030: audit store-backed retained sync after mirror quarantine and delete or narrow one
  redundant mirror update path if compatibility gates prove it is safe.

Fresh validation:

- `cargo fmt -p fret-node --check`: passed.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_mirror_owner`:
  passed, 1 test.
- `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers`:
  passed, 1 test.
- `cargo check -p fret-node --features compat-retained-canvas`: passed.
- `cargo check -p fret-node --no-default-features`: passed.
- `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## Constraints

- Do not delete retained public constructors in NCM-020.
- Do not weaken retained compatibility source-policy tests.
- Do not reopen `docs/workstreams/fret-node-runtime-store-contract-closure-v1/`.
- Repository docs and code comments should stay English.

## Next Action

Run NCM-030 with `run-workstream-task`. Start by reading store-backed retained sync paths in
`view_state/sync.rs`, `commit/*`, and `commit_legacy/*`; split a follow-on if deleting a mirror path
would change retained app-observable behavior.
