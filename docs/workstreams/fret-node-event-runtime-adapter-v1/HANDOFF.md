# Fret Node Event Runtime Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This is a follow-on to `fret-node-low-level-adapter-v1`. The low-level adapter lane proved common
host operations and command dispatch seams; event routing is now split into this dedicated lane.

NEA-020 introduced `event_runtime_adapter.rs` as the named event runtime adapter seam. Route
preparation and dispatch now live behind `CanvasEventRuntimeCx`/`dispatch_canvas_event`, while
`retained_widget_runtime_event.rs` binds retained `EventCx` and forwards into the adapter.

NEA-030 audited the remaining retained event runtime path and found no additional old event runtime
edge worth deleting inside this lane. Remaining retained `EventCx` impl files are policy-specific
bindings, and layout/semantics/paint retained runtime paths are outside this event lane.

## Active Task

- Task ID: NEA-040
- Owner: planner
- Files: `docs/workstreams/fret-node-event-runtime-adapter-v1/EVIDENCE_AND_GATES.md`,
  `docs/workstreams/fret-node-event-runtime-adapter-v1/HANDOFF.md`,
  `docs/workstreams/fret-node-event-runtime-adapter-v1/WORKSTREAM.json`
- Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
- Status: READY
- Review: not started
- Evidence: `docs/workstreams/fret-node-event-runtime-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Event routing is split from command dispatch and paint/prepaint work.
- First slice should name the event runtime adapter seam before deeper pointer/keyboard policy work.
- NEA-020 keeps route internals unchanged and moves retained event runtime preparation into the
  adapter seam.
- NEA-030 is audit-only: no further event runtime edge should be forced out of the retained binding
  unless a new route-preparation dependency appears.

## Blockers

- None known.

## Next Recommended Action

- Execute NEA-040 closeout: either close this lane or split a new route-family follow-on. Do not
  reopen command dispatch or paint/prepaint here.
