# Fret Node Paint Root Pass Clip Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## PCA-M0 - Scope Freeze

Exit criteria:

- The lane states that `cached_pass.rs` has no direct pass-router `cx.scene` access.
- The lane owns immediate pass static scene routing only.
- Validation commands and evidence anchors are listed.

Status: Complete.

## PCA-M1 - Pass Scene Adapter Seam

Exit criteria:

- A pass scene adapter exists under `paint_root/`.
- A retained `PaintCx` binding owns scene/services/scale-factor reads.
- `immediate_pass.rs` delegates static group/node scene routing through the adapter.
- Source-policy coverage locks the seam.

Status: Complete.

## PCA-M2 - Closeout

Exit criteria:

- Focused and crate gates pass.
- The workstream catalog is valid.
- A closeout audit records shipped state and residual cached follow-ons.

Status: Complete.
