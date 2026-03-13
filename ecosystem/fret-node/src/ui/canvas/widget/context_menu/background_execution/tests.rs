use super::*;
use crate::core::{Graph, GraphId};
use crate::ui::DefaultNodeGraphPresenter;

#[test]
fn background_insert_menu_plan_surfaces_create_node_errors() {
    let mut presenter = DefaultNodeGraphPresenter::default();
    let graph = Graph::new(GraphId::new());
    let candidate = super::test_support::regular_candidate();
    let plan =
        plan::plan_background_insert_menu_candidate_with_graph::<NoopNodeGraphCanvasMiddleware>(
            &mut presenter,
            &graph,
            &candidate,
            CanvasPoint { x: 10.0, y: 20.0 },
        );

    assert!(matches!(
        plan,
        BackgroundInsertMenuPlan::Reject(DiagnosticSeverity::Info, ref msg)
            if &**msg == "node insertion is not supported"
    ));
}
