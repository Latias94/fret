# `fret-node` Retained Canvas Mirror Cleanup (v1) - Handoff

Status: closed
Last updated: 2026-05-27

## Current State

This follow-on lane is open from the closed runtime/store contract workstream. The previous lane is
closed and must stay closed.

Completed:

- NCM-010 scope and evidence freeze.
- NCM-020 retained canvas mirror owner quarantine.
- NCM-030 store-first retained sync audit:
  - deleted unused `commit_legacy` duplicate retained transaction pipeline,
  - added source-policy coverage for the single retained commit pipeline.
- NCM-040 closeout verification.

Current task:

No current task remains. This workstream is closed.

Fresh validation:

- `cargo fmt -p fret-node --check`: passed.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_mirror_owner`:
  passed, 1 test.
- `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers`:
  passed, 1 test.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_commit_pipeline_has_no_legacy_mirror_writer retained_canvas_mirror_owner`:
  passed, 2 tests.
- `cargo fmt --check`: passed.
- `python3 tools/check_layering.py`: passed.
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

No next action remains for this workstream. Broader public retained-surface removal should use a
separate retained-surface exit lane.
