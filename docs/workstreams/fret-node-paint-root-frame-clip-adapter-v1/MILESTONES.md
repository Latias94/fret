# Fret Node Paint Root Frame Clip Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

Exit criteria:

- Follow-on relationship to the closed frame setup lane is explicit.
- Non-goals exclude diagnostics, background, grid, cached/immediate passes, and tail cleanup.
- Gate set is recorded.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/EVIDENCE_AND_GATES.md`

## M1 - Root Frame Clip Seam

Exit criteria:

- Root frame clip emission no longer directly writes retained `PaintCx::scene` in `frame.rs`.
- The retained `PaintCx::scene` binding lives in a dedicated retained adapter module.
- Source-policy coverage locks the clip adapter boundary.
- Background paint, grid paint, diagnostics, and tail cleanup remain out of scope.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_clip_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_clip_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`
- `ecosystem/fret-node/src/lib.rs`

## M2 - Closeout

Exit criteria:

- The shipped frame clip seam is recorded as closed evidence.
- Residual operation families are named as follow-on candidates rather than appended to this lane.
- Workstream status is closed.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
