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
pub enum GizmoPivotMode {
    /// The gizmo is positioned at the active target's pivot.
    Active,
    /// The gizmo is positioned at the selection center (average translation of targets).
    Center,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoConfig {
    pub mode: GizmoMode,
    pub orientation: GizmoOrientation,
    pub pivot_mode: GizmoPivotMode,
    pub depth_mode: DepthMode,
    pub depth_range: DepthRange,
    pub size_px: f32,
    pub pick_radius_px: f32,
    pub line_thickness_px: f32,
    pub drag_start_threshold_px: f32,
    pub translate_snap_step: Option<f32>,
    pub rotate_snap_step_radians: Option<f32>,
    pub scale_snap_step: Option<f32>,
    /// When `true`, draw a faint always-on-top pass so occluded segments remain visible.
    pub show_occluded: bool,
    /// Alpha multiplier for the occluded always-on-top pass.
    pub occluded_alpha: f32,
    /// When `true`, includes a view-axis rotation ring (camera-facing) in `Rotate`/`Universal`.
    pub show_view_axis_ring: bool,
    /// Radius multiplier for the view-axis ring (outer ring).
    pub view_axis_ring_radius_scale: f32,
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
            pivot_mode: GizmoPivotMode::Active,
            depth_mode: DepthMode::Test,
            depth_range: DepthRange::default(),
            size_px: 96.0,
            pick_radius_px: 10.0,
            line_thickness_px: 6.0,
            drag_start_threshold_px: 3.0,
            translate_snap_step: None,
            rotate_snap_step_radians: Some(15.0_f32.to_radians()),
            scale_snap_step: Some(0.1),
            show_occluded: true,
            occluded_alpha: 0.25,
            show_view_axis_ring: true,
            view_axis_ring_radius_scale: 1.2,
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
    pub snap: bool,
    pub cancel: bool,
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
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TranslateConstraint {
    Axis { axis_dir: Vec3 },
    Plane { u: Vec3, v: Vec3, normal: Vec3 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PickHit {
    handle: HandleId,
    score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoState {
    pub hovered: Option<HandleId>,
    pub active: Option<HandleId>,
    pub hovered_kind: Option<GizmoMode>,
    drag_mode: GizmoMode,
    drag_snap: bool,
    drag_has_started: bool,
    drag_start_cursor_px: Vec2,
    drag_axis_dir: Vec3,
    drag_origin: Vec3,
    drag_origin_z01: f32,
    drag_plane_normal: Vec3,
    drag_prev_hit_world: Vec3,
    drag_total_axis_raw: f32,
    drag_total_axis_applied: f32,
    drag_translate_is_plane: bool,
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
    drag_scale_axis: Option<usize>,
    drag_scale_is_uniform: bool,
    drag_total_scale_raw: f32,
    drag_total_scale_applied: f32,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            hovered: None,
            active: None,
            hovered_kind: None,
            drag_mode: GizmoMode::Translate,
            drag_snap: false,
            drag_has_started: false,
            drag_start_cursor_px: Vec2::ZERO,
            drag_axis_dir: Vec3::X,
            drag_origin: Vec3::ZERO,
            drag_origin_z01: 0.0,
            drag_plane_normal: Vec3::Z,
            drag_prev_hit_world: Vec3::ZERO,
            drag_total_axis_raw: 0.0,
            drag_total_axis_applied: 0.0,
            drag_translate_is_plane: false,
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
            drag_scale_axis: None,
            drag_scale_is_uniform: false,
            drag_total_scale_raw: 0.0,
            drag_total_scale_applied: 1.0,
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
    pub updated_targets: Vec<GizmoTarget3d>,
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

        let origin = match self.config.pivot_mode {
            GizmoPivotMode::Active => active_transform.translation,
            GizmoPivotMode::Center => {
                let sum = targets
                    .iter()
                    .fold(Vec3::ZERO, |acc, t| acc + t.transform.translation);
                sum / (targets.len().max(1) as f32)
            }
        };
        let Some(cursor_ray) = ray_from_screen(
            view_projection,
            viewport,
            input.cursor_px,
            self.config.depth_range,
        ) else {
            return None;
        };

        let axes = self.axis_dirs(&active_transform);
        let mut hovered: Option<HandleId> = None;
        let mut hovered_kind: Option<GizmoMode> = None;
        if self.state.active.is_none() && input.hovered {
            let pick = match self.config.mode {
                GizmoMode::Translate => self
                    .pick_translate_handle(view_projection, viewport, origin, input.cursor_px, axes)
                    .map(|h| (h, GizmoMode::Translate)),
                GizmoMode::Rotate => self
                    .pick_rotate_axis(view_projection, viewport, origin, input.cursor_px, axes)
                    .map(|h| (h, GizmoMode::Rotate)),
                GizmoMode::Scale => self
                    .pick_scale_handle(view_projection, viewport, origin, input.cursor_px, axes)
                    .map(|h| (h, GizmoMode::Scale)),
                GizmoMode::Universal => self.pick_universal_handle(
                    view_projection,
                    viewport,
                    origin,
                    input.cursor_px,
                    axes,
                ),
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
                    let begin = match self.config.mode {
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
                        ),
                        GizmoMode::Scale => self.begin_scale_drag(
                            view_projection,
                            viewport,
                            input,
                            targets,
                            cursor_ray,
                            origin,
                            h,
                            axes,
                        ),
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
                            ),
                            _ => None,
                        },
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

                    let delta_world = hit_world - self.state.drag_prev_hit_world;
                    self.state.drag_prev_hit_world = hit_world;

                    let (delta, total) = if self.state.drag_translate_is_plane {
                        let u = self.state.drag_translate_u;
                        let v = self.state.drag_translate_v;
                        self.state.drag_total_plane_raw +=
                            Vec2::new(delta_world.dot(u), delta_world.dot(v));
                        let desired_total = if input.snap {
                            self.config
                                .translate_snap_step
                                .filter(|s| s.is_finite() && *s > 0.0)
                                .map(|step| {
                                    Vec2::new(
                                        (self.state.drag_total_plane_raw.x / step).round() * step,
                                        (self.state.drag_total_plane_raw.y / step).round() * step,
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
                        self.state.drag_total_axis_raw += delta_world.dot(axis_dir);
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
                    };
                    let updated_targets = targets
                        .iter()
                        .map(|t| GizmoTarget3d {
                            id: t.id,
                            transform: Transform3d {
                                translation: t.transform.translation + delta,
                                ..t.transform
                            },
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
                let axis_dir = self.state.drag_axis_dir.normalize_or_zero();
                if axis_dir.length_squared() == 0.0 {
                    self.state.active = None;
                    return None;
                }

                if input.cancel {
                    let total = self.state.drag_total_angle_applied;
                    self.state.active = None;
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

                    let Some(angle) = angle_on_plane(
                        self.state.drag_origin,
                        hit_world,
                        axis_dir,
                        self.state.drag_basis_u,
                        self.state.drag_basis_v,
                    ) else {
                        return None;
                    };

                    let delta_angle = wrap_angle(angle - self.state.drag_prev_angle);
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

                    let delta_q = Quat::from_axis_angle(axis_dir, delta_apply);
                    let updated_targets = targets
                        .iter()
                        .map(|t| GizmoTarget3d {
                            id: t.id,
                            transform: Transform3d {
                                translation: self.state.drag_origin
                                    + delta_q * (t.transform.translation - self.state.drag_origin),
                                rotation: (delta_q * t.transform.rotation).normalize(),
                                ..t.transform
                            },
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
                    return None;
                }

                let total = self.state.drag_total_angle_applied;
                self.state.active = None;
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
            GizmoMode::Scale => {
                let length_world = axis_length_world(
                    view_projection,
                    viewport,
                    self.state.drag_origin,
                    self.config.depth_range,
                    self.config.size_px,
                )
                .unwrap_or(1.0)
                .max(1e-6);

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

                if input.cancel {
                    let total = total_vec(self.state.drag_total_scale_applied);
                    self.state.active = None;
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

                    let delta_world = hit_world - self.state.drag_prev_hit_world;
                    self.state.drag_prev_hit_world = hit_world;

                    let scale_dir = self.state.drag_axis_dir.normalize_or_zero();
                    if scale_dir.length_squared() == 0.0 {
                        return None;
                    }
                    self.state.drag_total_scale_raw += delta_world.dot(scale_dir);

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

                    let updated_targets = targets
                        .iter()
                        .map(|t| {
                            let origin = self.state.drag_origin;
                            let offset = t.transform.translation - origin;
                            let axis_dir = self.state.drag_axis_dir.normalize_or_zero();
                            let translation = if self.state.drag_scale_is_uniform {
                                origin + offset * delta_factor
                            } else if axis_dir.length_squared() > 0.0 {
                                let component = axis_dir * offset.dot(axis_dir);
                                origin + (offset + component * (delta_factor - 1.0))
                            } else {
                                t.transform.translation
                            };

                            let mut scale = t.transform.scale;
                            if self.state.drag_scale_is_uniform {
                                scale *= delta_factor;
                            } else if let Some(axis) = self.state.drag_scale_axis {
                                scale[axis] = (scale[axis] * delta_factor).max(1e-4);
                            }
                            GizmoTarget3d {
                                id: t.id,
                                transform: Transform3d {
                                    translation,
                                    scale,
                                    ..t.transform
                                },
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
                    return None;
                }

                let total = total_vec(self.state.drag_total_scale_applied);
                self.state.active = None;
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

        let origin = match self.config.pivot_mode {
            GizmoPivotMode::Active => active_transform.translation,
            GizmoPivotMode::Center => {
                let sum = targets
                    .iter()
                    .fold(Vec3::ZERO, |acc, t| acc + t.transform.translation);
                sum / (targets.len().max(1) as f32)
            }
        };
        let axes = self.axis_dirs(&active_transform);
        match self.config.mode {
            GizmoMode::Translate => {
                let mut out = GizmoDrawList3d::default();
                out.lines
                    .extend(self.draw_translate_axes(view_projection, viewport, origin, axes));
                out.lines.extend(self.draw_translate_planes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                ));
                out.lines
                    .extend(self.draw_translate_screen(view_projection, viewport, origin));
                out.triangles.extend(self.draw_translate_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                ));
                out
            }
            GizmoMode::Rotate => {
                let mut out = GizmoDrawList3d::default();
                out.lines
                    .extend(self.draw_rotate_rings(view_projection, viewport, origin, axes));
                let feedback = self.draw_rotate_feedback(view_projection, viewport, origin);
                out.lines.extend(feedback.lines);
                out.triangles.extend(feedback.triangles);
                out
            }
            GizmoMode::Scale => GizmoDrawList3d {
                lines: self.draw_scale_handles(view_projection, viewport, origin, axes),
                triangles: self.draw_scale_solids(view_projection, viewport, origin, axes),
            },
            GizmoMode::Universal => {
                let mut out = GizmoDrawList3d::default();
                out.lines
                    .extend(self.draw_translate_axes(view_projection, viewport, origin, axes));
                out.lines.extend(self.draw_translate_planes(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                ));
                out.lines
                    .extend(self.draw_translate_screen(view_projection, viewport, origin));
                out.lines
                    .extend(self.draw_rotate_rings(view_projection, viewport, origin, axes));
                let rotate_feedback = self.draw_rotate_feedback(view_projection, viewport, origin);
                out.lines.extend(rotate_feedback.lines);
                out.triangles.extend(self.draw_translate_solids(
                    view_projection,
                    viewport,
                    origin,
                    axes,
                ));
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

    fn push_tri(
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

        match constraint {
            TranslateConstraint::Axis { axis_dir } => {
                self.state.drag_translate_is_plane = false;
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
                self.state.drag_translate_u = u;
                self.state.drag_translate_v = v;
                self.state.drag_plane_normal = normal;
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
        self.state.drag_prev_hit_world = start_hit_world;

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
    ) -> Option<GizmoUpdate> {
        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;
        let axis_dir = if active.0 == 8 {
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

        let angle = angle_on_plane(origin, start_hit_world, axis_dir, u, v)?;
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
    ) -> Option<GizmoUpdate> {
        let origin_z01 = origin_z01(view_projection, viewport, origin, self.config.depth_range)?;

        let (scale_dir, plane_normal, axis) = match active.0 {
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
                (axis_dir, plane_normal, Some(axis_index))
            }
            7 => {
                let view_dir =
                    view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)?;
                let (u, v) = plane_basis(view_dir);
                let dir = (u + v).normalize_or_zero();
                let plane_normal = view_dir.normalize_or_zero();
                if dir.length_squared() == 0.0 || plane_normal.length_squared() == 0.0 {
                    return None;
                }
                (dir, plane_normal, None)
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
        self.state.drag_plane_normal = plane_normal;
        self.state.drag_axis_dir = scale_dir;
        self.state.drag_prev_hit_world = ray_plane_intersect(cursor_ray, origin, plane_normal)
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
        self.state.drag_total_scale_raw = 0.0;
        self.state.drag_total_scale_applied = 1.0;
        self.state.drag_scale_axis = axis;
        self.state.drag_scale_is_uniform = axis.is_none();

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

    fn draw_translate_axes(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
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
        let head_len = length_world * 0.18;
        let shaft_len = (length_world - head_len).max(length_world * 0.2);
        for &((axis_dir, color), handle) in &[
            ((axes[0], self.config.x_color), HandleId(1)),
            ((axes[1], self.config.y_color), HandleId(2)),
            ((axes[2], self.config.z_color), HandleId(3)),
        ] {
            let c = if self.is_handle_highlighted(GizmoMode::Translate, handle) {
                self.config.hover_color
            } else {
                color
            };
            self.push_line(
                &mut out,
                origin,
                origin + axis_dir * shaft_len,
                c,
                self.config.depth_mode,
            );
        }
        out
    }

    fn draw_translate_planes(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
    ) -> Vec<Line3d> {
        let length_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0);

        let off = length_world * 0.15;
        let size = length_world * 0.25;

        let mut out = Vec::new();
        for &(u, v, base_color, handle) in &[
            (
                axes[0],
                axes[1],
                mix_alpha(self.config.z_color, 0.55),
                TranslateHandle::PlaneXY,
            ), // XY
            (
                axes[0],
                axes[2],
                mix_alpha(self.config.y_color, 0.55),
                TranslateHandle::PlaneXZ,
            ), // XZ
            (
                axes[1],
                axes[2],
                mix_alpha(self.config.x_color, 0.55),
                TranslateHandle::PlaneYZ,
            ), // YZ
        ] {
            let handle_id = handle.id();
            let color = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
                mix_alpha(self.config.hover_color, 0.85)
            } else {
                base_color
            };

            let quad = translate_plane_quad_world(origin, u, v, off, size);
            for (a, b) in [
                (quad[0], quad[1]),
                (quad[1], quad[2]),
                (quad[2], quad[3]),
                (quad[3], quad[0]),
            ] {
                self.push_line(&mut out, a, b, color, self.config.depth_mode);
            }
        }
        out
    }

    fn draw_translate_screen(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
    ) -> Vec<Line3d> {
        let length_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0);

        let Some(view_dir) =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        else {
            return Vec::new();
        };
        let (u, v) = plane_basis(view_dir);
        let half = length_world * 0.08;
        let base = mix_alpha(self.config.hover_color, 0.65);
        let handle_id = TranslateHandle::Screen.id();
        let color = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
            mix_alpha(self.config.hover_color, 1.0)
        } else {
            base
        };

        let p0 = origin + (-u - v) * half;
        let p1 = origin + (u - v) * half;
        let p2 = origin + (u + v) * half;
        let p3 = origin + (-u + v) * half;

        let mut out = Vec::new();
        for (a, b) in [(p0, p1), (p1, p2), (p2, p3), (p3, p0)] {
            self.push_line(&mut out, a, b, color, DepthMode::Always);
        }
        out
    }

    fn draw_translate_solids(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
    ) -> Vec<Triangle3d> {
        let length_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0);

        let head_len = length_world * 0.18;
        let head_radius = length_world * 0.07;

        let mut out = Vec::new();

        for &((axis_dir, color), handle) in &[
            ((axes[0], self.config.x_color), HandleId(1)),
            ((axes[1], self.config.y_color), HandleId(2)),
            ((axes[2], self.config.z_color), HandleId(3)),
        ] {
            let axis_dir = axis_dir.normalize_or_zero();
            if axis_dir.length_squared() == 0.0 {
                continue;
            }
            let c = if self.is_handle_highlighted(GizmoMode::Translate, handle) {
                self.config.hover_color
            } else {
                color
            };

            let tip = origin + axis_dir * length_world;
            let base = tip - axis_dir * head_len;
            let (u, v) = plane_basis(axis_dir);
            let s = head_radius * 0.70710677;
            let c0 = base + (u + v) * s;
            let c1 = base + (-u + v) * s;
            let c2 = base + (-u - v) * s;
            let c3 = base + (u - v) * s;

            self.push_tri(&mut out, tip, c0, c1, c, self.config.depth_mode);
            self.push_tri(&mut out, tip, c1, c2, c, self.config.depth_mode);
            self.push_tri(&mut out, tip, c2, c3, c, self.config.depth_mode);
            self.push_tri(&mut out, tip, c3, c0, c, self.config.depth_mode);
        }

        // Plane handle fills.
        let off = length_world * 0.15;
        let size = length_world * 0.25;
        for &(u, v, base_color, handle) in &[
            (
                axes[0],
                axes[1],
                mix_alpha(self.config.z_color, 0.55),
                TranslateHandle::PlaneXY,
            ), // XY
            (
                axes[0],
                axes[2],
                mix_alpha(self.config.y_color, 0.55),
                TranslateHandle::PlaneXZ,
            ), // XZ
            (
                axes[1],
                axes[2],
                mix_alpha(self.config.x_color, 0.55),
                TranslateHandle::PlaneYZ,
            ), // YZ
        ] {
            let handle_id = handle.id();
            let outline = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
                mix_alpha(self.config.hover_color, 0.85)
            } else {
                base_color
            };
            let fill = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
                mix_alpha(outline, 0.55)
            } else {
                mix_alpha(outline, 0.30)
            };
            let quad = translate_plane_quad_world(origin, u, v, off, size);
            self.push_tri(
                &mut out,
                quad[0],
                quad[1],
                quad[2],
                fill,
                self.config.depth_mode,
            );
            self.push_tri(
                &mut out,
                quad[0],
                quad[2],
                quad[3],
                fill,
                self.config.depth_mode,
            );
        }

        // Screen translate fill (center handle), always-on-top.
        if let Some(view_dir) =
            view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
        {
            let (u, v) = plane_basis(view_dir);
            let half = length_world * 0.08;
            let handle_id = TranslateHandle::Screen.id();
            let outline = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
                mix_alpha(self.config.hover_color, 1.0)
            } else {
                mix_alpha(self.config.hover_color, 0.65)
            };
            let fill = if self.is_handle_highlighted(GizmoMode::Translate, handle_id) {
                mix_alpha(outline, 0.65)
            } else {
                mix_alpha(outline, 0.35)
            };

            let p0 = origin + (-u - v) * half;
            let p1 = origin + (u - v) * half;
            let p2 = origin + (u + v) * half;
            let p3 = origin + (-u + v) * half;
            self.push_tri(&mut out, p0, p1, p2, fill, DepthMode::Always);
            self.push_tri(&mut out, p0, p2, p3, fill, DepthMode::Always);
        }

        out
    }

    fn draw_rotate_rings(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
    ) -> Vec<Line3d> {
        let radius_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0);

        let segments: usize = 64;
        let mut out = Vec::with_capacity(segments * 3);

        for &((axis_dir, color), handle) in &[
            ((axes[0], self.config.x_color), HandleId(1)),
            ((axes[1], self.config.y_color), HandleId(2)),
            ((axes[2], self.config.z_color), HandleId(3)),
        ] {
            let axis_dir = axis_dir.normalize_or_zero();
            if axis_dir.length_squared() == 0.0 {
                continue;
            }
            let (u, v) = plane_basis(axis_dir);
            let c = if self.is_handle_highlighted(GizmoMode::Rotate, handle) {
                self.config.hover_color
            } else {
                color
            };

            let mut prev = origin + u * radius_world;
            for i in 1..=segments {
                let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                let p = origin + (u * t.cos() + v * t.sin()) * radius_world;
                self.push_line(&mut out, prev, p, c, self.config.depth_mode);
                prev = p;
            }
        }

        if self.config.show_view_axis_ring {
            if let Some(view_dir) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            {
                let axis_dir = view_dir.normalize_or_zero();
                if axis_dir.length_squared() > 0.0 {
                    let (u, v) = plane_basis(axis_dir);
                    let handle = HandleId(8);
                    let r = (radius_world * self.config.view_axis_ring_radius_scale).max(1e-6);
                    let base = Color {
                        r: 0.9,
                        g: 0.9,
                        b: 0.9,
                        a: 0.8,
                    };
                    let c = if self.is_handle_highlighted(GizmoMode::Rotate, handle) {
                        self.config.hover_color
                    } else {
                        base
                    };

                    let mut prev = origin + u * r;
                    for i in 1..=segments {
                        let t = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                        let p = origin + (u * t.cos() + v * t.sin()) * r;
                        self.push_line(&mut out, prev, p, c, DepthMode::Always);
                        prev = p;
                    }
                }
            }
        }

        out
    }

    fn draw_rotate_feedback(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
    ) -> GizmoDrawList3d {
        if self.state.drag_mode != GizmoMode::Rotate {
            return GizmoDrawList3d::default();
        }
        let Some(active) = self.state.active else {
            return GizmoDrawList3d::default();
        };
        if !self.state.drag_has_started {
            return GizmoDrawList3d::default();
        }

        let axis_dir = self.state.drag_axis_dir.normalize_or_zero();
        if axis_dir.length_squared() == 0.0 {
            return GizmoDrawList3d::default();
        }

        let base_radius_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0)
        .max(1e-6);
        let radius_world = if active.0 == 8 {
            (base_radius_world * self.config.view_axis_ring_radius_scale).max(1e-6)
        } else {
            base_radius_world
        };

        let thickness_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.line_thickness_px,
        )
        .unwrap_or(radius_world * 0.04);

        let half = (thickness_world * 1.15)
            .clamp(radius_world * 0.015, radius_world * 0.12)
            .max(1e-6);
        let inner_r = (radius_world - half).max(radius_world * 0.2);
        let outer_r = radius_world + half;

        let base = match active.0 {
            1 => self.config.x_color,
            2 => self.config.y_color,
            3 => self.config.z_color,
            8 => Color {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 0.8,
            },
            _ => self.config.hover_color,
        };
        let color = if self.state.active == Some(active) {
            self.config.hover_color
        } else {
            base
        };

        let total = self.state.drag_total_angle_applied;
        let start = self.state.drag_start_angle;
        let end = start + total;

        let u = self.state.drag_basis_u.normalize_or_zero();
        let v = self.state.drag_basis_v.normalize_or_zero();
        if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
            return GizmoDrawList3d::default();
        }

        let mut out = GizmoDrawList3d::default();

        if total.abs() > 1e-6 {
            let segs = ((total.abs() / std::f32::consts::TAU) * 96.0)
                .ceil()
                .clamp(12.0, 192.0) as usize;
            let step = total / (segs as f32);

            let fill_alpha = if self.state.drag_snap { 0.30 } else { 0.22 };
            let fill = mix_alpha(color, fill_alpha);
            let edge = mix_alpha(color, 0.95);

            let point =
                |theta: f32, r: f32| -> Vec3 { origin + (u * theta.cos() + v * theta.sin()) * r };

            let mut prev_outer = point(start, outer_r);
            for i in 0..segs {
                let t0 = start + step * (i as f32);
                let t1 = start + step * ((i + 1) as f32);
                let o0 = point(t0, outer_r);
                let i0 = point(t0, inner_r);
                let o1 = point(t1, outer_r);
                let i1 = point(t1, inner_r);

                self.push_tri(&mut out.triangles, o0, i0, i1, fill, DepthMode::Always);
                self.push_tri(&mut out.triangles, o0, i1, o1, fill, DepthMode::Always);

                self.push_line(&mut out.lines, prev_outer, o1, edge, DepthMode::Always);
                prev_outer = o1;
            }

            let start_dir = (u * start.cos() + v * start.sin()).normalize_or_zero();
            let end_dir = (u * end.cos() + v * end.sin()).normalize_or_zero();
            if start_dir.length_squared() > 0.0 {
                self.push_line(
                    &mut out.lines,
                    origin,
                    origin + start_dir * radius_world,
                    mix_alpha(color, 0.35),
                    DepthMode::Always,
                );
            }
            if end_dir.length_squared() > 0.0 {
                self.push_line(
                    &mut out.lines,
                    origin,
                    origin + end_dir * radius_world,
                    mix_alpha(color, 0.75),
                    DepthMode::Always,
                );
                self.push_line(
                    &mut out.lines,
                    origin + end_dir * inner_r,
                    origin + end_dir * (outer_r + half * 0.8),
                    edge,
                    DepthMode::Always,
                );
            }
        }

        if self.state.drag_snap {
            if let Some(step) = self
                .config
                .rotate_snap_step_radians
                .filter(|s| s.is_finite() && *s > 0.0)
            {
                let ticks = (std::f32::consts::TAU / step).round() as usize;
                if (4..=128).contains(&ticks) {
                    let tick_color = Color {
                        r: 0.9,
                        g: 0.9,
                        b: 0.9,
                        a: 0.35,
                    };
                    for k in 0..ticks {
                        let t = (k as f32) * step;
                        let dir = (u * t.cos() + v * t.sin()).normalize_or_zero();
                        if dir.length_squared() == 0.0 {
                            continue;
                        }
                        let a = origin + dir * (outer_r + half * 0.8);
                        let b = origin + dir * (outer_r + half * 2.2);
                        self.push_line(&mut out.lines, a, b, tick_color, DepthMode::Always);
                    }
                }
            }
        }

        out
    }

    fn draw_scale_handles(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
    ) -> Vec<Line3d> {
        let length_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0);

        let (u, v) = view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            .map(plane_basis)
            .unwrap_or((Vec3::X, Vec3::Y));

        let mut out = Vec::new();

        for &((axis_dir, color), handle) in &[
            ((axes[0], self.config.x_color), HandleId(1)),
            ((axes[1], self.config.y_color), HandleId(2)),
            ((axes[2], self.config.z_color), HandleId(3)),
        ] {
            let c = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
                self.config.hover_color
            } else {
                color
            };

            let end = origin + axis_dir * length_world;
            self.push_line(&mut out, origin, end, c, self.config.depth_mode);

            // End box, screen-facing.
            let half = length_world * 0.06;
            let p0 = end + (-u - v) * half;
            let p1 = end + (u - v) * half;
            let p2 = end + (u + v) * half;
            let p3 = end + (-u + v) * half;
            for (a, b) in [(p0, p1), (p1, p2), (p2, p3), (p3, p0)] {
                self.push_line(&mut out, a, b, c, self.config.depth_mode);
            }
        }

        // Uniform scale box at the origin (screen-facing).
        let handle = HandleId(7);
        let base = mix_alpha(self.config.hover_color, 0.65);
        let c = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
            self.config.hover_color
        } else {
            base
        };
        let half = length_world * 0.08;
        let p0 = origin + (-u - v) * half;
        let p1 = origin + (u - v) * half;
        let p2 = origin + (u + v) * half;
        let p3 = origin + (-u + v) * half;
        for (a, b) in [(p0, p1), (p1, p2), (p2, p3), (p3, p0)] {
            self.push_line(&mut out, a, b, c, DepthMode::Always);
        }

        out
    }

    fn draw_scale_solids(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        axes: [Vec3; 3],
    ) -> Vec<Triangle3d> {
        let length_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0);

        let (u, v) = view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            .map(plane_basis)
            .unwrap_or((Vec3::X, Vec3::Y));

        let mut out = Vec::new();

        // Axis end boxes (screen-facing filled quads).
        for &((axis_dir, color), handle) in &[
            ((axes[0], self.config.x_color), HandleId(1)),
            ((axes[1], self.config.y_color), HandleId(2)),
            ((axes[2], self.config.z_color), HandleId(3)),
        ] {
            let outline = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
                self.config.hover_color
            } else {
                color
            };
            let fill = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
                mix_alpha(outline, 0.70)
            } else {
                mix_alpha(outline, 0.45)
            };

            let end = origin + axis_dir * length_world;
            let half = length_world * 0.06;
            let p0 = end + (-u - v) * half;
            let p1 = end + (u - v) * half;
            let p2 = end + (u + v) * half;
            let p3 = end + (-u + v) * half;
            self.push_tri(&mut out, p0, p1, p2, fill, self.config.depth_mode);
            self.push_tri(&mut out, p0, p2, p3, fill, self.config.depth_mode);
        }

        // Uniform scale at the origin (screen-facing filled quad), always-on-top.
        let handle = HandleId(7);
        let outline = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
            self.config.hover_color
        } else {
            mix_alpha(self.config.hover_color, 0.65)
        };
        let fill = if self.is_handle_highlighted(GizmoMode::Scale, handle) {
            mix_alpha(outline, 0.75)
        } else {
            mix_alpha(outline, 0.40)
        };

        let half = length_world * 0.08;
        let p0 = origin + (-u - v) * half;
        let p1 = origin + (u - v) * half;
        let p2 = origin + (u + v) * half;
        let p3 = origin + (-u + v) * half;
        self.push_tri(&mut out, p0, p1, p2, fill, DepthMode::Always);
        self.push_tri(&mut out, p0, p2, p3, fill, DepthMode::Always);

        out
    }

    fn pick_translate_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
    ) -> Option<PickHit> {
        let length_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0);

        // Picking priority ladder (editor UX):
        // 1) Center / screen-plane handle (when within radius, always win)
        // 2) Plane handles (when cursor is inside the plane quad)
        // 3) Axis handles (distance to segment)
        //
        // This avoids a common frustration where the axis segment "steals" clicks near the origin.
        if let Some(p0) = project_point(view_projection, viewport, origin, self.config.depth_range)
        {
            let d = (cursor - p0.screen).length();
            let r = self.config.pick_radius_px.max(6.0);
            if d.is_finite() && d <= r {
                return Some(PickHit {
                    handle: TranslateHandle::Screen.id(),
                    score: d,
                });
            }
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
        for &(axis_dir, handle) in &[
            (axes[0], TranslateHandle::AxisX.id()),
            (axes[1], TranslateHandle::AxisY.id()),
            (axes[2], TranslateHandle::AxisZ.id()),
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
                consider(handle, d);
            }
        }

        // Plane handles (distance to projected quad; accept when inside).
        let off = length_world * 0.15;
        let size = length_world * 0.25;
        for &(u, v, handle) in &[
            (axes[0], axes[1], TranslateHandle::PlaneXY.id()),
            (axes[0], axes[2], TranslateHandle::PlaneXZ.id()),
            (axes[1], axes[2], TranslateHandle::PlaneYZ.id()),
        ] {
            let world = translate_plane_quad_world(origin, u, v, off, size);
            let Some(p) = project_quad(view_projection, viewport, world, self.config.depth_range)
            else {
                continue;
            };

            let inside = point_in_convex_quad(cursor, p);
            let edge_d = quad_edge_distance(cursor, p);
            if inside {
                // Prefer plane drags when the cursor is actually inside the plane handle quad.
                consider(handle, 0.20);
            } else if edge_d <= self.config.pick_radius_px {
                consider(handle, edge_d + 0.9);
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
    ) -> Option<PickHit> {
        let length_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )
        .unwrap_or(1.0);

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
        if let Some(p0) = project_point(view_projection, viewport, origin, self.config.depth_range)
        {
            let d = (cursor - p0.screen).length();
            let r = self.config.pick_radius_px.max(6.0);
            if d <= r {
                return Some(PickHit {
                    handle: HandleId(7),
                    score: d,
                });
            }
        }

        // Axis scaling handles.
        let (u, v) = view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            .map(plane_basis)
            .unwrap_or((Vec3::X, Vec3::Y));
        for &(axis_dir, handle) in &[
            (axes[0], HandleId(1)),
            (axes[1], HandleId(2)),
            (axes[2], HandleId(3)),
        ] {
            let end = origin + axis_dir * length_world;
            let half = length_world * 0.06;
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
            let inside = point_in_convex_quad(cursor, p);
            let edge_d = quad_edge_distance(cursor, p);
            if inside {
                consider(handle, 0.0);
            } else if edge_d <= self.config.pick_radius_px {
                consider(handle, edge_d);
            }
        }

        best
    }

    fn pick_universal_handle(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
    ) -> Option<(PickHit, GizmoMode)> {
        let translate = self
            .pick_translate_handle(view_projection, viewport, origin, cursor, axes)
            .map(|h| (h, GizmoMode::Translate));
        let rotate = self
            .pick_rotate_axis(view_projection, viewport, origin, cursor, axes)
            .map(|h| (h, GizmoMode::Rotate));

        match (translate, rotate) {
            (Some(a), Some(b)) => {
                // Universal mode overlays translate planes/center with rotation rings; bias toward
                // rotate when both hits are plausible (closer to the UX of mature editors).
                let translate_bias = match a.0.handle.0 {
                    4 | 5 | 6 | 10 => self.config.pick_radius_px * 0.75,
                    _ => 0.0,
                };
                let a_score = a.0.score + translate_bias;
                Some(if a_score <= b.0.score { a } else { b })
            }
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    fn pick_rotate_axis(
        &self,
        view_projection: Mat4,
        viewport: ViewportRect,
        origin: Vec3,
        cursor: Vec2,
        axes: [Vec3; 3],
    ) -> Option<PickHit> {
        let radius_world = axis_length_world(
            view_projection,
            viewport,
            origin,
            self.config.depth_range,
            self.config.size_px,
        )?;

        let segments: usize = 64;
        let mut best_axis: Option<PickHit> = None;

        for &(axis_dir, handle) in &[
            (axes[0], HandleId(1)),
            (axes[1], HandleId(2)),
            (axes[2], HandleId(3)),
        ] {
            let axis_dir = axis_dir.normalize_or_zero();
            if axis_dir.length_squared() == 0.0 {
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
                    let d = distance_point_to_segment_px(cursor, prev.screen, p.screen);
                    if d <= self.config.pick_radius_px {
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

        let mut view_hit: Option<PickHit> = None;
        if self.config.show_view_axis_ring {
            if let Some(view_dir) =
                view_dir_at_origin(view_projection, viewport, origin, self.config.depth_range)
            {
                let axis_dir = view_dir.normalize_or_zero();
                if axis_dir.length_squared() > 0.0 {
                    let (u, v) = plane_basis(axis_dir);
                    let handle = HandleId(8);
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
                            let d = distance_point_to_segment_px(cursor, prev.screen, p.screen);
                            if d <= self.config.pick_radius_px {
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

        match (best_axis, view_hit) {
            (Some(axis), Some(view)) => {
                // View ring is an outer, always-on-top affordance. Make it slightly easier to hit
                // without stealing clearly-intended axis ring drags.
                let view_score = (view.score - 0.35).max(0.0);
                if view_score <= axis.score {
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
        }
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

fn point_in_convex_quad(p: Vec2, q: [Vec2; 4]) -> bool {
    fn cross(a: Vec2, b: Vec2) -> f32 {
        a.x * b.y - a.y * b.x
    }

    let mut sign = 0.0f32;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let c = cross(b - a, p - a);
        if c.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = c;
        } else if sign.signum() != c.signum() {
            return false;
        }
    }
    true
}

fn quad_edge_distance(p: Vec2, q: [Vec2; 4]) -> f32 {
    let mut best = f32::INFINITY;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        best = best.min(distance_point_to_segment_px(p, a, b));
    }
    best
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_view_projection(viewport_px: (f32, f32)) -> Mat4 {
        let aspect = viewport_px.0.max(1.0) / viewport_px.1.max(1.0);
        let eye = Vec3::new(3.0, 2.0, 4.0);
        let target = Vec3::ZERO;
        let view = Mat4::look_at_rh(eye, target, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), aspect, 0.05, 100.0);
        proj * view
    }

    fn base_gizmo(mode: GizmoMode) -> Gizmo {
        let mut config = GizmoConfig::default();
        config.mode = mode;
        config.depth_range = DepthRange::ZeroToOne;
        config.drag_start_threshold_px = 0.0;
        Gizmo::new(config)
    }

    #[test]
    fn translate_center_handle_wins_near_origin() {
        let gizmo = base_gizmo(GizmoMode::Translate);
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let view_proj = test_view_projection((800.0, 600.0));
        let origin = Vec3::ZERO;
        let axes = gizmo.axis_dirs(&Transform3d::default());

        let p0 = project_point(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
        let hit = gizmo
            .pick_translate_handle(view_proj, vp, origin, p0.screen, axes)
            .unwrap();
        assert_eq!(hit.handle, TranslateHandle::Screen.id());
    }

    #[test]
    fn translate_axis_drag_returns_to_zero_when_cursor_returns() {
        let mut gizmo = base_gizmo(GizmoMode::Translate);
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let view_proj = test_view_projection((800.0, 600.0));

        let origin = Vec3::ZERO;
        let axes = gizmo.axis_dirs(&Transform3d::default());
        let length_world = axis_length_world(
            view_proj,
            vp,
            origin,
            gizmo.config.depth_range,
            gizmo.config.size_px,
        )
        .unwrap();

        let a = origin;
        let b = origin + axes[0].normalize_or_zero() * length_world;
        let pa = project_point(view_proj, vp, a, gizmo.config.depth_range).unwrap();
        let pb = project_point(view_proj, vp, b, gizmo.config.depth_range).unwrap();
        let axis_dir_screen = (pb.screen - pa.screen).normalize_or_zero();
        assert!(axis_dir_screen.length_squared() > 0.0);

        let cursor_start = pa.screen.lerp(pb.screen, 0.5);
        let cursor_moved = cursor_start + axis_dir_screen * 40.0;

        let targets = [GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d::default(),
        }];

        let input_down = GizmoInput {
            cursor_px: cursor_start,
            hovered: true,
            drag_started: true,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

        let input_move = GizmoInput {
            cursor_px: cursor_moved,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let moved = gizmo
            .update(view_proj, vp, input_move, targets[0].id, &targets)
            .unwrap();

        let moved_total = match moved.result {
            GizmoResult::Translation { total, .. } => total,
            _ => panic!("expected translation"),
        };
        assert!(moved_total.x.is_finite());
        assert!(moved_total.x > 0.0);
        assert!(moved_total.y.abs() < 1e-3);
        assert!(moved_total.z.abs() < 1e-3);

        let input_back = GizmoInput {
            cursor_px: cursor_start,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let back = gizmo
            .update(view_proj, vp, input_back, targets[0].id, &targets)
            .unwrap();

        let back_total = match back.result {
            GizmoResult::Translation { total, .. } => total,
            _ => panic!("expected translation"),
        };
        assert!(back_total.length() < 1e-3, "total={back_total:?}");
    }

    #[test]
    fn translate_plane_drag_returns_to_zero_when_cursor_returns() {
        let mut gizmo = base_gizmo(GizmoMode::Translate);
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let view_proj = test_view_projection((800.0, 600.0));

        let origin = Vec3::ZERO;
        let axes = gizmo.axis_dirs(&Transform3d::default());
        let length_world = axis_length_world(
            view_proj,
            vp,
            origin,
            gizmo.config.depth_range,
            gizmo.config.size_px,
        )
        .unwrap();

        let off = length_world * 0.15;
        let size = length_world * 0.25;
        let quad_world = translate_plane_quad_world(origin, axes[0], axes[1], off, size);
        let quad_screen =
            project_quad(view_proj, vp, quad_world, gizmo.config.depth_range).unwrap();
        let cursor_start =
            (quad_screen[0] + quad_screen[1] + quad_screen[2] + quad_screen[3]) * 0.25;
        let cursor_moved = cursor_start + Vec2::new(25.0, -15.0);

        let targets = [GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d::default(),
        }];

        let input_down = GizmoInput {
            cursor_px: cursor_start,
            hovered: true,
            drag_started: true,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

        let input_move = GizmoInput {
            cursor_px: cursor_moved,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let moved = gizmo
            .update(view_proj, vp, input_move, targets[0].id, &targets)
            .unwrap();

        let moved_total = match moved.result {
            GizmoResult::Translation { total, .. } => total,
            _ => panic!("expected translation"),
        };
        assert!(moved_total.length() > 1e-6);

        let input_back = GizmoInput {
            cursor_px: cursor_start,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let back = gizmo
            .update(view_proj, vp, input_back, targets[0].id, &targets)
            .unwrap();

        let back_total = match back.result {
            GizmoResult::Translation { total, .. } => total,
            _ => panic!("expected translation"),
        };
        assert!(back_total.length() < 1e-3, "total={back_total:?}");
    }

    #[test]
    fn rotate_axis_drag_returns_to_zero_when_cursor_returns() {
        let mut gizmo = base_gizmo(GizmoMode::Rotate);
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let view_proj = test_view_projection((800.0, 600.0));

        let origin = Vec3::ZERO;
        let axes = gizmo.axis_dirs(&Transform3d::default());
        let radius_world = axis_length_world(
            view_proj,
            vp,
            origin,
            gizmo.config.depth_range,
            gizmo.config.size_px,
        )
        .unwrap();

        let axis_dir = axes[0].normalize_or_zero();
        let (u, v) = plane_basis(axis_dir);
        let p_start_world = origin + u * radius_world;
        let p_move_world = origin + (u * 0.98 + v * 0.2).normalize_or_zero() * radius_world;

        let p_start =
            project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
        let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

        let targets = [GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d::default(),
        }];

        let input_down = GizmoInput {
            cursor_px: p_start.screen,
            hovered: true,
            drag_started: true,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

        let input_move = GizmoInput {
            cursor_px: p_move.screen,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let moved = gizmo
            .update(view_proj, vp, input_move, targets[0].id, &targets)
            .unwrap();
        let moved_total = match moved.result {
            GizmoResult::Rotation { total_radians, .. } => total_radians,
            _ => panic!("expected rotation"),
        };
        assert!(moved_total.is_finite());
        assert!(moved_total.abs() > 1e-6);

        let input_back = GizmoInput {
            cursor_px: p_start.screen,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let back = gizmo
            .update(view_proj, vp, input_back, targets[0].id, &targets)
            .unwrap();
        let back_total = match back.result {
            GizmoResult::Rotation { total_radians, .. } => total_radians,
            _ => panic!("expected rotation"),
        };
        assert!(back_total.abs() < 1e-3, "total={back_total}");
    }

    #[test]
    fn scale_axis_drag_returns_to_one_when_cursor_returns() {
        let mut gizmo = base_gizmo(GizmoMode::Scale);
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let view_proj = test_view_projection((800.0, 600.0));

        let origin = Vec3::ZERO;
        let axes = gizmo.axis_dirs(&Transform3d::default());
        let length_world = axis_length_world(
            view_proj,
            vp,
            origin,
            gizmo.config.depth_range,
            gizmo.config.size_px,
        )
        .unwrap();

        let axis_dir = axes[0].normalize_or_zero();
        assert!(axis_dir.length_squared() > 0.0);
        let p_start_world = origin + axis_dir * length_world;
        let p_move_world = origin + axis_dir * (length_world * 1.35);

        let p_start =
            project_point(view_proj, vp, p_start_world, gizmo.config.depth_range).unwrap();
        let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

        let targets = [GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d::default(),
        }];

        let input_down = GizmoInput {
            cursor_px: p_start.screen,
            hovered: true,
            drag_started: true,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

        let input_move = GizmoInput {
            cursor_px: p_move.screen,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let moved = gizmo
            .update(view_proj, vp, input_move, targets[0].id, &targets)
            .unwrap();
        let moved_total = match moved.result {
            GizmoResult::Scale { total, .. } => total,
            _ => panic!("expected scale"),
        };
        assert!(moved_total.x.is_finite());
        assert!(moved_total.x > 1.0 + 1e-6);

        let input_back = GizmoInput {
            cursor_px: p_start.screen,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let back = gizmo
            .update(view_proj, vp, input_back, targets[0].id, &targets)
            .unwrap();
        let back_total = match back.result {
            GizmoResult::Scale { total, .. } => total,
            _ => panic!("expected scale"),
        };
        assert!(
            (back_total - Vec3::ONE).length() < 1e-3,
            "total={back_total:?}"
        );
    }

    #[test]
    fn scale_uniform_drag_returns_to_one_when_cursor_returns() {
        let mut gizmo = base_gizmo(GizmoMode::Scale);
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let view_proj = test_view_projection((800.0, 600.0));

        let origin = Vec3::ZERO;
        let p0 = project_point(view_proj, vp, origin, gizmo.config.depth_range).unwrap();

        let radius_world = axis_length_world(
            view_proj,
            vp,
            origin,
            gizmo.config.depth_range,
            gizmo.config.size_px,
        )
        .unwrap();
        let view_dir = view_dir_at_origin(view_proj, vp, origin, gizmo.config.depth_range).unwrap();
        let (u, v) = plane_basis(view_dir);
        let dir = (u + v).normalize_or_zero();
        assert!(dir.length_squared() > 0.0);
        let p_move_world = origin + dir * (radius_world * 0.6);
        let p_move = project_point(view_proj, vp, p_move_world, gizmo.config.depth_range).unwrap();

        let targets = [GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: Transform3d::default(),
        }];

        let input_down = GizmoInput {
            cursor_px: p0.screen,
            hovered: true,
            drag_started: true,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let _ = gizmo.update(view_proj, vp, input_down, targets[0].id, &targets);

        let input_move = GizmoInput {
            cursor_px: p_move.screen,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let moved = gizmo
            .update(view_proj, vp, input_move, targets[0].id, &targets)
            .unwrap();
        let moved_total = match moved.result {
            GizmoResult::Scale { total, .. } => total,
            _ => panic!("expected scale"),
        };
        assert!(moved_total.x.is_finite());
        assert!(moved_total.x > 1.0 + 1e-6);
        assert!((moved_total.x - moved_total.y).abs() < 1e-5);
        assert!((moved_total.x - moved_total.z).abs() < 1e-5);

        let input_back = GizmoInput {
            cursor_px: p0.screen,
            hovered: true,
            drag_started: false,
            dragging: true,
            snap: false,
            cancel: false,
        };
        let back = gizmo
            .update(view_proj, vp, input_back, targets[0].id, &targets)
            .unwrap();
        let back_total = match back.result {
            GizmoResult::Scale { total, .. } => total,
            _ => panic!("expected scale"),
        };
        assert!(
            (back_total - Vec3::ONE).length() < 1e-3,
            "total={back_total:?}"
        );
    }
}
