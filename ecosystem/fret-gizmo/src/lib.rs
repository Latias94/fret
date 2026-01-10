//! Editor-grade 3D gizmos for engine viewports.
//!
//! This crate is intentionally policy-heavy and lives in `ecosystem/` (ADR 0027). The intended
//! rendering topology is:
//! - Gizmo geometry is rendered by the engine into the viewport render target (depth tested).
//! - Fret composites the viewport as an opaque `SceneOp::ViewportSurface` (ADR 0139).
//! - Optional screen-space affordances (labels/HUD) may be rendered as regular UI overlays.

mod gizmo;
mod math;

pub use gizmo::{
    Aabb3, DepthMode, Gizmo, GizmoConfig, GizmoDrawList3d, GizmoInput, GizmoMode, GizmoOrientation,
    GizmoPhase, GizmoPivotMode, GizmoResult, GizmoState, GizmoTarget3d, GizmoTargetId, GizmoUpdate,
    HandleId, Line3d, Transform3d, Triangle3d,
};
pub use math::{
    DepthRange, ProjectedPoint, Ray3d, ScreenPoint, ViewportRect, project_point, ray_from_screen,
    unproject_point,
};
