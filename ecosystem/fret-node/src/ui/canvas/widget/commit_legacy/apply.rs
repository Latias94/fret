use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(dead_code)]
    pub(in super::super) fn apply_transaction_result_legacy<H: UiHost>(
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
}
