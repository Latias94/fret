use crate::ui::canvas::widget::*;

use super::ConnectionInsertMenuPlan;

pub(super) fn plan_connection_insert_menu_candidate_with_graph<M: NodeGraphCanvasMiddleware>(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    from: PortId,
    at: CanvasPoint,
    mode: NodeGraphConnectionMode,
    candidate: &InsertNodeCandidate,
) -> ConnectionInsertMenuPlan {
    let insert_ops = NodeGraphCanvasWith::<M>::plan_insert_candidate_ops_with_graph(
        presenter, graph, candidate, at,
    );
    let insert_ops = match insert_ops {
        Ok(ops) => ops,
        Err(msg) => {
            return ConnectionInsertMenuPlan::Reject(DiagnosticSeverity::Info, msg);
        }
    };
    ConnectionInsertMenuPlan::Apply(workflow::plan_wire_drop_insert(
        presenter, graph, from, mode, insert_ops,
    ))
}

pub(super) fn plan_connection_insert_menu_candidate<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    from: PortId,
    at: CanvasPoint,
    mode: NodeGraphConnectionMode,
    candidate: &InsertNodeCandidate,
) -> ConnectionInsertMenuPlan {
    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            plan_connection_insert_menu_candidate_with_graph::<M>(
                presenter, graph, from, at, mode, candidate,
            )
        })
        .ok()
        .unwrap_or(ConnectionInsertMenuPlan::Ignore)
}
