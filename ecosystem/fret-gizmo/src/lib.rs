//! Editor-grade 3D gizmos for engine viewports.
//!
//! This crate is intentionally policy-heavy and lives in `ecosystem/` (ADR 0027). The intended
//! rendering topology is:
//! - Gizmo geometry is rendered by the engine into the viewport render target (depth tested).
//! - Fret composites the viewport as an opaque `SceneOp::ViewportSurface` (ADR 0139).
//! - Optional screen-space affordances (labels/HUD) may be rendered as regular UI overlays.

mod gizmo;
mod grid;
mod math;
mod picking;
mod plugin;
mod ring_scale_plugin;
mod style;
mod transform_plugin;
mod view_gizmo;
mod viewport_input;

pub use gizmo::{
    Aabb3, DepthMode, Gizmo, GizmoConfig, GizmoDrawList3d, GizmoHandedness, GizmoInput, GizmoMode,
    GizmoOps, GizmoOrientation, GizmoPhase, GizmoPickPolicy, GizmoPivotMode, GizmoPluginId,
    GizmoResult, GizmoSizePolicy, GizmoState, GizmoTarget3d, GizmoTargetId, GizmoUpdate, HandleId,
    Line3d, Transform3d, Triangle3d, delta_matrix_trs,
};
pub use grid::{Grid3d, Grid3dConfig};
pub use math::{
    DepthRange, ProjectedPoint, Ray3d, ScreenPoint, ViewportRect, project_point, ray_from_screen,
    unproject_point,
};
pub use picking::{
    PickCircle2d, PickConvexQuad2d, PickSegmentCapsule2d, distance_point_to_segment_px,
    point_in_convex_quad, quad_edge_distance,
};
pub use plugin::{
    GizmoPickHit, GizmoPickItem, GizmoPickShape2d, GizmoPlugin, GizmoPluginContext,
    GizmoPluginManager, GizmoPluginManagerConfig, GizmoPluginManagerState,
};
pub use ring_scale_plugin::{RingScaleGizmoConfig, RingScaleGizmoPlugin, RingScaleGizmoState};
pub use style::{
    GizmoPartVisuals, GizmoVisualPreset, GizmoVisuals, ViewGizmoVisualPreset, ViewGizmoVisuals,
};
pub use transform_plugin::TransformGizmoPlugin;
pub use view_gizmo::{
    ViewGizmo, ViewGizmoAnchor, ViewGizmoConfig, ViewGizmoFace, ViewGizmoInput, ViewGizmoLabel,
    ViewGizmoProjection, ViewGizmoSnap, ViewGizmoState, ViewGizmoUpdate,
};
pub use viewport_input::{
    viewport_input_cursor_target_px, viewport_input_cursor_target_px_clamped,
};
