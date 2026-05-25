# Fret Node Event Runtime Adapter v1 - TODO

Status: Active
Last updated: 2026-05-25

## NEA-M0 - Scope And Evidence Freeze

- [x] NEA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-event-runtime-adapter-v1]
  Goal: Freeze the event runtime adapter problem, target state, first proof, and gates.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-event-runtime-adapter-v1/WORKSTREAM.json`
  Evidence: `docs/workstreams/fret-node-event-runtime-adapter-v1/DESIGN.md`
  Handoff: First proof should target the retained event runtime entrypoint, not pointer policy.

## NEA-M1 - Event Runtime Adapter Proof

- [ ] NEA-020 [owner=unassigned] [deps=NEA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Introduce a named event runtime adapter seam for route preparation and dispatch, isolating
  retained `EventCx` binding outside route orchestration.
  Validation: `cargo check -p fret-node --features compat-retained-canvas`; narrow source-policy test.
  Evidence: `event_router.rs`, `retained_widget_runtime_event.rs`, source-policy test in
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Keep command dispatch and paint/prepaint out of this task.

## NEA-M2 - Retained Edge Shrink

- [ ] NEA-030 [owner=unassigned] [deps=NEA-020] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Delete or quarantine one old retained event runtime edge replaced by the adapter proof.
  Validation: `cargo check -p fret-node`; `cargo check -p fret-node --features compat-retained-canvas`
  Evidence: source-policy test plus retained adapter module.
  Handoff: If route internals expand beyond event runtime entrypoint wiring, split a narrower lane.

## NEA-M3 - Closeout Or Follow-On Split

- [ ] NEA-040 [owner=planner] [deps=NEA-030] [scope=docs/workstreams/fret-node-event-runtime-adapter-v1]
  Goal: Close this event runtime adapter lane or split the next route-family follow-on.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, optional closeout audit.
  Handoff: Event route policy rewrites require a fresh scoped lane.
