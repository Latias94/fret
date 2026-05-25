# Fret Node Paint Prepaint Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This is a follow-on to `fret-node-low-level-adapter-v1`. The low-level adapter lane proved common
host operations and command dispatch seams. Paint/prepaint is now split into this dedicated lane
because the retained paint tree is broad and should migrate one operation family at a time.

NPA-020 completed the first slice by moving prepaint cull-window route preparation behind
`prepaint_cull_window_adapter.rs`. `retained_widget_cull_window.rs` now only binds `PrepaintCx` to
the adapter and forwards; `retained_widget_cull_window_shift.rs` records debug output through the
adapter seam.

## Active Task

- Task ID: NPA-030
- Owner: unassigned
- Files: `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_paint.rs`,
  `ecosystem/fret-node/src/ui/canvas/widget/paint_root_helpers.rs`
- Validation: source audit plus `cargo check -p fret-node --features compat-retained-canvas`
- Status: NEEDS_CONTEXT
- Review: not started
- Evidence: `docs/workstreams/fret-node-paint-prepaint-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Paint/prepaint is split from event routing and command dispatch.
- The first proof should target prepaint cull-window operations, not the full paint tree.
- NPA-020 proved the prepaint cull-window seam without migrating paint root scene emission.

## Blockers

- None known.

## Next Recommended Action

- Execute NPA-030 as a source audit of paint root scene emission. If the audit finds more than one
  operation family, split the next task instead of broadening this lane.
