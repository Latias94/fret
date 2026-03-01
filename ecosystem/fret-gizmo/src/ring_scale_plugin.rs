use fret_core::Color;
use glam::{Mat4, Vec2, Vec3};

use crate::math::{DepthRange, ViewportRect, project_point, ray_from_screen, unproject_point};
use crate::picking::PickSegmentCapsule2d;
use crate::{
    DepthMode, GizmoDrawList3d, GizmoPhase, GizmoPickItem, GizmoPickShape2d, GizmoPlugin,
    GizmoPluginContext, GizmoPluginId, GizmoResult, GizmoTarget3d, GizmoTargetId, GizmoUpdate,
    HandleId, Line3d,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RingScaleGizmoConfig {
    pub radius_px: f32,
    pub pick_radius_px: f32,
    pub segments: usize,
    pub depth: DepthMode,
    pub color: Color,
    pub hover_color: Color,
    pub scale_snap_step: Option<f32>,
}

impl Default for RingScaleGizmoConfig {
    fn default() -> Self {
        Self {
            radius_px: 160.0,
            pick_radius_px: 10.0,
            segments: 64,
            depth: DepthMode::Always,
            color: Color {
                a: 0.75,
                ..Color::from_srgb_hex_rgb(0xf2_d1_40)
            },
            hover_color: Color {
                a: 0.95,
                ..Color::from_srgb_hex_rgb(0xff_eb_6b)
            },
            scale_snap_step: Some(0.1),
        }
    }
}

#[derive(Debug, Default)]
pub struct RingScaleGizmoState {
    pub active: Option<HandleId>,
    pub total_factor_applied: f32,
    drag_start_dist_px: f32,
    drag_start_targets: Vec<GizmoTarget3d>,
    drag_origin: Vec3,
}

#[derive(Debug, Default)]
pub struct RingScaleGizmoPlugin {
    pub config: RingScaleGizmoConfig,
    pub state: RingScaleGizmoState,
}

impl RingScaleGizmoPlugin {
    pub const PLUGIN_ID: GizmoPluginId = GizmoPluginId(1);
    pub const RING_HANDLE_LOCAL: u32 = 1;

    pub fn ring_handle() -> HandleId {
        HandleId::from_parts(Self::PLUGIN_ID, Self::RING_HANDLE_LOCAL)
    }

    fn active_origin(active_target: GizmoTargetId, targets: &[GizmoTarget3d]) -> Vec3 {
        targets
            .iter()
            .find(|t| t.id == active_target)
            .or_else(|| targets.first())
            .map(|t| t.transform.translation)
            .unwrap_or(Vec3::ZERO)
    }

    fn plane_basis(normal: Vec3) -> (Vec3, Vec3) {
        let n = normal.normalize_or_zero();
        let a = if n.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let u = n.cross(a).normalize_or_zero();
        let v = n.cross(u).normalize_or_zero();
        (u, v)
    }

    fn ndc_z_to_z01(depth_range: DepthRange, ndc_z: f32) -> f32 {
        match depth_range {
            DepthRange::ZeroToOne => ndc_z,
            DepthRange::NegOneToOne => (ndc_z + 1.0) * 0.5,
        }
        .clamp(0.0, 1.0)
    }

    fn world_per_px(
        view_projection: Mat4,
        viewport: ViewportRect,
        depth_range: DepthRange,
        origin: Vec3,
    ) -> Option<f32> {
        let p0 = project_point(view_projection, viewport, origin, depth_range)?;
        let z01 = Self::ndc_z_to_z01(depth_range, p0.ndc_z);
        let a = unproject_point(view_projection, viewport, p0.screen, depth_range, z01)?;
        let b = unproject_point(
            view_projection,
            viewport,
            p0.screen + Vec2::new(1.0, 0.0),
            depth_range,
            z01,
        )?;
        let d = (b - a).length();
        (d.is_finite() && d > 1e-7).then_some(d)
    }

    fn snap_factor(&self, input: &crate::GizmoInput, factor: f32) -> f32 {
        let mut factor = factor.max(0.01);
        if input.snap
            && let Some(step) = self
                .config
                .scale_snap_step
                .filter(|s| s.is_finite() && *s > 0.0)
        {
            factor = 1.0 + ((factor - 1.0) / step).round() * step;
            factor = factor.max(0.01);
        }
        factor
    }

    fn input_precision(precision: f32) -> f32 {
        if precision.is_finite() {
            precision.clamp(0.01, 100.0)
        } else {
            1.0
        }
    }
}

impl GizmoPlugin for RingScaleGizmoPlugin {
    fn plugin_id(&self) -> GizmoPluginId {
        Self::PLUGIN_ID
    }

    fn draw(
        &mut self,
        ctx: GizmoPluginContext<'_>,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> GizmoDrawList3d {
        if targets.is_empty() {
            return GizmoDrawList3d::default();
        }

        let origin = Self::active_origin(active_target, targets);
        let Some(p0) = project_point(ctx.view_projection, ctx.viewport, origin, ctx.depth_range)
        else {
            return GizmoDrawList3d::default();
        };
        let Some(world_per_px) =
            Self::world_per_px(ctx.view_projection, ctx.viewport, ctx.depth_range, origin)
        else {
            return GizmoDrawList3d::default();
        };
        let Some(ray) = ray_from_screen(
            ctx.view_projection,
            ctx.viewport,
            p0.screen,
            ctx.depth_range,
        ) else {
            return GizmoDrawList3d::default();
        };

        let (u, v) = Self::plane_basis(ray.dir);
        if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
            return GizmoDrawList3d::default();
        }

        let r_world = world_per_px * self.config.radius_px.max(0.0);
        if !r_world.is_finite() || r_world <= 1e-6 {
            return GizmoDrawList3d::default();
        }

        let handle = Self::ring_handle();
        let highlighted = ctx.active == Some(handle) || ctx.hovered == Some(handle);
        let c = if highlighted {
            self.config.hover_color
        } else {
            self.config.color
        };

        let segs = self.config.segments.clamp(3, 256);
        let mut out = GizmoDrawList3d::default();
        let mut prev = origin + u * r_world;
        for i in 1..=segs {
            let t = (i as f32) / (segs as f32) * std::f32::consts::TAU;
            let next = origin + (u * t.cos() + v * t.sin()) * r_world;
            out.lines.push(Line3d {
                a: prev,
                b: next,
                color: c,
                depth: self.config.depth,
            });
            prev = next;
        }
        out
    }

    fn pick_items(
        &mut self,
        ctx: GizmoPluginContext<'_>,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
        out: &mut Vec<GizmoPickItem>,
    ) {
        if !ctx.input.hovered || targets.is_empty() {
            return;
        }

        let origin = Self::active_origin(active_target, targets);
        let Some(p0) = project_point(ctx.view_projection, ctx.viewport, origin, ctx.depth_range)
        else {
            return;
        };
        let Some(world_per_px) =
            Self::world_per_px(ctx.view_projection, ctx.viewport, ctx.depth_range, origin)
        else {
            return;
        };
        let Some(ray) = ray_from_screen(
            ctx.view_projection,
            ctx.viewport,
            p0.screen,
            ctx.depth_range,
        ) else {
            return;
        };
        let (u, v) = Self::plane_basis(ray.dir);
        if u.length_squared() == 0.0 || v.length_squared() == 0.0 {
            return;
        }

        let r_world = world_per_px * self.config.radius_px.max(0.0);
        if !r_world.is_finite() || r_world <= 1e-6 {
            return;
        }

        let segs = self.config.segments.clamp(3, 256);
        let handle = Self::ring_handle();
        let radius = self.config.pick_radius_px.max(0.5);

        let start_world = origin + u * r_world;
        let mut prev_screen = project_point(
            ctx.view_projection,
            ctx.viewport,
            start_world,
            ctx.depth_range,
        )
        .map(|p| p.screen);
        for i in 1..=segs {
            let t = (i as f32) / (segs as f32) * std::f32::consts::TAU;
            let world = origin + (u * t.cos() + v * t.sin()) * r_world;
            let screen = project_point(ctx.view_projection, ctx.viewport, world, ctx.depth_range)
                .map(|p| p.screen);

            if let (Some(a), Some(b)) = (prev_screen, screen) {
                out.push(GizmoPickItem {
                    handle,
                    shape: GizmoPickShape2d::SegmentCapsule(PickSegmentCapsule2d { a, b, radius }),
                    bias_px: 0.0,
                });
            }

            prev_screen = screen;
        }
    }

    fn update(
        &mut self,
        ctx: GizmoPluginContext<'_>,
        phase: GizmoPhase,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
        active_handle: HandleId,
    ) -> Option<GizmoUpdate> {
        if active_handle != Self::ring_handle() || targets.is_empty() {
            return None;
        }

        let origin = match phase {
            GizmoPhase::Begin => Self::active_origin(active_target, targets),
            _ => self.state.drag_origin,
        };
        let p0 = project_point(ctx.view_projection, ctx.viewport, origin, ctx.depth_range)?;

        match phase {
            GizmoPhase::Begin => {
                self.state.active = Some(active_handle);
                self.state.total_factor_applied = 1.0;
                self.state.drag_origin = origin;
                self.state.drag_start_targets = targets.to_vec();

                let start_dist = (ctx.drag_start_cursor_px - p0.screen).length();
                self.state.drag_start_dist_px = if start_dist.is_finite() && start_dist > 1e-3 {
                    start_dist
                } else {
                    self.config.radius_px.max(1.0)
                };

                Some(GizmoUpdate {
                    phase: GizmoPhase::Begin,
                    active: active_handle,
                    result: GizmoResult::Scale {
                        delta: Vec3::ONE,
                        total: Vec3::ONE,
                    },
                    updated_targets: targets.to_vec(),
                    custom_edits: Vec::new(),
                })
            }
            GizmoPhase::Update => {
                if self.state.active != Some(active_handle) {
                    return None;
                }

                let dist = (ctx.input.cursor_px - p0.screen).length();
                if !dist.is_finite() {
                    return None;
                }
                let ratio = (dist / self.state.drag_start_dist_px).max(0.01);
                let precision = Self::input_precision(ctx.input.precision);
                let factor = 1.0 + (ratio - 1.0) * precision;
                let factor = self.snap_factor(&ctx.input, factor);

                let delta_factor = factor / self.state.total_factor_applied.max(1e-6);
                self.state.total_factor_applied = factor;

                let origin = self.state.drag_origin;
                let updated_targets = self
                    .state
                    .drag_start_targets
                    .iter()
                    .map(|t| {
                        let p = t.transform.translation - origin;
                        GizmoTarget3d {
                            id: t.id,
                            transform: crate::Transform3d {
                                translation: origin + p * factor,
                                rotation: t.transform.rotation,
                                scale: (t.transform.scale * factor).max(Vec3::splat(1e-4)),
                            },
                            local_bounds: t.local_bounds,
                        }
                    })
                    .collect::<Vec<_>>();

                Some(GizmoUpdate {
                    phase: GizmoPhase::Update,
                    active: active_handle,
                    result: GizmoResult::Scale {
                        delta: Vec3::splat(delta_factor),
                        total: Vec3::splat(factor),
                    },
                    updated_targets,
                    custom_edits: Vec::new(),
                })
            }
            GizmoPhase::Commit => {
                if self.state.active != Some(active_handle) {
                    return None;
                }
                let factor = self.state.total_factor_applied;
                self.state.active = None;
                self.state.drag_start_targets.clear();
                Some(GizmoUpdate {
                    phase: GizmoPhase::Commit,
                    active: active_handle,
                    result: GizmoResult::Scale {
                        delta: Vec3::ONE,
                        total: Vec3::splat(factor),
                    },
                    updated_targets: targets.to_vec(),
                    custom_edits: Vec::new(),
                })
            }
            GizmoPhase::Cancel => {
                if self.state.active != Some(active_handle) {
                    return None;
                }
                let factor = self.state.total_factor_applied;
                self.state.active = None;
                self.state.drag_start_targets.clear();
                Some(GizmoUpdate {
                    phase: GizmoPhase::Cancel,
                    active: active_handle,
                    result: GizmoResult::Scale {
                        delta: Vec3::ONE,
                        total: Vec3::splat(factor),
                    },
                    updated_targets: targets.to_vec(),
                    custom_edits: Vec::new(),
                })
            }
        }
    }
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

    #[test]
    fn ring_scale_returns_to_one_when_cursor_returns() {
        let mut mgr = crate::GizmoPluginManager::new(crate::GizmoPluginManagerConfig {
            drag_start_threshold_px: 0.0,
        });
        mgr.register(Box::new(RingScaleGizmoPlugin::default()));

        let targets = vec![GizmoTarget3d {
            id: GizmoTargetId(1),
            transform: crate::Transform3d::default(),
            local_bounds: None,
        }];
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let view_proj = test_view_projection((800.0, 600.0));
        let depth_range = DepthRange::ZeroToOne;

        let origin = Vec3::ZERO;
        let center = project_point(view_proj, vp, origin, depth_range)
            .unwrap()
            .screen;
        let start = center + Vec2::new(160.0, 0.0);
        let moved = center + Vec2::new(200.0, 0.0);

        let u0 = mgr.update(
            view_proj,
            vp,
            depth_range,
            crate::GizmoInput {
                cursor_px: start,
                hovered: true,
                drag_started: true,
                dragging: true,
                snap: false,
                cancel: false,
                precision: 1.0,
            },
            targets[0].id,
            &targets,
            None,
        );
        assert!(u0.is_some());
        assert_eq!(u0.unwrap().phase, GizmoPhase::Begin);

        let u1 = mgr.update(
            view_proj,
            vp,
            depth_range,
            crate::GizmoInput {
                cursor_px: moved,
                hovered: true,
                drag_started: false,
                dragging: true,
                snap: false,
                cancel: false,
                precision: 1.0,
            },
            targets[0].id,
            &targets,
            None,
        );
        assert!(u1.is_some());
        let u1 = u1.unwrap();
        assert_eq!(u1.phase, GizmoPhase::Update);

        let u2 = mgr.update(
            view_proj,
            vp,
            depth_range,
            crate::GizmoInput {
                cursor_px: start,
                hovered: true,
                drag_started: false,
                dragging: true,
                snap: false,
                cancel: false,
                precision: 1.0,
            },
            targets[0].id,
            &targets,
            None,
        );
        assert!(u2.is_some());
        let u2 = u2.unwrap();
        assert_eq!(u2.phase, GizmoPhase::Update);
        let crate::GizmoResult::Scale { total, .. } = u2.result else {
            panic!("expected scale result");
        };
        let d = (total - Vec3::ONE).length();
        assert!(d < 1e-3, "expected total near 1, got {total:?}");

        let u3 = mgr.update(
            view_proj,
            vp,
            depth_range,
            crate::GizmoInput {
                cursor_px: start,
                hovered: true,
                drag_started: false,
                dragging: false,
                snap: false,
                cancel: false,
                precision: 1.0,
            },
            targets[0].id,
            &targets,
            None,
        );
        assert!(u3.is_some());
        assert_eq!(u3.unwrap().phase, GizmoPhase::Commit);
    }
}
