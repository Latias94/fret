use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(dead_code)]
    pub(in super::super) fn commit_ops_legacy<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        label: Option<&str>,
        ops: Vec<GraphOp>,
    ) -> bool {
        if ops.is_empty() {
            return true;
        }

        let tx = GraphTransaction {
            label: label.map(|s| s.to_string()),
            ops,
        };
        self.commit_transaction(host, window, &tx)
    }

    #[allow(dead_code)]
    pub(in super::super) fn commit_transaction_legacy<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        tx: &GraphTransaction,
    ) -> bool {
        let mut tx = crate::ops::normalize_transaction(tx.clone());
        if tx.is_empty() {
            return true;
        }
        let snapshot = self.sync_view_state(host);
        let outcome = {
            let ctx = NodeGraphCanvasMiddlewareCx {
                graph: &self.graph,
                view_state: &self.view_state,
                style: &self.style,
                bounds: self.interaction.last_bounds,
                pan: snapshot.pan,
                zoom: snapshot.zoom,
            };
            self.middleware.before_commit(host, window, &ctx, &mut tx)
        };

        match outcome {
            NodeGraphCanvasCommitOutcome::Continue => {}
            NodeGraphCanvasCommitOutcome::Reject { diagnostics } => {
                if let Some((sev, msg)) = Self::toast_from_diagnostics(&diagnostics) {
                    self.show_toast(host, window, sev, msg);
                }
                return false;
            }
        }

        tx = crate::ops::normalize_transaction(tx);
        if tx.is_empty() {
            return true;
        }

        match self.apply_transaction_result(host, &tx) {
            Ok(committed) => {
                if self.store.is_none() {
                    self.history.record(committed);
                }
                true
            }
            Err(diags) => {
                if let Some((sev, msg)) = Self::toast_from_diagnostics(&diags) {
                    self.show_toast(host, window, sev, msg);
                }
                false
            }
        }
    }
}
