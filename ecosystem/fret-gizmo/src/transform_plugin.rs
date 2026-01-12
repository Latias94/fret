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
        let group = (handle.local() >> 16) as u32;
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
        ctx: GizmoPluginContext,
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
        ctx: GizmoPluginContext,
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
        ctx: GizmoPluginContext,
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
                let mut begin_input = input;
                begin_input.cursor_px = ctx.drag_start_cursor_px;
                begin_input.drag_started = true;
                begin_input.dragging = true;
                self.gizmo.begin_drag_with_handle(
                    ctx.view_projection,
                    ctx.viewport,
                    begin_input,
                    active_target,
                    targets,
                    active_handle,
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
