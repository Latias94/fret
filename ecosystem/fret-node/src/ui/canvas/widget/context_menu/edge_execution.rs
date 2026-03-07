use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn activate_edge_context_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        edge_id: EdgeId,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
    ) -> bool {
        match action {
            NodeGraphContextMenuAction::OpenInsertNodePicker => {
                edge_insert::open_edge_insert_context_menu(self, cx, edge_id, invoked_at);
                true
            }
            NodeGraphContextMenuAction::InsertReroute => {
                let outcome = self.plan_canvas_split_edge_reroute(cx.app, edge_id, invoked_at);
                match outcome {
                    Some(Ok(ops)) => {
                        self.apply_split_edge_reroute_ops(cx.app, cx.window, None, ops);
                    }
                    Some(Err(diags)) => {
                        let (sev, msg) = Self::split_edge_reroute_rejection_toast(&diags);
                        self.show_toast(cx.app, cx.window, sev, msg);
                    }
                    None => {}
                }
                true
            }
            NodeGraphContextMenuAction::DeleteEdge => {
                let remove_ops = {
                    let this = &*self;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            graph
                                .edges
                                .get(&edge_id)
                                .map(|edge| {
                                    vec![GraphOp::RemoveEdge {
                                        id: edge_id,
                                        edge: edge.clone(),
                                    }]
                                })
                                .unwrap_or_default()
                        })
                        .ok()
                        .unwrap_or_default()
                };
                self.apply_ops(cx.app, cx.window, remove_ops);
                self.update_view_state(cx.app, |view_state| {
                    view_state.selected_edges.retain(|id| *id != edge_id);
                });
                true
            }
            NodeGraphContextMenuAction::Custom(action_id) => {
                let ops = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            presenter.on_edge_context_menu_action(graph, edge_id, action_id)
                        })
                        .ok()
                        .flatten()
                        .unwrap_or_default()
                };
                if !ops.is_empty() {
                    self.apply_ops(cx.app, cx.window, ops);
                }
                true
            }
            _ => false,
        }
    }
}
