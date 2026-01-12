use std::any::Any;

use glam::{Mat4, Vec2};

use crate::math::ViewportRect;
use crate::picking::{PickCircle2d, PickConvexQuad2d, PickSegmentCapsule2d};
use crate::{
    GizmoDrawList3d, GizmoInput, GizmoPhase, GizmoPluginId, GizmoTarget3d, GizmoTargetId,
    GizmoUpdate, HandleId,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoPickShape2d {
    Circle(PickCircle2d),
    SegmentCapsule(PickSegmentCapsule2d),
    ConvexQuadInside(PickConvexQuad2d),
    ConvexQuadEdge { quad: PickConvexQuad2d, radius: f32 },
}

impl GizmoPickShape2d {
    pub fn hit_score_px(self, cursor_px: Vec2) -> Option<f32> {
        match self {
            GizmoPickShape2d::Circle(c) => c.hit_distance(cursor_px),
            GizmoPickShape2d::SegmentCapsule(c) => c.hit_distance(cursor_px),
            GizmoPickShape2d::ConvexQuadInside(q) => q.contains(cursor_px).then_some(0.0),
            GizmoPickShape2d::ConvexQuadEdge { quad, radius } => {
                if !radius.is_finite() || radius <= 0.0 {
                    return None;
                }
                let d = quad.edge_distance(cursor_px);
                (d.is_finite() && d <= radius).then_some(d)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoPickItem {
    pub handle: HandleId,
    pub shape: GizmoPickShape2d,
    /// Lower wins. Bias is expressed in pixels (same domain as hit scores).
    pub bias_px: f32,
}

impl GizmoPickItem {
    pub fn hit_score_px(self, cursor_px: Vec2) -> Option<f32> {
        let s = self.shape.hit_score_px(cursor_px)?;
        let bias = if self.bias_px.is_finite() {
            self.bias_px
        } else {
            0.0
        };
        Some(s + bias)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoPickHit {
    pub handle: HandleId,
    pub score_px: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoPluginContext {
    pub view_projection: Mat4,
    pub viewport: ViewportRect,
    pub input: GizmoInput,
    /// Cursor position at pointer-down for the current active drag.
    ///
    /// When there is no active drag, this equals `input.cursor_px`.
    pub drag_start_cursor_px: Vec2,
    pub hovered: Option<HandleId>,
    pub active: Option<HandleId>,
}

pub trait GizmoPlugin: Any {
    fn plugin_id(&self) -> GizmoPluginId;

    fn draw(
        &mut self,
        ctx: GizmoPluginContext,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> GizmoDrawList3d;

    fn pick_items(
        &mut self,
        ctx: GizmoPluginContext,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
        out: &mut Vec<GizmoPickItem>,
    );

    fn update(
        &mut self,
        ctx: GizmoPluginContext,
        phase: GizmoPhase,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
        active_handle: HandleId,
    ) -> Option<GizmoUpdate>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoPluginManagerConfig {
    pub drag_start_threshold_px: f32,
}

impl Default for GizmoPluginManagerConfig {
    fn default() -> Self {
        Self {
            drag_start_threshold_px: 0.0,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct GizmoPluginManagerState {
    pub hovered: Option<HandleId>,
    pub active: Option<HandleId>,
    drag_start_cursor_px: Vec2,
    drag_has_started: bool,
}

#[derive(Default)]
pub struct GizmoPluginManager {
    pub config: GizmoPluginManagerConfig,
    pub state: GizmoPluginManagerState,
    plugins: Vec<Box<dyn GizmoPlugin>>,
    scratch_pick_items: Vec<GizmoPickItem>,
}

impl GizmoPluginManager {
    pub fn new(config: GizmoPluginManagerConfig) -> Self {
        Self {
            config,
            state: GizmoPluginManagerState::default(),
            plugins: Vec::new(),
            scratch_pick_items: Vec::new(),
        }
    }

    pub fn plugins_mut(&mut self) -> &mut [Box<dyn GizmoPlugin>] {
        &mut self.plugins
    }

    pub fn plugin<T: Any>(&self) -> Option<&T> {
        self.plugins.iter().find_map(|p| {
            let any = p.as_ref() as &dyn Any;
            any.downcast_ref::<T>()
        })
    }

    pub fn plugin_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.plugins.iter_mut().find_map(|p| {
            let any = p.as_mut() as &mut dyn Any;
            any.downcast_mut::<T>()
        })
    }

    pub fn register(&mut self, plugin: Box<dyn GizmoPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn draw(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
        input: GizmoInput,
    ) -> GizmoDrawList3d {
        let mut out = GizmoDrawList3d::default();

        let ctx = GizmoPluginContext {
            view_projection,
            viewport,
            input,
            drag_start_cursor_px: if self.state.active.is_some() {
                self.state.drag_start_cursor_px
            } else {
                input.cursor_px
            },
            hovered: self.state.hovered,
            active: self.state.active,
        };

        for p in &mut self.plugins {
            let mut d = p.draw(ctx, active_target, targets);
            out.lines.append(&mut d.lines);
            out.triangles.append(&mut d.triangles);
        }

        out
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
            self.state.drag_has_started = false;
            return None;
        }

        if self.state.active.is_none() {
            self.state.hovered = None;
            if input.hovered {
                self.state.hovered =
                    self.pick_best_handle(view_projection, viewport, input, active_target, targets);
            }

            if input.drag_started {
                if let Some(h) = self.state.hovered {
                    self.state.active = Some(h);
                    self.state.drag_start_cursor_px = input.cursor_px;
                    self.state.drag_has_started = false;

                    if self.config.drag_start_threshold_px <= 0.0 {
                        let out = self.route_update(
                            view_projection,
                            viewport,
                            input,
                            GizmoPhase::Begin,
                            active_target,
                            targets,
                            h,
                        );
                        if out.is_some() {
                            self.state.drag_has_started = true;
                        }
                        return out;
                    }
                }
            }

            return None;
        }

        let active = self.state.active.unwrap();
        self.state.hovered = None;

        if input.cancel {
            let out = if self.state.drag_has_started {
                self.route_update(
                    view_projection,
                    viewport,
                    input,
                    GizmoPhase::Cancel,
                    active_target,
                    targets,
                    active,
                )
            } else {
                None
            };
            self.state.active = None;
            self.state.drag_has_started = false;
            return out;
        }

        if input.dragging {
            if !self.state.drag_has_started {
                let threshold = self.config.drag_start_threshold_px.max(0.0);
                if threshold > 0.0
                    && (input.cursor_px - self.state.drag_start_cursor_px).length() < threshold
                {
                    return None;
                }
                let out = self.route_update(
                    view_projection,
                    viewport,
                    input,
                    GizmoPhase::Begin,
                    active_target,
                    targets,
                    active,
                );
                if out.is_some() {
                    self.state.drag_has_started = true;
                }
                return out;
            }

            return self.route_update(
                view_projection,
                viewport,
                input,
                GizmoPhase::Update,
                active_target,
                targets,
                active,
            );
        }

        let out = if self.state.drag_has_started {
            self.route_update(
                view_projection,
                viewport,
                input,
                GizmoPhase::Commit,
                active_target,
                targets,
                active,
            )
        } else {
            None
        };

        self.state.active = None;
        self.state.drag_has_started = false;
        out
    }

    fn pick_best_handle(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> Option<HandleId> {
        self.scratch_pick_items.clear();

        let ctx = GizmoPluginContext {
            view_projection,
            viewport,
            input,
            drag_start_cursor_px: input.cursor_px,
            hovered: None,
            active: None,
        };

        for p in &mut self.plugins {
            p.pick_items(ctx, active_target, targets, &mut self.scratch_pick_items);
        }

        let cursor = input.cursor_px;
        let mut best: Option<GizmoPickHit> = None;
        for item in self.scratch_pick_items.iter().copied() {
            let Some(score_px) = item.hit_score_px(cursor) else {
                continue;
            };
            if !score_px.is_finite() {
                continue;
            }
            match best {
                Some(b) if score_px >= b.score_px => {}
                _ => {
                    best = Some(GizmoPickHit {
                        handle: item.handle,
                        score_px,
                    })
                }
            }
        }

        best.map(|h| h.handle)
    }

    fn route_update(
        &mut self,
        view_projection: Mat4,
        viewport: ViewportRect,
        input: GizmoInput,
        phase: GizmoPhase,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
        active_handle: HandleId,
    ) -> Option<GizmoUpdate> {
        let plugin_id = active_handle.plugin();
        let ctx = GizmoPluginContext {
            view_projection,
            viewport,
            input,
            drag_start_cursor_px: self.state.drag_start_cursor_px,
            hovered: self.state.hovered,
            active: self.state.active,
        };

        self.plugins
            .iter_mut()
            .find(|p| p.plugin_id() == plugin_id)
            .and_then(|p| p.update(ctx, phase, active_target, targets, active_handle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyPlugin {
        id: GizmoPluginId,
        handle: HandleId,
        last_phase: Option<GizmoPhase>,
    }

    impl DummyPlugin {
        fn new(id: u32) -> Self {
            let pid = GizmoPluginId(id);
            Self {
                id: pid,
                handle: HandleId::from_parts(pid, 1),
                last_phase: None,
            }
        }
    }

    impl GizmoPlugin for DummyPlugin {
        fn plugin_id(&self) -> GizmoPluginId {
            self.id
        }

        fn draw(
            &mut self,
            _ctx: GizmoPluginContext,
            _active_target: GizmoTargetId,
            _targets: &[GizmoTarget3d],
        ) -> GizmoDrawList3d {
            GizmoDrawList3d::default()
        }

        fn pick_items(
            &mut self,
            _ctx: GizmoPluginContext,
            _active_target: GizmoTargetId,
            _targets: &[GizmoTarget3d],
            out: &mut Vec<GizmoPickItem>,
        ) {
            out.push(GizmoPickItem {
                handle: self.handle,
                shape: GizmoPickShape2d::Circle(PickCircle2d {
                    center: Vec2::ZERO,
                    radius: 100.0,
                }),
                bias_px: 0.0,
            });
        }

        fn update(
            &mut self,
            _ctx: GizmoPluginContext,
            phase: GizmoPhase,
            _active_target: GizmoTargetId,
            targets: &[GizmoTarget3d],
            active_handle: HandleId,
        ) -> Option<GizmoUpdate> {
            self.last_phase = Some(phase);
            Some(GizmoUpdate {
                phase,
                active: active_handle,
                result: crate::GizmoResult::Translation {
                    delta: glam::Vec3::ZERO,
                    total: glam::Vec3::ZERO,
                },
                updated_targets: targets.to_vec(),
            })
        }
    }

    fn dummy_target(id: u64) -> GizmoTarget3d {
        GizmoTarget3d {
            id: GizmoTargetId(id),
            transform: crate::Transform3d::default(),
            local_bounds: None,
        }
    }

    #[test]
    fn manager_routes_begin_update_commit_to_active_plugin() {
        let mut mgr = GizmoPluginManager::new(GizmoPluginManagerConfig {
            drag_start_threshold_px: 0.0,
        });
        mgr.register(Box::new(DummyPlugin::new(7)));

        let targets = vec![dummy_target(1)];
        let vp = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let view_proj = Mat4::IDENTITY;

        let input_begin = GizmoInput {
            cursor_px: Vec2::ZERO,
            hovered: true,
            drag_started: true,
            dragging: true,
            snap: false,
            cancel: false,
            precision: 1.0,
        };
        let u0 = mgr.update(view_proj, vp, input_begin, targets[0].id, &targets);
        assert!(u0.is_some());
        assert_eq!(u0.unwrap().phase, GizmoPhase::Begin);

        let input_update = GizmoInput {
            drag_started: false,
            ..input_begin
        };
        let u1 = mgr.update(view_proj, vp, input_update, targets[0].id, &targets);
        assert!(u1.is_some());
        assert_eq!(u1.unwrap().phase, GizmoPhase::Update);

        let input_commit = GizmoInput {
            dragging: false,
            ..input_update
        };
        let u2 = mgr.update(view_proj, vp, input_commit, targets[0].id, &targets);
        assert!(u2.is_some());
        assert_eq!(u2.unwrap().phase, GizmoPhase::Commit);
    }

    #[test]
    fn handle_namespace_round_trips() {
        let pid = GizmoPluginId(42);
        let h = HandleId::from_parts(pid, 7);
        assert_eq!(h.plugin(), pid);
        assert_eq!(h.local(), 7);
    }
}
