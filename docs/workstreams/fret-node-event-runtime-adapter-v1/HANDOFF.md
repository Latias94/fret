# Fret Node Event Runtime Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This is a follow-on to `fret-node-low-level-adapter-v1`. The low-level adapter lane proved common
host operations and command dispatch seams; event routing is now split into this dedicated lane.

NEA-020 introduced `event_runtime_adapter.rs` as the named event runtime adapter seam. Route
preparation and dispatch now live behind `CanvasEventRuntimeCx`/`dispatch_canvas_event`, while
`retained_widget_runtime_event.rs` binds retained `EventCx` and forwards into the adapter.

## Active Task

- Task ID: NEA-030
- Owner: unassigned
- Files: `ecosystem/fret-node/src/ui/canvas/widget/event_runtime_adapter.rs`,
  `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_event.rs`,
  `ecosystem/fret-node/src/lib.rs`
- Validation: `cargo check -p fret-node`; `cargo check -p fret-node --features compat-retained-canvas`
- Status: READY
- Review: not started
- Evidence: `docs/workstreams/fret-node-event-runtime-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Event routing is split from command dispatch and paint/prepaint work.
- First slice should name the event runtime adapter seam before deeper pointer/keyboard policy work.
- NEA-020 keeps route internals unchanged and moves retained event runtime preparation into the
  adapter seam.

## Blockers

- None known.

## Next Recommended Action

- Execute NEA-030 only if another old retained event runtime edge can be deleted or quarantined
  without expanding into pointer, keyboard, command, or paint policy.
