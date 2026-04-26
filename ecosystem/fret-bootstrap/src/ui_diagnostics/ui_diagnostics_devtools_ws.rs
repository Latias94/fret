#[cfg(feature = "diagnostics-ws")]
use super::bundle::ui_diagnostics_monitor_topology_from_runner;
use super::*;

#[cfg(feature = "diagnostics-ws")]
use fret_diag_protocol::{
    DevtoolsEnvironmentSourcesGetAckV1, DevtoolsEnvironmentSourcesGetV1, DiagScreenshotRequestV1,
    DiagScreenshotWindowRequestV1, EnvironmentSourceAvailabilityV1, FilesystemEnvironmentSourceV1,
    HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1, HostMonitorTopologyEnvironmentPayloadV1,
    PLATFORM_CAPABILITIES_ENVIRONMENT_SOURCE_ID_V1, UiInspectConfigV1,
};

#[cfg(feature = "diagnostics-ws")]
#[derive(Debug, Clone)]
pub(super) struct PendingDevtoolsScreenshotRequest {
    pub(super) request_id: Option<u64>,
    pub(super) request_id_str: String,
    pub(super) label: Option<String>,
    pub(super) timeout_frames: u32,
    pub(super) window_ffi: u64,
    pub(super) bundle_dir_name: Option<String>,
    pub(super) remaining_frames: u32,
    pub(super) last_result_trigger_stamp: Option<u64>,
    pub(super) started: bool,
}

#[cfg(feature = "diagnostics-ws")]
#[derive(Debug, Clone)]
pub(super) struct PendingDevtoolsSemanticsNodeGetRequest {
    pub(super) request_id: Option<u64>,
    pub(super) window_ffi: u64,
    pub(super) node_id: u64,
}

#[cfg(feature = "diagnostics-ws")]
#[derive(Debug, Clone)]
pub(super) struct PendingDevtoolsHitTestExplainRequest {
    pub(super) request_id: Option<u64>,
    pub(super) window_ffi: u64,
    pub(super) target: UiSelectorV1,
}

#[cfg(feature = "diagnostics-ws")]
fn build_environment_sources_get_ack_v1(
    host_monitor_topology: Option<&fret_runtime::RunnerMonitorTopologySnapshotV1>,
    platform_capabilities: Option<&fret_runtime::PlatformCapabilities>,
) -> DevtoolsEnvironmentSourcesGetAckV1 {
    let host_monitor_topology =
        host_monitor_topology.map(|snapshot| HostMonitorTopologyEnvironmentPayloadV1 {
            schema_version: 1,
            source_id: HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1.to_string(),
            monitor_topology: ui_diagnostics_monitor_topology_from_runner(snapshot),
        });
    let platform_capabilities = platform_capabilities
        .map(super::fs_triggers::ui_diagnostics_platform_capabilities_environment_payload);
    let mut sources = Vec::new();
    if host_monitor_topology.is_some() {
        sources.push(FilesystemEnvironmentSourceV1 {
            source_id: HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1.to_string(),
            availability: EnvironmentSourceAvailabilityV1::PreflightTransportSession,
        });
    }
    if platform_capabilities.is_some() {
        sources.push(FilesystemEnvironmentSourceV1 {
            source_id: PLATFORM_CAPABILITIES_ENVIRONMENT_SOURCE_ID_V1.to_string(),
            availability: EnvironmentSourceAvailabilityV1::PreflightTransportSession,
        });
    }
    DevtoolsEnvironmentSourcesGetAckV1 {
        schema_version: 1,
        sources,
        runner_kind: Some("fret-bootstrap".to_string()),
        runner_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        host_monitor_topology,
        platform_capabilities,
    }
}

impl UiDiagnosticsService {
    pub(super) fn drive_devtools_requests_for_window(
        &mut self,
        app: &App,
        window: AppWindowId,
        scale_factor: f32,
        ui: Option<&mut UiTree<App>>,
    ) -> bool {
        #[cfg(feature = "diagnostics-ws")]
        {
            return self.drive_devtools_ws_requests_for_window(app, window, scale_factor, ui);
        }
        #[cfg(not(feature = "diagnostics-ws"))]
        {
            let _ = (app, window, scale_factor, ui);
            false
        }
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(super) fn ws_is_configured(&self) -> bool {
        self.cfg.devtools_ws_url.is_some() && self.cfg.devtools_token.is_some()
    }

    #[cfg(not(feature = "diagnostics-ws"))]
    pub(super) fn ws_is_configured(&self) -> bool {
        false
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(super) fn poll_ws_inbox(&mut self) {
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
    pub(super) fn poll_ws_inbox(&mut self) {}

    #[cfg(feature = "diagnostics-ws")]
    pub(super) fn ws_send_with_request_id(
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
    pub(super) fn ws_send(&mut self, ty: impl Into<String>, payload: serde_json::Value) {
        self.ws_send_with_request_id(ty, None, payload);
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(super) fn drive_devtools_ws_requests_for_window(
        &mut self,
        app: &App,
        window: AppWindowId,
        scale_factor: f32,
        mut ui: Option<&mut UiTree<App>>,
    ) -> bool {
        let mut request_redraw = false;
        request_redraw |= self.drive_devtools_ws_semantics_node_get(window, ui.as_deref());
        request_redraw |= self.drive_devtools_ws_hit_test_explain(window, ui.as_deref_mut());
        request_redraw |= self.drive_devtools_ws_screenshot_request(app, window, scale_factor);
        request_redraw
    }

    #[cfg(feature = "diagnostics-ws")]
    fn drive_devtools_ws_semantics_node_get(
        &mut self,
        window: AppWindowId,
        ui: Option<&UiTree<App>>,
    ) -> bool {
        let Some(pending) = self.pending_devtools_semantics_node_get.clone() else {
            return false;
        };
        if pending.window_ffi != window.data().as_ffi() {
            return false;
        }
        let pending = self
            .pending_devtools_semantics_node_get
            .take()
            .unwrap_or(pending);

        let raw = ui.and_then(|ui| ui.semantics_snapshot());
        let ack = build_semantics_node_get_ack_v1(
            raw,
            pending.window_ffi,
            pending.node_id,
            self.cfg.redact_text,
            self.cfg.max_debug_string_bytes,
        );
        let payload = serde_json::to_value(ack).unwrap_or(serde_json::Value::Null);
        self.ws_send_with_request_id("semantics.node.get_ack", pending.request_id, payload);
        false
    }

    #[cfg(feature = "diagnostics-ws")]
    fn drive_devtools_ws_hit_test_explain(
        &mut self,
        window: AppWindowId,
        ui: Option<&mut UiTree<App>>,
    ) -> bool {
        let Some(pending) = self.pending_devtools_hit_test_explain.clone() else {
            return false;
        };
        if pending.window_ffi != window.data().as_ffi() {
            return false;
        }
        let pending = self
            .pending_devtools_hit_test_explain
            .take()
            .unwrap_or(pending);

        let ack = build_hit_test_explain_ack_v1(
            ui,
            self.cfg.redact_text,
            self.cfg.max_debug_string_bytes,
            pending.window_ffi,
            pending.target.clone(),
        );
        let payload = serde_json::to_value(ack).unwrap_or(serde_json::Value::Null);
        self.ws_send_with_request_id("hit_test.explain_ack", pending.request_id, payload);
        false
    }

    #[cfg(feature = "diagnostics-ws")]
    fn drive_devtools_ws_screenshot_request(
        &mut self,
        app: &App,
        window: AppWindowId,
        scale_factor: f32,
    ) -> bool {
        let Some(mut pending) = self.pending_devtools_screenshot.take() else {
            return false;
        };
        if pending.window_ffi != window.data().as_ffi() {
            self.pending_devtools_screenshot = Some(pending);
            return false;
        }

        // Keep the app ticking while waiting for the runner-owned screenshot to complete.
        let request_redraw = true;

        if !pending.started {
            if self
                .active_scripts
                .values()
                .any(|active| active.screenshot_wait.is_some())
            {
                let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                    schema_version: 1,
                    status: "failed".to_string(),
                    reason: Some("busy".to_string()),
                    request_id: pending.request_id_str.clone(),
                    window: pending.window_ffi,
                    bundle_dir_name: "".to_string(),
                    screenshots_dir: None,
                    entry: None,
                })
                .unwrap_or(serde_json::Value::Null);
                self.ws_send_with_request_id("screenshot.result", pending.request_id, payload);
                return false;
            }

            let needs_fresh_bundle = pending.label.is_some();
            if needs_fresh_bundle || self.last_dump_dir.is_none() {
                let dump_label = pending
                    .label
                    .clone()
                    .unwrap_or_else(|| "devtools-screenshot".to_string());
                let _ = self.dump_bundle(Some(&dump_label));
            }

            let bundle_dir_name = self
                .last_dump_dir
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            if bundle_dir_name.is_empty() {
                let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                    schema_version: 1,
                    status: "failed".to_string(),
                    reason: Some("no_bundle_dir".to_string()),
                    request_id: pending.request_id_str.clone(),
                    window: pending.window_ffi,
                    bundle_dir_name,
                    screenshots_dir: None,
                    entry: None,
                })
                .unwrap_or(serde_json::Value::Null);
                self.ws_send_with_request_id("screenshot.result", pending.request_id, payload);
                return false;
            }

            let req = DiagScreenshotRequestV1 {
                schema_version: 1,
                out_dir: self.cfg.out_dir.to_string_lossy().to_string(),
                bundle_dir_name: bundle_dir_name.clone(),
                request_id: Some(pending.request_id_str.clone()),
                windows: vec![DiagScreenshotWindowRequestV1 {
                    window: pending.window_ffi,
                    tick_id: app.tick_id().0,
                    frame_id: app.frame_id().0,
                    scale_factor: scale_factor as f64,
                }],
            };

            let write_ok = serde_json::to_vec_pretty(&req).ok().is_some_and(|bytes| {
                if let Some(parent) = self.cfg.screenshot_request_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::write(&self.cfg.screenshot_request_path, bytes).is_ok()
                    && touch_file(&self.cfg.screenshot_trigger_path).is_ok()
            });

            if !write_ok {
                let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                    schema_version: 1,
                    status: "failed".to_string(),
                    reason: Some("screenshot_request_write_failed".to_string()),
                    request_id: pending.request_id_str.clone(),
                    window: pending.window_ffi,
                    bundle_dir_name: "".to_string(),
                    screenshots_dir: None,
                    entry: None,
                })
                .unwrap_or(serde_json::Value::Null);
                self.ws_send_with_request_id("screenshot.result", pending.request_id, payload);
                return false;
            }

            pending.bundle_dir_name = Some(bundle_dir_name);
            pending.started = true;
            pending.remaining_frames = pending.timeout_frames;
            pending.last_result_trigger_stamp = None;
            self.pending_devtools_screenshot = Some(pending);
            return request_redraw;
        }

        let trigger_stamp = read_touch_stamp(&self.cfg.screenshot_result_trigger_path);
        let completed = trigger_stamp.is_some()
            && trigger_stamp != pending.last_result_trigger_stamp
            && screenshot_request_completed(
                &self.cfg.screenshot_result_path,
                &pending.request_id_str,
                pending.window_ffi,
            );

        if completed {
            let entry = read_screenshot_result_entry(
                &self.cfg.screenshot_result_path,
                &pending.request_id_str,
                pending.window_ffi,
            );
            let screenshots_dir = entry
                .as_ref()
                .and_then(|e| e.get("screenshots_dir").and_then(|v| v.as_str()))
                .map(|s| s.to_string());
            let bundle_dir_name = pending.bundle_dir_name.clone().unwrap_or_default();
            let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                schema_version: 1,
                status: "completed".to_string(),
                reason: None,
                request_id: pending.request_id_str.clone(),
                window: pending.window_ffi,
                bundle_dir_name,
                screenshots_dir,
                entry,
            })
            .unwrap_or(serde_json::Value::Null);
            self.ws_send_with_request_id("screenshot.result", pending.request_id, payload);
            return false;
        }

        pending.last_result_trigger_stamp = trigger_stamp;
        if pending.remaining_frames == 0 {
            let bundle_dir_name = pending.bundle_dir_name.clone().unwrap_or_default();
            let payload = serde_json::to_value(DevtoolsScreenshotResultV1 {
                schema_version: 1,
                status: "timeout".to_string(),
                reason: Some("timeout_frames_exhausted".to_string()),
                request_id: pending.request_id_str.clone(),
                window: pending.window_ffi,
                bundle_dir_name,
                screenshots_dir: None,
                entry: None,
            })
            .unwrap_or(serde_json::Value::Null);
            self.ws_send_with_request_id("screenshot.result", pending.request_id, payload);
            return false;
        }

        pending.remaining_frames = pending.remaining_frames.saturating_sub(1);
        self.pending_devtools_screenshot = Some(pending);
        request_redraw
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(super) fn ws_send_bundle_dumped_v1(
        &mut self,
        exported_unix_ms: u64,
        dir: &Path,
        bundle: &impl serde::Serialize,
        request_id: Option<u64>,
    ) {
        let embed = self.cfg.devtools_embed_bundle || cfg!(target_arch = "wasm32");
        const DEVTOOLS_BUNDLE_CHUNK_THRESHOLD_BYTES: usize = 512 * 1024;
        const DEVTOOLS_BUNDLE_CHUNK_BYTES: usize = 256 * 1024;

        fn chunk_utf8_string(s: &str, max_bytes: usize) -> Vec<String> {
            let max_bytes = max_bytes.max(1);
            let mut chunks = Vec::<String>::new();
            let mut start = 0usize;
            while start < s.len() {
                let mut end = (start + max_bytes).min(s.len());
                while end > start && !s.is_char_boundary(end) {
                    end -= 1;
                }
                if end == start {
                    // This should be unreachable for valid UTF-8 strings, but avoid infinite loops
                    // in case of unexpected invariants.
                    end = (start + 1).min(s.len());
                    while end < s.len() && !s.is_char_boundary(end) {
                        end += 1;
                    }
                }
                chunks.push(s[start..end].to_string());
                start = end;
            }
            chunks
        }

        if embed {
            // Prefer the existing JSON Value embedding for small bundles. For larger bundles,
            // stream a chunked JSON string to avoid oversized WS messages.
            let bundle_json = serde_json::to_string(bundle).ok();
            if let Some(bundle_json) = bundle_json {
                if bundle_json.len() >= DEVTOOLS_BUNDLE_CHUNK_THRESHOLD_BYTES {
                    let chunks = chunk_utf8_string(&bundle_json, DEVTOOLS_BUNDLE_CHUNK_BYTES);
                    let chunk_count = chunks.len().min(u32::MAX as usize) as u32;
                    for (idx, chunk) in chunks.into_iter().enumerate() {
                        let payload = serde_json::to_value(DevtoolsBundleDumpedV1 {
                            schema_version: 1,
                            exported_unix_ms,
                            out_dir: self.cfg.out_dir.to_string_lossy().to_string(),
                            dir: display_path(&self.cfg.out_dir, dir),
                            bundle: None,
                            bundle_json_chunk: Some(chunk),
                            bundle_json_chunk_index: Some(idx as u32),
                            bundle_json_chunk_count: Some(chunk_count),
                        })
                        .unwrap_or(serde_json::Value::Null);
                        self.ws_send_with_request_id("bundle.dumped", request_id, payload);
                    }
                } else {
                    let bundle_value = serde_json::to_value(bundle).ok();
                    let payload = serde_json::to_value(DevtoolsBundleDumpedV1 {
                        schema_version: 1,
                        exported_unix_ms,
                        out_dir: self.cfg.out_dir.to_string_lossy().to_string(),
                        dir: display_path(&self.cfg.out_dir, dir),
                        bundle: bundle_value,
                        bundle_json_chunk: None,
                        bundle_json_chunk_index: None,
                        bundle_json_chunk_count: None,
                    })
                    .unwrap_or(serde_json::Value::Null);
                    self.ws_send_with_request_id("bundle.dumped", request_id, payload);
                }
            } else {
                let payload = serde_json::to_value(DevtoolsBundleDumpedV1 {
                    schema_version: 1,
                    exported_unix_ms,
                    out_dir: self.cfg.out_dir.to_string_lossy().to_string(),
                    dir: display_path(&self.cfg.out_dir, dir),
                    bundle: None,
                    bundle_json_chunk: None,
                    bundle_json_chunk_index: None,
                    bundle_json_chunk_count: None,
                })
                .unwrap_or(serde_json::Value::Null);
                self.ws_send_with_request_id("bundle.dumped", request_id, payload);
            }
        } else {
            let payload = serde_json::to_value(DevtoolsBundleDumpedV1 {
                schema_version: 1,
                exported_unix_ms,
                out_dir: self.cfg.out_dir.to_string_lossy().to_string(),
                dir: display_path(&self.cfg.out_dir, dir),
                bundle: None,
                bundle_json_chunk: None,
                bundle_json_chunk_index: None,
                bundle_json_chunk_count: None,
            })
            .unwrap_or(serde_json::Value::Null);
            self.ws_send_with_request_id("bundle.dumped", request_id, payload);
        }
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(super) fn ws_send_script_result_v1(&mut self, result: &UiScriptResultV1) {
        let payload = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
        self.ws_send("script.result", payload);
    }

    #[cfg(feature = "diagnostics-ws")]
    pub(super) fn ws_send_pick_result_v1(&mut self, result: &UiPickResultV1) {
        let payload = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
        self.ws_send("pick.result", payload);
    }

    #[cfg(feature = "diagnostics-ws")]
    fn apply_ws_message(&mut self, msg: DiagTransportMessageV1) {
        match msg.r#type.as_str() {
            "environment.sources.get" => {
                let Ok(_req) =
                    serde_json::from_value::<DevtoolsEnvironmentSourcesGetV1>(msg.payload)
                else {
                    return;
                };
                let payload = serde_json::to_value(build_environment_sources_get_ack_v1(
                    self.host_monitor_topology.as_ref(),
                    self.platform_capabilities.as_ref(),
                ))
                .unwrap_or(serde_json::Value::Null);
                self.ws_send_with_request_id(
                    "environment.sources.get_ack",
                    msg.request_id,
                    payload,
                );
            }
            "inspect.set" => {
                let Ok(cfg) = serde_json::from_value::<UiInspectConfigV1>(msg.payload) else {
                    return;
                };
                self.set_inspect_enabled(cfg.enabled, cfg.consume_clicks);
            }
            "pick.arm" => {
                let run_id = self.inspector.next_pick_run_id();
                self.inspector.arm_pick(run_id);
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
                let schema_version: u32 = script_value
                    .get("schema_version")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32;
                if schema_version == 1 && !self.cfg.allow_script_schema_v1 {
                    let run_id = self.next_script_run_id();
                    self.pending_script = None;
                    self.pending_script_run_id = None;
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: None,
                        stage: UiScriptStageV1::Failed,
                        step_index: None,
                        reason_code: Some("script.schema_v1_disabled".to_string()),
                        reason: Some(
                            "script schema_version=1 is disabled; upgrade to schema_version=2"
                                .to_string(),
                        ),
                        evidence: None,
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                        last_bundle_artifact: self.last_dump_artifact_stats.clone(),
                    });
                    return;
                }

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

#[cfg(all(test, feature = "diagnostics-ws"))]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, width: u32, height: u32) -> fret_runtime::RunnerMonitorRectPhysicalV1 {
        fret_runtime::RunnerMonitorRectPhysicalV1 {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn environment_sources_get_ack_publishes_transport_session_monitor_topology() {
        let mut caps = fret_runtime::PlatformCapabilities::default();
        caps.ui.window_tear_off = false;
        caps.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::None;

        let ack = build_environment_sources_get_ack_v1(
            Some(&fret_runtime::RunnerMonitorTopologySnapshotV1 {
                virtual_desktop_bounds_physical: Some(rect(0, 0, 3200, 1080)),
                monitors: vec![
                    fret_runtime::RunnerMonitorInfoV1 {
                        bounds_physical: rect(0, 0, 1920, 1080),
                        scale_factor: 1.0,
                    },
                    fret_runtime::RunnerMonitorInfoV1 {
                        bounds_physical: rect(1920, 0, 1280, 1024),
                        scale_factor: 1.25,
                    },
                ],
            }),
            Some(&caps),
        );

        assert_eq!(ack.sources.len(), 2);
        assert_eq!(
            ack.sources[0].availability,
            EnvironmentSourceAvailabilityV1::PreflightTransportSession
        );
        assert_eq!(
            ack.sources[0].source_id,
            HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1
        );
        assert_eq!(
            ack.host_monitor_topology
                .as_ref()
                .map(|payload| payload.monitor_topology.monitors.len()),
            Some(2)
        );
        assert_eq!(
            ack.sources[1].source_id,
            PLATFORM_CAPABILITIES_ENVIRONMENT_SOURCE_ID_V1
        );
        assert_eq!(
            ack.platform_capabilities
                .as_ref()
                .map(|payload| payload.ui.window_hover_detection.as_str()),
            Some("none")
        );
        assert_eq!(ack.runner_kind.as_deref(), Some("fret-bootstrap"));
    }
}
