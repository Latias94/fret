mod activate;
mod apply;
mod plan;

use super::connection_execution::ConnectionConversionMenuPlan;
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[cfg(test)]
    pub(super) fn plan_connection_conversion_menu_candidate_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        style: &NodeGraphStyle,
        zoom: f32,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionConversionMenuPlan {
        plan::plan_connection_conversion_menu_candidate_with_graph::<M>(
            presenter, graph, style, zoom, from, to, at, candidate,
        )
    }

    pub(super) fn plan_connection_conversion_menu_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionConversionMenuPlan {
        plan::plan_connection_conversion_menu_candidate(self, host, from, to, at, candidate)
    }

    pub(super) fn activate_connection_conversion_picker_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        activate::activate_connection_conversion_picker_action(
            self,
            cx,
            from,
            to,
            at,
            invoked_at,
            action,
            menu_candidates,
        )
    }

    pub(super) fn apply_connection_conversion_menu_plan<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: PortId,
        invoked_at: Point,
        plan: ConnectionConversionMenuPlan,
    ) {
        apply::apply_connection_conversion_menu_plan(self, cx, fallback_from, invoked_at, plan)
    }
}
