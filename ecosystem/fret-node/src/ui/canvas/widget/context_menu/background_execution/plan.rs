use crate::ui::canvas::widget::*;

use super::BackgroundInsertMenuPlan;

pub(super) fn plan_background_insert_menu_candidate_with_graph<M: NodeGraphCanvasMiddleware>(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    candidate: &InsertNodeCandidate,
    at: CanvasPoint,
) -> BackgroundInsertMenuPlan {
    match NodeGraphCanvasWith::<M>::plan_insert_candidate_ops_with_graph(
        presenter, graph, candidate, at,
    ) {
        Ok(ops) => BackgroundInsertMenuPlan::Apply(ops),
        Err(msg) => BackgroundInsertMenuPlan::Reject(DiagnosticSeverity::Info, msg),
    }
}

pub(super) fn plan_background_insert_menu_candidate<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    candidate: &InsertNodeCandidate,
    at: CanvasPoint,
) -> BackgroundInsertMenuPlan {
    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            plan_background_insert_menu_candidate_with_graph::<M>(presenter, graph, candidate, at)
        })
        .ok()
        .unwrap_or(BackgroundInsertMenuPlan::Ignore)
}
