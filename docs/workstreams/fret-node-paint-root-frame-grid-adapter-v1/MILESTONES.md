# Fret Node Paint Root Frame Grid Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

Exit criteria:

- The lane is opened as a follow-on from background.
- The problem is grid tile cache warmup scene sink access only.
- Non-goals keep grid diagnostics, grid plan policy, tail cleanup, and cached/immediate passes out
  of scope.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-grid-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-paint-root-frame-grid-adapter-v1/EVIDENCE_AND_GATES.md`

## M1 - Grid Tile Cache Adapter Seam

Exit criteria:

- `paint_grid_cache_adapter.rs` defines the retained-agnostic grid cache warmup contract.
- `paint_grid_cache_retained_cx.rs` owns the retained `PaintCx.scene` binding.
- `paint_grid_cache/warm.rs` delegates scene sink access and redraw requests through the adapter.
- Source-policy coverage proves the seam.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_grid_cache/warm.rs`
- `ecosystem/fret-node/src/lib.rs`

## M2 - Closeout

Exit criteria:

- The shipped grid cache warmup seam is recorded as closed evidence.
- Residual grid/frame operation families are named as follow-on candidates rather than appended to
  this lane.
- Workstream status is closed.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-grid-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
