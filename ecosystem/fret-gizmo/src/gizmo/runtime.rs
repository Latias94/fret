use glam::{Mat4, Quat, Vec2, Vec3};

use crate::style::GizmoPartVisuals;

use super::{GizmoMode, GizmoTarget3d, HandleId, delta_matrix_trs};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoInput {
    pub cursor_px: Vec2,
    pub hovered: bool,
    pub drag_started: bool,
    pub dragging: bool,
    pub snap: bool,
    pub cancel: bool,
    /// Drag precision multiplier (1.0 = normal). Values < 1.0 reduce sensitivity for fine control.
    ///
    /// This is intentionally host-defined (no hard-coded keybindings) so editor apps can map it to
    /// any modifier scheme (e.g. Ctrl/Alt/Shift).
    pub precision: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GizmoState {
    pub hovered: Option<HandleId>,
    pub active: Option<HandleId>,
    pub hovered_kind: Option<GizmoMode>,
    pub(super) part_visuals: GizmoPartVisuals,
    pub(super) drag_start_targets: Vec<GizmoTarget3d>,
    pub(super) drag_mode: GizmoMode,
    pub(super) drag_snap: bool,
    pub(super) drag_has_started: bool,
    pub(super) drag_start_cursor_px: Vec2,
    pub(super) drag_axis_dir: Vec3,
    pub(super) drag_origin: Vec3,
    pub(super) drag_origin_z01: f32,
    pub(super) drag_size_length_world: f32,
    pub(super) drag_plane_normal: Vec3,
    pub(super) drag_start_hit_world: Vec3,
    pub(super) drag_prev_hit_world: Vec3,
    pub(super) drag_translate_prev_axis_raw: f32,
    pub(super) drag_translate_prev_plane_raw: Vec2,
    pub(super) drag_scale_prev_axis_raw: f32,
    pub(super) drag_scale_prev_plane_raw: Vec2,
    pub(super) drag_bounds_prev_local_raw: Vec3,
    pub(super) drag_total_axis_raw: f32,
    pub(super) drag_total_axis_applied: f32,
    pub(super) drag_translate_is_plane: bool,
    pub(super) drag_translate_is_dolly: bool,
    pub(super) drag_translate_dolly_world_per_px: f32,
    pub(super) drag_translate_u: Vec3,
    pub(super) drag_translate_v: Vec3,
    pub(super) drag_total_plane_raw: Vec2,
    pub(super) drag_total_plane_applied: Vec2,
    pub(super) drag_basis_u: Vec3,
    pub(super) drag_basis_v: Vec3,
    pub(super) drag_start_angle: f32,
    pub(super) drag_prev_angle: f32,
    pub(super) drag_total_angle_raw: f32,
    pub(super) drag_total_angle_applied: f32,
    pub(super) drag_rotate_is_arcball: bool,
    pub(super) drag_arcball_center_px: Vec2,
    pub(super) drag_arcball_radius_px: f32,
    pub(super) drag_arcball_prev_vec: Vec3,
    pub(super) drag_total_arcball_raw: Quat,
    pub(super) drag_total_arcball_applied: Quat,
    pub(super) drag_scale_is_bounds: bool,
    pub(super) drag_bounds_basis: [Vec3; 3],
    pub(super) drag_bounds_min_local: Vec3,
    pub(super) drag_bounds_max_local: Vec3,
    pub(super) drag_bounds_anchor_local: Vec3,
    pub(super) drag_bounds_axes_mask: [bool; 3],
    pub(super) drag_bounds_axis_sign: [f32; 3],
    pub(super) drag_bounds_start_extent: Vec3,
    pub(super) drag_bounds_total_raw: Vec3,
    pub(super) drag_bounds_total_applied: Vec3,
    pub(super) drag_scale_axis: Option<usize>,
    pub(super) drag_scale_plane_axes: Option<(usize, usize)>,
    pub(super) drag_scale_plane_u: Vec3,
    pub(super) drag_scale_plane_v: Vec3,
    pub(super) drag_scale_is_uniform: bool,
    pub(super) drag_total_scale_raw: f32,
    pub(super) drag_total_scale_applied: f32,
    pub(super) drag_total_scale_plane_raw: Vec2,
    pub(super) drag_total_scale_plane_applied: Vec2,
}

impl GizmoState {
    pub fn is_over(&self) -> bool {
        self.hovered.is_some()
    }

    pub fn is_using(&self) -> bool {
        self.active.is_some()
    }

    pub fn is_over_handle(&self, handle: HandleId) -> bool {
        self.hovered == Some(handle)
    }

    pub fn is_using_handle(&self, handle: HandleId) -> bool {
        self.active == Some(handle)
    }
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            hovered: None,
            active: None,
            hovered_kind: None,
            part_visuals: GizmoPartVisuals::default(),
            drag_start_targets: Vec::new(),
            drag_mode: GizmoMode::Translate,
            drag_snap: false,
            drag_has_started: false,
            drag_start_cursor_px: Vec2::ZERO,
            drag_axis_dir: Vec3::X,
            drag_origin: Vec3::ZERO,
            drag_origin_z01: 0.0,
            drag_size_length_world: 1.0,
            drag_plane_normal: Vec3::Z,
            drag_start_hit_world: Vec3::ZERO,
            drag_prev_hit_world: Vec3::ZERO,
            drag_translate_prev_axis_raw: 0.0,
            drag_translate_prev_plane_raw: Vec2::ZERO,
            drag_scale_prev_axis_raw: 0.0,
            drag_scale_prev_plane_raw: Vec2::ZERO,
            drag_bounds_prev_local_raw: Vec3::ZERO,
            drag_total_axis_raw: 0.0,
            drag_total_axis_applied: 0.0,
            drag_translate_is_plane: false,
            drag_translate_is_dolly: false,
            drag_translate_dolly_world_per_px: 0.0,
            drag_translate_u: Vec3::X,
            drag_translate_v: Vec3::Y,
            drag_total_plane_raw: Vec2::ZERO,
            drag_total_plane_applied: Vec2::ZERO,
            drag_basis_u: Vec3::X,
            drag_basis_v: Vec3::Y,
            drag_start_angle: 0.0,
            drag_prev_angle: 0.0,
            drag_total_angle_raw: 0.0,
            drag_total_angle_applied: 0.0,
            drag_rotate_is_arcball: false,
            drag_arcball_center_px: Vec2::ZERO,
            drag_arcball_radius_px: 0.0,
            drag_arcball_prev_vec: Vec3::Z,
            drag_total_arcball_raw: Quat::IDENTITY,
            drag_total_arcball_applied: Quat::IDENTITY,
            drag_scale_is_bounds: false,
            drag_bounds_basis: [Vec3::X, Vec3::Y, Vec3::Z],
            drag_bounds_min_local: Vec3::ZERO,
            drag_bounds_max_local: Vec3::ZERO,
            drag_bounds_anchor_local: Vec3::ZERO,
            drag_bounds_axes_mask: [false; 3],
            drag_bounds_axis_sign: [1.0; 3],
            drag_bounds_start_extent: Vec3::ONE,
            drag_bounds_total_raw: Vec3::ONE,
            drag_bounds_total_applied: Vec3::ONE,
            drag_scale_axis: None,
            drag_scale_plane_axes: None,
            drag_scale_plane_u: Vec3::X,
            drag_scale_plane_v: Vec3::Y,
            drag_scale_is_uniform: false,
            drag_total_scale_raw: 0.0,
            drag_total_scale_applied: 1.0,
            drag_total_scale_plane_raw: Vec2::ZERO,
            drag_total_scale_plane_applied: Vec2::ONE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoPhase {
    Begin,
    Update,
    Commit,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoResult {
    Translation {
        delta: Vec3,
        total: Vec3,
    },
    Rotation {
        axis: Vec3,
        delta_radians: f32,
        total_radians: f32,
    },
    Arcball {
        delta: Quat,
        total: Quat,
    },
    Scale {
        delta: Vec3,
        total: Vec3,
    },
}

#[derive(Debug, Clone)]
pub struct GizmoUpdate {
    pub phase: GizmoPhase,
    pub active: HandleId,
    pub result: GizmoResult,
    /// Updated target transforms for the current interaction state.
    ///
    /// During drags (`Begin`/`Update`), this is computed from the snapshot captured at drag start
    /// plus the current total delta, so hosts do not need to feed back intermediate transforms to
    /// keep motion stable.
    pub updated_targets: Vec<GizmoTarget3d>,
}

impl GizmoUpdate {
    /// Computes an ImGuizmo-style `deltaMatrix` for a target, using the host's start transform.
    ///
    /// The returned matrix satisfies `delta * start == end` in world space (for TRS matrices),
    /// and includes any pivot-induced translation compensation that happened in the gizmo update.
    pub fn delta_matrix_for(&self, start: &GizmoTarget3d) -> Option<Mat4> {
        let end = self.updated_targets.iter().find(|t| t.id == start.id)?;
        delta_matrix_trs(start.transform, end.transform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_matrix_for_matches_delta_matrix_trs() {
        let start = GizmoTarget3d {
            id: super::super::GizmoTargetId(1),
            transform: super::super::Transform3d {
                translation: Vec3::new(1.0, 2.0, 3.0),
                rotation: Quat::from_rotation_z(0.25),
                scale: Vec3::new(1.0, 2.0, 0.5),
            },
            local_bounds: None,
        };
        let end = GizmoTarget3d {
            id: start.id,
            transform: super::super::Transform3d {
                translation: Vec3::new(4.0, 1.0, 0.0),
                rotation: Quat::from_rotation_z(0.75),
                scale: Vec3::new(2.0, 1.0, 1.25),
            },
            local_bounds: None,
        };

        let update = GizmoUpdate {
            phase: GizmoPhase::Update,
            active: HandleId(123),
            result: GizmoResult::Translation {
                delta: Vec3::ZERO,
                total: Vec3::ZERO,
            },
            updated_targets: vec![end],
        };

        let a = update
            .delta_matrix_for(&start)
            .expect("expected delta matrix");
        let b = delta_matrix_trs(start.transform, update.updated_targets[0].transform)
            .expect("expected delta matrix");

        for (x, y) in a.to_cols_array().into_iter().zip(b.to_cols_array()) {
            let d = (x - y).abs();
            assert!(d < 1e-6, "expected same delta matrices, got |x-y|={d}");
        }
    }
}
