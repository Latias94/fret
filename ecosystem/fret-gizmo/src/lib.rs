//! Editor-grade 3D gizmos for engine viewports.
//!
//! This crate is intentionally policy-heavy and lives in `ecosystem/` (ADR 0027). The intended
//! rendering topology is:
//! - Gizmo geometry is rendered by the engine into the viewport render target (depth tested).
//! - Fret composites the viewport via `SceneOp::ViewportSurface` (or `fret-ui`'s declarative
//!   `ViewportSurface` element) (ADR 0130).
//! - Optional screen-space affordances (labels/HUD) may be rendered as regular UI overlays.
//!
//! Optional feature:
//! - `f64-math`: uses f64 internally for projection/unprojection and ray construction to improve
//!   picking stability in very large worlds (public API remains f32).

#![allow(clippy::too_many_arguments)]

mod gizmo;
mod grid;
mod light_radius_plugin;
mod math;
mod picking;
mod plugin;
mod ring_scale_plugin;
mod style;
mod transform_plugin;
mod view_gizmo;

pub use fret_viewport_tooling::{
    ViewportTool, ViewportToolCx, ViewportToolId, ViewportToolInput, ViewportToolPriority,
    ViewportToolResult,
};
pub use gizmo::{
    Aabb3, BUILTIN_HANDLE_GROUP_ROTATE, BUILTIN_HANDLE_GROUP_SCALE, BUILTIN_HANDLE_GROUP_TRANSLATE,
    DepthMode, Gizmo, GizmoConfig, GizmoCustomEdit, GizmoDrawList3d, GizmoHandedness, GizmoInput,
    GizmoMode, GizmoOps, GizmoOrientation, GizmoPhase, GizmoPickPolicy, GizmoPivotMode,
    GizmoPluginId, GizmoPropertyKey, GizmoResult, GizmoSizePolicy, GizmoState, GizmoTarget3d,
    GizmoTargetId, GizmoUpdate, HANDLE_LOCAL_GROUP_SHIFT, HandleId, Line3d, Transform3d,
    Triangle3d, delta_matrix_trs,
};
pub use grid::{Grid3d, Grid3dConfig};
pub use light_radius_plugin::{
    LightRadiusGizmoConfig, LightRadiusGizmoPlugin, LightRadiusGizmoState,
};
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
    GizmoPluginManager, GizmoPluginManagerConfig, GizmoPluginManagerState, GizmoPropertySource,
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

/// Builds a `GizmoInput` from a `ViewportToolInput` plus host-defined policy flags.
pub fn gizmo_input_from_tool_input(
    tool: ViewportToolInput,
    hovered: bool,
    snap: bool,
    cancel: bool,
    precision: f32,
) -> GizmoInput {
    GizmoInput {
        cursor_px: tool.cursor_px,
        hovered,
        drag_started: tool.drag_started,
        dragging: tool.dragging,
        snap,
        cancel,
        precision,
    }
}

/// Builds a `ViewGizmoInput` from a `ViewportToolInput`.
pub fn view_gizmo_input_from_tool_input(tool: ViewportToolInput, hovered: bool) -> ViewGizmoInput {
    ViewGizmoInput {
        cursor_px: tool.cursor_px,
        hovered,
        drag_started: tool.drag_started,
        dragging: tool.dragging,
    }
}
