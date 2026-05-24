# UI Frame Pipeline v2 Phase Contract Follow-On v1 - TODO

Status: Active
Last updated: 2026-05-25

## FP2-M0 - Proof Surface Selection

- [ ] FP2-010 [owner=planner] [deps=none] [scope=crates/fret-ui]
  Goal: Select one proof surface and name the phase contract it should prove.
  Validation: Handoff records selected surface, phase owner, and gate.
  Evidence: `docs/workstreams/ui-frame-pipeline-v2-phase-contract-followon-v1/HANDOFF.md`
  Handoff: Avoid choosing a broad perf campaign as the first task.

## FP2-M1 - Phase Gate

- [ ] FP2-020 [owner=unassigned] [deps=FP2-010] [scope=crates/fret-ui]
  Goal: Add a focused gate that catches phase ownership regression for the chosen surface.
  Validation: `cargo test -p fret-ui <chosen-filter>`
  Evidence: test or diag path.
  Handoff: Update ADR alignment if a hard contract changes.
