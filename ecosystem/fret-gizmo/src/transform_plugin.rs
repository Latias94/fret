use crate::picking::PickCircle2d;
use crate::{
    Gizmo, GizmoDrawList3d, GizmoMode, GizmoPhase, GizmoPickItem, GizmoPickShape2d, GizmoPlugin,
    GizmoPluginContext, GizmoPluginId, GizmoTarget3d, GizmoTargetId, GizmoUpdate, HandleId,
};

#[derive(Debug)]
pub struct TransformGizmoPlugin {
    pub gizmo: Gizmo,
}

impl TransformGizmoPlugin {
    pub const PLUGIN_ID: GizmoPluginId = GizmoPluginId(0);

    pub fn new(gizmo: Gizmo) -> Self {
        Self { gizmo }
    }

    fn kind_for_handle(handle: HandleId) -> Option<GizmoMode> {
        let group = handle.local() >> 16;
        match group {
            1 => Some(GizmoMode::Translate),
            2 => Some(GizmoMode::Rotate),
            3 => Some(GizmoMode::Scale),
            _ => None,
        }
    }
}

impl GizmoPlugin for TransformGizmoPlugin {
    fn plugin_id(&self) -> GizmoPluginId {
        Self::PLUGIN_ID
    }

    fn draw(
        &mut self,
        ctx: GizmoPluginContext<'_>,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
    ) -> GizmoDrawList3d {
        self.gizmo.config.depth_range = ctx.depth_range;
        let hovered = ctx.hovered.filter(|h| h.plugin() == self.plugin_id());
        let active = ctx.active.filter(|h| h.plugin() == self.plugin_id());

        self.gizmo.state.hovered = hovered;
        self.gizmo.state.active = active;
        self.gizmo.state.hovered_kind = hovered.and_then(Self::kind_for_handle);

        self.gizmo
            .draw(ctx.view_projection, ctx.viewport, active_target, targets)
    }

    fn pick_items(
        &mut self,
        ctx: GizmoPluginContext<'_>,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
        out: &mut Vec<GizmoPickItem>,
    ) {
        self.gizmo.config.depth_range = ctx.depth_range;
        if !ctx.input.hovered || targets.is_empty() {
            return;
        }

        let Some((handle, score_px)) = self.gizmo.pick_hit(
            ctx.view_projection,
            ctx.viewport,
            ctx.input.cursor_px,
            active_target,
            targets,
        ) else {
            return;
        };

        if handle.plugin() != self.plugin_id() || !score_px.is_finite() {
            return;
        }

        out.push(GizmoPickItem {
            handle,
            shape: GizmoPickShape2d::Circle(PickCircle2d {
                center: ctx.input.cursor_px,
                radius: 0.0,
            }),
            bias_px: score_px,
        });
    }

    fn update(
        &mut self,
        ctx: GizmoPluginContext<'_>,
        phase: GizmoPhase,
        active_target: GizmoTargetId,
        targets: &[GizmoTarget3d],
        active_handle: HandleId,
    ) -> Option<GizmoUpdate> {
        self.gizmo.config.depth_range = ctx.depth_range;
        if active_handle.plugin() != self.plugin_id() {
            return None;
        }

        let input = ctx.input;
        match phase {
            GizmoPhase::Begin => {
                // `GizmoPluginManager` may call `Begin` repeatedly until the plugin returns
                // `Some(...)`. That interacts poorly with the built-in transform gizmo's optional
                // `drag_start_threshold_px`, which delays emitting `Begin` until the cursor moves.
                //
                // To support both:
                // - Arm the drag once at the pointer-down cursor (so the gizmo snapshots targets).
                // - Then feed subsequent cursor motion into `Gizmo::update` so the threshold can
                //   actually trip and emit the `Begin` phase.
                if self.gizmo.state.active != Some(active_handle) {
                    let mut begin_input = input;
                    begin_input.cursor_px = ctx.drag_start_cursor_px;
                    begin_input.drag_started = true;
                    begin_input.dragging = true;
                    if let Some(u) = self.gizmo.begin_drag_with_handle(
                        ctx.view_projection,
                        ctx.viewport,
                        begin_input,
                        active_target,
                        targets,
                        active_handle,
                    ) {
                        return Some(u);
                    }
                }

                let mut update_input = input;
                update_input.drag_started = false;
                update_input.dragging = true;
                self.gizmo.update(
                    ctx.view_projection,
                    ctx.viewport,
                    update_input,
                    active_target,
                    targets,
                )
            }
            GizmoPhase::Update => {
                let mut update_input = input;
                update_input.drag_started = false;
                update_input.dragging = true;
                self.gizmo.update(
                    ctx.view_projection,
                    ctx.viewport,
                    update_input,
                    active_target,
                    targets,
                )
            }
            GizmoPhase::Commit => {
                let mut commit_input = input;
                commit_input.drag_started = false;
                commit_input.dragging = false;
                self.gizmo.update(
                    ctx.view_projection,
                    ctx.viewport,
                    commit_input,
                    active_target,
                    targets,
                )
            }
            GizmoPhase::Cancel => {
                let mut cancel_input = input;
                cancel_input.cancel = true;
                self.gizmo.update(
                    ctx.view_projection,
                    ctx.viewport,
                    cancel_input,
                    active_target,
                    targets,
                )
            }
        }
    }
}

impl Default for TransformGizmoPlugin {
    fn default() -> Self {
        Self::new(Gizmo::new(crate::GizmoConfig::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BUILTIN_HANDLE_GROUP_TRANSLATE, DepthRange, GizmoConfig, GizmoInput, GizmoMode,
        HANDLE_LOCAL_GROUP_SHIFT, Transform3d, ViewportRect,
    };
    use glam::{Mat4, Vec2, Vec3};

    fn default_view_projection(aspect: f32) -> Mat4 {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        proj * view
    }

    fn dummy_target(id: u64) -> GizmoTarget3d {
        GizmoTarget3d {
            id: GizmoTargetId(id),
            transform: Transform3d::default(),
            local_bounds: None,
        }
    }

    #[test]
    fn begin_can_progress_with_internal_drag_threshold_when_begin_is_repeated() {
        let cfg = GizmoConfig {
            mode: GizmoMode::Translate,
            drag_start_threshold_px: 3.0,
            ..Default::default()
        };

        let mut plugin = TransformGizmoPlugin::new(Gizmo::new(cfg));

        let view_projection = default_view_projection(800.0 / 600.0);
        let viewport = ViewportRect::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let depth_range = DepthRange::ZeroToOne;

        let targets = vec![dummy_target(1)];
        let active_target = targets[0].id;

        let active_handle = HandleId::from_parts(
            TransformGizmoPlugin::PLUGIN_ID,
            (BUILTIN_HANDLE_GROUP_TRANSLATE << HANDLE_LOCAL_GROUP_SHIFT) | 1,
        );

        // Pointer-down: begin is armed but not emitted due to the threshold.
        let start = Vec2::new(400.0, 300.0);
        let ctx0 = GizmoPluginContext {
            view_projection,
            viewport,
            depth_range,
            input: GizmoInput {
                cursor_px: start,
                hovered: true,
                drag_started: true,
                dragging: true,
                snap: false,
                cancel: false,
                precision: 1.0,
            },
            properties: None,
            drag_start_cursor_px: start,
            hovered: Some(active_handle),
            active: Some(active_handle),
        };
        assert!(
            plugin
                .update(
                    ctx0,
                    GizmoPhase::Begin,
                    active_target,
                    &targets,
                    active_handle
                )
                .is_none()
        );

        // Cursor moves past the threshold; repeated `Begin` should now produce a `Begin` update.
        let moved = start + Vec2::new(10.0, 0.0);
        let ctx1 = GizmoPluginContext {
            input: GizmoInput {
                cursor_px: moved,
                hovered: true,
                drag_started: false,
                dragging: true,
                snap: false,
                cancel: false,
                precision: 1.0,
            },
            drag_start_cursor_px: start,
            hovered: Some(active_handle),
            active: Some(active_handle),
            ..ctx0
        };
        let u = plugin
            .update(
                ctx1,
                GizmoPhase::Begin,
                active_target,
                &targets,
                active_handle,
            )
            .expect("expected begin to be emitted once threshold is exceeded");
        assert_eq!(u.phase, GizmoPhase::Begin);
        assert_eq!(u.active, active_handle);
    }
}
