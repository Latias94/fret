# Fret Node Event Runtime Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This is a follow-on to `fret-node-low-level-adapter-v1`. The low-level adapter lane proved common
host operations and command dispatch seams; event routing is now split into this dedicated lane.

The event route stack already has retained-agnostic composition traits such as `EventRouteCx`,
`SystemRouteCx`, `PointerEventRouteCx`, and `KeyboardRouteCx`. The next slice should focus on the
retained runtime event entrypoint in `retained_widget_runtime_event.rs`, where `EventCx` currently
owns runtime theme sync, view state sync, bounds update, and route dispatch.

## Active Task

- Task ID: NEA-020
- Owner: unassigned
- Files: `ecosystem/fret-node/src/ui/canvas/widget/event_router.rs`,
  `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_event.rs`,
  `ecosystem/fret-node/src/lib.rs`
- Validation: `cargo check -p fret-node --features compat-retained-canvas`; narrow source-policy test
- Status: NEEDS_CONTEXT
- Review: not started
- Evidence: `docs/workstreams/fret-node-event-runtime-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Event routing is split from command dispatch and paint/prepaint work.
- First slice should name the event runtime adapter seam before deeper pointer/keyboard policy work.

## Blockers

- None known.

## Next Recommended Action

- Execute NEA-020 as a narrow adapter proof.
