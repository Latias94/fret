# Fret Node Paint Prepaint Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This is a follow-on to `fret-node-low-level-adapter-v1`. The low-level adapter lane proved common
host operations and command dispatch seams. Paint/prepaint is now split into this dedicated lane
because the retained paint tree is broad and should migrate one operation family at a time.

The first recommended slice is prepaint cull-window behavior. `retained_widget_cull_window.rs` and
`retained_widget_cull_window_shift.rs` are narrow enough to prove a retained-agnostic adapter seam
before touching paint root scene emission.

## Active Task

- Task ID: NPA-020
- Owner: unassigned
- Files: `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window.rs`,
  `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window_shift.rs`,
  `ecosystem/fret-node/src/lib.rs`
- Validation: `cargo check -p fret-node --features compat-retained-canvas`; narrow source-policy test
- Status: NEEDS_CONTEXT
- Review: not started
- Evidence: `docs/workstreams/fret-node-paint-prepaint-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Paint/prepaint is split from event routing and command dispatch.
- The first proof should target prepaint cull-window operations, not the full paint tree.

## Blockers

- None known.

## Next Recommended Action

- Execute NPA-020 as a narrow prepaint cull-window adapter proof.
