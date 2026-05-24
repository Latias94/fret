# Fret Launch Root Surface Convergence v1 - TODO

Status: Active
Last updated: 2026-05-25

## LRC-M0 - Gate Baseline

- [ ] LRC-010 [owner=planner] [deps=none] [scope=tools,docs/workstreams/fret-launch-app-surface-fearless-refactor-v1]
  Goal: Run launch posture gates and refresh whether any root-surface drift exists.
  Validation: gate outputs recorded in `EVIDENCE_AND_GATES.md`.
  Evidence: `docs/workstreams/fret-launch-root-surface-convergence-v1/EVIDENCE_AND_GATES.md`
  Handoff: If gates pass and no inventory drift exists, close as maintenance.

## LRC-M1 - First Surface Cleanup If Needed

- [ ] LRC-020 [owner=unassigned] [deps=LRC-010] [scope=crates/fret-launch,ecosystem/fret,examples]
  Goal: Clean one root export, helper, or example posture drift.
  Validation: launch posture gates pass.
  Evidence: export inventory and changed files.
  Handoff: Do not add launch API unless the audit proves a missing hook.
