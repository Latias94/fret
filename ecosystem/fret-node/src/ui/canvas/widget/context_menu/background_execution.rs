mod activate;
mod apply;
mod plan;

use crate::ui::canvas::widget::*;

#[derive(Debug)]
pub(super) enum BackgroundInsertMenuPlan {
    Apply(Vec<GraphOp>),
    Reject(DiagnosticSeverity, Arc<str>),
    Ignore,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn plan_background_insert_menu_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> BackgroundInsertMenuPlan {
        plan::plan_background_insert_menu_candidate(self, host, candidate, at)
    }

    pub(super) fn apply_background_insert_menu_plan<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        plan: BackgroundInsertMenuPlan,
    ) {
        apply::apply_background_insert_menu_plan(self, cx, plan)
    }

    pub(super) fn activate_background_context_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        at: CanvasPoint,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        activate::activate_background_context_action(self, cx, at, action, menu_candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Graph, GraphId, NodeKindKey};
    use crate::ui::DefaultNodeGraphPresenter;
    use serde_json::Value;

    fn regular_candidate() -> InsertNodeCandidate {
        InsertNodeCandidate {
            kind: NodeKindKey::new("regular"),
            label: Arc::<str>::from("Regular"),
            enabled: true,
            template: None,
            payload: Value::Null,
        }
    }

    #[test]
    fn background_insert_menu_plan_surfaces_create_node_errors() {
        let mut presenter = DefaultNodeGraphPresenter::default();
        let graph = Graph::new(GraphId::new());
        let candidate = regular_candidate();
        let plan = plan::plan_background_insert_menu_candidate_with_graph::<
            NoopNodeGraphCanvasMiddleware,
        >(
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
}
