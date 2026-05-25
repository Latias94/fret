# Fret Node Paint Root Frame Background Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

Exit criteria:

- The lane is opened as a follow-on from diagnostics.
- The problem is background scene emission only.
- Non-goals keep grid paint, tail cleanup, cached/immediate passes, and diagnostics out of scope.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-background-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-paint-root-frame-background-adapter-v1/EVIDENCE_AND_GATES.md`

## M1 - Background Adapter Seam

Exit criteria:

- `frame_background_adapter.rs` defines the retained-agnostic background paint contract.
- `frame_background_retained_cx.rs` owns the retained `PaintCx` scene quad emission.
- `frame/background.rs` resolves the background color and delegates emission through the adapter.
- Source-policy coverage proves the seam.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_background_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_background_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/background.rs`
- `ecosystem/fret-node/src/lib.rs`

## M2 - Closeout

Exit criteria:

- The shipped background seam is recorded as closed evidence.
- Residual operation families are named as follow-on candidates rather than appended to this lane.
- Workstream status is closed.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-background-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
