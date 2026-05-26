# Fret Node Paint Root Tail Cleanup Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

Exit criteria:

- The lane is opened as a follow-on from grid diagnostics.
- The problem is root frame tail cleanup pop emission only.
- Non-goals keep cached layer internals, cached/immediate passes, overlays, and pruning out of
  scope.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1/EVIDENCE_AND_GATES.md`

## M1 - Tail Cleanup Adapter Seam

Exit criteria:

- `tail_cleanup_adapter.rs` defines the retained-agnostic tail cleanup contract.
- `tail_cleanup_retained_cx.rs` owns the retained `PaintCx` root frame `PopClip` emission.
- `paint_root/tail.rs` delegates root frame pop emission through the adapter.
- Source-policy coverage proves the seam.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/tail_cleanup_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/tail_cleanup_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/tail.rs`
- `ecosystem/fret-node/src/lib.rs`

## M2 - Closeout

Exit criteria:

- The shipped tail cleanup seam is recorded as closed evidence.
- Residual frame operation families are named as follow-on candidates rather than appended to this
  lane.
- Workstream status is closed.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
