use fret_core::Color;
use glam::{Mat4, Vec2, Vec3};

use crate::gizmo::{Aabb3, DepthMode, GizmoDrawList3d, Line3d, Triangle3d};
use crate::math::{DepthRange, ViewportRect, ray_from_screen, unproject_point};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewGizmoAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for ViewGizmoAnchor {
    fn default() -> Self {
        Self::TopRight
    }
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
            depth_range: DepthRange::ZeroToOne,
            z01: 0.08,
            face_color: Color {
                r: 0.22,
                g: 0.22,
                b: 0.24,
                a: 0.35,
            },
            edge_color: Color {
                r: 0.95,
                g: 0.95,
                b: 0.98,
                a: 0.9,
            },
            hover_color: Color {
                r: 1.0,
                g: 0.85,
                b: 0.3,
                a: 0.55,
            },
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
        }
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
        hovered: Option<ViewGizmoFace>,
    },
    SnapView {
        face: ViewGizmoFace,
        view_dir: Vec3,
        up: Vec3,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ViewGizmoState {
    pub hovered_face: Option<ViewGizmoFace>,
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

    pub fn update(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: ViewGizmoInput,
    ) -> Option<ViewGizmoUpdate> {
        let prev_hover = self.state.hovered_face;
        let hovered = if input.hovered {
            self.pick_face(view_projection, viewport, input.cursor_px)
        } else {
            None
        };
        self.state.hovered_face = hovered;

        if input.drag_started && input.dragging {
            if let Some(face) = hovered {
                return Some(ViewGizmoUpdate::SnapView {
                    face,
                    view_dir: face.view_dir(),
                    up: face.default_up(),
                });
            }
        }

        (prev_hover != hovered).then_some(ViewGizmoUpdate::HoverChanged { hovered })
    }

    pub fn draw(&self, view_projection: Mat4, viewport: ViewportRect) -> GizmoDrawList3d {
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

        if let Some(face) = self.state.hovered_face {
            out.triangles
                .extend(hover_face(corners, face, self.config.hover_color));
        }

        for l in &mut out.lines {
            l.depth = DepthMode::Always;
        }
        for t in &mut out.triangles {
            t.depth = DepthMode::Always;
        }

        out
    }

    fn pick_face(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        cursor_px: Vec2,
    ) -> Option<ViewGizmoFace> {
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
        face_from_hit(center, hit)
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

fn face_from_hit(center: Vec3, hit: Vec3) -> Option<ViewGizmoFace> {
    let d = hit - center;
    if !d.is_finite() {
        return None;
    }
    let ad = d.abs();
    if ad.x >= ad.y && ad.x >= ad.z {
        Some(if d.x >= 0.0 {
            ViewGizmoFace::PosX
        } else {
            ViewGizmoFace::NegX
        })
    } else if ad.y >= ad.x && ad.y >= ad.z {
        Some(if d.y >= 0.0 {
            ViewGizmoFace::PosY
        } else {
            ViewGizmoFace::NegY
        })
    } else {
        Some(if d.z >= 0.0 {
            ViewGizmoFace::PosZ
        } else {
            ViewGizmoFace::NegZ
        })
    }
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
        let mut cfg = ViewGizmoConfig::default();
        cfg.anchor = ViewGizmoAnchor::TopLeft;
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
                gizmo.state.hovered_face,
                Some(expected),
                "eye={eye:?} center={center:?} expected={expected:?} hovered={:?}",
                gizmo.state.hovered_face
            );
        }
    }

    #[test]
    fn view_gizmo_click_emits_snap_view() {
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let mut gizmo = centered_gizmo(vp);

        let eye = Vec3::new(0.0, 0.0, 5.0);
        let view_proj = test_view_projection((800.0, 600.0), eye);
        let (center, _half, _) = gizmo.cube_params(view_proj, vp).unwrap();
        let expected = face_from_camera_position(eye, center);
        let cursor = Vec2::new(400.0, 300.0);

        let input = ViewGizmoInput {
            cursor_px: cursor,
            hovered: true,
            drag_started: true,
            dragging: true,
        };
        let update = gizmo.update(view_proj, vp, input).unwrap();
        match update {
            ViewGizmoUpdate::SnapView { face, view_dir, up } => {
                assert_eq!(face, expected);
                assert!((view_dir - expected.view_dir()).length() < 1e-6);
                assert!((up - expected.default_up()).length() < 1e-6);
            }
            _ => panic!("expected SnapView"),
        }
    }
}
