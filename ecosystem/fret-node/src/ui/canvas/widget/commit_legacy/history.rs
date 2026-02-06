use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(dead_code)]
    pub(in super::super) fn undo_last_legacy<H: UiHost>(
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
    pub(in super::super) fn redo_last_legacy<H: UiHost>(
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
