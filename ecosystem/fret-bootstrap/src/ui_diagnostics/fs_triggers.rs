use std::path::{Path, PathBuf};

use fret_diag_protocol::{
    EnvironmentSourceAvailabilityV1, FILESYSTEM_ENVIRONMENT_SOURCES_FILE_NAME_V1,
    FILESYSTEM_HOST_MONITOR_TOPOLOGY_ENVIRONMENT_PAYLOAD_FILE_NAME_V1, FilesystemCapabilitiesV1,
    FilesystemEnvironmentSourceV1, FilesystemEnvironmentSourcesV1,
    HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1, HostMonitorTopologyEnvironmentPayloadV1,
    UiActionScriptV1, UiActionScriptV2, UiInspectConfigV1, UiScriptEventLogEntryV1,
    UiScriptResultV1, UiScriptStageV1,
};

use super::{
    PendingScript, UiDiagnosticsService, bundle::ui_diagnostics_monitor_topology_from_runner,
    display_path, format_bundle_dump_note, push_script_event_log, read_touch_stamp, sanitize_label,
    touch_file, unix_ms_now,
};

#[derive(Debug, Clone, serde::Deserialize)]
struct DumpRequestV1 {
    schema_version: u32,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    max_snapshots: Option<u32>,
    #[serde(default)]
    request_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub(super) struct PendingForceDumpRequest {
    pub(super) label: String,
    pub(super) dump_max_snapshots: Option<usize>,
    pub(super) script_run_id: Option<u64>,
    pub(super) script_step_index: Option<u32>,
    pub(super) request_id: Option<u64>,
}

pub(super) fn best_effort_update_script_result_last_bundle_artifact(
    service: &mut UiDiagnosticsService,
    script_run_id: u64,
    dumped_dir: &Path,
) {
    let Ok(bytes) = std::fs::read(&service.cfg.script_result_path) else {
        return;
    };
    let Ok(mut result) = serde_json::from_slice::<UiScriptResultV1>(&bytes) else {
        return;
    };
    if result.run_id != script_run_id {
        return;
    }

    let bundle_dir = display_path(&service.cfg.out_dir, dumped_dir);
    let artifact = service.last_dump_artifact_stats.clone();
    let should_write = result.last_bundle_dir.as_deref() != Some(bundle_dir.as_str())
        || result.last_bundle_artifact.is_none();
    if !should_write {
        return;
    }

    result.updated_unix_ms = unix_ms_now();
    result.last_bundle_dir = Some(bundle_dir);
    result.last_bundle_artifact = artifact;
    service.write_script_result(result.clone());

    if !cfg!(target_arch = "wasm32") {
        let bundle_script_result_path = dumped_dir.join("script.result.json");
        if let Ok(serialized) = serde_json::to_vec_pretty(&result) {
            let _ = std::fs::write(&bundle_script_result_path, serialized);
            let _ = touch_file(&service.cfg.script_result_trigger_path);
        }
    }
}

fn warn_fs_once(
    warned: &mut bool,
    out_dir: &Path,
    message: &str,
    path: &Path,
    err: &dyn std::fmt::Display,
) {
    if *warned {
        return;
    }
    *warned = true;
    tracing::warn!(
        target: "fret",
        out_dir = ?out_dir,
        path = %path.display(),
        error = %err,
        "{message}"
    );
}

impl UiDiagnosticsService {
    pub(super) fn request_force_dump(
        &mut self,
        label: String,
        dump_max_snapshots: Option<usize>,
        script_run_id: Option<u64>,
        script_step_index: Option<u32>,
        request_id: Option<u64>,
    ) {
        self.pending_force_dump = Some(PendingForceDumpRequest {
            label: sanitize_label(&label),
            dump_max_snapshots,
            script_run_id,
            script_step_index,
            request_id,
        });
    }

    pub fn maybe_dump_if_triggered(&mut self) -> Option<PathBuf> {
        if !self.is_enabled() {
            return None;
        }

        self.poll_ws_inbox();

        if let Some(pending) = self.pending_force_dump.take() {
            let dumped = self.dump_bundle_with_options(
                Some(&pending.label),
                pending.dump_max_snapshots,
                pending.request_id,
            );
            if let (Some(script_run_id), Some(dir)) = (pending.script_run_id, dumped.as_ref()) {
                let bundle_dir = display_path(&self.cfg.out_dir, dir);
                for active in self
                    .active_scripts
                    .values_mut()
                    .filter(|active| active.run_id == script_run_id)
                {
                    push_script_event_log(
                        active,
                        &self.cfg,
                        UiScriptEventLogEntryV1 {
                            unix_ms: unix_ms_now(),
                            kind: "bundle_dumped".to_string(),
                            step_index: pending.script_step_index,
                            note: Some(format_bundle_dump_note(
                                &pending.label,
                                pending.dump_max_snapshots,
                                pending.request_id,
                            )),
                            bundle_dir: Some(bundle_dir.clone()),
                            window: None,
                            tick_id: None,
                            frame_id: None,
                            window_snapshot_seq: None,
                        },
                    );
                }
                best_effort_update_script_result_last_bundle_artifact(self, script_run_id, dir);
            }
            return dumped;
        }

        if self.is_wasm_ws_only() {
            return None;
        }

        let Some(stamp) = read_touch_stamp(&self.cfg.trigger_path) else {
            if let Some(dir) = self.cfg.trigger_path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            // Prime the trigger file with a baseline stamp so external drivers can reliably
            // advance it (Windows mtime resolution is not always sufficient for edge detection).
            let _ = std::fs::write(&self.cfg.trigger_path, b"0\n");
            self.last_trigger_stamp = Some(0);
            return None;
        };

        // Treat the first observed value as a baseline, not a trigger (avoids dumping stale runs
        // when the diagnostics directory is reused between launches).
        let Some(prev) = self.last_trigger_stamp else {
            self.last_trigger_stamp = Some(stamp);
            return None;
        };
        if prev == stamp {
            return None;
        }
        self.last_trigger_stamp = Some(stamp);

        let request_path = self.cfg.out_dir.join("dump.request.json");
        let mut label: Option<String> = None;
        let mut max_snapshots: Option<usize> = None;
        let mut request_id: Option<u64> = None;
        if let Ok(bytes) = std::fs::read(&request_path) {
            let parsed = serde_json::from_slice::<DumpRequestV1>(&bytes).ok();
            // Best-effort: consume the request once per trigger so stale metadata doesn't leak
            // into subsequent dumps.
            let _ = std::fs::remove_file(&request_path);
            if let Some(parsed) = parsed
                && parsed.schema_version == 1
            {
                label = parsed
                    .label
                    .as_deref()
                    .map(sanitize_label)
                    .filter(|s| !s.is_empty());
                max_snapshots = parsed.max_snapshots.map(|v| v as usize);
                request_id = parsed.request_id;
            }
        }

        if label.is_some() || max_snapshots.is_some() || request_id.is_some() {
            self.dump_bundle_with_options(label.as_deref(), max_snapshots, request_id)
        } else {
            self.dump_bundle(None)
        }
    }

    pub fn poll_exit_trigger(&mut self) -> bool {
        if !self.is_enabled() {
            return false;
        }

        if let Some(deadline) = self.ws_exit_deadline_unix_ms
            && unix_ms_now() >= deadline
        {
            self.ws_exit_deadline_unix_ms = None;
            return true;
        }

        let current_mtime = std::fs::metadata(&self.cfg.exit_path)
            .and_then(|m| m.modified())
            .ok();

        if !self.exit_armed {
            self.exit_last_mtime = current_mtime;
            self.exit_armed = true;
            return false;
        }

        let Some(current_mtime) = current_mtime else {
            return false;
        };

        let triggered = match self.exit_last_mtime {
            Some(prev) => current_mtime > prev,
            None => true,
        };
        self.exit_last_mtime = Some(current_mtime);
        triggered
    }

    pub(super) fn ensure_ready_file(&mut self) {
        if self.ready_written {
            return;
        }
        if !self.cfg.enabled {
            return;
        }

        if self.is_wasm_ws_only() {
            // Web runners do not have a stable filesystem surface for the legacy `ready.touch` file.
            self.ready_written = true;
            return;
        }

        if let Some(parent) = self.cfg.ready_path.parent()
            && let Err(err) = std::fs::create_dir_all(parent)
        {
            warn_fs_once(
                &mut self.ready_write_warned,
                &self.cfg.out_dir,
                "ui diagnostics: failed to create ready.touch parent dir",
                parent,
                &err,
            );
            return;
        }

        self.ensure_capabilities_file();
        self.refresh_environment_source_files();

        let ts = unix_ms_now();
        let mut f = match std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.cfg.ready_path)
        {
            Ok(f) => f,
            Err(err) => {
                warn_fs_once(
                    &mut self.ready_write_warned,
                    &self.cfg.out_dir,
                    "ui diagnostics: failed to open ready.touch",
                    &self.cfg.ready_path,
                    &err,
                );
                return;
            }
        };

        use std::io::Write as _;
        if let Err(err) = writeln!(f, "{ts}") {
            warn_fs_once(
                &mut self.ready_write_warned,
                &self.cfg.out_dir,
                "ui diagnostics: failed to write ready.touch",
                &self.cfg.ready_path,
                &err,
            );
            return;
        }
        if let Err(err) = f.flush() {
            warn_fs_once(
                &mut self.ready_write_warned,
                &self.cfg.out_dir,
                "ui diagnostics: failed to flush ready.touch",
                &self.cfg.ready_path,
                &err,
            );
            return;
        }

        self.ready_written = true;
    }

    pub(super) fn refresh_environment_source_files(&mut self) {
        if !self.cfg.enabled || self.is_wasm_ws_only() {
            return;
        }

        let current_topology = self.host_monitor_topology.clone();
        if self.environment_sources_catalog_written
            && self.published_host_monitor_topology == current_topology
        {
            return;
        }

        let catalog_path = self
            .cfg
            .out_dir
            .join(FILESYSTEM_ENVIRONMENT_SOURCES_FILE_NAME_V1);
        if let Some(parent) = catalog_path.parent()
            && let Err(err) = std::fs::create_dir_all(parent)
        {
            warn_fs_once(
                &mut self.environment_sources_write_warned,
                &self.cfg.out_dir,
                "ui diagnostics: failed to create environment.sources.json parent dir",
                parent,
                &err,
            );
            return;
        }

        let sources = current_topology
            .as_ref()
            .map(|_| {
                vec![FilesystemEnvironmentSourceV1 {
                    source_id: HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1.to_string(),
                    availability: EnvironmentSourceAvailabilityV1::LaunchTime,
                }]
            })
            .unwrap_or_default();
        let catalog_payload = FilesystemEnvironmentSourcesV1 {
            schema_version: 1,
            sources,
            runner_kind: Some("fret-bootstrap".to_string()),
            runner_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };

        if let Ok(mut text) = serde_json::to_string_pretty(&catalog_payload) {
            text.push('\n');
            if let Err(err) = std::fs::write(&catalog_path, text) {
                warn_fs_once(
                    &mut self.environment_sources_write_warned,
                    &self.cfg.out_dir,
                    "ui diagnostics: failed to write environment.sources.json",
                    &catalog_path,
                    &err,
                );
                return;
            }
        }

        let payload_path = self
            .cfg
            .out_dir
            .join(FILESYSTEM_HOST_MONITOR_TOPOLOGY_ENVIRONMENT_PAYLOAD_FILE_NAME_V1);
        match current_topology.as_ref() {
            Some(snapshot) => {
                let payload = HostMonitorTopologyEnvironmentPayloadV1 {
                    schema_version: 1,
                    source_id: HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1.to_string(),
                    monitor_topology: ui_diagnostics_monitor_topology_from_runner(snapshot),
                };
                if let Ok(mut text) = serde_json::to_string_pretty(&payload) {
                    text.push('\n');
                    if let Err(err) = std::fs::write(&payload_path, text) {
                        warn_fs_once(
                            &mut self.environment_sources_write_warned,
                            &self.cfg.out_dir,
                            "ui diagnostics: failed to write host monitor topology environment payload",
                            &payload_path,
                            &err,
                        );
                        return;
                    }
                }
            }
            None => {
                if payload_path.is_file()
                    && let Err(err) = std::fs::remove_file(&payload_path)
                {
                    warn_fs_once(
                        &mut self.environment_sources_write_warned,
                        &self.cfg.out_dir,
                        "ui diagnostics: failed to remove stale host monitor topology environment payload",
                        &payload_path,
                        &err,
                    );
                    return;
                }
            }
        }

        self.environment_sources_catalog_written = true;
        self.published_host_monitor_topology = current_topology;
    }

    pub(super) fn ensure_capabilities_file(&mut self) {
        if self.capabilities_written {
            return;
        }
        if !self.cfg.enabled {
            return;
        }
        if self.is_wasm_ws_only() {
            self.capabilities_written = true;
            return;
        }

        let mut caps = vec!["diag.script_v2".to_string()];
        if self.cfg.screenshots_enabled {
            caps.push("diag.screenshot_png".to_string());
        }
        caps.push("diag.pointer_kind_touch".to_string());
        caps.push("diag.pointer_kind_pen".to_string());
        caps.push("diag.gesture_tap".to_string());
        caps.push("diag.gesture_long_press".to_string());
        caps.push("diag.gesture_swipe".to_string());
        caps.push("diag.gesture_pinch".to_string());
        caps.push("diag.inject_ime".to_string());
        if !cfg!(target_arch = "wasm32") {
            caps.push("diag.multi_window".to_string());
        }
        caps.push("diag.text_ime_trace".to_string());
        caps.push("diag.text_input_snapshot".to_string());
        caps.push("diag.shortcut_routing_trace".to_string());
        caps.push("diag.command_dispatch_trace".to_string());
        caps.push("diag.overlay_placement_trace".to_string());
        if !cfg!(target_arch = "wasm32") {
            caps.push("diag.window_style_snapshot".to_string());
            caps.push("diag.window_background_material_snapshot".to_string());
            if cfg!(any(target_os = "windows", target_os = "macos")) {
                caps.push("diag.window_style_patch_v1".to_string());
                caps.push("diag.platform_window_receiver_at_cursor_v1".to_string());
            }
        }
        caps.push("diag.window_insets_override".to_string());
        caps.push("diag.clipboard_force_unavailable".to_string());
        if !cfg!(target_arch = "wasm32") {
            caps.push("diag.clipboard_text".to_string());
        }
        caps.push("diag.incoming_open_inject".to_string());
        if cfg!(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux"
        )) {
            caps.push("diag.cursor_screen_pos_override".to_string());
            caps.push("diag.mouse_buttons_override".to_string());
            caps.push("diag.wheel_burst_inject".to_string());
        }

        let path = self.cfg.out_dir.join("capabilities.json");
        if let Some(parent) = path.parent()
            && let Err(err) = std::fs::create_dir_all(parent)
        {
            warn_fs_once(
                &mut self.capabilities_write_warned,
                &self.cfg.out_dir,
                "ui diagnostics: failed to create capabilities.json parent dir",
                parent,
                &err,
            );
            return;
        }

        let payload = FilesystemCapabilitiesV1 {
            schema_version: 1,
            capabilities: caps,
            runner_kind: Some("fret-bootstrap".to_string()),
            runner_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            hints: Some(fret_diag_protocol::FilesystemCapabilitiesHintsV1 {
                allow_script_schema_v1: Some(self.cfg.allow_script_schema_v1),
                write_bundle_schema2: Some(self.cfg.write_bundle_schema2),
            }),
        };
        if let Ok(mut text) = serde_json::to_string_pretty(&payload) {
            text.push('\n');
            if let Err(err) = std::fs::write(&path, text) {
                warn_fs_once(
                    &mut self.capabilities_write_warned,
                    &self.cfg.out_dir,
                    "ui diagnostics: failed to write capabilities.json",
                    &path,
                    &err,
                );
                return;
            }
        }

        self.capabilities_written = true;
    }

    pub(super) fn poll_script_trigger(&mut self) {
        if self.poll_ws_inbox_and_is_wasm_ws_only() {
            return;
        }

        let Some(stamp) = read_touch_stamp(&self.cfg.script_trigger_path) else {
            if let Some(dir) = self.cfg.script_trigger_path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            // Prime the trigger file with a baseline stamp so external drivers can reliably
            // advance it (Windows mtime resolution is not always sufficient for edge detection).
            let _ = std::fs::write(&self.cfg.script_trigger_path, b"0\n");
            self.last_script_trigger_stamp = Some(0);
            return;
        };

        // Treat the first observed value as a baseline, not a trigger (avoids re-running stale scripts
        // when the diagnostics directory is reused between runs).
        let Some(prev) = self.last_script_trigger_stamp else {
            self.last_script_trigger_stamp = Some(stamp);
            return;
        };
        if prev == stamp {
            return;
        }
        self.last_script_trigger_stamp = Some(stamp);

        let run_id = self.next_script_run_id();
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

        let bytes = match std::fs::read(&self.cfg.script_path) {
            Ok(bytes) => bytes,
            Err(_) => {
                self.pending_script_run_id = None;
                self.write_script_result(UiScriptResultV1 {
                    schema_version: 1,
                    run_id,
                    updated_unix_ms: unix_ms_now(),
                    window: None,
                    stage: UiScriptStageV1::Failed,
                    step_index: None,
                    reason_code: Some("script.read_failed".to_string()),
                    reason: Some("failed to read script.json".to_string()),
                    evidence: None,
                    last_bundle_dir: self
                        .last_dump_dir
                        .as_ref()
                        .map(|p| display_path(&self.cfg.out_dir, p)),
                    last_bundle_artifact: self.last_dump_artifact_stats.clone(),
                });
                return;
            }
        };
        let schema_version: u32 = serde_json::from_slice::<serde_json::Value>(&bytes)
            .ok()
            .and_then(|v| v.get("schema_version").and_then(|v| v.as_u64()))
            .unwrap_or(0)
            .min(u32::MAX as u64) as u32;

        let script = match schema_version {
            1 => {
                if !self.cfg.allow_script_schema_v1 {
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
                let script = match serde_json::from_slice::<UiActionScriptV1>(&bytes) {
                    Ok(script) => script,
                    Err(err) => {
                        self.pending_script_run_id = None;
                        self.write_script_result(UiScriptResultV1 {
                            schema_version: 1,
                            run_id,
                            updated_unix_ms: unix_ms_now(),
                            window: None,
                            stage: UiScriptStageV1::Failed,
                            step_index: None,
                            reason_code: Some("script.parse_failed".to_string()),
                            reason: Some(format!("failed to parse script as schema v1: {err}")),
                            evidence: None,
                            last_bundle_dir: self
                                .last_dump_dir
                                .as_ref()
                                .map(|p| display_path(&self.cfg.out_dir, p)),
                            last_bundle_artifact: self.last_dump_artifact_stats.clone(),
                        });
                        return;
                    }
                };
                let Some(script) = PendingScript::from_v1(script) else {
                    self.pending_script_run_id = None;
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: None,
                        stage: UiScriptStageV1::Failed,
                        step_index: None,
                        reason_code: Some("script.invalid".to_string()),
                        reason: Some("invalid schema v1 script".to_string()),
                        evidence: None,
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                        last_bundle_artifact: self.last_dump_artifact_stats.clone(),
                    });
                    return;
                };
                script
            }
            2 => {
                let script = match serde_json::from_slice::<UiActionScriptV2>(&bytes) {
                    Ok(script) => script,
                    Err(err) => {
                        self.pending_script_run_id = None;
                        self.write_script_result(UiScriptResultV1 {
                            schema_version: 1,
                            run_id,
                            updated_unix_ms: unix_ms_now(),
                            window: None,
                            stage: UiScriptStageV1::Failed,
                            step_index: None,
                            reason_code: Some("script.parse_failed".to_string()),
                            reason: Some(format!("failed to parse script as schema v2: {err}")),
                            evidence: None,
                            last_bundle_dir: self
                                .last_dump_dir
                                .as_ref()
                                .map(|p| display_path(&self.cfg.out_dir, p)),
                            last_bundle_artifact: self.last_dump_artifact_stats.clone(),
                        });
                        return;
                    }
                };
                let Some(script) = PendingScript::from_v2(script) else {
                    self.pending_script_run_id = None;
                    self.write_script_result(UiScriptResultV1 {
                        schema_version: 1,
                        run_id,
                        updated_unix_ms: unix_ms_now(),
                        window: None,
                        stage: UiScriptStageV1::Failed,
                        step_index: None,
                        reason_code: Some("script.invalid".to_string()),
                        reason: Some("invalid schema v2 script".to_string()),
                        evidence: None,
                        last_bundle_dir: self
                            .last_dump_dir
                            .as_ref()
                            .map(|p| display_path(&self.cfg.out_dir, p)),
                        last_bundle_artifact: self.last_dump_artifact_stats.clone(),
                    });
                    return;
                };
                script
            }
            _ => {
                self.pending_script_run_id = None;
                self.write_script_result(UiScriptResultV1 {
                    schema_version: 1,
                    run_id,
                    updated_unix_ms: unix_ms_now(),
                    window: None,
                    stage: UiScriptStageV1::Failed,
                    step_index: None,
                    reason_code: Some("script.schema_unsupported".to_string()),
                    reason: Some(format!(
                        "unsupported script schema_version={schema_version}"
                    )),
                    evidence: None,
                    last_bundle_dir: self
                        .last_dump_dir
                        .as_ref()
                        .map(|p| display_path(&self.cfg.out_dir, p)),
                    last_bundle_artifact: self.last_dump_artifact_stats.clone(),
                });
                return;
            }
        };

        self.pending_script = Some(script);
        self.pending_script_run_id = Some(run_id);
    }

    pub(super) fn poll_pick_trigger(&mut self) {
        if self.poll_ws_inbox_and_is_wasm_ws_only() {
            return;
        }

        let modified =
            match std::fs::metadata(&self.cfg.pick_trigger_path).and_then(|m| m.modified()) {
                Ok(modified) => modified,
                Err(_) => {
                    if let Some(dir) = self.cfg.pick_trigger_path.parent() {
                        let _ = std::fs::create_dir_all(dir);
                    }
                    if std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&self.cfg.pick_trigger_path)
                        .is_ok()
                        && let Ok(modified) = std::fs::metadata(&self.cfg.pick_trigger_path)
                            .and_then(|m| m.modified())
                    {
                        self.last_pick_trigger_mtime = Some(modified);
                    }
                    return;
                }
            };
        if self.last_pick_trigger_mtime.is_none() {
            self.last_pick_trigger_mtime = Some(modified);
            return;
        }
        if self
            .last_pick_trigger_mtime
            .is_some_and(|prev| prev >= modified)
        {
            return;
        }
        self.last_pick_trigger_mtime = Some(modified);

        let run_id = self.inspector.next_pick_run_id();
        self.inspector.arm_pick(run_id);
    }

    pub(super) fn poll_inspect_trigger(&mut self) {
        if self.poll_ws_inbox_and_is_wasm_ws_only() {
            return;
        }

        let modified =
            match std::fs::metadata(&self.cfg.inspect_trigger_path).and_then(|m| m.modified()) {
                Ok(modified) => modified,
                Err(_) => {
                    if let Some(dir) = self.cfg.inspect_trigger_path.parent() {
                        let _ = std::fs::create_dir_all(dir);
                    }
                    if std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&self.cfg.inspect_trigger_path)
                        .is_ok()
                        && let Ok(modified) = std::fs::metadata(&self.cfg.inspect_trigger_path)
                            .and_then(|m| m.modified())
                    {
                        self.last_inspect_trigger_mtime = Some(modified);
                    }
                    return;
                }
            };
        if self.last_inspect_trigger_mtime.is_none() {
            self.last_inspect_trigger_mtime = Some(modified);
            return;
        }
        if self
            .last_inspect_trigger_mtime
            .is_some_and(|prev| prev >= modified)
        {
            return;
        }
        self.last_inspect_trigger_mtime = Some(modified);

        let bytes = std::fs::read(&self.cfg.inspect_path).ok();
        let Some(bytes) = bytes else {
            return;
        };
        let cfg: UiInspectConfigV1 = match serde_json::from_slice(&bytes) {
            Ok(cfg) => cfg,
            Err(_) => return,
        };
        if cfg.schema_version != 1 {
            return;
        }

        self.set_inspect_enabled(cfg.enabled, cfg.consume_clicks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_diag_protocol::{
        FilesystemEnvironmentSourcesV1, HostMonitorTopologyEnvironmentPayloadV1,
    };
    use fret_runtime::{
        RunnerMonitorInfoV1, RunnerMonitorRectPhysicalV1, RunnerMonitorTopologySnapshotV1,
    };

    fn rect(x: i32, y: i32, width: u32, height: u32) -> RunnerMonitorRectPhysicalV1 {
        RunnerMonitorRectPhysicalV1 {
            x,
            y,
            width,
            height,
        }
    }

    fn sample_topology(scale_factor: f32) -> RunnerMonitorTopologySnapshotV1 {
        RunnerMonitorTopologySnapshotV1 {
            virtual_desktop_bounds_physical: Some(rect(0, 0, 3200, 1080)),
            monitors: vec![
                RunnerMonitorInfoV1 {
                    bounds_physical: rect(0, 0, 1920, 1080),
                    scale_factor: 1.0,
                },
                RunnerMonitorInfoV1 {
                    bounds_physical: rect(1920, 0, 1280, 1024),
                    scale_factor,
                },
            ],
        }
    }

    #[test]
    fn refresh_environment_source_files_publishes_launch_time_monitor_topology_sidecars() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-environment-source-sidecars-{}-{}",
            unix_ms_now(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);

        let mut service = UiDiagnosticsService::default();
        service.cfg.enabled = true;
        service.cfg.out_dir = root.clone();
        service.host_monitor_topology = Some(sample_topology(1.25));

        service.refresh_environment_source_files();

        let catalog_path = root.join(FILESYSTEM_ENVIRONMENT_SOURCES_FILE_NAME_V1);
        let payload_path =
            root.join(FILESYSTEM_HOST_MONITOR_TOPOLOGY_ENVIRONMENT_PAYLOAD_FILE_NAME_V1);
        assert!(catalog_path.is_file());
        assert!(payload_path.is_file());

        let catalog = serde_json::from_slice::<FilesystemEnvironmentSourcesV1>(
            &std::fs::read(&catalog_path).unwrap(),
        )
        .unwrap();
        assert_eq!(catalog.sources.len(), 1);
        assert_eq!(
            catalog.sources[0].source_id,
            HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1
        );
        assert_eq!(
            catalog.sources[0].availability,
            EnvironmentSourceAvailabilityV1::LaunchTime
        );

        let payload = serde_json::from_slice::<HostMonitorTopologyEnvironmentPayloadV1>(
            &std::fs::read(&payload_path).unwrap(),
        )
        .unwrap();
        assert_eq!(
            payload.source_id,
            HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1
        );
        assert_eq!(payload.monitor_topology.monitors.len(), 2);
        assert_eq!(payload.monitor_topology.monitors[1].scale_factor, 1.25);

        service.host_monitor_topology = None;
        service.refresh_environment_source_files();

        let catalog = serde_json::from_slice::<FilesystemEnvironmentSourcesV1>(
            &std::fs::read(&catalog_path).unwrap(),
        )
        .unwrap();
        assert!(catalog.sources.is_empty());
        assert!(!payload_path.exists());

        let _ = std::fs::remove_dir_all(&root);
    }
}
