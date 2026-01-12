use fret_core::Color;
use glam::{Mat4, Quat, Vec2, Vec3};

use crate::math::{
    DepthRange, Ray3d, ViewportRect, project_point, ray_from_screen, unproject_point,
};
use crate::picking::{PickCircle2d, PickConvexQuad2d, PickSegmentCapsule2d};
use crate::style::GizmoPartVisuals;

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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum GizmoSizePolicy {
    /// Keep the gizmo a constant size in screen pixels (Unity/Godot/Unreal-style default).
    #[default]
    ConstantPixels,
    /// Compute size from `size_px` (constant-pixels baseline), but clamp the resulting world size
    /// to a range derived from the selection bounds.
    ///
    /// This preserves the "constant screen size" feel in normal cases, while preventing extreme
    /// world sizes when the selection is very large/small or the camera is very near/far.
    PixelsClampedBySelectionBounds {
        /// Minimum allowed gizmo length as `selection_max_extent * min_fraction_of_max_extent`.
        min_fraction_of_max_extent: f32,
        /// Maximum allowed gizmo length as `selection_max_extent * max_fraction_of_max_extent`.
        max_fraction_of_max_extent: f32,
    },
    /// Size the gizmo as a fraction of the selection's world-space bounds.
    ///
    /// This produces a gizmo that "tracks" the selection scale and camera distance (it is not
    /// constant-size on screen), and is most useful as an editor option or for in-world tools.
    SelectionBounds {
        /// Multiplier applied to the selection bounds' maximum extent.
        fraction_of_max_extent: f32,
    },
}

/// Picking priority policy for multi-mode gizmos (Universal / operation masks).
///
/// This controls how hits from different sub-gizmos are resolved when their projections overlap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoPickPolicy {
    /// Maximum score treated as a "plane interior" hit for translate planes (handle ids 4/5/6).
    ///
    /// Note: this is expressed in the `PickHit.score` domain, not pixels.
    pub translate_plane_inside_score_max: f32,
    /// Maximum score treated as an "explicit solid" hit for scale handles (axis end boxes).
    ///
    /// Scale end boxes produce `score == 0.0` when the cursor is inside their projected quad.
    pub scale_solid_inside_score_max: f32,
    /// Maximum score treated as "inside" for bounds handles (corners/faces).
    pub bounds_inside_score_max: f32,
    /// Multiplier applied to `pick_radius_px` when detecting "translate axis tip intent" in
    /// Universal tools (bias translate near the arrow tip over rotate rings).
    pub translate_axis_tip_radius_scale: f32,
}

impl Default for GizmoPickPolicy {
    fn default() -> Self {
        Self {
            translate_plane_inside_score_max: 0.25,
            scale_solid_inside_score_max: 1e-6,
            bounds_inside_score_max: 0.25,
            translate_axis_tip_radius_scale: 0.90,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoConfig {
    pub mode: GizmoMode,
    /// Optional fine-grained operation mask (ImGuizmo `OPERATION` / transform-gizmo mode set).
    ///
    /// When `Some`, this overrides `mode` and the `show_*` toggles that imply optional sub-modes
    /// (`show_view_axis_ring`, `show_arcball`, `show_bounds`). The mask controls both drawing and
    /// picking for the enabled sub-operations.
    pub operation_mask: Option<GizmoOps>,
    pub orientation: GizmoOrientation,
    pub pivot_mode: GizmoPivotMode,
    /// Coordinate system handedness used to interpret rotation direction.
    ///
    /// Most picking math is purely geometric and works for either convention as long as the host
    /// provides a consistent `view_projection`. The primary behavioral difference is the sign of
    /// "positive rotation" around an axis, which is editor-facing and should match the host
    /// engine's convention.
    pub handedness: GizmoHandedness,
    pub depth_mode: DepthMode,
    pub depth_range: DepthRange,
    /// Determines how the gizmo's world-space size is derived.
    pub size_policy: GizmoSizePolicy,
    pub size_px: f32,
    /// Optional clamp for the world-space gizmo size derived from `size_px`.
    ///
    /// When set, the pixel-to-world mapping used by `size_px`-derived geometry (axis length,
    /// ring radii, etc) is clamped to the provided `[min, max]` range in world units. This is a
    /// stylistic control to avoid extreme world-space gizmo sizes at very near/far camera
    /// distances while preserving the default "constant screen size" behavior when unset.
    pub size_world_clamp: Option<(f32, f32)>,
    /// Picking priority policy used to resolve hit overlaps in multi-mode configurations.
    pub pick_policy: GizmoPickPolicy,
    pub pick_radius_px: f32,
    pub line_thickness_px: f32,
    pub drag_start_threshold_px: f32,
    pub translate_snap_step: Option<f32>,
    pub rotate_snap_step_radians: Option<f32>,
    pub scale_snap_step: Option<f32>,
    /// Optional per-axis snap step for bounds/box scaling extents (ImGuizmo `boundsSnap` surface).
    ///
    /// This differs from `scale_snap_step` (which snaps scale *factors*): bounds snapping is
    /// expressed in **selection box units** along the bounds basis, i.e. it snaps the resulting
    /// box size rather than the multiplicative scale delta.
    ///
    /// Only used when `show_bounds` is enabled and the current interaction is bounds scaling.
    pub bounds_snap_step: Option<Vec3>,
    /// When `true`, draw a faint always-on-top pass so occluded segments remain visible.
    pub show_occluded: bool,
    /// Alpha multiplier for the occluded always-on-top pass.
    pub occluded_alpha: f32,
    /// When `true`, includes a view-axis rotation ring (camera-facing) in `Rotate`/`Universal`.
    pub show_view_axis_ring: bool,
    /// Radius multiplier for the view-axis ring (outer ring).
    pub view_axis_ring_radius_scale: f32,
    /// Rotate ring visibility fade window in terms of `abs(dot(view_dir, axis_dir))`.
    ///
    /// - When `dot <= rotate_ring_fade_dot.0`, the axis ring is fully faded out (and not pickable).
    /// - When `dot >= rotate_ring_fade_dot.1`, the axis ring is fully visible.
    ///
    /// This reduces ring clutter and prevents edge-on rings from stealing interaction.
    pub rotate_ring_fade_dot: (f32, f32),
    /// When `true`, includes a free-rotation arcball (trackball) in `Rotate`/`Universal`.
    ///
    /// This is intended to match transform-gizmo's `Arcball` affordance: click/drag inside the
    /// arcball circle to perform unconstrained rotation.
    pub show_arcball: bool,
    /// Radius multiplier (relative to `size_px`) for the arcball circle.
    pub arcball_radius_scale: f32,
    /// Rotation speed multiplier for arcball drags.
    pub arcball_rotation_speed: f32,
    /// When `true`, draws a bounds (box) scaling gizmo in `GizmoMode::Scale`.
    ///
    /// This is an ImGuizmo `BOUNDS`-style affordance: a box around the selection with corner and
    /// face handles. Bounds are derived from `GizmoTarget3d::local_bounds` when provided; otherwise
    /// they fall back to selected target translations.
    pub show_bounds: bool,
    /// Bounds handle visual size in pixels.
    pub bounds_handle_size_px: f32,
    /// When `true`, `GizmoMode::Universal` includes scale interaction (axis scaling).
    ///
    /// Note: uniform scaling (handle id 7) remains exclusive to `GizmoMode::Scale` to avoid
    /// center-handle conflicts with view-plane translation.
    pub universal_includes_scale: bool,
    /// When `true`, `GizmoMode::Universal` includes the dolly (depth) translation handle.
    ///
    /// This is a Fret extension: the depth handle is a small ring around the center that moves
    /// along the camera view direction.
    pub universal_includes_translate_depth: bool,
    /// When `true` (default), axes may flip direction for better screen-space visibility
    /// (ImGuizmo `AllowAxisFlip` behavior).
    pub allow_axis_flip: bool,
    /// Axis visibility fade window in pixels.
    ///
    /// - When the projected axis length is <= `axis_fade_px.0`, the axis is fully faded out.
    /// - When the projected axis length is >= `axis_fade_px.1`, the axis is fully visible.
    ///
    /// This primarily fades axes that are almost aligned with the view direction.
    pub axis_fade_px: (f32, f32),
    /// Plane visibility fade window in px^2.
    ///
    /// - When the projected plane quad area is <= `plane_fade_px2.0`, the plane is fully faded out.
    /// - When the projected plane quad area is >= `plane_fade_px2.1`, the plane is fully visible.
    ///
    /// This primarily fades planes that are nearly edge-on in screen space.
    pub plane_fade_px2: (f32, f32),
    /// Axis mask (true -> hidden) for X/Y/Z (ImGuizmo `SetAxisMask`).
    ///
    /// Mask semantics for plane handles follows ImGuizmo: if exactly one axis is hidden, only the
    /// plane perpendicular to that axis is shown; if multiple axes are hidden, all planes are hidden.
    pub axis_mask: [bool; 3],
    pub x_color: Color,
    pub y_color: Color,
    pub z_color: Color,
    pub hover_color: Color,
}

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            mode: GizmoMode::Translate,
            operation_mask: None,
            orientation: GizmoOrientation::World,
            pivot_mode: GizmoPivotMode::Active,
            handedness: GizmoHandedness::default(),
            depth_mode: DepthMode::Test,
            depth_range: DepthRange::default(),
            size_policy: GizmoSizePolicy::ConstantPixels,
            size_px: 96.0,
            size_world_clamp: None,
            pick_policy: GizmoPickPolicy::default(),
            pick_radius_px: 10.0,
            line_thickness_px: 6.0,
            drag_start_threshold_px: 3.0,
            translate_snap_step: None,
            rotate_snap_step_radians: Some(15.0_f32.to_radians()),
            scale_snap_step: Some(0.1),
            bounds_snap_step: None,
            show_occluded: true,
            occluded_alpha: 0.25,
            show_view_axis_ring: true,
            view_axis_ring_radius_scale: 1.2,
            rotate_ring_fade_dot: (0.10, 0.30),
            show_arcball: true,
            arcball_radius_scale: 0.85,
            arcball_rotation_speed: 1.0,
            show_bounds: false,
            bounds_handle_size_px: 12.0,
            universal_includes_scale: true,
            universal_includes_translate_depth: false,
            allow_axis_flip: true,
            axis_fade_px: (4.0, 18.0),
            plane_fade_px2: (120.0, 520.0),
            axis_mask: [false; 3],
            x_color: Color {
                r: 1.0,
                g: 0.2,
                b: 0.4,
                a: 1.0,
            },
            y_color: Color {
                r: 0.2,
                g: 1.0,
                b: 0.4,
                a: 1.0,
            },
            z_color: Color {
                r: 0.2,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
            hover_color: Color {
                r: 1.0,
                g: 0.85,
                b: 0.2,
                a: 1.0,
            },
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GizmoOps(u32);

impl GizmoOps {
    const TRANSLATE_AXIS: u32 = 1 << 0;
    const TRANSLATE_PLANE: u32 = 1 << 1;
    const TRANSLATE_VIEW: u32 = 1 << 2;
    const TRANSLATE_DEPTH: u32 = 1 << 10;
    const ROTATE_AXIS: u32 = 1 << 3;
    const ROTATE_VIEW: u32 = 1 << 4;
    const ROTATE_ARCBALL: u32 = 1 << 5;
    const SCALE_AXIS: u32 = 1 << 6;
    const SCALE_PLANE: u32 = 1 << 7;
    const SCALE_UNIFORM: u32 = 1 << 8;
    const SCALE_BOUNDS: u32 = 1 << 9;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn translate_axis() -> Self {
        Self(Self::TRANSLATE_AXIS)
    }

    pub const fn translate_plane() -> Self {
        Self(Self::TRANSLATE_PLANE)
    }

    pub const fn translate_view() -> Self {
        Self(Self::TRANSLATE_VIEW)
    }

    pub const fn translate_depth() -> Self {
        Self(Self::TRANSLATE_DEPTH)
    }

    pub const fn rotate_axis() -> Self {
        Self(Self::ROTATE_AXIS)
    }

    pub const fn rotate_view() -> Self {
        Self(Self::ROTATE_VIEW)
    }

    pub const fn rotate_arcball() -> Self {
        Self(Self::ROTATE_ARCBALL)
    }

    pub const fn scale_axis() -> Self {
        Self(Self::SCALE_AXIS)
    }

    pub const fn scale_plane() -> Self {
        Self(Self::SCALE_PLANE)
    }

    pub const fn scale_uniform() -> Self {
        Self(Self::SCALE_UNIFORM)
    }

    pub const fn scale_bounds() -> Self {
        Self(Self::SCALE_BOUNDS)
    }

    pub const fn translate_all() -> Self {
        Self(
            Self::TRANSLATE_AXIS
                | Self::TRANSLATE_PLANE
                | Self::TRANSLATE_VIEW
                | Self::TRANSLATE_DEPTH,
        )
    }

    pub const fn rotate_all() -> Self {
        Self(Self::ROTATE_AXIS | Self::ROTATE_VIEW | Self::ROTATE_ARCBALL)
    }

    pub const fn scale_all() -> Self {
        Self(Self::SCALE_AXIS | Self::SCALE_PLANE | Self::SCALE_UNIFORM | Self::SCALE_BOUNDS)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for GizmoOps {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for GizmoOps {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for GizmoOps {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl GizmoConfig {
    /// Returns a more editor-friendly baseline configuration.
    ///
    /// `Default` stays conservative; this opt-in preset biases toward "easy to hit" and
    /// "readable at a glance" in typical game-editor viewports.
    pub fn editor_default() -> Self {
        let mut cfg = Self::default();
        cfg.size_px = 112.0;
        cfg.pick_radius_px = 12.0;
        cfg.line_thickness_px = 8.0;
        cfg.bounds_handle_size_px = 14.0;
        cfg
    }

    /// Scales pixel-based knobs when the host's cursor/viewport units are logical "points".
    ///
    /// Pass the platform scale factor (e.g. pixels-per-point / device pixel ratio) so gizmo
    /// visuals and picking remain consistent across DPI settings.
    pub fn scale_for_pixels_per_point(mut self, pixels_per_point: f32) -> Self {
        let s = pixels_per_point.clamp(0.1, 16.0);
        self.size_px *= s;
        self.pick_radius_px *= s;
        self.line_thickness_px *= s;
        self.drag_start_threshold_px *= s;
        self.bounds_handle_size_px *= s;
        self
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TranslateHandle {
    AxisX,
    AxisY,
    AxisZ,
    PlaneXY,
    PlaneXZ,
    PlaneYZ,
    Screen,
    Depth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundsHandle {
    Corner {
        x_max: bool,
        y_max: bool,
        z_max: bool,
    },
    Face {
        axis: usize,
        max_side: bool,
    },
}

impl TranslateHandle {
    fn id(self) -> HandleId {
        HandleId(match self {
            TranslateHandle::AxisX => 1,
            TranslateHandle::AxisY => 2,
            TranslateHandle::AxisZ => 3,
            TranslateHandle::PlaneXY => 4,
            TranslateHandle::PlaneXZ => 5,
            TranslateHandle::PlaneYZ => 6,
            TranslateHandle::Screen => 10,
            TranslateHandle::Depth => 11,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScaleHandle {
    AxisX,
    AxisY,
    AxisZ,
    PlaneXY,
    PlaneXZ,
    PlaneYZ,
    Uniform,
}

impl ScaleHandle {
    fn id(self) -> HandleId {
        HandleId(match self {
            ScaleHandle::AxisX => 1,
            ScaleHandle::AxisY => 2,
            ScaleHandle::AxisZ => 3,
            ScaleHandle::Uniform => 7,
            // Keep plane-scale handle IDs disjoint from translation plane IDs (4/5/6) so Universal
            // can include axis scale without fighting translate planes.
            ScaleHandle::PlaneXY => 14,
            ScaleHandle::PlaneXZ => 15,
            ScaleHandle::PlaneYZ => 16,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TranslateConstraint {
    Axis { axis_dir: Vec3 },
    Plane { u: Vec3, v: Vec3, normal: Vec3 },
    Dolly { view_dir: Vec3 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PickHit {
    handle: HandleId,
    score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MixedPickBand {
    TranslateCenter = 0,
    TranslatePlaneInside = 1,
    ScaleSolidInside = 2,
    TranslateAxisTipIntent = 3,
    Default = 4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GizmoState {
    pub hovered: Option<HandleId>,
    pub active: Option<HandleId>,
    pub hovered_kind: Option<GizmoMode>,
    part_visuals: GizmoPartVisuals,
    drag_start_targets: Vec<GizmoTarget3d>,
    drag_mode: GizmoMode,
    drag_snap: bool,
    drag_has_started: bool,
    drag_start_cursor_px: Vec2,
    drag_axis_dir: Vec3,
    drag_origin: Vec3,
    drag_origin_z01: f32,
    drag_size_length_world: f32,
    drag_plane_normal: Vec3,
    drag_start_hit_world: Vec3,
    drag_prev_hit_world: Vec3,
    drag_translate_prev_axis_raw: f32,
    drag_translate_prev_plane_raw: Vec2,
    drag_scale_prev_axis_raw: f32,
    drag_scale_prev_plane_raw: Vec2,
    drag_bounds_prev_local_raw: Vec3,
    drag_total_axis_raw: f32,
    drag_total_axis_applied: f32,
    drag_translate_is_plane: bool,
    drag_translate_is_dolly: bool,
    drag_translate_dolly_world_per_px: f32,
    drag_translate_u: Vec3,
    drag_translate_v: Vec3,
    drag_total_plane_raw: Vec2,
    drag_total_plane_applied: Vec2,
    drag_basis_u: Vec3,
    drag_basis_v: Vec3,
    drag_start_angle: f32,
    drag_prev_angle: f32,
    drag_total_angle_raw: f32,
    drag_total_angle_applied: f32,
    drag_rotate_is_arcball: bool,
    drag_arcball_center_px: Vec2,
    drag_arcball_radius_px: f32,
    drag_arcball_prev_vec: Vec3,
    drag_total_arcball_raw: Quat,
    drag_total_arcball_applied: Quat,
    drag_scale_is_bounds: bool,
    drag_bounds_basis: [Vec3; 3],
    drag_bounds_min_local: Vec3,
    drag_bounds_max_local: Vec3,
    drag_bounds_anchor_local: Vec3,
    drag_bounds_axes_mask: [bool; 3],
    drag_bounds_axis_sign: [f32; 3],
    drag_bounds_start_extent: Vec3,
    drag_bounds_total_raw: Vec3,
    drag_bounds_total_applied: Vec3,
    drag_scale_axis: Option<usize>,
    drag_scale_plane_axes: Option<(usize, usize)>,
    drag_scale_plane_u: Vec3,
    drag_scale_plane_v: Vec3,
    drag_scale_is_uniform: bool,
    drag_total_scale_raw: f32,
    drag_total_scale_applied: f32,
    drag_total_scale_plane_raw: Vec2,
    drag_total_scale_plane_applied: Vec2,
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

#[derive(Debug, Default)]
pub struct Gizmo {
    pub config: GizmoConfig,
    pub state: GizmoState,
}

impl Gizmo {
    const UNIVERSAL_TRANSLATE_TIP_SCALE: f32 = 1.25;
    const ROTATE_VIEW_HANDLE: HandleId = HandleId(8);
    const ROTATE_ARCBALL_HANDLE: HandleId = HandleId(9);
    const BOUNDS_CORNER_BASE: u64 = 20;
    const BOUNDS_CORNER_END: u64 = 27;
    const BOUNDS_FACE_BASE: u64 = 30;
    const BOUNDS_FACE_END: u64 = 35;

    fn effective_ops(&self) -> GizmoOps {
        if let Some(mask) = self.config.operation_mask {
            return mask;
        }

        match self.config.mode {
            GizmoMode::Translate => GizmoOps::translate_all(),
            GizmoMode::Rotate => {
                let mut ops = GizmoOps::rotate_axis();
                if self.config.show_view_axis_ring {
                    ops |= GizmoOps::rotate_view();
                }
                if self.config.show_arcball {
                    ops |= GizmoOps::rotate_arcball();
                }
                ops
            }
            GizmoMode::Scale => {
                let mut ops =
                    GizmoOps::scale_axis() | GizmoOps::scale_plane() | GizmoOps::scale_uniform();
                if self.config.show_bounds {
                    ops |= GizmoOps::scale_bounds();
                }
                ops
            }
            GizmoMode::Universal => {
                let mut ops = GizmoOps::translate_all() | GizmoOps::rotate_axis();
                if self.config.show_view_axis_ring {
                    ops |= GizmoOps::rotate_view();
                }
                if self.config.show_arcball {
                    ops |= GizmoOps::rotate_arcball();
                }
                if self.config.universal_includes_scale {
                    ops |= GizmoOps::scale_axis();
                }
                ops
            }
        }
    }

    fn translate_axis_tip_scale(&self) -> f32 {
        if self.config.mode == GizmoMode::Universal && self.config.universal_includes_scale {
            return Self::UNIVERSAL_TRANSLATE_TIP_SCALE;
        }
        if let Some(mask) = self.config.operation_mask {
            let translate_axis = mask.contains(GizmoOps::translate_axis());
            let scale_axis = mask.contains(GizmoOps::scale_axis());
            if translate_axis && scale_axis {
                return Self::UNIVERSAL_TRANSLATE_TIP_SCALE;
            }
        }
        1.0
    }

    fn handedness_rotation_sign(&self) -> f32 {
        match self.config.handedness {
            GizmoHandedness::RightHanded => 1.0,
            GizmoHandedness::LeftHanded => -1.0,
        }
    }

    fn pivot_origin(
        active_transform: Transform3d,
        targets: &[GizmoTarget3d],
        mode: GizmoPivotMode,
    ) -> Vec3 {
        match mode {
            GizmoPivotMode::Active => active_transform.translation,
            GizmoPivotMode::Center => {
                // Editor convention: "center" means the selection bounds center, not the average
                // of entity origins (which can drift for uneven distributions).
                if let Some(bounds) = Self::selection_world_aabb(targets) {
                    (bounds.min + bounds.max) * 0.5
                } else {
                    let sum = targets
                        .iter()
                        .fold(Vec3::ZERO, |acc, t| acc + t.transform.translation);
                    sum / (targets.len().max(1) as f32)
                }
            }
        }
    }

    fn selection_world_aabb(targets: &[GizmoTarget3d]) -> Option<Aabb3> {
        let mut min_v = Vec3::splat(f32::INFINITY);
        let mut max_v = Vec3::splat(f32::NEG_INFINITY);

        for t in targets {
            if let Some(aabb) = t.local_bounds {
                let aabb = aabb.normalized();
                let m = t.transform.to_mat4();
                for c in aabb.corners() {
                    let world = m.transform_point3(c);
                    if !world.is_finite() {
                        continue;
                    }
                    min_v = min_v.min(world);
                    max_v = max_v.max(world);
                }
            } else {
                let world = t.transform.translation;
                if !world.is_finite() {
                    continue;
                }
                min_v = min_v.min(world);
                max_v = max_v.max(world);
            }
        }

        if !min_v.is_finite() || !max_v.is_finite() {
            return None;
        }

        Some(
            Aabb3 {
                min: min_v,
                max: max_v,
            }
            .normalized(),
        )
    }

    fn clamp_size_length_world(&self, length_world: f32) -> f32 {
        let length_world = if length_world.is_finite() {
            length_world.max(0.0)
        } else {
            0.0
        };
        let Some((a, b)) = self.config.size_world_clamp else {
            return length_world;
        };
        if !a.is_finite() || !b.is_finite() {
            return length_world;
        }
        let (min_world, max_world) = if a <= b { (a, b) } else { (b, a) };
        length_world.clamp(min_world.max(0.0), max_world.max(0.0))
    }

    fn size_length_world(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        targets: &[GizmoTarget3d],
    ) -> Option<f32> {
        let length_world = match self.config.size_policy {
            GizmoSizePolicy::ConstantPixels => axis_length_world(
                view_projection,
                viewport,
                origin,
                self.config.depth_range,
                self.config.size_px,
            )?,
            GizmoSizePolicy::PixelsClampedBySelectionBounds {
                min_fraction_of_max_extent,
                max_fraction_of_max_extent,
            } => {
                let base = axis_length_world(
                    view_projection,
                    viewport,
                    origin,
                    self.config.depth_range,
                    self.config.size_px,
                )?;
                match Self::selection_world_aabb(targets) {
                    None => base,
                    Some(bounds) => {
                        let extent = (bounds.max - bounds.min).abs();
                        let max_extent = extent.max_element().max(1e-6);

                        let min_frac = if min_fraction_of_max_extent.is_finite() {
                            min_fraction_of_max_extent.clamp(0.0, 1000.0)
                        } else {
                            0.0
                        };
                        let max_frac = if max_fraction_of_max_extent.is_finite() {
                            max_fraction_of_max_extent.clamp(0.0, 1000.0)
                        } else {
                            0.0
                        };
                        let (min_frac, max_frac) = if min_frac <= max_frac {
                            (min_frac, max_frac)
                        } else {
                            (max_frac, min_frac)
                        };

                        let min_world = max_extent * min_frac;
                        let max_world = max_extent * max_frac;
                        if max_world <= 1e-6 {
                            base
                        } else {
                            base.clamp(min_world.max(0.0), max_world.max(min_world))
                        }
                    }
                }
            }
            GizmoSizePolicy::SelectionBounds {
                fraction_of_max_extent,
            } => {
                let fraction = if fraction_of_max_extent.is_finite() {
                    fraction_of_max_extent.clamp(0.01, 100.0)
                } else {
                    1.0
                };

                let bounds = Self::selection_world_aabb(targets);
                let len = bounds.map(|b| {
                    let extent = (b.max - b.min).abs();
                    extent.max_element().max(1e-6) * fraction
                });
                match len {
                    Some(v) if v.is_finite() && v > 1e-6 => v,
                    _ => axis_length_world(
                        view_projection,
                        viewport,
                        origin,
                        self.config.depth_range,
                        self.config.size_px,
                    )?,
                }
            }
        };

        Some(self.clamp_size_length_world(length_world))
    }

    fn size_length_world_or_one(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        targets: &[GizmoTarget3d],
    ) -> f32 {
        self.size_length_world(view_projection, viewport, origin, targets)
            .unwrap_or(1.0)
    }

    fn axis_is_masked(&self, axis_index: usize) -> bool {
        self.config
            .axis_mask
            .get(axis_index)
            .copied()
            .unwrap_or(false)
    }

    fn plane_allowed_by_mask(&self, plane_axes: (usize, usize)) -> bool {
        let (a, b) = plane_axes;
        if a == b || a > 2 || b > 2 {
            return false;
        }
        let masked = self.config.axis_mask;
        let masked_count = masked.iter().filter(|m| **m).count();
        if masked_count == 0 {
            return true;
        }
        if masked_count == 1 {
            // Show only the plane perpendicular to the masked axis.
            let perp = 3usize.saturating_sub(a + b);
            return perp <= 2 && masked[perp];
        }
        false
    }

    fn flip_axes_for_view(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> [Vec3; 3] {
        if !self.config.allow_axis_flip {
            return axes;
        }
        let length_world = size_length_world.max(0.0);

        let mut out = axes;
        for i in 0..3 {
            let axis = axes[i].normalize_or_zero();
            if axis.length_squared() == 0.0 {
                continue;
            }

            let len_plus = axis_segment_len_px(
                view_projection,
                viewport,
                origin,
                self.config.depth_range,
                axis,
                length_world,
            );
            let len_minus = axis_segment_len_px(
                view_projection,
                viewport,
                origin,
                self.config.depth_range,
                -axis,
                length_world,
            );

            out[i] = match (len_plus, len_minus) {
                (Some(a), Some(b)) => {
                    if b > a + 1e-3 {
                        -axis
                    } else {
                        axis
                    }
                }
                (None, Some(_)) => -axis,
                _ => axis,
            };
        }
        out
    }

    fn axis_visibility_alpha(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axis_dir: Vec3,
        axis_len_world: f32,
    ) -> f32 {
        let (lo, hi) = self.config.axis_fade_px;
        if !(lo.is_finite() && hi.is_finite()) {
            return 1.0;
        }
        let lo = lo.min(hi);
        let hi = hi.max(lo + 1e-3);
        let len_px = axis_segment_len_px(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            axis_dir,
            axis_len_world,
        )
        .unwrap_or(hi);
        ((len_px - lo) / (hi - lo)).clamp(0.0, 1.0)
    }

    fn rotate_ring_visibility_alpha(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axis_dir: Vec3,
    ) -> f32 {
        let (lo, hi) = self.config.rotate_ring_fade_dot;
        if !(lo.is_finite() && hi.is_finite()) {
            return 1.0;
        }
        let lo = lo.min(hi);
        let hi = hi.max(lo + 1e-3);

        let Some(view_dir) =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        else {
            return 1.0;
        };
        let axis = axis_dir.normalize_or_zero();
        let view = view_dir.normalize_or_zero();
        if axis.length_squared() == 0.0 || view.length_squared() == 0.0 {
            return 1.0;
        }

        let dot = view.dot(axis).abs().clamp(0.0, 1.0);
        let t = ((dot - lo) / (hi - lo)).clamp(0.0, 1.0);
        // smoothstep
        t * t * (3.0 - 2.0 * t)
    }

    fn plane_visibility_alpha(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        quad_world: [Vec3; 4],
    ) -> f32 {
        let (lo, hi) = self.config.plane_fade_px2;
        if !(lo.is_finite() && hi.is_finite()) {
            return 1.0;
        }
        let lo = lo.min(hi);
        let hi = hi.max(lo + 1e-3);
        let p = project_quad(
            view_projection,
            viewport,
            quad_world,
            self.config.depth_range,
        );
        let area = p.map(quad_area_px2).unwrap_or(hi);
        ((area - lo) / (hi - lo)).clamp(0.0, 1.0)
    }

    pub fn new(config: GizmoConfig) -> Self {
        Self {
            config,
            state: GizmoState::default(),
        }
    }

    pub fn set_part_visuals(&mut self, visuals: GizmoPartVisuals) {
        self.state.part_visuals = visuals;
    }

    pub fn part_visuals(&self) -> GizmoPartVisuals {
        self.state.part_visuals
    }

    pub fn update(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> Option<GizmoUpdate> {
        if targets.is_empty() {
            self.state.hovered = None;
            self.state.active = None;
            return None;
        }

        let active_index = targets
            .iter()
            .position(|t| t.id == active_target)
            .unwrap_or(0);
        let active_transform = targets
            .get(active_index)
            .map(|t| t.transform)
            .unwrap_or_else(|| targets[0].transform);

        let origin = Self::pivot_origin(active_transform, targets, self.config.pivot_mode);
        let Some(cursor_ray) = ray_from_screen(
            view_projection,
            viewport,
            input.cursor_px,
            self.config.depth_range,
        ) else {
            return None;
        };

        let size_length_world =
            self.size_length_world_or_one(view_projection, viewport, origin, targets);

        let axes_raw = self.axis_dirs(&active_transform);
        let axes = self.flip_axes_for_view(
            view_projection,
            viewport,
            origin,
            axes_raw,
            size_length_world,
        );
        let mut hovered: Option<HandleId> = None;
        let mut hovered_kind: Option<GizmoMode> = None;
        if self.state.active.is_none() && input.hovered {
            let pick = if self.config.operation_mask.is_some() {
                self.pick_operation_mask_handle(
                    view_projection,
                    viewport,
                    origin,
                    input.cursor_px,
                    axes,
                    axes_raw,
                    size_length_world,
                    targets,
                )
            } else {
                match self.config.mode {
                    GizmoMode::Translate => self
                        .pick_translate_handle(
                            view_projection,
                            viewport,
                            origin,
                            input.cursor_px,
                            axes,
                            size_length_world,
                            true,
                            true,
                            true,
                            true,
                        )
                        .map(|h| (h, GizmoMode::Translate)),
                    GizmoMode::Rotate => self
                        .pick_rotate_axis(
                            view_projection,
                            viewport,
                            origin,
                            input.cursor_px,
                            axes,
                            size_length_world,
                        )
                        .map(|h| (h, GizmoMode::Rotate)),
                    GizmoMode::Scale => self
                        .pick_scale_or_bounds_handle(
                            view_projection,
                            viewport,
                            origin,
                            input.cursor_px,
                            axes,
                            axes_raw,
                            size_length_world,
                            targets,
                        )
                        .map(|h| (h, GizmoMode::Scale)),
                    GizmoMode::Universal => self.pick_universal_handle(
                        view_projection,
                        viewport,
                        origin,
                        input.cursor_px,
                        axes,
                        size_length_world,
                    ),
                }
            };

            if let Some((h, kind)) = pick {
                hovered = Some(h.handle);
                hovered_kind = Some(kind);
            }
        }
        self.state.hovered = hovered;
        self.state.hovered_kind = hovered_kind;

        if self.state.active.is_none() {
            if input.drag_started {
                if let Some(h) = hovered {
                    let begin = if self.config.operation_mask.is_some() {
                        match self.state.hovered_kind {
                            Some(GizmoMode::Translate) => self.begin_translate_drag(
                                view_projection,
                                viewport,
                                input,
                                targets,
                                cursor_ray,
                                origin,
                                h,
                                axes,
                            ),
                            Some(GizmoMode::Rotate) => self.begin_rotate_drag(
                                view_projection,
                                viewport,
                                input,
                                targets,
                                cursor_ray,
                                origin,
                                h,
                                axes,
                                size_length_world,
                            ),
                            Some(GizmoMode::Scale) => {
                                if let Some(bounds_handle) = Self::bounds_handle_from_id(h) {
                                    let origin_z01 = origin_z01(
                                        view_projection,
                                        viewport,
                                        origin,
                                        self.config.depth_range,
                                    )?;
                                    self.begin_bounds_drag(
                                        view_projection,
                                        viewport,
                                        input,
                                        targets,
                                        cursor_ray,
                                        origin,
                                        origin_z01,
                                        size_length_world,
                                        bounds_handle,
                                        h,
                                        axes_raw,
                                    )
                                } else {
                                    self.begin_scale_drag(
                                        view_projection,
                                        viewport,
                                        input,
                                        targets,
                                        cursor_ray,
                                        origin,
                                        h,
                                        axes,
                                        size_length_world,
                                    )
                                }
                            }
                            _ => None,
                        }
                    } else {
                        match self.config.mode {
                            GizmoMode::Translate => self.begin_translate_drag(
                                view_projection,
                                viewport,
                                input,
                                targets,
                                cursor_ray,
                                origin,
                                h,
                                axes,
                            ),
                            GizmoMode::Rotate => self.begin_rotate_drag(
                                view_projection,
                                viewport,
                                input,
                                targets,
                                cursor_ray,
                                origin,
                                h,
                                axes,
                                size_length_world,
                            ),
                            GizmoMode::Scale => {
                                if self.config.show_bounds {
                                    if let Some(bounds_handle) = Self::bounds_handle_from_id(h) {
                                        let origin_z01 = origin_z01(
                                            view_projection,
                                            viewport,
                                            origin,
                                            self.config.depth_range,
                                        )?;
                                        self.begin_bounds_drag(
                                            view_projection,
                                            viewport,
                                            input,
                                            targets,
                                            cursor_ray,
                                            origin,
                                            origin_z01,
                                            size_length_world,
                                            bounds_handle,
                                            h,
                                            axes_raw,
                                        )
                                    } else {
                                        self.begin_scale_drag(
                                            view_projection,
                                            viewport,
                                            input,
                                            targets,
                                            cursor_ray,
                                            origin,
                                            h,
                                            axes,
                                            size_length_world,
                                        )
                                    }
                                } else {
                                    self.begin_scale_drag(
                                        view_projection,
                                        viewport,
                                        input,
                                        targets,
                                        cursor_ray,
                                        origin,
                                        h,
                                        axes,
                                        size_length_world,
                                    )
                                }
                            }
                            GizmoMode::Universal => match self.state.hovered_kind {
                                Some(GizmoMode::Translate) => self.begin_translate_drag(
                                    view_projection,
                                    viewport,
                                    input,
                                    targets,
                                    cursor_ray,
                                    origin,
                                    h,
                                    axes,
                                ),
                                Some(GizmoMode::Rotate) => self.begin_rotate_drag(
                                    view_projection,
                                    viewport,
                                    input,
                                    targets,
                                    cursor_ray,
                                    origin,
                                    h,
                                    axes,
                                    size_length_world,
                                ),
                                Some(GizmoMode::Scale) => self.begin_scale_drag(
                                    view_projection,
                                    viewport,
                                    input,
                                    targets,
                                    cursor_ray,
                                    origin,
                                    h,
                                    axes,
                                    size_length_world,
                                ),
                                _ => None,
                            },
                        }
                    };

                    // If a drag threshold is configured, we arm the interaction on pointer down
                    // but only emit the `Begin` phase once the pointer has actually moved.
                    if self.config.drag_start_threshold_px > 0.0 {
                        return None;
                    }
                    return begin;
                }
            }
            return None;
        }

        let active = self.state.active.unwrap();
        let precision = input_precision(input.precision);

        match self.state.drag_mode {
            GizmoMode::Translate => {
                let axis_dir = self.state.drag_axis_dir;

                if input.cancel {
                    let total = if self.state.drag_translate_is_plane {
                        self.state.drag_translate_u * self.state.drag_total_plane_applied.x
                            + self.state.drag_translate_v * self.state.drag_total_plane_applied.y
                    } else {
                        self.state.drag_total_axis_applied * axis_dir
                    };
                    self.state.active = None;
                    self.state.drag_start_targets.clear();
                    return Some(GizmoUpdate {
                        phase: GizmoPhase::Cancel,
                        active,
                        result: GizmoResult::Translation {
                            delta: Vec3::ZERO,
                            total,
                        },
                        updated_targets: targets.to_vec(),
                    });
                }

                if input.dragging {
                    self.state.drag_snap = input.snap;
                    let started_this_call = if !self.state.drag_has_started {
                        let threshold = self.config.drag_start_threshold_px.max(0.0);
                        if threshold > 0.0
                            && (input.cursor_px - self.state.drag_start_cursor_px).length()
                                < threshold
                        {
                            return None;
                        }
                        self.state.drag_has_started = true;
                        true
                    } else {
                        false
                    };
                    let (delta, total) = if self.state.drag_translate_is_dolly {
                        // Dolly (depth) translation is screen-delta driven rather than ray-plane driven:
                        // translating along the view direction has no stable plane intersection anchor.
                        //
                        // Invariant: returning the cursor to `drag_start_cursor_px` returns total close to 0.
                        let dy = input.cursor_px.y - self.state.drag_start_cursor_px.y;
                        if !dy.is_finite() {
                            return None;
                        }
                        self.state.drag_total_axis_raw =
                            dy * self.state.drag_translate_dolly_world_per_px * precision;
                        let desired_total = if input.snap {
                            self.config
                                .translate_snap_step
                                .filter(|s| s.is_finite() && *s > 0.0)
                                .map(|step| (self.state.drag_total_axis_raw / step).round() * step)
                                .unwrap_or(self.state.drag_total_axis_raw)
                        } else {
                            self.state.drag_total_axis_raw
                        };
                        let delta_axis = desired_total - self.state.drag_total_axis_applied;
                        self.state.drag_total_axis_applied = desired_total;
                        (delta_axis * axis_dir, desired_total * axis_dir)
                    } else {
                        let hit_world = ray_plane_intersect(
                            cursor_ray,
                            self.state.drag_origin,
                            self.state.drag_plane_normal,
                        )
                        .filter(|p| p.is_finite())
                        .unwrap_or_else(|| {
                            unproject_point(
                                view_projection,
                                viewport,
                                input.cursor_px,
                                self.config.depth_range,
                                self.state.drag_origin_z01,
                            )
                            .unwrap_or(self.state.drag_origin)
                        });

                        let diff_world = hit_world - self.state.drag_start_hit_world;
                        self.state.drag_prev_hit_world = hit_world;

                        if self.state.drag_translate_is_plane {
                            let u = self.state.drag_translate_u;
                            let v = self.state.drag_translate_v;
                            let raw = Vec2::new(diff_world.dot(u), diff_world.dot(v));
                            let delta_raw = raw - self.state.drag_translate_prev_plane_raw;
                            self.state.drag_translate_prev_plane_raw = raw;
                            self.state.drag_total_plane_raw += delta_raw * precision;
                            let desired_total = if input.snap {
                                self.config
                                    .translate_snap_step
                                    .filter(|s| s.is_finite() && *s > 0.0)
                                    .map(|step| {
                                        Vec2::new(
                                            (self.state.drag_total_plane_raw.x / step).round()
                                                * step,
                                            (self.state.drag_total_plane_raw.y / step).round()
                                                * step,
                                        )
                                    })
                                    .unwrap_or(self.state.drag_total_plane_raw)
                            } else {
                                self.state.drag_total_plane_raw
                            };
                            let delta_plane = desired_total - self.state.drag_total_plane_applied;
                            self.state.drag_total_plane_applied = desired_total;
                            let delta = u * delta_plane.x + v * delta_plane.y;
                            let total = u * desired_total.x + v * desired_total.y;
                            (delta, total)
                        } else {
                            let raw = diff_world.dot(axis_dir);
                            let delta_raw = raw - self.state.drag_translate_prev_axis_raw;
                            self.state.drag_translate_prev_axis_raw = raw;
                            self.state.drag_total_axis_raw += delta_raw * precision;
                            let desired_total = if input.snap {
                                self.config
                                    .translate_snap_step
                                    .filter(|s| s.is_finite() && *s > 0.0)
                                    .map(|step| {
                                        (self.state.drag_total_axis_raw / step).round() * step
                                    })
                                    .unwrap_or(self.state.drag_total_axis_raw)
                            } else {
                                self.state.drag_total_axis_raw
                            };
                            let delta_axis = desired_total - self.state.drag_total_axis_applied;
                            self.state.drag_total_axis_applied = desired_total;
                            (delta_axis * axis_dir, desired_total * axis_dir)
                        }
                    };
                    let updated_targets = self
                        .state
                        .drag_start_targets
                        .iter()
                        .map(|t| GizmoTarget3d {
                            id: t.id,
                            transform: Transform3d {
                                translation: t.transform.translation + total,
                                ..t.transform
                            },
                            local_bounds: t.local_bounds,
                        })
                        .collect::<Vec<_>>();
                    return Some(GizmoUpdate {
                        phase: if started_this_call {
                            GizmoPhase::Begin
                        } else {
                            GizmoPhase::Update
                        },
                        active,
                        result: GizmoResult::Translation { delta, total },
                        updated_targets,
                    });
                }

                if !self.state.drag_has_started {
                    self.state.active = None;
                    self.state.drag_start_targets.clear();
                    return None;
                }

                // Pointer released: end the interaction. The host is responsible for undo/redo boundaries.
                let total = if self.state.drag_translate_is_plane {
                    self.state.drag_translate_u * self.state.drag_total_plane_applied.x
                        + self.state.drag_translate_v * self.state.drag_total_plane_applied.y
                } else {
                    self.state.drag_total_axis_applied * axis_dir
                };
                self.state.active = None;
                self.state.drag_start_targets.clear();
                Some(GizmoUpdate {
                    phase: GizmoPhase::Commit,
                    active,
                    result: GizmoResult::Translation {
                        delta: Vec3::ZERO,
                        total,
                    },
                    updated_targets: targets.to_vec(),
                })
            }
            GizmoMode::Rotate => {
                if self.state.drag_rotate_is_arcball {
                    let total = self.state.drag_total_arcball_applied;
                    if input.cancel {
                        self.state.active = None;
                        self.state.drag_rotate_is_arcball = false;
                        self.state.drag_start_targets.clear();
                        return Some(GizmoUpdate {
                            phase: GizmoPhase::Cancel,
                            active,
                            result: GizmoResult::Arcball {
                                delta: Quat::IDENTITY,
                                total,
                            },
                            updated_targets: targets.to_vec(),
                        });
                    }

                    if input.dragging {
                        self.state.drag_snap = input.snap;
                        let started_this_call = if !self.state.drag_has_started {
                            let threshold = self.config.drag_start_threshold_px.max(0.0);
                            if threshold > 0.0
                                && (input.cursor_px - self.state.drag_start_cursor_px).length()
                                    < threshold
                            {
                                return None;
                            }
                            self.state.drag_has_started = true;
                            true
                        } else {
                            false
                        };

                        let current = self.arcball_vector_world(input.cursor_px)?;
                        let prev = self.state.drag_arcball_prev_vec.normalize_or_zero();
                        self.state.drag_arcball_prev_vec = current;
                        if prev.length_squared() == 0.0 {
                            return None;
                        }

                        let mut delta_q = Quat::from_rotation_arc(prev, current);
                        let mut angle_scale = precision;
                        if let Some(speed) = self
                            .config
                            .arcball_rotation_speed
                            .is_finite()
                            .then_some(self.config.arcball_rotation_speed)
                            .filter(|s| *s > 0.0 && (*s - 1.0).abs() > 1e-3)
                        {
                            angle_scale *= speed;
                        }
                        if (angle_scale - 1.0).abs() > 1e-3 {
                            let (axis, angle) = quat_axis_angle(delta_q);
                            delta_q = Quat::from_axis_angle(axis, angle * angle_scale);
                        }

                        self.state.drag_total_arcball_raw =
                            (delta_q * self.state.drag_total_arcball_raw).normalize();

                        let desired_total = if input.snap {
                            self.config
                                .rotate_snap_step_radians
                                .filter(|s| s.is_finite() && *s > 0.0)
                                .map(|step| {
                                    snap_quat_to_angle_step(self.state.drag_total_arcball_raw, step)
                                })
                                .unwrap_or(self.state.drag_total_arcball_raw)
                        } else {
                            self.state.drag_total_arcball_raw
                        };

                        let delta_apply = (desired_total
                            * self.state.drag_total_arcball_applied.inverse())
                        .normalize();
                        self.state.drag_total_arcball_applied = desired_total;

                        let updated_targets = self
                            .state
                            .drag_start_targets
                            .iter()
                            .map(|t| GizmoTarget3d {
                                id: t.id,
                                transform: Transform3d {
                                    translation: self.state.drag_origin
                                        + desired_total
                                            * (t.transform.translation - self.state.drag_origin),
                                    rotation: (desired_total * t.transform.rotation).normalize(),
                                    ..t.transform
                                },
                                local_bounds: t.local_bounds,
                            })
                            .collect::<Vec<_>>();
                        return Some(GizmoUpdate {
                            phase: if started_this_call {
                                GizmoPhase::Begin
                            } else {
                                GizmoPhase::Update
                            },
                            active,
                            result: GizmoResult::Arcball {
                                delta: delta_apply,
                                total: desired_total,
                            },
                            updated_targets,
                        });
                    }

                    if !self.state.drag_has_started {
                        self.state.active = None;
                        self.state.drag_rotate_is_arcball = false;
                        self.state.drag_start_targets.clear();
                        return None;
                    }

                    self.state.active = None;
                    self.state.drag_rotate_is_arcball = false;
                    self.state.drag_start_targets.clear();
                    Some(GizmoUpdate {
                        phase: GizmoPhase::Commit,
                        active,
                        result: GizmoResult::Arcball {
                            delta: Quat::IDENTITY,
                            total,
                        },
                        updated_targets: targets.to_vec(),
                    })
                } else {
                    let axis_dir = self.state.drag_axis_dir.normalize_or_zero();
                    if axis_dir.length_squared() == 0.0 {
                        self.state.active = None;
                        self.state.drag_start_targets.clear();
                        return None;
                    }

                    if input.cancel {
                        let total = self.state.drag_total_angle_applied;
                        self.state.active = None;
                        self.state.drag_start_targets.clear();
                        return Some(GizmoUpdate {
                            phase: GizmoPhase::Cancel,
                            active,
                            result: GizmoResult::Rotation {
                                axis: axis_dir,
                                delta_radians: 0.0,
                                total_radians: total,
                            },
                            updated_targets: targets.to_vec(),
                        });
                    }

                    if input.dragging {
                        self.state.drag_snap = input.snap;
                        let started_this_call = if !self.state.drag_has_started {
                            let threshold = self.config.drag_start_threshold_px.max(0.0);
                            if threshold > 0.0
                                && (input.cursor_px - self.state.drag_start_cursor_px).length()
                                    < threshold
                            {
                                return None;
                            }
                            self.state.drag_has_started = true;
                            true
                        } else {
                            false
                        };
                        let hit_world = ray_plane_intersect(
                            cursor_ray,
                            self.state.drag_origin,
                            self.state.drag_plane_normal,
                        )
                        .filter(|p| p.is_finite())
                        .unwrap_or_else(|| {
                            unproject_point(
                                view_projection,
                                viewport,
                                input.cursor_px,
                                self.config.depth_range,
                                self.state.drag_origin_z01,
                            )
                            .unwrap_or(self.state.drag_origin)
                        });

                        let Some(mut angle) = angle_on_plane(
                            self.state.drag_origin,
                            hit_world,
                            axis_dir,
                            self.state.drag_basis_u,
                            self.state.drag_basis_v,
                        ) else {
                            return None;
                        };
                        angle *= self.handedness_rotation_sign();

                        let delta_angle =
                            wrap_angle(angle - self.state.drag_prev_angle) * precision;
                        self.state.drag_prev_angle = angle;
                        self.state.drag_total_angle_raw += delta_angle;

                        let desired_total = if input.snap {
                            self.config
                                .rotate_snap_step_radians
                                .filter(|s| s.is_finite() && *s > 0.0)
                                .map(|step| (self.state.drag_total_angle_raw / step).round() * step)
                                .unwrap_or(self.state.drag_total_angle_raw)
                        } else {
                            self.state.drag_total_angle_raw
                        };
                        let delta_apply = desired_total - self.state.drag_total_angle_applied;
                        self.state.drag_total_angle_applied = desired_total;

                        let total_q = Quat::from_axis_angle(axis_dir, desired_total);
                        let updated_targets = self
                            .state
                            .drag_start_targets
                            .iter()
                            .map(|t| GizmoTarget3d {
                                id: t.id,
                                transform: Transform3d {
                                    translation: self.state.drag_origin
                                        + total_q
                                            * (t.transform.translation - self.state.drag_origin),
                                    rotation: (total_q * t.transform.rotation).normalize(),
                                    ..t.transform
                                },
                                local_bounds: t.local_bounds,
                            })
                            .collect::<Vec<_>>();
                        return Some(GizmoUpdate {
                            phase: if started_this_call {
                                GizmoPhase::Begin
                            } else {
                                GizmoPhase::Update
                            },
                            active,
                            result: GizmoResult::Rotation {
                                axis: axis_dir,
                                delta_radians: delta_apply,
                                total_radians: desired_total,
                            },
                            updated_targets,
                        });
                    }

                    if !self.state.drag_has_started {
                        self.state.active = None;
                        self.state.drag_start_targets.clear();
                        return None;
                    }

                    let total = self.state.drag_total_angle_applied;
                    self.state.active = None;
                    self.state.drag_start_targets.clear();
                    Some(GizmoUpdate {
                        phase: GizmoPhase::Commit,
                        active,
                        result: GizmoResult::Rotation {
                            axis: axis_dir,
                            delta_radians: 0.0,
                            total_radians: total,
                        },
                        updated_targets: targets.to_vec(),
                    })
                }
            }
            GizmoMode::Scale => {
                let _ = (view_projection, viewport);
                let length_world = self.state.drag_size_length_world.max(1e-6);

                let total_vec = |total_factor: f32| -> Vec3 {
                    if self.state.drag_scale_is_uniform {
                        Vec3::splat(total_factor)
                    } else if let Some(axis) = self.state.drag_scale_axis {
                        let mut v = Vec3::ONE;
                        v[axis] = total_factor;
                        v
                    } else {
                        Vec3::ONE
                    }
                };
                let total_plane_vec = |total_factors: Vec2| -> Vec3 {
                    let Some((a, b)) = self.state.drag_scale_plane_axes else {
                        return Vec3::ONE;
                    };
                    let mut v = Vec3::ONE;
                    v[a] = total_factors.x;
                    v[b] = total_factors.y;
                    v
                };

                if input.cancel {
                    let total = if self.state.drag_scale_is_bounds {
                        self.state.drag_bounds_total_applied
                    } else if self.state.drag_scale_plane_axes.is_some() {
                        total_plane_vec(self.state.drag_total_scale_plane_applied)
                    } else {
                        total_vec(self.state.drag_total_scale_applied)
                    };
                    self.state.active = None;
                    self.state.drag_scale_is_bounds = false;
                    self.state.drag_start_targets.clear();
                    return Some(GizmoUpdate {
                        phase: GizmoPhase::Cancel,
                        active,
                        result: GizmoResult::Scale {
                            delta: Vec3::ONE,
                            total,
                        },
                        updated_targets: targets.to_vec(),
                    });
                }

                if input.dragging {
                    self.state.drag_snap = input.snap;
                    let started_this_call = if !self.state.drag_has_started {
                        let threshold = self.config.drag_start_threshold_px.max(0.0);
                        if threshold > 0.0
                            && (input.cursor_px - self.state.drag_start_cursor_px).length()
                                < threshold
                        {
                            return None;
                        }
                        self.state.drag_has_started = true;
                        true
                    } else {
                        false
                    };

                    if self.state.drag_scale_is_bounds {
                        let hit_world = ray_plane_intersect(
                            cursor_ray,
                            self.state.drag_origin,
                            self.state.drag_plane_normal,
                        )
                        .filter(|p| p.is_finite())
                        .unwrap_or_else(|| {
                            unproject_point(
                                view_projection,
                                viewport,
                                input.cursor_px,
                                self.config.depth_range,
                                self.state.drag_origin_z01,
                            )
                            .unwrap_or(self.state.drag_origin)
                        });

                        let basis = self.state.drag_bounds_basis;
                        let diff_world = hit_world - self.state.drag_start_hit_world;
                        self.state.drag_prev_hit_world = hit_world;

                        let raw_local = Vec3::new(
                            diff_world.dot(basis[0]),
                            diff_world.dot(basis[1]),
                            diff_world.dot(basis[2]),
                        );
                        let delta_local =
                            (raw_local - self.state.drag_bounds_prev_local_raw) * precision;
                        self.state.drag_bounds_prev_local_raw = raw_local;

                        for i in 0..3 {
                            if self.state.drag_bounds_axes_mask[i] {
                                let sign = self.state.drag_bounds_axis_sign[i];
                                let extent = self.state.drag_bounds_start_extent[i].max(1e-6);
                                self.state.drag_bounds_total_raw[i] +=
                                    (delta_local[i] * sign) / extent;
                            }
                        }

                        let mut desired = Vec3::ONE;
                        for i in 0..3 {
                            if self.state.drag_bounds_axes_mask[i] {
                                let mut factor =
                                    (1.0 + self.state.drag_bounds_total_raw[i]).max(0.01);
                                if input.snap {
                                    if let Some(steps) =
                                        self.config.bounds_snap_step.filter(|v| v.is_finite())
                                    {
                                        let step = steps[i];
                                        if step.is_finite() && step > 0.0 {
                                            factor = snap_bounds_extent_factor(
                                                self.state.drag_bounds_start_extent[i].max(1e-6),
                                                factor,
                                                step,
                                            );
                                        }
                                    } else if let Some(step) = self
                                        .config
                                        .scale_snap_step
                                        .filter(|s| s.is_finite() && *s > 0.0)
                                    {
                                        factor = 1.0 + ((factor - 1.0) / step).round() * step;
                                        factor = factor.max(0.01);
                                    }
                                }
                                desired[i] = factor;
                            }
                        }

                        let delta = Vec3::new(
                            desired.x / self.state.drag_bounds_total_applied.x,
                            desired.y / self.state.drag_bounds_total_applied.y,
                            desired.z / self.state.drag_bounds_total_applied.z,
                        );
                        self.state.drag_bounds_total_applied = desired;

                        let updated_targets = self
                            .state
                            .drag_start_targets
                            .iter()
                            .map(|t| {
                                let origin = self.state.drag_origin;
                                let basis = self.state.drag_bounds_basis;
                                let anchor = self.state.drag_bounds_anchor_local;

                                let p = t.transform.translation - origin;
                                let coords =
                                    Vec3::new(p.dot(basis[0]), p.dot(basis[1]), p.dot(basis[2]));
                                let mut next = coords;
                                for i in 0..3 {
                                    if self.state.drag_bounds_axes_mask[i] {
                                        next[i] = anchor[i] + (coords[i] - anchor[i]) * desired[i];
                                    }
                                }
                                let translation = origin
                                    + basis[0] * next.x
                                    + basis[1] * next.y
                                    + basis[2] * next.z;

                                let mut scale = t.transform.scale;
                                for i in 0..3 {
                                    if self.state.drag_bounds_axes_mask[i] {
                                        scale[i] = (scale[i] * desired[i]).max(1e-4);
                                    }
                                }

                                GizmoTarget3d {
                                    id: t.id,
                                    transform: Transform3d {
                                        translation,
                                        scale,
                                        ..t.transform
                                    },
                                    local_bounds: t.local_bounds,
                                }
                            })
                            .collect::<Vec<_>>();

                        return Some(GizmoUpdate {
                            phase: if started_this_call {
                                GizmoPhase::Begin
                            } else {
                                GizmoPhase::Update
                            },
                            active,
                            result: GizmoResult::Scale {
                                delta,
                                total: desired,
                            },
                            updated_targets,
                        });
                    }

                    let hit_world = ray_plane_intersect(
                        cursor_ray,
                        self.state.drag_origin,
                        self.state.drag_plane_normal,
                    )
                    .filter(|p| p.is_finite())
                    .unwrap_or_else(|| {
                        unproject_point(
                            view_projection,
                            viewport,
                            input.cursor_px,
                            self.config.depth_range,
                            self.state.drag_origin_z01,
                        )
                        .unwrap_or(self.state.drag_origin)
                    });

                    let diff_world = hit_world - self.state.drag_start_hit_world;
                    self.state.drag_prev_hit_world = hit_world;

                    if let Some((a, b)) = self.state.drag_scale_plane_axes {
                        let u_dir = self.state.drag_scale_plane_u.normalize_or_zero();
                        let v_dir = self.state.drag_scale_plane_v.normalize_or_zero();
                        if u_dir.length_squared() == 0.0 || v_dir.length_squared() == 0.0 {
                            return None;
                        }

                        let raw = Vec2::new(diff_world.dot(u_dir), diff_world.dot(v_dir));
                        let delta_raw = raw - self.state.drag_scale_prev_plane_raw;
                        self.state.drag_scale_prev_plane_raw = raw;
                        self.state.drag_total_scale_plane_raw += delta_raw * precision;

                        let delta_norm = self.state.drag_total_scale_plane_raw / length_world;
                        let mut desired = if input.snap {
                            self.config
                                .scale_snap_step
                                .filter(|s| s.is_finite() && *s > 0.0)
                                .map(|step| {
                                    Vec2::new(
                                        1.0 + (delta_norm.x / step).round() * step,
                                        1.0 + (delta_norm.y / step).round() * step,
                                    )
                                })
                                .unwrap_or(Vec2::ONE + delta_norm)
                        } else {
                            Vec2::ONE + delta_norm
                        };
                        desired.x = desired.x.max(0.01);
                        desired.y = desired.y.max(0.01);

                        let delta_factors = Vec2::new(
                            desired.x / self.state.drag_total_scale_plane_applied.x,
                            desired.y / self.state.drag_total_scale_plane_applied.y,
                        );
                        self.state.drag_total_scale_plane_applied = desired;

                        let mut delta = Vec3::ONE;
                        delta[a] = delta_factors.x;
                        delta[b] = delta_factors.y;

                        let total = total_plane_vec(desired);

                        let updated_targets = self
                            .state
                            .drag_start_targets
                            .iter()
                            .map(|t| {
                                let origin = self.state.drag_origin;
                                let offset = t.transform.translation - origin;
                                let comp_u = u_dir * offset.dot(u_dir);
                                let comp_v = v_dir * offset.dot(v_dir);
                                let translation = origin
                                    + (offset
                                        + comp_u * (desired.x - 1.0)
                                        + comp_v * (desired.y - 1.0));

                                let mut scale = t.transform.scale;
                                scale[a] = (scale[a] * desired.x).max(1e-4);
                                scale[b] = (scale[b] * desired.y).max(1e-4);

                                GizmoTarget3d {
                                    id: t.id,
                                    transform: Transform3d {
                                        translation,
                                        scale,
                                        ..t.transform
                                    },
                                    local_bounds: t.local_bounds,
                                }
                            })
                            .collect::<Vec<_>>();

                        return Some(GizmoUpdate {
                            phase: if started_this_call {
                                GizmoPhase::Begin
                            } else {
                                GizmoPhase::Update
                            },
                            active,
                            result: GizmoResult::Scale { delta, total },
                            updated_targets,
                        });
                    }

                    let scale_dir = self.state.drag_axis_dir.normalize_or_zero();
                    if scale_dir.length_squared() == 0.0 {
                        return None;
                    }
                    let raw = diff_world.dot(scale_dir);
                    let delta_raw = raw - self.state.drag_scale_prev_axis_raw;
                    self.state.drag_scale_prev_axis_raw = raw;
                    self.state.drag_total_scale_raw += delta_raw * precision;

                    let delta_norm = self.state.drag_total_scale_raw / length_world;
                    let mut desired_factor = if input.snap {
                        self.config
                            .scale_snap_step
                            .filter(|s| s.is_finite() && *s > 0.0)
                            .map(|step| 1.0 + (delta_norm / step).round() * step)
                            .unwrap_or(1.0 + delta_norm)
                    } else {
                        1.0 + delta_norm
                    };
                    desired_factor = desired_factor.max(0.01);

                    let delta_factor = desired_factor / self.state.drag_total_scale_applied;
                    self.state.drag_total_scale_applied = desired_factor;

                    let delta = if self.state.drag_scale_is_uniform {
                        Vec3::splat(delta_factor)
                    } else if let Some(axis) = self.state.drag_scale_axis {
                        let mut v = Vec3::ONE;
                        v[axis] = delta_factor;
                        v
                    } else {
                        Vec3::ONE
                    };
                    let total = total_vec(desired_factor);

                    let updated_targets = self
                        .state
                        .drag_start_targets
                        .iter()
                        .map(|t| {
                            let origin = self.state.drag_origin;
                            let offset = t.transform.translation - origin;
                            let axis_dir = self.state.drag_axis_dir.normalize_or_zero();
                            let translation = if self.state.drag_scale_is_uniform {
                                origin + offset * desired_factor
                            } else if axis_dir.length_squared() > 0.0 {
                                let component = axis_dir * offset.dot(axis_dir);
                                origin + (offset + component * (desired_factor - 1.0))
                            } else {
                                t.transform.translation
                            };

                            let mut scale = t.transform.scale;
                            if self.state.drag_scale_is_uniform {
                                scale *= desired_factor;
                            } else if let Some(axis) = self.state.drag_scale_axis {
                                scale[axis] = (scale[axis] * desired_factor).max(1e-4);
                            }
                            GizmoTarget3d {
                                id: t.id,
                                transform: Transform3d {
                                    translation,
                                    scale,
                                    ..t.transform
                                },
                                local_bounds: t.local_bounds,
                            }
                        })
                        .collect::<Vec<_>>();

                    return Some(GizmoUpdate {
                        phase: if started_this_call {
                            GizmoPhase::Begin
                        } else {
                            GizmoPhase::Update
                        },
                        active,
                        result: GizmoResult::Scale { delta, total },
                        updated_targets,
                    });
                }

                if !self.state.drag_has_started {
                    self.state.active = None;
                    self.state.drag_scale_is_bounds = false;
                    self.state.drag_start_targets.clear();
                    return None;
                }

                let total = if self.state.drag_scale_is_bounds {
                    self.state.drag_bounds_total_applied
                } else if self.state.drag_scale_plane_axes.is_some() {
                    total_plane_vec(self.state.drag_total_scale_plane_applied)
                } else {
                    total_vec(self.state.drag_total_scale_applied)
                };
                self.state.active = None;
                self.state.drag_scale_is_bounds = false;
                self.state.drag_start_targets.clear();
                Some(GizmoUpdate {
                    phase: GizmoPhase::Commit,
                    active,
                    result: GizmoResult::Scale {
                        delta: Vec3::ONE,
                        total,
                    },
                    updated_targets: targets.to_vec(),
                })
            }
            GizmoMode::Universal => {
                self.state.active = None;
                None
            }
        }
    }

    pub fn draw(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> GizmoDrawList3d {
        if targets.is_empty() {
            return GizmoDrawList3d::default();
        }

        let active_index = targets
            .iter()
            .position(|t| t.id == active_target)
            .unwrap_or(0);
        let active_transform = targets
            .get(active_index)
            .map(|t| t.transform)
            .unwrap_or_else(|| targets[0].transform);

        let origin = Self::pivot_origin(active_transform, targets, self.config.pivot_mode);
        let size_length_world =
            self.size_length_world_or_one(view_projection, viewport, origin, targets);
        let axes_raw = self.axis_dirs(&active_transform);
        let axes = self.flip_axes_for_view(
            view_projection,
            viewport,
            origin,
            axes_raw,
            size_length_world,
        );

        if self.config.operation_mask.is_some() {
            let ops = self.effective_ops();
            let mut out = GizmoDrawList3d::default();

            let translate_axes = ops.contains(GizmoOps::translate_axis());
            let translate_planes = ops.contains(GizmoOps::translate_plane());
            let translate_screen = ops.contains(GizmoOps::translate_view());
            let translate_depth = ops.contains(GizmoOps::translate_depth());
            let rotate_any = ops.intersects(GizmoOps::rotate_all());
            let scale_axes = ops.contains(GizmoOps::scale_axis());
            let scale_planes = ops.contains(GizmoOps::scale_plane());
            let scale_uniform = ops.contains(GizmoOps::scale_uniform());
            let scale_bounds = ops.contains(GizmoOps::scale_bounds());

            if scale_bounds {
                let bounds_axes = [
                    axes_raw[0].normalize_or_zero(),
                    axes_raw[1].normalize_or_zero(),
                    axes_raw[2].normalize_or_zero(),
                ];
                self.draw_bounds(
                    &mut out,
                    view_projection,
                    viewport,
                    origin,
                    bounds_axes,
                    size_length_world,
                    targets,
                );
            }

            // When scale axes are present, skip the translate axis *lines* to reduce overlap.
            // The translate arrow tips remain as the explicit "grab" affordance.
            if translate_axes && !scale_axes {
                out.lines.extend(self.draw_translate_axes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
            }
            if translate_planes {
                out.lines.extend(self.draw_translate_planes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
            }
            if translate_screen {
                out.lines.extend(self.draw_translate_screen(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
            }
            if translate_depth {
                out.lines.extend(self.draw_translate_depth(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
            }

            if rotate_any {
                let rings = self.draw_rotate_rings(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                );
                out.lines.extend(rings.lines);
                out.triangles.extend(rings.triangles);
            }

            if scale_axes || scale_planes || scale_uniform {
                out.lines.extend(self.draw_scale_handles(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    scale_axes,
                    scale_uniform,
                    scale_planes,
                ));
            }

            let rotate_feedback =
                self.draw_rotate_feedback(view_projection, viewport, origin, size_length_world);
            let translate_feedback =
                self.draw_translate_feedback(view_projection, viewport, origin, size_length_world);
            let scale_feedback = self.draw_scale_feedback(view_projection, viewport, origin);
            out.lines.extend(rotate_feedback.lines);
            out.lines.extend(translate_feedback.lines);
            out.lines.extend(scale_feedback.lines);

            if translate_axes || translate_planes || translate_screen {
                out.triangles.extend(self.draw_translate_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    translate_axes,
                    translate_planes,
                    translate_screen,
                ));
            }
            if scale_axes || scale_planes || scale_uniform {
                out.triangles.extend(self.draw_scale_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    scale_axes,
                    scale_uniform,
                    scale_planes,
                ));
            }
            out.triangles.extend(rotate_feedback.triangles);

            return out;
        }

        match self.config.mode {
            GizmoMode::Translate => {
                let mut out = GizmoDrawList3d::default();
                out.lines.extend(self.draw_translate_axes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
                out.lines.extend(self.draw_translate_planes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
                out.lines.extend(self.draw_translate_screen(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
                out.lines.extend(self.draw_translate_depth(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
                let feedback = self.draw_translate_feedback(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                );
                out.lines.extend(feedback.lines);
                out.triangles.extend(self.draw_translate_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    true,
                    true,
                    true,
                ));
                out
            }
            GizmoMode::Rotate => {
                let mut out = GizmoDrawList3d::default();
                let rings = self.draw_rotate_rings(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                );
                out.lines.extend(rings.lines);
                out.triangles.extend(rings.triangles);
                let feedback =
                    self.draw_rotate_feedback(view_projection, viewport, origin, size_length_world);
                out.lines.extend(feedback.lines);
                out.triangles.extend(feedback.triangles);
                out
            }
            GizmoMode::Scale => {
                let mut out = GizmoDrawList3d::default();
                if self.config.show_bounds {
                    let bounds_axes = [
                        axes_raw[0].normalize_or_zero(),
                        axes_raw[1].normalize_or_zero(),
                        axes_raw[2].normalize_or_zero(),
                    ];
                    self.draw_bounds(
                        &mut out,
                        view_projection,
                        viewport,
                        origin,
                        bounds_axes,
                        size_length_world,
                        targets,
                    );
                }
                out.lines.extend(self.draw_scale_handles(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    true,
                    true,
                    true,
                ));
                let feedback = self.draw_scale_feedback(view_projection, viewport, origin);
                out.lines.extend(feedback.lines);
                out.triangles.extend(self.draw_scale_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    true,
                    true,
                    true,
                ));
                out
            }
            GizmoMode::Universal => {
                let mut out = GizmoDrawList3d::default();
                if !self.config.universal_includes_scale {
                    out.lines.extend(self.draw_translate_axes(
                        view_projection,
                        viewport,
                        origin,
                        axes,
                        size_length_world,
                    ));
                }
                out.lines.extend(self.draw_translate_planes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                ));
                out.lines.extend(self.draw_translate_screen(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                ));
                if self.config.universal_includes_translate_depth {
                    out.lines.extend(self.draw_translate_depth(
                        view_projection,
                        viewport,
                        origin,
                        size_length_world,
                    ));
                }
                let rings = self.draw_rotate_rings(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                );
                out.lines.extend(rings.lines);
                out.triangles.extend(rings.triangles);
                if self.config.universal_includes_scale {
                    out.lines.extend(self.draw_scale_handles(
                        view_projection,
                        viewport,
                        origin,
                        axes,
                        size_length_world,
                        true,
                        false,
                        false,
                    ));
                }
                let rotate_feedback =
                    self.draw_rotate_feedback(view_projection, viewport, origin, size_length_world);
                let translate_feedback = self.draw_translate_feedback(
                    view_projection,
                    viewport,
                    origin,
                    size_length_world,
                );
                let scale_feedback = self.draw_scale_feedback(view_projection, viewport, origin);
                out.lines.extend(rotate_feedback.lines);
                out.lines.extend(translate_feedback.lines);
                out.lines.extend(scale_feedback.lines);
                out.triangles.extend(self.draw_translate_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                    size_length_world,
                    true,
                    true,
                    true,
                ));
                if self.config.universal_includes_scale {
                    out.triangles.extend(self.draw_scale_solids(
                        view_projection,
                        viewport,
                        origin,
                        axes,
                        size_length_world,
                        true,
                        false,
                        false,
                    ));
                }
                out.triangles.extend(rotate_feedback.triangles);
                out
            }
        }
    }

    fn is_handle_highlighted(&self, kind: GizmoMode, handle: HandleId) -> bool {
        if self.state.active == Some(handle) {
            return self.state.drag_mode == kind;
        }
        self.state.hovered == Some(handle) && self.state.hovered_kind == Some(kind)
    }

    fn push_line(&self, out: &mut Vec<Line3d>, a: Vec3, b: Vec3, color: Color, depth: DepthMode) {
        match (depth, self.config.show_occluded) {
            (DepthMode::Test, true) => {
                out.push(Line3d {
                    a,
                    b,
                    color: mix_alpha(color, self.config.occluded_alpha),
                    depth: DepthMode::Ghost,
                });
                out.push(Line3d {
                    a,
                    b,
                    color,
                    depth: DepthMode::Test,
                });
            }
            _ => {
                out.push(Line3d { a, b, color, depth });
            }
        }
    }

    fn push_line_no_ghost(
        &self,
        out: &mut Vec<Line3d>,
        a: Vec3,
        b: Vec3,
        color: Color,
        depth: DepthMode,
    ) {
        out.push(Line3d { a, b, color, depth });
    }

    fn push_quad_outline(
        &self,
        out: &mut Vec<Line3d>,
        quad: [Vec3; 4],
        color: Color,
        depth: DepthMode,
        allow_ghost: bool,
    ) {
        for (a, b) in [
            (quad[0], quad[1]),
            (quad[1], quad[2]),
            (quad[2], quad[3]),
            (quad[3], quad[0]),
        ] {
            if allow_ghost {
                self.push_line(out, a, b, color, depth);
            } else {
                self.push_line_no_ghost(out, a, b, color, depth);
            }
        }
    }

    fn push_tri(
        &self,
        out: &mut Vec<Triangle3d>,
        a: Vec3,
        b: Vec3,
        c: Vec3,
        color: Color,
        depth: DepthMode,
    ) {
        match (depth, self.config.show_occluded) {
            (DepthMode::Test, true) => {
                out.push(Triangle3d {
                    a,
                    b,
                    c,
                    color: mix_alpha(color, self.config.occluded_alpha),
                    depth: DepthMode::Ghost,
                });
                out.push(Triangle3d {
                    a,
                    b,
                    c,
                    color,
                    depth: DepthMode::Test,
                });
            }
            _ => {
                out.push(Triangle3d {
                    a,
                    b,
                    c,
                    color,
                    depth,
                });
            }
        }
    }

    fn push_tri_no_ghost(
        &self,
        out: &mut Vec<Triangle3d>,
        a: Vec3,
        b: Vec3,
        c: Vec3,
        color: Color,
        depth: DepthMode,
    ) {
        out.push(Triangle3d {
            a,
            b,
            c,
            color,
            depth,
        });
    }

    fn push_quad_fill(
        &self,
        out: &mut Vec<Triangle3d>,
        quad: [Vec3; 4],
        color: Color,
        depth: DepthMode,
        allow_ghost: bool,
    ) {
        if allow_ghost {
            self.push_tri(out, quad[0], quad[1], quad[2], color, depth);
            self.push_tri(out, quad[0], quad[2], quad[3], color, depth);
        } else {
            self.push_tri_no_ghost(out, quad[0], quad[1], quad[2], color, depth);
            self.push_tri_no_ghost(out, quad[0], quad[2], quad[3], color, depth);
        }
    }

    fn push_ring_band(
        &self,
        out: &mut GizmoDrawList3d,
        origin: Vec3,
        u: Vec3,
        v: Vec3,
        inner_r: f32,
        outer_r: f32,
        fill: Color,
        edge: Color,
        depth: DepthMode,
        allow_ghost: bool,
        segments: usize,
    ) {
        let point =
            |theta: f32, r: f32| -> Vec3 { origin + (u * theta.cos() + v * theta.sin()) * r };

        let step = std::f32::consts::TAU / (segments as f32);
        let mut prev_outer = point(0.0, outer_r);
        for i in 0..segments {
            let t0 = step * (i as f32);
            let t1 = step * ((i + 1) as f32);
            let o0 = point(t0, outer_r);
            let i0 = point(t0, inner_r);
            let o1 = point(t1, outer_r);
            let i1 = point(t1, inner_r);

            self.push_quad_fill(
                &mut out.triangles,
                [o0, i0, i1, o1],
                fill,
                depth,
                allow_ghost,
            );
            if allow_ghost {
                self.push_line(&mut out.lines, prev_outer, o1, edge, depth);
            } else {
                self.push_line_no_ghost(&mut out.lines, prev_outer, o1, edge, depth);
            }
            prev_outer = o1;
        }
    }

    fn axis_dirs(&self, target: &Transform3d) -> [Vec3; 3] {
        match self.config.orientation {
            GizmoOrientation::World => [Vec3::X, Vec3::Y, Vec3::Z],
            GizmoOrientation::Local => [
                target.rotation * Vec3::X,
                target.rotation * Vec3::Y,
                target.rotation * Vec3::Z,
            ],
        }
    }

    fn bounds_min_max_local(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        basis: [Vec3; 3],
        size_length_world: f32,
        targets: &[GizmoTarget3d],
    ) -> (Vec3, Vec3) {
        let mut min_v = Vec3::splat(f32::INFINITY);
        let mut max_v = Vec3::splat(f32::NEG_INFINITY);

        for t in targets {
            if let Some(aabb) = t.local_bounds {
                let aabb = aabb.normalized();
                let m = t.transform.to_mat4();
                for c in aabb.corners() {
                    let world = m.transform_point3(c);
                    if !world.is_finite() {
                        continue;
                    }
                    let p = world - origin;
                    let v = Vec3::new(p.dot(basis[0]), p.dot(basis[1]), p.dot(basis[2]));
                    min_v = min_v.min(v);
                    max_v = max_v.max(v);
                }
            } else {
                let p = t.transform.translation - origin;
                let v = Vec3::new(p.dot(basis[0]), p.dot(basis[1]), p.dot(basis[2]));
                min_v = min_v.min(v);
                max_v = max_v.max(v);
            }
        }

        let _ = (view_projection, viewport);
        let min_extent = size_length_world.max(1e-6) * 0.25;

        if !min_v.is_finite() || !max_v.is_finite() {
            let half = Vec3::splat(min_extent.max(1e-6) * 0.5);
            return (-half, half);
        }

        let center = (min_v + max_v) * 0.5;
        let extent = (max_v - min_v).max(Vec3::splat(min_extent));
        (center - extent * 0.5, center + extent * 0.5)
    }

    fn bounds_corner_id(x_max: bool, y_max: bool, z_max: bool) -> HandleId {
        let bits = (x_max as u64) | ((y_max as u64) << 1) | ((z_max as u64) << 2);
        HandleId(Self::BOUNDS_CORNER_BASE + bits)
    }

    fn bounds_face_id(axis: usize, max_side: bool) -> HandleId {
        let axis = axis.min(2) as u64;
        let side = if max_side { 1u64 } else { 0u64 };
        HandleId(Self::BOUNDS_FACE_BASE + axis * 2 + side)
    }

    fn bounds_handle_from_id(handle: HandleId) -> Option<BoundsHandle> {
        match handle.0 {
            Self::BOUNDS_CORNER_BASE..=Self::BOUNDS_CORNER_END => {
                let bits = handle.0 - Self::BOUNDS_CORNER_BASE;
                Some(BoundsHandle::Corner {
                    x_max: (bits & 1) != 0,
                    y_max: (bits & 2) != 0,
                    z_max: (bits & 4) != 0,
                })
            }
            Self::BOUNDS_FACE_BASE..=Self::BOUNDS_FACE_END => {
                let v = handle.0 - Self::BOUNDS_FACE_BASE;
                let axis = (v / 2) as usize;
                let max_side = (v % 2) == 1;
                Some(BoundsHandle::Face { axis, max_side })
            }
            _ => None,
        }
    }

    fn begin_translate_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        cursor_ray: Ray3d,
        origin: Vec3,
        active: HandleId,
        axes: [Vec3; 3],
    ) -> Option<GizmoUpdate> {
        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;
        let constraint = translate_constraint_for_handle(
            view_projection,
            viewport,
            self.config.depth_range,
            origin,
            active,
            axes,
        )?;

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Translate;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_total_axis_raw = 0.0;
        self.state.drag_total_axis_applied = 0.0;
        self.state.drag_total_plane_raw = Vec2::ZERO;
        self.state.drag_total_plane_applied = Vec2::ZERO;

        self.state.drag_start_targets = targets.to_vec();

        match constraint {
            TranslateConstraint::Axis { axis_dir } => {
                self.state.drag_translate_is_plane = false;
                self.state.drag_translate_is_dolly = false;
                self.state.drag_translate_dolly_world_per_px = 0.0;
                self.state.drag_axis_dir = axis_dir;
                let plane_normal = axis_drag_plane_normal(
                    view_projection,
                    viewport,
                    self.config.depth_range,
                    origin,
                    axis_dir,
                )?;
                self.state.drag_plane_normal = plane_normal;
            }
            TranslateConstraint::Plane { u, v, normal } => {
                self.state.drag_translate_is_plane = true;
                self.state.drag_translate_is_dolly = false;
                self.state.drag_translate_dolly_world_per_px = 0.0;
                self.state.drag_translate_u = u;
                self.state.drag_translate_v = v;
                self.state.drag_plane_normal = normal;
            }
            TranslateConstraint::Dolly { view_dir } => {
                let axis_dir = view_dir.normalize_or_zero();
                if axis_dir.length_squared() == 0.0 {
                    return None;
                }
                self.state.drag_translate_is_plane = false;
                self.state.drag_translate_is_dolly = true;
                self.state.drag_axis_dir = axis_dir;

                // A stable plane isn't required for dolly updates, but we still set a sane value so
                // fallback unprojection stays close to the origin depth when needed.
                self.state.drag_plane_normal = axis_dir;

                let world_per_px = axis_length_world(
                    view_projection,
                    viewport,
                    origin,
                    self.config.depth_range,
                    1.0,
                )
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or_else(|| {
                    let size_length_world =
                        self.size_length_world_or_one(view_projection, viewport, origin, targets);
                    let px = self.config.size_px.max(1.0);
                    let v = size_length_world / px;
                    if v.is_finite() && v > 0.0 { v } else { 1e-3 }
                });
                self.state.drag_translate_dolly_world_per_px = world_per_px;
            }
        }

        let start_hit_world = ray_plane_intersect(cursor_ray, origin, self.state.drag_plane_normal)
            .filter(|p| p.is_finite())
            .unwrap_or_else(|| {
                unproject_point(
                    view_projection,
                    viewport,
                    input.cursor_px,
                    self.config.depth_range,
                    origin_z01,
                )
                .unwrap_or(origin)
            });
        self.state.drag_start_hit_world = start_hit_world;
        self.state.drag_prev_hit_world = start_hit_world;
        self.state.drag_translate_prev_axis_raw = 0.0;
        self.state.drag_translate_prev_plane_raw = Vec2::ZERO;

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Translation {
                delta: Vec3::ZERO,
                total: Vec3::ZERO,
            },
            updated_targets: targets.to_vec(),
        })
    }

    fn begin_rotate_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        cursor_ray: Ray3d,
        origin: Vec3,
        active: HandleId,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Option<GizmoUpdate> {
        if active == Self::ROTATE_ARCBALL_HANDLE {
            return self.begin_arcball_drag(
                view_projection,
                viewport,
                input,
                targets,
                origin,
                size_length_world,
                active,
            );
        }

        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;
        let axis_dir = if active == Self::ROTATE_VIEW_HANDLE {
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?
        } else {
            let (_, axis_index) = axis_for_handle(active);
            axes[axis_index]
        }
        .normalize_or_zero();
        if axis_dir.length_squared() == 0.0 {
            return None;
        }

        let (u, v) = plane_basis(axis_dir);

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Rotate;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_axis_dir = axis_dir;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_plane_normal = axis_dir;
        self.state.drag_basis_u = u;
        self.state.drag_basis_v = v;
        self.state.drag_total_angle_raw = 0.0;
        self.state.drag_total_angle_applied = 0.0;
        self.state.drag_rotate_is_arcball = false;
        self.state.drag_total_arcball_raw = Quat::IDENTITY;
        self.state.drag_total_arcball_applied = Quat::IDENTITY;
        self.state.drag_start_targets = targets.to_vec();

        let start_hit_world = ray_plane_intersect(cursor_ray, origin, axis_dir)
            .filter(|p| p.is_finite())
            .unwrap_or_else(|| {
                unproject_point(
                    view_projection,
                    viewport,
                    input.cursor_px,
                    self.config.depth_range,
                    origin_z01,
                )
                .unwrap_or(origin + u)
            });

        let mut angle = angle_on_plane(origin, start_hit_world, axis_dir, u, v)?;
        angle *= self.handedness_rotation_sign();
        self.state.drag_start_angle = angle;
        self.state.drag_prev_angle = angle;

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Rotation {
                axis: axis_dir,
                delta_radians: 0.0,
                total_radians: 0.0,
            },
            updated_targets: targets.to_vec(),
        })
    }

    fn begin_arcball_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        origin: Vec3,
        size_length_world: f32,
        active: HandleId,
    ) -> Option<GizmoUpdate> {
        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;
        let view_dir =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?;
        let n = (-view_dir).normalize_or_zero();
        if n.length_squared() == 0.0 {
            return None;
        }
        let (u, v) = plane_basis(n);

        let center_px =
            project_point(view_projection, viewport, origin, self.config.depth_range)?.screen;
        let radius_px = match self.config.size_policy {
            GizmoSizePolicy::ConstantPixels => {
                self.config.size_px * self.config.arcball_radius_scale
            }
            GizmoSizePolicy::PixelsClampedBySelectionBounds { .. }
            | GizmoSizePolicy::SelectionBounds { .. } => {
                let r_world = (size_length_world * self.config.arcball_radius_scale).max(1e-6);
                axis_segment_len_px(
                    view_projection,
                    viewport,
                    origin,
                    self.config.depth_range,
                    u,
                    r_world,
                )
                .unwrap_or(0.0)
            }
        }
        .max(self.config.pick_radius_px.max(6.0));

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Rotate;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_axis_dir = n;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_plane_normal = n;
        self.state.drag_basis_u = u;
        self.state.drag_basis_v = v;
        self.state.drag_rotate_is_arcball = true;
        self.state.drag_arcball_center_px = center_px;
        self.state.drag_arcball_radius_px = radius_px;
        self.state.drag_arcball_prev_vec = self.arcball_vector_world(input.cursor_px)?;
        self.state.drag_total_arcball_raw = Quat::IDENTITY;
        self.state.drag_total_arcball_applied = Quat::IDENTITY;
        self.state.drag_total_angle_raw = 0.0;
        self.state.drag_total_angle_applied = 0.0;
        self.state.drag_start_targets = targets.to_vec();

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Arcball {
                delta: Quat::IDENTITY,
                total: Quat::IDENTITY,
            },
            updated_targets: targets.to_vec(),
        })
    }

    fn arcball_vector_world(&self, cursor_px: Vec2) -> Option<Vec3> {
        let r = self.state.drag_arcball_radius_px;
        if !r.is_finite() || r <= 1e-3 {
            return None;
        }
        let p = (cursor_px - self.state.drag_arcball_center_px) / r;
        if !p.x.is_finite() || !p.y.is_finite() {
            return None;
        }

        // Note: screen Y is down, but arcball math expects Y up.
        let mut x = p.x;
        let mut y = -p.y;
        let d2 = x * x + y * y;
        let z = if d2 <= 1.0 {
            (1.0 - d2).sqrt()
        } else {
            let inv = d2.sqrt().recip();
            x *= inv;
            y *= inv;
            0.0
        };

        let u = self.state.drag_basis_u.normalize_or_zero();
        let v = self.state.drag_basis_v.normalize_or_zero();
        let n = self.state.drag_plane_normal.normalize_or_zero();
        if u.length_squared() == 0.0 || v.length_squared() == 0.0 || n.length_squared() == 0.0 {
            return None;
        }

        let w = (u * x + v * y + n * z).normalize_or_zero();
        (w.length_squared() > 0.0).then_some(w)
    }

    fn begin_scale_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        cursor_ray: Ray3d,
        origin: Vec3,
        active: HandleId,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Option<GizmoUpdate> {
        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;

        let (scale_dir, plane_normal, axis, plane_axes, plane_u, plane_v) = match active.0 {
            1 | 2 | 3 => {
                let (_, axis_index) = axis_for_handle(active);
                let axis_dir = axes[axis_index].normalize_or_zero();
                if axis_dir.length_squared() == 0.0 {
                    return None;
                }
                let plane_normal = axis_drag_plane_normal(
                    view_projection,
                    viewport,
                    self.config.depth_range,
                    origin,
                    axis_dir,
                )?;
                (
                    axis_dir,
                    plane_normal,
                    Some(axis_index),
                    None,
                    Vec3::X,
                    Vec3::Y,
                )
            }
            7 => {
                let view_dir =
                    view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?;
                let (u, v) = plane_basis(view_dir);
                let dir = (u + v).normalize_or_zero();
                let n = view_dir.normalize_or_zero();
                if dir.length_squared() == 0.0 || n.length_squared() == 0.0 {
                    return None;
                }
                (dir, n, None, None, Vec3::X, Vec3::Y)
            }
            14 | 15 | 16 => {
                let (a, b) = match active.0 {
                    14 => (0, 1),
                    15 => (0, 2),
                    16 => (1, 2),
                    _ => unreachable!(),
                };
                let u = axes[a].normalize_or_zero();
                let v = axes[b].normalize_or_zero();
                if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
                    return None;
                }
                let n = u.cross(v).normalize_or_zero();
                if n.length_squared() == 0.0 {
                    return None;
                }
                (Vec3::ZERO, n, None, Some((a, b)), u, v)
            }
            _ => return None,
        };

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Scale;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_size_length_world = size_length_world;
        self.state.drag_plane_normal = plane_normal;
        self.state.drag_axis_dir = scale_dir;
        self.state.drag_scale_axis = axis;
        self.state.drag_scale_plane_axes = plane_axes;
        self.state.drag_scale_plane_u = plane_u;
        self.state.drag_scale_plane_v = plane_v;
        let start_hit_world = ray_plane_intersect(cursor_ray, origin, plane_normal)
            .filter(|p| p.is_finite())
            .unwrap_or_else(|| {
                unproject_point(
                    view_projection,
                    viewport,
                    input.cursor_px,
                    self.config.depth_range,
                    origin_z01,
                )
                .unwrap_or(origin)
            });
        self.state.drag_start_hit_world = start_hit_world;
        self.state.drag_prev_hit_world = start_hit_world;
        self.state.drag_scale_prev_axis_raw = 0.0;
        self.state.drag_scale_prev_plane_raw = Vec2::ZERO;
        self.state.drag_total_scale_raw = 0.0;
        self.state.drag_total_scale_applied = 1.0;
        self.state.drag_total_scale_plane_raw = Vec2::ZERO;
        self.state.drag_total_scale_plane_applied = Vec2::ONE;
        self.state.drag_scale_is_uniform = axis.is_none() && plane_axes.is_none();
        self.state.drag_scale_is_bounds = false;
        self.state.drag_start_targets = targets.to_vec();

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Scale {
                delta: Vec3::ONE,
                total: Vec3::ONE,
            },
            updated_targets: targets.to_vec(),
        })
    }

    fn begin_bounds_drag(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[GizmoTarget3d],
        cursor_ray: Ray3d,
        origin: Vec3,
        origin_z01: f32,
        size_length_world: f32,
        handle: BoundsHandle,
        active: HandleId,
        axes_raw: [Vec3; 3],
    ) -> Option<GizmoUpdate> {
        let basis = [
            axes_raw[0].normalize_or_zero(),
            axes_raw[1].normalize_or_zero(),
            axes_raw[2].normalize_or_zero(),
        ];
        if basis.iter().any(|v| v.length_squared() == 0.0) {
            return None;
        }

        let (min_local, max_local) = self.bounds_min_max_local(
            view_projection,
            viewport,
            origin,
            basis,
            size_length_world,
            targets,
        );
        let center_local = (min_local + max_local) * 0.5;
        let extent = (max_local - min_local).max(Vec3::splat(1e-6));

        let mut axes_mask = [false; 3];
        let mut axis_sign = [1.0f32; 3];
        let mut anchor_local = min_local;

        match handle {
            BoundsHandle::Corner {
                x_max,
                y_max,
                z_max,
            } => {
                let sides = [x_max, y_max, z_max];
                for i in 0..3 {
                    axes_mask[i] = true;
                    axis_sign[i] = if sides[i] { 1.0 } else { -1.0 };
                    anchor_local[i] = if sides[i] { min_local[i] } else { max_local[i] };
                }
            }
            BoundsHandle::Face { axis, max_side } => {
                axes_mask[axis.min(2)] = true;
                axis_sign[axis.min(2)] = if max_side { 1.0 } else { -1.0 };
                for i in 0..3 {
                    anchor_local[i] = match (i, axis.min(2)) {
                        (a, b) if a == b => {
                            if max_side {
                                min_local[i]
                            } else {
                                max_local[i]
                            }
                        }
                        _ => center_local[i],
                    };
                }
            }
        }

        let axes_count = axes_mask.iter().filter(|v| **v).count();
        let plane_normal = if axes_count == 1 {
            let axis = axes_mask.iter().position(|v| *v).unwrap_or(0);
            let axis_dir = basis[axis] * axis_sign[axis];
            axis_drag_plane_normal_facing_camera(
                view_projection,
                viewport,
                self.config.depth_range,
                origin,
                axis_dir,
            )?
        } else {
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?
        };

        let start_hit_world = ray_plane_intersect(cursor_ray, origin, plane_normal)
            .filter(|p| p.is_finite())
            .unwrap_or(origin);

        self.state.active = Some(active);
        self.state.drag_mode = GizmoMode::Scale;
        self.state.drag_snap = input.snap;
        self.state.drag_has_started = false;
        self.state.drag_start_cursor_px = input.cursor_px;
        self.state.drag_origin = origin;
        self.state.drag_origin_z01 = origin_z01;
        self.state.drag_size_length_world = size_length_world;
        self.state.drag_plane_normal = plane_normal.normalize_or_zero();
        self.state.drag_start_hit_world = start_hit_world;
        self.state.drag_prev_hit_world = start_hit_world;
        self.state.drag_bounds_prev_local_raw = Vec3::ZERO;

        self.state.drag_scale_is_bounds = true;
        self.state.drag_bounds_basis = basis;
        self.state.drag_bounds_min_local = min_local;
        self.state.drag_bounds_max_local = max_local;
        self.state.drag_bounds_anchor_local = anchor_local;
        self.state.drag_bounds_axes_mask = axes_mask;
        self.state.drag_bounds_axis_sign = axis_sign;
        self.state.drag_bounds_start_extent = extent;
        self.state.drag_bounds_total_raw = Vec3::ZERO;
        self.state.drag_bounds_total_applied = Vec3::ONE;

        self.state.drag_total_scale_raw = 0.0;
        self.state.drag_total_scale_applied = 1.0;
        self.state.drag_total_scale_plane_raw = Vec2::ZERO;
        self.state.drag_total_scale_plane_applied = Vec2::ONE;
        self.state.drag_scale_axis = None;
        self.state.drag_scale_plane_axes = None;
        self.state.drag_scale_is_uniform = false;
        self.state.drag_start_targets = targets.to_vec();

        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active,
            result: GizmoResult::Scale {
                delta: Vec3::ONE,
                total: Vec3::ONE,
            },
            updated_targets: targets.to_vec(),
        })
    }

    fn tick_perp_dir(
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        depth: DepthRange,
        dir: Vec3,
    ) -> Vec3 {
        let dir = dir.normalize_or_zero();
        if dir.length_squared() == 0.0 {
            return Vec3::X;
        }
        if let Some(view_dir) = view_dir_at_origin(view_projection, viewport, origin, depth) {
            let perp = dir.cross(view_dir).normalize_or_zero();
            if perp.length_squared() > 0.0 {
                return perp;
            }
        }
        plane_basis(dir).0.normalize_or_zero()
    }

    fn pick_translate_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        include_axes: bool,
        include_planes: bool,
        include_screen: bool,
        include_depth: bool,
    ) -> Option<PickHit> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;
        let axis_tip_len = length_world * self.translate_axis_tip_scale();

        // Picking priority ladder (editor UX):
        // 1) Center / screen-plane handle (when within radius, always win)
        // 2) Plane handles (when cursor is inside the plane quad)
        // 3) Axis handles (distance to segment)
        //
        // This avoids a common frustration where the axis segment "steals" clicks near the origin.
        if include_screen {
            if let Some(p0) =
                project_point(view_projection, viewport, origin, self.config.depth_range)
            {
                let r = self.config.pick_radius_px.max(6.0);
                if let Some(d) = (PickCircle2d {
                    center: p0.screen,
                    radius: r,
                })
                .hit_distance(cursor)
                {
                    return Some(PickHit {
                        handle: TranslateHandle::Screen.id(),
                        score: d,
                    });
                }
            }
        }

        // Dolly translation handle (a small ring in the view plane around the center).
        if include_depth {
            if let Some(view_dir) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            {
                let axis_dir = view_dir.normalize_or_zero();
                if axis_dir.length_squared() > 0.0 {
                    let (u, v) = plane_basis(axis_dir);
                    let r_world = (length_world * pv.translate_depth_ring_radius_fraction.max(0.0))
                        .max(length_world * pv.translate_depth_ring_radius_min_fraction.max(0.0));
                    let segments: usize = 36;
                    let mut prev_world = origin + u * r_world;
                    let mut best_d = f32::INFINITY;
                    for i in 1..=segments {
                        let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                        let world = origin + (u * t.cos() + v * t.sin()) * r_world;
                        let Some(pa) = project_point(
                            view_projection,
                            viewport,
                            prev_world,
                            self.config.depth_range,
                        ) else {
                            prev_world = world;
                            continue;
                        };
                        let Some(pb) = project_point(
                            view_projection,
                            viewport,
                            world,
                            self.config.depth_range,
                        ) else {
                            prev_world = world;
                            continue;
                        };
                        best_d =
                            best_d.min(distance_point_to_segment_px(cursor, pa.screen, pb.screen));
                        prev_world = world;
                    }
                    let r = self.config.pick_radius_px.max(6.0);
                    if best_d.is_finite() && best_d <= r {
                        return Some(PickHit {
                            handle: TranslateHandle::Depth.id(),
                            score: best_d,
                        });
                    }
                }
            }
        }

        // Plane handles (distance to projected quad; accept when inside).
        let off = length_world * pv.translate_plane_offset_fraction.max(0.0);
        let size = length_world * pv.translate_plane_size_fraction.max(0.0);
        let mut plane_inside: Option<(HandleId, f32)> = None;
        if include_planes {
            for &((u, v, handle), plane_axes) in &[
                (
                    (axes[0], axes[1], TranslateHandle::PlaneXY.id()),
                    (0usize, 1usize),
                ),
                (
                    (axes[0], axes[2], TranslateHandle::PlaneXZ.id()),
                    (0usize, 2usize),
                ),
                (
                    (axes[1], axes[2], TranslateHandle::PlaneYZ.id()),
                    (1usize, 2usize),
                ),
            ] {
                if !self.plane_allowed_by_mask(plane_axes) {
                    continue;
                }
                let world = translate_plane_quad_world(origin, u, v, off, size);
                let Some(p) =
                    project_quad(view_projection, viewport, world, self.config.depth_range)
                else {
                    continue;
                };
                let alpha = self.plane_visibility_alpha(view_projection, viewport, world);
                if alpha <= 0.01 {
                    continue;
                }

                let quad = PickConvexQuad2d { points: p };
                let inside = quad.contains(cursor);
                let edge_d = quad.edge_distance(cursor);
                if inside {
                    // When the cursor is actually inside the plane handle quad, always prefer plane
                    // drags over axis segments (common editor expectation).
                    //
                    // If multiple plane quads overlap in projection, prefer the one where the cursor
                    // is deeper inside (larger edge distance).
                    match plane_inside {
                        Some((_, best_edge_d)) if edge_d <= best_edge_d => {}
                        _ => plane_inside = Some((handle, edge_d)),
                    }
                } else {
                    // Edge-picking is handled below as part of the general "best score" selection.
                }
            }
        }

        if let Some((handle, _)) = plane_inside {
            return Some(PickHit { handle, score: 0.0 });
        }

        let mut best: Option<PickHit> = None;
        let mut consider = |handle: HandleId, score: f32| {
            if !score.is_finite() {
                return;
            }
            match best {
                Some(best) if score >= best.score => {}
                _ => best = Some(PickHit { handle, score }),
            }
        };

        // Axis handles (distance to projected segments).
        if include_axes {
            for &((axis_dir, handle), axis_index) in &[
                ((axes[0], TranslateHandle::AxisX.id()), 0usize),
                ((axes[1], TranslateHandle::AxisY.id()), 1usize),
                ((axes[2], TranslateHandle::AxisZ.id()), 2usize),
            ] {
                if self.axis_is_masked(axis_index) {
                    continue;
                }
                let a = origin;
                let b = origin + axis_dir * axis_tip_len;
                let Some(pa) = project_point(view_projection, viewport, a, self.config.depth_range)
                else {
                    continue;
                };
                let Some(pb) = project_point(view_projection, viewport, b, self.config.depth_range)
                else {
                    continue;
                };

                let alpha = self.axis_visibility_alpha(
                    view_projection,
                    viewport,
                    origin,
                    axis_dir,
                    axis_tip_len,
                );
                if alpha <= 0.01 {
                    continue;
                }
                let r = self.config.pick_radius_px * alpha.sqrt();
                if let Some(d) = (PickSegmentCapsule2d {
                    a: pa.screen,
                    b: pb.screen,
                    radius: r,
                })
                .hit_distance(cursor)
                {
                    consider(handle, d / alpha.max(0.05));
                }
            }
        }

        // Plane handle edge picking.
        if include_planes {
            for &((u, v, handle), plane_axes) in &[
                (
                    (axes[0], axes[1], TranslateHandle::PlaneXY.id()),
                    (0usize, 1usize),
                ),
                (
                    (axes[0], axes[2], TranslateHandle::PlaneXZ.id()),
                    (0usize, 2usize),
                ),
                (
                    (axes[1], axes[2], TranslateHandle::PlaneYZ.id()),
                    (1usize, 2usize),
                ),
            ] {
                if !self.plane_allowed_by_mask(plane_axes) {
                    continue;
                }
                let world = translate_plane_quad_world(origin, u, v, off, size);
                let Some(p) =
                    project_quad(view_projection, viewport, world, self.config.depth_range)
                else {
                    continue;
                };
                let alpha = self.plane_visibility_alpha(view_projection, viewport, world);
                if alpha <= 0.01 {
                    continue;
                }

                let edge_d = PickConvexQuad2d { points: p }.edge_distance(cursor);
                let r = self.config.pick_radius_px * alpha.sqrt();
                if edge_d <= r {
                    consider(handle, (edge_d + 0.9) / alpha.max(0.05));
                }
            }
        }

        best
    }

    fn pick_scale_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        include_axes: bool,
        include_uniform: bool,
        include_planes: bool,
    ) -> Option<PickHit> {
        let pv = self.state.part_visuals;
        let length_world = size_length_world;

        // Picking priority ladder (editor UX):
        // 1) Uniform scale at origin (within radius, always win)
        // 2) Axis end boxes (match visuals; avoid picking the entire shaft)
        let mut best: Option<PickHit> = None;
        let mut consider = |handle: HandleId, score: f32| {
            if !score.is_finite() {
                return;
            }
            match best {
                Some(best) if score >= best.score => {}
                _ => best = Some(PickHit { handle, score }),
            }
        };

        // Uniform scale at the origin.
        if include_uniform {
            if let Some(p0) =
                project_point(view_projection, viewport, origin, self.config.depth_range)
            {
                let r = self.config.pick_radius_px.max(6.0);
                if let Some(d) = (PickCircle2d {
                    center: p0.screen,
                    radius: r,
                })
                .hit_distance(cursor)
                {
                    return Some(PickHit {
                        handle: ScaleHandle::Uniform.id(),
                        score: d,
                    });
                }
            }
        }

        // Axis scaling handles.
        if include_axes {
            let (u, v) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
                    .map(plane_basis)
                    .unwrap_or((Vec3::X, Vec3::Y));
            for &((axis_dir, handle), axis_index) in &[
                ((axes[0], ScaleHandle::AxisX.id()), 0usize),
                ((axes[1], ScaleHandle::AxisY.id()), 1usize),
                ((axes[2], ScaleHandle::AxisZ.id()), 2usize),
            ] {
                if self.axis_is_masked(axis_index) {
                    continue;
                }
                let alpha = self.axis_visibility_alpha(
                    view_projection,
                    viewport,
                    origin,
                    axis_dir,
                    length_world,
                );
                if alpha <= 0.01 {
                    continue;
                }

                let end = origin + axis_dir * length_world;
                let half = length_world * pv.scale_axis_end_box_half_fraction.max(0.0);
                let quad_world = [
                    end + (-u - v) * half,
                    end + (u - v) * half,
                    end + (u + v) * half,
                    end + (-u + v) * half,
                ];
                let Some(p) = project_quad(
                    view_projection,
                    viewport,
                    quad_world,
                    self.config.depth_range,
                ) else {
                    continue;
                };
                let quad = PickConvexQuad2d { points: p };
                let inside = quad.contains(cursor);
                let edge_d = quad.edge_distance(cursor);
                if inside {
                    consider(handle, 0.0);
                } else {
                    let r = self.config.pick_radius_px * alpha.sqrt();
                    if edge_d <= r {
                        consider(handle, edge_d / alpha.max(0.05));
                    }
                }
            }
        }

        if include_planes {
            let off = length_world * pv.scale_plane_offset_fraction.max(0.0);
            let size = length_world * pv.scale_plane_size_fraction.max(0.0);
            for &((u, v, handle), plane_axes) in &[
                (
                    (axes[0], axes[1], ScaleHandle::PlaneXY.id()),
                    (0usize, 1usize),
                ),
                (
                    (axes[0], axes[2], ScaleHandle::PlaneXZ.id()),
                    (0usize, 2usize),
                ),
                (
                    (axes[1], axes[2], ScaleHandle::PlaneYZ.id()),
                    (1usize, 2usize),
                ),
            ] {
                if !self.plane_allowed_by_mask(plane_axes) {
                    continue;
                }
                let world = translate_plane_quad_world(origin, u, v, off, size);
                let Some(p) =
                    project_quad(view_projection, viewport, world, self.config.depth_range)
                else {
                    continue;
                };
                let alpha = self.plane_visibility_alpha(view_projection, viewport, world);
                if alpha <= 0.01 {
                    continue;
                }

                let quad = PickConvexQuad2d { points: p };
                let inside = quad.contains(cursor);
                let edge_d = quad.edge_distance(cursor);
                if inside {
                    consider(handle, 0.20 / alpha.max(0.05));
                } else {
                    let r = self.config.pick_radius_px * alpha.sqrt();
                    if edge_d <= r {
                        consider(handle, (edge_d + 0.9) / alpha.max(0.05));
                    }
                }
            }
        }

        best
    }

    fn mixed_translate_axis_tip_intent(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        hit: PickHit,
    ) -> bool {
        if !matches!(hit.handle.0, 1 | 2 | 3) {
            return false;
        }
        let axis_dir = axes[(hit.handle.0.saturating_sub(1)) as usize].normalize_or_zero();
        if axis_dir.length_squared() == 0.0 {
            return false;
        }

        let axis_tip_len = size_length_world * self.translate_axis_tip_scale();
        let tip_world = origin + axis_dir * axis_tip_len;
        let Some(tip) = project_point(
            view_projection,
            viewport,
            tip_world,
            self.config.depth_range,
        ) else {
            return false;
        };
        let d = (cursor - tip.screen).length();
        if !d.is_finite() {
            return false;
        }
        let r = self.config.pick_radius_px.max(6.0)
            * self.config.pick_policy.translate_axis_tip_radius_scale;
        d <= r
    }

    fn mixed_pick_band(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        rotate_present: bool,
        allow_translate_axis_tip_intent: bool,
        hit: PickHit,
        kind: GizmoMode,
    ) -> MixedPickBand {
        match kind {
            GizmoMode::Translate => {
                if hit.handle == TranslateHandle::Screen.id() {
                    return MixedPickBand::TranslateCenter;
                }
                if matches!(hit.handle.0, 4 | 5 | 6)
                    && hit.score <= self.config.pick_policy.translate_plane_inside_score_max
                {
                    return MixedPickBand::TranslatePlaneInside;
                }
                if rotate_present
                    && allow_translate_axis_tip_intent
                    && self.mixed_translate_axis_tip_intent(
                        view_projection,
                        viewport,
                        origin,
                        cursor,
                        axes,
                        size_length_world,
                        hit,
                    )
                {
                    return MixedPickBand::TranslateAxisTipIntent;
                }
                MixedPickBand::Default
            }
            GizmoMode::Scale => {
                if hit.score <= self.config.pick_policy.scale_solid_inside_score_max {
                    return MixedPickBand::ScaleSolidInside;
                }
                if Self::bounds_handle_from_id(hit.handle).is_some()
                    && hit.score <= self.config.pick_policy.bounds_inside_score_max
                {
                    return MixedPickBand::ScaleSolidInside;
                }
                MixedPickBand::Default
            }
            GizmoMode::Rotate | GizmoMode::Universal => MixedPickBand::Default,
        }
    }

    fn pick_best_mixed_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
        allow_translate_axis_tip_intent: bool,
        rotate: Option<(PickHit, GizmoMode)>,
        scale: Option<(PickHit, GizmoMode)>,
        translate: Option<(PickHit, GizmoMode)>,
    ) -> Option<(PickHit, GizmoMode)> {
        let rotate_present = rotate.is_some();

        let mut best: Option<(PickHit, GizmoMode, MixedPickBand, f32, u8)> = None;
        let mut consider = |candidate: Option<(PickHit, GizmoMode)>| {
            let Some((hit, kind)) = candidate else {
                return;
            };
            if !hit.score.is_finite() {
                return;
            }

            let band = self.mixed_pick_band(
                view_projection,
                viewport,
                origin,
                cursor,
                axes,
                size_length_world,
                rotate_present,
                allow_translate_axis_tip_intent,
                hit,
                kind,
            );
            let kind_priority: u8 = match kind {
                GizmoMode::Rotate => 0,
                GizmoMode::Scale => 1,
                GizmoMode::Translate => 2,
                GizmoMode::Universal => 3,
            };

            match best {
                Some((_best_hit, _best_kind, best_band, best_score, best_kind_pri)) => {
                    if band < best_band
                        || (band == best_band && (hit.score < best_score))
                        || (band == best_band
                            && hit.score == best_score
                            && kind_priority < best_kind_pri)
                    {
                        best = Some((hit, kind, band, hit.score, kind_priority));
                    }
                }
                None => best = Some((hit, kind, band, hit.score, kind_priority)),
            }
        };

        consider(rotate);
        consider(scale);
        consider(translate);
        best.map(|(hit, kind, _, _, _)| (hit, kind))
    }

    fn pick_universal_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Option<(PickHit, GizmoMode)> {
        let translate = self
            .pick_translate_handle(
                view_projection,
                viewport,
                origin,
                cursor,
                axes,
                size_length_world,
                true,
                true,
                true,
                self.config.universal_includes_translate_depth,
            )
            .map(|h| (h, GizmoMode::Translate));
        let rotate = self
            .pick_rotate_axis(
                view_projection,
                viewport,
                origin,
                cursor,
                axes,
                size_length_world,
            )
            .map(|h| (h, GizmoMode::Rotate));
        let scale = self
            .config
            .universal_includes_scale
            .then(|| {
                self.pick_scale_handle(
                    view_projection,
                    viewport,
                    origin,
                    cursor,
                    axes,
                    size_length_world,
                    true,
                    false,
                    false,
                )
            })
            .flatten()
            .map(|h| (h, GizmoMode::Scale));

        self.pick_best_mixed_handle(
            view_projection,
            viewport,
            origin,
            cursor,
            axes,
            size_length_world,
            true,
            rotate,
            scale,
            translate,
        )
    }

    fn pick_operation_mask_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes_flipped: [Vec3; 3],
        axes_raw: [Vec3; 3],
        size_length_world: f32,
        targets: &[GizmoTarget3d],
    ) -> Option<(PickHit, GizmoMode)> {
        let ops = self.effective_ops();

        let translate_enabled = ops.intersects(GizmoOps::translate_all());
        let translate = translate_enabled
            .then(|| {
                self.pick_translate_handle(
                    view_projection,
                    viewport,
                    origin,
                    cursor,
                    axes_flipped,
                    size_length_world,
                    ops.contains(GizmoOps::translate_axis()),
                    ops.contains(GizmoOps::translate_plane()),
                    ops.contains(GizmoOps::translate_view()),
                    ops.contains(GizmoOps::translate_depth()),
                )
            })
            .flatten()
            .map(|h| (h, GizmoMode::Translate));

        let rotate_enabled = ops.intersects(GizmoOps::rotate_all());
        let rotate = rotate_enabled
            .then(|| {
                self.pick_rotate_axis(
                    view_projection,
                    viewport,
                    origin,
                    cursor,
                    axes_flipped,
                    size_length_world,
                )
            })
            .flatten()
            .map(|h| (h, GizmoMode::Rotate));

        let scale_enabled = ops.intersects(GizmoOps::scale_all());
        let scale_pickers_enabled = ops.intersects(
            GizmoOps::scale_axis() | GizmoOps::scale_plane() | GizmoOps::scale_uniform(),
        );
        let bounds_enabled = ops.contains(GizmoOps::scale_bounds());

        let scale = scale_enabled
            .then(|| {
                let scale = scale_pickers_enabled
                    .then(|| {
                        self.pick_scale_handle(
                            view_projection,
                            viewport,
                            origin,
                            cursor,
                            axes_flipped,
                            size_length_world,
                            ops.contains(GizmoOps::scale_axis()),
                            ops.contains(GizmoOps::scale_uniform()),
                            ops.contains(GizmoOps::scale_plane()),
                        )
                    })
                    .flatten()
                    .map(|h| (h, 1usize));

                if let Some((hit, _)) = scale {
                    if hit.handle == ScaleHandle::Uniform.id() {
                        return Some((hit, GizmoMode::Scale));
                    }
                }

                let bounds = bounds_enabled
                    .then(|| {
                        self.pick_bounds_handle(
                            view_projection,
                            viewport,
                            origin,
                            cursor,
                            axes_raw,
                            size_length_world,
                            targets,
                        )
                    })
                    .flatten()
                    .map(|h| (h, 0usize));

                // Bounds handles are explicit solid affordances. If the cursor is inside a bounds
                // handle, it should win over other scaling candidates that may overlap in
                // projection (axis end boxes, plane edges, etc).
                if let Some((hit, _)) = bounds {
                    if hit.score <= self.config.pick_policy.bounds_inside_score_max {
                        return Some((hit, GizmoMode::Scale));
                    }
                }

                let mut best: Option<(PickHit, usize)> = None;
                let mut consider = |cand: Option<(PickHit, usize)>| {
                    let Some((hit, pri)) = cand else {
                        return;
                    };
                    if !hit.score.is_finite() {
                        return;
                    }
                    match best {
                        Some((best_hit, best_pri)) => {
                            if hit.score < best_hit.score
                                || (hit.score == best_hit.score && pri < best_pri)
                            {
                                best = Some((hit, pri));
                            }
                        }
                        None => best = Some((hit, pri)),
                    }
                };

                consider(bounds);
                consider(scale);
                best.map(|(h, _)| (h, GizmoMode::Scale))
            })
            .flatten();

        self.pick_best_mixed_handle(
            view_projection,
            viewport,
            origin,
            cursor,
            axes_flipped,
            size_length_world,
            ops.contains(GizmoOps::translate_axis()),
            rotate,
            scale,
            translate,
        )
    }

    fn pick_scale_or_bounds_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes_flipped: [Vec3; 3],
        axes_raw: [Vec3; 3],
        size_length_world: f32,
        targets: &[GizmoTarget3d],
    ) -> Option<PickHit> {
        let scale = self
            .pick_scale_handle(
                view_projection,
                viewport,
                origin,
                cursor,
                axes_flipped,
                size_length_world,
                true,
                true,
                true,
            )
            .map(|h| (h, 1usize));

        if let Some((hit, _)) = scale {
            if hit.handle == ScaleHandle::Uniform.id() {
                return Some(hit);
            }
        }

        let bounds = self
            .config
            .show_bounds
            .then(|| {
                self.pick_bounds_handle(
                    view_projection,
                    viewport,
                    origin,
                    cursor,
                    axes_raw,
                    size_length_world,
                    targets,
                )
            })
            .flatten()
            .map(|h| (h, 0usize));

        // Bounds handles are explicit solid affordances. If the cursor is inside a bounds handle,
        // it should win over axis end-box scaling that may overlap in projection.
        if let Some((hit, _)) = bounds {
            if hit.score <= self.config.pick_policy.bounds_inside_score_max {
                return Some(hit);
            }
        }

        let mut best: Option<(PickHit, usize)> = None;
        let mut consider = |cand: Option<(PickHit, usize)>| {
            let Some((hit, pri)) = cand else {
                return;
            };
            if !hit.score.is_finite() {
                return;
            }
            match best {
                Some((best_hit, best_pri)) => {
                    if hit.score < best_hit.score || (hit.score == best_hit.score && pri < best_pri)
                    {
                        best = Some((hit, pri));
                    }
                }
                None => best = Some((hit, pri)),
            }
        };

        consider(bounds);
        consider(scale);
        best.map(|(h, _)| h)
    }

    fn pick_bounds_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes_raw: [Vec3; 3],
        size_length_world: f32,
        targets: &[GizmoTarget3d],
    ) -> Option<PickHit> {
        if targets.is_empty() {
            return None;
        }
        let basis = [
            axes_raw[0].normalize_or_zero(),
            axes_raw[1].normalize_or_zero(),
            axes_raw[2].normalize_or_zero(),
        ];
        if basis.iter().any(|v| v.length_squared() == 0.0) {
            return None;
        }

        let (min_local, max_local) = self.bounds_min_max_local(
            view_projection,
            viewport,
            origin,
            basis,
            size_length_world,
            targets,
        );
        let center_local = (min_local + max_local) * 0.5;

        let Some(view_dir) =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        else {
            return None;
        };
        let (u, v) = plane_basis(view_dir);

        let handle_half_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.bounds_handle_size_px.max(1.0),
        )
        .unwrap_or(1.0)
        .max(1e-6)
            * 0.5;

        let mut best: Option<PickHit> = None;
        let mut consider = |handle: HandleId, score: f32| {
            if !score.is_finite() {
                return;
            }
            match best {
                Some(best) if score >= best.score => {}
                _ => best = Some(PickHit { handle, score }),
            }
        };

        // Corner handles.
        for z_max in [false, true] {
            for y_max in [false, true] {
                for x_max in [false, true] {
                    let local = Vec3::new(
                        if x_max { max_local.x } else { min_local.x },
                        if y_max { max_local.y } else { min_local.y },
                        if z_max { max_local.z } else { min_local.z },
                    );
                    let world =
                        origin + basis[0] * local.x + basis[1] * local.y + basis[2] * local.z;
                    let quad_world = [
                        world + (-u - v) * handle_half_world,
                        world + (u - v) * handle_half_world,
                        world + (u + v) * handle_half_world,
                        world + (-u + v) * handle_half_world,
                    ];
                    let Some(p) = project_quad(
                        view_projection,
                        viewport,
                        quad_world,
                        self.config.depth_range,
                    ) else {
                        continue;
                    };
                    let quad = PickConvexQuad2d { points: p };
                    let inside = quad.contains(cursor);
                    let edge_d = quad.edge_distance(cursor);
                    let handle = Self::bounds_corner_id(x_max, y_max, z_max);
                    if inside {
                        consider(handle, 0.0);
                    } else if edge_d <= self.config.pick_radius_px {
                        consider(handle, edge_d);
                    }
                }
            }
        }

        // Face handles.
        for axis in 0..3 {
            for &max_side in &[false, true] {
                let mut local = center_local;
                local[axis] = if max_side {
                    max_local[axis]
                } else {
                    min_local[axis]
                };
                let world = origin + basis[0] * local.x + basis[1] * local.y + basis[2] * local.z;
                let quad_world = [
                    world + (-u - v) * handle_half_world,
                    world + (u - v) * handle_half_world,
                    world + (u + v) * handle_half_world,
                    world + (-u + v) * handle_half_world,
                ];
                let Some(p) = project_quad(
                    view_projection,
                    viewport,
                    quad_world,
                    self.config.depth_range,
                ) else {
                    continue;
                };
                let quad = PickConvexQuad2d { points: p };
                let inside = quad.contains(cursor);
                let edge_d = quad.edge_distance(cursor);
                let handle = Self::bounds_face_id(axis, max_side);
                if inside {
                    consider(handle, 0.25);
                } else if edge_d <= self.config.pick_radius_px {
                    consider(handle, edge_d + 0.8);
                }
            }
        }

        best
    }

    fn pick_rotate_axis(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
        size_length_world: f32,
    ) -> Option<PickHit> {
        let (include_axis, include_view, include_arcball) =
            if let Some(mask) = self.config.operation_mask {
                (
                    mask.contains(GizmoOps::rotate_axis()),
                    mask.contains(GizmoOps::rotate_view()),
                    mask.contains(GizmoOps::rotate_arcball()),
                )
            } else {
                (
                    true,
                    self.config.show_view_axis_ring,
                    self.config.show_arcball,
                )
            };

        let radius_world = size_length_world.max(0.0);

        let segments: usize = 64;
        let mut best_axis: Option<PickHit> = None;

        if include_axis {
            for &((axis_dir, handle), axis_index) in &[
                ((axes[0], HandleId(1)), 0usize),
                ((axes[1], HandleId(2)), 1usize),
                ((axes[2], HandleId(3)), 2usize),
            ] {
                if self.axis_is_masked(axis_index) {
                    continue;
                }
                let axis_dir = axis_dir.normalize_or_zero();
                if axis_dir.length_squared() == 0.0 {
                    continue;
                }
                let alpha =
                    self.rotate_ring_visibility_alpha(view_projection, viewport, origin, axis_dir);
                if alpha <= 0.01 {
                    continue;
                }
                let (u, v) = plane_basis(axis_dir);

                let mut prev_world = origin + u * radius_world;
                let mut prev = match project_point(
                    view_projection,
                    viewport,
                    prev_world,
                    self.config.depth_range,
                ) {
                    Some(p) => p,
                    None => continue,
                };

                for i in 1..=segments {
                    let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                    let world = origin + (u * t.cos() + v * t.sin()) * radius_world;
                    let Some(p) =
                        project_point(view_projection, viewport, world, self.config.depth_range)
                    else {
                        prev_world = world;
                        continue;
                    };

                    if prev.inside_clip && p.inside_clip {
                        let r = self.config.pick_radius_px * alpha.sqrt();
                        if let Some(d) = (PickSegmentCapsule2d {
                            a: prev.screen,
                            b: p.screen,
                            radius: r,
                        })
                        .hit_distance(cursor)
                        {
                            match best_axis {
                                Some(best) if d >= best.score => {}
                                _ => best_axis = Some(PickHit { handle, score: d }),
                            }
                        }
                    }

                    prev = p;
                    prev_world = world;
                }
            }
        }

        let mut view_hit: Option<PickHit> = None;
        if include_view {
            if let Some(view_dir) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            {
                let axis_dir = view_dir.normalize_or_zero();
                if axis_dir.length_squared() > 0.0 {
                    let (u, v) = plane_basis(axis_dir);
                    let handle = Self::ROTATE_VIEW_HANDLE;
                    let r = (radius_world * self.config.view_axis_ring_radius_scale).max(1e-6);

                    let mut prev = match project_point(
                        view_projection,
                        viewport,
                        origin + u * r,
                        self.config.depth_range,
                    ) {
                        Some(p) => p,
                        None => return best_axis,
                    };

                    for i in 1..=segments {
                        let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                        let world = origin + (u * t.cos() + v * t.sin()) * r;
                        let Some(p) = project_point(
                            view_projection,
                            viewport,
                            world,
                            self.config.depth_range,
                        ) else {
                            continue;
                        };

                        if prev.inside_clip && p.inside_clip {
                            if let Some(d) = (PickSegmentCapsule2d {
                                a: prev.screen,
                                b: p.screen,
                                radius: self.config.pick_radius_px,
                            })
                            .hit_distance(cursor)
                            {
                                match view_hit {
                                    Some(best) if d >= best.score => {}
                                    _ => view_hit = Some(PickHit { handle, score: d }),
                                };
                            }
                        }
                        prev = p;
                    }
                }
            }
        }

        let ring_hit = match (best_axis, view_hit) {
            (Some(axis), Some(view)) => {
                // View ring is an outer, always-on-top affordance. Make it slightly easier to hit
                // while preventing it from stealing clearly-intended axis ring drags.
                //
                // Rule of thumb:
                // - If the cursor is close to an axis ring (strong intent), axis wins.
                // - Otherwise the view ring can win only if it is meaningfully closer.
                let axis_strong = axis.score <= self.config.pick_radius_px.max(1.0) * 0.35;
                let view_score = (view.score - 0.15).max(0.0);
                if !axis_strong && view_score + 0.75 < axis.score {
                    Some(PickHit {
                        handle: view.handle,
                        score: view_score,
                    })
                } else {
                    Some(axis)
                }
            }
            (Some(axis), None) => Some(axis),
            (None, Some(view)) => Some(view),
            (None, None) => None,
        };
        if ring_hit.is_some() {
            return ring_hit;
        }

        if include_arcball {
            let center = project_point(view_projection, viewport, origin, self.config.depth_range)?;
            let r = match self.config.size_policy {
                GizmoSizePolicy::ConstantPixels => {
                    self.config.size_px * self.config.arcball_radius_scale
                }
                GizmoSizePolicy::PixelsClampedBySelectionBounds { .. }
                | GizmoSizePolicy::SelectionBounds { .. } => {
                    let r_world = (radius_world * self.config.arcball_radius_scale).max(1e-6);
                    // The arcball circle is camera-facing, so any in-plane direction yields a
                    // stable projected radius.
                    let (u, _) = view_dir_at_origin(
                        view_projection,
                        viewport,
                        origin,
                        self.config.depth_range,
                    )
                    .map(plane_basis)
                    .unwrap_or((Vec3::X, Vec3::Y));
                    axis_segment_len_px(
                        view_projection,
                        viewport,
                        origin,
                        self.config.depth_range,
                        u,
                        r_world,
                    )
                    .unwrap_or(0.0)
                }
            }
            .max(self.config.pick_radius_px.max(6.0));
            if let Some(d) = (PickCircle2d {
                center: center.screen,
                radius: r,
            })
            .hit_distance(cursor)
            {
                return Some(PickHit {
                    handle: Self::ROTATE_ARCBALL_HANDLE,
                    score: 10.0 + (d / r.max(1.0)),
                });
            }
        }

        None
    }
}

fn axis_for_handle(handle: HandleId) -> (Vec3, usize) {
    match handle.0 {
        1 => (Vec3::X, 0),
        2 => (Vec3::Y, 1),
        3 => (Vec3::Z, 2),
        _ => (Vec3::X, 0),
    }
}

fn plane_basis(normal: Vec3) -> (Vec3, Vec3) {
    let n = normal.normalize_or_zero();
    let a = if n.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let u = n.cross(a).normalize_or_zero();
    let v = n.cross(u).normalize_or_zero();
    (u, v)
}

fn angle_on_plane(origin: Vec3, point: Vec3, axis_dir: Vec3, u: Vec3, v: Vec3) -> Option<f32> {
    let axis = axis_dir.normalize_or_zero();
    if axis.length_squared() == 0.0 {
        return None;
    }

    let w = point - origin;
    let w_plane = w - axis * w.dot(axis);
    if w_plane.length_squared() < 1e-6 {
        return None;
    }
    let w_plane = w_plane.normalize();
    let x = w_plane.dot(u);
    let y = w_plane.dot(v);
    Some(y.atan2(x))
}

fn wrap_angle(mut a: f32) -> f32 {
    // Map to [-pi, pi] for stable incremental deltas.
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

fn quat_axis_angle(q: Quat) -> (Vec3, f32) {
    let q = q.normalize();
    let (axis, angle) = q.to_axis_angle();
    let axis = axis.normalize_or_zero();
    if axis.length_squared() == 0.0 || !angle.is_finite() {
        (Vec3::Z, 0.0)
    } else {
        (axis, angle)
    }
}

fn snap_quat_to_angle_step(total: Quat, step: f32) -> Quat {
    let step = step.max(1e-6);
    let (axis, angle) = quat_axis_angle(total);
    if angle.abs() < 1e-6 {
        return Quat::IDENTITY;
    }
    let snapped = (angle / step).round() * step;
    if snapped.abs() < 1e-6 {
        Quat::IDENTITY
    } else {
        Quat::from_axis_angle(axis, snapped).normalize()
    }
}

fn snap_bounds_extent_factor(start_extent: f32, factor: f32, step: f32) -> f32 {
    if !start_extent.is_finite() || start_extent <= 0.0 {
        return factor;
    }
    if !factor.is_finite() {
        return factor;
    }
    if !step.is_finite() || step <= 0.0 {
        return factor;
    }

    let extent = start_extent.max(1e-6);
    let desired_extent = (extent * factor).max(0.0);
    let snapped_extent = (desired_extent / step).round() * step;
    (snapped_extent / extent).max(0.01)
}

fn ray_plane_intersect(ray: Ray3d, plane_point: Vec3, plane_normal: Vec3) -> Option<Vec3> {
    let n = plane_normal.normalize_or_zero();
    if n.length_squared() == 0.0 {
        return None;
    }
    let denom = n.dot(ray.dir);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = n.dot(plane_point - ray.origin) / denom;
    if !t.is_finite() || t < 0.0 {
        return None;
    }
    Some(ray.origin + ray.dir * t)
}

fn axis_drag_plane_normal(
    view_projection: Mat4,
    viewport: ViewportRect,
    depth: DepthRange,
    origin: Vec3,
    axis_dir: Vec3,
) -> Option<Vec3> {
    // Best-practice axis dragging: intersect the cursor ray against a plane that:
    // - passes through the gizmo origin
    // - contains the axis direction
    // - is as "camera-facing" as possible for stability
    //
    // A robust choice is: plane normal = axis x view_dir_at_origin.
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let view_ray = ray_from_screen(view_projection, viewport, p0.screen, depth)?;

    let axis = axis_dir.normalize_or_zero();
    if axis.length_squared() == 0.0 {
        return None;
    }
    let view_dir = view_ray.dir.normalize_or_zero();

    let mut n = axis.cross(view_dir);
    if n.length_squared() < 1e-6 {
        // Degenerate when viewing straight down the axis; pick a stable fallback basis.
        n = axis.cross(Vec3::Y);
        if n.length_squared() < 1e-6 {
            n = axis.cross(Vec3::X);
        }
    }
    let plane_normal = n.normalize_or_zero();
    (plane_normal.length_squared() > 0.0).then_some(plane_normal)
}

fn axis_drag_plane_normal_facing_camera(
    view_projection: Mat4,
    viewport: ViewportRect,
    depth: DepthRange,
    origin: Vec3,
    axis_dir: Vec3,
) -> Option<Vec3> {
    // Camera-facing variant for axis dragging:
    // - plane passes through origin
    // - contains the axis direction (plane normal ⟂ axis)
    // - normal is as close to the view direction as possible (stable ray-plane intersection)
    //
    // A good choice is: n = axis × (axis × view_dir) = axis*(axis·view_dir) - view_dir.
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let view_ray = ray_from_screen(view_projection, viewport, p0.screen, depth)?;

    let axis = axis_dir.normalize_or_zero();
    if axis.length_squared() == 0.0 {
        return None;
    }
    let view_dir = view_ray.dir.normalize_or_zero();

    let mut n = axis.cross(axis.cross(view_dir));
    if n.length_squared() < 1e-6 {
        // Degenerate when viewing straight down the axis; pick a stable fallback basis.
        n = axis.cross(Vec3::Y);
        if n.length_squared() < 1e-6 {
            n = axis.cross(Vec3::X);
        }
    }
    let plane_normal = n.normalize_or_zero();
    (plane_normal.length_squared() > 0.0).then_some(plane_normal)
}

fn distance_point_to_segment_px(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = if ab.length_squared() > 0.0 {
        ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let q = a + ab * t;
    (p - q).length()
}

fn axis_segment_len_px(
    view_projection: Mat4,
    viewport: ViewportRect,
    origin: Vec3,
    depth: DepthRange,
    axis_dir: Vec3,
    axis_len_world: f32,
) -> Option<f32> {
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let p1 = project_point(
        view_projection,
        viewport,
        origin + axis_dir * axis_len_world,
        depth,
    )?;
    let d = (p1.screen - p0.screen).length();
    d.is_finite().then_some(d)
}

fn quad_area_px2(q: [Vec2; 4]) -> f32 {
    // Shoelace formula (absolute area of the convex quad).
    let mut a = 0.0f32;
    for i in 0..4 {
        let p0 = q[i];
        let p1 = q[(i + 1) % 4];
        a += p0.x * p1.y - p1.x * p0.y;
    }
    (a * 0.5).abs()
}

fn mix_alpha(mut c: Color, a: f32) -> Color {
    c.a = (c.a * a).clamp(0.0, 1.0);
    c
}

fn translate_plane_quad_world(origin: Vec3, u: Vec3, v: Vec3, off: f32, size: f32) -> [Vec3; 4] {
    let u = u.normalize_or_zero();
    let v = v.normalize_or_zero();
    let p0 = origin + u * off + v * off;
    let p1 = origin + u * (off + size) + v * off;
    let p2 = origin + u * (off + size) + v * (off + size);
    let p3 = origin + u * off + v * (off + size);
    [p0, p1, p2, p3]
}

fn project_quad(
    view_projection: Mat4,
    viewport: ViewportRect,
    world: [Vec3; 4],
    depth: DepthRange,
) -> Option<[Vec2; 4]> {
    let mut out = [Vec2::ZERO; 4];
    for (i, w) in world.iter().enumerate() {
        let p = project_point(view_projection, viewport, *w, depth)?;
        if !p.screen.is_finite() {
            return None;
        }
        out[i] = p.screen;
    }
    Some(out)
}

fn view_dir_at_origin(
    view_projection: Mat4,
    viewport: ViewportRect,
    origin: Vec3,
    depth: DepthRange,
) -> Option<Vec3> {
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let ray = ray_from_screen(view_projection, viewport, p0.screen, depth)?;
    Some(ray.dir.normalize_or_zero())
}

fn origin_z01(
    view_projection: Mat4,
    viewport: ViewportRect,
    origin: Vec3,
    depth: DepthRange,
) -> Option<f32> {
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let z01 = match depth {
        DepthRange::ZeroToOne => p0.ndc_z,
        DepthRange::NegOneToOne => (p0.ndc_z + 1.0) * 0.5,
    };
    Some(z01.clamp(0.0, 1.0))
}

fn input_precision(precision: f32) -> f32 {
    if !precision.is_finite() {
        return 1.0;
    }
    precision.clamp(0.01, 100.0)
}

fn axis_length_world(
    view_projection: Mat4,
    viewport: ViewportRect,
    origin: Vec3,
    depth: DepthRange,
    desired_px: f32,
) -> Option<f32> {
    let p0 = project_point(view_projection, viewport, origin, depth)?;
    let z01 = match depth {
        DepthRange::ZeroToOne => p0.ndc_z,
        DepthRange::NegOneToOne => (p0.ndc_z + 1.0) * 0.5,
    }
    .clamp(0.0, 1.0);

    let p_world = unproject_point(view_projection, viewport, p0.screen, depth, z01)?;
    let p_world_dx = unproject_point(
        view_projection,
        viewport,
        p0.screen + Vec2::new(1.0, 0.0),
        depth,
        z01,
    )?;
    let d = (p_world_dx - p_world).length();
    if !d.is_finite() || d <= 1e-7 {
        return None;
    }
    Some(d * desired_px.max(0.0))
}

fn translate_constraint_for_handle(
    view_projection: Mat4,
    viewport: ViewportRect,
    depth: DepthRange,
    origin: Vec3,
    handle: HandleId,
    axes: [Vec3; 3],
) -> Option<TranslateConstraint> {
    let axes = [
        axes[0].normalize_or_zero(),
        axes[1].normalize_or_zero(),
        axes[2].normalize_or_zero(),
    ];
    match handle.0 {
        1 => Some(TranslateConstraint::Axis { axis_dir: axes[0] }),
        2 => Some(TranslateConstraint::Axis { axis_dir: axes[1] }),
        3 => Some(TranslateConstraint::Axis { axis_dir: axes[2] }),
        4 => plane_constraint(axes[0], axes[1]),
        5 => plane_constraint(axes[0], axes[2]),
        6 => plane_constraint(axes[1], axes[2]),
        10 => {
            let view_dir = view_dir_at_origin(view_projection, viewport, origin, depth)?;
            let (u, v) = plane_basis(view_dir);
            let n = view_dir.normalize_or_zero();
            (n.length_squared() > 0.0).then_some(TranslateConstraint::Plane { u, v, normal: n })
        }
        11 => {
            let view_dir = view_dir_at_origin(view_projection, viewport, origin, depth)?;
            let n = view_dir.normalize_or_zero();
            (n.length_squared() > 0.0).then_some(TranslateConstraint::Dolly { view_dir: n })
        }
        _ => None,
    }
}

fn plane_constraint(u: Vec3, v: Vec3) -> Option<TranslateConstraint> {
    let u = u.normalize_or_zero();
    let v = v.normalize_or_zero();
    if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
        return None;
    }
    let n = u.cross(v).normalize_or_zero();
    (n.length_squared() > 0.0).then_some(TranslateConstraint::Plane { u, v, normal: n })
}

mod draw_bounds;
mod draw_rotate;
mod draw_scale;
mod draw_translate;

#[cfg(test)]
mod tests;
