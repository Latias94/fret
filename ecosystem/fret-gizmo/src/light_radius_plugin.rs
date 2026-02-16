use std::collections::HashMap;

use fret_core::Color;
use glam::{Mat4, Vec2, Vec3};

use crate::math::{DepthRange, ViewportRect, project_point, ray_from_screen, unproject_point};
use crate::picking::PickSegmentCapsule2d;
use crate::{
    DepthMode, GizmoCustomEdit, GizmoDrawList3d, GizmoPhase, GizmoPickItem, GizmoPickShape2d,
    GizmoPlugin, GizmoPluginContext, GizmoPluginId, GizmoPropertyKey, GizmoResult, GizmoTarget3d,
    GizmoTargetId, GizmoUpdate, HandleId, Line3d, Triangle3d,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightRadiusGizmoConfig {
    pub min_radius_world: f32,
    pub default_radius_world: f32,
    pub pick_radius_px: f32,
    pub segments: usize,
    pub edge_depth: DepthMode,
    pub edge_color: Color,
    pub edge_hover_color: Color,
    pub fill_depth: DepthMode,
    pub fill_color: Color,
    pub fill_hover_color: Color,
    pub snap_step_world: Option<f32>,
}

impl Default for LightRadiusGizmoConfig {
    fn default() -> Self {
        Self {
            min_radius_world: 0.01,
            default_radius_world: 2.0,
            pick_radius_px: 10.0,
            segments: 64,
            edge_depth: DepthMode::Test,
            edge_color: Color {
                r: 0.25,
                g: 0.75,
                b: 0.95,
                a: 0.85,
            },
            edge_hover_color: Color {
                r: 0.35,
                g: 0.85,
                b: 1.0,
                a: 1.0,
            },
            fill_depth: DepthMode::Ghost,
            fill_color: Color {
                r: 0.25,
                g: 0.75,
                b: 0.95,
                a: 0.12,
            },
            fill_hover_color: Color {
                r: 0.35,
                g: 0.85,
                b: 1.0,
                a: 0.18,
            },
            snap_step_world: Some(0.1),
        }
    }
}

#[derive(Debug, Default)]
pub struct LightRadiusGizmoState {
    pub active: Option<HandleId>,
    drag_origin: Vec3,
    drag_center_px: Vec2,
    drag_world_per_px: f32,
    drag_prev_dist_px: f32,
    drag_total_world_raw: f32,
    drag_start_radii: HashMap<GizmoTargetId, f32>,
    drag_total_applied: HashMap<GizmoTargetId, f32>,
}

#[derive(Debug, Default)]
pub struct LightRadiusGizmoPlugin {
    pub config: LightRadiusGizmoConfig,
    pub state: LightRadiusGizmoState,
}

impl LightRadiusGizmoPlugin {
    pub const PLUGIN_ID: GizmoPluginId = GizmoPluginId(2);
    pub const RING_HANDLE_LOCAL: u32 = 1;
    pub const PROPERTY_RADIUS: GizmoPropertyKey = GizmoPropertyKey::new(Self::PLUGIN_ID, 1);

    pub fn ring_handle() -> HandleId {
        HandleId::from_parts(Self::PLUGIN_ID, Self::RING_HANDLE_LOCAL)
    }

    fn radius_world_from_properties(
        ctx: GizmoPluginContext<'_>,
        target: GizmoTargetId,
    ) -> Option<f32> {
        ctx.properties
            .and_then(|p| p.read_scalar(target, Self::PROPERTY_RADIUS))
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

    fn input_precision(precision: f32) -> f32 {
        if precision.is_finite() {
            precision.clamp(0.01, 100.0)
        } else {
            1.0
        }
    }

    fn snap_total_world(&self, input: &crate::GizmoInput, total_world: f32) -> f32 {
        let mut total_world = if total_world.is_finite() {
            total_world
        } else {
            0.0
        };
        if input.snap
            && let Some(step) = self
                .config
                .snap_step_world
                .filter(|s| s.is_finite() && *s > 0.0)
        {
            total_world = (total_world / step).round() * step;
        }
        total_world
    }

    fn radius_for_target(&self, ctx: GizmoPluginContext<'_>, target: GizmoTargetId) -> f32 {
        Self::radius_world_from_properties(ctx, target)
            .unwrap_or(self.config.default_radius_world)
            .max(self.config.min_radius_world.max(0.0))
    }

    fn begin_drag(
        &mut self,
        ctx: GizmoPluginContext<'_>,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> Option<GizmoUpdate> {
        let origin = Self::active_origin(active_target, targets);
        let p0 = project_point(ctx.view_projection, ctx.viewport, origin, ctx.depth_range)?;
        let world_per_px =
            Self::world_per_px(ctx.view_projection, ctx.viewport, ctx.depth_range, origin)?;

        let start_dist = (ctx.drag_start_cursor_px - p0.screen).length();
        let start_dist = if start_dist.is_finite() && start_dist > 1e-3 {
            start_dist
        } else {
            1.0
        };

        self.state.active = Some(Self::ring_handle());
        self.state.drag_origin = origin;
        self.state.drag_center_px = p0.screen;
        self.state.drag_world_per_px = world_per_px;
        self.state.drag_prev_dist_px = start_dist;
        self.state.drag_total_world_raw = 0.0;
        self.state.drag_start_radii.clear();
        self.state.drag_total_applied.clear();

        for t in targets {
            let r0 = self.radius_for_target(ctx, t.id);
            self.state.drag_start_radii.insert(t.id, r0);
            self.state.drag_total_applied.insert(t.id, 0.0);
        }

        let active_value = self.state.drag_start_radii.get(&active_target).copied();
        Some(GizmoUpdate {
            phase: GizmoPhase::Begin,
            active: Self::ring_handle(),
            result: GizmoResult::CustomScalar {
                key: Self::PROPERTY_RADIUS,
                delta: 0.0,
                total: 0.0,
                value: active_value,
            },
            updated_targets: targets.to_vec(),
            custom_edits: targets
                .iter()
                .map(|t| GizmoCustomEdit::Scalar {
                    target: t.id,
                    key: Self::PROPERTY_RADIUS,
                    delta: 0.0,
                    total: 0.0,
                })
                .collect(),
        })
    }

    fn finish_drag(
        &mut self,
        phase: GizmoPhase,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> Option<GizmoUpdate> {
        let _ = self.state.active.take()?;

        let mut custom_edits = Vec::with_capacity(targets.len());
        let mut active_total = 0.0;
        let mut active_value = None;

        for t in targets {
            let start = self.state.drag_start_radii.get(&t.id).copied().unwrap_or(
                self.config
                    .default_radius_world
                    .max(self.config.min_radius_world.max(0.0)),
            );
            let prev_total = self
                .state
                .drag_total_applied
                .get(&t.id)
                .copied()
                .unwrap_or(0.0);

            let (delta, total, value) = match phase {
                GizmoPhase::Cancel => {
                    let delta = -prev_total;
                    let total = 0.0;
                    let value = start;
                    (delta, total, value)
                }
                GizmoPhase::Commit => {
                    let delta = 0.0;
                    let total = prev_total;
                    let value = (start + total).max(self.config.min_radius_world.max(0.0));
                    (delta, total, value)
                }
                _ => return None,
            };

            self.state.drag_total_applied.insert(t.id, total);

            if t.id == active_target {
                active_total = total;
                active_value = Some(value);
            }

            custom_edits.push(GizmoCustomEdit::Scalar {
                target: t.id,
                key: Self::PROPERTY_RADIUS,
                delta,
                total,
            });
        }

        Some(GizmoUpdate {
            phase,
            active: Self::ring_handle(),
            result: GizmoResult::CustomScalar {
                key: Self::PROPERTY_RADIUS,
                delta: 0.0,
                total: active_total,
                value: active_value,
            },
            updated_targets: targets.to_vec(),
            custom_edits,
        })
    }
}

impl GizmoPlugin for LightRadiusGizmoPlugin {
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

        let handle = Self::ring_handle();
        let highlighted = ctx.active == Some(handle) || ctx.hovered == Some(handle);
        let edge = if highlighted {
            self.config.edge_hover_color
        } else {
            self.config.edge_color
        };
        let fill = if highlighted {
            self.config.fill_hover_color
        } else {
            self.config.fill_color
        };

        let radius_world = self.radius_for_target(ctx, active_target);
        if !radius_world.is_finite() || radius_world <= 1e-6 {
            return GizmoDrawList3d::default();
        }

        let segs = self.config.segments.clamp(3, 256);
        let mut out = GizmoDrawList3d::default();

        // Fill (triangle fan).
        let mut prev = origin + u * radius_world;
        for i in 1..=segs {
            let t = (i as f32) / (segs as f32) * std::f32::consts::TAU;
            let next = origin + (u * t.cos() + v * t.sin()) * radius_world;
            out.triangles.push(Triangle3d {
                a: origin,
                b: prev,
                c: next,
                color: fill,
                depth: self.config.fill_depth,
            });
            prev = next;
        }

        // Edge.
        let mut prev = origin + u * radius_world;
        for i in 1..=segs {
            let t = (i as f32) / (segs as f32) * std::f32::consts::TAU;
            let next = origin + (u * t.cos() + v * t.sin()) * radius_world;
            out.lines.push(Line3d {
                a: prev,
                b: next,
                color: edge,
                depth: self.config.edge_depth,
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
        if targets.is_empty() {
            return;
        }

        let origin = Self::active_origin(active_target, targets);
        let Some(p0) = project_point(ctx.view_projection, ctx.viewport, origin, ctx.depth_range)
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

        let radius_world = self.radius_for_target(ctx, active_target);
        if !radius_world.is_finite() || radius_world <= 1e-6 {
            return;
        }

        let segs = self.config.segments.clamp(3, 256);
        let handle = Self::ring_handle();
        let bias = 0.0;
        let radius_px = self.config.pick_radius_px.max(1.0);

        let first_world = origin + u * radius_world;
        let mut prev_px = project_point(
            ctx.view_projection,
            ctx.viewport,
            first_world,
            ctx.depth_range,
        )
        .map(|p| p.screen);

        for i in 1..=segs {
            let t = (i as f32) / (segs as f32) * std::f32::consts::TAU;
            let next_world = origin + (u * t.cos() + v * t.sin()) * radius_world;
            let next_px = project_point(
                ctx.view_projection,
                ctx.viewport,
                next_world,
                ctx.depth_range,
            )
            .map(|p| p.screen);

            if let (Some(a), Some(b)) = (prev_px, next_px) {
                out.push(GizmoPickItem {
                    handle,
                    shape: GizmoPickShape2d::SegmentCapsule(PickSegmentCapsule2d {
                        a,
                        b,
                        radius: radius_px,
                    }),
                    bias_px: bias,
                });
            }

            prev_px = next_px;
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

        match phase {
            GizmoPhase::Begin => self.begin_drag(ctx, active_target, targets),
            GizmoPhase::Update => {
                if self.state.active != Some(active_handle) {
                    return None;
                }

                let dist = (ctx.input.cursor_px - self.state.drag_center_px).length();
                if !dist.is_finite() {
                    return None;
                }

                let delta_px = dist - self.state.drag_prev_dist_px;
                self.state.drag_prev_dist_px = dist;

                let precision = Self::input_precision(ctx.input.precision);
                let delta_world = delta_px * self.state.drag_world_per_px * precision;
                if !delta_world.is_finite() {
                    return None;
                }
                self.state.drag_total_world_raw += delta_world;

                let desired_total_world =
                    self.snap_total_world(&ctx.input, self.state.drag_total_world_raw);

                let mut custom_edits = Vec::with_capacity(targets.len());
                let mut active_delta = 0.0;
                let mut active_total = 0.0;
                let mut active_value = None;

                for t in targets {
                    let start = self.state.drag_start_radii.get(&t.id).copied().unwrap_or(
                        self.config
                            .default_radius_world
                            .max(self.config.min_radius_world.max(0.0)),
                    );
                    let desired_value =
                        (start + desired_total_world).max(self.config.min_radius_world.max(0.0));
                    let total = desired_value - start;
                    let prev_total = self
                        .state
                        .drag_total_applied
                        .get(&t.id)
                        .copied()
                        .unwrap_or(0.0);
                    let delta = total - prev_total;
                    self.state.drag_total_applied.insert(t.id, total);

                    if t.id == active_target {
                        active_delta = delta;
                        active_total = total;
                        active_value = Some(desired_value);
                    }

                    custom_edits.push(GizmoCustomEdit::Scalar {
                        target: t.id,
                        key: Self::PROPERTY_RADIUS,
                        delta,
                        total,
                    });
                }

                Some(GizmoUpdate {
                    phase: GizmoPhase::Update,
                    active: active_handle,
                    result: GizmoResult::CustomScalar {
                        key: Self::PROPERTY_RADIUS,
                        delta: active_delta,
                        total: active_total,
                        value: active_value,
                    },
                    updated_targets: targets.to_vec(),
                    custom_edits,
                })
            }
            GizmoPhase::Commit => self.finish_drag(GizmoPhase::Commit, active_target, targets),
            GizmoPhase::Cancel => self.finish_drag(GizmoPhase::Cancel, active_target, targets),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestProperties {
        scalars: HashMap<(GizmoPropertyKey, GizmoTargetId), f32>,
    }

    impl crate::GizmoPropertySource for TestProperties {
        fn read_scalar(&self, target: GizmoTargetId, key: GizmoPropertyKey) -> Option<f32> {
            self.scalars.get(&(key, target)).copied()
        }
    }

    fn test_view_projection(viewport_px: (f32, f32)) -> Mat4 {
        let aspect = viewport_px.0.max(1.0) / viewport_px.1.max(1.0);
        let eye = Vec3::new(3.0, 2.0, 4.0);
        let target = Vec3::ZERO;
        let view = Mat4::look_at_rh(eye, target, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), aspect, 0.05, 100.0);
        proj * view
    }

    #[test]
    fn light_radius_emits_custom_scalar_edits() {
        let mut mgr = crate::GizmoPluginManager::new(crate::GizmoPluginManagerConfig {
            drag_start_threshold_px: 0.0,
        });
        mgr.register(Box::new(LightRadiusGizmoPlugin::default()));

        let mut properties = TestProperties::default();
        properties.scalars.insert(
            (LightRadiusGizmoPlugin::PROPERTY_RADIUS, GizmoTargetId(1)),
            2.0,
        );

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

        // Click near the current radius ring (roughly to the right).
        let world_per_px =
            LightRadiusGizmoPlugin::world_per_px(view_proj, vp, depth_range, origin).unwrap();
        let radius_px = 2.0 / world_per_px;
        let start = center + Vec2::new(radius_px, 0.0);
        let moved = start + Vec2::new(40.0, 0.0);

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
            Some(&properties),
        );
        let u0 = u0.expect("begin update");
        assert_eq!(u0.phase, GizmoPhase::Begin);
        assert!(!u0.custom_edits.is_empty());

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
            Some(&properties),
        );
        let u1 = u1.expect("update");
        assert_eq!(u1.phase, GizmoPhase::Update);

        match u1.result {
            GizmoResult::CustomScalar { total, .. } => {
                assert!(total.is_finite());
                assert!(total > 0.0);
            }
            _ => panic!("expected custom scalar result"),
        }

        assert_eq!(u1.custom_edits.len(), 1);
        match u1.custom_edits[0] {
            GizmoCustomEdit::Scalar { total, .. } => {
                assert!(total.is_finite());
                assert!(total > 0.0);
            }
        }
    }
}
