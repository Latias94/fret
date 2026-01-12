use fret_core::Color;
use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
    Universal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoOrientation {
    World,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoPivotMode {
    /// The gizmo is positioned at the active target's pivot.
    Active,
    /// The gizmo is positioned at the selection center.
    ///
    /// When `GizmoTarget3d::local_bounds` are available, this uses the selection world AABB center
    /// (editor convention). Otherwise it falls back to the average of target translations.
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoHandedness {
    /// Right-handed coordinate convention.
    RightHanded,
    /// Left-handed coordinate convention.
    LeftHanded,
}

impl Default for GizmoHandedness {
    fn default() -> Self {
        Self::RightHanded
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthMode {
    Test,
    /// Draws regardless of depth but should be rendered *before* `Test` so visible parts can
    /// overwrite the ghosted pass.
    Ghost,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleId(pub u64);

/// App-defined stable identity for targets controlled by a gizmo.
///
/// This is intentionally lightweight and does not imply an entity/component model. It allows
/// applications to:
/// - derive undo coalescing keys (ADR 0024),
/// - maintain stable selection across frames,
/// - map updated transforms back to domain objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GizmoTargetId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3d {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3 {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb3 {
    pub fn normalized(self) -> Self {
        let min = self.min.min(self.max);
        let max = self.min.max(self.max);
        Self { min, max }
    }

    pub fn corners(self) -> [Vec3; 8] {
        let a = self.normalized();
        let min = a.min;
        let max = a.max;
        [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
        ]
    }
}

impl Default for Transform3d {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform3d {
    pub fn to_mat4(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Attempts to decompose a TRS matrix into `Transform3d`.
    ///
    /// This is intended as a convenience for ImGuizmo-style integrations. The input is assumed
    /// to be a typical editor transform matrix (translation + rotation + scale). Matrices with
    /// skew/shear or a non-invertible basis may produce lossy results.
    pub fn try_from_mat4_trs(mat: Mat4) -> Option<Self> {
        let (scale, rotation, translation) = mat.to_scale_rotation_translation();
        if !scale.is_finite() || !rotation.is_finite() || !translation.is_finite() {
            return None;
        }
        Some(Self {
            translation,
            rotation,
            scale,
        })
    }
}

/// Computes a "delta matrix" that maps `start` to `end` in world space.
///
/// This is the common ImGuizmo-style `deltaMatrix` interpretation: `delta * start = end`.
/// Returns `None` if `start` is non-invertible.
pub fn delta_matrix_trs(start: Transform3d, end: Transform3d) -> Option<Mat4> {
    let start_mat = start.to_mat4();
    let det = start_mat.determinant();
    if !det.is_finite() || det.abs() <= 1e-8 {
        return None;
    }
    let delta = end.to_mat4() * start_mat.inverse();
    delta.is_finite().then_some(delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trs_decompose_round_trips_basic() {
        let t0 = Transform3d {
            translation: Vec3::new(1.25, -2.5, 3.0),
            rotation: Quat::from_euler(glam::EulerRot::YXZ, 0.3, -1.1, 0.7),
            scale: Vec3::new(1.2, 0.8, 2.0),
        };

        let m = t0.to_mat4();
        let t1 = Transform3d::try_from_mat4_trs(m).expect("expected TRS decomposition");
        let m1 = t1.to_mat4();

        for (a, b) in m.to_cols_array().into_iter().zip(m1.to_cols_array()) {
            let d = (a - b).abs();
            assert!(d < 1e-4, "expected round-trip matrix close, got |a-b|={d}");
        }
    }

    #[test]
    fn delta_matrix_maps_start_to_end() {
        let start = Transform3d {
            translation: Vec3::new(-4.0, 2.0, 1.0),
            rotation: Quat::from_rotation_y(0.7),
            scale: Vec3::new(1.0, 2.0, 0.5),
        };
        let end = Transform3d {
            translation: Vec3::new(-1.0, 1.0, 0.0),
            rotation: Quat::from_rotation_y(1.4),
            scale: Vec3::new(2.0, 1.5, 0.75),
        };

        let delta = delta_matrix_trs(start, end).expect("expected invertible start transform");
        let out = delta * start.to_mat4();
        let target = end.to_mat4();

        for (a, b) in out.to_cols_array().into_iter().zip(target.to_cols_array()) {
            let d = (a - b).abs();
            assert!(d < 1e-4, "expected delta*start == end, got |a-b|={d}");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoTarget3d {
    pub id: GizmoTargetId,
    pub transform: Transform3d,
    /// Optional local-space AABB (model-space bounds before TRS).
    ///
    /// When provided, bounds/box scaling uses these corners transformed by the target's TRS to
    /// compute the selection bounds (ImGuizmo `localBounds` equivalent surface).
    pub local_bounds: Option<Aabb3>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line3d {
    pub a: Vec3,
    pub b: Vec3,
    pub color: Color,
    pub depth: DepthMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle3d {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    pub color: Color,
    pub depth: DepthMode,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct GizmoDrawList3d {
    pub lines: Vec<Line3d>,
    pub triangles: Vec<Triangle3d>,
}
