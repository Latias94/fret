use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BundleSemanticsModeV1 {
    All,
    Changed,
    Last,
    Off,
}

impl BundleSemanticsModeV1 {
    pub(super) fn from_env() -> Self {
        let v = std::env::var("FRET_DIAG_BUNDLE_SEMANTICS_MODE")
            .ok()
            .map(|v| v.trim().to_ascii_lowercase());
        match v.as_deref() {
            None | Some("") | Some("all") => Self::All,
            Some("changed") | Some("changed_only") | Some("changed-only") => Self::Changed,
            Some("last") | Some("last_only") | Some("last-only") => Self::Last,
            Some("off") | Some("none") => Self::Off,
            Some(_) => Self::All,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiDiagnosticsBundleV1 {
    pub schema_version: u32,
    pub exported_unix_ms: u64,
    pub out_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<UiDiagnosticsEnvFingerprintV1>,
    pub config: UiDiagnosticsBundleConfigV1,
    pub windows: Vec<UiDiagnosticsWindowBundleV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDiagnosticsEnvFingerprintV1 {
    pub schema_version: u32,
    pub runner_kind: String,
    pub target_os: String,
    pub target_family: String,
    pub target_arch: String,
    pub debug_assertions: bool,
    pub diagnostics: UiDiagnosticsEnvDiagnosticsV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scale_factors_seen: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDiagnosticsEnvDiagnosticsV1 {
    pub enabled: bool,
    pub capture_semantics: bool,
    pub redact_text: bool,
    pub screenshots_enabled: bool,
    pub screenshot_on_dump: bool,
    pub max_events: usize,
    pub max_snapshots: usize,
    pub devtools_ws_configured: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiDiagnosticsBundleConfigV1 {
    pub trigger_path: String,
    pub max_events: usize,
    pub max_snapshots: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dump_max_snapshots: Option<usize>,
    pub capture_semantics: bool,
    #[serde(default)]
    pub max_semantics_nodes: usize,
    #[serde(default)]
    pub semantics_test_ids_only: bool,
    pub script_path: String,
    pub script_trigger_path: String,
    pub script_result_path: String,
    pub script_result_trigger_path: String,
    pub script_auto_dump: bool,
    pub pick_trigger_path: String,
    pub pick_result_path: String,
    pub pick_result_trigger_path: String,
    pub pick_auto_dump: bool,
    #[serde(default)]
    pub inspect_path: String,
    #[serde(default)]
    pub inspect_trigger_path: String,
    pub redact_text: bool,
    pub max_debug_string_bytes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_clock_fixed_delta_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiDiagnosticsWindowBundleV1 {
    pub window: u64,
    pub events: Vec<RecordedUiEventV1>,
    pub snapshots: Vec<UiDiagnosticsSnapshotV1>,
}

impl UiDiagnosticsBundleV1 {
    pub(super) fn from_service(
        exported_unix_ms: u64,
        out_dir: &Path,
        svc: &UiDiagnosticsService,
        dump_max_snapshots: usize,
    ) -> Self {
        Self {
            schema_version: 1,
            exported_unix_ms,
            out_dir: sanitize_path_for_bundle(&svc.cfg.out_dir, out_dir),
            env: Some(UiDiagnosticsEnvFingerprintV1::from_service(svc)),
            config: UiDiagnosticsBundleConfigV1 {
                trigger_path: sanitize_path_for_bundle(&svc.cfg.out_dir, &svc.cfg.trigger_path),
                max_events: svc.cfg.max_events,
                max_snapshots: svc.cfg.max_snapshots,
                dump_max_snapshots: (dump_max_snapshots != svc.cfg.max_snapshots)
                    .then_some(dump_max_snapshots),
                capture_semantics: svc.cfg.capture_semantics,
                max_semantics_nodes: svc.cfg.max_semantics_nodes,
                semantics_test_ids_only: svc.cfg.semantics_test_ids_only,
                script_path: sanitize_path_for_bundle(&svc.cfg.out_dir, &svc.cfg.script_path),
                script_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.script_trigger_path,
                ),
                script_result_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.script_result_path,
                ),
                script_result_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.script_result_trigger_path,
                ),
                script_auto_dump: svc.cfg.script_auto_dump,
                pick_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.pick_trigger_path,
                ),
                pick_result_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.pick_result_path,
                ),
                pick_result_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.pick_result_trigger_path,
                ),
                pick_auto_dump: svc.cfg.pick_auto_dump,
                inspect_path: sanitize_path_for_bundle(&svc.cfg.out_dir, &svc.cfg.inspect_path),
                inspect_trigger_path: sanitize_path_for_bundle(
                    &svc.cfg.out_dir,
                    &svc.cfg.inspect_trigger_path,
                ),
                redact_text: svc.cfg.redact_text,
                max_debug_string_bytes: svc.cfg.max_debug_string_bytes,
                frame_clock_fixed_delta_ms: svc.cfg.frame_clock_fixed_delta_ms,
            },
            windows: svc
                .per_window
                .iter()
                .map(|(window, ring)| UiDiagnosticsWindowBundleV1 {
                    window: window.data().as_ffi(),
                    events: ring.events.iter().cloned().collect(),
                    snapshots: take_last_vecdeque(&ring.snapshots, dump_max_snapshots),
                })
                .collect(),
        }
    }

    pub(super) fn apply_semantics_mode_v1(&mut self, mode: BundleSemanticsModeV1) {
        match mode {
            BundleSemanticsModeV1::All => {}
            BundleSemanticsModeV1::Off => {
                for w in &mut self.windows {
                    for s in &mut w.snapshots {
                        s.debug.semantics = None;
                    }
                }
            }
            BundleSemanticsModeV1::Last => {
                for w in &mut self.windows {
                    let mut keep_idx: Option<usize> = None;
                    for (idx, s) in w.snapshots.iter().enumerate() {
                        if s.debug.semantics.is_some() {
                            keep_idx = Some(idx);
                        }
                    }
                    for (idx, s) in w.snapshots.iter_mut().enumerate() {
                        if Some(idx) != keep_idx {
                            s.debug.semantics = None;
                        }
                    }
                }
            }
            BundleSemanticsModeV1::Changed => {
                for w in &mut self.windows {
                    let mut last_kept_fingerprint: Option<u64> = None;
                    for (idx, s) in w.snapshots.iter_mut().enumerate() {
                        if s.debug.semantics.is_none() {
                            continue;
                        }
                        let is_last = idx + 1 == w.snapshots.len();
                        if is_last {
                            last_kept_fingerprint = s.semantics_fingerprint;
                            continue;
                        }
                        let keep = match (last_kept_fingerprint, s.semantics_fingerprint) {
                            (None, _) => true,
                            (_, None) => true,
                            (Some(a), Some(b)) => a != b,
                        };
                        if keep {
                            last_kept_fingerprint = s.semantics_fingerprint;
                        } else {
                            s.debug.semantics = None;
                        }
                    }
                }
            }
        }
    }
}

impl UiDiagnosticsEnvFingerprintV1 {
    fn from_service(svc: &UiDiagnosticsService) -> Self {
        let runner_kind = if cfg!(target_arch = "wasm32") {
            "web".to_string()
        } else {
            "native".to_string()
        };

        let mut capabilities: Vec<String> = vec!["diag.script_v2".to_string()];
        if svc.cfg.screenshots_enabled {
            capabilities.push("diag.screenshot_png".to_string());
        }
        capabilities.push("diag.inject_ime".to_string());
        capabilities.push("diag.text_ime_trace".to_string());
        capabilities.push("diag.text_input_snapshot".to_string());
        capabilities.push("diag.shortcut_routing_trace".to_string());
        capabilities.push("diag.overlay_placement_trace".to_string());
        capabilities.sort();
        capabilities.dedup();

        let mut scale_factors_seen: Vec<f32> = Vec::new();
        for (_window, ring) in svc.per_window.iter() {
            if let Some(last) = ring.snapshots.back() {
                let sf = last.scale_factor;
                if !scale_factors_seen
                    .iter()
                    .any(|v| (*v - sf).abs() < f32::EPSILON)
                {
                    scale_factors_seen.push(sf);
                }
            }
        }
        scale_factors_seen.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        Self {
            schema_version: 1,
            runner_kind,
            target_os: std::env::consts::OS.to_string(),
            target_family: std::env::consts::FAMILY.to_string(),
            target_arch: std::env::consts::ARCH.to_string(),
            debug_assertions: cfg!(debug_assertions),
            diagnostics: UiDiagnosticsEnvDiagnosticsV1 {
                enabled: svc.cfg.enabled,
                capture_semantics: svc.cfg.capture_semantics,
                redact_text: svc.cfg.redact_text,
                screenshots_enabled: svc.cfg.screenshots_enabled,
                screenshot_on_dump: svc.cfg.screenshot_on_dump,
                max_events: svc.cfg.max_events,
                max_snapshots: svc.cfg.max_snapshots,
                devtools_ws_configured: svc.ws_is_configured(),
            },
            capabilities,
            scale_factors_seen,
        }
    }
}
