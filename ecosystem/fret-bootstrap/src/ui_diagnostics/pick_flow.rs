use super::*;

impl UiDiagnosticsService {
    pub(super) fn write_pick_result(&mut self, result: UiPickResultV1) {
        if !self.is_enabled() {
            return;
        }

        if !cfg!(target_arch = "wasm32") {
            let _ = write_json(self.cfg.pick_result_path.clone(), &result);
            let _ = touch_file(&self.cfg.pick_result_trigger_path);
        }

        #[cfg(feature = "diagnostics-ws")]
        {
            self.ws_send_pick_result_v1(&result);
        }
    }

    pub(super) fn resolve_pending_pick_for_window(
        &mut self,
        pending: PendingPick,
        raw_semantics: Option<&fret_core::SemanticsSnapshot>,
        ui: &UiTree<App>,
        element_runtime: Option<&ElementRuntime>,
    ) {
        let window = pending.window;
        let position = pending.position;

        let mut result = UiPickResultV1 {
            schema_version: 1,
            run_id: pending.run_id,
            updated_unix_ms: unix_ms_now(),
            window: Some(window.data().as_ffi()),
            stage: UiPickStageV1::Failed,
            position: Some(PointV1::from(position)),
            selection: None,
            reason: None,
            last_bundle_dir: self
                .last_dump_dir
                .as_ref()
                .map(|p| display_path(&self.cfg.out_dir, p)),
        };

        if let Some(snapshot) = raw_semantics {
            let selection = pick_semantics_node_at(snapshot, ui, position).map(|node| {
                let (element, element_path) = element_runtime
                    .and_then(|runtime| {
                        runtime.element_for_node(window, node.id).map(|id| {
                            let path = runtime.debug_path_for_element(window, id);
                            (Some(id.0), path)
                        })
                    })
                    .unwrap_or((None, None));
                UiPickSelectionV1::from_node(snapshot, node, element, element_path, &self.cfg)
            });

            match selection {
                Some(sel) => {
                    result.stage = UiPickStageV1::Picked;

                    let selector_json = snapshot
                        .nodes
                        .iter()
                        .find(|n| n.id.data().as_ffi() == sel.node.id)
                        .and_then(|node| {
                            let element = element_runtime
                                .and_then(|runtime| runtime.element_for_node(window, node.id))
                                .map(|id| id.0);
                            best_selector_for_node_validated(
                                snapshot,
                                window,
                                element_runtime,
                                node,
                                element,
                                &self.cfg,
                            )
                            .or_else(|| best_selector_for_node(snapshot, node, element, &self.cfg))
                        })
                        .or_else(|| sel.selectors.first().cloned())
                        .and_then(|sel| serde_json::to_string(&sel).ok());

                    self.inspector
                        .on_pick_success(window, sel.node.id, selector_json.clone());
                    result.selection = Some(sel);
                }
                None => {
                    result.reason = Some("no matching semantics node under pointer".to_string());
                }
            }
        } else {
            result.reason = Some("no semantics snapshot".to_string());
        }

        if self.cfg.pick_auto_dump {
            if let Some(dir) = self.dump_bundle(Some("pick")) {
                result.last_bundle_dir = Some(display_path(&self.cfg.out_dir, &dir));
            }
        }

        self.write_pick_result(result);
    }
}
