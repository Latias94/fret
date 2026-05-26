# Fret Node Paint Root Frame Grid Diagnostics Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

Exit criteria:

- The lane is opened as a follow-on from grid cache warmup.
- The problem is grid tile diagnostics recording only.
- Non-goals keep grid cache warmup, grid plan policy, tail cleanup, and cached/immediate passes out
  of scope.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1/EVIDENCE_AND_GATES.md`

## M1 - Grid Diagnostics Adapter Seam

Exit criteria:

- `paint_grid_diagnostics_adapter.rs` defines the retained-agnostic grid diagnostics contract.
- `paint_grid_diagnostics_retained_cx.rs` owns the retained `PaintCx` registry write.
- `paint_grid_stats.rs` collects a stats snapshot and delegates recording through the adapter.
- Source-policy coverage proves the seam.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_diagnostics_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_diagnostics_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_stats.rs`
- `ecosystem/fret-node/src/lib.rs`

## M2 - Closeout

Exit criteria:

- The shipped grid diagnostics seam is recorded as closed evidence.
- Residual grid/frame operation families are named as follow-on candidates rather than appended to
  this lane.
- Workstream status is closed.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
