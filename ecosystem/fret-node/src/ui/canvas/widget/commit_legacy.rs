use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(dead_code)]
    pub(super) fn apply_transaction_result_legacy<H: UiHost>(
        &mut self,
        host: &mut H,
        tx: &GraphTransaction,
    ) -> Result<GraphTransaction, Vec<Diagnostic>> {
        if let Some(store) = self.store.as_ref() {
            if let Some(profile) = self.presenter.profile_mut() {
                match store.update(host, |store, _cx| {
                    store.dispatch_transaction_with_profile(tx, profile)
                }) {
                    Ok(Ok(outcome)) => {
                        self.sync_view_state(host);
                        self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                        return Ok(outcome.committed);
                    }
                    Ok(Err(err)) => match &err {
                        ApplyPipelineError::Rejected {
                            diagnostics: diags, ..
                        } => return Err(diags.clone()),
                        _ => {
                            return Err(vec![Diagnostic {
                                key: "tx.apply_failed".to_string(),
                                severity: DiagnosticSeverity::Error,
                                target: crate::rules::DiagnosticTarget::Graph,
                                message: err.to_string(),
                                fixes: Vec::new(),
                            }]);
                        }
                    },
                    Err(_) => {
                        return Err(vec![Diagnostic {
                            key: "tx.apply_failed".to_string(),
                            severity: DiagnosticSeverity::Error,
                            target: crate::rules::DiagnosticTarget::Graph,
                            message: "store unavailable".to_string(),
                            fixes: Vec::new(),
                        }]);
                    }
                }
            }

            match store.update(host, |store, _cx| store.dispatch_transaction(tx)) {
                Ok(Ok(outcome)) => {
                    self.sync_view_state(host);
                    self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                    return Ok(outcome.committed);
                }
                Ok(Err(err)) => {
                    let message = match &err {
                        crate::runtime::store::DispatchError::Apply(err) => match err {
                            ApplyPipelineError::Rejected {
                                diagnostics: diags, ..
                            } => return Err(diags.clone()),
                            _ => err.to_string(),
                        },
                        crate::runtime::store::DispatchError::Changes(err) => err.to_string(),
                    };
                    return Err(vec![Diagnostic {
                        key: "tx.apply_failed".to_string(),
                        severity: DiagnosticSeverity::Error,
                        target: crate::rules::DiagnosticTarget::Graph,
                        message,
                        fixes: Vec::new(),
                    }]);
                }
                Err(_) => {
                    return Err(vec![Diagnostic {
                        key: "tx.apply_failed".to_string(),
                        severity: DiagnosticSeverity::Error,
                        target: crate::rules::DiagnosticTarget::Graph,
                        message: "store unavailable".to_string(),
                        fixes: Vec::new(),
                    }]);
                }
            }
        }

        let Some(mut scratch) = self.graph.read_ref(host, |g| g.clone()).ok() else {
            return Err(vec![Diagnostic {
                key: "tx.graph_unavailable".to_string(),
                severity: DiagnosticSeverity::Error,
                target: crate::rules::DiagnosticTarget::Graph,
                message: "graph unavailable".to_string(),
                fixes: Vec::new(),
            }]);
        };

        let committed = if let Some(profile) = self.presenter.profile_mut() {
            match apply_transaction_with_profile(&mut scratch, profile, tx) {
                Ok(committed) => committed,
                Err(err) => match &err {
                    ApplyPipelineError::Rejected {
                        diagnostics: diags, ..
                    } => return Err(diags.clone()),
                    _ => {
                        return Err(vec![Diagnostic {
                            key: "tx.apply_failed".to_string(),
                            severity: DiagnosticSeverity::Error,
                            target: crate::rules::DiagnosticTarget::Graph,
                            message: err.to_string(),
                            fixes: Vec::new(),
                        }]);
                    }
                },
            }
        } else {
            match apply_transaction(&mut scratch, tx) {
                Ok(()) => GraphTransaction {
                    label: tx.label.clone(),
                    ops: tx.ops.clone(),
                },
                Err(err) => {
                    return Err(vec![Diagnostic {
                        key: "tx.apply_failed".to_string(),
                        severity: DiagnosticSeverity::Error,
                        target: crate::rules::DiagnosticTarget::Graph,
                        message: err.to_string(),
                        fixes: Vec::new(),
                    }]);
                }
            }
        };

        let _ = self.graph.update(host, |g, _cx| {
            *g = scratch;
        });

        let changes = NodeGraphChanges::from_transaction(&committed);
        self.emit_graph_callbacks(&committed, &changes);
        Ok(committed)
    }

    #[allow(dead_code)]
    pub(super) fn commit_ops_legacy<H: UiHost>(
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
    pub(super) fn commit_transaction_legacy<H: UiHost>(
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

    #[allow(dead_code)]
    pub(super) fn undo_last_legacy<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
    ) -> bool {
        let preferred_focus = self.interaction.focused_edge;

        if let Some(store) = self.store.as_ref() {
            if let Some(profile) = self.presenter.profile_mut() {
                match store.update(host, |store, _cx| store.undo_with_profile(profile)) {
                    Ok(Ok(Some(outcome))) => {
                        self.sync_view_state(host);
                        self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                        self.update_view_state(host, |s| {
                            s.selected_edges.clear();
                            s.selected_nodes.clear();
                            s.selected_groups.clear();
                        });
                        self.repair_focused_edge_after_graph_change(host, preferred_focus);
                        return true;
                    }
                    Ok(Ok(None)) => return false,
                    Ok(Err(err)) => {
                        self.show_toast(
                            host,
                            window,
                            DiagnosticSeverity::Error,
                            Arc::<str>::from(err.to_string()),
                        );
                        return false;
                    }
                    Err(_) => {
                        self.show_toast(
                            host,
                            window,
                            DiagnosticSeverity::Error,
                            Arc::<str>::from("store unavailable"),
                        );
                        return false;
                    }
                }
            }

            match store.update(host, |store, _cx| store.undo()) {
                Ok(Ok(Some(outcome))) => {
                    self.sync_view_state(host);
                    self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                    self.update_view_state(host, |s| {
                        s.selected_edges.clear();
                        s.selected_nodes.clear();
                        s.selected_groups.clear();
                    });
                    self.repair_focused_edge_after_graph_change(host, preferred_focus);
                    return true;
                }
                Ok(Ok(None)) => return false,
                Ok(Err(err)) => {
                    self.show_toast(
                        host,
                        window,
                        DiagnosticSeverity::Error,
                        Arc::<str>::from(err.to_string()),
                    );
                    return false;
                }
                Err(_) => {
                    self.show_toast(
                        host,
                        window,
                        DiagnosticSeverity::Error,
                        Arc::<str>::from("store unavailable"),
                    );
                    return false;
                }
            }
        }

        let mut history = std::mem::take(&mut self.history);
        let result = history.undo(|tx| self.apply_transaction_result(host, tx));
        self.history = history;

        match result {
            Ok(true) => {
                self.update_view_state(host, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes.clear();
                    s.selected_groups.clear();
                });
                self.repair_focused_edge_after_graph_change(host, preferred_focus);
                true
            }
            Ok(false) => false,
            Err(diags) => {
                if let Some((sev, msg)) = Self::toast_from_diagnostics(&diags) {
                    self.show_toast(host, window, sev, msg);
                }
                false
            }
        }
    }

    #[allow(dead_code)]
    pub(super) fn redo_last_legacy<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
    ) -> bool {
        let preferred_focus = self.interaction.focused_edge;

        if let Some(store) = self.store.as_ref() {
            if let Some(profile) = self.presenter.profile_mut() {
                match store.update(host, |store, _cx| store.redo_with_profile(profile)) {
                    Ok(Ok(Some(outcome))) => {
                        self.sync_view_state(host);
                        self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                        self.update_view_state(host, |s| {
                            s.selected_edges.clear();
                            s.selected_nodes.clear();
                            s.selected_groups.clear();
                        });
                        self.repair_focused_edge_after_graph_change(host, preferred_focus);
                        return true;
                    }
                    Ok(Ok(None)) => return false,
                    Ok(Err(err)) => {
                        self.show_toast(
                            host,
                            window,
                            DiagnosticSeverity::Error,
                            Arc::<str>::from(err.to_string()),
                        );
                        return false;
                    }
                    Err(_) => {
                        self.show_toast(
                            host,
                            window,
                            DiagnosticSeverity::Error,
                            Arc::<str>::from("store unavailable"),
                        );
                        return false;
                    }
                }
            }

            match store.update(host, |store, _cx| store.redo()) {
                Ok(Ok(Some(outcome))) => {
                    self.sync_view_state(host);
                    self.emit_graph_callbacks(&outcome.committed, &outcome.changes);
                    self.update_view_state(host, |s| {
                        s.selected_edges.clear();
                        s.selected_nodes.clear();
                        s.selected_groups.clear();
                    });
                    self.repair_focused_edge_after_graph_change(host, preferred_focus);
                    return true;
                }
                Ok(Ok(None)) => return false,
                Ok(Err(err)) => {
                    self.show_toast(
                        host,
                        window,
                        DiagnosticSeverity::Error,
                        Arc::<str>::from(err.to_string()),
                    );
                    return false;
                }
                Err(_) => {
                    self.show_toast(
                        host,
                        window,
                        DiagnosticSeverity::Error,
                        Arc::<str>::from("store unavailable"),
                    );
                    return false;
                }
            }
        }

        let mut history = std::mem::take(&mut self.history);
        let result = history.redo(|tx| self.apply_transaction_result(host, tx));
        self.history = history;

        match result {
            Ok(true) => {
                self.update_view_state(host, |s| {
                    s.selected_edges.clear();
                    s.selected_nodes.clear();
                    s.selected_groups.clear();
                });
                self.repair_focused_edge_after_graph_change(host, preferred_focus);
                true
            }
            Ok(false) => false,
            Err(diags) => {
                if let Some((sev, msg)) = Self::toast_from_diagnostics(&diags) {
                    self.show_toast(host, window, sev, msg);
                }
                false
            }
        }
    }
}
