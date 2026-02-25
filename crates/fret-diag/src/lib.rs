//! Diagnostics tooling for the Fret workspace.
//!
//! This crate is primarily used by `fretboard` to:
//! - run scripted UI interactions,
//! - capture diagnostics bundles (JSON + optional screenshots),
//! - compare runs and enforce performance/behavior gates.
//!
//! This is a tooling-focused crate (not a runtime dependency for apps).

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
pub mod artifacts;
mod bundle_index;
mod cli;
mod commands;
mod compare;
pub mod devtools;
mod diag_compare;
mod diag_matrix;
mod diag_perf;
mod diag_perf_baseline;
mod diag_policy;
mod diag_repeat;
mod diag_repro;
mod diag_run;
mod diag_simple_dispatch;
mod diag_stats;
mod diag_suite;
mod diag_suite_scripts;
mod evidence_index;
mod frames_index;
mod gates;
mod hotspots_lite;
mod json_bundle;
mod latest;
mod lint;
mod pack_zip;
mod paths;
mod perf_hint_gate;
mod perf_seed_policy;
mod post_run_checks;
mod run_artifacts;
mod script_tooling;
mod shrink;
mod stats;
mod suite_summary;
mod test_id_bloom;
mod tooling_failures;
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
    default_triage_out_path, expand_script_inputs, resolve_bundle_artifact_path,
    resolve_bundle_artifact_path_no_materialize, resolve_bundle_root_dir,
    resolve_bundle_schema2_artifact_path_no_materialize, resolve_path,
    resolve_raw_bundle_artifact_path_no_materialize, wait_for_bundle_artifact_from_script_result,
    wait_for_bundle_artifact_in_dir,
};

use compare::{
    CompareOptions, CompareReport, PerfThresholdAggregate, PerfThresholds, RenderdocDumpAttempt,
    apply_perf_baseline_floor, apply_perf_baseline_headroom, cargo_run_inject_feature,
    compare_bundles, ensure_env_var, find_latest_export_dir, maybe_launch_demo,
    normalize_repo_relative_path, read_latest_pointer, read_perf_baseline_file, resolve_threshold,
    run_fret_renderdoc_dump, scan_perf_threshold_failures, stop_launched_demo,
    wait_for_files_with_extensions,
};
use devtools::DevtoolsOps;
use gates::{
    RedrawHitchesGateResult, ResourceFootprintGateResult, ResourceFootprintThresholds,
    check_redraw_hitches_max_total_ms, check_resource_footprint_thresholds,
};
use lint::{LintOptions, lint_bundle_from_path};
use perf_seed_policy::{PerfBaselineSeed, PerfSeedMetric, ResolvedPerfBaselineSeedPolicy};
use run_artifacts::{
    refresh_run_id_manifest_file_index, run_id_artifact_dir, write_run_id_bundle_json,
    write_run_id_script_result,
};

use stats::{
    BundleStatsOptions, BundleStatsReport, BundleStatsSort, ScriptResultSummary,
    bundle_stats_diff_from_paths, bundle_stats_from_path,
    check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change,
    check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change,
    check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance,
    check_out_dir_for_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps,
    check_report_for_hover_layout_invalidations, clear_script_result_files, report_result_and_exit,
    run_script_and_wait, wait_for_failure_dump_bundle,
};
use tooling_failures::{
    mark_existing_script_result_tooling_failure, push_tooling_event_log_entry,
    write_tooling_failure_script_result, write_tooling_failure_script_result_if_missing,
};
use util::{now_unix_ms, read_json_value, touch, write_json_value, write_script};

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
    let mut out_dir: Option<PathBuf> = None;
    let mut trigger_path: Option<PathBuf> = None;
    let mut pack_out: Option<PathBuf> = None;
    let mut pack_include_root_artifacts: bool = false;
    let mut pack_include_triage: bool = false;
    let mut pack_include_screenshots: bool = false;
    let mut pack_after_run: bool = false;
    let mut pack_schema2_only: bool = false;
    let mut ensure_ai_packet: bool = false;
    let mut pack_ai_only: bool = false;
    let mut triage_out: Option<PathBuf> = None;
    let mut lint_out: Option<PathBuf> = None;
    let mut meta_out: Option<PathBuf> = None;
    let mut meta_report: bool = false;
    let mut index_out: Option<PathBuf> = None;
    let mut test_ids_out: Option<PathBuf> = None;
    let mut hotspots_out: Option<PathBuf> = None;
    let mut bundle_v2_out: Option<PathBuf> = None;
    let mut query_out: Option<PathBuf> = None;
    let mut slice_out: Option<PathBuf> = None;
    let mut ai_packet_out: Option<PathBuf> = None;
    let mut script_path: Option<PathBuf> = None;
    let mut script_trigger_path: Option<PathBuf> = None;
    let mut script_result_path: Option<PathBuf> = None;
    let mut script_result_trigger_path: Option<PathBuf> = None;
    let mut pick_trigger_path: Option<PathBuf> = None;
    let mut pick_result_path: Option<PathBuf> = None;
    let mut pick_result_trigger_path: Option<PathBuf> = None;
    let mut pick_script_out: Option<PathBuf> = None;
    let mut pick_apply_pointer: Option<String> = None;
    let mut pick_apply_out: Option<PathBuf> = None;
    let mut inspect_path: Option<PathBuf> = None;
    let mut inspect_trigger_path: Option<PathBuf> = None;
    let mut inspect_consume_clicks: Option<bool> = None;
    let mut timeout_ms: u64 = 240_000;
    let mut poll_ms: u64 = 50;
    let mut stats_top: usize = 5;
    let mut stats_verbose: bool = false;
    let mut sort_override: Option<BundleStatsSort> = None;
    let mut stats_json: bool = false;
    let mut stats_diff: Option<(PathBuf, PathBuf)> = None;
    let mut trace_chrome: bool = false;
    let mut trace_out: Option<PathBuf> = None;
    let mut warmup_frames: u64 = 0;
    let mut max_test_ids: usize = 200;
    let mut lint_all_test_ids_bounds: bool = false;
    let mut lint_eps_px: f32 = 0.5;
    let mut suite_lint: bool = true;
    let mut perf_repeat: u64 = 1;
    let mut reuse_launch: bool = false;
    let mut reuse_launch_per_script: bool = false;
    let mut launch_high_priority: bool = false;
    let mut keep_open: bool = false;
    let mut script_tool_write: bool = false;
    let mut script_tool_check: bool = false;
    let mut script_tool_check_out: Option<PathBuf> = None;
    let mut shrink_out: Option<PathBuf> = None;
    let mut shrink_any_fail: bool = false;
    let mut shrink_match_reason_code: Option<String> = None;
    let mut shrink_match_reason: Option<String> = None;
    let mut shrink_min_steps: u64 = 1;
    let mut shrink_max_iters: u64 = 200;
    let mut max_top_total_us: Option<u64> = None;
    let mut max_top_layout_us: Option<u64> = None;
    let mut max_top_solve_us: Option<u64> = None;
    let mut max_frame_p95_total_us: Option<u64> = None;
    let mut max_frame_p95_layout_us: Option<u64> = None;
    let mut max_frame_p95_solve_us: Option<u64> = None;
    let mut max_pointer_move_dispatch_us: Option<u64> = None;
    let mut max_pointer_move_hit_test_us: Option<u64> = None;
    let mut max_pointer_move_global_changes: Option<u64> = None;
    let mut min_run_paint_cache_hit_test_only_replay_allowed_max: Option<u64> = None;
    let mut max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Option<u64> = None;
    let mut check_perf_hints: bool = false;
    let mut check_perf_hints_deny: Vec<String> = Vec::new();
    let mut check_perf_hints_min_severity: Option<String> = None;
    let mut perf_threshold_agg: PerfThresholdAggregate = PerfThresholdAggregate::Max;
    let mut max_working_set_bytes: Option<u64> = None;
    let mut max_peak_working_set_bytes: Option<u64> = None;
    let mut max_cpu_avg_percent_total_cores: Option<f64> = None;
    let mut perf_baseline_path: Option<PathBuf> = None;
    let mut perf_baseline_out: Option<PathBuf> = None;
    let mut perf_baseline_headroom_pct: u32 = 20;
    let mut perf_baseline_seed_preset_paths: Vec<PathBuf> = Vec::new();
    let mut perf_baseline_seed_specs: Vec<String> = Vec::new();
    let mut check_idle_no_paint_min: Option<u64> = None;
    let mut check_stale_paint_test_id: Option<String> = None;
    let mut check_stale_paint_eps: f32 = 0.5;
    let mut check_stale_scene_test_id: Option<String> = None;
    let mut check_stale_scene_eps: f32 = 0.5;
    let mut check_pixels_changed_test_id: Option<String> = None;
    let mut check_ui_gallery_code_editor_torture_marker_present: bool = false;
    let mut check_ui_gallery_code_editor_torture_undo_redo: bool = false;
    let mut check_ui_gallery_code_editor_torture_geom_fallbacks_low: bool = false;
    let mut check_ui_gallery_code_editor_torture_read_only_blocks_edits: bool = false;
    let mut check_ui_gallery_markdown_editor_source_read_only_blocks_edits: bool = false;
    let mut check_ui_gallery_markdown_editor_source_disabled_blocks_edits: bool = false;
    let mut check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable: bool = false;
    let mut check_ui_gallery_markdown_editor_source_word_boundary: bool = false;
    let mut check_ui_gallery_web_ime_bridge_enabled: bool = false;
    let mut check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps: bool = false;
    let mut check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change: bool = false;
    let mut check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change: bool = false;
    let mut check_ui_gallery_text_mixed_script_bundled_fallback_conformance: bool = false;
    let mut check_ui_gallery_markdown_editor_source_line_boundary_triple_click: bool = false;
    let mut check_ui_gallery_markdown_editor_source_a11y_composition: bool = false;
    let mut check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap: bool = false;
    let mut check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable: bool =
        false;
    let mut check_ui_gallery_markdown_editor_source_folds_toggle_stable: bool = false;
    let mut check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds: bool =
        false;
    let mut check_ui_gallery_markdown_editor_source_folds_placeholder_present: bool = false;
    let mut check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap: bool =
        false;
    let mut check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit: bool =
        false;
    let mut check_ui_gallery_markdown_editor_source_inlays_toggle_stable: bool = false;
    let mut check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable: bool = false;
    let mut check_ui_gallery_markdown_editor_source_inlays_present: bool = false;
    let mut check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap: bool = false;
    let mut check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_folds_placeholder_present: bool = false;
    let mut check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_inlays_present: bool = false;
    let mut check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit: bool = false;
    let mut check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed: bool =
        false;
    let mut check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap: bool = false;
    let mut check_ui_gallery_code_editor_word_boundary: bool = false;
    let mut check_ui_gallery_code_editor_a11y_selection: bool = false;
    let mut check_ui_gallery_code_editor_a11y_composition: bool = false;
    let mut check_ui_gallery_code_editor_a11y_selection_wrap: bool = false;
    let mut check_ui_gallery_code_editor_a11y_composition_wrap: bool = false;
    let mut check_ui_gallery_code_editor_a11y_composition_wrap_scroll: bool = false;
    let mut check_ui_gallery_code_editor_a11y_composition_drag: bool = false;
    let mut check_semantics_changed_repainted: bool = false;
    let mut dump_semantics_changed_repainted_json: bool = false;
    let mut check_wheel_scroll_test_id: Option<String> = None;
    let mut check_wheel_scroll_hit_changes_test_id: Option<String> = None;
    let mut check_drag_cache_root_paint_only_test_id: Option<String> = None;
    let mut check_hover_layout_max: Option<u32> = None;
    let mut check_prepaint_actions_min: Option<u64> = None;
    let mut check_chart_sampling_window_shifts_min: Option<u64> = None;
    let mut check_node_graph_cull_window_shifts_min: Option<u64> = None;
    let mut check_node_graph_cull_window_shifts_max: Option<u64> = None;
    let mut check_vlist_visible_range_refreshes_min: Option<u64> = None;
    let mut check_vlist_visible_range_refreshes_max: Option<u64> = None;
    let mut check_vlist_window_shifts_explainable: bool = false;
    let mut check_vlist_window_shifts_have_prepaint_actions: bool = false;
    let mut check_vlist_window_shifts_non_retained_max: Option<u64> = None;
    let mut check_vlist_window_shifts_prefetch_max: Option<u64> = None;
    let mut check_vlist_window_shifts_escape_max: Option<u64> = None;
    let mut check_vlist_policy_key_stable: bool = false;
    let mut check_windowed_rows_offset_changes_min: Option<u64> = None;
    let mut check_windowed_rows_offset_changes_eps: f32 = 0.5;
    let mut check_windowed_rows_visible_start_changes_repainted: bool = false;
    let mut check_layout_fast_path_min: Option<u64> = None;
    let mut check_gc_sweep_liveness: bool = false;
    let mut check_notify_hotspot_file_max: Vec<(String, u64)> = Vec::new();
    let mut check_view_cache_reuse_min: Option<u64> = None;
    let mut check_view_cache_reuse_stable_min: Option<u64> = None;
    let mut check_redraw_hitches_max_total_ms_threshold: Option<u64> = None;
    let mut check_overlay_synthesis_min: Option<u64> = None;
    let mut check_viewport_input_min: Option<u64> = None;
    let mut check_dock_drag_min: Option<u64> = None;
    let mut check_viewport_capture_min: Option<u64> = None;
    let mut check_retained_vlist_reconcile_no_notify_min: Option<u64> = None;
    let mut check_retained_vlist_attach_detach_max: Option<u64> = None;
    let mut check_retained_vlist_keep_alive_reuse_min: Option<u64> = None;
    let mut check_retained_vlist_keep_alive_budget: Option<(u64, u64)> = None;
    let mut compare_eps_px: f32 = 0.5;
    let mut compare_ignore_bounds: bool = false;
    let mut compare_ignore_scene_fingerprint: bool = false;
    let mut launch: Option<Vec<String>> = None;
    let mut launch_env: Vec<(String, String)> = Vec::new();
    let mut fixed_frame_delta_ms: Option<u64> = None;
    let mut with_tracy: bool = false;
    let mut with_renderdoc: bool = false;
    let mut renderdoc_after_frames: Option<u32> = None;
    let mut renderdoc_markers: Vec<String> = Vec::new();
    let mut renderdoc_no_outputs_png: bool = false;
    let mut devtools_ws_url: Option<String> = None;
    let mut devtools_token: Option<String> = None;
    let mut devtools_session_id: Option<String> = None;
    let mut exit_after_run: bool = false;
    let mut suite_script_inputs: Vec<String> = Vec::new();
    let mut suite_prewarm_scripts: Vec<PathBuf> = Vec::new();
    let mut suite_prelude_scripts: Vec<PathBuf> = Vec::new();
    let mut suite_prelude_each_run: bool = false;

    fn push_env_if_missing(env: &mut Vec<(String, String)>, key: &str, value: &str) {
        if env.iter().any(|(k, _v)| k == key) {
            return;
        }
        env.push((key.to_string(), value.to_string()));
    }

    // Parse global `diag` flags regardless of their position, leaving positional args intact.
    // This keeps the behavior aligned with the help text in `apps/fretboard/src/cli.rs`.
    let mut positionals: Vec<String> = Vec::new();
    let mut i: usize = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--dir" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --dir".to_string());
                };
                out_dir = Some(PathBuf::from(v));
                i += 1;
            }
            "--trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --trigger-path".to_string());
                };
                trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--pack-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pack-out".to_string());
                };
                pack_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--packet-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --packet-out".to_string());
                };
                ai_packet_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--ai-packet" => {
                ensure_ai_packet = true;
                i += 1;
            }
            "--ai-only" => {
                pack_ai_only = true;
                ensure_ai_packet = true;
                i += 1;
            }
            "--include-root-artifacts" => {
                pack_include_root_artifacts = true;
                i += 1;
            }
            "--include-all" => {
                pack_include_root_artifacts = true;
                pack_include_triage = true;
                pack_include_screenshots = true;
                i += 1;
            }
            "--include-triage" => {
                pack_include_triage = true;
                i += 1;
            }
            "--include-screenshots" => {
                pack_include_screenshots = true;
                i += 1;
            }
            "--pack-schema2-only" | "--schema2-only" => {
                pack_schema2_only = true;
                i += 1;
            }
            "--pack" => {
                pack_after_run = true;
                i += 1;
            }
            "--script-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --script-path".to_string());
                };
                script_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--script-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --script-trigger-path".to_string());
                };
                script_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--script-result-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --script-result-path".to_string());
                };
                script_result_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--script-result-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --script-result-trigger-path".to_string());
                };
                script_result_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--write" => {
                script_tool_write = true;
                i += 1;
            }
            "--check" => {
                script_tool_check = true;
                i += 1;
            }
            "--check-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-out".to_string());
                };
                script_tool_check_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--shrink-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --shrink-out".to_string());
                };
                shrink_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--shrink-any-fail" => {
                shrink_any_fail = true;
                i += 1;
            }
            "--shrink-match-reason-code" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --shrink-match-reason-code".to_string());
                };
                shrink_match_reason_code = Some(v);
                i += 1;
            }
            "--shrink-match-reason" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --shrink-match-reason".to_string());
                };
                shrink_match_reason = Some(v);
                i += 1;
            }
            "--shrink-min-steps" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --shrink-min-steps".to_string());
                };
                shrink_min_steps = v.parse::<u64>().map_err(|_| {
                    "invalid value for --shrink-min-steps (expected u64)".to_string()
                })?;
                i += 1;
            }
            "--shrink-max-iters" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --shrink-max-iters".to_string());
                };
                shrink_max_iters = v.parse::<u64>().map_err(|_| {
                    "invalid value for --shrink-max-iters (expected u64)".to_string()
                })?;
                i += 1;
            }
            "--devtools-ws-url" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --devtools-ws-url".to_string());
                };
                devtools_ws_url = Some(v);
                i += 1;
            }
            "--devtools-token" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --devtools-token".to_string());
                };
                devtools_token = Some(v);
                i += 1;
            }
            "--devtools-session-id" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --devtools-session-id".to_string());
                };
                devtools_session_id = Some(v);
                i += 1;
            }
            "--exit-after-run" | "--touch-exit-after-run" => {
                exit_after_run = true;
                i += 1;
            }
            "--script-dir" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --script-dir".to_string());
                };
                suite_script_inputs.push(v);
                i += 1;
            }
            "--glob" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --glob".to_string());
                };
                suite_script_inputs.push(v);
                i += 1;
            }
            "--suite-prewarm" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --suite-prewarm".to_string());
                };
                suite_prewarm_scripts.push(PathBuf::from(v));
                i += 1;
            }
            "--suite-prelude" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --suite-prelude".to_string());
                };
                suite_prelude_scripts.push(PathBuf::from(v));
                i += 1;
            }
            "--suite-prelude-each-run" => {
                suite_prelude_each_run = true;
                i += 1;
            }
            "--pick-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pick-trigger-path".to_string());
                };
                pick_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--pick-result-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pick-result-path".to_string());
                };
                pick_result_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--pick-result-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pick-result-trigger-path".to_string());
                };
                pick_result_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--pick-script-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --pick-script-out".to_string());
                };
                pick_script_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--ptr" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --ptr".to_string());
                };
                pick_apply_pointer = Some(v);
                i += 1;
            }
            "--out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --out".to_string());
                };
                let p = PathBuf::from(v);
                pick_apply_out = Some(p.clone());
                triage_out = Some(p.clone());
                lint_out = Some(p.clone());
                meta_out = Some(p.clone());
                index_out = Some(p.clone());
                hotspots_out = Some(p.clone());
                bundle_v2_out = Some(p.clone());
                query_out = Some(p.clone());
                slice_out = Some(p.clone());
                ai_packet_out = Some(p.clone());
                test_ids_out = Some(p);
                i += 1;
            }
            "--max-test-ids" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-test-ids".to_string());
                };
                max_test_ids = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --max-test-ids (expected usize)".to_string())?;
                i += 1;
            }
            "--all-test-ids" => {
                lint_all_test_ids_bounds = true;
                i += 1;
            }
            "--lint-eps-px" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --lint-eps-px".to_string());
                };
                lint_eps_px = v
                    .parse::<f32>()
                    .map_err(|_| "invalid value for --lint-eps-px".to_string())?;
                i += 1;
            }
            "--no-lint" => {
                suite_lint = false;
                i += 1;
            }
            "--lint" => {
                suite_lint = true;
                i += 1;
            }
            "--inspect-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --inspect-path".to_string());
                };
                inspect_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--inspect-trigger-path" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --inspect-trigger-path".to_string());
                };
                inspect_trigger_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--consume-clicks" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --consume-clicks".to_string());
                };
                inspect_consume_clicks = Some(
                    parse_bool(&v).map_err(|_| "invalid value for --consume-clicks".to_string())?,
                );
                i += 1;
            }
            "--timeout-ms" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --timeout-ms".to_string());
                };
                timeout_ms = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --timeout-ms".to_string())?;
                i += 1;
            }
            "--poll-ms" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --poll-ms".to_string());
                };
                poll_ms = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --poll-ms".to_string())?;
                i += 1;
            }
            "--sort" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --sort".to_string());
                };
                sort_override = Some(BundleStatsSort::parse(&v)?);
                i += 1;
            }
            "--trace" => {
                trace_chrome = true;
                i += 1;
            }
            "--trace-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --trace-out".to_string());
                };
                trace_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--diff" => {
                i += 1;
                let Some(a) = args.get(i).cloned() else {
                    return Err("missing bundle artifact path a for --diff".to_string());
                };
                i += 1;
                let Some(b) = args.get(i).cloned() else {
                    return Err("missing bundle artifact path b for --diff".to_string());
                };
                stats_diff = Some((PathBuf::from(a), PathBuf::from(b)));
                i += 1;
            }
            "--top" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --top".to_string());
                };
                stats_top = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --top".to_string())?;
                i += 1;
            }
            "--warmup-frames" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --warmup-frames".to_string());
                };
                warmup_frames = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --warmup-frames".to_string())?;
                i += 1;
            }
            "--repeat" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --repeat".to_string());
                };
                perf_repeat = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --repeat".to_string())?
                    .max(1);
                i += 1;
            }
            "--max-top-total-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-top-total-us".to_string());
                };
                max_top_total_us = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-top-total-us".to_string())?,
                );
                i += 1;
            }
            "--max-top-layout-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-top-layout-us".to_string());
                };
                max_top_layout_us = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-top-layout-us".to_string())?,
                );
                i += 1;
            }
            "--max-top-solve-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-top-solve-us".to_string());
                };
                max_top_solve_us = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-top-solve-us".to_string())?,
                );
                i += 1;
            }
            "--max-frame-p95-total-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-frame-p95-total-us".to_string());
                };
                max_frame_p95_total_us = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-frame-p95-total-us".to_string())?,
                );
                i += 1;
            }
            "--max-frame-p95-layout-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-frame-p95-layout-us".to_string());
                };
                max_frame_p95_layout_us = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-frame-p95-layout-us".to_string())?,
                );
                i += 1;
            }
            "--max-frame-p95-solve-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-frame-p95-solve-us".to_string());
                };
                max_frame_p95_solve_us = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-frame-p95-solve-us".to_string())?,
                );
                i += 1;
            }
            "--max-pointer-move-dispatch-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-pointer-move-dispatch-us".to_string());
                };
                max_pointer_move_dispatch_us =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --max-pointer-move-dispatch-us".to_string()
                    })?);
                i += 1;
            }
            "--max-pointer-move-hit-test-us" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-pointer-move-hit-test-us".to_string());
                };
                max_pointer_move_hit_test_us =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --max-pointer-move-hit-test-us".to_string()
                    })?);
                i += 1;
            }
            "--max-pointer-move-global-changes" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-pointer-move-global-changes".to_string());
                };
                max_pointer_move_global_changes = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --max-pointer-move-global-changes".to_string()
                })?);
                i += 1;
            }
            "--min-run-paint-cache-hit-test-only-replay-allowed-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --min-run-paint-cache-hit-test-only-replay-allowed-max"
                            .to_string(),
                    );
                };
                min_run_paint_cache_hit_test_only_replay_allowed_max =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --min-run-paint-cache-hit-test-only-replay-allowed-max"
                            .to_string()
                    })?);
                i += 1;
            }
            "--max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max"
                            .to_string(),
                    );
                };
                max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max"
                            .to_string()
                    })?);
                i += 1;
            }
            "--check-perf-hints" => {
                check_perf_hints = true;
                i += 1;
            }
            "--check-perf-hints-deny" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-perf-hints-deny".to_string());
                };
                check_perf_hints_deny.push(v);
                check_perf_hints = true;
                i += 1;
            }
            "--check-perf-hints-min-severity" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-perf-hints-min-severity".to_string());
                };
                check_perf_hints_min_severity = Some(v);
                check_perf_hints = true;
                i += 1;
            }
            "--max-working-set-bytes" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-working-set-bytes".to_string());
                };
                max_working_set_bytes = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --max-working-set-bytes".to_string())?,
                );
                i += 1;
            }
            "--max-peak-working-set-bytes" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-peak-working-set-bytes".to_string());
                };
                max_peak_working_set_bytes =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --max-peak-working-set-bytes".to_string()
                    })?);
                i += 1;
            }
            "--max-cpu-avg-percent-total-cores" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --max-cpu-avg-percent-total-cores".to_string());
                };
                let pct = v.parse::<f64>().map_err(|_| {
                    "invalid value for --max-cpu-avg-percent-total-cores".to_string()
                })?;
                if pct < 0.0 {
                    return Err("invalid value for --max-cpu-avg-percent-total-cores".to_string());
                }
                max_cpu_avg_percent_total_cores = Some(pct);
                i += 1;
            }
            "--perf-threshold-agg" | "--perf-threshold-aggregate" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --perf-threshold-agg".to_string());
                };
                perf_threshold_agg = v
                    .parse::<PerfThresholdAggregate>()
                    .map_err(|e| format!("invalid value for --perf-threshold-agg: {e}"))?;
                i += 1;
            }
            "--perf-baseline" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --perf-baseline".to_string());
                };
                perf_baseline_path = Some(PathBuf::from(v));
                i += 1;
            }
            "--perf-baseline-out" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --perf-baseline-out".to_string());
                };
                perf_baseline_out = Some(PathBuf::from(v));
                i += 1;
            }
            "--perf-baseline-headroom-pct" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --perf-baseline-headroom-pct".to_string());
                };
                perf_baseline_headroom_pct = v
                    .parse::<u32>()
                    .map_err(|_| "invalid value for --perf-baseline-headroom-pct".to_string())?;
                i += 1;
            }
            "--perf-baseline-seed-preset" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --perf-baseline-seed-preset".to_string());
                };
                perf_baseline_seed_preset_paths.push(PathBuf::from(v));
                i += 1;
            }
            "--perf-baseline-seed" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --perf-baseline-seed".to_string());
                };
                perf_baseline_seed_specs.push(v);
                i += 1;
            }
            "--check-idle-no-paint-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-idle-no-paint-min".to_string());
                };
                check_idle_no_paint_min = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --check-idle-no-paint-min".to_string())?,
                );
                i += 1;
            }
            "--check-stale-paint" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-stale-paint".to_string());
                };
                check_stale_paint_test_id = Some(v);
                i += 1;
            }
            "--check-stale-paint-eps" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-stale-paint-eps".to_string());
                };
                check_stale_paint_eps = v
                    .parse::<f32>()
                    .map_err(|_| "invalid value for --check-stale-paint-eps".to_string())?;
                i += 1;
            }
            "--check-stale-scene" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-stale-scene".to_string());
                };
                check_stale_scene_test_id = Some(v);
                i += 1;
            }
            "--check-stale-scene-eps" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-stale-scene-eps".to_string());
                };
                check_stale_scene_eps = v
                    .parse::<f32>()
                    .map_err(|_| "invalid value for --check-stale-scene-eps".to_string())?;
                i += 1;
            }
            "--check-pixels-changed" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-pixels-changed".to_string());
                };
                check_pixels_changed_test_id = Some(v);
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-marker-present" => {
                check_ui_gallery_code_editor_torture_marker_present = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-undo-redo" => {
                check_ui_gallery_code_editor_torture_undo_redo = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-geom-fallbacks-low" => {
                check_ui_gallery_code_editor_torture_geom_fallbacks_low = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-read-only-blocks-edits" => {
                check_ui_gallery_code_editor_torture_read_only_blocks_edits = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-read-only-blocks-edits" => {
                check_ui_gallery_markdown_editor_source_read_only_blocks_edits = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-disabled-blocks-edits" => {
                check_ui_gallery_markdown_editor_source_disabled_blocks_edits = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-soft-wrap-toggle-stable" => {
                check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-word-boundary" => {
                check_ui_gallery_markdown_editor_source_word_boundary = true;
                i += 1;
            }
            "--check-ui-gallery-web-ime-bridge-enabled" => {
                check_ui_gallery_web_ime_bridge_enabled = true;
                i += 1;
            }
            "--check-ui-gallery-text-rescan-system-fonts-font-stack-key-bumps" => {
                check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps = true;
                i += 1;
            }
            "--check-ui-gallery-text-fallback-policy-key-bumps-on-settings-change" => {
                check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change = true;
                i += 1;
            }
            "--check-ui-gallery-text-fallback-policy-key-bumps-on-locale-change" => {
                check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change = true;
                i += 1;
            }
            "--check-ui-gallery-text-mixed-script-bundled-fallback-conformance" => {
                check_ui_gallery_text_mixed_script_bundled_fallback_conformance = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-line-boundary-triple-click" => {
                check_ui_gallery_markdown_editor_source_line_boundary_triple_click = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-a11y-composition" => {
                check_ui_gallery_markdown_editor_source_a11y_composition = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-a11y-composition-soft-wrap" => {
                check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-soft-wrap-editing-selection-wrap-stable" => {
                check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable =
                    true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-folds-toggle-stable" => {
                check_ui_gallery_markdown_editor_source_folds_toggle_stable = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-folds-clamp-selection-out-of-folds" => {
                check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-folds-placeholder-present" => {
                check_ui_gallery_markdown_editor_source_folds_placeholder_present = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-folds-placeholder-present-under-soft-wrap" =>
            {
                check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap =
                    true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-folds-placeholder-absent-under-inline-preedit" =>
            {
                check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit =
                    true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-inlays-toggle-stable" => {
                check_ui_gallery_markdown_editor_source_inlays_toggle_stable = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-inlays-caret-navigation-stable" => {
                check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-inlays-present" => {
                check_ui_gallery_markdown_editor_source_inlays_present = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-inlays-present-under-soft-wrap" => {
                check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap = true;
                i += 1;
            }
            "--check-ui-gallery-markdown-editor-source-inlays-absent-under-inline-preedit" => {
                check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-folds-placeholder-absent-under-inline-preedit" =>
            {
                check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-folds-placeholder-present-under-inline-preedit-unwrapped" =>
            {
                check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-folds-placeholder-present-under-inline-preedit-with-decorations" =>
            {
                check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-folds-placeholder-present-under-inline-preedit-with-decorations-composed" =>
            {
                check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-decorations-toggle-stable-under-inline-preedit-composed" =>
            {
                check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-decorations-toggle-a11y-composition-consistent-under-inline-preedit-composed" =>
            {
                check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-composed-preedit-stable-after-wheel-scroll" => {
                check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-composed-preedit-cancels-on-drag-selection" => {
                check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-folds-placeholder-present" => {
                check_ui_gallery_code_editor_torture_folds_placeholder_present = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-folds-placeholder-present-under-soft-wrap" => {
                check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-inlays-present" => {
                check_ui_gallery_code_editor_torture_inlays_present = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-inlays-absent-under-inline-preedit" => {
                check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-inlays-present-under-inline-preedit-unwrapped" =>
            {
                check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-inlays-present-under-inline-preedit-with-decorations" =>
            {
                check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-inlays-present-under-inline-preedit-with-decorations-composed" =>
            {
                check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed =
                    true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-torture-inlays-present-under-soft-wrap" => {
                check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-word-boundary" => {
                check_ui_gallery_code_editor_word_boundary = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-a11y-selection" => {
                check_ui_gallery_code_editor_a11y_selection = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-a11y-composition" => {
                check_ui_gallery_code_editor_a11y_composition = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-a11y-selection-wrap" => {
                check_ui_gallery_code_editor_a11y_selection_wrap = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-a11y-composition-wrap" => {
                check_ui_gallery_code_editor_a11y_composition_wrap = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-a11y-composition-wrap-scroll" => {
                check_ui_gallery_code_editor_a11y_composition_wrap_scroll = true;
                i += 1;
            }
            "--check-ui-gallery-code-editor-a11y-composition-drag" => {
                check_ui_gallery_code_editor_a11y_composition_drag = true;
                i += 1;
            }
            "--check-semantics-changed-repainted" => {
                check_semantics_changed_repainted = true;
                i += 1;
            }
            "--dump-semantics-changed-repainted-json" => {
                check_semantics_changed_repainted = true;
                dump_semantics_changed_repainted_json = true;
                i += 1;
            }
            "--check-wheel-scroll" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-wheel-scroll".to_string());
                };
                check_wheel_scroll_test_id = Some(v);
                i += 1;
            }
            "--check-wheel-scroll-hit-changes" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-wheel-scroll-hit-changes".to_string());
                };
                check_wheel_scroll_hit_changes_test_id = Some(v);
                i += 1;
            }
            "--check-vlist-visible-range-refreshes-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-vlist-visible-range-refreshes-max".to_string(),
                    );
                };
                check_vlist_visible_range_refreshes_max = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-vlist-visible-range-refreshes-max".to_string()
                })?);
                i += 1;
            }
            "--check-vlist-visible-range-refreshes-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-vlist-visible-range-refreshes-min".to_string(),
                    );
                };
                check_vlist_visible_range_refreshes_min = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-vlist-visible-range-refreshes-min".to_string()
                })?);
                i += 1;
            }
            "--check-vlist-window-shifts-explainable" => {
                check_vlist_window_shifts_explainable = true;
                i += 1;
            }
            "--check-vlist-window-shifts-have-prepaint-actions" => {
                check_vlist_window_shifts_have_prepaint_actions = true;
                i += 1;
            }
            "--check-vlist-window-shifts-non-retained-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-vlist-window-shifts-non-retained-max"
                            .to_string(),
                    );
                };
                check_vlist_window_shifts_non_retained_max =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-vlist-window-shifts-non-retained-max".to_string()
                    })?);
                i += 1;
            }
            "--check-vlist-window-shifts-prefetch-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-vlist-window-shifts-prefetch-max".to_string()
                    );
                };
                check_vlist_window_shifts_prefetch_max = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-vlist-window-shifts-prefetch-max".to_string()
                })?);
                i += 1;
            }
            "--check-vlist-window-shifts-escape-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-vlist-window-shifts-escape-max".to_string()
                    );
                };
                check_vlist_window_shifts_escape_max = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-vlist-window-shifts-escape-max".to_string()
                })?);
                i += 1;
            }
            "--check-vlist-policy-key-stable" => {
                check_vlist_policy_key_stable = true;
                i += 1;
            }
            "--check-windowed-rows-offset-changes-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-windowed-rows-offset-changes-min".to_string()
                    );
                };
                check_windowed_rows_offset_changes_min = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-windowed-rows-offset-changes-min".to_string()
                })?);
                i += 1;
            }
            "--check-windowed-rows-offset-changes-eps" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-windowed-rows-offset-changes-eps".to_string()
                    );
                };
                check_windowed_rows_offset_changes_eps = v.parse::<f32>().map_err(|_| {
                    "invalid value for --check-windowed-rows-offset-changes-eps".to_string()
                })?;
                i += 1;
            }
            "--check-windowed-rows-visible-start-changes-repainted" => {
                check_windowed_rows_visible_start_changes_repainted = true;
                i += 1;
            }
            "--check-layout-fast-path-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-layout-fast-path-min".to_string());
                };
                check_layout_fast_path_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-layout-fast-path-min".to_string()
                    })?);
                i += 1;
            }
            "--check-prepaint-actions-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-prepaint-actions-min".to_string());
                };
                check_prepaint_actions_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-prepaint-actions-min".to_string()
                    })?);
                i += 1;
            }
            "--check-chart-sampling-window-shifts-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-chart-sampling-window-shifts-min".to_string()
                    );
                };
                check_chart_sampling_window_shifts_min = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-chart-sampling-window-shifts-min".to_string()
                })?);
                i += 1;
            }
            "--check-node-graph-cull-window-shifts-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-node-graph-cull-window-shifts-min".to_string(),
                    );
                };
                check_node_graph_cull_window_shifts_min = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-node-graph-cull-window-shifts-min".to_string()
                })?);
                i += 1;
            }
            "--check-node-graph-cull-window-shifts-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-node-graph-cull-window-shifts-max".to_string(),
                    );
                };
                check_node_graph_cull_window_shifts_max = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-node-graph-cull-window-shifts-max".to_string()
                })?);
                i += 1;
            }
            "--check-drag-cache-root-paint-only" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-drag-cache-root-paint-only".to_string());
                };
                check_drag_cache_root_paint_only_test_id = Some(v);
                i += 1;
            }
            "--check-hover-layout" => {
                check_hover_layout_max = Some(0);
                i += 1;
            }
            "--check-hover-layout-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-hover-layout-max".to_string());
                };
                check_hover_layout_max = Some(
                    v.parse::<u32>()
                        .map_err(|_| "invalid value for --check-hover-layout-max".to_string())?,
                );
                i += 1;
            }
            "--check-gc-sweep-liveness" => {
                check_gc_sweep_liveness = true;
                i += 1;
            }
            "--check-notify-hotspot-file-max" => {
                i += 1;
                let Some(file) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-notify-hotspot-file-max (file)".to_string()
                    );
                };
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-notify-hotspot-file-max (max)".to_string()
                    );
                };
                let max = v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-notify-hotspot-file-max (max)".to_string()
                })?;
                check_notify_hotspot_file_max.push((file, max));
                i += 1;
            }
            "--check-view-cache-reuse-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-view-cache-reuse-min".to_string());
                };
                check_view_cache_reuse_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-view-cache-reuse-min".to_string()
                    })?);
                i += 1;
            }
            "--check-view-cache-reuse-stable-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-view-cache-reuse-stable-min".to_string());
                };
                check_view_cache_reuse_stable_min = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-view-cache-reuse-stable-min".to_string()
                })?);
                i += 1;
            }
            "--check-redraw-hitches-max-total-ms" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-redraw-hitches-max-total-ms".to_string());
                };
                check_redraw_hitches_max_total_ms_threshold =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-redraw-hitches-max-total-ms".to_string()
                    })?);
                i += 1;
            }
            "--check-overlay-synthesis-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-overlay-synthesis-min".to_string());
                };
                check_overlay_synthesis_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-overlay-synthesis-min".to_string()
                    })?);
                i += 1;
            }
            "--check-viewport-input-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-viewport-input-min".to_string());
                };
                check_viewport_input_min = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --check-viewport-input-min".to_string())?,
                );
                i += 1;
            }
            "--check-dock-drag-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-dock-drag-min".to_string());
                };
                check_dock_drag_min = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --check-dock-drag-min".to_string())?,
                );
                i += 1;
            }
            "--check-viewport-capture-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --check-viewport-capture-min".to_string());
                };
                check_viewport_capture_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-viewport-capture-min".to_string()
                    })?);
                i += 1;
            }
            "--check-retained-vlist-reconcile-no-notify" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-retained-vlist-reconcile-no-notify".to_string(),
                    );
                };
                check_retained_vlist_reconcile_no_notify_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-retained-vlist-reconcile-no-notify".to_string()
                    })?);
                i += 1;
            }
            "--check-retained-vlist-attach-detach-max" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-retained-vlist-attach-detach-max".to_string()
                    );
                };
                check_retained_vlist_attach_detach_max = Some(v.parse::<u64>().map_err(|_| {
                    "invalid value for --check-retained-vlist-attach-detach-max".to_string()
                })?);
                i += 1;
            }
            "--check-retained-vlist-keep-alive-reuse-min" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err(
                        "missing value for --check-retained-vlist-keep-alive-reuse-min".to_string(),
                    );
                };
                check_retained_vlist_keep_alive_reuse_min =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --check-retained-vlist-keep-alive-reuse-min".to_string()
                    })?);
                i += 1;
            }
            "--check-retained-vlist-keep-alive-budget" => {
                i += 1;
                let Some(min_max_pool_len_after) = args.get(i).cloned() else {
                    return Err("missing value for --check-retained-vlist-keep-alive-budget (expected MIN_MAX_POOL_LEN_AFTER)".to_string());
                };
                let min_max_pool_len_after =
                    min_max_pool_len_after.parse::<u64>().map_err(|_| {
                        "invalid value for --check-retained-vlist-keep-alive-budget (expected MIN_MAX_POOL_LEN_AFTER)".to_string()
                    })?;

                i += 1;
                let Some(max_total_evicted_items) = args.get(i).cloned() else {
                    return Err("missing value for --check-retained-vlist-keep-alive-budget (expected MAX_TOTAL_EVICTED_ITEMS)".to_string());
                };
                let max_total_evicted_items =
                    max_total_evicted_items.parse::<u64>().map_err(|_| {
                        "invalid value for --check-retained-vlist-keep-alive-budget (expected MAX_TOTAL_EVICTED_ITEMS)".to_string()
                    })?;
                check_retained_vlist_keep_alive_budget =
                    Some((min_max_pool_len_after, max_total_evicted_items));
                i += 1;
            }
            "--compare-eps-px" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --compare-eps-px".to_string());
                };
                compare_eps_px = v
                    .parse::<f32>()
                    .map_err(|_| "invalid value for --compare-eps-px".to_string())?;
                i += 1;
            }
            "--compare-ignore-bounds" => {
                compare_ignore_bounds = true;
                i += 1;
            }
            "--compare-ignore-scene-fingerprint" => {
                compare_ignore_scene_fingerprint = true;
                i += 1;
            }
            "--json" => {
                stats_json = true;
                i += 1;
            }
            "--meta-report" => {
                meta_report = true;
                i += 1;
            }
            "--verbose" => {
                stats_verbose = true;
                i += 1;
            }
            "--with" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --with (expected tracy|renderdoc)".to_string());
                };
                match v.as_str() {
                    "tracy" => with_tracy = true,
                    "renderdoc" => with_renderdoc = true,
                    other => {
                        return Err(format!(
                            "invalid value for --with: {other} (expected tracy|renderdoc)"
                        ));
                    }
                }
                i += 1;
            }
            "--with-tracy" => {
                with_tracy = true;
                i += 1;
            }
            "--with-renderdoc" => {
                with_renderdoc = true;
                i += 1;
            }
            "--renderdoc-after-frames" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --renderdoc-after-frames".to_string());
                };
                let parsed = v
                    .parse::<u32>()
                    .map_err(|_| "invalid value for --renderdoc-after-frames".to_string())?;
                if parsed == 0 {
                    return Err(
                        "invalid value for --renderdoc-after-frames (must be > 0)".to_string()
                    );
                }
                renderdoc_after_frames = Some(parsed);
                i += 1;
            }
            "--renderdoc-marker" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --renderdoc-marker".to_string());
                };
                renderdoc_markers.push(v);
                i += 1;
            }
            "--renderdoc-no-outputs-png" => {
                renderdoc_no_outputs_png = true;
                i += 1;
            }
            "--fixed-frame-delta-ms" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --fixed-frame-delta-ms".to_string());
                };
                let parsed = v
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --fixed-frame-delta-ms".to_string())?;
                if parsed == 0 {
                    return Err(
                        "invalid value for --fixed-frame-delta-ms (must be > 0)".to_string()
                    );
                }
                fixed_frame_delta_ms = Some(parsed);
                i += 1;
            }
            "--env" => {
                i += 1;
                let Some(v) = args.get(i).cloned() else {
                    return Err("missing value for --env (expected KEY=VALUE)".to_string());
                };
                let (key, value) = v
                    .split_once('=')
                    .ok_or_else(|| "invalid value for --env (expected KEY=VALUE)".to_string())?;
                let key = key.trim();
                if key.is_empty() {
                    return Err("invalid value for --env (empty KEY)".to_string());
                }
                launch_env.push((key.to_string(), value.to_string()));
                i += 1;
            }
            "--reuse-launch" => {
                reuse_launch = true;
                i += 1;
            }
            "--reuse-launch-per-script" => {
                reuse_launch_per_script = true;
                i += 1;
            }
            "--launch-high-priority" => {
                launch_high_priority = true;
                i += 1;
            }
            "--keep-open" => {
                keep_open = true;
                i += 1;
            }
            "--launch" => {
                i += 1;
                let launch_args = args.get(i..).unwrap_or_default();
                if launch_args.is_empty() {
                    return Err("missing command after --launch (try: --launch -- cargo run -p fret-demo --bin todo_demo)".to_string());
                }
                let launch_args: Vec<String> = if launch_args.first().is_some_and(|v| v == "--") {
                    launch_args.iter().skip(1).cloned().collect()
                } else {
                    launch_args.to_vec()
                };
                if launch_args.is_empty() {
                    return Err("missing command after --launch --".to_string());
                }
                launch = Some(launch_args);
                break;
            }
            other if other.starts_with('-') => {
                // Once we have the subcommand, allow subcommand-specific flags to pass through.
                if positionals.is_empty() {
                    return Err(format!("unknown diag flag: {other}"));
                }
                positionals.push(arg.clone());
                i += 1;
            }
            _ => {
                positionals.push(arg.clone());
                i += 1;
            }
        }
    }

    let Some(sub) = positionals.first().cloned() else {
        return Err("missing diag subcommand (try: fretboard diag poke)".to_string());
    };
    let rest: Vec<String> = positionals.into_iter().skip(1).collect();

    if launch_high_priority && launch.is_none() {
        return Err("--launch-high-priority requires --launch".to_string());
    }

    if fixed_frame_delta_ms.is_some() && launch.is_none() && devtools_ws_url.is_some() {
        return Err(
            "--fixed-frame-delta-ms requires --launch when used with --devtools-ws-url (or start the app with FRET_DIAG_FIXED_FRAME_DELTA_MS)"
                .to_string(),
        );
    }
    if let Some(ms) = fixed_frame_delta_ms {
        push_env_if_missing(
            &mut launch_env,
            "FRET_DIAG_FIXED_FRAME_DELTA_MS",
            &ms.to_string(),
        );
    }
    if check_pixels_changed_test_id.is_some() {
        push_env_if_missing(&mut launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
    }

    let resource_footprint_thresholds = ResourceFootprintThresholds {
        max_working_set_bytes,
        max_peak_working_set_bytes,
        max_cpu_avg_percent_total_cores,
    };

    if sub != "repro" && (with_tracy || with_renderdoc || renderdoc_after_frames.is_some()) {
        return Err(
            "--with tracy/renderdoc and --renderdoc-after-frames are only supported with `diag repro` for now"
                .to_string(),
        );
    }
    if sub != "repro" && (!renderdoc_markers.is_empty() || renderdoc_no_outputs_png) {
        return Err(
            "--renderdoc-marker and --renderdoc-no-outputs-png are only supported with `diag repro` for now"
                .to_string(),
        );
    }
    if sub != "repro" && resource_footprint_thresholds.any() {
        return Err(
            "--max-working-set-bytes/--max-peak-working-set-bytes/--max-cpu-avg-percent-total-cores are only supported with `diag repro` for now"
                .to_string(),
        );
    }
    if sub != "repro" && check_redraw_hitches_max_total_ms_threshold.is_some() {
        return Err(
            "--check-redraw-hitches-max-total-ms is only supported with `diag repro` for now"
                .to_string(),
        );
    }
    if sub != "run" && exit_after_run {
        return Err("--exit-after-run is only supported with `diag run`".to_string());
    }
    if keep_open && sub != "run" && sub != "suite" {
        return Err("--keep-open is only supported with `diag run` or `diag suite`".to_string());
    }
    if keep_open && launch.is_none() {
        return Err("--keep-open requires --launch".to_string());
    }
    if keep_open && exit_after_run {
        return Err("--keep-open conflicts with --exit-after-run".to_string());
    }
    if sub != "suite" && !suite_script_inputs.is_empty() {
        return Err("--glob/--script-dir are only supported with `diag suite`".to_string());
    }
    if sub != "script"
        && (shrink_out.is_some()
            || shrink_any_fail
            || shrink_match_reason_code.is_some()
            || shrink_match_reason.is_some()
            || shrink_min_steps != 1
            || shrink_max_iters != 200)
    {
        return Err("--shrink-* flags are only supported with `diag script shrink`".to_string());
    }

    let workspace_root = crate::cli::workspace_root()?;

    let resolved_out_dir = {
        let raw = out_dir
            .clone()
            .or_else(|| {
                std::env::var_os("FRET_DIAG_DIR")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| PathBuf::from("target").join("fret-diag"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_trigger_path = {
        let raw = trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("trigger.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_ready_path = {
        let raw = std::env::var_os("FRET_DIAG_READY_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| resolved_out_dir.join("ready.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_exit_path = {
        let raw = std::env::var_os("FRET_DIAG_EXIT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| resolved_out_dir.join("exit.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_script_path = {
        let raw = script_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_SCRIPT_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("script.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_script_trigger_path = {
        let raw = script_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_SCRIPT_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("script.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_script_result_path = {
        let raw = script_result_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_SCRIPT_RESULT_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("script.result.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_script_result_trigger_path = {
        let raw = script_result_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("script.result.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_pick_trigger_path = {
        let raw = pick_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_PICK_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("pick.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_pick_result_path = {
        let raw = pick_result_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_PICK_RESULT_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("pick.result.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_pick_result_trigger_path = {
        let raw = pick_result_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_PICK_RESULT_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("pick.result.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_pick_script_out = {
        let raw = pick_script_out.unwrap_or_else(|| resolved_out_dir.join("picked.script.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_inspect_path = {
        let raw = inspect_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_INSPECT_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("inspect.json"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_inspect_trigger_path = {
        let raw = inspect_trigger_path
            .or_else(|| {
                std::env::var_os("FRET_DIAG_INSPECT_TRIGGER_PATH")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| resolved_out_dir.join("inspect.touch"));
        resolve_path(&workspace_root, raw)
    };

    // If the tooling is launching the app and we intend to produce schema2-focused artifacts
    // (`bundle.schema2.json`, `frames.index.json`, ai packets), default to enabling schema2
    // emission at the runtime. This is opt-in at the runtime layer, but the tooling can safely
    // enable it for launched runs.
    let wants_runtime_schema2 = pack_schema2_only || pack_ai_only || ensure_ai_packet;
    if launch.is_some()
        && wants_runtime_schema2
        && !launch_env
            .iter()
            .any(|(k, _)| k == "FRET_DIAG_BUNDLE_WRITE_SCHEMA2")
        && std::env::var_os("FRET_DIAG_BUNDLE_WRITE_SCHEMA2").is_none()
    {
        launch_env.push((
            "FRET_DIAG_BUNDLE_WRITE_SCHEMA2".to_string(),
            "1".to_string(),
        ));
    }

    let fs_transport_cfg = crate::transport::FsDiagTransportConfig {
        out_dir: resolved_out_dir.clone(),
        trigger_path: resolved_trigger_path.clone(),
        script_path: resolved_script_path.clone(),
        script_trigger_path: resolved_script_trigger_path.clone(),
        script_result_path: resolved_script_result_path.clone(),
        script_result_trigger_path: resolved_script_result_trigger_path.clone(),
        pick_trigger_path: resolved_pick_trigger_path.clone(),
        pick_result_path: resolved_pick_result_path.clone(),
        pick_result_trigger_path: resolved_pick_result_trigger_path.clone(),
        inspect_path: resolved_inspect_path.clone(),
        inspect_trigger_path: resolved_inspect_trigger_path.clone(),
        screenshots_request_path: resolved_out_dir.join("screenshots.request.json"),
        screenshots_trigger_path: resolved_out_dir.join("screenshots.touch"),
        screenshots_result_path: resolved_out_dir.join("screenshots.result.json"),
        screenshots_result_trigger_path: resolved_out_dir.join("screenshots.result.touch"),
    };

    if let Some(res) = diag_simple_dispatch::dispatch_simple(
        sub.as_str(),
        &rest,
        pack_after_run,
        &workspace_root,
        &resolved_out_dir,
        &resolved_trigger_path,
        trace_out.clone(),
        &pack_out,
        ensure_ai_packet,
        pack_ai_only,
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
        pack_schema2_only,
        &triage_out,
        &lint_out,
        &meta_out,
        meta_report,
        &index_out,
        &test_ids_out,
        &hotspots_out,
        &bundle_v2_out,
        &query_out,
        &slice_out,
        &ai_packet_out,
        stats_top,
        sort_override,
        warmup_frames,
        stats_json,
        max_test_ids,
        lint_all_test_ids_bounds,
        lint_eps_px,
        &resolved_script_path,
        &resolved_script_trigger_path,
        &resolved_script_result_path,
        &resolved_script_result_trigger_path,
        &resolved_ready_path,
        &resolved_exit_path,
        script_tool_check,
        script_tool_write,
        &script_tool_check_out,
        &shrink_out,
        shrink_any_fail,
        &shrink_match_reason_code,
        &shrink_match_reason,
        shrink_min_steps,
        shrink_max_iters,
        &launch,
        &launch_env,
        timeout_ms,
        poll_ms,
    ) {
        return res;
    }

    let run_checks = diag_run::RunChecks {
        check_chart_sampling_window_shifts_min: check_chart_sampling_window_shifts_min.clone(),
        check_dock_drag_min: check_dock_drag_min.clone(),
        check_drag_cache_root_paint_only_test_id: check_drag_cache_root_paint_only_test_id.clone(),
        check_gc_sweep_liveness: check_gc_sweep_liveness.clone(),
        check_hover_layout_max: check_hover_layout_max.clone(),
        check_idle_no_paint_min: check_idle_no_paint_min.clone(),
        check_layout_fast_path_min: check_layout_fast_path_min.clone(),
        check_node_graph_cull_window_shifts_max: check_node_graph_cull_window_shifts_max.clone(),
        check_node_graph_cull_window_shifts_min: check_node_graph_cull_window_shifts_min.clone(),
        check_notify_hotspot_file_max: check_notify_hotspot_file_max.clone(),
        check_overlay_synthesis_min: check_overlay_synthesis_min.clone(),
        check_pixels_changed_test_id: check_pixels_changed_test_id.clone(),
        check_prepaint_actions_min: check_prepaint_actions_min.clone(),
        check_retained_vlist_attach_detach_max: check_retained_vlist_attach_detach_max.clone(),
        check_retained_vlist_keep_alive_budget: check_retained_vlist_keep_alive_budget.clone(),
        check_retained_vlist_keep_alive_reuse_min: check_retained_vlist_keep_alive_reuse_min.clone(),
        check_retained_vlist_reconcile_no_notify_min: check_retained_vlist_reconcile_no_notify_min.clone(),
        check_semantics_changed_repainted: check_semantics_changed_repainted.clone(),
        check_stale_paint_eps: check_stale_paint_eps.clone(),
        check_stale_paint_test_id: check_stale_paint_test_id.clone(),
        check_stale_scene_eps: check_stale_scene_eps.clone(),
        check_stale_scene_test_id: check_stale_scene_test_id.clone(),
        check_ui_gallery_code_editor_a11y_composition: check_ui_gallery_code_editor_a11y_composition.clone(),
        check_ui_gallery_code_editor_a11y_composition_drag: check_ui_gallery_code_editor_a11y_composition_drag.clone(),
        check_ui_gallery_code_editor_a11y_composition_wrap: check_ui_gallery_code_editor_a11y_composition_wrap.clone(),
        check_ui_gallery_code_editor_a11y_composition_wrap_scroll: check_ui_gallery_code_editor_a11y_composition_wrap_scroll.clone(),
        check_ui_gallery_code_editor_a11y_selection: check_ui_gallery_code_editor_a11y_selection.clone(),
        check_ui_gallery_code_editor_a11y_selection_wrap: check_ui_gallery_code_editor_a11y_selection_wrap.clone(),
        check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection: check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection.clone(),
        check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll: check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll.clone(),
        check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed: check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed.clone(),
        check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed: check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed.clone(),
        check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit: check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit.clone(),
        check_ui_gallery_code_editor_torture_folds_placeholder_present: check_ui_gallery_code_editor_torture_folds_placeholder_present.clone(),
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped: check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped.clone(),
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations: check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations.clone(),
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed: check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed.clone(),
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap: check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap.clone(),
        check_ui_gallery_code_editor_torture_geom_fallbacks_low: check_ui_gallery_code_editor_torture_geom_fallbacks_low.clone(),
        check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit: check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit.clone(),
        check_ui_gallery_code_editor_torture_inlays_present: check_ui_gallery_code_editor_torture_inlays_present.clone(),
        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped: check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped.clone(),
        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations: check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations.clone(),
        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed: check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed.clone(),
        check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap: check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap.clone(),
        check_ui_gallery_code_editor_torture_marker_present: check_ui_gallery_code_editor_torture_marker_present.clone(),
        check_ui_gallery_code_editor_torture_read_only_blocks_edits: check_ui_gallery_code_editor_torture_read_only_blocks_edits.clone(),
        check_ui_gallery_code_editor_torture_undo_redo: check_ui_gallery_code_editor_torture_undo_redo.clone(),
        check_ui_gallery_code_editor_word_boundary: check_ui_gallery_code_editor_word_boundary.clone(),
        check_ui_gallery_markdown_editor_source_a11y_composition: check_ui_gallery_markdown_editor_source_a11y_composition.clone(),
        check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap: check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap.clone(),
        check_ui_gallery_markdown_editor_source_disabled_blocks_edits: check_ui_gallery_markdown_editor_source_disabled_blocks_edits.clone(),
        check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds: check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds.clone(),
        check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit: check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit.clone(),
        check_ui_gallery_markdown_editor_source_folds_placeholder_present: check_ui_gallery_markdown_editor_source_folds_placeholder_present.clone(),
        check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap: check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap.clone(),
        check_ui_gallery_markdown_editor_source_folds_toggle_stable: check_ui_gallery_markdown_editor_source_folds_toggle_stable.clone(),
        check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit: check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit.clone(),
        check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable: check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable.clone(),
        check_ui_gallery_markdown_editor_source_inlays_present: check_ui_gallery_markdown_editor_source_inlays_present.clone(),
        check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap: check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap.clone(),
        check_ui_gallery_markdown_editor_source_inlays_toggle_stable: check_ui_gallery_markdown_editor_source_inlays_toggle_stable.clone(),
        check_ui_gallery_markdown_editor_source_line_boundary_triple_click: check_ui_gallery_markdown_editor_source_line_boundary_triple_click.clone(),
        check_ui_gallery_markdown_editor_source_read_only_blocks_edits: check_ui_gallery_markdown_editor_source_read_only_blocks_edits.clone(),
        check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable: check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable.clone(),
        check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable: check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable.clone(),
        check_ui_gallery_markdown_editor_source_word_boundary: check_ui_gallery_markdown_editor_source_word_boundary.clone(),
        check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change: check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change.clone(),
        check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change: check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change.clone(),
        check_ui_gallery_text_mixed_script_bundled_fallback_conformance: check_ui_gallery_text_mixed_script_bundled_fallback_conformance.clone(),
        check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps: check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps.clone(),
        check_ui_gallery_web_ime_bridge_enabled: check_ui_gallery_web_ime_bridge_enabled.clone(),
        check_view_cache_reuse_min: check_view_cache_reuse_min.clone(),
        check_view_cache_reuse_stable_min: check_view_cache_reuse_stable_min.clone(),
        check_viewport_capture_min: check_viewport_capture_min.clone(),
        check_viewport_input_min: check_viewport_input_min.clone(),
        check_vlist_policy_key_stable: check_vlist_policy_key_stable.clone(),
        check_vlist_visible_range_refreshes_max: check_vlist_visible_range_refreshes_max.clone(),
        check_vlist_visible_range_refreshes_min: check_vlist_visible_range_refreshes_min.clone(),
        check_vlist_window_shifts_escape_max: check_vlist_window_shifts_escape_max.clone(),
        check_vlist_window_shifts_explainable: check_vlist_window_shifts_explainable.clone(),
        check_vlist_window_shifts_have_prepaint_actions: check_vlist_window_shifts_have_prepaint_actions.clone(),
        check_vlist_window_shifts_non_retained_max: check_vlist_window_shifts_non_retained_max.clone(),
        check_vlist_window_shifts_prefetch_max: check_vlist_window_shifts_prefetch_max.clone(),
        check_wheel_scroll_hit_changes_test_id: check_wheel_scroll_hit_changes_test_id.clone(),
        check_wheel_scroll_test_id: check_wheel_scroll_test_id.clone(),
        check_windowed_rows_offset_changes_eps: check_windowed_rows_offset_changes_eps.clone(),
        check_windowed_rows_offset_changes_min: check_windowed_rows_offset_changes_min.clone(),
        check_windowed_rows_visible_start_changes_repainted: check_windowed_rows_visible_start_changes_repainted.clone(),
        dump_semantics_changed_repainted_json: dump_semantics_changed_repainted_json.clone(),
    };

    match sub.as_str() {
        "run" => {
            diag_run::cmd_run(diag_run::RunCmdContext {
                pack_after_run,
                ensure_ai_packet,
                rest: rest.clone(),
                workspace_root: workspace_root.clone(),
                resolved_out_dir: resolved_out_dir.clone(),
                resolved_trigger_path: resolved_trigger_path.clone(),
                resolved_ready_path: resolved_ready_path.clone(),
                resolved_exit_path: resolved_exit_path.clone(),
                resolved_script_path: resolved_script_path.clone(),
                resolved_script_result_path: resolved_script_result_path.clone(),
                fs_transport_cfg: fs_transport_cfg.clone(),
                pack_out: pack_out.clone(),
                pack_include_root_artifacts,
                pack_include_triage,
                pack_include_screenshots,
                pack_schema2_only,
                stats_top,
                sort_override,
                warmup_frames,
                timeout_ms,
                poll_ms,
                trace_chrome,
                devtools_ws_url: devtools_ws_url.clone(),
                devtools_token: devtools_token.clone(),
                devtools_session_id: devtools_session_id.clone(),
                exit_after_run,
                launch: launch.clone(),
                launch_env: launch_env.clone(),
                reuse_launch,
                launch_high_priority,
                keep_open,
                checks: run_checks.clone(),
            })
        }
        "repeat" => {
            diag_repeat::cmd_repeat(diag_repeat::RepeatCmdContext {
                pack_after_run,
                rest: rest.clone(),
                workspace_root: workspace_root.clone(),
                resolved_out_dir: resolved_out_dir.clone(),
                resolved_ready_path: resolved_ready_path.clone(),
                resolved_exit_path: resolved_exit_path.clone(),
                resolved_script_path: resolved_script_path.clone(),
                resolved_script_trigger_path: resolved_script_trigger_path.clone(),
                resolved_script_result_path: resolved_script_result_path.clone(),
                resolved_script_result_trigger_path: resolved_script_result_trigger_path.clone(),
                pack_include_screenshots,
                check_pixels_changed_test_id: check_pixels_changed_test_id.clone(),
                reuse_launch,
                launch: launch.clone(),
                launch_env: launch_env.clone(),
                launch_high_priority,
                perf_repeat,
                compare_eps_px,
                compare_ignore_bounds,
                compare_ignore_scene_fingerprint,
                warmup_frames,
                lint_all_test_ids_bounds,
                lint_eps_px,
                stats_json,
                timeout_ms,
                poll_ms,
            })
        }
        "repro" => {
            diag_repro::cmd_repro(diag_repro::ReproCmdContext {
                rest: rest.clone(),
                workspace_root: workspace_root.clone(),
                resolved_out_dir: resolved_out_dir.clone(),
                resolved_ready_path: resolved_ready_path.clone(),
                resolved_exit_path: resolved_exit_path.clone(),
                resolved_script_path: resolved_script_path.clone(),
                resolved_script_trigger_path: resolved_script_trigger_path.clone(),
                resolved_script_result_path: resolved_script_result_path.clone(),
                resolved_script_result_trigger_path: resolved_script_result_trigger_path.clone(),
                fs_transport_cfg: fs_transport_cfg.clone(),
                pack_out: pack_out.clone(),
                ensure_ai_packet,
                pack_ai_only,
                pack_include_root_artifacts,
                pack_include_triage,
                pack_include_screenshots,
                pack_schema2_only,
                stats_top,
                sort_override,
                warmup_frames,
                timeout_ms,
                poll_ms,
                trace_chrome,
                launch: launch.clone(),
                launch_env: launch_env.clone(),
                launch_high_priority,
                with_tracy,
                with_renderdoc,
                renderdoc_after_frames,
                renderdoc_markers: renderdoc_markers.clone(),
                renderdoc_no_outputs_png,
                resource_footprint_thresholds,
                check_redraw_hitches_max_total_ms_threshold,
                checks: run_checks.clone(),
            })
        }
        "suite" => {
            diag_suite::cmd_suite(diag_suite::SuiteCmdContext {
                pack_after_run,
                rest: rest.clone(),
                suite_script_inputs: suite_script_inputs.clone(),
                suite_prewarm_scripts: suite_prewarm_scripts.clone(),
                suite_prelude_scripts: suite_prelude_scripts.clone(),
                suite_prelude_each_run,
                workspace_root: workspace_root.clone(),
                resolved_out_dir: resolved_out_dir.clone(),
                resolved_ready_path: resolved_ready_path.clone(),
                resolved_script_result_path: resolved_script_result_path.clone(),
                devtools_ws_url: devtools_ws_url.clone(),
                devtools_token: devtools_token.clone(),
                devtools_session_id: devtools_session_id.clone(),
                timeout_ms,
                poll_ms,
                stats_top,
                stats_json,
                warmup_frames,
                max_test_ids,
                lint_all_test_ids_bounds,
                lint_eps_px,
                suite_lint,
                pack_include_screenshots,
                reuse_launch,
                launch: launch.clone(),
                launch_env: launch_env.clone(),
                launch_high_priority,
                keep_open,
                checks: diag_suite::SuiteChecks {
                check_chart_sampling_window_shifts_min: check_chart_sampling_window_shifts_min.clone(),
                check_dock_drag_min: check_dock_drag_min.clone(),
                check_drag_cache_root_paint_only_test_id: check_drag_cache_root_paint_only_test_id.clone(),
                check_gc_sweep_liveness: check_gc_sweep_liveness.clone(),
                check_hover_layout_max: check_hover_layout_max.clone(),
                check_idle_no_paint_min: check_idle_no_paint_min.clone(),
                check_layout_fast_path_min: check_layout_fast_path_min.clone(),
                check_node_graph_cull_window_shifts_max: check_node_graph_cull_window_shifts_max.clone(),
                check_node_graph_cull_window_shifts_min: check_node_graph_cull_window_shifts_min.clone(),
                check_notify_hotspot_file_max: check_notify_hotspot_file_max.clone(),
                check_overlay_synthesis_min: check_overlay_synthesis_min.clone(),
                check_pixels_changed_test_id: check_pixels_changed_test_id.clone(),
                check_prepaint_actions_min: check_prepaint_actions_min.clone(),
                check_retained_vlist_attach_detach_max: check_retained_vlist_attach_detach_max.clone(),
                check_retained_vlist_keep_alive_budget: check_retained_vlist_keep_alive_budget.clone(),
                check_retained_vlist_keep_alive_reuse_min: check_retained_vlist_keep_alive_reuse_min.clone(),
                check_retained_vlist_reconcile_no_notify_min: check_retained_vlist_reconcile_no_notify_min.clone(),
                check_semantics_changed_repainted: check_semantics_changed_repainted.clone(),
                check_stale_paint_eps: check_stale_paint_eps.clone(),
                check_stale_paint_test_id: check_stale_paint_test_id.clone(),
                check_stale_scene_eps: check_stale_scene_eps.clone(),
                check_stale_scene_test_id: check_stale_scene_test_id.clone(),
                check_ui_gallery_code_editor_a11y_composition: check_ui_gallery_code_editor_a11y_composition.clone(),
                check_ui_gallery_code_editor_a11y_composition_drag: check_ui_gallery_code_editor_a11y_composition_drag.clone(),
                check_ui_gallery_code_editor_a11y_composition_wrap: check_ui_gallery_code_editor_a11y_composition_wrap.clone(),
                check_ui_gallery_code_editor_a11y_composition_wrap_scroll: check_ui_gallery_code_editor_a11y_composition_wrap_scroll.clone(),
                check_ui_gallery_code_editor_a11y_selection: check_ui_gallery_code_editor_a11y_selection.clone(),
                check_ui_gallery_code_editor_a11y_selection_wrap: check_ui_gallery_code_editor_a11y_selection_wrap.clone(),
                check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection: check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection.clone(),
                check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll: check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll.clone(),
                check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed: check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed.clone(),
                check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed: check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed.clone(),
                check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit: check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit.clone(),
                check_ui_gallery_code_editor_torture_folds_placeholder_present: check_ui_gallery_code_editor_torture_folds_placeholder_present.clone(),
                check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped: check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped.clone(),
                check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations: check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations.clone(),
                check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed: check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed.clone(),
                check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap: check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap.clone(),
                check_ui_gallery_code_editor_torture_geom_fallbacks_low: check_ui_gallery_code_editor_torture_geom_fallbacks_low.clone(),
                check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit: check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit.clone(),
                check_ui_gallery_code_editor_torture_inlays_present: check_ui_gallery_code_editor_torture_inlays_present.clone(),
                check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped: check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped.clone(),
                check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations: check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations.clone(),
                check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed: check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed.clone(),
                check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap: check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap.clone(),
                check_ui_gallery_code_editor_torture_marker_present: check_ui_gallery_code_editor_torture_marker_present.clone(),
                check_ui_gallery_code_editor_torture_read_only_blocks_edits: check_ui_gallery_code_editor_torture_read_only_blocks_edits.clone(),
                check_ui_gallery_code_editor_torture_undo_redo: check_ui_gallery_code_editor_torture_undo_redo.clone(),
                check_ui_gallery_code_editor_word_boundary: check_ui_gallery_code_editor_word_boundary.clone(),
                check_ui_gallery_markdown_editor_source_a11y_composition: check_ui_gallery_markdown_editor_source_a11y_composition.clone(),
                check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap: check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap.clone(),
                check_ui_gallery_markdown_editor_source_disabled_blocks_edits: check_ui_gallery_markdown_editor_source_disabled_blocks_edits.clone(),
                check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds: check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds.clone(),
                check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit: check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit.clone(),
                check_ui_gallery_markdown_editor_source_folds_placeholder_present: check_ui_gallery_markdown_editor_source_folds_placeholder_present.clone(),
                check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap: check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap.clone(),
                check_ui_gallery_markdown_editor_source_folds_toggle_stable: check_ui_gallery_markdown_editor_source_folds_toggle_stable.clone(),
                check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit: check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit.clone(),
                check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable: check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable.clone(),
                check_ui_gallery_markdown_editor_source_inlays_present: check_ui_gallery_markdown_editor_source_inlays_present.clone(),
                check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap: check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap.clone(),
                check_ui_gallery_markdown_editor_source_inlays_toggle_stable: check_ui_gallery_markdown_editor_source_inlays_toggle_stable.clone(),
                check_ui_gallery_markdown_editor_source_line_boundary_triple_click: check_ui_gallery_markdown_editor_source_line_boundary_triple_click.clone(),
                check_ui_gallery_markdown_editor_source_read_only_blocks_edits: check_ui_gallery_markdown_editor_source_read_only_blocks_edits.clone(),
                check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable: check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable.clone(),
                check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable: check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable.clone(),
                check_ui_gallery_markdown_editor_source_word_boundary: check_ui_gallery_markdown_editor_source_word_boundary.clone(),
                check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change: check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change.clone(),
                check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change: check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change.clone(),
                check_ui_gallery_text_mixed_script_bundled_fallback_conformance: check_ui_gallery_text_mixed_script_bundled_fallback_conformance.clone(),
                check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps: check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps.clone(),
                check_ui_gallery_web_ime_bridge_enabled: check_ui_gallery_web_ime_bridge_enabled.clone(),
                check_view_cache_reuse_min: check_view_cache_reuse_min.clone(),
                check_view_cache_reuse_stable_min: check_view_cache_reuse_stable_min.clone(),
                check_viewport_capture_min: check_viewport_capture_min.clone(),
                check_viewport_input_min: check_viewport_input_min.clone(),
                check_vlist_policy_key_stable: check_vlist_policy_key_stable.clone(),
                check_vlist_visible_range_refreshes_max: check_vlist_visible_range_refreshes_max.clone(),
                check_vlist_visible_range_refreshes_min: check_vlist_visible_range_refreshes_min.clone(),
                check_vlist_window_shifts_escape_max: check_vlist_window_shifts_escape_max.clone(),
                check_vlist_window_shifts_explainable: check_vlist_window_shifts_explainable.clone(),
                check_vlist_window_shifts_have_prepaint_actions: check_vlist_window_shifts_have_prepaint_actions.clone(),
                check_vlist_window_shifts_non_retained_max: check_vlist_window_shifts_non_retained_max.clone(),
                check_vlist_window_shifts_prefetch_max: check_vlist_window_shifts_prefetch_max.clone(),
                check_wheel_scroll_hit_changes_test_id: check_wheel_scroll_hit_changes_test_id.clone(),
                check_wheel_scroll_test_id: check_wheel_scroll_test_id.clone(),
                check_windowed_rows_offset_changes_eps: check_windowed_rows_offset_changes_eps.clone(),
                check_windowed_rows_offset_changes_min: check_windowed_rows_offset_changes_min.clone(),
                check_windowed_rows_visible_start_changes_repainted: check_windowed_rows_visible_start_changes_repainted.clone(),
                dump_semantics_changed_repainted_json: dump_semantics_changed_repainted_json.clone(),
                },
            })
        }
        "perf-baseline-from-bundles" => {
            diag_perf_baseline::cmd_perf_baseline_from_bundles(
                diag_perf_baseline::PerfBaselineFromBundlesContext {
                    pack_after_run,
                    rest: rest.clone(),
                    workspace_root: workspace_root.clone(),
                    sort_override,
                    perf_baseline_out: perf_baseline_out.clone(),
                    perf_baseline_headroom_pct,
                    warmup_frames,
                    stats_json,
                },
            )
        }
        "perf" => {
            diag_perf::cmd_perf(diag_perf::PerfCmdContext {
                pack_after_run,
                rest: rest.clone(),
                workspace_root: workspace_root.clone(),
                resolved_out_dir: resolved_out_dir.clone(),
                resolved_ready_path: resolved_ready_path.clone(),
                resolved_exit_path: resolved_exit_path.clone(),
                resolved_script_path: resolved_script_path.clone(),
                resolved_script_trigger_path: resolved_script_trigger_path.clone(),
                resolved_script_result_path: resolved_script_result_path.clone(),
                resolved_script_result_trigger_path: resolved_script_result_trigger_path.clone(),
                check_perf_hints: check_perf_hints.clone(),
                check_perf_hints_deny: check_perf_hints_deny.clone(),
                check_perf_hints_min_severity: check_perf_hints_min_severity.clone(),
                check_pixels_changed_test_id: check_pixels_changed_test_id.clone(),
                devtools_session_id: devtools_session_id.clone(),
                devtools_token: devtools_token.clone(),
                devtools_ws_url: devtools_ws_url.clone(),
                keep_open: keep_open.clone(),
                launch: launch.clone(),
                launch_env: launch_env.clone(),
                launch_high_priority: launch_high_priority.clone(),
                max_frame_p95_layout_us: max_frame_p95_layout_us.clone(),
                max_frame_p95_solve_us: max_frame_p95_solve_us.clone(),
                max_frame_p95_total_us: max_frame_p95_total_us.clone(),
                max_pointer_move_dispatch_us: max_pointer_move_dispatch_us.clone(),
                max_pointer_move_global_changes: max_pointer_move_global_changes.clone(),
                max_pointer_move_hit_test_us: max_pointer_move_hit_test_us.clone(),
                max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max.clone(),
                max_top_layout_us: max_top_layout_us.clone(),
                max_top_solve_us: max_top_solve_us.clone(),
                max_top_total_us: max_top_total_us.clone(),
                min_run_paint_cache_hit_test_only_replay_allowed_max: min_run_paint_cache_hit_test_only_replay_allowed_max.clone(),
                perf_baseline_headroom_pct: perf_baseline_headroom_pct.clone(),
                perf_baseline_out: perf_baseline_out.clone(),
                perf_baseline_path: perf_baseline_path.clone(),
                perf_baseline_seed_preset_paths: perf_baseline_seed_preset_paths.clone(),
                perf_baseline_seed_specs: perf_baseline_seed_specs.clone(),
                perf_repeat: perf_repeat.clone(),
                perf_threshold_agg: perf_threshold_agg.clone(),
                poll_ms: poll_ms.clone(),
                reuse_launch: reuse_launch.clone(),
                reuse_launch_per_script: reuse_launch_per_script.clone(),
                sort_override: sort_override.clone(),
                stats_json: stats_json.clone(),
                stats_top: stats_top.clone(),
                suite_prelude_each_run: suite_prelude_each_run.clone(),
                suite_prelude_scripts: suite_prelude_scripts.clone(),
                suite_prewarm_scripts: suite_prewarm_scripts.clone(),
                timeout_ms: timeout_ms.clone(),
                trace_chrome: trace_chrome.clone(),
                warmup_frames: warmup_frames.clone(),
            })
        }
        "stats" => {
            diag_stats::cmd_stats(diag_stats::StatsCmdContext {
                rest: rest.clone(),
                stats_diff: stats_diff.take(),
                workspace_root: workspace_root.clone(),
                sort_override: sort_override.clone(),
                stats_top,
                stats_json,
                stats_verbose,
                warmup_frames,
                check_stale_paint_test_id: check_stale_paint_test_id.clone(),
                check_stale_paint_eps,
                check_stale_scene_test_id: check_stale_scene_test_id.clone(),
                check_stale_scene_eps,
                check_idle_no_paint_min,
                check_pixels_changed_test_id: check_pixels_changed_test_id.clone(),
                check_semantics_changed_repainted,
                dump_semantics_changed_repainted_json,
                check_wheel_scroll_test_id: check_wheel_scroll_test_id.clone(),
                check_wheel_scroll_hit_changes_test_id: check_wheel_scroll_hit_changes_test_id.clone(),
                check_drag_cache_root_paint_only_test_id: check_drag_cache_root_paint_only_test_id.clone(),
                check_hover_layout_max,
                check_gc_sweep_liveness,
                check_notify_hotspot_file_max: check_notify_hotspot_file_max.clone(),
                check_view_cache_reuse_stable_min,
                check_view_cache_reuse_min,
                check_overlay_synthesis_min,
                check_viewport_input_min,
                check_dock_drag_min,
                check_viewport_capture_min,
                check_retained_vlist_reconcile_no_notify_min,
                check_retained_vlist_attach_detach_max,
                check_retained_vlist_keep_alive_reuse_min,
            })
        }
        "matrix" => {
            diag_matrix::cmd_matrix(diag_matrix::MatrixCmdContext {
                rest: rest.clone(),
                launch: launch.clone(),
                launch_env: launch_env.clone(),
                launch_high_priority,
                workspace_root: workspace_root.clone(),
                resolved_out_dir: resolved_out_dir.clone(),
                timeout_ms,
                poll_ms,
                warmup_frames,
                compare_eps_px,
                compare_ignore_bounds,
                compare_ignore_scene_fingerprint,
                check_view_cache_reuse_min,
                check_view_cache_reuse_stable_min,
                check_overlay_synthesis_min,
                check_viewport_input_min,
                stats_json,
            })
        }
        "compare" => {
            diag_compare::cmd_compare(diag_compare::CompareCmdContext {
                rest: rest.clone(),
                workspace_root: workspace_root.clone(),
                warmup_frames,
                compare_eps_px,
                compare_ignore_bounds,
                compare_ignore_scene_fingerprint,
                stats_json,
            })
        }
        "inspect" => commands::inspect::cmd_inspect(
            &rest,
            &resolved_inspect_path,
            &resolved_inspect_trigger_path,
            inspect_consume_clicks,
        ),
        "pick-arm" => commands::pick::cmd_pick_arm(&rest, &resolved_pick_trigger_path),
        "pick" => commands::pick::cmd_pick(
            &rest,
            &resolved_pick_trigger_path,
            &resolved_pick_result_path,
            &resolved_pick_result_trigger_path,
            timeout_ms,
            poll_ms,
        ),
        "pick-script" => commands::pick::cmd_pick_script(
            &rest,
            &resolved_pick_trigger_path,
            &resolved_pick_result_path,
            &resolved_pick_result_trigger_path,
            &resolved_pick_script_out,
            timeout_ms,
            poll_ms,
        ),
        "pick-apply" => commands::pick::cmd_pick_apply(
            &rest,
            &workspace_root,
            &resolved_pick_trigger_path,
            &resolved_pick_result_path,
            &resolved_pick_result_trigger_path,
            pick_apply_pointer.as_deref(),
            pick_apply_out,
            timeout_ms,
            poll_ms,
        ),
        other => Err(format!("unknown diag subcommand: {other}")),
    }
}

pub(crate) fn triage_json_from_stats(
    bundle_path: &Path,
    report: &BundleStatsReport,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> serde_json::Value {
    triage_json::triage_json_from_stats(bundle_path, report, sort, warmup_frames)
}

fn parse_bool(s: &str) -> Result<bool, ()> {
    match s {
        "1" | "true" | "True" | "TRUE" => Ok(true),
        "0" | "false" | "False" | "FALSE" => Ok(false),
        _ => Err(()),
    }
}

pub(crate) fn script_requests_screenshots(script: &Path) -> bool {
    let Ok(bytes) = std::fs::read(script) else {
        return false;
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    script_requests_screenshots_value(&value)
}

fn script_required_capabilities(script: &Path) -> Vec<String> {
    let Ok(bytes) = std::fs::read(script) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Vec::new();
    };
    script_required_capabilities_value(&value)
}

fn script_env_defaults(script: &Path) -> Vec<(String, String)> {
    let Ok(bytes) = std::fs::read(script) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Vec::new();
    };
    script_env_defaults_value(&value)
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
        .filter_map(|c| normalize_capability_string(&c))
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

fn read_filesystem_capabilities(out_dir: &Path) -> Vec<String> {
    let path = out_dir.join("capabilities.json");
    let Ok(bytes) = std::fs::read(&path) else {
        return Vec::new();
    };
    let Ok(parsed) = serde_json::from_slice::<fret_diag_protocol::FilesystemCapabilitiesV1>(&bytes)
    else {
        return Vec::new();
    };
    let mut caps: Vec<String> = parsed
        .capabilities
        .into_iter()
        .filter_map(|c| normalize_capability_string(&c))
        .collect();
    caps.sort();
    caps.dedup();
    caps
}

fn normalize_capability_string(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    if raw.contains('.') {
        return Some(raw.to_string());
    }

    let mapped = match raw {
        "script_v2" => "diag.script_v2",
        "screenshot_png" => "diag.screenshot_png",
        "multi_window" => "diag.multi_window",
        "pointer_kind_touch" => "diag.pointer_kind_touch",
        "gesture_pinch" => "diag.gesture_pinch",
        _ => raw,
    };
    Some(mapped.to_string())
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
    fn gates_missing_capability_writes_script_result_with_structured_evidence() {
        let out_dir = make_temp_dir("fret-diag-capabilities-script-result");
        let script_path = out_dir.join("script.json");
        let check_path = out_dir.join("check.capabilities.json");
        let script_result_path = out_dir.join("script.result.json");

        let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
            schema_version: 1,
            capabilities: vec!["diag.script_v2".to_string()],
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
struct ResolvedScriptPaths {
    out_dir: PathBuf,
    ready_path: PathBuf,
    exit_path: PathBuf,
    script_path: PathBuf,
    script_trigger_path: PathBuf,
    script_result_path: PathBuf,
    script_result_trigger_path: PathBuf,
}

impl ResolvedScriptPaths {
    fn for_out_dir(workspace_root: &Path, out_dir: &Path) -> Self {
        let out_dir = resolve_path(workspace_root, out_dir.to_path_buf());
        Self {
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

fn wait_for_devtools_message<T>(
    devtools: &DevtoolsOps,
    timeout_ms: u64,
    poll_ms: u64,
    mut decode: impl FnMut(fret_diag_protocol::DiagTransportMessageV1) -> Option<T>,
) -> Result<T, String> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(1));
    loop {
        while let Some(msg) = devtools.try_recv() {
            if let Some(v) = decode(msg) {
                return Ok(v);
            }
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DevTools WS message".to_string());
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

fn wait_for_devtools_bundle_dumped(
    devtools: &DevtoolsOps,
    selected_session_id: &str,
    expected_request_id: Option<u64>,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<DevtoolsBundleDumpedV1, String> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(1));

    let mut chunk_exported_unix_ms: Option<u64> = None;
    let mut chunk_out_dir: Option<String> = None;
    let mut chunk_dir: Option<String> = None;
    let mut chunks: Vec<Option<String>> = Vec::new();

    loop {
        while let Some(msg) = devtools.try_recv() {
            if msg.r#type != "bundle.dumped"
                || msg.session_id.as_deref() != Some(selected_session_id)
            {
                continue;
            }
            if let Some(expected) = expected_request_id
                && msg.request_id != Some(expected)
            {
                continue;
            }
            let Ok(dumped) = serde_json::from_value::<DevtoolsBundleDumpedV1>(msg.payload) else {
                continue;
            };

            if dumped.bundle.is_some() {
                return Ok(dumped);
            }

            if let (Some(chunk), Some(chunk_index), Some(chunk_count_value)) = (
                dumped.bundle_json_chunk.clone(),
                dumped.bundle_json_chunk_index,
                dumped.bundle_json_chunk_count,
            ) {
                if chunk_exported_unix_ms.is_none() {
                    chunk_exported_unix_ms = Some(dumped.exported_unix_ms);
                    chunk_out_dir = Some(dumped.out_dir.clone());
                    chunk_dir = Some(dumped.dir.clone());
                    chunks = vec![None; chunk_count_value.max(1) as usize];
                }

                if chunk_exported_unix_ms != Some(dumped.exported_unix_ms)
                    || chunk_dir.as_deref() != Some(dumped.dir.as_str())
                {
                    // A new dump started (or messages interleaved); reset to the latest seen.
                    chunk_exported_unix_ms = Some(dumped.exported_unix_ms);
                    chunk_out_dir = Some(dumped.out_dir.clone());
                    chunk_dir = Some(dumped.dir.clone());
                    chunks = vec![None; chunk_count_value.max(1) as usize];
                }

                if let Some(slot) = chunks.get_mut(chunk_index as usize) {
                    *slot = Some(chunk);
                }

                if chunks.iter().all(|c| c.is_some()) {
                    let mut json = String::new();
                    for part in chunks.iter().flatten() {
                        json.push_str(part);
                    }
                    let bundle = serde_json::from_str::<serde_json::Value>(&json).map_err(|e| {
                        format!("bundle.dumped chunked JSON was not valid JSON: {e}")
                    })?;
                    return Ok(DevtoolsBundleDumpedV1 {
                        schema_version: dumped.schema_version,
                        exported_unix_ms: chunk_exported_unix_ms.unwrap_or(dumped.exported_unix_ms),
                        out_dir: chunk_out_dir.clone().unwrap_or(dumped.out_dir),
                        dir: chunk_dir.clone().unwrap_or(dumped.dir),
                        bundle: Some(bundle),
                        bundle_json_chunk: None,
                        bundle_json_chunk_index: None,
                        bundle_json_chunk_count: None,
                    });
                }

                continue;
            }

            // Non-embedded bundle (native filesystem case): allow materialization to fall back to
            // reading the runtime's bundle artifact.
            return Ok(dumped);
        }

        if Instant::now() >= deadline {
            return Err("timed out waiting for DevTools WS bundle.dumped".to_string());
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

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
    source: &'static str,
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
    ];

    let client = ToolingDiagClient::connect_ws(WsDiagTransportConfig::native(cfg))?;
    let devtools = DevtoolsOps::new(client);

    let sessions = wait_for_devtools_message(&devtools, timeout_ms, poll_ms, |msg| {
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
        .filter_map(|c| normalize_capability_string(&c))
        .collect();
    available_caps.sort();
    available_caps.dedup();

    Ok(ConnectedToolingTransport {
        devtools,
        selected_session_id,
        available_caps,
        source: "devtools_ws",
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

    let sessions = wait_for_devtools_message(&devtools, timeout_ms, poll_ms, |msg| {
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
        .filter_map(|c| normalize_capability_string(&c))
        .collect();
    available_caps.sort();
    available_caps.dedup();

    Ok(ConnectedToolingTransport {
        devtools,
        selected_session_id,
        available_caps,
        source: "filesystem",
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

    fn start_grace_ms(timeout_ms: u64, poll_ms: u64) -> u64 {
        let baseline_race_ms = poll_ms.saturating_mul(4).clamp(250, 5_000);
        baseline_race_ms.min(timeout_ms.saturating_div(2).max(250))
    }

    let required_caps = script_required_capabilities_value(&script_json);
    if !required_caps.is_empty() {
        gate_required_capabilities_with_script_result(
            capabilities_check_path,
            script_result_path,
            &required_caps,
            &connected.available_caps,
            connected.source,
        )?;
    }

    let prev_run_id = read_prev_run_id(script_result_path);
    let mut target_run_id: Option<u64> = None;
    let mut last_seen_stage: Option<&'static str> = None;
    let mut last_seen_step_index: Option<u32> = None;

    let mut next_retouch_at =
        Instant::now() + Duration::from_millis(start_grace_ms(timeout_ms, poll_ms));
    let mut retouch_interval_ms: u64 = 2_000;
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

            if target_run_id.is_none() && parsed.run_id > prev_run_id {
                target_run_id = Some(parsed.run_id);
            }
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
            // Note: `script_result_path` is a tooling output file (not the in-app filesystem
            // transport `runtime.script.result.json`), so it is safe to update it even when the
            // underlying transport is filesystem-based.
            let _ = write_json_value(
                script_result_path,
                &serde_json::to_value(&parsed).unwrap_or_else(|_| serde_json::json!({})),
            );
            write_run_id_script_result(out_dir, parsed.run_id, &parsed);

            if matches!(
                parsed.stage,
                UiScriptStageV1::Passed | UiScriptStageV1::Failed
            ) {
                break 'wait parsed;
            }
        }

        if Instant::now() >= deadline {
            let ws_hint = match connected.devtools.client().kind() {
                crate::transport::DiagTransportKind::WebSocket => Some(
                    "devtools_ws_hint=keep the app actively rendering (web: tab must be visible; background tabs may throttle rAF) and ensure the page URL includes fret_devtools_ws + fret_devtools_token",
                ),
                _ => None,
            };
            let note = format!(
                "source={} prev_run_id={} target_run_id={:?} last_seen_stage={} last_seen_step_index={:?} {}",
                connected.source,
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

        if connected.devtools.client().kind() == crate::transport::DiagTransportKind::FileSystem
            && target_run_id.is_none()
            && Instant::now() >= next_retouch_at
        {
            // Give the app a chance to observe the initial trigger file stamp baseline before
            // consuming a stamp as "the trigger". Retrying by re-sending the same script payload
            // mitigates the baseline race without requiring in-app changes.
            connected
                .devtools
                .script_run_value(None, script_json_value.clone());
            retouch_interval_ms = (retouch_interval_ms.saturating_mul(2)).min(10_000);
            next_retouch_at = Instant::now() + Duration::from_millis(retouch_interval_ms);
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    };

    let bundle_path = if dump_bundle {
        let expected_request_id = if connected.devtools.client().kind()
            == crate::transport::DiagTransportKind::WebSocket
        {
            if let Some(max) = dump_max_snapshots {
                Some(
                    connected
                        .devtools
                        .bundle_dump_with_max_snapshots(None, bundle_label, max),
                )
            } else {
                Some(connected.devtools.bundle_dump(None, bundle_label))
            }
        } else {
            if let Some(max) = dump_max_snapshots {
                connected
                    .devtools
                    .bundle_dump_with_max_snapshots(None, bundle_label, max);
            } else {
                connected.devtools.bundle_dump(None, bundle_label);
            }
            None
        };
        let dumped = (|| {
            // Filesystem transport can miss the first `trigger.touch` edge if the app has not yet
            // established its baseline stamp (similar to the `script.touch` baseline race).
            //
            // Mitigate by doing a short initial wait and re-touching once before consuming the
            // full timeout budget.
            if connected.devtools.client().kind() == crate::transport::DiagTransportKind::FileSystem
                && expected_request_id.is_none()
            {
                let short_ms = timeout_ms.min(2_000);
                match wait_for_devtools_bundle_dumped(
                    &connected.devtools,
                    &connected.selected_session_id,
                    None,
                    short_ms,
                    poll_ms,
                ) {
                    Ok(v) => return Ok(v),
                    Err(err) if err.contains("timed out waiting") => {
                        // Re-touch and fall through to the full wait below.
                        connected.devtools.bundle_dump(None, bundle_label);
                    }
                    Err(err) => return Err(err),
                }
            }

            wait_for_devtools_bundle_dumped(
                &connected.devtools,
                &connected.selected_session_id,
                expected_request_id,
                timeout_ms,
                poll_ms,
            )
        })()
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
        write_run_id_bundle_json(out_dir, result.run_id, &bundle_path);
        if trace_chrome {
            let run_dir = run_id_artifact_dir(out_dir, result.run_id);
            let stable_bundle_path = crate::resolve_bundle_artifact_path(&run_dir);
            let src = if stable_bundle_path.is_file() {
                stable_bundle_path
            } else {
                bundle_path.clone()
            };
            let trace_path = run_dir.join("trace.chrome.json");
            if let Err(err) = crate::trace::write_chrome_trace_from_bundle_path(&src, &trace_path) {
                push_tooling_event_log_entry(&mut result, "tooling_trace_chrome_failed", Some(err));
            } else {
                refresh_run_id_manifest_file_index(out_dir, result.run_id);
            }
        }
        result.last_bundle_dir = Some(devtools_sanitize_export_dir_name(&dumped.dir));
        result.last_bundle_artifact = Some(artifact_stats_from_bundle_json_path(&bundle_path));
        Some(bundle_path)
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
    let expected_request_id =
        if connected.devtools.client().kind() == crate::transport::DiagTransportKind::WebSocket {
            if let Some(max) = dump_max_snapshots {
                Some(
                    connected
                        .devtools
                        .bundle_dump_with_max_snapshots(None, bundle_label, max),
                )
            } else {
                Some(connected.devtools.bundle_dump(None, bundle_label))
            }
        } else {
            if let Some(max) = dump_max_snapshots {
                connected
                    .devtools
                    .bundle_dump_with_max_snapshots(None, bundle_label, max);
            } else {
                connected.devtools.bundle_dump(None, bundle_label);
            }
            None
        };

    let dumped = wait_for_devtools_bundle_dumped(
        &connected.devtools,
        &connected.selected_session_id,
        expected_request_id,
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
    let mut child = maybe_launch_demo(
        &launch,
        launch_env,
        workspace_root,
        &paths.out_dir,
        &paths.ready_path,
        &paths.exit_path,
        scripts.iter().any(|src| script_requests_screenshots(src)),
        timeout_ms,
        poll_ms,
        launch_high_priority,
    )?;

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

    let _ = stop_launched_demo(&mut child, &paths.exit_path, poll_ms);
    Ok(bundle_paths)
}

#[derive(Debug, Clone)]
struct IdleNoPaintWindowReport {
    window: u64,
    examined_snapshots: u64,
    idle_frames_total: u64,
    paint_frames_total: u64,
    idle_streak_max: u64,
    idle_streak_tail: u64,
    last_paint: Option<serde_json::Value>,
}

fn snapshot_is_idle_no_paint(snapshot: &serde_json::Value) -> bool {
    let stats = snapshot.get("debug").and_then(|v| v.get("stats"));
    let prepaint_time_us = stats
        .and_then(|v| v.get("prepaint_time_us"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let paint_time_us = stats
        .and_then(|v| v.get("paint_time_us"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let paint_nodes_performed = stats
        .and_then(|v| v.get("paint_nodes_performed"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    prepaint_time_us == 0 && paint_time_us == 0 && paint_nodes_performed == 0
}

fn check_bundle_for_idle_no_paint_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_idle_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

    let mut reports: Vec<IdleNoPaintWindowReport> = Vec::new();
    let mut failures: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut examined_snapshots: u64 = 0;
        let mut idle_frames_total: u64 = 0;
        let mut paint_frames_total: u64 = 0;
        let mut idle_streak: u64 = 0;
        let mut idle_streak_max: u64 = 0;
        let mut last_paint: Option<serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let is_idle = snapshot_is_idle_no_paint(s);
            if is_idle {
                idle_frames_total = idle_frames_total.saturating_add(1);
                idle_streak = idle_streak.saturating_add(1);
                idle_streak_max = idle_streak_max.max(idle_streak);
            } else {
                paint_frames_total = paint_frames_total.saturating_add(1);
                idle_streak = 0;

                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let stats = s.get("debug").and_then(|v| v.get("stats"));
                let prepaint_time_us = stats
                    .and_then(|v| v.get("prepaint_time_us"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let paint_time_us = stats
                    .and_then(|v| v.get("paint_time_us"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let paint_nodes_performed = stats
                    .and_then(|v| v.get("paint_nodes_performed"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                last_paint = Some(serde_json::json!({
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "prepaint_time_us": prepaint_time_us,
                    "paint_time_us": paint_time_us,
                    "paint_nodes_performed": paint_nodes_performed,
                }));
            }
        }

        reports.push(IdleNoPaintWindowReport {
            window,
            examined_snapshots,
            idle_frames_total,
            paint_frames_total,
            idle_streak_max,
            idle_streak_tail: idle_streak,
            last_paint: last_paint.clone(),
        });

        let mut fail_reason: Option<&'static str> = None;
        if min_idle_frames > 0 && examined_snapshots < min_idle_frames {
            fail_reason = Some("insufficient_snapshots");
        } else if min_idle_frames > 0 && idle_streak < min_idle_frames {
            fail_reason = Some("idle_tail_streak_too_small");
        }

        if let Some(reason) = fail_reason {
            failures.push(serde_json::json!({
                "window": window,
                "reason": reason,
                "examined_snapshots": examined_snapshots,
                "idle_streak_tail": idle_streak,
                "idle_streak_max": idle_streak_max,
                "idle_frames_total": idle_frames_total,
                "paint_frames_total": paint_frames_total,
                "last_paint": last_paint,
            }));
        }
    }

    let out_path = out_dir.join("check.idle_no_paint.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "idle_no_paint",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_idle_frames": min_idle_frames,
        "windows": reports.iter().map(|r| serde_json::json!({
            "window": r.window,
            "examined_snapshots": r.examined_snapshots,
            "idle_frames_total": r.idle_frames_total,
            "paint_frames_total": r.paint_frames_total,
            "idle_streak_max": r.idle_streak_max,
            "idle_streak_tail": r.idle_streak_tail,
            "last_paint": r.last_paint,
        })).collect::<Vec<_>>(),
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    if payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|v| v.is_empty())
        .unwrap_or(true)
    {
        return Ok(());
    }

    Err(format!(
        "idle no-paint gate failed (min_idle_frames={min_idle_frames}, warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

#[derive(Debug, Clone, Copy)]
struct RectF {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Debug, Clone, Copy)]
struct RectPx {
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
}

impl RectPx {
    fn width_px(self) -> u32 {
        self.x1.saturating_sub(self.x0)
    }
    fn height_px(self) -> u32 {
        self.y1.saturating_sub(self.y0)
    }
}

#[derive(Debug, Clone)]
struct PixelCheckResolvedShot {
    bundle_dir_name: String,
    file: String,
    window: u64,
    tick_id: u64,
    frame_id: u64,
    scale_factor: f64,
    rect_px: RectPx,
    hash: u64,
}

fn check_out_dir_for_pixels_changed(
    out_dir: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    use std::collections::HashMap;

    let screenshots_result_path = out_dir.join("screenshots.result.json");
    if !screenshots_result_path.is_file() {
        return Err(format!(
            "pixels changed check requires screenshots results under {} (set FRET_DIAG_GPU_SCREENSHOTS=1 and add capture_screenshot steps): {}",
            out_dir.display(),
            screenshots_result_path.display()
        ));
    }

    let bytes = std::fs::read(&screenshots_result_path).map_err(|e| e.to_string())?;
    let root: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let completed = root
        .get("completed")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid screenshots.result.json: missing completed array".to_string())?;

    let mut bundles_cache: HashMap<String, serde_json::Value> = HashMap::new();
    let mut resolved: Vec<PixelCheckResolvedShot> = Vec::new();

    for entry in completed {
        let bundle_dir_name = entry
            .get("bundle_dir_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if bundle_dir_name.trim().is_empty() {
            continue;
        }

        let window = entry.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let tick_id = entry.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let frame_id = entry.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
        if frame_id < warmup_frames {
            continue;
        }

        let scale_factor = entry
            .get("scale_factor")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        if !(scale_factor.is_finite() && scale_factor > 0.0) {
            continue;
        }

        let file = entry
            .get("file")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if file.trim().is_empty() {
            continue;
        }

        let screenshot_path = out_dir
            .join("screenshots")
            .join(&bundle_dir_name)
            .join(&file);
        if !screenshot_path.is_file() {
            continue;
        }

        let bundle_dir = out_dir.join(&bundle_dir_name);
        let bundle_artifact_path = crate::resolve_bundle_artifact_path(&bundle_dir);
        if !bundle_artifact_path.is_file() {
            continue;
        }

        let bundle = if let Some(b) = bundles_cache.get(&bundle_dir_name) {
            b.clone()
        } else {
            let bytes = std::fs::read(&bundle_artifact_path).map_err(|e| e.to_string())?;
            let bundle: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            bundles_cache.insert(bundle_dir_name.clone(), bundle.clone());
            bundle
        };

        let semantics = crate::json_bundle::SemanticsResolver::new(&bundle);

        let bounds = match find_semantics_bounds_for_test_id(
            &bundle, &semantics, window, tick_id, frame_id, test_id,
        ) {
            Some(r) => r,
            None => match find_semantics_bounds_for_test_id_latest(
                &bundle,
                &semantics,
                window,
                warmup_frames,
                test_id,
            ) {
                Some(r) => r,
                None => continue,
            },
        };

        let img = image::ImageReader::open(&screenshot_path)
            .map_err(|e| {
                format!(
                    "failed to open screenshot png: {}: {e}",
                    screenshot_path.display()
                )
            })?
            .with_guessed_format()
            .map_err(|e| {
                format!(
                    "failed to read screenshot format: {}: {e}",
                    screenshot_path.display()
                )
            })?
            .decode()
            .map_err(|e| {
                format!(
                    "failed to decode screenshot png: {}: {e}",
                    screenshot_path.display()
                )
            })?
            .to_rgba8();

        let (img_w, img_h) = img.dimensions();
        let rect_px = rect_f_to_px(bounds, scale_factor, img_w, img_h);
        if rect_px.width_px() == 0 || rect_px.height_px() == 0 {
            continue;
        }

        let hash = hash_rgba_region(&img, rect_px);
        resolved.push(PixelCheckResolvedShot {
            bundle_dir_name,
            file,
            window,
            tick_id,
            frame_id,
            scale_factor,
            rect_px,
            hash,
        });
    }

    resolved.sort_by(|a, b| {
        a.tick_id
            .cmp(&b.tick_id)
            .then_with(|| a.frame_id.cmp(&b.frame_id))
            .then_with(|| a.file.cmp(&b.file))
    });

    let out_path = out_dir.join("check.pixels_changed.json");

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "pixels_changed",
        "out_dir": out_dir.display().to_string(),
        "test_id": test_id,
        "warmup_frames": warmup_frames,
        "resolved": resolved.iter().map(|s| serde_json::json!({
            "bundle_dir_name": s.bundle_dir_name,
            "file": s.file,
            "window": s.window,
            "tick_id": s.tick_id,
            "frame_id": s.frame_id,
            "scale_factor": s.scale_factor,
            "rect_px": { "x0": s.rect_px.x0, "y0": s.rect_px.y0, "x1": s.rect_px.x1, "y1": s.rect_px.y1 },
            "hash": format!("0x{:016x}", s.hash),
        })).collect::<Vec<_>>(),
    });
    let _ = write_json_value(&out_path, &payload);

    if resolved.len() < 2 {
        return Err(format!(
            "pixels changed check requires at least 2 resolved screenshots for test_id={test_id} (resolved={}, out_dir={})",
            resolved.len(),
            out_dir.display()
        ));
    }

    let first = &resolved[0];
    let last = &resolved[resolved.len() - 1];
    if first.hash != last.hash {
        return Ok(());
    }

    Err(format!(
        "pixels unchanged suspected for test_id={test_id} (hash=0x{hash:016x})\n  first: bundle={b0} file={f0} tick={t0} frame={fr0} rect_px=({x0},{y0})-({x1},{y1})\n  last:  bundle={b1} file={f1} tick={t1} frame={fr1} rect_px=({x2},{y2})-({x3},{y3})\n  evidence: {}",
        out_path.display(),
        hash = first.hash,
        b0 = first.bundle_dir_name,
        f0 = first.file,
        t0 = first.tick_id,
        fr0 = first.frame_id,
        x0 = first.rect_px.x0,
        y0 = first.rect_px.y0,
        x1 = first.rect_px.x1,
        y1 = first.rect_px.y1,
        b1 = last.bundle_dir_name,
        f1 = last.file,
        t1 = last.tick_id,
        fr1 = last.frame_id,
        x2 = last.rect_px.x0,
        y2 = last.rect_px.y0,
        x3 = last.rect_px.x1,
        y3 = last.rect_px.y1,
    ))
}

fn find_semantics_bounds_for_test_id(
    bundle: &serde_json::Value,
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    window: u64,
    tick_id: u64,
    frame_id: u64,
    test_id: &str,
) -> Option<RectF> {
    let windows = bundle.get("windows").and_then(|v| v.as_array())?;
    let w = windows
        .iter()
        .find(|w| w.get("window").and_then(|v| v.as_u64()) == Some(window))?;
    let snaps = w.get("snapshots").and_then(|v| v.as_array())?;

    let snap = snaps.iter().find(|s| {
        s.get("tick_id").and_then(|v| v.as_u64()) == Some(tick_id)
            && s.get("frame_id").and_then(|v| v.as_u64()) == Some(frame_id)
    })?;

    let node = crate::json_bundle::semantics_node_for_test_id(semantics, snap, test_id)?;

    let bounds = node.get("bounds")?;
    Some(RectF {
        x: bounds.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
        y: bounds.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
        w: bounds.get("w").and_then(|v| v.as_f64()).unwrap_or(0.0),
        h: bounds.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0),
    })
}

fn find_semantics_bounds_for_test_id_latest(
    bundle: &serde_json::Value,
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    window: u64,
    warmup_frames: u64,
    test_id: &str,
) -> Option<RectF> {
    let windows = bundle.get("windows").and_then(|v| v.as_array())?;
    let w = windows
        .iter()
        .find(|w| w.get("window").and_then(|v| v.as_u64()) == Some(window))?;
    let snaps = w.get("snapshots").and_then(|v| v.as_array())?;

    let ts = |s: &serde_json::Value| -> u64 {
        s.get("timestamp_unix_ms")
            .and_then(|v| v.as_u64())
            .or_else(|| s.get("timestamp_ms").and_then(|v| v.as_u64()))
            .unwrap_or(0)
    };

    let snap = snaps
        .iter()
        .filter(|s| crate::json_bundle::snapshot_frame_id(s) >= warmup_frames)
        .filter(|s| semantics.nodes(s).is_some())
        .max_by_key(|s| ts(s))
        .or_else(|| {
            snaps
                .iter()
                .filter(|s| semantics.nodes(s).is_some())
                .max_by_key(|s| ts(s))
        })
        .or_else(|| snaps.iter().max_by_key(|s| ts(s)))?;

    let node = crate::json_bundle::semantics_node_for_test_id(semantics, snap, test_id)?;

    let bounds = node.get("bounds")?;
    Some(RectF {
        x: bounds.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
        y: bounds.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
        w: bounds.get("w").and_then(|v| v.as_f64()).unwrap_or(0.0),
        h: bounds.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0),
    })
}

fn rect_f_to_px(bounds: RectF, scale_factor: f64, img_w: u32, img_h: u32) -> RectPx {
    let sx0 = (bounds.x * scale_factor).floor();
    let sy0 = (bounds.y * scale_factor).floor();
    let sx1 = ((bounds.x + bounds.w) * scale_factor).ceil();
    let sy1 = ((bounds.y + bounds.h) * scale_factor).ceil();

    let clamp = |v: f64, max: u32| -> u32 {
        if !v.is_finite() {
            return 0;
        }
        let v = v.max(0.0).min(max as f64);
        v as u32
    };

    let x0 = clamp(sx0, img_w);
    let y0 = clamp(sy0, img_h);
    let x1 = clamp(sx1, img_w);
    let y1 = clamp(sy1, img_h);

    RectPx { x0, y0, x1, y1 }
}

fn hash_rgba_region(img: &image::RgbaImage, rect: RectPx) -> u64 {
    // Stable, tiny hash for CI gates (not cryptographic).
    let mut h: u64 = 0xcbf29ce484222325;
    let prime: u64 = 0x100000001b3;

    let (w, _h_px) = img.dimensions();
    let bytes = img.as_raw();

    // Mix dimensions so two different rects are unlikely to collide.
    for b in rect.x0.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(prime);
    }
    for b in rect.y0.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(prime);
    }
    for b in rect.x1.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(prime);
    }
    for b in rect.y1.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(prime);
    }

    let row_bytes = (rect.width_px() as usize).saturating_mul(4);
    for y in rect.y0..rect.y1 {
        let start = (y as usize)
            .saturating_mul(w as usize)
            .saturating_add(rect.x0 as usize)
            .saturating_mul(4);
        let end = start.saturating_add(row_bytes).min(bytes.len());
        for &b in &bytes[start..end] {
            h ^= b as u64;
            h = h.wrapping_mul(prime);
        }
    }

    h
}

fn summarize_times_us(values: &[u64]) -> serde_json::Value {
    if values.is_empty() {
        return serde_json::json!({
            "min": 0,
            "p50": 0,
            "p95": 0,
            "max": 0,
        });
    }

    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let min = *sorted.first().unwrap_or(&0);
    let max = *sorted.last().unwrap_or(&0);
    let p50 = percentile_nearest_rank_sorted(&sorted, 0.50);
    let p95 = percentile_nearest_rank_sorted(&sorted, 0.95);

    serde_json::json!({
        "min": min,
        "p50": p50,
        "p95": p95,
        "max": max,
    })
}

fn percentile_nearest_rank_sorted(sorted: &[u64], percentile: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let percentile = percentile.clamp(0.0, 1.0);
    let n = sorted.len();
    let rank_1_based = (percentile * n as f64).ceil().max(1.0) as usize;
    let idx = rank_1_based.saturating_sub(1).min(n - 1);
    sorted[idx]
}

#[cfg(test)]
mod tests;
