use fret_core::Color;
use glam::Vec3;

use crate::math::DepthRange;

use super::{DepthMode, GizmoHandedness, GizmoMode, GizmoOrientation, GizmoPivotMode};

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
    /// When `Some`, this overrides `mode` and the `show_*`/`universal_includes_*` toggles that
    /// imply optional sub-modes. The mask controls both drawing and picking for the enabled
    /// sub-operations.
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
    /// When `true`, includes a view-axis rotation ring (camera-facing) in `GizmoMode::Rotate`.
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
    /// When `true`, includes a free-rotation arcball (trackball) in `GizmoMode::Rotate`.
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
    /// When `true`, `GizmoMode::Universal` includes the view-axis rotate ring.
    ///
    /// This corresponds to ImGuizmo's `ROTATE_SCREEN` behavior and transform-gizmo's `RotateView`.
    pub universal_includes_rotate_view_ring: bool,
    /// When `true`, `GizmoMode::Universal` includes the arcball rotate handle.
    ///
    /// ImGuizmo's "universal" mode does not include arcball (it is a transform-gizmo-style
    /// affordance), so this is opt-in by default.
    pub universal_includes_arcball: bool,
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
            universal_includes_rotate_view_ring: true,
            universal_includes_arcball: false,
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
        Self {
            size_px: 112.0,
            pick_radius_px: 12.0,
            line_thickness_px: 8.0,
            bounds_handle_size_px: 14.0,
            ..Self::default()
        }
    }

    /// Scales pixel-based knobs to match the cursor's coordinate units.
    ///
    /// The gizmo config stores many values in "pixels" (hit radii, thickness, drag thresholds).
    /// Those pixels are expected to match the units of `GizmoInput.cursor_px` provided by the host.
    ///
    /// - If the host feeds window-local logical pixels ("screen px"), pass `1.0`.
    /// - If the host feeds physical pixels, pass the platform scale factor (pixels-per-point / DPR).
    /// - If the host feeds viewport render-target pixels (typical for `ViewportSurface`), pass the
    ///   target-pixels-per-screen-pixel scale derived from the viewport mapping.
    pub fn scale_for_cursor_units_per_screen_px(mut self, cursor_units_per_screen_px: f32) -> Self {
        let s = cursor_units_per_screen_px.clamp(0.1, 16.0);
        self.size_px *= s;
        self.pick_radius_px *= s;
        self.line_thickness_px *= s;
        self.drag_start_threshold_px *= s;
        self.bounds_handle_size_px *= s;
        self
    }

    /// Backwards-compatible alias for [`Self::scale_for_cursor_units_per_screen_px`].
    pub fn scale_for_pixels_per_point(self, pixels_per_point: f32) -> Self {
        self.scale_for_cursor_units_per_screen_px(pixels_per_point)
    }
}
