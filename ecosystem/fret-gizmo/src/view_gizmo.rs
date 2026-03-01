use fret_core::Color;
use glam::{Mat4, Vec2, Vec3};

use crate::gizmo::{Aabb3, DepthMode, GizmoDrawList3d, Line3d, Triangle3d};
use crate::math::{DepthRange, ViewportRect, ray_from_screen, unproject_point};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewGizmoLabel {
    pub screen_px: Vec2,
    pub text: &'static str,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewGizmoAnchor {
    TopLeft,
    #[default]
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewGizmoProjection {
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewGizmoFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl ViewGizmoFace {
    pub fn normal(self) -> Vec3 {
        match self {
            ViewGizmoFace::PosX => Vec3::X,
            ViewGizmoFace::NegX => Vec3::NEG_X,
            ViewGizmoFace::PosY => Vec3::Y,
            ViewGizmoFace::NegY => Vec3::NEG_Y,
            ViewGizmoFace::PosZ => Vec3::Z,
            ViewGizmoFace::NegZ => Vec3::NEG_Z,
        }
    }

    pub fn view_dir(self) -> Vec3 {
        // This is a "look direction" (from camera towards the target), not a camera position.
        -self.normal()
    }

    pub fn default_up(self) -> Vec3 {
        match self {
            // Use +Y as the default up for side/front/back views.
            ViewGizmoFace::PosX
            | ViewGizmoFace::NegX
            | ViewGizmoFace::PosZ
            | ViewGizmoFace::NegZ => Vec3::Y,
            // For top/bottom views, use Z to avoid a degenerate basis.
            ViewGizmoFace::PosY => Vec3::Z,
            ViewGizmoFace::NegY => Vec3::NEG_Z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewGizmoSnap {
    /// Direction from the camera towards the pivot in world axes.
    ///
    /// Each component is in `{-1, 0, 1}` and at least one component is non-zero.
    pub view_dir: [i8; 3],
}

impl ViewGizmoSnap {
    pub const fn new(view_dir: [i8; 3]) -> Option<Self> {
        let ok = (view_dir[0] >= -1 && view_dir[0] <= 1)
            && (view_dir[1] >= -1 && view_dir[1] <= 1)
            && (view_dir[2] >= -1 && view_dir[2] <= 1)
            && !(view_dir[0] == 0 && view_dir[1] == 0 && view_dir[2] == 0);
        if ok { Some(Self { view_dir }) } else { None }
    }

    pub const fn from_face(face: ViewGizmoFace) -> Self {
        // Face normal points away from the cube center; view_dir points from the camera towards
        // the pivot, so it is `-normal`.
        match face {
            ViewGizmoFace::PosX => Self {
                view_dir: [-1, 0, 0],
            },
            ViewGizmoFace::NegX => Self {
                view_dir: [1, 0, 0],
            },
            ViewGizmoFace::PosY => Self {
                view_dir: [0, -1, 0],
            },
            ViewGizmoFace::NegY => Self {
                view_dir: [0, 1, 0],
            },
            ViewGizmoFace::PosZ => Self {
                view_dir: [0, 0, -1],
            },
            ViewGizmoFace::NegZ => Self {
                view_dir: [0, 0, 1],
            },
        }
    }

    pub fn face(self) -> Option<ViewGizmoFace> {
        match self.view_dir {
            [-1, 0, 0] => Some(ViewGizmoFace::PosX),
            [1, 0, 0] => Some(ViewGizmoFace::NegX),
            [0, -1, 0] => Some(ViewGizmoFace::PosY),
            [0, 1, 0] => Some(ViewGizmoFace::NegY),
            [0, 0, -1] => Some(ViewGizmoFace::PosZ),
            [0, 0, 1] => Some(ViewGizmoFace::NegZ),
            _ => None,
        }
    }

    pub fn view_dir_vec3(self) -> Vec3 {
        let v = Vec3::new(
            self.view_dir[0] as f32,
            self.view_dir[1] as f32,
            self.view_dir[2] as f32,
        );
        v.normalize_or_zero()
    }

    pub fn up_vec3(self) -> Vec3 {
        match self.view_dir {
            [0, -1, 0] => Vec3::Z,
            [0, 1, 0] => Vec3::NEG_Z,
            _ => Vec3::Y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewGizmoConfig {
    /// Screen-space anchor corner (viewport coordinates are top-left origin).
    pub anchor: ViewGizmoAnchor,
    /// Margin from the viewport edge to the cube's center in pixels.
    pub margin_px: Vec2,
    /// Approximate cube size in pixels.
    pub size_px: f32,
    /// Additional hit padding in pixels for easier interaction.
    pub pick_padding_px: f32,
    /// Local-space threshold used to resolve face/edge/corner snaps.
    ///
    /// Values near `1.0` require clicking very close to edges/corners, while values near `0.0`
    /// make edges/corners too easy to hit.
    pub snap_feature_threshold: f32,
    /// Drag distance threshold (in pixels) to treat the interaction as orbit rather than click.
    pub drag_threshold_px: f32,
    /// Orbit sensitivity expressed in radians per pixel.
    pub orbit_sensitivity_radians_per_px: f32,
    /// Radius of the center "projection toggle" button in pixels.
    ///
    /// Clicking this toggles perspective/orthographic in the host (ImGuizmo-style behavior).
    pub center_button_radius_px: f32,
    /// Depth range convention used by the camera projection.
    pub depth_range: DepthRange,
    /// Unprojected depth used to place the gizmo in front of the camera (`[0, 1]`).
    pub z01: f32,
    pub face_color: Color,
    pub edge_color: Color,
    pub hover_color: Color,
    pub x_color: Color,
    pub y_color: Color,
    pub z_color: Color,
}

impl Default for ViewGizmoConfig {
    fn default() -> Self {
        Self {
            anchor: ViewGizmoAnchor::default(),
            margin_px: Vec2::new(16.0, 16.0),
            size_px: 84.0,
            pick_padding_px: 6.0,
            snap_feature_threshold: 0.78,
            drag_threshold_px: 3.0,
            orbit_sensitivity_radians_per_px: 0.008,
            center_button_radius_px: 12.0,
            depth_range: DepthRange::ZeroToOne,
            z01: 0.08,
            face_color: Color {
                a: 0.35,
                ..Color::from_srgb_hex_rgb(0x38_38_3d)
            },
            edge_color: Color {
                a: 0.9,
                ..Color::from_srgb_hex_rgb(0xf2_f2_fa)
            },
            hover_color: Color {
                a: 0.55,
                ..Color::from_srgb_hex_rgb(0xff_d9_4d)
            },
            x_color: Color::from_srgb_hex_rgb(0xff_33_66),
            y_color: Color::from_srgb_hex_rgb(0x33_ff_66),
            z_color: Color::from_srgb_hex_rgb(0x33_80_ff),
        }
    }
}

impl ViewGizmoConfig {
    /// Scales pixel-based knobs to match the cursor's coordinate units.
    ///
    /// The view gizmo uses screen-space pixels for layout and hit testing. Those pixels are
    /// expected to match the units of `ViewGizmoInput.cursor_px` provided by the host.
    ///
    /// - If the host feeds window-local logical pixels ("screen px"), pass `1.0`.
    /// - If the host feeds physical pixels, pass the platform scale factor (pixels-per-point / DPR).
    /// - If the host feeds viewport render-target pixels, pass the target-pixels-per-screen-pixel
    ///   scale derived from the viewport mapping.
    pub fn scale_for_cursor_units_per_screen_px(mut self, cursor_units_per_screen_px: f32) -> Self {
        let s = cursor_units_per_screen_px.clamp(0.1, 16.0);
        self.margin_px *= s;
        self.size_px *= s;
        self.pick_padding_px *= s;
        self.drag_threshold_px *= s;
        self.center_button_radius_px *= s;
        self.orbit_sensitivity_radians_per_px /= s;
        self
    }

    /// Backwards-compatible alias for [`Self::scale_for_cursor_units_per_screen_px`].
    pub fn scale_for_pixels_per_point(self, pixels_per_point: f32) -> Self {
        self.scale_for_cursor_units_per_screen_px(pixels_per_point)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewGizmoInput {
    pub cursor_px: Vec2,
    pub hovered: bool,
    pub drag_started: bool,
    pub dragging: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewGizmoUpdate {
    HoverChanged {
        hovered: Option<ViewGizmoSnap>,
        center_button_hovered: bool,
    },
    OrbitDelta {
        delta_yaw_radians: f32,
        delta_pitch_radians: f32,
    },
    ToggleProjection,
    SnapView {
        snap: ViewGizmoSnap,
        view_dir: Vec3,
        up: Vec3,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewGizmoDragAction {
    ToggleProjection,
    Snap(ViewGizmoSnap),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ViewGizmoState {
    pub hovered: Option<ViewGizmoSnap>,
    pub hovered_center_button: bool,
    pub drag_active: bool,
    pub drag_orbiting: bool,
    pub drag_start_cursor_px: Vec2,
    pub drag_last_cursor_px: Vec2,
    pub drag_total_delta_px: Vec2,
    drag_action: Option<ViewGizmoDragAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewGizmo {
    pub config: ViewGizmoConfig,
    pub state: ViewGizmoState,
}

impl ViewGizmo {
    pub fn new(config: ViewGizmoConfig) -> Self {
        Self {
            config,
            state: ViewGizmoState::default(),
        }
    }

    /// Cancels an in-progress interaction, if any.
    ///
    /// This is intended for host-driven cancellation (e.g. Escape, mode switches, viewport teardown).
    /// It resets drag state without triggering click actions.
    pub fn cancel(&mut self) {
        self.state.drag_active = false;
        self.state.drag_orbiting = false;
        self.state.drag_action = None;
        self.state.drag_start_cursor_px = Vec2::ZERO;
        self.state.drag_last_cursor_px = Vec2::ZERO;
        self.state.drag_total_delta_px = Vec2::ZERO;
        self.state.hovered = None;
        self.state.hovered_center_button = false;
    }

    pub fn hit_test(&self, view_projection: Mat4, viewport: ViewportRect, cursor_px: Vec2) -> bool {
        let center_radius_px = self.config.center_button_radius_px;
        if center_radius_px > 0.0 {
            let center_px = self.anchor_center_px(viewport, self.config.size_px.max(1.0));
            if (cursor_px - center_px).length() <= center_radius_px {
                return true;
            }
        }

        self.pick_snap(view_projection, viewport, cursor_px)
            .is_some()
    }

    pub fn update(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: ViewGizmoInput,
    ) -> Option<ViewGizmoUpdate> {
        let threshold_px = self.config.drag_threshold_px.max(0.0);
        let center_px = self.anchor_center_px(viewport, self.config.size_px.max(1.0));
        let center_radius_px = self.config.center_button_radius_px;
        let center_button_hovered = center_radius_px > 0.0
            && input.hovered
            && (input.cursor_px - center_px).length() <= center_radius_px;

        if input.drag_started && input.dragging && input.hovered {
            let action = if center_button_hovered {
                Some(ViewGizmoDragAction::ToggleProjection)
            } else {
                self.pick_snap(view_projection, viewport, input.cursor_px)
                    .map(ViewGizmoDragAction::Snap)
            };

            if let Some(action) = action {
                self.state.drag_active = true;
                self.state.drag_orbiting = false;
                self.state.drag_start_cursor_px = input.cursor_px;
                self.state.drag_last_cursor_px = input.cursor_px;
                self.state.drag_total_delta_px = Vec2::ZERO;
                self.state.drag_action = Some(action);
                self.state.hovered = None;
                self.state.hovered_center_button = false;
                return Some(ViewGizmoUpdate::HoverChanged {
                    hovered: None,
                    center_button_hovered: false,
                });
            }
        }

        let drag_ended = self.state.drag_active && !input.dragging;
        if self.state.drag_active && input.dragging {
            let delta_px = input.cursor_px - self.state.drag_last_cursor_px;
            self.state.drag_last_cursor_px = input.cursor_px;
            self.state.drag_total_delta_px += delta_px;

            if !self.state.drag_orbiting && self.state.drag_total_delta_px.length() >= threshold_px
            {
                self.state.drag_orbiting = true;
            }

            if self.state.drag_orbiting {
                let sens = self.config.orbit_sensitivity_radians_per_px.max(0.0);
                return Some(ViewGizmoUpdate::OrbitDelta {
                    delta_yaw_radians: -delta_px.x * sens,
                    delta_pitch_radians: -delta_px.y * sens,
                });
            }

            return None;
        }

        if drag_ended {
            let click = self.state.drag_total_delta_px.length() < threshold_px;
            let action = self.state.drag_action;
            self.state.drag_active = false;
            self.state.drag_orbiting = false;
            self.state.drag_action = None;

            if click {
                match action {
                    Some(ViewGizmoDragAction::ToggleProjection) => {
                        return Some(ViewGizmoUpdate::ToggleProjection);
                    }
                    Some(ViewGizmoDragAction::Snap(snap)) => {
                        return Some(ViewGizmoUpdate::SnapView {
                            snap,
                            view_dir: snap.view_dir_vec3(),
                            up: snap.up_vec3(),
                        });
                    }
                    None => {}
                }
            }
        }

        let prev_hover = self.state.hovered;
        let prev_center = self.state.hovered_center_button;
        let hovered = if input.hovered && !center_button_hovered {
            self.pick_snap(view_projection, viewport, input.cursor_px)
        } else {
            None
        };
        self.state.hovered = hovered;
        self.state.hovered_center_button = center_button_hovered;

        (prev_hover != hovered || prev_center != center_button_hovered).then_some(
            ViewGizmoUpdate::HoverChanged {
                hovered,
                center_button_hovered,
            },
        )
    }

    pub fn draw(&self, view_projection: Mat4, viewport: ViewportRect) -> GizmoDrawList3d {
        self.draw_with_projection(view_projection, viewport, ViewGizmoProjection::Perspective)
    }

    pub fn draw_with_projection(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        projection: ViewGizmoProjection,
    ) -> GizmoDrawList3d {
        let mut out = GizmoDrawList3d::default();
        let Some((center, half, _world_per_px)) = self.cube_params(view_projection, viewport)
        else {
            return out;
        };

        let corners = cube_corners(center, half);
        out.lines
            .extend(cube_edges(corners, self.config.edge_color));
        out.lines.extend(axis_stubs(
            center,
            half,
            self.config.x_color,
            self.config.y_color,
            self.config.z_color,
        ));

        let face_color = self.config.face_color;
        out.triangles.extend(cube_faces(corners, face_color));

        if let Some(snap) = self.state.hovered {
            for face in faces_for_snap(snap) {
                out.triangles
                    .extend(hover_face(corners, face, self.config.hover_color));
            }
        }

        if self.config.center_button_radius_px > 0.0 {
            let base_color = match projection {
                ViewGizmoProjection::Perspective => self.config.edge_color,
                ViewGizmoProjection::Orthographic => self.config.y_color,
            };
            let center_button_color = if self.state.hovered_center_button {
                self.config.hover_color
            } else {
                base_color
            };
            out.lines.extend(cube_edges(
                cube_corners(center, half * 0.28),
                center_button_color,
            ));

            if matches!(projection, ViewGizmoProjection::Orthographic) {
                let mut fill = base_color;
                fill.a = (fill.a * 0.35).clamp(0.0, 1.0);
                let c = cube_corners(center, half * 0.26);
                out.triangles.extend(cube_faces(c, fill));
            }
        }

        for l in &mut out.lines {
            l.depth = DepthMode::Always;
        }
        for t in &mut out.triangles {
            t.depth = DepthMode::Always;
        }

        out
    }

    pub fn labels(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        projection: ViewGizmoProjection,
    ) -> Vec<ViewGizmoLabel> {
        let Some((center, half, _world_per_px)) = self.cube_params(view_projection, viewport)
        else {
            return Vec::new();
        };

        let mut out = Vec::with_capacity(8);
        let axis_len = half * 1.25;
        let push =
            |out: &mut Vec<ViewGizmoLabel>, world: Vec3, text: &'static str, color: Color| {
                if let Some(p) = crate::math::project_point(
                    view_projection,
                    viewport,
                    world,
                    self.config.depth_range,
                ) {
                    out.push(ViewGizmoLabel {
                        screen_px: p.screen,
                        text,
                        color,
                    });
                }
            };

        push(
            &mut out,
            center + Vec3::X * axis_len,
            "X",
            self.config.x_color,
        );
        push(
            &mut out,
            center + Vec3::Y * axis_len,
            "Y",
            self.config.y_color,
        );
        push(
            &mut out,
            center + Vec3::Z * axis_len,
            "Z",
            self.config.z_color,
        );

        if self.config.center_button_radius_px > 0.0 {
            let text = match projection {
                ViewGizmoProjection::Perspective => "P",
                ViewGizmoProjection::Orthographic => "O",
            };
            push(&mut out, center, text, self.config.edge_color);
        }

        out
    }

    fn pick_snap(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        cursor_px: Vec2,
    ) -> Option<ViewGizmoSnap> {
        let ray = ray_from_screen(
            view_projection,
            viewport,
            cursor_px,
            self.config.depth_range,
        )?;
        let (center, half, world_per_px) = self.cube_params(view_projection, viewport)?;
        let half = half + world_per_px * self.config.pick_padding_px.max(0.0);
        let aabb = Aabb3 {
            min: center - Vec3::splat(half),
            max: center + Vec3::splat(half),
        };

        let t = ray_aabb_intersect(ray.origin, ray.dir, aabb)?;
        let hit = ray.origin + ray.dir * t;
        let local = (hit - center) / half;
        let threshold = self.config.snap_feature_threshold.clamp(0.0, 1.0);
        snap_from_local(local, threshold)
    }

    fn cube_params(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
    ) -> Option<(Vec3, f32, f32)> {
        let size_px = self.config.size_px.max(1.0);
        let center_px = self.anchor_center_px(viewport, size_px);
        let z01 = self.config.z01.clamp(0.0, 1.0);

        let center = unproject_point(
            view_projection,
            viewport,
            center_px,
            self.config.depth_range,
            z01,
        )?;

        let px_dx = unproject_point(
            view_projection,
            viewport,
            center_px + Vec2::new(1.0, 0.0),
            self.config.depth_range,
            z01,
        )?;
        let px_dy = unproject_point(
            view_projection,
            viewport,
            center_px + Vec2::new(0.0, 1.0),
            self.config.depth_range,
            z01,
        )?;
        let world_per_px = ((px_dx - center).length() + (px_dy - center).length()) * 0.5;
        if !world_per_px.is_finite() || world_per_px <= 0.0 {
            return None;
        }

        let half = world_per_px * (size_px * 0.5);
        if !half.is_finite() || half <= 0.0 {
            return None;
        }

        Some((center, half, world_per_px))
    }

    fn anchor_center_px(&self, viewport: ViewportRect, size_px: f32) -> Vec2 {
        let half = size_px.max(1.0) * 0.5;
        let min = viewport.min;
        let max = viewport.max();
        let m = self.config.margin_px.max(Vec2::ZERO);

        match self.config.anchor {
            ViewGizmoAnchor::TopLeft => Vec2::new(min.x + m.x + half, min.y + m.y + half),
            ViewGizmoAnchor::TopRight => Vec2::new(max.x - m.x - half, min.y + m.y + half),
            ViewGizmoAnchor::BottomLeft => Vec2::new(min.x + m.x + half, max.y - m.y - half),
            ViewGizmoAnchor::BottomRight => Vec2::new(max.x - m.x - half, max.y - m.y - half),
        }
    }
}

fn cube_corners(center: Vec3, half: f32) -> [Vec3; 8] {
    let h = Vec3::splat(half);
    let min = center - h;
    let max = center + h;
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

fn cube_edges(corners: [Vec3; 8], color: Color) -> Vec<Line3d> {
    let idx = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    idx.iter()
        .map(|(a, b)| Line3d {
            a: corners[*a],
            b: corners[*b],
            color,
            depth: DepthMode::Always,
        })
        .collect()
}

fn axis_stubs(center: Vec3, half: f32, x: Color, y: Color, z: Color) -> Vec<Line3d> {
    let len = half * 1.25;
    vec![
        Line3d {
            a: center,
            b: center + Vec3::X * len,
            color: x,
            depth: DepthMode::Always,
        },
        Line3d {
            a: center,
            b: center + Vec3::Y * len,
            color: y,
            depth: DepthMode::Always,
        },
        Line3d {
            a: center,
            b: center + Vec3::Z * len,
            color: z,
            depth: DepthMode::Always,
        },
    ]
}

fn cube_faces(corners: [Vec3; 8], color: Color) -> Vec<Triangle3d> {
    // Two triangles per face. Winding is not relied upon (typically rendered without culling).
    let faces = [
        // -Z
        (0, 1, 2, 3),
        // +Z
        (4, 5, 6, 7),
        // -Y
        (0, 1, 5, 4),
        // +Y
        (3, 2, 6, 7),
        // -X
        (0, 3, 7, 4),
        // +X
        (1, 2, 6, 5),
    ];
    let mut out = Vec::with_capacity(12);
    for (a, b, c, d) in faces {
        out.push(Triangle3d {
            a: corners[a],
            b: corners[b],
            c: corners[c],
            color,
            depth: DepthMode::Always,
        });
        out.push(Triangle3d {
            a: corners[a],
            b: corners[c],
            c: corners[d],
            color,
            depth: DepthMode::Always,
        });
    }
    out
}

fn hover_face(corners: [Vec3; 8], face: ViewGizmoFace, color: Color) -> Vec<Triangle3d> {
    let (a, b, c, d) = match face {
        ViewGizmoFace::NegZ => (0, 1, 2, 3),
        ViewGizmoFace::PosZ => (4, 5, 6, 7),
        ViewGizmoFace::NegY => (0, 1, 5, 4),
        ViewGizmoFace::PosY => (3, 2, 6, 7),
        ViewGizmoFace::NegX => (0, 3, 7, 4),
        ViewGizmoFace::PosX => (1, 2, 6, 5),
    };
    vec![
        Triangle3d {
            a: corners[a],
            b: corners[b],
            c: corners[c],
            color,
            depth: DepthMode::Always,
        },
        Triangle3d {
            a: corners[a],
            b: corners[c],
            c: corners[d],
            color,
            depth: DepthMode::Always,
        },
    ]
}

fn faces_for_snap(snap: ViewGizmoSnap) -> Vec<ViewGizmoFace> {
    let local = [-snap.view_dir[0], -snap.view_dir[1], -snap.view_dir[2]];
    let mut out = Vec::with_capacity(3);

    let face_for = |axis: usize, sign: i8| -> Option<ViewGizmoFace> {
        if sign == 0 {
            return None;
        }
        match axis {
            0 => Some(if sign > 0 {
                ViewGizmoFace::PosX
            } else {
                ViewGizmoFace::NegX
            }),
            1 => Some(if sign > 0 {
                ViewGizmoFace::PosY
            } else {
                ViewGizmoFace::NegY
            }),
            2 => Some(if sign > 0 {
                ViewGizmoFace::PosZ
            } else {
                ViewGizmoFace::NegZ
            }),
            _ => None,
        }
    };

    for (axis, &sign) in local.iter().enumerate() {
        if let Some(face) = face_for(axis, sign) {
            out.push(face);
        }
    }

    if out.is_empty()
        && let Some(face) = snap.face()
    {
        out.push(face);
    }

    out
}

fn ray_aabb_intersect(origin: Vec3, dir: Vec3, aabb: Aabb3) -> Option<f32> {
    let mut tmin = 0.0;
    let mut tmax = f32::INFINITY;

    let check_axis = |o: f32, d: f32, min: f32, max: f32, tmin: &mut f32, tmax: &mut f32| {
        if d.abs() < 1e-8 {
            // Ray is parallel to the slab; reject if origin is outside.
            (min..=max).contains(&o)
        } else {
            let inv = 1.0 / d;
            let mut t1 = (min - o) * inv;
            let mut t2 = (max - o) * inv;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            *tmin = (*tmin).max(t1);
            *tmax = (*tmax).min(t2);
            *tmin <= *tmax
        }
    };

    if !check_axis(
        origin.x, dir.x, aabb.min.x, aabb.max.x, &mut tmin, &mut tmax,
    ) {
        return None;
    }
    if !check_axis(
        origin.y, dir.y, aabb.min.y, aabb.max.y, &mut tmin, &mut tmax,
    ) {
        return None;
    }
    if !check_axis(
        origin.z, dir.z, aabb.min.z, aabb.max.z, &mut tmin, &mut tmax,
    ) {
        return None;
    }

    if !tmax.is_finite() || tmax < 0.0 {
        return None;
    }
    let t = if tmin >= 0.0 { tmin } else { tmax };
    (t.is_finite() && t >= 0.0).then_some(t)
}

fn snap_component(v: f32, threshold: f32) -> i8 {
    if v.abs() >= threshold {
        if v >= 0.0 { 1 } else { -1 }
    } else {
        0
    }
}

fn snap_from_local(local: Vec3, threshold: f32) -> Option<ViewGizmoSnap> {
    if !local.is_finite() {
        return None;
    }
    let threshold = threshold.clamp(0.0, 1.0);
    // `local` is in `[-1, 1]` and is expected to lie on the AABB surface. Convert the hit location
    // into a 6/12/8-way direction by snapping components near +/-1.
    let sx = snap_component(local.x, threshold);
    let sy = snap_component(local.y, threshold);
    let sz = snap_component(local.z, threshold);

    // A face hit always has at least one component at +/-1.
    let view_dir = [-sx, -sy, -sz];
    ViewGizmoSnap::new(view_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_view_projection(viewport_px: (f32, f32), eye: Vec3) -> Mat4 {
        let aspect = viewport_px.0.max(1.0) / viewport_px.1.max(1.0);
        let view_dir = (Vec3::ZERO - eye).normalize_or_zero();
        let up = if view_dir.dot(Vec3::Y).abs() > 0.95 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        let view = Mat4::look_at_rh(eye, Vec3::ZERO, up);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), aspect, 0.05, 100.0);
        proj * view
    }

    fn centered_gizmo(viewport: ViewportRect) -> ViewGizmo {
        let mut cfg = ViewGizmoConfig {
            anchor: ViewGizmoAnchor::TopLeft,
            ..Default::default()
        };
        let half = cfg.size_px.max(1.0) * 0.5;
        let desired_center = viewport.min + viewport.size * 0.5;
        cfg.margin_px = desired_center - Vec2::splat(half);
        ViewGizmo::new(cfg)
    }

    fn face_from_camera_position(eye: Vec3, cube_center: Vec3) -> ViewGizmoFace {
        let v = eye - cube_center;
        let av = v.abs();
        if av.x >= av.y && av.x >= av.z {
            if v.x >= 0.0 {
                ViewGizmoFace::PosX
            } else {
                ViewGizmoFace::NegX
            }
        } else if av.y >= av.x && av.y >= av.z {
            if v.y >= 0.0 {
                ViewGizmoFace::PosY
            } else {
                ViewGizmoFace::NegY
            }
        } else if v.z >= 0.0 {
            ViewGizmoFace::PosZ
        } else {
            ViewGizmoFace::NegZ
        }
    }

    #[test]
    fn view_gizmo_hover_picks_front_face_for_axis_cameras() {
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let mut gizmo = centered_gizmo(vp);
        gizmo.config.center_button_radius_px = 0.0;

        let eyes = [
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(-5.0, 0.0, 0.0),
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.0, -5.0, 0.0),
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, -5.0),
        ];

        for eye in eyes {
            let view_proj = test_view_projection((800.0, 600.0), eye);
            let (center, _half, _) = gizmo.cube_params(view_proj, vp).unwrap();
            let expected = face_from_camera_position(eye, center);
            let cursor = Vec2::new(400.0, 300.0);

            let input = ViewGizmoInput {
                cursor_px: cursor,
                hovered: true,
                drag_started: false,
                dragging: false,
            };
            let update = gizmo.update(view_proj, vp, input);
            assert!(
                matches!(update, Some(ViewGizmoUpdate::HoverChanged { .. }) | None),
                "hover update should be either None or HoverChanged"
            );
            assert_eq!(
                gizmo.state.hovered,
                Some(ViewGizmoSnap::from_face(expected)),
                "eye={eye:?} center={center:?} expected={expected:?} hovered={:?}",
                gizmo.state.hovered
            );
        }
    }

    #[test]
    fn view_gizmo_click_emits_snap_view() {
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let mut gizmo = centered_gizmo(vp);
        gizmo.config.center_button_radius_px = 0.0;

        let eye = Vec3::new(0.0, 0.0, 5.0);
        let view_proj = test_view_projection((800.0, 600.0), eye);
        let (center, _half, _) = gizmo.cube_params(view_proj, vp).unwrap();
        let expected = face_from_camera_position(eye, center);
        let cursor = Vec2::new(400.0, 300.0);

        let input_down = ViewGizmoInput {
            cursor_px: cursor,
            hovered: true,
            drag_started: true,
            dragging: true,
        };
        let _ = gizmo.update(view_proj, vp, input_down);

        let input_up = ViewGizmoInput {
            cursor_px: cursor,
            hovered: true,
            drag_started: false,
            dragging: false,
        };
        let update = gizmo.update(view_proj, vp, input_up).unwrap();
        match update {
            ViewGizmoUpdate::SnapView { snap, view_dir, up } => {
                assert_eq!(snap, ViewGizmoSnap::from_face(expected));
                assert!((view_dir - snap.view_dir_vec3()).length() < 1e-6);
                assert!((up - snap.up_vec3()).length() < 1e-6);
            }
            _ => panic!("expected SnapView"),
        };
    }

    #[test]
    fn view_gizmo_click_center_emits_toggle_projection() {
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let mut gizmo = centered_gizmo(vp);
        gizmo.config.center_button_radius_px = 32.0;

        let eye = Vec3::new(0.0, 0.0, 5.0);
        let view_proj = test_view_projection((800.0, 600.0), eye);
        let cursor = Vec2::new(400.0, 300.0);

        let input_down = ViewGizmoInput {
            cursor_px: cursor,
            hovered: true,
            drag_started: true,
            dragging: true,
        };
        let _ = gizmo.update(view_proj, vp, input_down);

        let input_up = ViewGizmoInput {
            cursor_px: cursor,
            hovered: true,
            drag_started: false,
            dragging: false,
        };
        let update = gizmo.update(view_proj, vp, input_up).unwrap();
        assert!(matches!(update, ViewGizmoUpdate::ToggleProjection));
    }

    #[test]
    fn view_gizmo_drag_emits_orbit_delta() {
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let mut gizmo = centered_gizmo(vp);
        gizmo.config.center_button_radius_px = 0.0;
        gizmo.config.drag_threshold_px = 0.0;
        gizmo.config.orbit_sensitivity_radians_per_px = 0.01;

        let eye = Vec3::new(0.0, 0.0, 5.0);
        let view_proj = test_view_projection((800.0, 600.0), eye);
        let cursor = Vec2::new(400.0, 300.0);

        let input_down = ViewGizmoInput {
            cursor_px: cursor,
            hovered: true,
            drag_started: true,
            dragging: true,
        };
        let _ = gizmo.update(view_proj, vp, input_down);

        let moved_cursor = cursor + Vec2::new(10.0, -5.0);
        let input_move = ViewGizmoInput {
            cursor_px: moved_cursor,
            hovered: true,
            drag_started: false,
            dragging: true,
        };
        let update = gizmo.update(view_proj, vp, input_move).unwrap();
        match update {
            ViewGizmoUpdate::OrbitDelta {
                delta_yaw_radians,
                delta_pitch_radians,
            } => {
                assert!((delta_yaw_radians - (-0.1)).abs() < 1e-6);
                assert!((delta_pitch_radians - 0.05).abs() < 1e-6);
            }
            _ => panic!("expected OrbitDelta"),
        }
    }

    #[test]
    fn snap_from_local_produces_edge_and_corner_directions() {
        let threshold = 0.78;
        let corner = snap_from_local(Vec3::new(1.0, 0.9, 0.9), threshold).unwrap();
        assert_eq!(corner.view_dir, [-1, -1, -1]);

        let edge = snap_from_local(Vec3::new(1.0, 0.9, 0.2), threshold).unwrap();
        assert_eq!(edge.view_dir, [-1, -1, 0]);
    }

    #[test]
    fn scale_for_cursor_units_matches_pixels_per_point_alias() {
        let base = ViewGizmoConfig {
            margin_px: Vec2::new(3.0, 5.0),
            size_px: 10.0,
            pick_padding_px: 2.0,
            drag_threshold_px: 4.0,
            orbit_sensitivity_radians_per_px: 0.5,
            center_button_radius_px: 6.0,
            ..ViewGizmoConfig::default()
        };

        let scaled = base.scale_for_cursor_units_per_screen_px(2.0);
        assert_eq!(scaled.margin_px, Vec2::new(6.0, 10.0));
        assert!((scaled.orbit_sensitivity_radians_per_px - 0.25).abs() < 1e-6);

        let alias = base.scale_for_pixels_per_point(2.0);
        assert_eq!(scaled, alias);
    }
}
