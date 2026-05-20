use fret_ui::{UiHost, retained_bridge::EventCx};

use crate::core::GroupId;
use crate::ui::canvas::widget::*;

use super::command::CommandContextActionCx;
use super::target::TargetContextActionCx;

impl<H: UiHost, M: NodeGraphCanvasMiddleware> CommandContextActionCx<M> for EventCx<'_, H> {
    fn select_group_context_target(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        group_id: GroupId,
    ) {
        canvas.select_group_context_target(self.app, group_id);
    }

    fn dispatch_context_command(&mut self, command: fret_runtime::CommandId) {
        self.dispatch_command(command);
    }
}

impl<H: UiHost, M: NodeGraphCanvasMiddleware> TargetContextActionCx<M> for EventCx<'_, H> {
    fn activate_background_context_action(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        at: CanvasPoint,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        canvas.activate_background_context_action(self, at, action, menu_candidates)
    }

    fn activate_connection_insert_picker_action(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        from: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        canvas.activate_connection_insert_picker_action(
            self,
            from,
            at,
            invoked_at,
            action,
            menu_candidates,
        )
    }

    fn activate_edge_context_action(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        edge_id: EdgeId,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
    ) -> bool {
        canvas.activate_edge_context_action(self, edge_id, invoked_at, action)
    }

    fn activate_edge_insert_picker_action(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        edge_id: EdgeId,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        edge_insert::activate_edge_insert_picker_action(
            canvas,
            self,
            edge_id,
            invoked_at,
            action,
            menu_candidates,
        )
    }

    fn activate_connection_conversion_picker_action(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        canvas.activate_connection_conversion_picker_action(
            self,
            from,
            to,
            at,
            invoked_at,
            action,
            menu_candidates,
        )
    }
}
