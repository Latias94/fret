//! Diagnostics tooling for the Fret workspace.
//!
//! This crate is primarily used by `fretboard` to:
//! - run scripted UI interactions,
//! - capture diagnostics bundles (JSON + optional screenshots),
//! - compare runs and enforce performance/behavior gates.
//!
//! This is a tooling-focused crate (not a runtime dependency for apps).

#![cfg(not(target_arch = "wasm32"))]
#![recursion_limit = "512"]

use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use fret_diag_protocol::{
    DevtoolsBundleDumpedV1, DevtoolsSessionListV1, DevtoolsSessionRemovedV1, UiArtifactStatsV1,
    UiCapabilitiesCheckV1, UiScriptEventLogEntryV1, UiScriptEvidenceV1, UiScriptResultV1,
    UiScriptStageV1,
};

pub mod api;
mod artifact_alias;
mod artifact_lint;
mod artifact_store;
pub mod artifacts;
mod bundle_index;
mod cli;
mod commands;
mod compare;
mod compat;
pub mod devtools;
mod diag_campaign;
mod diag_compare;
mod diag_dashboard;
mod diag_list;
mod diag_matrix;
mod diag_perf;
mod diag_perf_baseline;
mod diag_policy;
mod diag_repeat;
mod diag_repro;
mod diag_run;
mod diag_sessions;
mod diag_stats;
mod diag_suite;
mod diag_suite_scripts;
mod diag_summarize;
mod evidence_index;
mod frames_index;
mod gates;
mod hotspots_lite;
mod json_bundle;
mod json_stream;
mod latest;
mod launch_env_policy;
mod layout_perf_summary;
mod lint;
#[cfg(target_os = "macos")]
mod macos_footprint_tool;
#[cfg(target_os = "macos")]
mod macos_vmmap;
mod math;
mod pack_zip;
mod paths;
mod perf_hint_gate;
mod perf_seed_policy;
mod post_run_checks;
mod promoted_registry_builder;
mod registry;
pub mod regression_summary;
mod run_artifacts;
mod script_execution;
mod script_registry;
mod script_tooling;
mod session;
mod shrink;
mod stats;
mod suite_summary;
mod test_id_bloom;
mod tooling_failures;
mod tooling_warnings;
mod trace;
pub mod transport;
mod triage_json;
mod util;

pub(crate) use post_run_checks::apply_post_run_checks;

pub(crate) use evidence_index::write_evidence_index;
pub(crate) use pack_zip::{
    ReproZipBundle, pack_ai_packet_dir_to_zip, pack_bundle_dir_to_zip, pack_repro_ai_zip_multi,
    pack_repro_zip_multi, repro_zip_prefix_for_script, zip_safe_component,
};
pub(crate) use perf_hint_gate::{
    parse_perf_hint_gate_options, perf_hint_gate_failures_for_triage_json,
};

pub(crate) use paths::{
    default_lint_out_path, default_meta_out_path, default_pack_out_path, default_test_ids_out_path,
    default_triage_out_path, expand_script_inputs, prefer_schema2_sibling_for_bundle_json_path,
    resolve_bundle_artifact_path, resolve_bundle_artifact_path_no_materialize,
    resolve_bundle_root_dir, resolve_bundle_schema2_artifact_path_no_materialize, resolve_path,
    resolve_raw_bundle_artifact_path_no_materialize, wait_for_bundle_artifact_from_script_result,
    wait_for_bundle_artifact_in_dir,
};

use artifact_store::{RunArtifactStore, run_id_artifact_dir};
use compare::{
    CompareOptions, CompareReport, PerfThresholdAggregate, PerfThresholds, RenderdocDumpAttempt,
    apply_perf_baseline_floor, apply_perf_baseline_headroom, cargo_run_inject_feature,
    compare_bundles, ensure_env_var, find_latest_export_dir, maybe_launch_demo,
    maybe_launch_demo_without_diagnostics, normalize_repo_relative_path, read_latest_pointer,
    read_perf_baseline_file, resolve_threshold, run_fret_renderdoc_dump,
    scan_perf_threshold_failures, stop_launched_demo, wait_for_files_with_extensions,
};
use devtools::DevtoolsOps;
use gates::{
    CodeEditorMemoryGateResult, CodeEditorMemoryThresholds, LinearBytesVsImagesGateResult,
    LinearBytesVsImagesThreshold, RedrawHitchesGateResult, RenderTextAtlasBytesGateResult,
    RenderTextFontDbGateResult, RenderTextFontDbThresholds, RendererGpuBudgetThresholds,
    RendererGpuBudgetsGateResult, ResourceFootprintGateResult, ResourceFootprintThresholds,
    WgpuHubCountsGateResult, WgpuHubCountsThresholds, WgpuMetalAllocatedSizeGateResult,
    check_code_editor_memory_thresholds,
    check_macos_owned_unmapped_memory_dirty_bytes_linear_vs_renderer_gpu_images,
    check_redraw_hitches_max_total_ms, check_render_text_atlas_bytes_live_estimate_total_threshold,
    check_render_text_font_db_thresholds, check_renderer_gpu_budget_thresholds,
    check_resource_footprint_thresholds, check_wgpu_hub_counts_thresholds,
    check_wgpu_metal_current_allocated_size_bytes_linear_vs_renderer_gpu_images,
    check_wgpu_metal_current_allocated_size_threshold,
};
use lint::{LintOptions, lint_bundle_from_path};
use perf_seed_policy::{PerfBaselineSeed, PerfSeedMetric, ResolvedPerfBaselineSeedPolicy};

use stats::{
    BundleStatsOptions, BundleStatsReport, BundleStatsSort, ScriptResultSummary,
    bundle_stats_diff_from_paths, bundle_stats_from_path,
    check_report_for_hover_layout_invalidations, clear_script_result_files, report_result_and_exit,
    run_script_and_wait, wait_for_failure_dump_bundle,
};
use tooling_failures::{
    mark_existing_script_result_tooling_failure, push_tooling_event_log_entry,
    write_tooling_failure_script_result, write_tooling_failure_script_result_if_missing,
};
use util::{advance_target_run_id, now_unix_ms, read_json_value, touch, write_json_value};

pub use diag_dashboard::{
    DashboardCountEntry, DashboardFailingSummaryEntry, DashboardReasonCodeEntry,
    DashboardSummaryProjection, dashboard_counter_entries, dashboard_failing_summary_entries,
    dashboard_human_lines_from_projection, dashboard_reason_code_entries,
    project_dashboard_summary,
};
pub(crate) use math::{percentile_nearest_rank_sorted, summarize_times_us};

#[derive(Debug, Clone)]
struct ReproPackItem {
    script_path: PathBuf,
    bundle_artifact: PathBuf,
}

#[derive(Debug)]
struct LaunchedDemo {
    child: Child,
    launched_unix_ms: u64,
    launched_instant: Instant,
    launch_cmd: Vec<String>,
    #[cfg(not(windows))]
    process_footprint_sampler: Option<crate::compare::ProcessFootprintSamplerHandle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BundleDoctorMode {
    Off,
    CheckRequired,
    CheckAll,
    Fix,
    FixDryRun,
}

fn parse_bundle_doctor_mode_value(v: &str) -> Option<BundleDoctorMode> {
    match v.trim() {
        "" => Some(BundleDoctorMode::CheckRequired),
        "off" | "0" | "false" => Some(BundleDoctorMode::Off),
        "check" | "required" | "check-required" | "check_required" => {
            Some(BundleDoctorMode::CheckRequired)
        }
        "check-all" | "check_all" | "all" | "strict" => Some(BundleDoctorMode::CheckAll),
        "fix" => Some(BundleDoctorMode::Fix),
        "fix-dry-run" | "fix_dry_run" | "fix-plan" | "fix_plan" => {
            Some(BundleDoctorMode::FixDryRun)
        }
        _ => None,
    }
}

fn parse_bundle_doctor_mode_from_rest(
    rest: &[String],
) -> Result<(BundleDoctorMode, Vec<String>), String> {
    let mut mode: BundleDoctorMode = BundleDoctorMode::Off;
    let mut out: Vec<String> = Vec::with_capacity(rest.len());

    let mut i: usize = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let (is_flag, value_inline) = if let Some(v) = arg.strip_prefix("--bundle-doctor=") {
            (true, Some(v))
        } else if let Some(v) = arg.strip_prefix("--doctor=") {
            (true, Some(v))
        } else if arg == "--bundle-doctor" || arg == "--doctor" {
            (true, None)
        } else {
            (false, None)
        };

        if !is_flag {
            out.push(rest[i].clone());
            i += 1;
            continue;
        }

        if let Some(v) = value_inline {
            mode = parse_bundle_doctor_mode_value(v).ok_or_else(|| {
                format!("invalid value for {arg} (expected off|check|check-all|fix|fix-dry-run)")
            })?;
            i += 1;
            continue;
        }

        let next = rest.get(i + 1).map(|s| s.as_str()).unwrap_or("");
        if next.starts_with('-') || next.is_empty() {
            mode = BundleDoctorMode::CheckRequired;
            i += 1;
            continue;
        }

        mode = parse_bundle_doctor_mode_value(next).ok_or_else(|| {
            format!("invalid value for {arg} {next} (expected off|check|check-all|fix|fix-dry-run)")
        })?;
        i += 2;
    }

    Ok((mode, out))
}

fn run_bundle_doctor_for_bundle_path(
    bundle_path: &Path,
    mode: BundleDoctorMode,
    warmup_frames: u64,
) -> Result<(), String> {
    if mode == BundleDoctorMode::Off {
        return Ok(());
    }

    let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
    let opts = match mode {
        BundleDoctorMode::Off => crate::commands::doctor::DoctorRunOptions::default(),
        BundleDoctorMode::CheckRequired => crate::commands::doctor::DoctorRunOptions {
            check_required: true,
            ..Default::default()
        },
        BundleDoctorMode::CheckAll => crate::commands::doctor::DoctorRunOptions {
            check_all: true,
            ..Default::default()
        },
        BundleDoctorMode::Fix => crate::commands::doctor::DoctorRunOptions {
            fix_bundle_json: true,
            fix_schema2: true,
            fix_sidecars: true,
            check_required: true,
            ..Default::default()
        },
        BundleDoctorMode::FixDryRun => crate::commands::doctor::DoctorRunOptions {
            fix_bundle_json: true,
            fix_schema2: true,
            fix_sidecars: true,
            fix_dry_run: true,
            check_required: true,
            ..Default::default()
        },
    };

    let run = crate::commands::doctor::run_doctor_for_bundle_dir(&bundle_dir, warmup_frames, opts)?;
    let ok = run
        .report
        .get("ok")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let required_ok = run
        .report
        .get("required_ok")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if mode == BundleDoctorMode::FixDryRun {
        if !run.fixes_planned.is_empty() {
            eprintln!("doctor: bundle_dir: {}", run.bundle_dir.display());
            eprintln!("doctor: warmup_frames: {warmup_frames}");
            for f in &run.fixes_planned {
                eprintln!("doctor: plan: {f}");
            }
            return Err(
                "bundle-doctor dry-run planned fixes; re-run with `--bundle-doctor fix`"
                    .to_string(),
            );
        }
        return Ok(());
    }

    for f in &run.fixes_applied {
        eprintln!("doctor: fixed: {f}");
    }

    match mode {
        BundleDoctorMode::CheckRequired => {
            if !required_ok {
                return Err(format!(
                    "bundle-doctor check-required failed (tip: fretboard diag doctor --fix-sidecars {} --warmup-frames {})",
                    run.bundle_dir.display(),
                    warmup_frames
                ));
            }
        }
        BundleDoctorMode::CheckAll => {
            if !ok {
                return Err(format!(
                    "bundle-doctor check-all failed (tip: fretboard diag doctor --fix-sidecars {} --warmup-frames {})",
                    run.bundle_dir.display(),
                    warmup_frames
                ));
            }
        }
        BundleDoctorMode::Fix => {
            if !required_ok {
                return Err(format!(
                    "bundle-doctor fix did not reach required_ok (tip: fretboard diag doctor {} --warmup-frames {})",
                    run.bundle_dir.display(),
                    warmup_frames
                ));
            }
        }
        BundleDoctorMode::Off | BundleDoctorMode::FixDryRun => {}
    }

    Ok(())
}

pub fn diag_cmd(args: Vec<String>) -> Result<(), String> {
    crate::cli::dispatch_diag_command(&args)
}

pub(crate) fn triage_json_from_stats(
    bundle_path: &Path,
    report: &BundleStatsReport,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> serde_json::Value {
    triage_json::triage_json_from_stats(bundle_path, report, sort, warmup_frames)
}

pub(crate) fn script_requests_screenshots(script: &Path) -> bool {
    let Ok(resolved) = crate::script_tooling::read_script_json_resolving_redirects(script) else {
        return false;
    };
    script_requests_screenshots_value(&resolved.value)
}

fn script_required_capabilities(script: &Path) -> Vec<String> {
    let Ok(resolved) = crate::script_tooling::read_script_json_resolving_redirects(script) else {
        return Vec::new();
    };
    script_required_capabilities_value(&resolved.value)
}

fn script_env_defaults(script: &Path) -> Vec<(String, String)> {
    let Ok(resolved) = crate::script_tooling::read_script_json_resolving_redirects(script) else {
        return Vec::new();
    };
    script_env_defaults_value(&resolved.value)
}

fn script_requests_screenshots_value(value: &serde_json::Value) -> bool {
    value
        .get("steps")
        .and_then(|v| v.as_array())
        .is_some_and(|steps| {
            steps.iter().any(|s| {
                s.get("type")
                    .and_then(|v| v.as_str())
                    .is_some_and(|t| t == "capture_screenshot")
            })
        })
}

fn script_required_capabilities_value(value: &serde_json::Value) -> Vec<String> {
    let mut required: Vec<String> = Vec::new();

    let schema_version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if schema_version >= 2 {
        required.push("diag.script_v2".to_string());
    }

    if script_requests_screenshots_value(value) {
        required.push("diag.screenshot_png".to_string());
    }

    if schema_version >= 2 {
        fn window_target_requires_multi_window(window: &serde_json::Value) -> bool {
            let Some(kind) = window.get("kind").and_then(|v| v.as_str()) else {
                return false;
            };
            matches!(kind, "first_seen_other" | "last_seen_other" | "window_ffi")
        }

        if value
            .get("steps")
            .and_then(|v| v.as_array())
            .is_some_and(|steps| {
                steps.iter().any(|s| {
                    s.get("window")
                        .is_some_and(|w| window_target_requires_multi_window(w))
                })
            })
        {
            required.push("diag.multi_window".to_string());
        }
    }

    for cap in crate::script_tooling::infer_required_capabilities_from_value(value) {
        required.push(cap);
    }

    if let Some(meta_required) = value
        .get("meta")
        .and_then(|m| m.get("required_capabilities"))
        .and_then(|v| v.as_array())
    {
        for cap in meta_required.iter().filter_map(|v| v.as_str()) {
            let cap = cap.trim();
            if cap.is_empty() {
                continue;
            }
            required.push(cap.to_string());
        }
    }

    let mut normalized: Vec<String> = required
        .into_iter()
        .filter_map(|c| compat::normalize_capability(&c))
        .collect();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn script_env_defaults_value(value: &serde_json::Value) -> Vec<(String, String)> {
    use std::collections::BTreeMap;

    fn is_valid_key(key: &str) -> bool {
        let key = key.trim();
        if key.is_empty() {
            return false;
        }
        if key.contains('=') {
            return false;
        }
        true
    }

    fn normalize_value(v: &serde_json::Value) -> Option<String> {
        match v {
            serde_json::Value::String(s) => Some(s.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }

    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let Some(meta) = value.get("meta") else {
        return Vec::new();
    };
    let Some(raw) = meta.get("env_defaults") else {
        return Vec::new();
    };

    match raw {
        serde_json::Value::Object(map) => {
            for (key, v) in map.iter() {
                if !is_valid_key(key) {
                    continue;
                }
                let Some(value) = normalize_value(v) else {
                    continue;
                };
                out.insert(key.trim().to_string(), value);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items.iter().filter_map(|v| v.as_str()) {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                let Some((key, value)) = item.split_once('=') else {
                    continue;
                };
                let key = key.trim();
                if !is_valid_key(key) {
                    continue;
                }
                out.insert(key.to_string(), value.to_string());
            }
        }
        _ => {}
    }

    out.into_iter().collect()
}

pub(crate) fn resolve_filesystem_capabilities_path(base_dir: &Path) -> Option<PathBuf> {
    let direct = base_dir.join("capabilities.json");
    if direct.is_file() {
        return Some(direct);
    }
    let root = base_dir.join("_root").join("capabilities.json");
    if root.is_file() {
        return Some(root);
    }
    if let Some(parent) = base_dir.parent() {
        let from_parent = parent.join("capabilities.json");
        if from_parent.is_file() {
            return Some(from_parent);
        }
    }
    None
}

pub(crate) fn read_filesystem_capabilities_payload(
    path: &Path,
) -> Option<fret_diag_protocol::FilesystemCapabilitiesV1> {
    let Ok(bytes) = std::fs::read(path) else {
        return None;
    };
    serde_json::from_slice::<fret_diag_protocol::FilesystemCapabilitiesV1>(&bytes).ok()
}

pub(crate) fn normalize_filesystem_capabilities(
    parsed: &fret_diag_protocol::FilesystemCapabilitiesV1,
) -> Vec<String> {
    let mut caps: Vec<String> = parsed
        .capabilities
        .iter()
        .filter_map(|c| compat::normalize_capability(c))
        .collect();
    caps.sort();
    caps.dedup();
    caps
}

fn read_filesystem_capabilities(out_dir: &Path) -> Vec<String> {
    let Some(path) = resolve_filesystem_capabilities_path(out_dir) else {
        return Vec::new();
    };
    let Some(parsed) = read_filesystem_capabilities_payload(&path) else {
        return Vec::new();
    };
    normalize_filesystem_capabilities(&parsed)
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CapabilitySourceKind {
    Filesystem,
    TransportSession,
    Inline,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CapabilitySource {
    kind: CapabilitySourceKind,
    path: Option<PathBuf>,
    label: Option<String>,
    transport: Option<String>,
    session_id: Option<String>,
}

impl CapabilitySource {
    fn kind_str(&self) -> &'static str {
        match self.kind {
            CapabilitySourceKind::Filesystem => "filesystem",
            CapabilitySourceKind::TransportSession => "transport_session",
            CapabilitySourceKind::Inline => "inline",
            CapabilitySourceKind::Unknown => "unknown",
        }
    }

    pub(crate) fn filesystem(path: Option<&Path>) -> Self {
        Self {
            kind: CapabilitySourceKind::Filesystem,
            path: path.map(Path::to_path_buf),
            label: None,
            transport: Some("filesystem".to_string()),
            session_id: None,
        }
    }

    pub(crate) fn transport_session(transport: &str, session_id: &str) -> Self {
        Self {
            kind: CapabilitySourceKind::TransportSession,
            path: None,
            label: Some(format!("{transport}:{session_id}")),
            transport: Some(transport.to_string()),
            session_id: Some(session_id.to_string()),
        }
    }

    pub(crate) fn transport_name(&self) -> &str {
        self.transport.as_deref().unwrap_or(match self.kind {
            CapabilitySourceKind::Filesystem => "filesystem",
            CapabilitySourceKind::TransportSession => "transport_session",
            CapabilitySourceKind::Inline => "inline",
            CapabilitySourceKind::Unknown => "unknown",
        })
    }

    pub(crate) fn source_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub(crate) fn legacy_check_source(&self) -> String {
        self.transport_name().to_string()
    }

    pub(crate) fn legacy_label(&self) -> String {
        if let Some(path) = self.path.as_deref() {
            return format!("{}:{}", self.transport_name(), path.display());
        }
        if let Some(label) = self.label.as_deref() {
            return label.to_string();
        }
        match self.kind {
            CapabilitySourceKind::Filesystem => {
                "filesystem:<missing capabilities.json>".to_string()
            }
            CapabilitySourceKind::TransportSession => self
                .session_id
                .as_deref()
                .map(|session_id| format!("{}:{session_id}", self.transport_name()))
                .unwrap_or_else(|| format!("{}:<missing session>", self.transport_name())),
            CapabilitySourceKind::Inline => "inline".to_string(),
            CapabilitySourceKind::Unknown => "unknown".to_string(),
        }
    }

    pub(crate) fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "kind": self.kind_str(),
            "path": self.path.as_ref().map(|path| path.display().to_string()),
            "label": self.label.clone().or_else(|| Some(self.legacy_label())),
            "transport": self.transport.clone(),
            "session_id": self.session_id.clone(),
        })
    }
}

pub(crate) fn resolve_filesystem_capabilities_source(base_dir: &Path) -> CapabilitySource {
    let source_path = resolve_filesystem_capabilities_path(base_dir);
    CapabilitySource::filesystem(source_path.as_deref())
}

pub(crate) fn read_filesystem_capabilities_with_provenance(
    base_dir: &Path,
) -> (CapabilitySource, Vec<String>) {
    let source = resolve_filesystem_capabilities_source(base_dir);
    let available = source
        .source_path()
        .and_then(read_filesystem_capabilities_payload)
        .map(|parsed| normalize_filesystem_capabilities(&parsed))
        .unwrap_or_default();
    (source, available)
}

fn capabilities_check_v1(
    source: &str,
    required: &[String],
    available: &[String],
) -> UiCapabilitiesCheckV1 {
    let available_set: std::collections::HashSet<&str> =
        available.iter().map(|s| s.as_str()).collect();
    let mut missing: Vec<String> = required
        .iter()
        .filter(|c| !available_set.contains(c.as_str()))
        .cloned()
        .collect();
    missing.sort();
    missing.dedup();

    UiCapabilitiesCheckV1 {
        schema_version: 1,
        source: source.to_string(),
        required: required.to_vec(),
        available: available.to_vec(),
        missing,
    }
}

fn write_script_result_capability_missing(
    script_result_path: &Path,
    check: &UiCapabilitiesCheckV1,
) {
    let now = now_unix_ms();
    let missing = check.missing.join(", ");
    let reason = format!(
        "missing required diagnostics capabilities: {} (source={})",
        missing, check.source
    );

    let evidence = UiScriptEvidenceV1 {
        event_log: vec![UiScriptEventLogEntryV1 {
            unix_ms: now,
            kind: "capability_missing".to_string(),
            step_index: None,
            note: Some(missing),
            bundle_dir: None,
            window: None,
            tick_id: None,
            frame_id: None,
            window_snapshot_seq: None,
        }],
        capabilities_check: Some(check.clone()),
        ..UiScriptEvidenceV1::default()
    };

    let result = UiScriptResultV1 {
        schema_version: 1,
        run_id: 0,
        updated_unix_ms: now,
        window: None,
        stage: UiScriptStageV1::Failed,
        step_index: None,
        reason_code: Some("capability.missing".to_string()),
        reason: Some(reason),
        evidence: Some(evidence),
        last_bundle_dir: None,
        last_bundle_artifact: None,
    };

    let _ = write_json_value(
        script_result_path,
        &serde_json::to_value(&result).unwrap_or_else(|_| serde_json::json!({})),
    );
}

fn gate_required_capabilities_with_script_result(
    out_path: &Path,
    script_result_path: &Path,
    required: &[String],
    available: &[String],
    source: &str,
) -> Result<(), String> {
    let check = capabilities_check_v1(source, required, available);
    if check.missing.is_empty() {
        return Ok(());
    }

    let missing = check.missing.clone();
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": "failed",
        "source": source,
        "required": required,
        "available": available,
        "missing": missing,
    });
    let _ = write_json_value(out_path, &payload);

    write_script_result_capability_missing(script_result_path, &check);

    Err(format!(
        "missing required diagnostics capabilities: {} (see {})",
        check.missing.join(", "),
        out_path.display()
    ))
}

#[cfg(test)]
mod capability_tests {
    use super::*;

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let unique = format!(
            "{}-{}-{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        dir.push(unique);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn gates_missing_screenshot_capability_and_writes_check_file() {
        let out_dir = make_temp_dir("fret-diag-capabilities-gate");
        let script_path = out_dir.join("script.json");
        let check_path = out_dir.join("check.capabilities.json");
        let script_result_path = out_dir.join("script.result.json");

        let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
            schema_version: 1,
            capabilities: vec!["diag.script_v2".to_string()],
            runner_kind: None,
            runner_version: None,
            hints: None,
        };
        std::fs::write(
            out_dir.join("capabilities.json"),
            serde_json::to_string_pretty(&caps).unwrap() + "\n",
        )
        .unwrap();

        let script = serde_json::json!({
            "schema_version": 2,
            "steps": [
                { "type": "capture_screenshot", "label": null, "timeout_frames": 30 }
            ]
        });
        std::fs::write(
            &script_path,
            serde_json::to_string_pretty(&script).unwrap() + "\n",
        )
        .unwrap();

        let required = script_required_capabilities(&script_path);
        assert!(required.contains(&"diag.script_v2".to_string()));
        assert!(required.contains(&"diag.screenshot_png".to_string()));

        let available = read_filesystem_capabilities(&out_dir);
        assert_eq!(available, vec!["diag.script_v2".to_string()]);

        let err = gate_required_capabilities_with_script_result(
            &check_path,
            &script_result_path,
            &required,
            &available,
            "filesystem",
        )
        .unwrap_err();
        assert!(err.contains("missing required diagnostics capabilities"));
        assert!(check_path.is_file());

        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&check_path).unwrap()).unwrap();
        let missing = value
            .get("missing")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();
        assert!(missing.contains(&"diag.screenshot_png".to_string()));

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn read_filesystem_capabilities_with_provenance_falls_back_to_parent_path() {
        let parent_dir = make_temp_dir("fret-diag-capabilities-parent");
        let run_dir = parent_dir.join("session").join("bundle");
        std::fs::create_dir_all(&run_dir).unwrap();

        let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
            schema_version: 1,
            capabilities: vec!["multi_window".to_string(), "diag.script_v2".to_string()],
            runner_kind: None,
            runner_version: None,
            hints: None,
        };
        std::fs::write(
            parent_dir.join("session").join("capabilities.json"),
            serde_json::to_string_pretty(&caps).unwrap() + "\n",
        )
        .unwrap();
        let expected_source_path = parent_dir.join("session").join("capabilities.json");

        let (source, available) = read_filesystem_capabilities_with_provenance(&run_dir);

        assert_eq!(source.source_path(), Some(expected_source_path.as_path()));
        assert_eq!(
            available,
            vec![
                "diag.multi_window".to_string(),
                "diag.script_v2".to_string()
            ]
        );

        let _ = std::fs::remove_dir_all(&parent_dir);
    }

    #[test]
    fn resolve_filesystem_capabilities_source_formats_legacy_label() {
        let parent_dir = make_temp_dir("fret-diag-capabilities-source-label");
        let run_dir = parent_dir.join("session").join("bundle");
        std::fs::create_dir_all(&run_dir).unwrap();

        let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
            schema_version: 1,
            capabilities: vec!["diag.script_v2".to_string()],
            runner_kind: None,
            runner_version: None,
            hints: None,
        };
        let capabilities_path = parent_dir.join("session").join("capabilities.json");
        std::fs::write(
            &capabilities_path,
            serde_json::to_string_pretty(&caps).unwrap() + "\n",
        )
        .unwrap();

        let source = resolve_filesystem_capabilities_source(&run_dir);

        assert_eq!(source.source_path(), Some(capabilities_path.as_path()));
        assert_eq!(
            source.legacy_label(),
            format!("filesystem:{}", capabilities_path.display())
        );

        let _ = std::fs::remove_dir_all(&parent_dir);
    }

    #[test]
    fn transport_session_capability_source_keeps_transport_identity() {
        let source = CapabilitySource::transport_session("devtools_ws", "session-123");

        assert_eq!(source.transport_name(), "devtools_ws");
        assert_eq!(source.source_path(), None);
        assert_eq!(source.legacy_check_source(), "devtools_ws");
        assert_eq!(source.legacy_label(), "devtools_ws:session-123");
    }

    #[test]
    fn doctor_report_includes_normalized_capabilities_from_shared_loader() {
        let bundle_dir = make_temp_dir("fret-diag-doctor-capabilities");
        let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
            schema_version: 1,
            capabilities: vec!["multi_window".to_string(), "diag.script_v2".to_string()],
            runner_kind: Some("filesystem".to_string()),
            runner_version: Some("1".to_string()),
            hints: None,
        };
        std::fs::write(
            bundle_dir.join("capabilities.json"),
            serde_json::to_string_pretty(&caps).unwrap() + "\n",
        )
        .unwrap();
        let expected_capabilities_path = bundle_dir.join("capabilities.json");
        let expected_capabilities_path_str = expected_capabilities_path.display().to_string();

        let report = crate::commands::doctor::doctor_report_json(&bundle_dir, 4);

        assert_eq!(
            report
                .get("capabilities_path")
                .and_then(|value| value.as_str()),
            Some(expected_capabilities_path_str.as_str())
        );
        assert_eq!(
            report
                .get("capability_source")
                .and_then(|value| value.get("kind"))
                .and_then(|value| value.as_str()),
            Some("filesystem")
        );
        assert_eq!(
            report
                .get("capability_source")
                .and_then(|value| value.get("path"))
                .and_then(|value| value.as_str()),
            Some(expected_capabilities_path_str.as_str())
        );
        assert_eq!(
            report
                .get("capabilities")
                .and_then(|value| value.get("normalized_capabilities_total"))
                .and_then(|value| value.as_u64()),
            Some(2)
        );
        assert_eq!(
            report
                .get("capabilities")
                .and_then(|value| value.get("normalized_capabilities"))
                .and_then(|value| value.as_array())
                .map(|items| items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .collect::<Vec<_>>()),
            Some(vec!["diag.multi_window", "diag.script_v2"])
        );

        let _ = std::fs::remove_dir_all(&bundle_dir);
    }

    #[test]
    fn script_required_capabilities_include_step_inferred_caps() {
        let out_dir = make_temp_dir("fret-diag-capabilities-infer");
        let script_path = out_dir.join("script.json");

        let script = serde_json::json!({
            "schema_version": 2,
            "steps": [
                { "type": "set_cursor_in_window_logical", "window": null, "x_px": 1.0, "y_px": 2.0 },
                { "type": "set_clipboard_text", "text": "hello" },
                { "type": "inject_incoming_open", "items": [
                    { "kind": "text", "text": "hello", "media_type": "text/plain" }
                ] }
            ]
        });
        std::fs::write(
            &script_path,
            serde_json::to_string_pretty(&script).unwrap() + "\n",
        )
        .unwrap();

        let required = script_required_capabilities(&script_path);
        assert!(required.contains(&"diag.script_v2".to_string()));
        assert!(required.contains(&"diag.cursor_screen_pos_override".to_string()));
        assert!(required.contains(&"diag.clipboard_text".to_string()));
        assert!(required.contains(&"diag.incoming_open_inject".to_string()));

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn gates_missing_capability_writes_script_result_with_structured_evidence() {
        let out_dir = make_temp_dir("fret-diag-capabilities-script-result");
        let script_path = out_dir.join("script.json");
        let check_path = out_dir.join("check.capabilities.json");
        let script_result_path = out_dir.join("script.result.json");

        let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
            schema_version: 1,
            capabilities: vec!["diag.script_v2".to_string()],
            runner_kind: None,
            runner_version: None,
            hints: None,
        };
        std::fs::write(
            out_dir.join("capabilities.json"),
            serde_json::to_string_pretty(&caps).unwrap() + "\n",
        )
        .unwrap();

        let script = serde_json::json!({
            "schema_version": 2,
            "steps": [
                { "type": "capture_screenshot", "label": null, "timeout_frames": 30 }
            ]
        });
        std::fs::write(
            &script_path,
            serde_json::to_string_pretty(&script).unwrap() + "\n",
        )
        .unwrap();

        let required = script_required_capabilities(&script_path);
        let available = read_filesystem_capabilities(&out_dir);
        let err = gate_required_capabilities_with_script_result(
            &check_path,
            &script_result_path,
            &required,
            &available,
            "filesystem",
        )
        .unwrap_err();
        assert!(err.contains("missing required diagnostics capabilities"));
        assert!(check_path.is_file());
        assert!(script_result_path.is_file());

        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&script_result_path).unwrap()).unwrap();
        assert_eq!(
            value.get("reason_code").and_then(|v| v.as_str()),
            Some("capability.missing")
        );
        let check = value
            .get("evidence")
            .and_then(|v| v.get("capabilities_check"))
            .cloned()
            .unwrap_or_default();
        let missing = check
            .get("missing")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();
        assert!(missing.contains(&"diag.screenshot_png".to_string()));

        let _ = std::fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn parses_script_env_defaults_from_meta() {
        let script = serde_json::json!({
            "schema_version": 2,
            "meta": {
                "env_defaults": {
                    "FRET_TEXT_SYSTEM_FONTS": 0,
                    "FRET_UI_GALLERY_BOOTSTRAP_FONTS": "1",
                    "": "ignored",
                    "NOT=ALLOWED": "ignored"
                }
            },
            "steps": []
        });
        let parsed = script_env_defaults_value(&script);
        assert_eq!(
            parsed,
            vec![
                ("FRET_TEXT_SYSTEM_FONTS".to_string(), "0".to_string()),
                (
                    "FRET_UI_GALLERY_BOOTSTRAP_FONTS".to_string(),
                    "1".to_string()
                ),
            ]
        );

        let script = serde_json::json!({
            "schema_version": 2,
            "meta": {
                "env_defaults": [
                    "FRET_A=1",
                    "FRET_B=two",
                    "FRET_A=3"
                ]
            },
            "steps": []
        });
        let parsed = script_env_defaults_value(&script);
        assert_eq!(
            parsed,
            vec![
                ("FRET_A".to_string(), "3".to_string()),
                ("FRET_B".to_string(), "two".to_string()),
            ]
        );
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedScriptPaths {
    pub(crate) out_dir: PathBuf,
    pub(crate) trigger_path: PathBuf,
    pub(crate) ready_path: PathBuf,
    pub(crate) exit_path: PathBuf,
    pub(crate) script_path: PathBuf,
    pub(crate) script_trigger_path: PathBuf,
    pub(crate) script_result_path: PathBuf,
    pub(crate) script_result_trigger_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedRunContext {
    pub(crate) paths: ResolvedScriptPaths,
    pub(crate) fs_transport_cfg: crate::transport::FsDiagTransportConfig,
}

pub(crate) fn script_run_fs_transport_cfg(
    out_dir: &Path,
    script_path: &Path,
    script_trigger_path: &Path,
    script_result_path: &Path,
    script_result_trigger_path: &Path,
) -> crate::transport::FsDiagTransportConfig {
    let mut cfg = crate::transport::FsDiagTransportConfig::from_out_dir(out_dir.to_path_buf());
    cfg.script_path = script_path.to_path_buf();
    cfg.script_trigger_path = script_trigger_path.to_path_buf();
    cfg.script_result_path = script_result_path.to_path_buf();
    cfg.script_result_trigger_path = script_result_trigger_path.to_path_buf();
    cfg
}

pub(crate) fn script_result_fs_transport_cfg(
    out_dir: &Path,
    script_result_path: &Path,
    script_result_trigger_path: &Path,
) -> crate::transport::FsDiagTransportConfig {
    let mut cfg = crate::transport::FsDiagTransportConfig::from_out_dir(out_dir.to_path_buf());
    cfg.script_result_path = script_result_path.to_path_buf();
    cfg.script_result_trigger_path = script_result_trigger_path.to_path_buf();
    cfg
}

impl ResolvedScriptPaths {
    pub(crate) fn for_out_dir(workspace_root: &Path, out_dir: &Path) -> Self {
        let out_dir = resolve_path(workspace_root, out_dir.to_path_buf());
        Self {
            trigger_path: resolve_path(workspace_root, out_dir.join("trigger.touch")),
            ready_path: resolve_path(workspace_root, out_dir.join("ready.touch")),
            exit_path: resolve_path(workspace_root, out_dir.join("exit.touch")),
            script_path: resolve_path(workspace_root, out_dir.join("script.json")),
            script_trigger_path: resolve_path(workspace_root, out_dir.join("script.touch")),
            script_result_path: resolve_path(workspace_root, out_dir.join("script.result.json")),
            script_result_trigger_path: resolve_path(
                workspace_root,
                out_dir.join("script.result.touch"),
            ),
            out_dir,
        }
    }

    pub(crate) fn launch_fs_transport_cfg(&self) -> crate::transport::FsDiagTransportConfig {
        script_run_fs_transport_cfg(
            &self.out_dir,
            &self.script_path,
            &self.script_trigger_path,
            &self.script_result_path,
            &self.script_result_trigger_path,
        )
    }
}

fn matrix_launch_env(
    base: &[(String, String)],
    view_cache_enabled: bool,
) -> Result<Vec<(String, String)>, String> {
    if base
        .iter()
        .any(|(k, _)| k.as_str() == "FRET_UI_GALLERY_VIEW_CACHE")
    {
        return Err(
            "--env cannot override reserved var for diag matrix: FRET_UI_GALLERY_VIEW_CACHE"
                .to_string(),
        );
    }
    let mut env = base.to_vec();
    env.push((
        "FRET_UI_GALLERY_VIEW_CACHE".to_string(),
        if view_cache_enabled { "1" } else { "0" }.to_string(),
    ));
    Ok(env)
}

fn devtools_sanitize_export_dir_name(raw: &str) -> String {
    std::path::Path::new(raw)
        .file_name()
        .and_then(|v| v.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("bundle")
        .to_string()
}

// DevTools message waiting helpers live in `crates/fret-diag/src/devtools.rs`.

fn materialize_devtools_bundle_dumped(
    out_dir: &Path,
    dumped: &DevtoolsBundleDumpedV1,
) -> Result<PathBuf, String> {
    let export_dir_name = devtools_sanitize_export_dir_name(&dumped.dir);
    let export_dir = out_dir.join(&export_dir_name);
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    let bundle_path = export_dir.join("bundle.json");

    match dumped.bundle.clone() {
        Some(bundle) => {
            write_json_value(&bundle_path, &bundle)?;
        }
        None => {
            // Native apps may choose to omit embedding the bundle payload in the WS message
            // because the bundle is already written to disk. When possible, materialize by
            // reading the runtime's bundle.json from the advertised output directory.
            let runtime_out_dir = PathBuf::from(dumped.out_dir.as_str());
            let dumped_dir = PathBuf::from(dumped.dir.as_str());
            let runtime_dir = if dumped_dir.is_absolute() {
                dumped_dir
            } else {
                runtime_out_dir.join(dumped_dir)
            };
            let runtime_bundle_path = resolve_bundle_artifact_path(&runtime_dir);

            if runtime_bundle_path != bundle_path || !bundle_path.is_file() {
                let bytes = std::fs::read(&runtime_bundle_path).map_err(|e| {
                    format!(
                        "bundle.dumped did not include an embedded bundle payload, and the runtime bundle artifact was not readable ({}): {}",
                        runtime_bundle_path.display(),
                        e
                    )
                })?;
                let bundle = serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| {
                    format!(
                        "runtime bundle artifact was not valid JSON ({}): {}",
                        runtime_bundle_path.display(),
                        e
                    )
                })?;
                write_json_value(&bundle_path, &bundle)?;
            }
        }
    }

    let dumped_path = export_dir.join("bundle.dumped.json");
    let dumped_meta = DevtoolsBundleDumpedV1 {
        schema_version: dumped.schema_version,
        exported_unix_ms: dumped.exported_unix_ms,
        out_dir: dumped.out_dir.clone(),
        dir: dumped.dir.clone(),
        bundle: None,
        bundle_json_chunk: None,
        bundle_json_chunk_index: None,
        bundle_json_chunk_count: None,
    };
    write_json_value(
        &dumped_path,
        &serde_json::to_value(dumped_meta).unwrap_or_else(|_| serde_json::json!({})),
    )?;
    let _ = std::fs::write(out_dir.join("latest.txt"), export_dir_name.as_bytes());

    Ok(bundle_path)
}

fn artifact_stats_from_bundle_json_path(bundle_path: &Path) -> UiArtifactStatsV1 {
    let bundle_json_bytes = std::fs::metadata(bundle_path).ok().map(|m| m.len());
    let v = read_json_value(bundle_path).unwrap_or_else(|| serde_json::json!({}));

    let windows = v
        .get("windows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut event_count: u64 = 0;
    let mut snapshot_count: u64 = 0;
    for w in &windows {
        event_count = event_count.saturating_add(
            w.get("events")
                .and_then(|v| v.as_array())
                .map(|a| a.len() as u64)
                .unwrap_or(0),
        );
        snapshot_count = snapshot_count.saturating_add(
            w.get("snapshots")
                .and_then(|v| v.as_array())
                .map(|a| a.len() as u64)
                .unwrap_or(0),
        );
    }

    let (max_snapshots, dump_max_snapshots) = v
        .get("config")
        .and_then(|v| v.as_object())
        .map(|cfg| {
            let max = cfg
                .get("max_snapshots")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dump = cfg.get("dump_max_snapshots").and_then(|v| v.as_u64());
            (max, dump)
        })
        .unwrap_or((0, None));

    UiArtifactStatsV1 {
        schema_version: 1,
        bundle_json_bytes,
        window_count: windows.len() as u64,
        event_count,
        snapshot_count,
        max_snapshots,
        dump_max_snapshots,
    }
}

fn try_resolve_existing_filesystem_bundle_artifact(
    out_dir: &Path,
    connected: &ConnectedToolingTransport,
    result: &UiScriptResultV1,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    if connected.capability_source.transport_name() != "filesystem" {
        return None;
    }

    let runtime_reported_bundle = result
        .last_bundle_dir
        .as_deref()
        .is_some_and(|dir| !dir.trim().is_empty())
        || result.last_bundle_artifact.is_some();
    if !runtime_reported_bundle {
        return None;
    }

    let summary = crate::stats::ScriptResultSummary {
        run_id: result.run_id,
        stage: Some(
            match result.stage {
                UiScriptStageV1::Queued => "queued",
                UiScriptStageV1::Running => "running",
                UiScriptStageV1::Passed => "passed",
                UiScriptStageV1::Failed => "failed",
            }
            .to_string(),
        ),
        step_index: result.step_index.map(u64::from),
        reason_code: result.reason_code.clone(),
        reason: result.reason.clone(),
        last_bundle_dir: result.last_bundle_dir.clone(),
    };

    wait_for_bundle_artifact_from_script_result(out_dir, &summary, timeout_ms, poll_ms)
}

fn devtools_select_session_id(
    list: &DevtoolsSessionListV1,
    want: Option<&str>,
) -> Result<String, String> {
    if let Some(want) = want {
        if list.sessions.iter().any(|s| s.session_id == want) {
            return Ok(want.to_string());
        }
        let known = list
            .sessions
            .iter()
            .map(|s| s.session_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "unknown --devtools-session-id: {want} (known: {known}). Hint: refresh the session id via `cargo run -p fret-diag-export -- --list-sessions --token <token>`"
        ));
    }

    // DevTools servers include the caller (tooling) in `session.list`. When the target app is not
    // connected (or isn't configured to connect), tooling-only sessions would otherwise "select"
    // themselves and later hang waiting for script/bundle responses. Prefer selecting a non-tooling
    // app session by default.
    let non_tooling = list
        .sessions
        .iter()
        .filter(|s| s.client_kind != "tooling")
        .collect::<Vec<_>>();
    let sessions = if non_tooling.is_empty() {
        // Preserve the legacy error message while surfacing enough context to debug.
        let known = list
            .sessions
            .iter()
            .map(|s| format!("{}({})", s.session_id, s.client_kind))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(if known.is_empty() {
            "no DevTools sessions available (is the app connected?)".to_string()
        } else {
            format!(
                "no DevTools app sessions available (is the app connected?) (sessions: {known})"
            )
        });
    } else {
        non_tooling
    };

    if sessions.len() == 1 {
        return Ok(sessions[0].session_id.clone());
    }

    let web_apps = sessions
        .iter()
        .copied()
        .filter(|s| s.client_kind == "web_app")
        .collect::<Vec<_>>();
    if web_apps.len() == 1 {
        return Ok(web_apps[0].session_id.clone());
    }

    let known = list
        .sessions
        .iter()
        .map(|s| format!("{}({})", s.session_id, s.client_kind))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "multiple DevTools sessions available; pass --devtools-session-id (sessions: {known})"
    ))
}

struct ConnectedToolingTransport {
    devtools: DevtoolsOps,
    selected_session_id: String,
    available_caps: Vec<String>,
    capability_source: CapabilitySource,
}

fn connect_devtools_ws_tooling(
    ws_url: &str,
    token: &str,
    want_session_id: Option<&str>,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<ConnectedToolingTransport, String> {
    use crate::transport::{
        ClientKindV1, DevtoolsWsClientConfig, ToolingDiagClient, WsDiagTransportConfig,
    };

    let mut cfg = DevtoolsWsClientConfig::with_defaults(ws_url.to_string(), token.to_string());
    cfg.client_kind = ClientKindV1::Tooling;
    cfg.capabilities = vec![
        // Backwards-compatible (legacy, un-namespaced) control plane capabilities.
        "inspect".to_string(),
        "pick".to_string(),
        "scripts".to_string(),
        "bundles".to_string(),
        "sessions".to_string(),
        // Namespaced control plane capabilities (recommended).
        "devtools.inspect".to_string(),
        "devtools.pick".to_string(),
        "devtools.scripts".to_string(),
        "devtools.bundles".to_string(),
        "devtools.sessions".to_string(),
        // Script runner surface (v2+).
        "diag.script_v2".to_string(),
    ];

    let client = ToolingDiagClient::connect_ws(WsDiagTransportConfig::native(cfg))?;
    let devtools = DevtoolsOps::new(client);

    let sessions = crate::devtools::wait_for_message(&devtools, timeout_ms, poll_ms, |msg| {
        if msg.r#type != "session.list" {
            return None;
        }
        serde_json::from_value::<DevtoolsSessionListV1>(msg.payload).ok()
    })?;

    let selected_session_id = devtools_select_session_id(&sessions, want_session_id)?;
    devtools.set_default_session_id(Some(selected_session_id.clone()));

    let mut available_caps: Vec<String> = sessions
        .sessions
        .iter()
        .find(|s| s.session_id == selected_session_id)
        .map(|s| s.capabilities.clone())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| compat::normalize_capability(&c))
        .collect();
    available_caps.sort();
    available_caps.dedup();

    Ok(ConnectedToolingTransport {
        devtools,
        capability_source: CapabilitySource::transport_session("devtools_ws", &selected_session_id),
        selected_session_id,
        available_caps,
    })
}

fn connect_filesystem_tooling(
    cfg: &crate::transport::FsDiagTransportConfig,
    ready_path: &Path,
    require_ready: bool,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<ConnectedToolingTransport, String> {
    use crate::transport::ToolingDiagClient;

    if require_ready {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        while Instant::now() < deadline {
            if std::fs::metadata(ready_path).is_ok() {
                break;
            }
            std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
        }
    }

    let client = ToolingDiagClient::connect_fs(cfg.clone())?;
    let devtools = DevtoolsOps::new(client);

    let sessions = crate::devtools::wait_for_message(&devtools, timeout_ms, poll_ms, |msg| {
        if msg.r#type != "session.list" {
            return None;
        }
        serde_json::from_value::<DevtoolsSessionListV1>(msg.payload).ok()
    })?;

    let selected_session_id = devtools_select_session_id(&sessions, None)?;
    devtools.set_default_session_id(Some(selected_session_id.clone()));

    let mut available_caps: Vec<String> = sessions
        .sessions
        .iter()
        .find(|s| s.session_id == selected_session_id)
        .map(|s| s.capabilities.clone())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| compat::normalize_capability(&c))
        .collect();
    available_caps.sort();
    available_caps.dedup();

    Ok(ConnectedToolingTransport {
        devtools,
        capability_source: CapabilitySource::transport_session("filesystem", &selected_session_id),
        selected_session_id,
        available_caps,
    })
}

#[allow(clippy::too_many_arguments)]
fn run_script_over_transport(
    out_dir: &Path,
    connected: &ConnectedToolingTransport,
    script_json: serde_json::Value,
    dump_bundle: bool,
    trace_chrome: bool,
    bundle_label: Option<&str>,
    dump_max_snapshots: Option<u32>,
    timeout_ms: u64,
    poll_ms: u64,
    script_result_path: &Path,
    capabilities_check_path: &Path,
) -> Result<(UiScriptResultV1, Option<PathBuf>), String> {
    fn read_prev_run_id(path: &Path) -> u64 {
        read_json_value(path)
            .and_then(|v| v.get("run_id").and_then(|v| v.as_u64()))
            .unwrap_or(0)
    }

    let required_caps = script_required_capabilities_value(&script_json);
    if !required_caps.is_empty() {
        let source = connected.capability_source.legacy_check_source();
        gate_required_capabilities_with_script_result(
            capabilities_check_path,
            script_result_path,
            &required_caps,
            &connected.available_caps,
            &source,
        )?;
    }

    let prev_run_id = read_prev_run_id(script_result_path);
    let mut target_run_id: Option<u64> = None;
    let mut last_seen_stage: Option<&'static str> = None;
    let mut last_seen_step_index: Option<u32> = None;

    let seam = connected.devtools.client().seam_v1();
    let mut script_retoucher = seam.new_script_run_retoucher(timeout_ms, poll_ms);
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);

    let script_json_value = script_json;
    connected
        .devtools
        .script_run_value(None, script_json_value.clone());

    let mut result = 'wait: loop {
        while let Some(msg) = connected.devtools.try_recv() {
            if msg.r#type == "session.removed"
                && let Ok(removed) =
                    serde_json::from_value::<DevtoolsSessionRemovedV1>(msg.payload.clone())
                && removed.session_id == connected.selected_session_id
            {
                return Err(format!(
                    "DevTools session disconnected while waiting for script result (session_id={}). Hint: refresh the page and re-run `cargo run -p fret-diag-export -- --list-sessions --token <token>`.",
                    connected.selected_session_id
                ));
            }

            if msg.r#type != "script.result"
                || msg.session_id.as_deref() != Some(&connected.selected_session_id)
            {
                continue;
            }
            let Ok(parsed) = serde_json::from_value::<UiScriptResultV1>(msg.payload) else {
                continue;
            };

            advance_target_run_id(prev_run_id, &mut target_run_id, parsed.run_id);
            if Some(parsed.run_id) != target_run_id {
                continue;
            }

            last_seen_stage = Some(match parsed.stage {
                UiScriptStageV1::Queued => "queued",
                UiScriptStageV1::Running => "running",
                UiScriptStageV1::Passed => "passed",
                UiScriptStageV1::Failed => "failed",
            });
            last_seen_step_index = parsed.step_index;

            // Transport-agnostic streaming hook: persist incremental script progress so external
            // tooling can observe long runs without waiting for completion.
            //
            // IMPORTANT: In filesystem mode, the runtime owns the shared control-plane
            // `<out_dir>/script.result.json`. Re-writing it from tooling can race with the runtime
            // and cause missed edge-detection updates (e.g. clobbering the final `passed` result
            // with a stale `running` snapshot). Tooling should only write to a distinct output
            // path in filesystem mode (for example `<out_dir>/tool.script.result.json`).
            let runtime_owned_path = connected.capability_source.transport_name() == "filesystem"
                && *script_result_path == out_dir.join("script.result.json");
            if !runtime_owned_path {
                let _ = write_json_value(
                    script_result_path,
                    &serde_json::to_value(&parsed).unwrap_or_else(|_| serde_json::json!({})),
                );
            }
            RunArtifactStore::new(out_dir, parsed.run_id).write_script_result(&parsed);

            if matches!(
                parsed.stage,
                UiScriptStageV1::Passed | UiScriptStageV1::Failed
            ) {
                break 'wait parsed;
            }
        }

        if Instant::now() >= deadline {
            let ws_hint = seam.timeout_hint_for_waiting_script_result();
            let note = format!(
                "source={} prev_run_id={} target_run_id={:?} last_seen_stage={} last_seen_step_index={:?} {}",
                connected.capability_source.transport_name(),
                prev_run_id,
                target_run_id,
                last_seen_stage.unwrap_or("none"),
                last_seen_step_index,
                ws_hint.unwrap_or(""),
            );
            write_tooling_failure_script_result_if_missing(
                script_result_path,
                "timeout.tooling.script_result",
                "timeout waiting for script result",
                "tooling_timeout",
                Some(note),
            );
            return Err(
                "timeout waiting for script result (DevTools WS: keep the app actively rendering; web tabs may be throttled in the background)"
                    .to_string(),
            );
        }

        if let Some(retoucher) = script_retoucher.as_mut() {
            retoucher.maybe_retouch_at(
                &connected.devtools,
                None,
                &script_json_value,
                target_run_id.is_some(),
                Instant::now(),
            );
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    };

    let bundle_path = if dump_bundle {
        if let Some(bundle_path) = try_resolve_existing_filesystem_bundle_artifact(
            out_dir, connected, &result, timeout_ms, poll_ms,
        ) {
            let run_artifacts = RunArtifactStore::new(out_dir, result.run_id);
            run_artifacts.write_bundle_artifact(&bundle_path);
            if result
                .last_bundle_dir
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            {
                result.last_bundle_dir = bundle_path
                    .parent()
                    .and_then(|path| path.file_name())
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string());
            }
            if result.last_bundle_artifact.is_none() {
                result.last_bundle_artifact =
                    Some(artifact_stats_from_bundle_json_path(&bundle_path));
            }
            Some(bundle_path)
        } else {
            let expected_request_id = seam.bundle_dump_request_id(
                &connected.devtools,
                None,
                bundle_label,
                dump_max_snapshots,
            );
            let dumped = seam
                .wait_for_bundle_dumped_with_baseline_mitigation(
                    &connected.devtools,
                    &connected.selected_session_id,
                    expected_request_id,
                    bundle_label,
                    timeout_ms,
                    poll_ms,
                )
                .inspect_err(|err| {
                    let reason_code = if err.contains("timed out waiting") {
                        "timeout.tooling.bundle_dump"
                    } else {
                        "tooling.bundle_dump.failed"
                    };
                    push_tooling_event_log_entry(
                        &mut result,
                        "tooling_bundle_dump_failed",
                        Some(err.clone()),
                    );
                    if matches!(result.stage, UiScriptStageV1::Passed) {
                        result.stage = UiScriptStageV1::Failed;
                        result.reason_code = Some(reason_code.to_string());
                        result.reason = Some(err.clone());
                    }
                    let _ = write_json_value(
                        script_result_path,
                        &serde_json::to_value(&result).unwrap_or_else(|_| serde_json::json!({})),
                    );
                })?;

            let bundle_path = match materialize_devtools_bundle_dumped(out_dir, &dumped) {
                Ok(v) => v,
                Err(err) => {
                    push_tooling_event_log_entry(
                        &mut result,
                        "tooling_bundle_materialize_failed",
                        Some(err.clone()),
                    );
                    if matches!(result.stage, UiScriptStageV1::Passed) {
                        result.stage = UiScriptStageV1::Failed;
                        result.reason_code = Some("tooling.bundle_materialize.failed".to_string());
                        result.reason = Some(err.clone());
                    }
                    let _ = write_json_value(
                        script_result_path,
                        &serde_json::to_value(&result).unwrap_or_else(|_| serde_json::json!({})),
                    );
                    return Err(err);
                }
            };
            let run_artifacts = RunArtifactStore::new(out_dir, result.run_id);
            run_artifacts.write_bundle_artifact(&bundle_path);
            if trace_chrome {
                let run_dir = run_artifacts.run_dir();
                let stable_bundle_path = crate::resolve_bundle_artifact_path(&run_dir);
                let src = if stable_bundle_path.is_file() {
                    stable_bundle_path
                } else {
                    bundle_path.clone()
                };
                let trace_path = run_dir.join("trace.chrome.json");
                if let Err(err) =
                    crate::trace::write_chrome_trace_from_bundle_path(&src, &trace_path)
                {
                    push_tooling_event_log_entry(
                        &mut result,
                        "tooling_trace_chrome_failed",
                        Some(err),
                    );
                } else {
                    run_artifacts.refresh_manifest_file_index();
                }
            }
            result.last_bundle_dir = Some(devtools_sanitize_export_dir_name(&dumped.dir));
            result.last_bundle_artifact = Some(artifact_stats_from_bundle_json_path(&bundle_path));
            Some(bundle_path)
        }
    } else {
        None
    };

    let _ = write_json_value(
        script_result_path,
        &serde_json::to_value(&result).unwrap_or_else(|_| serde_json::json!({})),
    );

    Ok((result, bundle_path))
}

fn dump_bundle_over_transport(
    out_dir: &Path,
    connected: &ConnectedToolingTransport,
    bundle_label: Option<&str>,
    dump_max_snapshots: Option<u32>,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<PathBuf, String> {
    let seam = connected.devtools.client().seam_v1();
    let expected_request_id =
        seam.bundle_dump_request_id(&connected.devtools, None, bundle_label, dump_max_snapshots);

    let dumped = seam.wait_for_bundle_dumped_with_baseline_mitigation(
        &connected.devtools,
        &connected.selected_session_id,
        expected_request_id,
        bundle_label,
        timeout_ms,
        poll_ms,
    )?;

    materialize_devtools_bundle_dumped(out_dir, &dumped)
}

#[allow(clippy::too_many_arguments)]
fn run_script_suite_collect_bundles(
    scripts: &[PathBuf],
    paths: &ResolvedScriptPaths,
    launch: &[String],
    launch_env: &[(String, String)],
    launch_high_priority: bool,
    workspace_root: &Path,
    timeout_ms: u64,
    poll_ms: u64,
    warmup_frames: u64,
    check_view_cache_reuse_stable_min: Option<u64>,
    check_view_cache_reuse_min: Option<u64>,
    check_overlay_synthesis_min: Option<u64>,
    overlay_synthesis_gate_predicate: Option<fn(&Path) -> bool>,
    check_viewport_input_min: Option<u64>,
    viewport_input_gate_predicate: Option<fn(&Path) -> bool>,
    check_dock_drag_min: Option<u64>,
    check_viewport_capture_min: Option<u64>,
) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(&paths.out_dir).map_err(|e| e.to_string())?;

    let launch = Some(launch.to_vec());
    let launch_fs_transport_cfg = paths.launch_fs_transport_cfg();
    let mut child = maybe_launch_demo(
        &launch,
        launch_env,
        workspace_root,
        &paths.ready_path,
        &paths.exit_path,
        &launch_fs_transport_cfg,
        scripts.iter().any(|src| script_requests_screenshots(src)),
        false,
        timeout_ms,
        poll_ms,
        launch_high_priority,
    )
    .inspect_err(|err| {
        write_tooling_failure_script_result_if_missing(
            &paths.script_result_path,
            "tooling.launch.failed",
            err,
            "tooling_error",
            Some("maybe_launch_demo".to_string()),
        );
    })?;

    let mut required_caps: Vec<String> = Vec::new();
    for src in scripts {
        required_caps.extend(script_required_capabilities(src));
    }
    required_caps.sort();
    required_caps.dedup();
    if !required_caps.is_empty() {
        let available_caps = read_filesystem_capabilities(&paths.out_dir);
        if let Err(e) = gate_required_capabilities_with_script_result(
            &paths.out_dir.join("check.capabilities.json"),
            &paths.script_result_path,
            &required_caps,
            &available_caps,
            "filesystem",
        ) {
            let _ = stop_launched_demo(&mut child, &paths.exit_path, poll_ms);
            return Err(e);
        }
    }

    let mut bundle_paths: Vec<PathBuf> = Vec::new();
    for src in scripts {
        let mut result = run_script_and_wait(
            src,
            &paths.script_path,
            &paths.script_trigger_path,
            &paths.script_result_path,
            &paths.script_result_trigger_path,
            timeout_ms,
            poll_ms,
        );
        if let Ok(summary) = &result
            && summary.stage.as_deref() == Some("failed")
            && let Some(dir) =
                wait_for_failure_dump_bundle(&paths.out_dir, summary, timeout_ms, poll_ms)
            && let Some(name) = dir.file_name().and_then(|s| s.to_str())
            && let Ok(summary) = result.as_mut()
        {
            summary.last_bundle_dir = Some(name.to_string());
        }
        let result = result?;
        if result.stage.as_deref() != Some("passed") {
            let _ = stop_launched_demo(&mut child, &paths.exit_path, poll_ms);
            return Err(format!(
                "unexpected script stage for {}: {:?}",
                src.display(),
                result.stage
            ));
        }

        let bundle_path = wait_for_bundle_artifact_from_script_result(
            &paths.out_dir,
            &result,
            timeout_ms,
            poll_ms,
        )
        .ok_or_else(|| {
            format!(
                "script passed but no bundle artifact was found (required for matrix): {}",
                src.display()
            )
        })?;

        if let Some(min) = check_view_cache_reuse_stable_min
            && min > 0
        {
            stats::check_bundle_for_view_cache_reuse_stable_min(
                &bundle_path,
                &paths.out_dir,
                min,
                warmup_frames,
            )?;
        }
        if let Some(min) = check_view_cache_reuse_min
            && min > 0
        {
            stats::check_bundle_for_view_cache_reuse_min(&bundle_path, min, warmup_frames)?;
        }
        if let Some(min) = check_overlay_synthesis_min
            && min > 0
        {
            let should_gate = overlay_synthesis_gate_predicate
                .map(|pred| pred(src))
                .unwrap_or(true);
            if should_gate {
                stats::check_bundle_for_overlay_synthesis_min(&bundle_path, min, warmup_frames)?;
            }
        }
        if let Some(min) = check_viewport_input_min
            && min > 0
        {
            let should_gate = viewport_input_gate_predicate
                .map(|pred| pred(src))
                .unwrap_or(true);
            if should_gate {
                stats::check_bundle_for_viewport_input_min(&bundle_path, min, warmup_frames)?;
            }
        }
        if let Some(min) = check_dock_drag_min
            && min > 0
        {
            stats::check_bundle_for_dock_drag_min(&bundle_path, min, warmup_frames)?;
        }
        if let Some(min) = check_viewport_capture_min
            && min > 0
        {
            stats::check_bundle_for_viewport_capture_min(&bundle_path, min, warmup_frames)?;
        }

        bundle_paths.push(bundle_path);
    }

    if let Some(footprint) = stop_launched_demo(&mut child, &paths.exit_path, poll_ms) {
        let killed = footprint
            .get("killed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if killed {
            crate::tooling_failures::mark_existing_script_result_tooling_failure(
                &paths.out_dir,
                &paths.script_result_path,
                "tooling.demo_exit.killed",
                "tool-launched demo did not exit cleanly (killed=true in resource.footprint.json)",
                "tooling_error",
                Some("stop_launched_demo".to_string()),
            );
            return Err("tool-launched demo did not exit cleanly (killed=true)".to_string());
        }
    }
    Ok(bundle_paths)
}

#[cfg(test)]
mod tests;
