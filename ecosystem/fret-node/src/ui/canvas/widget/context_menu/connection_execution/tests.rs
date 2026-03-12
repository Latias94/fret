use super::*;
use crate::core::{Graph, GraphId, PortId};
use crate::ui::{DefaultNodeGraphPresenter, NodeGraphStyle};

#[test]
fn connection_insert_menu_plan_surfaces_create_node_errors() {
    let mut presenter = DefaultNodeGraphPresenter::default();
    let graph = Graph::new(GraphId::new());
    let candidate = super::test_support::regular_candidate();
    let plan =
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::plan_connection_insert_menu_candidate_with_graph(
            &mut presenter,
            &graph,
            PortId::new(),
            CanvasPoint { x: 10.0, y: 20.0 },
            NodeGraphConnectionMode::Strict,
            &candidate,
        );

    assert!(matches!(
        plan,
        ConnectionInsertMenuPlan::Reject(DiagnosticSeverity::Info, ref msg)
            if &**msg == "node insertion is not supported"
    ));
}

#[test]
fn connection_conversion_menu_plan_rejects_missing_template() {
    let mut presenter = DefaultNodeGraphPresenter::default();
    let graph = Graph::new(GraphId::new());
    let candidate = super::test_support::regular_candidate();
    let plan =
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::plan_connection_conversion_menu_candidate_with_graph(
            &mut presenter,
            &graph,
            &NodeGraphStyle::default(),
            1.0,
            PortId::new(),
            PortId::new(),
            CanvasPoint { x: 10.0, y: 20.0 },
            &candidate,
        );

    assert!(matches!(
        plan,
        ConnectionConversionMenuPlan::Reject(DiagnosticSeverity::Error, ref msg)
            if &**msg == "conversion candidate is missing template"
    ));
}
