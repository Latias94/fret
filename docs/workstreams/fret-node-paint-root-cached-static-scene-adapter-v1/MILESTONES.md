# Fret Node Paint Root Cached Static Scene Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## CSA-M0 - Scope Freeze

Exit criteria:

- The lane owns cached static group/node replay only.
- Cached edge replay and overlays are explicitly out of scope.
- Validation commands and evidence anchors are listed.

Status: Complete.

## CSA-M1 - Cached Static Scene Adapter Seam

Exit criteria:

- A cached static scene adapter exists under `paint_root/`.
- A retained `PaintCx` binding owns host/services/scale/scene reads.
- `static_layer.rs`, `static_cache.rs`, `cached_groups.rs`, and `cached_nodes.rs` route through the
  adapter seam instead of direct retained field reads.
- Source-policy coverage locks the seam.

Status: Complete.

## CSA-M2 - Closeout

Exit criteria:

- Focused and crate gates pass.
- The workstream catalog is valid.
- A closeout audit records shipped state and residual cached follow-ons.

Status: Complete.
