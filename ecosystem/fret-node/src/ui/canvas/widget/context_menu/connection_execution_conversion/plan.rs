use crate::ui::canvas::widget::*;

use super::ConnectionConversionMenuPlan;

pub(super) fn plan_connection_conversion_menu_candidate_with_graph<M: NodeGraphCanvasMiddleware>(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    style: &NodeGraphStyle,
    zoom: f32,
    from: PortId,
    to: PortId,
    at: CanvasPoint,
    candidate: &InsertNodeCandidate,
) -> ConnectionConversionMenuPlan {
    let template = match &candidate.template {
        Some(template) => template,
        None => {
            return ConnectionConversionMenuPlan::Reject(
                DiagnosticSeverity::Error,
                Arc::<str>::from("conversion candidate is missing template"),
            );
        }
    };
    let plan =
        conversion::plan_insert_conversion(presenter, graph, style, zoom, from, to, at, template);
    match plan.decision {
        ConnectDecision::Accept => ConnectionConversionMenuPlan::Apply(plan.ops),
        ConnectDecision::Reject => {
            NodeGraphCanvasWith::<M>::toast_from_diagnostics(&plan.diagnostics)
                .map(|(severity, message)| ConnectionConversionMenuPlan::Reject(severity, message))
                .unwrap_or(ConnectionConversionMenuPlan::Ignore)
        }
    }
}

pub(super) fn plan_connection_conversion_menu_candidate<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    from: PortId,
    to: PortId,
    at: CanvasPoint,
    candidate: &InsertNodeCandidate,
) -> ConnectionConversionMenuPlan {
    let zoom = canvas.cached_zoom;
    let style = canvas.style.clone();
    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            plan_connection_conversion_menu_candidate_with_graph::<M>(
                presenter, graph, &style, zoom, from, to, at, candidate,
            )
        })
        .ok()
        .unwrap_or(ConnectionConversionMenuPlan::Ignore)
}
