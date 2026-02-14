use super::*;

impl UiDiagnosticsService {
    #[cfg(feature = "diagnostics-ws")]
    fn ws_is_configured(&self) -> bool {
        self.cfg.devtools_ws_url.is_some() && self.cfg.devtools_token.is_some()
    }

    #[cfg(not(feature = "diagnostics-ws"))]
    fn ws_is_configured(&self) -> bool {
        false
    }

    #[cfg(feature = "diagnostics-ws")]
    fn poll_ws_inbox(&mut self) {
        if !self.ws_is_configured() {
            return;
        }

        let mut msgs: Vec<DiagTransportMessageV1> = Vec::new();
        self.ws_bridge.drain_inbox(
            self.cfg.devtools_ws_url.as_deref(),
            self.cfg.devtools_token.as_deref(),
            self.cfg.screenshots_enabled,
            &mut msgs,
        );

        for msg in msgs {
            self.apply_ws_message(msg);
        }
    }

    #[cfg(not(feature = "diagnostics-ws"))]
    fn poll_ws_inbox(&mut self) {}

    #[cfg(feature = "diagnostics-ws")]
    fn ws_send_with_request_id(
        &mut self,
        ty: impl Into<String>,
        request_id: Option<u64>,
        payload: serde_json::Value,
    ) {
        if !self.ws_is_configured() {
            return;
        }
        self.ws_bridge.send(
            self.cfg.devtools_ws_url.as_deref(),
            self.cfg.devtools_token.as_deref(),
            self.cfg.screenshots_enabled,
            DiagTransportMessageV1 {
                schema_version: 1,
                r#type: ty.into(),
                session_id: None,
                request_id,
                payload,
            },
        );
    }

    #[cfg(feature = "diagnostics-ws")]
    fn ws_send(&mut self, ty: impl Into<String>, payload: serde_json::Value) {
        self.ws_send_with_request_id(ty, None, payload);
    }

    #[cfg(feature = "diagnostics-ws")]
    fn apply_ws_message(&mut self, msg: DiagTransportMessageV1) {
        match msg.r#type.as_str() {
            "inspect.set" => {
                let Ok(cfg) = serde_json::from_value::<UiInspectConfigV1>(msg.payload) else {
                    return;
                };
                self.inspect_enabled = cfg.enabled;
                self.inspect_consume_clicks = cfg.consume_clicks;
                if !self.inspect_enabled {
                    self.inspect_locked_windows.clear();
                }
            }
            "pick.arm" => {
                self.pending_pick = None;
                self.pick_armed_run_id = Some(self.next_pick_run_id());
            }
            "bundle.dump" => {
                let (label, dump_max_snapshots) =
                    if let Ok(req) = serde_json::from_value::<DevtoolsBundleDumpV1>(msg.payload) {
                        (
                            req.label.unwrap_or_else(|| "bundle".to_string()),
                            req.max_snapshots.map(|n| n as usize),
                        )
                    } else {
                        ("bundle".to_string(), None)
                    };
                self.request_force_dump(label, dump_max_snapshots, None, None, msg.request_id);
            }
            "screenshot.request" => {
                let Ok(req) = serde_json::from_value::<DevtoolsScreenshotRequestV1>(msg.payload)
                else {
                    return;
                };

                let request_id_str = msg
                    .request_id
                    .map(|id| format!("devtools-screenshot-{id}"))
                    .unwrap_or_else(|| format!("devtools-screenshot-{}", unix_ms_now()));

                if cfg!(target_arch = "wasm32") {
                    let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                        schema_version: 1,
                        status: "unsupported".to_string(),
                        reason: Some("screenshots_not_supported_wasm".to_string()),
                        request_id: request_id_str,
                        window: req.window.unwrap_or(0),
                        bundle_dir_name: "".to_string(),
                        screenshots_dir: None,
                        entry: None,
                    })
                    .unwrap_or(serde_json::Value::Null);
                    self.ws_send_with_request_id("screenshot.result", msg.request_id, payload);
                    return;
                }

                if !self.cfg.screenshots_enabled {
                    let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                        schema_version: 1,
                        status: "disabled".to_string(),
                        reason: Some("screenshots_disabled".to_string()),
                        request_id: request_id_str,
                        window: req.window.unwrap_or(0),
                        bundle_dir_name: "".to_string(),
                        screenshots_dir: None,
                        entry: None,
                    })
                    .unwrap_or(serde_json::Value::Null);
                    self.ws_send_with_request_id("screenshot.result", msg.request_id, payload);
                    return;
                }

                if self.pending_devtools_screenshot.is_some() {
                    let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                        schema_version: 1,
                        status: "failed".to_string(),
                        reason: Some("busy".to_string()),
                        request_id: request_id_str,
                        window: req.window.unwrap_or(0),
                        bundle_dir_name: "".to_string(),
                        screenshots_dir: None,
                        entry: None,
                    })
                    .unwrap_or(serde_json::Value::Null);
                    self.ws_send_with_request_id("screenshot.result", msg.request_id, payload);
                    return;
                }

                let window_ffi = if let Some(window) = req.window {
                    let want = AppWindowId::from(KeyData::from_ffi(window));
                    if !self.known_windows.contains(&want) {
                        let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                            schema_version: 1,
                            status: "failed".to_string(),
                            reason: Some("unknown_window".to_string()),
                            request_id: request_id_str,
                            window,
                            bundle_dir_name: "".to_string(),
                            screenshots_dir: None,
                            entry: None,
                        })
                        .unwrap_or(serde_json::Value::Null);
                        self.ws_send_with_request_id("screenshot.result", msg.request_id, payload);
                        return;
                    }
                    window
                } else if let Some(first) = self.known_windows.first().copied() {
                    first.data().as_ffi()
                } else {
                    let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                        schema_version: 1,
                        status: "failed".to_string(),
                        reason: Some("no_window".to_string()),
                        request_id: request_id_str,
                        window: 0,
                        bundle_dir_name: "".to_string(),
                        screenshots_dir: None,
                        entry: None,
                    })
                    .unwrap_or(serde_json::Value::Null);
                    self.ws_send_with_request_id("screenshot.result", msg.request_id, payload);
                    return;
                };

                self.pending_devtools_screenshot = Some(PendingDevtoolsScreenshotRequest {
                    request_id: msg.request_id,
                    request_id_str,
                    label: req.label.map(|s| sanitize_label(&s)),
                    timeout_frames: req.timeout_frames,
                    window_ffi,
                    bundle_dir_name: None,
                    remaining_frames: req.timeout_frames,
                    last_result_trigger_stamp: None,
                    started: false,
                });
            }
            "semantics.node.get" => {
                let Ok(req) = serde_json::from_value::<UiSemanticsNodeGetV1>(msg.payload) else {
                    return;
                };

                let want = AppWindowId::from(KeyData::from_ffi(req.window));
                if !self.known_windows.contains(&want) {
                    let payload = serde_json::to_value(UiSemanticsNodeGetAckV1 {
                        schema_version: 1,
                        status: "no_semantics".to_string(),
                        reason: Some("unknown_window".to_string()),
                        window: req.window,
                        node_id: req.node_id,
                        semantics_fingerprint: None,
                        node: None,
                        children: Vec::new(),
                        captured_unix_ms: Some(unix_ms_now()),
                    })
                    .unwrap_or(serde_json::Value::Null);
                    self.ws_send_with_request_id("semantics.node.get_ack", msg.request_id, payload);
                    return;
                }

                self.pending_devtools_semantics_node_get =
                    Some(PendingDevtoolsSemanticsNodeGetRequest {
                        request_id: msg.request_id,
                        window_ffi: req.window,
                        node_id: req.node_id,
                    });
            }
            "app.exit.request" => {
                let delay_ms = serde_json::from_value::<DevtoolsAppExitRequestV1>(msg.payload)
                    .ok()
                    .and_then(|req| req.delay_ms)
                    .unwrap_or(0);
                self.ws_exit_deadline_unix_ms = Some(unix_ms_now().saturating_add(delay_ms));
            }
            "script.push" | "script.run" => {
                let script_value = msg
                    .payload
                    .get("script")
                    .cloned()
                    .unwrap_or_else(|| msg.payload.clone());
                let Some(script) = PendingScript::from_json_value(script_value) else {
                    return;
                };
                let run_id = self.next_script_run_id();
                self.pending_script = Some(script);
                self.pending_script_run_id = Some(run_id);
                self.write_script_result(UiScriptResultV1 {
                    schema_version: 1,
                    run_id,
                    updated_unix_ms: unix_ms_now(),
                    window: None,
                    stage: UiScriptStageV1::Queued,
                    step_index: None,
                    reason_code: None,
                    reason: None,
                    evidence: None,
                    last_bundle_dir: self
                        .last_dump_dir
                        .as_ref()
                        .map(|p| display_path(&self.cfg.out_dir, p)),
                    last_bundle_artifact: self.last_dump_artifact_stats.clone(),
                });
            }
            _ => {}
        }
    }
}

