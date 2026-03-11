mod activate;
mod apply;
mod plan;
mod recovery;

use super::connection_execution::ConnectionInsertMenuPlan;
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn activate_connection_insert_picker_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        from: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        activate::activate_connection_insert_picker_action(
            self,
            cx,
            from,
            at,
            invoked_at,
            action,
            menu_candidates,
        )
    }

    #[cfg(test)]
    pub(super) fn plan_connection_insert_menu_candidate_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        from: PortId,
        at: CanvasPoint,
        mode: NodeGraphConnectionMode,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionInsertMenuPlan {
        plan::plan_connection_insert_menu_candidate_with_graph::<M>(
            presenter, graph, from, at, mode, candidate,
        )
    }

    pub(super) fn plan_connection_insert_menu_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        at: CanvasPoint,
        mode: NodeGraphConnectionMode,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionInsertMenuPlan {
        plan::plan_connection_insert_menu_candidate(self, host, from, at, mode, candidate)
    }

    pub(super) fn apply_connection_insert_menu_plan<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: PortId,
        invoked_at: Point,
        plan: ConnectionInsertMenuPlan,
    ) {
        apply::apply_connection_insert_menu_plan(self, cx, fallback_from, invoked_at, plan)
    }

    fn resume_connection_insert_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: PortId,
        invoked_at: Point,
        continue_from: Option<PortId>,
    ) {
        recovery::resume_connection_insert_wire_drag(
            self,
            cx,
            fallback_from,
            invoked_at,
            continue_from,
        )
    }

    pub(super) fn restore_connection_menu_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: PortId,
        invoked_at: Point,
    ) {
        recovery::restore_connection_menu_wire_drag(self, cx, fallback_from, invoked_at)
    }
}
