use fret_core::Color;
use glam::{Mat4, Quat, Vec2, Vec3};

use crate::math::{
    DepthRange, Ray3d, ViewportRect, project_point, ray_from_screen, unproject_point,
};

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
pub enum DepthMode {
    Test,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3d {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
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
pub struct Line3d {
    pub a: Vec3,
    pub b: Vec3,
    pub color: Color,
    pub depth: DepthMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoConfig {
    pub mode: GizmoMode,
    pub orientation: GizmoOrientation,
    pub depth_mode: DepthMode,
    pub depth_range: DepthRange,
    pub size_px: f32,
    pub pick_radius_px: f32,
    pub x_color: Color,
    pub y_color: Color,
    pub z_color: Color,
    pub hover_color: Color,
}

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            mode: GizmoMode::Translate,
            orientation: GizmoOrientation::World,
            depth_mode: DepthMode::Test,
            depth_range: DepthRange::default(),
            size_px: 96.0,
            pick_radius_px: 10.0,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoInput {
    pub cursor_px: Vec2,
    pub hovered: bool,
    pub drag_started: bool,
    pub dragging: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoState {
    pub hovered: Option<HandleId>,
    pub active: Option<HandleId>,
    drag_start_axis_t: f32,
    drag_axis_dir: Vec3,
    drag_origin: Vec3,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            hovered: None,
            active: None,
            drag_start_axis_t: 0.0,
            drag_axis_dir: Vec3::X,
            drag_origin: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoResult {
    Translation { delta: Vec3, total: Vec3 },
}

#[derive(Debug, Clone)]
pub struct GizmoUpdate {
    pub result: GizmoResult,
    pub updated_targets: Vec<Transform3d>,
}

#[derive(Debug, Default)]
pub struct Gizmo {
    pub config: GizmoConfig,
    pub state: GizmoState,
}

impl Gizmo {
    pub fn new(config: GizmoConfig) -> Self {
        Self {
            config,
            state: GizmoState::default(),
        }
    }

    pub fn update(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        targets: &[Transform3d],
    ) -> Option<GizmoUpdate> {
        if targets.is_empty() {
            self.state.hovered = None;
            self.state.active = None;
            return None;
        }

        let origin = targets[0].translation;

        let Some(ray) = ray_from_screen(
            view_projection,
            viewport,
            input.cursor_px,
            self.config.depth_range,
        ) else {
            return None;
        };

        let mut hovered = None;
        if self.state.active.is_none() && input.hovered {
            hovered = self.pick_translate_axis(view_projection, viewport, origin, input.cursor_px);
        }
        self.state.hovered = hovered;

        if self.state.active.is_none() {
            if input.drag_started {
                if let Some(h) = hovered {
                    self.state.active = Some(h);
                    let (axis_dir, _) = axis_for_handle(h);
                    self.state.drag_axis_dir = axis_dir;
                    self.state.drag_origin = origin;
                    self.state.drag_start_axis_t = closest_t_on_axis(ray, origin, axis_dir);
                }
            }
            return None;
        }

        let active = self.state.active.unwrap();
        let (axis_dir, _) = axis_for_handle(active);

        if input.dragging {
            let t = closest_t_on_axis(ray, self.state.drag_origin, axis_dir);
            let delta_t = t - self.state.drag_start_axis_t;
            let delta = delta_t * axis_dir;
            let updated_targets = targets
                .iter()
                .map(|t| Transform3d {
                    translation: t.translation + delta,
                    ..*t
                })
                .collect::<Vec<_>>();
            return Some(GizmoUpdate {
                result: GizmoResult::Translation {
                    delta,
                    total: delta,
                },
                updated_targets,
            });
        }

        self.state.active = None;
        None
    }

    pub fn draw_translate_axes(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
    ) -> Vec<Line3d> {
        let mut out = Vec::new();
        let length_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0);
        for &(axis_dir, color, handle) in &[
            (Vec3::X, self.config.x_color, HandleId(1)),
            (Vec3::Y, self.config.y_color, HandleId(2)),
            (Vec3::Z, self.config.z_color, HandleId(3)),
        ] {
            let c = if self.state.active == Some(handle) || self.state.hovered == Some(handle) {
                self.config.hover_color
            } else {
                color
            };
            out.push(Line3d {
                a: origin,
                b: origin + axis_dir * length_world,
                color: c,
                depth: self.config.depth_mode,
            });
        }
        out
    }

    fn pick_translate_axis(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
    ) -> Option<HandleId> {
        let length_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )?;

        let mut best: Option<(HandleId, f32)> = None;
        for &(axis_dir, handle) in &[
            (Vec3::X, HandleId(1)),
            (Vec3::Y, HandleId(2)),
            (Vec3::Z, HandleId(3)),
        ] {
            let a = origin;
            let b = origin + axis_dir * length_world;
            let Some(pa) = project_point(view_projection, viewport, a, self.config.depth_range)
            else {
                continue;
            };
            let Some(pb) = project_point(view_projection, viewport, b, self.config.depth_range)
            else {
                continue;
            };
            let d = distance_point_to_segment_px(cursor, pa.screen, pb.screen);
            if d <= self.config.pick_radius_px {
                match best {
                    Some((_, best_d)) if d >= best_d => {}
                    _ => best = Some((handle, d)),
                }
            }
        }
        best.map(|(h, _)| h)
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

fn closest_t_on_axis(ray: Ray3d, axis_origin: Vec3, axis_dir: Vec3) -> f32 {
    let u = axis_dir.normalize_or_zero();
    let v = ray.dir;
    let w0 = axis_origin - ray.origin;
    let a = u.dot(u);
    let b = u.dot(v);
    let c = v.dot(v);
    let d = u.dot(w0);
    let e = v.dot(w0);
    let denom = a * c - b * b;
    if denom.abs() < 1e-6 {
        return 0.0;
    }
    (b * e - c * d) / denom
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
