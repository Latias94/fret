# Fret Node Paint Root Cached Edge Replay Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## CEA-M0 - Scope Freeze

Exit criteria:

- The lane owns cached edge and edge-label replay scene sinks only.
- Build-state route inputs, temporary scenes, and overlays are explicitly out of scope.
- Validation commands and evidence anchors are listed.

Status: Complete.

## CEA-M1 - Cached Edge Replay Adapter Seam

Exit criteria:

- A cached edge replay adapter exists under `cached_edges/`.
- A retained `PaintCx` binding owns retained scene access.
- `edges/replay.rs` and `labels/replay.rs` route through the adapter seam instead of direct
  `cx.scene` reads.
- Source-policy coverage locks the seam.

Status: Complete.

## CEA-M2 - Closeout

Exit criteria:

- Focused and crate gates pass.
- The workstream catalog is valid.
- A closeout audit records shipped state and residual cached edge follow-ons.

Status: Complete.
