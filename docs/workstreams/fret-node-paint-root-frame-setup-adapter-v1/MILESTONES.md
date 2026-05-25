# Fret Node Paint Root Frame Setup Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- Frame setup audit scope is explicit.
- Non-goals exclude static layer replay/store and cached/immediate passes.
- Gate set is recorded.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/EVIDENCE_AND_GATES.md`

## M1 - Frame Setup Operation-Family Audit

Exit criteria:

- Frame setup operation families are listed with evidence anchors.
- First implementation candidate is selected, or a narrower follow-on is proposed.

Primary gates:

- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/FRAME_SETUP_SCOPE_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/cache.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/background.rs`

## M2 - Bounds/Viewport Frame Seam

Exit criteria:

- Bounds/viewport route inputs no longer require direct retained `PaintCx` reads in frame setup.
- Source-policy coverage locks the frame viewport adapter boundary.
- Diagnostics, clip emission, background paint, and grid paint remain out of scope.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_viewport_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_viewport_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`
- `ecosystem/fret-node/src/lib.rs`

## M3 - Closeout

Exit criteria:

- The shipped frame viewport seam is recorded as closed evidence.
- Residual operation families are named as follow-on candidates rather than appended to this lane.
- Workstream status is closed.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
