use crate::ui::canvas::widget::*;

#[derive(Debug)]
pub(super) enum BackgroundInsertMenuPlan {
    Apply(Vec<GraphOp>),
    Reject(DiagnosticSeverity, Arc<str>),
    Ignore,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn plan_background_insert_menu_candidate_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> BackgroundInsertMenuPlan {
        match Self::plan_insert_candidate_ops_with_graph(presenter, graph, candidate, at) {
            Ok(ops) => BackgroundInsertMenuPlan::Apply(ops),
            Err(msg) => BackgroundInsertMenuPlan::Reject(DiagnosticSeverity::Info, msg),
        }
    }

    pub(super) fn plan_background_insert_menu_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> BackgroundInsertMenuPlan {
        let presenter = &mut *self.presenter;
        self.graph
            .read_ref(host, |graph| {
                Self::plan_background_insert_menu_candidate_with_graph(
                    presenter, graph, candidate, at,
                )
            })
            .ok()
            .unwrap_or(BackgroundInsertMenuPlan::Ignore)
    }

    pub(super) fn apply_background_insert_menu_plan<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        plan: BackgroundInsertMenuPlan,
    ) {
        match plan {
            BackgroundInsertMenuPlan::Apply(ops) => {
                let node_id = Self::first_added_node_id(&ops);
                if self.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                    self.select_inserted_node(cx.app, node_id);
                }
            }
            BackgroundInsertMenuPlan::Reject(sev, msg) => {
                self.show_toast(cx.app, cx.window, sev, msg);
            }
            BackgroundInsertMenuPlan::Ignore => {}
        }
    }

    pub(super) fn activate_background_context_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        at: CanvasPoint,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        match action {
            NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix) => {
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return true;
                };
                self.record_recent_kind(&candidate.kind);
                let plan = self.plan_background_insert_menu_candidate(cx.app, &candidate, at);
                self.apply_background_insert_menu_plan(cx, plan);
                true
            }
            _ => false,
        }
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
        let plan = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::plan_background_insert_menu_candidate_with_graph(
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
