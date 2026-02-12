#![recursion_limit = "512"]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use fret_diag_protocol::{DevtoolsBundleDumpedV1, DevtoolsSessionListV1, UiScriptResultV1};

use zip::write::FileOptions;

pub mod api;
pub mod artifacts;
mod cli;
mod compare;
pub mod devtools;
mod gates;
mod lint;
mod perf_seed_policy;
mod script_tooling;
mod shrink;
mod stats;
mod suite_summary;
pub mod transport;
mod util;

use compare::{
    CompareOptions, CompareReport, PerfThresholds, RenderdocDumpAttempt, apply_perf_baseline_floor,
    apply_perf_baseline_headroom, cargo_run_inject_feature, compare_bundles, ensure_env_var,
    find_latest_export_dir, maybe_launch_demo, normalize_repo_relative_path, read_latest_pointer,
    read_perf_baseline_file, resolve_threshold, run_fret_renderdoc_dump,
    scan_perf_threshold_failures, stop_launched_demo, wait_for_files_with_extensions,
};
use devtools::DevtoolsOps;
use gates::{
    RedrawHitchesGateResult, ResourceFootprintGateResult, ResourceFootprintThresholds,
    check_redraw_hitches_max_total_ms, check_resource_footprint_thresholds,
};
use lint::{LintOptions, lint_bundle_from_path};
use perf_seed_policy::{PerfBaselineSeed, PerfSeedMetric, ResolvedPerfBaselineSeedPolicy};
use script_tooling::{
    NormalizedScript, ScriptLintReport, ScriptSchemaReport, lint_scripts,
    normalize_script_from_path, validate_scripts,
};
use stats::{
    BundleStatsOptions, BundleStatsReport, BundleStatsSort, ScriptResultSummary,
    apply_pick_to_script, bundle_stats_from_path,
    check_bundle_for_chart_sampling_window_shifts_min, check_bundle_for_dock_drag_min,
    check_bundle_for_drag_cache_root_paint_only, check_bundle_for_gc_sweep_liveness,
    check_bundle_for_layout_fast_path_min, check_bundle_for_node_graph_cull_window_shifts_max,
    check_bundle_for_node_graph_cull_window_shifts_min, check_bundle_for_notify_hotspot_file_max,
    check_bundle_for_overlay_synthesis_min, check_bundle_for_prepaint_actions_min,
    check_bundle_for_retained_vlist_attach_detach_max,
    check_bundle_for_retained_vlist_keep_alive_budget,
    check_bundle_for_retained_vlist_keep_alive_reuse_min,
    check_bundle_for_retained_vlist_reconcile_no_notify_min,
    check_bundle_for_semantics_changed_repainted, check_bundle_for_stale_paint,
    check_bundle_for_stale_scene, check_bundle_for_ui_gallery_code_editor_a11y_composition,
    check_bundle_for_ui_gallery_code_editor_a11y_composition_drag,
    check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap,
    check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_scroll,
    check_bundle_for_ui_gallery_code_editor_a11y_selection,
    check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap,
    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection,
    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll,
    check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
    check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed,
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit,
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present,
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped,
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations,
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed,
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap,
    check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low,
    check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit,
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present,
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped,
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations,
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed,
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap,
    check_bundle_for_ui_gallery_code_editor_torture_marker_present,
    check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo,
    check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits,
    check_bundle_for_ui_gallery_code_editor_word_boundary,
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition,
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap,
    check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits,
    check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds,
    check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit,
    check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present,
    check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap,
    check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable,
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit,
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable,
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_present,
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap,
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable,
    check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click,
    check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits,
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable,
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable,
    check_bundle_for_ui_gallery_markdown_editor_source_word_boundary,
    check_bundle_for_ui_gallery_web_ime_bridge_enabled, check_bundle_for_view_cache_reuse_min,
    check_bundle_for_view_cache_reuse_stable_min, check_bundle_for_viewport_capture_min,
    check_bundle_for_viewport_input_min, check_bundle_for_vlist_policy_key_stable,
    check_bundle_for_vlist_visible_range_refreshes_max,
    check_bundle_for_vlist_visible_range_refreshes_min,
    check_bundle_for_vlist_window_shifts_explainable,
    check_bundle_for_vlist_window_shifts_have_prepaint_actions,
    check_bundle_for_vlist_window_shifts_kind_max,
    check_bundle_for_vlist_window_shifts_non_retained_max, check_bundle_for_wheel_scroll,
    check_bundle_for_wheel_scroll_hit_changes, check_bundle_for_windowed_rows_offset_changes_min,
    check_bundle_for_windowed_rows_visible_start_changes_repainted,
    check_report_for_hover_layout_invalidations, clear_script_result_files,
    report_pick_result_and_exit, report_result_and_exit, run_pick_and_wait, run_script_and_wait,
    wait_for_failure_dump_bundle, write_pick_script,
};
use util::{now_unix_ms, read_json_value, touch, write_json_value, write_script};

#[derive(Debug, Clone)]
struct ReproPackItem {
    script_path: PathBuf,
    bundle_json: PathBuf,
}

#[derive(Debug)]
struct LaunchedDemo {
    child: Child,
    launched_unix_ms: u64,
    launched_instant: Instant,
    launch_cmd: Vec<String>,
}

pub fn diag_cmd(args: Vec<String>) -> Result<(), String> {
    let mut out_dir: Option<PathBuf> = None;
    let mut trigger_path: Option<PathBuf> = None;
    let mut pack_out: Option<PathBuf> = None;
    let mut pack_include_root_artifacts: bool = false;
    let mut pack_include_triage: bool = false;
    let mut pack_include_screenshots: bool = false;
    let mut pack_after_run: bool = false;
    let mut triage_out: Option<PathBuf> = None;
    let mut lint_out: Option<PathBuf> = None;
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
    let mut timeout_ms: u64 = 30_000;
    let mut poll_ms: u64 = 50;
    let mut stats_top: usize = 5;
    let mut sort_override: Option<BundleStatsSort> = None;
    let mut stats_json: bool = false;
    let mut warmup_frames: u64 = 0;
    let mut lint_all_test_ids_bounds: bool = false;
    let mut lint_eps_px: f32 = 0.5;
    let mut suite_lint: bool = true;
    let mut perf_repeat: u64 = 1;
    let mut reuse_launch: bool = false;
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
    let mut max_pointer_move_dispatch_us: Option<u64> = None;
    let mut max_pointer_move_hit_test_us: Option<u64> = None;
    let mut max_pointer_move_global_changes: Option<u64> = None;
    let mut min_run_paint_cache_hit_test_only_replay_allowed_max: Option<u64> = None;
    let mut max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Option<u64> = None;
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
    let mut suite_script_inputs: Vec<String> = Vec::new();

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
                lint_out = Some(p);
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
            other if other.starts_with('-') => return Err(format!("unknown diag flag: {other}")),
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

    match sub.as_str() {
        "path" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            println!("{}", resolved_trigger_path.display());
            Ok(())
        }
        "poke" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            touch(&resolved_trigger_path)?;
            println!("{}", resolved_trigger_path.display());
            Ok(())
        }
        "latest" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            if let Some(path) = read_latest_pointer(&resolved_out_dir)
                .or_else(|| find_latest_export_dir(&resolved_out_dir))
            {
                println!("{}", path.display());
                return Ok(());
            }
            Err(format!(
                "no diagnostics bundle found under {}",
                resolved_out_dir.display()
            ))
        }
        "pack" => {
            if rest.len() > 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let bundle_dir = match rest.first() {
                Some(src) => {
                    let src = resolve_path(&workspace_root, PathBuf::from(src));
                    resolve_bundle_root_dir(&src)?
                }
                None => read_latest_pointer(&resolved_out_dir)
                    .or_else(|| find_latest_export_dir(&resolved_out_dir))
                    .ok_or_else(|| {
                        format!(
                            "no diagnostics bundle found under {} (try: fretboard diag pack ./target/fret-diag/<timestamp>)",
                            resolved_out_dir.display()
                        )
                    })?,
            };

            let bundle_dir = resolve_bundle_root_dir(&bundle_dir)?;
            let out = pack_out
                .map(|p| resolve_path(&workspace_root, p))
                .unwrap_or_else(|| default_pack_out_path(&resolved_out_dir, &bundle_dir));

            let artifacts_root = if bundle_dir.starts_with(&resolved_out_dir) {
                resolved_out_dir.clone()
            } else {
                bundle_dir
                    .parent()
                    .unwrap_or(&resolved_out_dir)
                    .to_path_buf()
            };

            pack_bundle_dir_to_zip(
                &bundle_dir,
                &out,
                pack_include_root_artifacts,
                pack_include_triage,
                pack_include_screenshots,
                false,
                false,
                &artifacts_root,
                stats_top,
                sort_override.unwrap_or(BundleStatsSort::Invalidation),
                warmup_frames,
            )?;
            println!("{}", out.display());
            Ok(())
        }
        "triage" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing bundle path (try: fretboard diag triage ./target/fret-diag/1234/bundle.json)"
                        .to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            let bundle_path = resolve_bundle_json_path(&src);
            let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);

            let report = bundle_stats_from_path(
                &bundle_path,
                stats_top,
                sort,
                BundleStatsOptions { warmup_frames },
            )?;
            let payload = triage_json_from_stats(&bundle_path, &report, sort, warmup_frames);

            let out = triage_out
                .map(|p| resolve_path(&workspace_root, p))
                .unwrap_or_else(|| default_triage_out_path(&bundle_path));

            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let pretty =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
            std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

            if stats_json {
                println!("{pretty}");
            } else {
                println!("{}", out.display());
            }
            Ok(())
        }
        "lint" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing bundle path (try: fretboard diag lint ./target/fret-diag/1234/bundle.json)"
                        .to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            let bundle_path = resolve_bundle_json_path(&src);

            let report = lint_bundle_from_path(
                &bundle_path,
                warmup_frames,
                LintOptions {
                    all_test_ids_bounds: lint_all_test_ids_bounds,
                    eps_px: lint_eps_px,
                },
            )?;

            let out = lint_out
                .map(|p| resolve_path(&workspace_root, p))
                .unwrap_or_else(|| default_lint_out_path(&bundle_path));

            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let pretty =
                serde_json::to_string_pretty(&report.payload).unwrap_or_else(|_| "{}".to_string());
            std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

            if stats_json {
                println!("{pretty}");
            } else {
                println!("{}", out.display());
            }

            if report.error_issues > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
        "script" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            let Some(op) = rest.first().map(|s| s.as_str()) else {
                return Err("missing script subcommand or script path (try: fretboard diag script ./script.json | fretboard diag script normalize ./script.json)".to_string());
            };

            let shrink_flags_used = shrink_out.is_some()
                || shrink_any_fail
                || shrink_match_reason_code.is_some()
                || shrink_match_reason.is_some()
                || shrink_min_steps != 1
                || shrink_max_iters != 200;
            if shrink_flags_used && op != "shrink" {
                return Err(
                    "--shrink-* flags are only supported with `diag script shrink`".to_string(),
                );
            }

            match op {
                "normalize" => {
                    if script_tool_check && script_tool_write {
                        return Err("--check cannot be combined with --write".to_string());
                    }
                    if script_tool_check_out.is_some() {
                        return Err(
                            "--check-out is not supported with `diag script normalize`".to_string()
                        );
                    }

                    let inputs: Vec<String> = rest[1..].iter().cloned().collect();
                    if inputs.is_empty() {
                        return Err("missing script path (try: fretboard diag script normalize ./script.json)".to_string());
                    }
                    if inputs.len() != 1 && !script_tool_check && !script_tool_write {
                        return Err("normalize expects exactly one script unless --check or --write is used".to_string());
                    }

                    let scripts = expand_script_inputs(&workspace_root, &inputs)?;
                    if scripts.len() != 1 && !script_tool_check && !script_tool_write {
                        return Err("normalize expects exactly one script unless --check or --write is used".to_string());
                    }

                    let mut any_changed = false;
                    for src in scripts {
                        let NormalizedScript {
                            normalized,
                            changed,
                        } = normalize_script_from_path(&src)?;

                        if script_tool_check {
                            if changed {
                                any_changed = true;
                                eprintln!("not normalized: {}", src.display());
                            } else {
                                println!("{}", src.display());
                            }
                            continue;
                        }

                        if script_tool_write {
                            if changed {
                                any_changed = true;
                                std::fs::write(&src, normalized.as_bytes())
                                    .map_err(|e| e.to_string())?;
                            }
                            println!("{}", src.display());
                            continue;
                        }

                        print!("{normalized}");
                    }

                    if script_tool_check && any_changed {
                        std::process::exit(1);
                    }
                    Ok(())
                }
                "validate" => {
                    if script_tool_check || script_tool_write {
                        return Err(
                            "--check/--write are not supported with `diag script validate`"
                                .to_string(),
                        );
                    }

                    let inputs: Vec<String> = rest[1..].iter().cloned().collect();
                    if inputs.is_empty() {
                        return Err("missing script path (try: fretboard diag script validate ./script.json)".to_string());
                    }
                    let scripts = expand_script_inputs(&workspace_root, &inputs)?;

                    let ScriptSchemaReport {
                        payload,
                        error_scripts,
                    } = validate_scripts(&scripts);

                    let out = script_tool_check_out
                        .map(|p| resolve_path(&workspace_root, p))
                        .unwrap_or_else(|| resolved_out_dir.join("check.script_schema.json"));
                    if let Some(parent) = out.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let pretty =
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
                    std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

                    if stats_json {
                        println!("{pretty}");
                    } else {
                        println!("{}", out.display());
                    }

                    if error_scripts > 0 {
                        std::process::exit(1);
                    }
                    Ok(())
                }
                "lint" => {
                    if script_tool_check || script_tool_write {
                        return Err(
                            "--check/--write are not supported with `diag script lint`".to_string()
                        );
                    }

                    let inputs: Vec<String> = rest[1..].iter().cloned().collect();
                    if inputs.is_empty() {
                        return Err(
                            "missing script path (try: fretboard diag script lint ./script.json)"
                                .to_string(),
                        );
                    }
                    let scripts = expand_script_inputs(&workspace_root, &inputs)?;

                    let ScriptLintReport {
                        payload,
                        error_scripts,
                    } = lint_scripts(&scripts);

                    let out = script_tool_check_out
                        .map(|p| resolve_path(&workspace_root, p))
                        .unwrap_or_else(|| resolved_out_dir.join("check.script_lint.json"));
                    if let Some(parent) = out.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let pretty =
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
                    std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

                    if stats_json {
                        println!("{pretty}");
                    } else {
                        println!("{}", out.display());
                    }

                    if error_scripts > 0 {
                        std::process::exit(1);
                    }
                    Ok(())
                }
                "shrink" => {
                    if script_tool_check || script_tool_write || script_tool_check_out.is_some() {
                        return Err("--check/--write/--check-out are not supported with `diag script shrink`".to_string());
                    }
                    let inputs: Vec<String> = rest[1..].iter().cloned().collect();
                    if inputs.is_empty() {
                        return Err(
                            "missing script path (try: fretboard diag script shrink ./script.json)"
                                .to_string(),
                        );
                    }
                    if inputs.len() != 1 {
                        return Err(format!("unexpected arguments: {}", inputs[1..].join(" ")));
                    }
                    if launch.is_some() && !reuse_launch {
                        return Err("`diag script shrink` requires --reuse-launch when using --launch (to avoid restarting for every attempt)".to_string());
                    }

                    #[derive(Debug, Clone)]
                    enum ActionScript {
                        V1(fret_diag_protocol::UiActionScriptV1),
                        V2(fret_diag_protocol::UiActionScriptV2),
                    }

                    impl ActionScript {
                        fn steps_len(&self) -> usize {
                            match self {
                                Self::V1(s) => s.steps.len(),
                                Self::V2(s) => s.steps.len(),
                            }
                        }

                        fn keep_steps(&self, keep: &[usize]) -> Self {
                            match self {
                                Self::V1(s) => {
                                    let steps = keep
                                        .iter()
                                        .filter_map(|&i| s.steps.get(i).cloned())
                                        .collect();
                                    Self::V1(fret_diag_protocol::UiActionScriptV1 {
                                        schema_version: 1,
                                        meta: s.meta.clone(),
                                        steps,
                                    })
                                }
                                Self::V2(s) => {
                                    let steps = keep
                                        .iter()
                                        .filter_map(|&i| s.steps.get(i).cloned())
                                        .collect();
                                    Self::V2(fret_diag_protocol::UiActionScriptV2 {
                                        schema_version: 2,
                                        meta: s.meta.clone(),
                                        steps,
                                    })
                                }
                            }
                        }

                        fn to_pretty_json(&self) -> Result<String, String> {
                            let mut value = match self {
                                Self::V1(s) => {
                                    serde_json::to_value(s).map_err(|e| e.to_string())?
                                }
                                Self::V2(s) => {
                                    serde_json::to_value(s).map_err(|e| e.to_string())?
                                }
                            };
                            script_tooling::canonicalize_json_value(&mut value);
                            let mut s =
                                serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
                            s.push('\n');
                            Ok(s)
                        }
                    }

                    fn read_action_script(path: &Path) -> Result<ActionScript, String> {
                        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
                        let value: serde_json::Value =
                            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                        let schema_version = value
                            .get("schema_version")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                            .min(u32::MAX as u64)
                            as u32;
                        match schema_version {
                            1 => Ok(ActionScript::V1(
                                serde_json::from_value(value).map_err(|e| e.to_string())?,
                            )),
                            2 => Ok(ActionScript::V2(
                                serde_json::from_value(value).map_err(|e| e.to_string())?,
                            )),
                            _ => Err(format!(
                                "unknown script schema_version (expected 1 or 2): {}",
                                schema_version
                            )),
                        }
                    }

                    fn matches_failure(
                        s: &ScriptResultSummary,
                        any_fail: bool,
                        reason_code: Option<&str>,
                        reason: Option<&str>,
                    ) -> bool {
                        if s.stage.as_deref() != Some("failed") {
                            return false;
                        }
                        if any_fail {
                            return true;
                        }
                        if let Some(code) = reason_code {
                            return s.reason_code.as_deref() == Some(code);
                        }
                        if let Some(r) = reason {
                            return s.reason.as_deref() == Some(r);
                        }
                        true
                    }

                    let scripts = expand_script_inputs(&workspace_root, &inputs)?;
                    if scripts.len() != 1 {
                        return Err("shrink expects exactly one script input".to_string());
                    }
                    let src = scripts.into_iter().next().unwrap();

                    let shrink_dir = resolved_out_dir.join("shrink");
                    std::fs::create_dir_all(&shrink_dir).map_err(|e| e.to_string())?;

                    let out_path = shrink_out
                        .clone()
                        .map(|p| resolve_path(&workspace_root, p))
                        .unwrap_or_else(|| shrink_dir.join("script.min.json"));
                    let summary_path = shrink_dir.join("shrink.summary.json");
                    let candidate_path = shrink_dir.join("script.candidate.json");

                    let script = read_action_script(&src)?;
                    let total_steps = script.steps_len();
                    if total_steps == 0 && shrink_min_steps > 0 {
                        return Err("script has no steps; nothing to shrink".to_string());
                    }

                    let wants_screenshots = script_requests_screenshots(&src);
                    let shrink_launch_env = launch_env.clone();
                    let mut child = maybe_launch_demo(
                        &launch,
                        &shrink_launch_env,
                        &workspace_root,
                        &resolved_out_dir,
                        &resolved_ready_path,
                        &resolved_exit_path,
                        wants_screenshots,
                        timeout_ms,
                        poll_ms,
                    )?;

                    let baseline = run_script_and_wait(
                        &src,
                        &resolved_script_path,
                        &resolved_script_trigger_path,
                        &resolved_script_result_path,
                        &resolved_script_result_trigger_path,
                        timeout_ms,
                        poll_ms,
                    )?;

                    if baseline.stage.as_deref() != Some("failed") {
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                        return Err(format!(
                            "baseline script did not fail (stage={:?}); shrink expects a failing script",
                            baseline.stage
                        ));
                    }

                    let desired_reason_code = shrink_match_reason_code
                        .as_deref()
                        .or(baseline.reason_code.as_deref());
                    let desired_reason = shrink_match_reason
                        .as_deref()
                        .or(baseline.reason.as_deref());

                    let mut attempts_total: u64 = 0;
                    let mut attempts_errors: u64 = 0;
                    let mut last_error: Option<String> = None;

                    let min_steps = usize::try_from(shrink_min_steps)
                        .unwrap_or(usize::MAX)
                        .min(total_steps);
                    let (keep, reductions, iters) = shrink::ddmin_keep_indices(
                        total_steps,
                        min_steps,
                        shrink_max_iters,
                        |keep| {
                            attempts_total += 1;
                            let candidate = script.keep_steps(keep);
                            let pretty = match candidate.to_pretty_json() {
                                Ok(s) => s,
                                Err(err) => {
                                    attempts_errors += 1;
                                    last_error = Some(err);
                                    return false;
                                }
                            };
                            if let Err(err) = std::fs::write(&candidate_path, pretty.as_bytes()) {
                                attempts_errors += 1;
                                last_error = Some(err.to_string());
                                return false;
                            }

                            match run_script_and_wait(
                                &candidate_path,
                                &resolved_script_path,
                                &resolved_script_trigger_path,
                                &resolved_script_result_path,
                                &resolved_script_result_trigger_path,
                                timeout_ms,
                                poll_ms,
                            ) {
                                Ok(s) => matches_failure(
                                    &s,
                                    shrink_any_fail,
                                    desired_reason_code,
                                    desired_reason,
                                ),
                                Err(err) => {
                                    attempts_errors += 1;
                                    last_error = Some(err);
                                    false
                                }
                            }
                        },
                    );

                    let minimized = script.keep_steps(&keep);
                    let minimized_pretty = minimized.to_pretty_json()?;
                    std::fs::write(&out_path, minimized_pretty.as_bytes())
                        .map_err(|e| e.to_string())?;

                    let final_result = run_script_and_wait(
                        &out_path,
                        &resolved_script_path,
                        &resolved_script_trigger_path,
                        &resolved_script_result_path,
                        &resolved_script_result_trigger_path,
                        timeout_ms,
                        poll_ms,
                    )?;

                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);

                    let ok = matches_failure(
                        &final_result,
                        shrink_any_fail,
                        desired_reason_code,
                        desired_reason,
                    );
                    if !ok {
                        return Err(format!(
                            "minimized script does not reproduce baseline failure (stage={:?} reason_code={:?} reason={:?})",
                            final_result.stage, final_result.reason_code, final_result.reason
                        ));
                    }

                    let keep_set: std::collections::BTreeSet<usize> =
                        keep.iter().copied().collect();
                    let removed: Vec<usize> =
                        (0..total_steps).filter(|i| !keep_set.contains(i)).collect();
                    let reductions_json: Vec<serde_json::Value> = reductions
                        .into_iter()
                        .map(|r| {
                            serde_json::json!({
                                "granularity": r.granularity,
                                "kept_len": r.kept_len,
                                "removed": r.removed,
                            })
                        })
                        .collect();

                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "status": "passed",
                        "script": src.display().to_string(),
                        "out": out_path.display().to_string(),
                        "params": {
                            "min_steps": shrink_min_steps,
                            "max_iters": shrink_max_iters,
                            "any_fail": shrink_any_fail,
                            "match_reason_code": desired_reason_code,
                            "match_reason": desired_reason,
                        },
                        "baseline": {
                            "run_id": baseline.run_id,
                            "stage": baseline.stage,
                            "step_index": baseline.step_index,
                            "reason_code": baseline.reason_code,
                            "reason": baseline.reason,
                            "last_bundle_dir": baseline.last_bundle_dir,
                        },
                        "final": {
                            "run_id": final_result.run_id,
                            "stage": final_result.stage,
                            "step_index": final_result.step_index,
                            "reason_code": final_result.reason_code,
                            "reason": final_result.reason,
                            "last_bundle_dir": final_result.last_bundle_dir,
                        },
                        "steps": {
                            "original": total_steps,
                            "kept": keep.len(),
                            "removed": removed.len(),
                            "removed_indices": removed,
                        },
                        "search": {
                            "iters": iters,
                            "attempts_total": attempts_total,
                            "attempts_errors": attempts_errors,
                            "last_error": last_error,
                            "reductions": reductions_json,
                        },
                    });

                    if let Some(parent) = summary_path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let pretty =
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
                    std::fs::write(&summary_path, pretty.as_bytes()).map_err(|e| e.to_string())?;

                    if stats_json {
                        println!("{pretty}");
                    } else {
                        println!("{}", out_path.display());
                    }
                    Ok(())
                }
                _ => {
                    let Some(src) = rest.first().cloned() else {
                        return Err(
                            "missing script path (try: fretboard diag script ./script.json)"
                                .to_string(),
                        );
                    };
                    if rest.len() != 1 {
                        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
                    }

                    let src = resolve_path(&workspace_root, PathBuf::from(src));
                    write_script(&src, &resolved_script_path)?;
                    touch(&resolved_script_trigger_path)?;
                    println!("{}", resolved_script_trigger_path.display());
                    Ok(())
                }
            }
        }
        "run" => {
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing script path (try: fretboard diag run ./script.json)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            struct StopLaunchedDemoOnDrop<'a> {
                child: &'a mut Option<LaunchedDemo>,
                exit_path: &'a Path,
                poll_ms: u64,
            }

            impl Drop for StopLaunchedDemoOnDrop<'_> {
                fn drop(&mut self) {
                    let _ = stop_launched_demo(self.child, self.exit_path, self.poll_ms);
                }
            }

            let wants_pack = pack_after_run
                || pack_out.is_some()
                || pack_include_root_artifacts
                || pack_include_triage
                || pack_include_screenshots;

            let mut pack_defaults = (
                pack_include_root_artifacts,
                pack_include_triage,
                pack_include_screenshots,
            );
            if pack_after_run && !pack_defaults.0 && !pack_defaults.1 && !pack_defaults.2 {
                pack_defaults = (true, true, true);
            }

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            let use_devtools_ws = devtools_ws_url.is_some()
                || devtools_token.is_some()
                || devtools_session_id.is_some();
            if use_devtools_ws {
                if launch.is_some() || reuse_launch {
                    return Err(
                        "--launch/--reuse-launch is not supported with --devtools-ws-url"
                            .to_string(),
                    );
                }
                if wants_pack {
                    return Err("--pack is not supported with --devtools-ws-url yet".to_string());
                }

                let ws_url = devtools_ws_url.clone().ok_or_else(|| {
                    "missing --devtools-ws-url (required when using DevTools WS transport)"
                        .to_string()
                })?;
                let token = devtools_token.clone().ok_or_else(|| {
                    "missing --devtools-token (required when using DevTools WS transport)"
                        .to_string()
                })?;

                std::fs::create_dir_all(&resolved_out_dir).map_err(|e| e.to_string())?;
                let script_json =
                    serde_json::from_slice(&std::fs::read(&src).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;

                let wants_post_run_checks = check_stale_paint_test_id.is_some()
                    || check_stale_scene_test_id.is_some()
                    || check_idle_no_paint_min.is_some()
                    || check_pixels_changed_test_id.is_some()
                    || check_ui_gallery_code_editor_torture_marker_present
                    || check_ui_gallery_code_editor_torture_undo_redo
                    || check_ui_gallery_code_editor_torture_geom_fallbacks_low
                    || check_ui_gallery_code_editor_torture_read_only_blocks_edits
                    || check_ui_gallery_markdown_editor_source_read_only_blocks_edits
                    || check_ui_gallery_markdown_editor_source_disabled_blocks_edits
                    || check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable
                    || check_ui_gallery_markdown_editor_source_word_boundary
                    || check_ui_gallery_web_ime_bridge_enabled
                    || check_ui_gallery_markdown_editor_source_line_boundary_triple_click
                    || check_ui_gallery_markdown_editor_source_a11y_composition
                    || check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap
                    || check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable
                    || check_ui_gallery_markdown_editor_source_folds_toggle_stable
                    || check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds
                    || check_ui_gallery_markdown_editor_source_folds_placeholder_present
                    || check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap
                    || check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit
                    || check_ui_gallery_markdown_editor_source_inlays_toggle_stable
                    || check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable
                    || check_ui_gallery_markdown_editor_source_inlays_present
                    || check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap
                    || check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit
                    || check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed
                    || check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed
                    || check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed
                    || check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll
                    || check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap
                    || check_ui_gallery_code_editor_torture_inlays_present
                    || check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit
                    || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped
                    || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations
                    || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed
                    || check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap
                    || check_ui_gallery_code_editor_word_boundary
                    || check_ui_gallery_code_editor_a11y_selection
                    || check_ui_gallery_code_editor_a11y_composition
                    || check_ui_gallery_code_editor_a11y_selection_wrap
                    || check_ui_gallery_code_editor_a11y_composition_wrap
                    || check_ui_gallery_code_editor_a11y_composition_wrap_scroll
                    || check_semantics_changed_repainted
                    || check_wheel_scroll_test_id.is_some()
                    || check_wheel_scroll_hit_changes_test_id.is_some()
                    || check_prepaint_actions_min.is_some()
                    || check_chart_sampling_window_shifts_min.is_some()
                    || check_node_graph_cull_window_shifts_min.is_some()
                    || check_node_graph_cull_window_shifts_max.is_some()
                    || check_vlist_visible_range_refreshes_min.is_some()
                    || check_vlist_visible_range_refreshes_max.is_some()
                    || check_vlist_window_shifts_explainable
                    || check_vlist_window_shifts_have_prepaint_actions
                    || check_vlist_window_shifts_non_retained_max.is_some()
                    || check_vlist_window_shifts_prefetch_max.is_some()
                    || check_vlist_window_shifts_escape_max.is_some()
                    || check_vlist_policy_key_stable
                    || check_windowed_rows_offset_changes_min.is_some()
                    || check_windowed_rows_visible_start_changes_repainted
                    || check_layout_fast_path_min.is_some()
                    || check_drag_cache_root_paint_only_test_id.is_some()
                    || check_hover_layout_max.is_some()
                    || check_gc_sweep_liveness
                    || !check_notify_hotspot_file_max.is_empty()
                    || check_view_cache_reuse_min.is_some()
                    || check_view_cache_reuse_stable_min.is_some()
                    || check_overlay_synthesis_min.is_some()
                    || check_viewport_input_min.is_some()
                    || check_dock_drag_min.is_some()
                    || check_viewport_capture_min.is_some()
                    || check_retained_vlist_reconcile_no_notify_min.is_some()
                    || check_retained_vlist_attach_detach_max.is_some()
                    || check_retained_vlist_keep_alive_reuse_min.is_some()
                    || check_retained_vlist_keep_alive_budget.is_some();

                let (result, bundle_path) = run_script_over_devtools_ws(
                    &resolved_out_dir,
                    ws_url.as_str(),
                    token.as_str(),
                    devtools_session_id.as_deref(),
                    script_json,
                    wants_post_run_checks,
                    timeout_ms,
                    poll_ms,
                )?;

                let _ = write_json_value(
                    &resolved_script_result_path,
                    &serde_json::to_value(&result).unwrap_or_else(|_| serde_json::json!({})),
                );

                if !matches!(result.stage, fret_diag_protocol::UiScriptStageV1::Passed) {
                    eprintln!(
                        "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                        src.display(),
                        result.run_id,
                        result.step_index.unwrap_or(0),
                        result.reason.as_deref().unwrap_or("unknown"),
                        result.last_bundle_dir.as_deref().unwrap_or("")
                    );
                    std::process::exit(1);
                }

                if wants_post_run_checks {
                    let Some(bundle_path) = bundle_path.as_ref() else {
                        return Err(
                            "script passed but no bundle.json was captured (required for post-run checks)"
                                .to_string(),
                        );
                    };
                    apply_post_run_checks(
                        bundle_path,
                        &resolved_out_dir,
                        check_idle_no_paint_min,
                        check_stale_paint_test_id.as_deref(),
                        check_stale_paint_eps,
                        check_stale_scene_test_id.as_deref(),
                        check_stale_scene_eps,
                        check_pixels_changed_test_id.as_deref(),
                        check_ui_gallery_code_editor_torture_marker_present,
                        check_ui_gallery_code_editor_torture_undo_redo,
                        check_ui_gallery_code_editor_torture_geom_fallbacks_low,
                        check_ui_gallery_code_editor_torture_read_only_blocks_edits,
                        check_ui_gallery_markdown_editor_source_read_only_blocks_edits,
                        check_ui_gallery_markdown_editor_source_disabled_blocks_edits,
                        check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable,
                        check_ui_gallery_markdown_editor_source_word_boundary,
                        check_ui_gallery_web_ime_bridge_enabled,
                        check_ui_gallery_markdown_editor_source_line_boundary_triple_click,
                        check_ui_gallery_markdown_editor_source_a11y_composition,
                        check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap,
                        check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable,
                        check_ui_gallery_markdown_editor_source_folds_toggle_stable,
                        check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds,
                        check_ui_gallery_markdown_editor_source_folds_placeholder_present,
                        check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap,
                        check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit,
                        check_ui_gallery_markdown_editor_source_inlays_toggle_stable,
                        check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable,
                        check_ui_gallery_markdown_editor_source_inlays_present,
                        check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap,
                        check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit,
                        check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed,
                        check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed,
                        check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
                        check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll,
                        check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap,
                        check_ui_gallery_code_editor_torture_inlays_present,
                        check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit,
                        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped,
                        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations,
                        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed,
                        check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap,
                        check_ui_gallery_code_editor_word_boundary,
                        check_ui_gallery_code_editor_a11y_selection,
                        check_ui_gallery_code_editor_a11y_composition,
                        check_ui_gallery_code_editor_a11y_selection_wrap,
                        check_ui_gallery_code_editor_a11y_composition_wrap,
                        check_ui_gallery_code_editor_a11y_composition_wrap_scroll,
                        check_ui_gallery_code_editor_a11y_composition_drag,
                        check_semantics_changed_repainted,
                        dump_semantics_changed_repainted_json,
                        check_wheel_scroll_test_id.as_deref(),
                        check_wheel_scroll_hit_changes_test_id.as_deref(),
                        check_prepaint_actions_min,
                        check_chart_sampling_window_shifts_min,
                        check_node_graph_cull_window_shifts_min,
                        check_node_graph_cull_window_shifts_max,
                        check_vlist_visible_range_refreshes_min,
                        check_vlist_visible_range_refreshes_max,
                        check_vlist_window_shifts_explainable,
                        check_vlist_window_shifts_have_prepaint_actions,
                        check_vlist_window_shifts_non_retained_max,
                        check_vlist_window_shifts_prefetch_max,
                        check_vlist_window_shifts_escape_max,
                        check_vlist_policy_key_stable,
                        check_windowed_rows_offset_changes_min,
                        check_windowed_rows_offset_changes_eps,
                        check_windowed_rows_visible_start_changes_repainted,
                        check_layout_fast_path_min,
                        check_drag_cache_root_paint_only_test_id.as_deref(),
                        check_hover_layout_max,
                        check_gc_sweep_liveness,
                        &check_notify_hotspot_file_max,
                        check_view_cache_reuse_stable_min,
                        check_view_cache_reuse_min,
                        check_overlay_synthesis_min,
                        check_viewport_input_min,
                        check_dock_drag_min,
                        check_viewport_capture_min,
                        check_retained_vlist_reconcile_no_notify_min,
                        check_retained_vlist_attach_detach_max,
                        check_retained_vlist_keep_alive_reuse_min,
                        check_retained_vlist_keep_alive_budget,
                        warmup_frames,
                    )?;
                }
                return Ok(());
            }
            let script_wants_screenshots = script_requests_screenshots(&src);
            let mut run_launch_env = launch_env.clone();
            let _ = ensure_env_var(&mut run_launch_env, "FRET_DIAG_RENDERER_PERF", "1");
            let mut child = maybe_launch_demo(
                &launch,
                &run_launch_env,
                &workspace_root,
                &resolved_out_dir,
                &resolved_ready_path,
                &resolved_exit_path,
                pack_defaults.2
                    || check_pixels_changed_test_id.is_some()
                    || script_wants_screenshots,
                timeout_ms,
                poll_ms,
            )?;
            let _stop_guard = StopLaunchedDemoOnDrop {
                child: &mut child,
                exit_path: &resolved_exit_path,
                poll_ms,
            };

            let required_caps = script_required_capabilities(&src);
            if !required_caps.is_empty() {
                let available_caps = read_filesystem_capabilities(&resolved_out_dir);
                gate_required_capabilities(
                    &resolved_out_dir.join("check.capabilities.json"),
                    &required_caps,
                    &available_caps,
                )?;
            }
            let mut result = run_script_and_wait(
                &src,
                &resolved_script_path,
                &resolved_script_trigger_path,
                &resolved_script_result_path,
                &resolved_script_result_trigger_path,
                timeout_ms,
                poll_ms,
            );
            if let Ok(summary) = &result
                && summary.stage.as_deref() == Some("failed")
            {
                if let Some(dir) =
                    wait_for_failure_dump_bundle(&resolved_out_dir, summary, timeout_ms, poll_ms)
                {
                    if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                        if let Ok(summary) = result.as_mut() {
                            summary.last_bundle_dir = Some(name.to_string());
                        }
                    }
                }
            }
            let result = result?;
            if result.stage.as_deref() == Some("passed") {
                if check_stale_paint_test_id.is_some()
                    || check_stale_scene_test_id.is_some()
                    || check_idle_no_paint_min.is_some()
                    || check_pixels_changed_test_id.is_some()
                    || check_ui_gallery_code_editor_torture_marker_present
                    || check_ui_gallery_code_editor_torture_undo_redo
                    || check_ui_gallery_code_editor_torture_geom_fallbacks_low
                    || check_ui_gallery_code_editor_torture_read_only_blocks_edits
                    || check_ui_gallery_markdown_editor_source_read_only_blocks_edits
                    || check_ui_gallery_markdown_editor_source_disabled_blocks_edits
                    || check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable
                    || check_ui_gallery_markdown_editor_source_word_boundary
                    || check_ui_gallery_web_ime_bridge_enabled
                    || check_ui_gallery_markdown_editor_source_line_boundary_triple_click
                    || check_ui_gallery_markdown_editor_source_a11y_composition
                    || check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap
                    || check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable
                    || check_ui_gallery_markdown_editor_source_folds_toggle_stable
                    || check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds
                    || check_ui_gallery_markdown_editor_source_folds_placeholder_present
                    || check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap
                    || check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit
                    || check_ui_gallery_markdown_editor_source_inlays_toggle_stable
                    || check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable
                    || check_ui_gallery_markdown_editor_source_inlays_present
                    || check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap
                    || check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit
                    || check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap
                    || check_ui_gallery_code_editor_torture_inlays_present
                    || check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit
                    || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped
                    || check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap
                    || check_ui_gallery_code_editor_word_boundary
                    || check_ui_gallery_code_editor_a11y_selection
                    || check_ui_gallery_code_editor_a11y_composition
                    || check_ui_gallery_code_editor_a11y_selection_wrap
                    || check_ui_gallery_code_editor_a11y_composition_wrap
                    || check_ui_gallery_code_editor_a11y_composition_wrap_scroll
                    || check_semantics_changed_repainted
                    || check_wheel_scroll_test_id.is_some()
                    || check_wheel_scroll_hit_changes_test_id.is_some()
                    || check_prepaint_actions_min.is_some()
                    || check_chart_sampling_window_shifts_min.is_some()
                    || check_node_graph_cull_window_shifts_min.is_some()
                    || check_node_graph_cull_window_shifts_max.is_some()
                    || check_vlist_visible_range_refreshes_min.is_some()
                    || check_vlist_visible_range_refreshes_max.is_some()
                    || check_vlist_window_shifts_explainable
                    || check_vlist_window_shifts_have_prepaint_actions
                    || check_vlist_window_shifts_non_retained_max.is_some()
                    || check_vlist_window_shifts_prefetch_max.is_some()
                    || check_vlist_window_shifts_escape_max.is_some()
                    || check_vlist_policy_key_stable
                    || check_windowed_rows_offset_changes_min.is_some()
                    || check_windowed_rows_visible_start_changes_repainted
                    || check_layout_fast_path_min.is_some()
                    || check_drag_cache_root_paint_only_test_id.is_some()
                    || check_hover_layout_max.is_some()
                    || check_gc_sweep_liveness
                    || !check_notify_hotspot_file_max.is_empty()
                    || check_view_cache_reuse_min.is_some()
                    || check_view_cache_reuse_stable_min.is_some()
                    || check_overlay_synthesis_min.is_some()
                    || check_viewport_input_min.is_some()
                    || check_dock_drag_min.is_some()
                    || check_viewport_capture_min.is_some()
                    || check_retained_vlist_reconcile_no_notify_min.is_some()
                    || check_retained_vlist_attach_detach_max.is_some()
                    || check_retained_vlist_keep_alive_reuse_min.is_some()
                    || check_retained_vlist_keep_alive_budget.is_some()
                {
                    let bundle_path = wait_for_bundle_json_from_script_result(
                        &resolved_out_dir,
                        &result,
                        timeout_ms,
                        poll_ms,
                    )
                    .ok_or_else(|| {
                        "script passed but no bundle.json was found (required for post-run checks)"
                            .to_string()
                    })?;

                    apply_post_run_checks(
                        &bundle_path,
                        &resolved_out_dir,
                        check_idle_no_paint_min,
                        check_stale_paint_test_id.as_deref(),
                        check_stale_paint_eps,
                        check_stale_scene_test_id.as_deref(),
                        check_stale_scene_eps,
                        check_pixels_changed_test_id.as_deref(),
                        check_ui_gallery_code_editor_torture_marker_present,
                        check_ui_gallery_code_editor_torture_undo_redo,
                        check_ui_gallery_code_editor_torture_geom_fallbacks_low,
                        check_ui_gallery_code_editor_torture_read_only_blocks_edits,
                        check_ui_gallery_markdown_editor_source_read_only_blocks_edits,
                        check_ui_gallery_markdown_editor_source_disabled_blocks_edits,
                        check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable,
                        check_ui_gallery_markdown_editor_source_word_boundary,
                        check_ui_gallery_web_ime_bridge_enabled,
                        check_ui_gallery_markdown_editor_source_line_boundary_triple_click,
                        check_ui_gallery_markdown_editor_source_a11y_composition,
                        check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap,
                        check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable,
                        check_ui_gallery_markdown_editor_source_folds_toggle_stable,
                        check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds,
                        check_ui_gallery_markdown_editor_source_folds_placeholder_present,
                        check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap,
                        check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit,
                        check_ui_gallery_markdown_editor_source_inlays_toggle_stable,
                        check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable,
                        check_ui_gallery_markdown_editor_source_inlays_present,
                        check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap,
                        check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit,
                        check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed,
                        check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed,
                        check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
                        check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll,
                        check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap,
                        check_ui_gallery_code_editor_torture_inlays_present,
                        check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit,
                        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped,
                        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations,
                        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed,
                        check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap,
                        check_ui_gallery_code_editor_word_boundary,
                        check_ui_gallery_code_editor_a11y_selection,
                        check_ui_gallery_code_editor_a11y_composition,
                        check_ui_gallery_code_editor_a11y_selection_wrap,
                        check_ui_gallery_code_editor_a11y_composition_wrap,
                        check_ui_gallery_code_editor_a11y_composition_wrap_scroll,
                        check_ui_gallery_code_editor_a11y_composition_drag,
                        check_semantics_changed_repainted,
                        dump_semantics_changed_repainted_json,
                        check_wheel_scroll_test_id.as_deref(),
                        check_wheel_scroll_hit_changes_test_id.as_deref(),
                        check_prepaint_actions_min,
                        check_chart_sampling_window_shifts_min,
                        check_node_graph_cull_window_shifts_min,
                        check_node_graph_cull_window_shifts_max,
                        check_vlist_visible_range_refreshes_min,
                        check_vlist_visible_range_refreshes_max,
                        check_vlist_window_shifts_explainable,
                        check_vlist_window_shifts_have_prepaint_actions,
                        check_vlist_window_shifts_non_retained_max,
                        check_vlist_window_shifts_prefetch_max,
                        check_vlist_window_shifts_escape_max,
                        check_vlist_policy_key_stable,
                        check_windowed_rows_offset_changes_min,
                        check_windowed_rows_offset_changes_eps,
                        check_windowed_rows_visible_start_changes_repainted,
                        check_layout_fast_path_min,
                        check_drag_cache_root_paint_only_test_id.as_deref(),
                        check_hover_layout_max,
                        check_gc_sweep_liveness,
                        &check_notify_hotspot_file_max,
                        check_view_cache_reuse_stable_min,
                        check_view_cache_reuse_min,
                        check_overlay_synthesis_min,
                        check_viewport_input_min,
                        check_dock_drag_min,
                        check_viewport_capture_min,
                        check_retained_vlist_reconcile_no_notify_min,
                        check_retained_vlist_attach_detach_max,
                        check_retained_vlist_keep_alive_reuse_min,
                        check_retained_vlist_keep_alive_budget,
                        warmup_frames,
                    )?;
                }
            }

            if wants_pack {
                let mut bundle_path = wait_for_bundle_json_from_script_result(
                    &resolved_out_dir,
                    &result,
                    timeout_ms,
                    poll_ms,
                );
                if bundle_path.is_none() {
                    let _ = touch(&resolved_trigger_path);
                    bundle_path = wait_for_bundle_json_from_script_result(
                        &resolved_out_dir,
                        &result,
                        timeout_ms,
                        poll_ms,
                    );
                }

                if let Some(bundle_path) = bundle_path {
                    let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
                    let out = pack_out
                        .clone()
                        .map(|p| resolve_path(&workspace_root, p))
                        .unwrap_or_else(|| default_pack_out_path(&resolved_out_dir, &bundle_dir));

                    let artifacts_root = if bundle_dir.starts_with(&resolved_out_dir) {
                        resolved_out_dir.clone()
                    } else {
                        bundle_dir
                            .parent()
                            .unwrap_or(&resolved_out_dir)
                            .to_path_buf()
                    };

                    if let Err(err) = pack_bundle_dir_to_zip(
                        &bundle_dir,
                        &out,
                        pack_defaults.0,
                        pack_defaults.1,
                        pack_defaults.2,
                        false,
                        false,
                        &artifacts_root,
                        stats_top,
                        sort_override.unwrap_or(BundleStatsSort::Invalidation),
                        warmup_frames,
                    ) {
                        eprintln!("PACK-ERROR {err}");
                    } else {
                        println!("PACK {}", out.display());
                    }
                } else {
                    eprintln!(
                        "PACK-ERROR no bundle.json found (add `capture_bundle` or enable script auto-dumps)"
                    );
                }
            }

            report_result_and_exit(&result);
        }
        "repeat" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing script path (try: fretboard diag repeat ./script.json --repeat 7)"
                        .to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let repeat = perf_repeat.max(1) as usize;

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            let wants_screenshots = script_requests_screenshots(&src)
                || pack_include_screenshots
                || check_pixels_changed_test_id.is_some();

            let repeat_launch_env = launch_env.clone();
            let reuse_process = launch.is_none() || reuse_launch;

            let mut child = if reuse_process {
                maybe_launch_demo(
                    &launch,
                    &repeat_launch_env,
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    wants_screenshots,
                    timeout_ms,
                    poll_ms,
                )?
            } else {
                None
            };

            let mut runs: Vec<serde_json::Value> = Vec::with_capacity(repeat);

            let mut baseline_run: Option<usize> = None;
            let mut baseline_bundle: Option<PathBuf> = None;

            let mut failed_runs: u64 = 0;
            let mut differing_runs: u64 = 0;
            let mut first_failed_run: Option<usize> = None;
            let mut first_differing_run: Option<usize> = None;
            let mut worst_perf: Option<(usize, u64, u64)> = None; // (index, total_us, frame_id)
            let mut stage_counts: std::collections::BTreeMap<String, u64> =
                std::collections::BTreeMap::new();
            let mut reason_code_counts: std::collections::BTreeMap<String, u64> =
                std::collections::BTreeMap::new();
            let mut lint_error_runs: Vec<usize> = Vec::new();
            let mut lint_counts_by_code: std::collections::BTreeMap<String, (u64, u64)> =
                std::collections::BTreeMap::new();
            let mut evidence_present_runs: Vec<usize> = Vec::new();
            let mut focus_mismatch_runs: Vec<usize> = Vec::new();
            let mut blocking_reason_counts: std::collections::BTreeMap<String, u64> =
                std::collections::BTreeMap::new();
            let mut overlay_chosen_side_counts: std::collections::BTreeMap<String, u64> =
                std::collections::BTreeMap::new();
            let mut ime_event_kind_counts: std::collections::BTreeMap<String, u64> =
                std::collections::BTreeMap::new();

            fn read_script_result_typed(path: &Path) -> Option<UiScriptResultV1> {
                let bytes = std::fs::read(path).ok()?;
                serde_json::from_slice::<UiScriptResultV1>(&bytes).ok()
            }

            fn push_count(map: &mut std::collections::BTreeMap<String, u64>, key: &str) {
                if key.trim().is_empty() {
                    return;
                }
                *map.entry(key.to_string()).or_default() += 1;
            }

            fn overlay_side_as_str(side: fret_diag_protocol::UiOverlaySideV1) -> &'static str {
                match side {
                    fret_diag_protocol::UiOverlaySideV1::Top => "top",
                    fret_diag_protocol::UiOverlaySideV1::Bottom => "bottom",
                    fret_diag_protocol::UiOverlaySideV1::Left => "left",
                    fret_diag_protocol::UiOverlaySideV1::Right => "right",
                }
            }

            fn summarize_script_result_evidence(
                result: &UiScriptResultV1,
                blocking_reason_counts: &mut std::collections::BTreeMap<String, u64>,
                overlay_chosen_side_counts: &mut std::collections::BTreeMap<String, u64>,
                ime_event_kind_counts: &mut std::collections::BTreeMap<String, u64>,
            ) -> (Option<serde_json::Value>, bool, bool) {
                let Some(e) = result.evidence.as_ref() else {
                    return (None, false, false);
                };

                let mut evidence_present = false;
                let mut focus_mismatch = false;

                let mut trace_counts = std::collections::BTreeMap::<&str, u64>::new();
                trace_counts.insert(
                    "selector_resolution_trace",
                    e.selector_resolution_trace.len() as u64,
                );
                trace_counts.insert("hit_test_trace", e.hit_test_trace.len() as u64);
                trace_counts.insert("click_stable_trace", e.click_stable_trace.len() as u64);
                trace_counts.insert("bounds_stable_trace", e.bounds_stable_trace.len() as u64);
                trace_counts.insert("focus_trace", e.focus_trace.len() as u64);
                trace_counts.insert(
                    "shortcut_routing_trace",
                    e.shortcut_routing_trace.len() as u64,
                );
                trace_counts.insert(
                    "overlay_placement_trace",
                    e.overlay_placement_trace.len() as u64,
                );
                trace_counts.insert("web_ime_trace", e.web_ime_trace.len() as u64);
                trace_counts.insert("ime_event_trace", e.ime_event_trace.len() as u64);

                if trace_counts.values().any(|&n| n > 0) {
                    evidence_present = true;
                }

                let mut hit_test_blocking = std::collections::BTreeMap::<String, u64>::new();
                for entry in &e.hit_test_trace {
                    if let Some(reason) = entry.blocking_reason.as_deref() {
                        push_count(&mut hit_test_blocking, reason);
                        push_count(blocking_reason_counts, reason);
                    }
                }

                let mut focus = serde_json::json!({
                    "mismatch_count": 0u64,
                    "text_input_snapshots": 0u64,
                    "composing_true": 0u64,
                });
                let mut mismatch_count: u64 = 0;
                let mut text_input_snapshots: u64 = 0;
                let mut composing_true: u64 = 0;
                for entry in &e.focus_trace {
                    if entry.matches_expected == Some(false) {
                        mismatch_count += 1;
                    }
                    if let Some(snap) = entry.text_input_snapshot.as_ref() {
                        text_input_snapshots += 1;
                        if snap.is_composing {
                            composing_true += 1;
                        }
                    }
                }
                if mismatch_count > 0 {
                    focus_mismatch = true;
                }
                focus["mismatch_count"] = serde_json::Value::Number(mismatch_count.into());
                focus["text_input_snapshots"] =
                    serde_json::Value::Number(text_input_snapshots.into());
                focus["composing_true"] = serde_json::Value::Number(composing_true.into());

                let mut shortcut_outcomes = std::collections::BTreeMap::<String, u64>::new();
                for entry in &e.shortcut_routing_trace {
                    push_count(&mut shortcut_outcomes, &entry.outcome);
                }

                let mut overlay_kinds = std::collections::BTreeMap::<&str, u64>::new();
                let mut overlay_chosen_sides = std::collections::BTreeMap::<String, u64>::new();
                for entry in &e.overlay_placement_trace {
                    match entry {
                        fret_diag_protocol::UiOverlayPlacementTraceEntryV1::AnchoredPanel {
                            chosen_side,
                            ..
                        } => {
                            *overlay_kinds.entry("anchored_panel").or_default() += 1;
                            let side = overlay_side_as_str(*chosen_side);
                            push_count(&mut overlay_chosen_sides, side);
                            push_count(overlay_chosen_side_counts, side);
                        }
                        fret_diag_protocol::UiOverlayPlacementTraceEntryV1::PlacedRect {
                            ..
                        } => {
                            *overlay_kinds.entry("placed_rect").or_default() += 1;
                        }
                    }
                }

                let mut web_ime = serde_json::json!({
                    "enabled_true": 0u64,
                    "enabled_false": 0u64,
                    "composing_true": 0u64,
                });
                let mut web_ime_enabled_true: u64 = 0;
                let mut web_ime_enabled_false: u64 = 0;
                let mut web_ime_composing_true: u64 = 0;
                for entry in &e.web_ime_trace {
                    if entry.enabled {
                        web_ime_enabled_true += 1;
                    } else {
                        web_ime_enabled_false += 1;
                    }
                    if entry.composing {
                        web_ime_composing_true += 1;
                    }
                }
                web_ime["enabled_true"] = serde_json::Value::Number(web_ime_enabled_true.into());
                web_ime["enabled_false"] = serde_json::Value::Number(web_ime_enabled_false.into());
                web_ime["composing_true"] =
                    serde_json::Value::Number(web_ime_composing_true.into());

                let mut ime_kinds = std::collections::BTreeMap::<String, u64>::new();
                for entry in &e.ime_event_trace {
                    push_count(&mut ime_kinds, &entry.kind);
                    push_count(ime_event_kind_counts, &entry.kind);
                }

                let summary = serde_json::json!({
                    "trace_counts": trace_counts,
                    "hit_test": {
                        "blocking_reason_counts": hit_test_blocking,
                    },
                    "focus": focus,
                    "shortcuts": {
                        "outcome_counts": shortcut_outcomes,
                    },
                    "overlay": {
                        "kind_counts": overlay_kinds,
                        "chosen_side_counts": overlay_chosen_sides,
                    },
                    "web_ime": web_ime,
                    "ime_events": {
                        "kind_counts": ime_kinds,
                    },
                });

                (Some(summary), evidence_present, focus_mismatch)
            }

            for run_index in 0..repeat {
                if !reuse_process {
                    child = maybe_launch_demo(
                        &launch,
                        &repeat_launch_env,
                        &workspace_root,
                        &resolved_out_dir,
                        &resolved_ready_path,
                        &resolved_exit_path,
                        wants_screenshots,
                        timeout_ms,
                        poll_ms,
                    )?;
                }

                let mut summary = run_script_and_wait(
                    &src,
                    &resolved_script_path,
                    &resolved_script_trigger_path,
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                    timeout_ms,
                    poll_ms,
                );

                if let Ok(s) = &summary
                    && s.stage.as_deref() == Some("failed")
                {
                    if let Some(dir) =
                        wait_for_failure_dump_bundle(&resolved_out_dir, s, timeout_ms, poll_ms)
                    {
                        if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                            if let Ok(s) = summary.as_mut() {
                                s.last_bundle_dir = Some(name.to_string());
                            }
                        }
                    }
                }

                if !reuse_process {
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                }

                let entry = match summary {
                    Ok(s) => {
                        let stage = s.stage.as_deref().unwrap_or("unknown").to_string();

                        let bundle_json = s
                            .last_bundle_dir
                            .as_deref()
                            .and_then(|d| (!d.trim().is_empty()).then_some(d.trim()))
                            .map(PathBuf::from)
                            .map(|p| {
                                if p.is_absolute() {
                                    p
                                } else {
                                    resolved_out_dir.join(p)
                                }
                            })
                            .and_then(|p| {
                                if p.is_dir() {
                                    wait_for_bundle_json_in_dir(&p, timeout_ms, poll_ms)
                                } else if p.is_file() {
                                    Some(p)
                                } else {
                                    None
                                }
                            });

                        if stage == "failed" {
                            failed_runs += 1;
                            if first_failed_run.is_none() {
                                first_failed_run = Some(run_index);
                            }
                        }
                        *stage_counts.entry(stage.clone()).or_default() += 1;
                        if let Some(code) = s.reason_code.as_deref().filter(|v| !v.is_empty()) {
                            *reason_code_counts.entry(code.to_string()).or_default() += 1;
                        }

                        let mut evidence: Option<serde_json::Value> = None;
                        if let Some(full) = read_script_result_typed(&resolved_script_result_path) {
                            let (summary, present, focus_mismatch) =
                                summarize_script_result_evidence(
                                    &full,
                                    &mut blocking_reason_counts,
                                    &mut overlay_chosen_side_counts,
                                    &mut ime_event_kind_counts,
                                );
                            evidence = summary;
                            if present {
                                evidence_present_runs.push(run_index);
                            }
                            if focus_mismatch {
                                focus_mismatch_runs.push(run_index);
                            }
                        }

                        let mut perf: Option<serde_json::Value> = None;
                        if let Some(bundle_json) = bundle_json.as_ref() {
                            if let Ok(report) = bundle_stats_from_path(
                                bundle_json,
                                1,
                                BundleStatsSort::Time,
                                BundleStatsOptions { warmup_frames },
                            ) {
                                if let Some(top) = report.top.first() {
                                    let total_us = top.total_time_us;
                                    match &worst_perf {
                                        Some((_idx, best_total, _frame))
                                            if *best_total >= total_us => {}
                                        _ => {
                                            worst_perf = Some((run_index, total_us, top.frame_id));
                                        }
                                    }
                                    perf = Some(serde_json::json!({
                                        "top_total_time_us": top.total_time_us,
                                        "top_layout_time_us": top.layout_time_us,
                                        "top_layout_engine_solve_time_us": top.layout_engine_solve_time_us,
                                        "frame_id": top.frame_id,
                                    }));
                                }
                            }
                        }

                        let mut lint: Option<serde_json::Value> = None;
                        if let Some(bundle_json) = bundle_json.as_ref() {
                            if let Ok(report) = lint_bundle_from_path(
                                bundle_json,
                                warmup_frames,
                                LintOptions {
                                    all_test_ids_bounds: lint_all_test_ids_bounds,
                                    eps_px: lint_eps_px,
                                },
                            ) {
                                let warning_issues = report
                                    .payload
                                    .get("warning_issues")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                let counts_by_code = report.payload.get("counts_by_code").cloned();
                                if report.error_issues > 0 {
                                    lint_error_runs.push(run_index);
                                }
                                if let Some(serde_json::Value::Array(entries)) =
                                    counts_by_code.as_ref()
                                {
                                    for entry in entries {
                                        let Some(code) = entry.get("code").and_then(|v| v.as_str())
                                        else {
                                            continue;
                                        };
                                        let errors = entry
                                            .get("errors")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0);
                                        let warnings = entry
                                            .get("warnings")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0);
                                        if errors == 0 && warnings == 0 {
                                            continue;
                                        }
                                        let entry = lint_counts_by_code
                                            .entry(code.to_string())
                                            .or_insert((0, 0));
                                        entry.0 = entry.0.saturating_add(errors);
                                        entry.1 = entry.1.saturating_add(warnings);
                                    }
                                }
                                lint = Some(serde_json::json!({
                                    "error_issues": report.error_issues,
                                    "warning_issues": warning_issues,
                                    "counts_by_code": counts_by_code,
                                }));
                            }
                        }

                        let mut compare_to_baseline: Option<serde_json::Value> = None;
                        if stage == "passed" {
                            if baseline_bundle.is_none() {
                                if let Some(bundle_json) = bundle_json.clone() {
                                    baseline_run = Some(run_index);
                                    baseline_bundle = Some(bundle_json);
                                }
                            } else if let (Some(base), Some(cur)) =
                                (baseline_bundle.as_ref(), bundle_json.as_ref())
                            {
                                let report = compare_bundles(
                                    base,
                                    cur,
                                    CompareOptions {
                                        warmup_frames,
                                        eps_px: compare_eps_px,
                                        ignore_bounds: compare_ignore_bounds,
                                        ignore_scene_fingerprint: compare_ignore_scene_fingerprint,
                                    },
                                )?;

                                let mut kinds: std::collections::BTreeMap<&str, u64> =
                                    std::collections::BTreeMap::new();
                                let mut semantics_diffs: u64 = 0;
                                let mut layout_diffs: u64 = 0;
                                let mut scene_fp_mismatch: u64 = 0;
                                for d in &report.diffs {
                                    *kinds.entry(d.kind).or_default() += 1;
                                    if d.kind == "scene_fingerprint_mismatch" {
                                        scene_fp_mismatch += 1;
                                        continue;
                                    }
                                    if d.kind == "node_field_mismatch" && d.field == Some("bounds")
                                    {
                                        layout_diffs += 1;
                                        continue;
                                    }
                                    semantics_diffs += 1;
                                }

                                if !report.ok {
                                    differing_runs += 1;
                                    if first_differing_run.is_none() {
                                        first_differing_run = Some(run_index);
                                    }
                                }

                                compare_to_baseline = Some(serde_json::json!({
                                    "ok": report.ok,
                                    "diffs": report.diffs.len(),
                                    "semantics_diffs": semantics_diffs,
                                    "layout_diffs": layout_diffs,
                                    "scene_fingerprint_mismatch": scene_fp_mismatch,
                                    "diff_kinds": kinds,
                                }));
                            }
                        }

                        serde_json::json!({
                            "index": run_index,
                            "stage": stage,
                            "run_id": s.run_id,
                            "reason_code": s.reason_code,
                            "reason": s.reason,
                            "last_bundle_dir": s.last_bundle_dir,
                            "bundle_json": bundle_json.as_ref().map(|p| p.display().to_string()),
                            "perf": perf,
                            "lint": lint,
                            "evidence": evidence,
                            "compare_to_baseline": compare_to_baseline,
                        })
                    }
                    Err(err) => {
                        failed_runs += 1;
                        if first_failed_run.is_none() {
                            first_failed_run = Some(run_index);
                        }
                        *stage_counts.entry("error".to_string()).or_default() += 1;
                        serde_json::json!({
                            "index": run_index,
                            "stage": "error",
                            "error": err,
                        })
                    }
                };

                runs.push(entry);
            }

            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);

            let status = if failed_runs == 0 && differing_runs == 0 {
                "passed"
            } else {
                "failed"
            };

            let lint_counts_by_code_json: std::collections::BTreeMap<String, serde_json::Value> =
                lint_counts_by_code
                    .into_iter()
                    .map(|(code, (errors, warnings))| {
                        (
                            code,
                            serde_json::json!({
                                "errors": errors,
                                "warnings": warnings,
                            }),
                        )
                    })
                    .collect();
            let highlights = serde_json::json!({
                "stage_counts": stage_counts,
                "reason_code_counts": reason_code_counts,
                "first_failed_run": first_failed_run,
                "first_differing_run": first_differing_run,
                "worst_perf": worst_perf.map(|(idx, total_us, frame_id)| serde_json::json!({
                    "run": idx,
                    "top_total_time_us": total_us,
                    "frame_id": frame_id,
                })),
                "lint_error_runs": lint_error_runs,
                "lint_counts_by_code": lint_counts_by_code_json,
                "evidence_present_runs": evidence_present_runs,
                "focus_mismatch_runs": focus_mismatch_runs,
                "blocking_reason_counts": blocking_reason_counts,
                "overlay_chosen_side_counts": overlay_chosen_side_counts,
                "ime_event_kind_counts": ime_event_kind_counts,
            });
            let payload = serde_json::json!({
                "schema_version": 1,
                "status": status,
                "script": src.display().to_string(),
                "repeat": repeat,
                "baseline_run": baseline_run,
                "highlights": highlights,
                "options": {
                    "warmup_frames": warmup_frames,
                    "compare_eps_px": compare_eps_px,
                    "compare_ignore_bounds": compare_ignore_bounds,
                    "compare_ignore_scene_fingerprint": compare_ignore_scene_fingerprint,
                },
                "failed_runs": failed_runs,
                "differing_runs": differing_runs,
                "runs": runs,
            });

            let out_path = resolved_out_dir.join("repeat.summary.json");
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let pretty =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
            std::fs::write(&out_path, pretty.as_bytes()).map_err(|e| e.to_string())?;

            if stats_json {
                println!("{pretty}");
            } else {
                println!("{}", out_path.display());
            }

            if status != "passed" {
                std::process::exit(1);
            }
            Ok(())
        }
        "repro" => {
            if rest.is_empty() {
                return Err(
                    "missing script path or suite name (try: fretboard diag repro ui-gallery | fretboard diag repro ./script.json)"
                        .to_string(),
                );
            }

            let mut pack_defaults = (
                pack_include_root_artifacts,
                pack_include_triage,
                pack_include_screenshots,
            );
            if !pack_defaults.0 && !pack_defaults.1 && !pack_defaults.2 {
                pack_defaults = (true, true, true);
            }

            let (scripts, suite_name): (Vec<PathBuf>, Option<String>) =
                if rest.len() == 1 && rest[0] == "ui-gallery" {
                    (
                        ui_gallery_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some("ui-gallery".to_string()),
                    )
                } else if rest.len() == 1 && rest[0] == "ui-gallery-code-editor" {
                    (
                        ui_gallery_code_editor_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some("ui-gallery-code-editor".to_string()),
                    )
                } else if rest.len() == 1 && rest[0] == "docking-arbitration" {
                    (
                        docking_arbitration_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some("docking-arbitration".to_string()),
                    )
                } else {
                    (
                        rest.into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        None,
                    )
                };

            let summary_path = resolved_out_dir.join("repro.summary.json");

            let mut repro_launch = launch.clone();
            let mut repro_launch_env = launch_env.clone();
            let _ = ensure_env_var(&mut repro_launch_env, "FRET_DIAG_RENDERER_PERF", "1");

            if check_redraw_hitches_max_total_ms_threshold.is_some() {
                let _ = ensure_env_var(&mut repro_launch_env, "FRET_REDRAW_HITCH_LOG", "1");
                let _ = ensure_env_var(
                    &mut repro_launch_env,
                    "FRET_REDRAW_HITCH_LOG_PATH",
                    "redraw_hitches.log",
                );
            }

            let mut tracy_feature_injected: bool = false;
            let mut tracy_env_enabled: bool = false;
            if with_tracy {
                tracy_env_enabled = ensure_env_var(&mut repro_launch_env, "FRET_TRACY", "1");
                if let Some(cmd) = repro_launch.as_mut() {
                    tracy_feature_injected = cargo_run_inject_feature(cmd, "fret-bootstrap/tracy");
                }

                let note = "\
# Tracy capture (best-effort)\n\
\n\
This repro was run with `FRET_TRACY=1` (and may have auto-injected `--features fret-bootstrap/tracy` when the launch command was `cargo run`).\n\
\n\
Notes:\n\
- Tracy requires running the target with the `fret-bootstrap/tracy` feature enabled.\n\
- The capture file is not recorded automatically by `fretboard` yet. Use the Tracy UI to connect and save a capture.\n\
\n\
See: `docs/tracy.md`.\n";
                let _ = std::fs::write(resolved_out_dir.join("tracy.note.md"), note);
            }

            let mut renderdoc_capture_dir: Option<PathBuf> = None;
            let mut renderdoc_autocapture_after_frames: Option<u32> = None;
            if with_renderdoc {
                let after = renderdoc_after_frames.unwrap_or(60);
                let capture_dir = resolved_out_dir.join("renderdoc");
                let _ = std::fs::create_dir_all(&capture_dir);

                let _ = ensure_env_var(&mut repro_launch_env, "FRET_RENDERDOC", "1");
                let _ = ensure_env_var(
                    &mut repro_launch_env,
                    "FRET_RENDERDOC_CAPTURE_DIR",
                    capture_dir.to_string_lossy().as_ref(),
                );
                let _ = ensure_env_var(
                    &mut repro_launch_env,
                    "FRET_RENDERDOC_AUTOCAPTURE_AFTER_FRAMES",
                    &after.to_string(),
                );

                renderdoc_capture_dir = Some(capture_dir);
                renderdoc_autocapture_after_frames = Some(after);
            }

            let mut child = maybe_launch_demo(
                &repro_launch,
                &repro_launch_env,
                &workspace_root,
                &resolved_out_dir,
                &resolved_ready_path,
                &resolved_exit_path,
                pack_defaults.2
                    || check_pixels_changed_test_id.is_some()
                    || scripts.iter().any(|p| script_requests_screenshots(p)),
                timeout_ms,
                poll_ms,
            )?;

            let mut required_caps: Vec<String> = Vec::new();
            for src in scripts.iter() {
                required_caps.extend(script_required_capabilities(src));
            }
            required_caps.sort();
            required_caps.dedup();
            let available_caps = read_filesystem_capabilities(&resolved_out_dir);
            let capabilities_check_path = resolved_out_dir.join("check.capabilities.json");

            let mut repro_process_footprint: Option<serde_json::Value> = None;
            let mut resource_footprint_gate: Option<ResourceFootprintGateResult> = None;
            let mut redraw_hitches_gate: Option<RedrawHitchesGateResult> = None;

            let mut run_rows: Vec<serde_json::Value> = Vec::new();
            let mut selected_bundle_path: Option<PathBuf> = None;
            let mut last_script_result: Option<ScriptResultSummary> = None;
            let mut overall_error: Option<String> = None;
            let mut pack_items: Vec<ReproPackItem> = Vec::new();

            if !required_caps.is_empty() {
                if let Err(err) = gate_required_capabilities(
                    &capabilities_check_path,
                    &required_caps,
                    &available_caps,
                ) {
                    overall_error = Some(err);
                }
            }

            for src in scripts {
                if overall_error.is_some() {
                    break;
                }
                let mut result = run_script_and_wait(
                    &src,
                    &resolved_script_path,
                    &resolved_script_trigger_path,
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                    timeout_ms,
                    poll_ms,
                );

                if let Ok(summary) = &result
                    && summary.stage.as_deref() == Some("failed")
                {
                    if let Some(dir) = wait_for_failure_dump_bundle(
                        &resolved_out_dir,
                        summary,
                        timeout_ms,
                        poll_ms,
                    ) {
                        if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                            if let Ok(summary) = result.as_mut() {
                                summary.last_bundle_dir = Some(name.to_string());
                            }
                        }
                    }
                }

                let result = match result {
                    Ok(r) => r,
                    Err(err) => {
                        overall_error = Some(err);
                        break;
                    }
                };
                last_script_result = Some(result.clone());

                let mut bundle_path = wait_for_bundle_json_from_script_result(
                    &resolved_out_dir,
                    &result,
                    timeout_ms,
                    poll_ms,
                );
                if bundle_path.is_none() {
                    let _ = touch(&resolved_trigger_path);
                    bundle_path = wait_for_bundle_json_from_script_result(
                        &resolved_out_dir,
                        &result,
                        timeout_ms,
                        poll_ms,
                    );
                }

                if let Some(bundle_path) = bundle_path.as_ref() {
                    pack_items.push(ReproPackItem {
                        script_path: src.clone(),
                        bundle_json: bundle_path.clone(),
                    });
                }

                if result.stage.as_deref() == Some("failed") && bundle_path.is_some() {
                    selected_bundle_path = bundle_path.clone();
                }
                if selected_bundle_path.is_none() {
                    selected_bundle_path = bundle_path.clone();
                }

                run_rows.push(serde_json::json!({
                    "script_path": src.display().to_string(),
                    "run_id": result.run_id,
                    "stage": result.stage,
                    "step_index": result.step_index,
                    "reason": result.reason,
                    "last_bundle_dir": result.last_bundle_dir,
                    "bundle_json": bundle_path.as_ref().map(|p| p.display().to_string()),
                }));

                if result.stage.as_deref() == Some("passed") {
                    let wants_post_run_checks_for_script = check_stale_paint_test_id.is_some()
                        || check_stale_scene_test_id.is_some()
                        || check_idle_no_paint_min.is_some()
                        || check_pixels_changed_test_id.is_some()
                        || check_ui_gallery_code_editor_torture_marker_present
                        || check_ui_gallery_code_editor_torture_undo_redo
                        || check_ui_gallery_code_editor_torture_geom_fallbacks_low
                        || check_ui_gallery_code_editor_torture_read_only_blocks_edits
                        || check_ui_gallery_markdown_editor_source_read_only_blocks_edits
                        || check_ui_gallery_markdown_editor_source_disabled_blocks_edits
                        || check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable
                        || check_ui_gallery_markdown_editor_source_word_boundary
                        || check_ui_gallery_web_ime_bridge_enabled
                        || check_ui_gallery_markdown_editor_source_line_boundary_triple_click
                        || check_ui_gallery_markdown_editor_source_a11y_composition
                        || check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap
                        || check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable
                        || check_ui_gallery_markdown_editor_source_folds_toggle_stable
                        || check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds
                        || check_ui_gallery_markdown_editor_source_folds_placeholder_present
                        || check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap
                        || check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit
                        || check_ui_gallery_markdown_editor_source_inlays_toggle_stable
                        || check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable
                        || check_ui_gallery_markdown_editor_source_inlays_present
                        || check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap
                        || check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit
                        || check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit
                        || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
                        || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations
                        || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed
                        || check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed
                        || check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed
                        || check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll
                        || check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection
                        || check_ui_gallery_code_editor_torture_folds_placeholder_present
                        || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap
                        || check_ui_gallery_code_editor_torture_inlays_present
                        || check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit
                        || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped
                        || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations
                        || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed
                        || check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap
                        || check_ui_gallery_code_editor_word_boundary
                        || check_ui_gallery_code_editor_a11y_selection
                        || check_ui_gallery_code_editor_a11y_composition
                        || check_ui_gallery_code_editor_a11y_selection_wrap
                        || check_ui_gallery_code_editor_a11y_composition_wrap
                        || check_ui_gallery_code_editor_a11y_composition_wrap_scroll
                        || check_ui_gallery_code_editor_a11y_composition_drag
                        || check_semantics_changed_repainted
                        || check_wheel_scroll_test_id.is_some()
                        || check_wheel_scroll_hit_changes_test_id.is_some()
                        || check_prepaint_actions_min.is_some()
                        || check_chart_sampling_window_shifts_min.is_some()
                        || check_node_graph_cull_window_shifts_min.is_some()
                        || check_node_graph_cull_window_shifts_max.is_some()
                        || check_vlist_visible_range_refreshes_min.is_some()
                        || check_vlist_visible_range_refreshes_max.is_some()
                        || check_vlist_window_shifts_explainable
                        || check_vlist_window_shifts_have_prepaint_actions
                        || check_vlist_window_shifts_non_retained_max.is_some()
                        || check_vlist_window_shifts_prefetch_max.is_some()
                        || check_vlist_window_shifts_escape_max.is_some()
                        || check_vlist_policy_key_stable
                        || check_windowed_rows_offset_changes_min.is_some()
                        || check_layout_fast_path_min.is_some()
                        || check_drag_cache_root_paint_only_test_id.is_some()
                        || check_hover_layout_max.is_some()
                        || check_gc_sweep_liveness
                        || !check_notify_hotspot_file_max.is_empty()
                        || check_view_cache_reuse_min.is_some()
                        || check_view_cache_reuse_stable_min.is_some()
                        || check_overlay_synthesis_min.is_some()
                        || check_viewport_input_min.is_some()
                        || check_dock_drag_min.is_some()
                        || check_viewport_capture_min.is_some()
                        || check_retained_vlist_reconcile_no_notify_min.is_some()
                        || check_retained_vlist_attach_detach_max.is_some()
                        || check_retained_vlist_keep_alive_reuse_min.is_some()
                        || check_retained_vlist_keep_alive_budget.is_some();

                    if wants_post_run_checks_for_script {
                        let Some(bundle_path) = bundle_path.as_ref() else {
                            overall_error = Some(
                                "script passed but no bundle.json was found (required for post-run checks)"
                                    .to_string(),
                            );
                            break;
                        };

                        if let Err(err) = apply_post_run_checks(
                            bundle_path,
                            &resolved_out_dir,
                            check_idle_no_paint_min,
                            check_stale_paint_test_id.as_deref(),
                            check_stale_paint_eps,
                            check_stale_scene_test_id.as_deref(),
                            check_stale_scene_eps,
                            check_pixels_changed_test_id.as_deref(),
                            check_ui_gallery_code_editor_torture_marker_present,
                            check_ui_gallery_code_editor_torture_undo_redo,
                            check_ui_gallery_code_editor_torture_geom_fallbacks_low,
                            check_ui_gallery_code_editor_torture_read_only_blocks_edits,
                            check_ui_gallery_markdown_editor_source_read_only_blocks_edits,
                            check_ui_gallery_markdown_editor_source_disabled_blocks_edits,
                            check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable,
                            check_ui_gallery_markdown_editor_source_word_boundary,
                            check_ui_gallery_web_ime_bridge_enabled,
                            check_ui_gallery_markdown_editor_source_line_boundary_triple_click,
                            check_ui_gallery_markdown_editor_source_a11y_composition,
                            check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap,
                            check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable,
                            check_ui_gallery_markdown_editor_source_folds_toggle_stable,
                            check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds,
                            check_ui_gallery_markdown_editor_source_folds_placeholder_present,
                            check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap,
                            check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit,
                            check_ui_gallery_markdown_editor_source_inlays_toggle_stable,
                            check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable,
                            check_ui_gallery_markdown_editor_source_inlays_present,
                            check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap,
                            check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit,
                            check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit,
                            check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped,
                            check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations,
                            check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed,
                            check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed,
                            check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
                            check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll,
                            check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection,
                            check_ui_gallery_code_editor_torture_folds_placeholder_present,
                            check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap,
                            check_ui_gallery_code_editor_torture_inlays_present,
                            check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit,
                            check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped,
                            check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations,
                            check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed,
                            check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap,
                            check_ui_gallery_code_editor_word_boundary,
                            check_ui_gallery_code_editor_a11y_selection,
                            check_ui_gallery_code_editor_a11y_composition,
                            check_ui_gallery_code_editor_a11y_selection_wrap,
                            check_ui_gallery_code_editor_a11y_composition_wrap,
                            check_ui_gallery_code_editor_a11y_composition_wrap_scroll,
                            check_ui_gallery_code_editor_a11y_composition_drag,
                            check_semantics_changed_repainted,
                            dump_semantics_changed_repainted_json,
                            check_wheel_scroll_test_id.as_deref(),
                            check_wheel_scroll_hit_changes_test_id.as_deref(),
                            check_prepaint_actions_min,
                            check_chart_sampling_window_shifts_min,
                            check_node_graph_cull_window_shifts_min,
                            check_node_graph_cull_window_shifts_max,
                            check_vlist_visible_range_refreshes_min,
                            check_vlist_visible_range_refreshes_max,
                            check_vlist_window_shifts_explainable,
                            check_vlist_window_shifts_have_prepaint_actions,
                            check_vlist_window_shifts_non_retained_max,
                            check_vlist_window_shifts_prefetch_max,
                            check_vlist_window_shifts_escape_max,
                            check_vlist_policy_key_stable,
                            check_windowed_rows_offset_changes_min,
                            check_windowed_rows_offset_changes_eps,
                            check_windowed_rows_visible_start_changes_repainted,
                            check_layout_fast_path_min,
                            check_drag_cache_root_paint_only_test_id.as_deref(),
                            check_hover_layout_max,
                            check_gc_sweep_liveness,
                            &check_notify_hotspot_file_max,
                            check_view_cache_reuse_stable_min,
                            check_view_cache_reuse_min,
                            check_overlay_synthesis_min,
                            check_viewport_input_min,
                            check_dock_drag_min,
                            check_viewport_capture_min,
                            check_retained_vlist_reconcile_no_notify_min,
                            check_retained_vlist_attach_detach_max,
                            check_retained_vlist_keep_alive_reuse_min,
                            check_retained_vlist_keep_alive_budget,
                            warmup_frames,
                        ) {
                            overall_error = Some(err);
                            break;
                        }
                    }
                } else {
                    overall_error = Some(format!(
                        "script failed: {} (run_id={}, step={:?}, reason={:?})",
                        src.display(),
                        result.run_id,
                        result.step_index,
                        result.reason
                    ));
                    break;
                }
            }

            let zip_out = pack_out
                .map(|p| resolve_path(&workspace_root, p))
                .unwrap_or_else(|| resolved_out_dir.join("repro.zip"));

            let mut packed_zip: Option<PathBuf> = None;
            let mut packed_bundle_json: Option<PathBuf> = None;
            if let Some(bundle_path) = selected_bundle_path.as_ref() {
                let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
                packed_bundle_json = Some(bundle_dir.join("bundle.json"));
            }

            let multi_pack = pack_items.len() > 1;
            let packed_bundles = if multi_pack {
                serde_json::Value::Array(
                    pack_items
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            serde_json::json!({
                                "zip_prefix": repro_zip_prefix_for_script(item, idx),
                                "script_path": item.script_path.display().to_string(),
                                "bundle_json": item.bundle_json.display().to_string(),
                            })
                        })
                        .collect(),
                )
            } else {
                serde_json::Value::Null
            };

            let mut renderdoc_capture_payload: Option<serde_json::Value> = None;
            if with_renderdoc {
                let markers: Vec<String> = if renderdoc_markers.is_empty() {
                    vec![
                        "fret clip mask pass".to_string(),
                        "fret downsample-nearest pass".to_string(),
                        "fret upscale-nearest pass".to_string(),
                    ]
                } else {
                    renderdoc_markers.clone()
                };

                if let Some(dir) = renderdoc_capture_dir.as_ref() {
                    let captures = wait_for_files_with_extensions(dir, &["rdc"], 10_000, poll_ms);
                    repro_process_footprint = repro_process_footprint.or(stop_launched_demo(
                        &mut child,
                        &resolved_exit_path,
                        poll_ms,
                    ));

                    let mut capture_rows: Vec<serde_json::Value> = Vec::new();
                    for (cap_idx, capture) in captures.iter().enumerate() {
                        let stem = capture
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or("capture");
                        let safe_stem = format!(
                            "{:02}-{}",
                            cap_idx.saturating_add(1),
                            zip_safe_component(stem)
                        );
                        let inspect_root = dir.join("inspect").join(&safe_stem);

                        let summary_dir = inspect_root.join("summary");
                        let summary_attempt = run_fret_renderdoc_dump(
                            &workspace_root,
                            capture,
                            &summary_dir,
                            "summary",
                            "",
                            Some(200_000),
                            true,
                            true,
                            Some(30),
                        );

                        let mut attempts: Vec<RenderdocDumpAttempt> = Vec::new();
                        attempts.push(summary_attempt);

                        for (idx, marker) in markers.iter().enumerate() {
                            let safe_marker = zip_safe_component(marker);
                            let out_dir = inspect_root
                                .join(format!("marker_{:02}_{safe_marker}", idx.saturating_add(1)));
                            let attempt = run_fret_renderdoc_dump(
                                &workspace_root,
                                capture,
                                &out_dir,
                                "dump",
                                marker,
                                Some(2_000),
                                true,
                                renderdoc_no_outputs_png,
                                None,
                            );
                            attempts.push(attempt);
                        }

                        let attempt_rows = attempts
                            .into_iter()
                            .map(|a| {
                                let out_dir = a
                                    .out_dir
                                    .strip_prefix(&resolved_out_dir)
                                    .unwrap_or(&a.out_dir)
                                    .display()
                                    .to_string();
                                let stdout_file = a.stdout_file.as_ref().map(|p| {
                                    p.strip_prefix(&resolved_out_dir)
                                        .unwrap_or(p)
                                        .display()
                                        .to_string()
                                });
                                let stderr_file = a.stderr_file.as_ref().map(|p| {
                                    p.strip_prefix(&resolved_out_dir)
                                        .unwrap_or(p)
                                        .display()
                                        .to_string()
                                });
                                let response_json = a.response_json.as_ref().map(|p| {
                                    p.strip_prefix(&resolved_out_dir)
                                        .unwrap_or(p)
                                        .display()
                                        .to_string()
                                });

                                serde_json::json!({
                                    "marker": a.marker,
                                    "out_dir": out_dir,
                                    "exit_code": a.exit_code,
                                    "response_json": response_json,
                                    "stdout_file": stdout_file,
                                    "stderr_file": stderr_file,
                                    "error": a.error,
                                })
                            })
                            .collect::<Vec<_>>();

                        let capture_rel = capture
                            .strip_prefix(&resolved_out_dir)
                            .unwrap_or(capture)
                            .display()
                            .to_string();
                        let inspect_rel = inspect_root
                            .strip_prefix(&resolved_out_dir)
                            .unwrap_or(&inspect_root)
                            .display()
                            .to_string();

                        capture_rows.push(serde_json::json!({
                            "capture": capture_rel,
                            "inspect_dir": inspect_rel,
                            "dumps": attempt_rows,
                        }));
                    }

                    let payload = serde_json::json!({
                        "schema_version": 2,
                        "generated_unix_ms": now_unix_ms(),
                        "capture_dir": "renderdoc",
                        "autocapture_after_frames": renderdoc_autocapture_after_frames,
                        "captures": capture_rows,
                    });
                    let _ = write_json_value(
                        &resolved_out_dir.join("renderdoc.captures.json"),
                        &payload,
                    );
                    renderdoc_capture_payload = Some(payload);
                } else {
                    repro_process_footprint = repro_process_footprint.or(stop_launched_demo(
                        &mut child,
                        &resolved_exit_path,
                        poll_ms,
                    ));
                }
            } else {
                repro_process_footprint = repro_process_footprint.or(stop_launched_demo(
                    &mut child,
                    &resolved_exit_path,
                    poll_ms,
                ));
            }

            let repro_process_footprint_file = resolved_out_dir.join("resource.footprint.json");
            if let Some(payload) = repro_process_footprint.as_ref() {
                let _ = write_json_value(&repro_process_footprint_file, payload);
            }
            if resource_footprint_thresholds.any() {
                resource_footprint_gate = check_resource_footprint_thresholds(
                    &resolved_out_dir,
                    &repro_process_footprint_file,
                    &resource_footprint_thresholds,
                )
                .ok();
            }
            if let Some(max_total_ms) = check_redraw_hitches_max_total_ms_threshold {
                redraw_hitches_gate =
                    check_redraw_hitches_max_total_ms(&resolved_out_dir, max_total_ms).ok();
            }

            let captures_json = serde_json::json!({
                "tracy": if with_tracy {
                    serde_json::json!({
                        "requested": true,
                        "env_enabled": tracy_env_enabled,
                        "feature_injected": tracy_feature_injected,
                        "note": "Capture is not recorded automatically yet; use the Tracy UI to save a capture."
                    })
                } else {
                    serde_json::Value::Null
                },
                "renderdoc": if with_renderdoc {
                    renderdoc_capture_payload.clone().unwrap_or_else(|| serde_json::json!({
                        "schema_version": 2,
                        "generated_unix_ms": now_unix_ms(),
                        "capture_dir": "renderdoc",
                        "autocapture_after_frames": renderdoc_autocapture_after_frames,
                        "captures": [],
                    }))
                } else {
                    serde_json::Value::Null
                }
            });

            let summary_json = serde_json::json!({
                "schema_version": 1,
                "generated_unix_ms": now_unix_ms(),
                "out_dir": resolved_out_dir.display().to_string(),
                "suite": suite_name,
                "capabilities": serde_json::json!({
                    "required": required_caps,
                    "available": available_caps,
                    "check_file": if capabilities_check_path.is_file() { Some("check.capabilities.json") } else { None },
                }),
                "scripts": run_rows,
                "selected_bundle_json": selected_bundle_path.as_ref().map(|p| p.display().to_string()),
                "packed_bundle_json": packed_bundle_json.as_ref().map(|p| p.display().to_string()),
                "packed_bundles": packed_bundles,
                "repro_zip": Some(zip_out.display().to_string()),
                "resources": serde_json::json!({
                    "process_footprint_file": if repro_process_footprint_file.is_file() {
                        Some("resource.footprint.json")
                    } else {
                        None
                    },
                    "process_footprint": repro_process_footprint,
                }),
                "captures": captures_json,
                "last_result": last_script_result.as_ref().map(|r| serde_json::json!({
                    "run_id": r.run_id,
                    "stage": r.stage,
                    "step_index": r.step_index,
                    "reason": r.reason,
                    "last_bundle_dir": r.last_bundle_dir,
                })),
                "error": overall_error,
            });

            if let Some(parent) = summary_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = write_json_value(&summary_path, &summary_json);
            let _ = write_evidence_index(&resolved_out_dir, &summary_path, Some(&summary_json));

            if overall_error.is_none() {
                let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);
                if multi_pack {
                    let bundles: Vec<ReproZipBundle> = pack_items
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| ReproZipBundle {
                            prefix: repro_zip_prefix_for_script(item, idx),
                            bundle_json: item.bundle_json.clone(),
                            source_script: item.script_path.clone(),
                        })
                        .collect();

                    if let Err(err) = pack_repro_zip_multi(
                        &zip_out,
                        pack_defaults.0,
                        pack_defaults.1,
                        pack_defaults.2,
                        with_renderdoc,
                        with_tracy,
                        &resolved_out_dir,
                        &summary_path,
                        &bundles,
                        stats_top,
                        sort,
                        warmup_frames,
                    ) {
                        overall_error = Some(format!("failed to pack repro zip: {err}"));
                    } else {
                        packed_zip = Some(zip_out.clone());
                    }
                } else if let Some(bundle_path) = selected_bundle_path.as_ref() {
                    let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
                    let artifacts_root = if bundle_dir.starts_with(&resolved_out_dir) {
                        resolved_out_dir.clone()
                    } else {
                        bundle_dir
                            .parent()
                            .unwrap_or(&resolved_out_dir)
                            .to_path_buf()
                    };

                    if let Err(err) = pack_bundle_dir_to_zip(
                        &bundle_dir,
                        &zip_out,
                        pack_defaults.0,
                        pack_defaults.1,
                        pack_defaults.2,
                        with_renderdoc,
                        with_tracy,
                        &artifacts_root,
                        stats_top,
                        sort,
                        warmup_frames,
                    ) {
                        overall_error = Some(format!("failed to pack repro zip: {err}"));
                    } else {
                        packed_zip = Some(zip_out.clone());
                    }
                } else {
                    overall_error = Some(
                        "no bundle.json found (add `capture_bundle` or enable script auto-dumps)"
                            .to_string(),
                    );
                }

                if overall_error.is_some() {
                    // Keep the summary coherent even when packing fails.
                    let _ = write_json_value(
                        &summary_path,
                        &summary_json
                            .as_object()
                            .cloned()
                            .map(|mut obj| {
                                obj.insert(
                                    "error".to_string(),
                                    serde_json::Value::String(
                                        overall_error.clone().unwrap_or_default(),
                                    ),
                                );
                                serde_json::Value::Object(obj)
                            })
                            .unwrap_or(summary_json.clone()),
                    );
                }
            }

            if let Some(r) = resource_footprint_gate.as_ref()
                && r.failures > 0
                && overall_error.is_none()
            {
                overall_error = Some(format!(
                    "resource footprint threshold gate failed (failures={}, evidence={})",
                    r.failures,
                    r.evidence_path.display()
                ));
            }
            if let Some(r) = redraw_hitches_gate.as_ref()
                && r.failures > 0
                && overall_error.is_none()
            {
                overall_error = Some(format!(
                    "redraw hitch threshold gate failed (failures={}, evidence={})",
                    r.failures,
                    r.evidence_path.display()
                ));
            }

            let final_summary_json = summary_json
                .as_object()
                .cloned()
                .map(|mut obj| {
                    if let Some(err) = overall_error.as_ref() {
                        obj.insert("error".to_string(), serde_json::Value::String(err.clone()));
                    }
                    serde_json::Value::Object(obj)
                })
                .unwrap_or_else(|| summary_json.clone());
            let _ = write_json_value(&summary_path, &final_summary_json);
            if let Err(err) =
                write_evidence_index(&resolved_out_dir, &summary_path, Some(&final_summary_json))
            {
                eprintln!("WARN failed to write evidence index: {err}");
            }

            if let Some(path) = packed_bundle_json.as_ref() {
                println!("BUNDLE {}", path.display());
            }
            if let Some(path) = packed_zip.as_ref() {
                println!("PACK {}", path.display());
            }
            println!("SUMMARY {}", summary_path.display());

            if let Some(err) = overall_error {
                eprintln!("REPRO-FAIL {err}");
                std::process::exit(1);
            }

            println!("REPRO-OK");
            std::process::exit(0);
        }
        "suite" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if rest.is_empty() && suite_script_inputs.is_empty() {
                return Err(
                    "missing suite name or script paths (try: fretboard diag suite ui-gallery | fretboard diag suite docking-arbitration)"
                        .to_string(),
                );
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            enum BuiltinSuite {
                UiGallery,
                UiGalleryCodeEditor,
                UiGalleryLayout,
                DockingArbitration,
            }

            let suite_args: Vec<String> = rest.clone();

            let is_ui_gallery_suite = suite_args.len() == 1 && suite_args[0] == "ui-gallery";
            let is_ui_gallery_overlay_steady_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-overlay-steady";
            let is_ui_gallery_code_editor_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-code-editor";
            let is_ui_gallery_layout_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-layout";
            let is_ui_gallery_date_picker_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-date-picker";
            let is_ui_gallery_text_ime_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-text-ime";
            let is_ui_gallery_combobox_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-combobox";
            let is_ui_gallery_select_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-select";
            let is_ui_gallery_shadcn_conformance_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-shadcn-conformance";
            let is_ui_gallery_virt_retained_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-virt-retained";
            let is_ui_gallery_virt_retained_measured_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-virt-retained-measured";
            let is_ui_gallery_tree_retained_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-tree-retained";
            let is_ui_gallery_tree_retained_measured_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-tree-retained-measured";
            let is_ui_gallery_data_table_retained_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-data-table-retained";
            let is_ui_gallery_data_table_retained_measured_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-data-table-retained-measured";
            let is_ui_gallery_data_table_retained_keep_alive_suite = suite_args.len() == 1
                && suite_args[0] == "ui-gallery-data-table-retained-keep-alive";
            let is_ui_gallery_table_retained_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-table-retained";
            let is_ui_gallery_table_retained_measured_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-table-retained-measured";
            let is_ui_gallery_retained_measured_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-retained-measured";
            let is_ui_gallery_ai_transcript_retained_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-ai-transcript-retained";
            let is_ui_gallery_canvas_cull_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-canvas-cull";
            let is_ui_gallery_node_graph_cull_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-node-graph-cull";
            let is_ui_gallery_node_graph_cull_window_shifts_suite = suite_args.len() == 1
                && suite_args[0] == "ui-gallery-node-graph-cull-window-shifts";
            let is_ui_gallery_node_graph_cull_window_no_shifts_small_pan_suite = suite_args.len()
                == 1
                && suite_args[0] == "ui-gallery-node-graph-cull-window-no-shifts-small-pan";
            let is_ui_gallery_chart_torture_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-chart-torture";
            let is_ui_gallery_vlist_window_boundary_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-vlist-window-boundary";
            let is_ui_gallery_vlist_window_boundary_retained_suite = suite_args.len() == 1
                && suite_args[0] == "ui-gallery-vlist-window-boundary-retained";
            let is_ui_gallery_vlist_no_window_shifts_small_scroll_suite = suite_args.len() == 1
                && suite_args[0] == "ui-gallery-vlist-no-window-shifts-small-scroll";
            let is_ui_gallery_ui_kit_list_retained_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-ui-kit-list-retained";
            let is_ui_gallery_inspector_torture_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-inspector-torture";
            let is_ui_gallery_inspector_torture_keep_alive_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-inspector-torture-keep-alive";
            let is_ui_gallery_file_tree_torture_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-file-tree-torture";
            let is_ui_gallery_file_tree_torture_interactive_suite = suite_args.len() == 1
                && suite_args[0] == "ui-gallery-file-tree-torture-interactive";
            let is_ui_gallery_cache005_suite =
                suite_args.len() == 1 && suite_args[0] == "ui-gallery-cache005";
            let is_components_gallery_file_tree_suite =
                suite_args.len() == 1 && suite_args[0] == "components-gallery-file-tree";
            let is_components_gallery_table_suite =
                suite_args.len() == 1 && suite_args[0] == "components-gallery-table";
            let is_components_gallery_table_keep_alive_suite =
                suite_args.len() == 1 && suite_args[0] == "components-gallery-table-keep-alive";
            let is_workspace_shell_demo_suite =
                suite_args.len() == 1 && suite_args[0] == "workspace-shell-demo";
            let is_workspace_shell_demo_file_tree_keep_alive_suite = suite_args.len() == 1
                && suite_args[0] == "workspace-shell-demo-file-tree-keep-alive";
            let is_docking_arbitration_suite =
                suite_args.len() == 1 && suite_args[0] == "docking-arbitration";

            let (mut scripts, builtin_suite): (Vec<PathBuf>, Option<BuiltinSuite>) =
                if is_ui_gallery_suite {
                    // The UI Gallery suite includes scripts that run the `--check-pixels-changed`
                    // post-run gate. Enable screenshots so those checks can resolve semantics
                    // bounds against captured PNGs.
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_SCREENSHOTS", "1");
                    (
                        ui_gallery_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_overlay_steady_suite {
                    (
                        ui_gallery_overlay_steady_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_code_editor_suite {
                    // The code-editor-focused UI Gallery suite also includes the pixels-changed
                    // gate (soft-wrap editing baseline), so screenshots must be enabled.
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_SCREENSHOTS", "1");
                    (
                        ui_gallery_code_editor_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some(BuiltinSuite::UiGalleryCodeEditor),
                    )
                } else if is_ui_gallery_date_picker_suite {
                    // Date picker scripts rely on stable role-and-name semantics selectors; ensure
                    // diagnostics redaction is disabled so selectors can match.
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_REDACT_TEXT", "0");
                    // Use deterministic date + page seed so roving navigation is repeatable.
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_UI_GALLERY_START_PAGE",
                        "date_picker",
                    );
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_UI_GALLERY_DIAG_CALENDAR_ROVING",
                        "1",
                    );
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_UI_GALLERY_FIXED_TODAY",
                        "2024-02-01",
                    );
                    (
                        ui_gallery_date_picker_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_text_ime_suite {
                    (
                        ui_gallery_text_ime_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_select_suite {
                    // Keep this suite redaction-friendly: scripts should prefer `test_id` selectors
                    // so we can share bundles by default without leaking labels/values.
                    (
                        ui_gallery_select_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_combobox_suite {
                    (
                        ui_gallery_combobox_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_shadcn_conformance_suite {
                    // Conformance scripts rely on stable role-and-name semantics selectors and use
                    // screenshot evidence for overlap regressions.
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_REDACT_TEXT", "0");
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_SCREENSHOTS", "1");
                    // Ensure bundled fonts are loaded on desktop so font metrics are deterministic.
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_BOOTSTRAP_FONTS", "1");

                    let mut scripts: Vec<PathBuf> = ui_gallery_shadcn_conformance_suite_scripts()
                        .into_iter()
                        .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                        .collect();
                    scripts.extend(
                        ui_gallery_select_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p))),
                    );

                    (scripts, Some(BuiltinSuite::UiGallery))
                } else if is_ui_gallery_layout_suite {
                    (
                        ui_gallery_layout_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some(BuiltinSuite::UiGalleryLayout),
                    )
                } else if is_ui_gallery_virt_retained_suite
                    || is_ui_gallery_virt_retained_measured_suite
                {
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll-retained.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_tree_retained_suite
                    || is_ui_gallery_tree_retained_measured_suite
                {
                    (
                        vec![
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-tree-window-boundary-scroll-retained.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-tree-retained-toggle-and-scroll.json",
                                ),
                            ),
                        ],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_data_table_retained_suite
                    || is_ui_gallery_data_table_retained_measured_suite
                {
                    (
                        vec![
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-window-boundary-scroll-retained.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-sort-select-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-multi-sort-shift-click.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-visibility-toggle.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-column-actions-menu.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-global-filter.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-column-filter.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-faceted-filter.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-reset-filters.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-column-pinning-sticky-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-column-pinning-toggle.json",
                                ),
                            ),
                        ],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_data_table_retained_keep_alive_suite {
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-data-table-window-boundary-bounce-keep-alive.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_table_retained_suite
                    || is_ui_gallery_table_retained_measured_suite
                {
                    (
                        vec![
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-window-boundary-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-sort-select-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-sort-desc.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-multi-sort-shift-click.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-row-pinning-keep-pinned-true.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-row-pinning-keep-pinned-false.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-keyboard-typeahead.json",
                                ),
                            ),
                        ],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_retained_measured_suite {
                    (
                        vec![
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll-retained.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-tree-window-boundary-scroll-retained.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-tree-retained-toggle-and-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-window-boundary-scroll-retained.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-data-table-retained-sort-select-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-window-boundary-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-sort-select-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-table-retained-keyboard-typeahead.json",
                                ),
                            ),
                        ],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_ai_transcript_retained_suite {
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_UI_GALLERY_AI_TRANSCRIPT_VARIABLE_HEIGHT",
                        "1",
                    );
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-ai-transcript-torture-scroll.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_canvas_cull_suite {
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
                    // This harness uses `capture_screenshot` to enable the `--check-pixels-changed` gate.
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_SCREENSHOTS", "1");
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-canvas-cull-torture-pan-zoom.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_node_graph_cull_suite {
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
                    // This harness uses `capture_screenshot` to enable the `--check-pixels-changed` gate.
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_SCREENSHOTS", "1");
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-node-graph-cull-torture-pan-zoom.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_node_graph_cull_window_shifts_suite {
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
                    // This harness uses `capture_screenshot` to enable the `--check-pixels-changed` gate.
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_SCREENSHOTS", "1");
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-node-graph-cull-window-shifts.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_node_graph_cull_window_no_shifts_small_pan_suite {
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
                    // This harness uses `capture_screenshot` to enable the `--check-pixels-changed` gate.
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_SCREENSHOTS", "1");
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-node-graph-cull-window-no-shifts-small-pan.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_chart_torture_suite {
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
                    // This harness uses `capture_screenshot` to enable the `--check-pixels-changed` gate.
                    push_env_if_missing(&mut launch_env, "FRET_DIAG_SCREENSHOTS", "1");
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-chart-torture-pan-zoom.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_vlist_window_boundary_suite {
                    // The window-boundary harness is specifically intended to exercise the
                    // view-cache + shell reuse seam under a stable (known-heights) VirtualList
                    // baseline. Make these env defaults implicit so the suite is reproducible
                    // without requiring the caller to remember a pile of `--env` flags.
                    //
                    // Callers can still override them explicitly via `--env KEY=...`.
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS",
                        "1",
                    );
                    // Default to the non-retained VirtualList path so this harness gates the
                    // highest-risk, most common implementation track (ADR 0175 Track B). The
                    // retained-host track (ADR 0177) has dedicated suites/scripts.
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VLIST_RETAINED", "0");
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_vlist_no_window_shifts_small_scroll_suite {
                    // Guard rail harness: under view-cache + shell, small scroll deltas should
                    // not force a non-retained VirtualList window shift (which currently implies
                    // a cache-root rerender to rebuild visible items).
                    //
                    // Callers can still override env explicitly via `--env KEY=...`.
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VLIST_MINIMAL", "1");
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS",
                        "1",
                    );
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VLIST_RETAINED", "0");
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-virtual-list-small-scroll-no-window-shifts.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_vlist_window_boundary_retained_suite {
                    // Retained-host counterpart of the window-boundary harness. This suite is used
                    // to validate the ADR 0177 track (retained reconcile) with the same script and
                    // baseline env, while keeping the non-retained suite as the default.
                    //
                    // Callers can still override them explicitly via `--env KEY=...`.
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS",
                        "1",
                    );
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VLIST_RETAINED", "1");
                    // Enable keep-alive in the retained-host harness so boundary scroll back can
                    // reuse detached row subtrees (reduces attach cost and stabilizes worst tick).
                    push_env_if_missing(&mut launch_env, "FRET_UI_GALLERY_VLIST_KEEP_ALIVE", "128");
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_ui_kit_list_retained_suite {
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-ui-kit-list-window-boundary-scroll.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_inspector_torture_suite {
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-inspector-torture-scroll.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_inspector_torture_keep_alive_suite {
                    (
                        vec![
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-inspector-torture-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-inspector-torture-bounce-keep-alive.json",
                                ),
                            ),
                        ],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_file_tree_torture_suite {
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-file-tree-torture-scroll.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_file_tree_torture_interactive_suite {
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/ui-gallery-file-tree-torture-toggle.json",
                            ),
                        )],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_ui_gallery_cache005_suite {
                    (
                        vec![
                            resolve_path(
                                &workspace_root,
                                PathBuf::from("tools/diag-scripts/ui-gallery-overlay-torture.json"),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/ui-gallery-sidebar-scroll-refresh.json",
                                ),
                            ),
                        ],
                        Some(BuiltinSuite::UiGallery),
                    )
                } else if is_components_gallery_file_tree_suite {
                    // components_gallery's "file tree torture" surface is behind env gates; the
                    // scripted harness assumes it is enabled and large enough to cross overscan
                    // boundaries deterministically.
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_COMPONENTS_GALLERY_FILE_TREE_TORTURE",
                        "1",
                    );
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_COMPONENTS_GALLERY_FILE_TREE_TORTURE_N",
                        "50000",
                    );
                    // Enable view-cache reuse by default for suite regressions. (components_gallery
                    // reads `FRET_EXAMPLES_VIEW_CACHE`.)
                    push_env_if_missing(&mut launch_env, "FRET_EXAMPLES_VIEW_CACHE", "1");
                    // Keep-alive is only observed by the `*bounce*` script, but setting it here
                    // keeps the suite defaults consistent.
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_COMPONENTS_GALLERY_FILE_TREE_KEEP_ALIVE",
                        "256",
                    );
                    (
                        vec![
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/components-gallery-file-tree-window-boundary-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/components-gallery-file-tree-toggle-and-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/components-gallery-file-tree-window-boundary-bounce.json",
                                ),
                            ),
                        ],
                        None,
                    )
                } else if is_components_gallery_table_suite {
                    // components_gallery's "table torture" surface is behind an env gate; the
                    // scripted harness assumes it is enabled.
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_COMPONENTS_GALLERY_TABLE_TORTURE",
                        "1",
                    );
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_COMPONENTS_GALLERY_TABLE_TORTURE_N",
                        "50000",
                    );
                    push_env_if_missing(&mut launch_env, "FRET_EXAMPLES_VIEW_CACHE", "1");
                    (
                        vec![
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/components-gallery-table-window-boundary-scroll.json",
                                ),
                            ),
                            resolve_path(
                                &workspace_root,
                                PathBuf::from(
                                    "tools/diag-scripts/components-gallery-table-sort-and-scroll.json",
                                ),
                            ),
                        ],
                        None,
                    )
                } else if is_components_gallery_table_keep_alive_suite {
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_COMPONENTS_GALLERY_TABLE_TORTURE",
                        "1",
                    );
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_COMPONENTS_GALLERY_TABLE_TORTURE_N",
                        "50000",
                    );
                    push_env_if_missing(&mut launch_env, "FRET_EXAMPLES_VIEW_CACHE", "1");
                    push_env_if_missing(
                        &mut launch_env,
                        "FRET_COMPONENTS_GALLERY_TABLE_KEEP_ALIVE",
                        "256",
                    );
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/components-gallery-table-window-boundary-bounce.json",
                            ),
                        )],
                        None,
                    )
                } else if is_workspace_shell_demo_suite {
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/workspace-shell-demo-tab-drag-and-scroll.json",
                            ),
                        )],
                        None,
                    )
                } else if is_workspace_shell_demo_file_tree_keep_alive_suite {
                    (
                        vec![resolve_path(
                            &workspace_root,
                            PathBuf::from(
                                "tools/diag-scripts/workspace-shell-demo-file-tree-bounce-keep-alive.json",
                            ),
                        )],
                        None,
                    )
                } else if is_docking_arbitration_suite {
                    (
                        docking_arbitration_suite_scripts()
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        Some(BuiltinSuite::DockingArbitration),
                    )
                } else {
                    (
                        suite_args
                            .into_iter()
                            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                            .collect(),
                        None,
                    )
                };

            if !suite_script_inputs.is_empty() {
                scripts.extend(expand_script_inputs(&workspace_root, &suite_script_inputs)?);
                scripts.sort();
                scripts.dedup();
            }

            if scripts.is_empty() {
                return Err("suite produced no scripts".to_string());
            }

            let suite_wants_screenshots = pack_include_screenshots
                || check_pixels_changed_test_id.is_some()
                || (check_pixels_changed_test_id.is_none()
                    && scripts
                        .iter()
                        .any(|src| ui_gallery_script_pixels_changed_test_id(src).is_some()))
                || scripts.iter().any(|src| script_requests_screenshots(src));
            // Suite defaults: most suites only need a small warmup to skip startup churn, but
            // "no shift" gates should avoid the initial VirtualList window stabilization phase.
            if warmup_frames == 0 && is_ui_gallery_vlist_no_window_shifts_small_scroll_suite {
                warmup_frames = 32;
            }

            if warmup_frames == 0
                && (is_ui_gallery_vlist_window_boundary_suite
                    || is_ui_gallery_vlist_window_boundary_retained_suite
                    || is_ui_gallery_canvas_cull_suite
                    || is_ui_gallery_node_graph_cull_suite
                    || is_ui_gallery_node_graph_cull_window_shifts_suite
                    || is_ui_gallery_node_graph_cull_window_no_shifts_small_pan_suite
                    || is_ui_gallery_chart_torture_suite
                    || is_ui_gallery_ai_transcript_retained_suite)
            {
                warmup_frames = 5;
            }

            let suite_launch_env = launch_env.clone();

            let reuse_process = launch.is_none() || reuse_launch;
            let mut child = if reuse_process {
                maybe_launch_demo(
                    &launch,
                    &suite_launch_env,
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    suite_wants_screenshots,
                    timeout_ms,
                    poll_ms,
                )?
            } else {
                None
            };

            let suite_summary_path = resolved_out_dir.join("suite.summary.json");
            let suite_summary_suite = (rest.len() == 1).then(|| rest[0].clone());
            let suite_summary_generated_unix_ms = now_unix_ms();
            let mut suite_stage_counts: std::collections::BTreeMap<String, u64> =
                std::collections::BTreeMap::new();
            let mut suite_reason_code_counts: std::collections::BTreeMap<String, u64> =
                std::collections::BTreeMap::new();
            let mut suite_rows: Vec<serde_json::Value> = Vec::new();
            let mut suite_evidence_agg = suite_summary::SuiteEvidenceAggregate::default();

            for src in scripts {
                if !reuse_process {
                    child = maybe_launch_demo(
                        &launch,
                        &suite_launch_env,
                        &workspace_root,
                        &resolved_out_dir,
                        &resolved_ready_path,
                        &resolved_exit_path,
                        suite_wants_screenshots,
                        timeout_ms,
                        poll_ms,
                    )?;
                }
                let mut result = run_script_and_wait(
                    &src,
                    &resolved_script_path,
                    &resolved_script_trigger_path,
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                    timeout_ms,
                    poll_ms,
                );
                if let Ok(summary) = &result
                    && summary.stage.as_deref() == Some("failed")
                {
                    if let Some(dir) = wait_for_failure_dump_bundle(
                        &resolved_out_dir,
                        summary,
                        timeout_ms,
                        poll_ms,
                    ) {
                        if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                            if let Ok(summary) = result.as_mut() {
                                summary.last_bundle_dir = Some(name.to_string());
                            }
                        }
                    }
                }

                let result = match result {
                    Ok(v) => v,
                    Err(e) => {
                        let script_key = normalize_repo_relative_path(&workspace_root, &src);
                        suite_rows.push(serde_json::json!({
                            "script": script_key,
                            "error": e,
                        }));
                        let payload = serde_json::json!({
                            "schema_version": 1,
                            "generated_unix_ms": suite_summary_generated_unix_ms,
                            "kind": "suite_summary",
                            "status": "error",
                            "suite": suite_summary_suite,
                            "out_dir": resolved_out_dir.display().to_string(),
                            "warmup_frames": warmup_frames,
                            "reuse_launch": reuse_launch,
                            "wants_screenshots": suite_wants_screenshots,
                            "stage_counts": suite_stage_counts,
                            "reason_code_counts": suite_reason_code_counts,
                            "evidence_aggregate": suite_evidence_agg.as_json(),
                            "rows": suite_rows,
                        });
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                        let _ = write_json_value(&suite_summary_path, &payload);
                        return Err("suite run failed (see suite.summary.json)".to_string());
                    }
                };

                if let Some(stage) = result.stage.as_deref() {
                    *suite_stage_counts.entry(stage.to_string()).or_default() += 1;
                }
                if let Some(code) = result.reason_code.as_deref() {
                    if !code.trim().is_empty() {
                        *suite_reason_code_counts
                            .entry(code.to_string())
                            .or_default() += 1;
                    }
                }

                let script_key = normalize_repo_relative_path(&workspace_root, &src);
                let mut lint_summary: Option<serde_json::Value> = None;
                let evidence_highlights =
                    suite_summary::evidence_highlights_from_script_result_path(
                        &resolved_script_result_path,
                        &mut suite_evidence_agg,
                    );
                match result.stage.as_deref() {
                    Some("passed") => {
                        println!("PASS {} (run_id={})", src.display(), result.run_id)
                    }
                    Some("failed") => {
                        eprintln!(
                            "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                            src.display(),
                            result.run_id,
                            result.step_index.unwrap_or(0),
                            result.reason.as_deref().unwrap_or("unknown"),
                            result.last_bundle_dir.as_deref().unwrap_or("")
                        );
                        suite_rows.push(serde_json::json!({
                            "script": script_key,
                            "run_id": result.run_id,
                            "stage": result.stage,
                            "step_index": result.step_index,
                            "reason_code": result.reason_code,
                            "reason": result.reason,
                            "last_bundle_dir": result.last_bundle_dir,
                            "lint": lint_summary,
                            "evidence_highlights": evidence_highlights,
                        }));
                        let payload = serde_json::json!({
                            "schema_version": 1,
                            "generated_unix_ms": suite_summary_generated_unix_ms,
                            "kind": "suite_summary",
                            "status": "failed",
                            "suite": suite_summary_suite,
                            "out_dir": resolved_out_dir.display().to_string(),
                            "warmup_frames": warmup_frames,
                            "reuse_launch": reuse_launch,
                            "wants_screenshots": suite_wants_screenshots,
                            "stage_counts": suite_stage_counts,
                            "reason_code_counts": suite_reason_code_counts,
                            "evidence_aggregate": suite_evidence_agg.as_json(),
                            "rows": suite_rows,
                        });
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                        let _ = write_json_value(&suite_summary_path, &payload);
                        std::process::exit(1);
                    }
                    _ => {
                        eprintln!(
                            "unexpected script stage for {}: {:?}",
                            src.display(),
                            result
                        );
                        suite_rows.push(serde_json::json!({
                            "script": script_key,
                            "run_id": result.run_id,
                            "stage": result.stage,
                            "step_index": result.step_index,
                            "reason_code": result.reason_code,
                            "reason": result.reason,
                            "last_bundle_dir": result.last_bundle_dir,
                            "lint": lint_summary,
                            "evidence_highlights": evidence_highlights,
                        }));
                        let payload = serde_json::json!({
                            "schema_version": 1,
                            "generated_unix_ms": suite_summary_generated_unix_ms,
                            "kind": "suite_summary",
                            "status": "failed",
                            "suite": suite_summary_suite,
                            "out_dir": resolved_out_dir.display().to_string(),
                            "warmup_frames": warmup_frames,
                            "reuse_launch": reuse_launch,
                            "wants_screenshots": suite_wants_screenshots,
                            "stage_counts": suite_stage_counts,
                            "reason_code_counts": suite_reason_code_counts,
                            "evidence_aggregate": suite_evidence_agg.as_json(),
                            "rows": suite_rows,
                        });
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                        let _ = write_json_value(&suite_summary_path, &payload);
                        std::process::exit(1);
                    }
                }

                if suite_lint {
                    let last_bundle_dir = result
                        .last_bundle_dir
                        .as_deref()
                        .and_then(|s| (!s.trim().is_empty()).then_some(s.trim()));
                    if let Some(last_bundle_dir) = last_bundle_dir {
                        let bundle_dir = PathBuf::from(last_bundle_dir);
                        let bundle_dir = if bundle_dir.is_absolute() {
                            bundle_dir
                        } else {
                            resolved_out_dir.join(bundle_dir)
                        };
                        let bundle_path = wait_for_bundle_json_in_dir(
                            &bundle_dir,
                            timeout_ms,
                            poll_ms,
                        )
                        .ok_or_else(|| {
                            format!(
                                "suite lint is enabled but bundle.json was not found in time: {}",
                                bundle_dir.display()
                            )
                        })?;

                        let report = lint_bundle_from_path(
                            &bundle_path,
                            warmup_frames,
                            LintOptions {
                                all_test_ids_bounds: lint_all_test_ids_bounds,
                                eps_px: lint_eps_px,
                            },
                        )?;

                        let out = default_lint_out_path(&bundle_path);
                        lint_summary = Some(serde_json::json!({
                            "out": out.display().to_string(),
                            "error_issues": report
                                .payload
                                .get("error_issues")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(report.error_issues),
                            "warning_issues": report
                                .payload
                                .get("warning_issues")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0),
                            "counts_by_code": report.payload.get("counts_by_code").cloned(),
                        }));
                        if let Some(parent) = out.parent() {
                            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                        }
                        let pretty = serde_json::to_string_pretty(&report.payload)
                            .unwrap_or_else(|_| "{}".to_string());
                        std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

                        if report.error_issues > 0 {
                            eprintln!(
                                "LINT-FAIL {} (run_id={}) errors={} out={}",
                                src.display(),
                                result.run_id,
                                report.error_issues,
                                out.display()
                            );
                            suite_rows.push(serde_json::json!({
                                "script": script_key,
                                "run_id": result.run_id,
                                "stage": result.stage,
                                "step_index": result.step_index,
                                "reason_code": result.reason_code,
                                "reason": result.reason,
                                "last_bundle_dir": result.last_bundle_dir,
                                "lint": lint_summary,
                                "evidence_highlights": evidence_highlights,
                            }));
                            let payload = serde_json::json!({
                                "schema_version": 1,
                                "generated_unix_ms": suite_summary_generated_unix_ms,
                                "kind": "suite_summary",
                                "status": "failed",
                                "suite": suite_summary_suite,
                                "out_dir": resolved_out_dir.display().to_string(),
                                "warmup_frames": warmup_frames,
                                "reuse_launch": reuse_launch,
                                "wants_screenshots": suite_wants_screenshots,
                                "stage_counts": suite_stage_counts,
                                "reason_code_counts": suite_reason_code_counts,
                                "evidence_aggregate": suite_evidence_agg.as_json(),
                                "rows": suite_rows,
                                "failure_kind": "lint_failed",
                            });
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            let _ = write_json_value(&suite_summary_path, &payload);
                            std::process::exit(1);
                        }
                    }
                }

                let retained_vlist_gate_for_script = check_retained_vlist_reconcile_no_notify_min
                    .filter(|_| ui_gallery_script_requires_retained_vlist_reconcile_gate(&src));
                let retained_vlist_attach_detach_max_for_script =
                    check_retained_vlist_attach_detach_max
                        .filter(|_| ui_gallery_script_requires_retained_vlist_reconcile_gate(&src));
                let retained_vlist_keep_alive_reuse_min_for_script =
                    check_retained_vlist_keep_alive_reuse_min.filter(|_| {
                        ui_gallery_script_requires_retained_vlist_keep_alive_reuse_gate(&src)
                    });
                let retained_vlist_keep_alive_budget_for_script =
                    check_retained_vlist_keep_alive_budget.filter(|_| {
                        ui_gallery_script_requires_retained_vlist_keep_alive_reuse_gate(&src)
                    });
                let vlist_window_shifts_non_retained_max_for_script =
                    check_vlist_window_shifts_non_retained_max
                        .filter(|_| ui_gallery_script_requires_retained_vlist_reconcile_gate(&src));
                let wants_post_run_checks_for_script = check_stale_paint_test_id.is_some()
                    || check_stale_scene_test_id.is_some()
                    || check_idle_no_paint_min.is_some()
                    || check_pixels_changed_test_id.is_some()
                    || check_ui_gallery_web_ime_bridge_enabled
                    || check_ui_gallery_code_editor_torture_marker_present
                    || check_ui_gallery_code_editor_torture_undo_redo
                    || check_ui_gallery_code_editor_torture_geom_fallbacks_low
                    || check_ui_gallery_code_editor_torture_read_only_blocks_edits
                    || check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed
                    || check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed
                    || check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed
                    || check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll
                    || check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection
                    || check_ui_gallery_code_editor_torture_folds_placeholder_present
                    || check_ui_gallery_code_editor_torture_inlays_present
                    || check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit
                    || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped
                    || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations
                    || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed
                    || check_ui_gallery_code_editor_word_boundary
                    || check_ui_gallery_code_editor_a11y_selection
                    || check_ui_gallery_code_editor_a11y_composition
                    || check_ui_gallery_code_editor_a11y_selection_wrap
                    || check_ui_gallery_code_editor_a11y_composition_wrap
                    || check_ui_gallery_code_editor_a11y_composition_wrap_scroll
                    || check_ui_gallery_code_editor_a11y_composition_drag
                    || check_semantics_changed_repainted
                    || check_wheel_scroll_test_id.is_some()
                    || check_wheel_scroll_hit_changes_test_id.is_some()
                    || check_prepaint_actions_min.is_some()
                    || check_chart_sampling_window_shifts_min.is_some()
                    || check_node_graph_cull_window_shifts_min.is_some()
                    || check_node_graph_cull_window_shifts_max.is_some()
                    || check_vlist_visible_range_refreshes_min.is_some()
                    || check_vlist_visible_range_refreshes_max.is_some()
                    || check_vlist_window_shifts_explainable
                    || check_drag_cache_root_paint_only_test_id.is_some()
                    || check_vlist_policy_key_stable
                    || check_windowed_rows_offset_changes_min.is_some()
                    || check_windowed_rows_visible_start_changes_repainted
                    || check_layout_fast_path_min.is_some()
                    || check_hover_layout_max.is_some()
                    || check_gc_sweep_liveness
                    || !check_notify_hotspot_file_max.is_empty()
                    || check_view_cache_reuse_min.is_some()
                    || check_view_cache_reuse_stable_min.is_some()
                    || check_overlay_synthesis_min.is_some()
                    || check_viewport_input_min.is_some()
                    || check_dock_drag_min.is_some()
                    || check_viewport_capture_min.is_some()
                    || retained_vlist_gate_for_script.is_some()
                    || retained_vlist_attach_detach_max_for_script.is_some()
                    || retained_vlist_keep_alive_reuse_min_for_script.is_some()
                    || retained_vlist_keep_alive_budget_for_script.is_some()
                    || vlist_window_shifts_non_retained_max_for_script.is_some()
                    || ui_gallery_script_requires_windowed_rows_offset_changes_gate(&src)
                    || ui_gallery_script_requires_windowed_rows_visible_start_repaint_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_read_only_blocks_edits_gate(
                        &src,
                    )
                    || ui_gallery_script_requires_markdown_editor_source_disabled_blocks_edits_gate(
                        &src,
                    )
                    || ui_gallery_script_requires_markdown_editor_source_soft_wrap_toggle_stable_gate(
                        &src,
                    )
                    || ui_gallery_script_requires_markdown_editor_source_word_boundary_gate(&src)
                    || ui_gallery_script_requires_web_ime_bridge_enabled_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_line_boundary_triple_click_gate(
                        &src,
                    )
                    || ui_gallery_script_requires_markdown_editor_source_a11y_composition_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_a11y_composition_soft_wrap_gate(
                        &src,
                    )
                    || ui_gallery_script_requires_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_folds_toggle_stable_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_gate(
                        &src,
                    )
                    || ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_under_soft_wrap_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_inlays_toggle_stable_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_inlays_caret_navigation_stable_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_inlays_present_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_inlays_present_under_soft_wrap_gate(&src)
                    || ui_gallery_script_requires_markdown_editor_source_inlays_absent_under_inline_preedit_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_composed_preedit_stable_after_wheel_scroll_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_composed_preedit_cancels_on_drag_selection_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_gate(&src)
                    || ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_gate(&src)
                    || ui_gallery_script_wheel_scroll_hit_changes_test_id(&src).is_some()
                    || ui_gallery_script_requires_retained_vlist_reconcile_gate(&src);

                let is_gc_liveness_script =
                    src.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                        n == "ui-gallery-overlay-torture.json"
                            || n == "ui-gallery-sidebar-scroll-refresh.json"
                    });

                let wants_post_run_checks_for_script = wants_post_run_checks_for_script
                    || builtin_suite == Some(BuiltinSuite::DockingArbitration)
                    || builtin_suite == Some(BuiltinSuite::UiGalleryCodeEditor)
                    || is_ui_gallery_canvas_cull_suite
                    || is_ui_gallery_chart_torture_suite
                    || is_ui_gallery_vlist_window_boundary_suite
                    || is_ui_gallery_vlist_window_boundary_retained_suite
                    || is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
                    || is_components_gallery_file_tree_suite
                    || is_components_gallery_table_suite
                    || is_components_gallery_table_keep_alive_suite
                    || (builtin_suite == Some(BuiltinSuite::UiGallery) && is_gc_liveness_script);

                if result.stage.as_deref() == Some("passed") && wants_post_run_checks_for_script {
                    let bundle_path = wait_for_bundle_json_from_script_result(
                        &resolved_out_dir,
                        &result,
                        timeout_ms,
                        poll_ms,
                    )
                    .ok_or_else(|| {
                        format!(
                            "script passed but no bundle.json was found (required for post-run checks): {}",
                            src.display()
                        )
                    })?;

                    let (suite_viewport_input_min, suite_dock_drag_min, suite_viewport_capture_min) =
                        if builtin_suite == Some(BuiltinSuite::DockingArbitration) {
                            docking_arbitration_script_default_gates(&src)
                        } else {
                            (None, None, None)
                        };
                    let vlist_window_boundary_suite = is_ui_gallery_vlist_window_boundary_suite
                        || is_ui_gallery_vlist_window_boundary_retained_suite;
                    let vlist_window_boundary_retained_suite =
                        is_ui_gallery_vlist_window_boundary_retained_suite;
                    let components_gallery_suite = is_components_gallery_file_tree_suite
                        || is_components_gallery_table_suite
                        || is_components_gallery_table_keep_alive_suite;
                    let pan_zoom_suite =
                        is_ui_gallery_canvas_cull_suite || is_ui_gallery_chart_torture_suite;
                    let ai_transcript_suite = is_ui_gallery_ai_transcript_retained_suite;
                    let suite_wheel_scroll_test_id =
                        is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
                            .then_some("ui-gallery-virtual-list-row-0-label")
                            .filter(|_| check_wheel_scroll_test_id.is_none());
                    let suite_components_gallery_stale_paint_test_id =
                        is_components_gallery_file_tree_suite
                            .then_some("components-gallery-file-tree-root")
                            .or_else(|| {
                                (is_components_gallery_table_suite
                                    || is_components_gallery_table_keep_alive_suite)
                                    .then_some("components-gallery-table-root")
                            })
                            .filter(|_| check_stale_paint_test_id.is_none());
                    let suite_ai_transcript_stale_paint_test_id = ai_transcript_suite
                        .then_some("ui-gallery-ai-transcript-row-0")
                        .filter(|_| check_stale_paint_test_id.is_none());
                    let suite_wheel_scroll_hit_changes_test_id =
                        ui_gallery_script_wheel_scroll_hit_changes_test_id(&src)
                            .or_else(|| {
                                is_components_gallery_file_tree_suite
                                    .then_some("components-gallery-file-tree-root")
                                    .or_else(|| {
                                        (is_components_gallery_table_suite
                                            || is_components_gallery_table_keep_alive_suite)
                                            .then_some("components-gallery-table-root")
                                    })
                            })
                            .filter(|_| check_wheel_scroll_hit_changes_test_id.is_none());
                    let suite_components_gallery_view_cache_reuse_min = components_gallery_suite
                        .then_some(1u64)
                        .filter(|_| check_view_cache_reuse_min.is_none());
                    let suite_layout_fast_path_min = components_gallery_suite
                        .then_some(1u64)
                        .filter(|_| check_layout_fast_path_min.is_none());
                    let suite_stale_paint_test_id = vlist_window_boundary_suite
                        .then_some("ui-gallery-virtual-list-root")
                        .or(suite_ai_transcript_stale_paint_test_id)
                        .filter(|_| check_stale_paint_test_id.is_none());
                    let suite_view_cache_reuse_min = (vlist_window_boundary_suite
                        || pan_zoom_suite)
                        .then_some(1u64)
                        .or_else(|| ai_transcript_suite.then_some(10u64))
                        .filter(|_| check_view_cache_reuse_min.is_none());
                    let suite_view_cache_reuse_stable_min = ai_transcript_suite
                        .then_some(10u64)
                        .filter(|_| check_view_cache_reuse_stable_min.is_none());
                    let suite_default_pixels_changed_test_id = is_ui_gallery_canvas_cull_suite
                        .then_some("ui-gallery-canvas-cull-root")
                        .or_else(|| {
                            is_ui_gallery_chart_torture_suite
                                .then_some("ui-gallery-chart-torture-root")
                        })
                        .filter(|_| check_pixels_changed_test_id.is_none());
                    let suite_vlist_visible_range_refreshes_min = vlist_window_boundary_suite
                        .then_some(1u64)
                        .filter(|_| check_vlist_visible_range_refreshes_min.is_none());
                    let suite_vlist_visible_range_refreshes_max = vlist_window_boundary_suite
                        // Default budget:
                        // - Non-retained path: keep this relatively tight so we catch churn
                        //   regressions early while still allowing prefetch shifts.
                        // - Retained-host path: allow a looser cap since reconcile can legitimately
                        //   refresh more often (and we have additional retained-only gates).
                        .then_some(if vlist_window_boundary_retained_suite {
                            50u64
                        } else {
                            20u64
                        })
                        .filter(|_| check_vlist_visible_range_refreshes_max.is_none());
                    let suite_vlist_window_shifts_explainable =
                        vlist_window_boundary_suite && !check_vlist_window_shifts_explainable;
                    let suite_prepaint_actions_min = vlist_window_boundary_suite
                        .then_some(1u64)
                        .filter(|_| check_prepaint_actions_min.is_none());
                    let suite_hover_layout_max = ai_transcript_suite
                        .then_some(0u32)
                        .filter(|_| check_hover_layout_max.is_none());
                    let suite_chart_sampling_window_shifts_min = is_ui_gallery_chart_torture_suite
                        .then_some(1u64)
                        .filter(|_| check_chart_sampling_window_shifts_min.is_none());
                    let suite_node_graph_cull_window_shifts_min =
                        is_ui_gallery_node_graph_cull_window_shifts_suite
                            .then_some(1u64)
                            .or_else(|| is_ui_gallery_node_graph_cull_suite.then_some(0u64))
                            .filter(|_| check_node_graph_cull_window_shifts_min.is_none());
                    let suite_node_graph_cull_window_shifts_max =
                        is_ui_gallery_node_graph_cull_window_no_shifts_small_pan_suite
                            .then_some(0u64)
                            .filter(|_| check_node_graph_cull_window_shifts_max.is_none());
                    let suite_vlist_window_shifts_have_prepaint_actions =
                        vlist_window_boundary_suite
                            && !check_vlist_window_shifts_have_prepaint_actions;
                    let suite_vlist_window_shifts_prefetch_max = vlist_window_boundary_suite
                        .then_some(if vlist_window_boundary_retained_suite {
                            100u64
                        } else {
                            12u64
                        })
                        .filter(|_| check_vlist_window_shifts_prefetch_max.is_none());
                    let suite_vlist_window_shifts_escape_max = vlist_window_boundary_suite
                        .then_some(if vlist_window_boundary_retained_suite {
                            6u64
                        } else {
                            4u64
                        })
                        .filter(|_| check_vlist_window_shifts_escape_max.is_none());
                    let script_requires_retained_vlist_reconcile_gate =
                        ui_gallery_script_requires_retained_vlist_reconcile_gate(&src)
                            || vlist_window_boundary_retained_suite;
                    let suite_vlist_window_shifts_non_retained_max =
                        script_requires_retained_vlist_reconcile_gate
                            .then_some(0u64)
                            .filter(|_| check_vlist_window_shifts_non_retained_max.is_none());
                    let suite_vlist_small_scroll_window_shifts_non_retained_max =
                        is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
                            .then_some(0u64)
                            .filter(|_| check_vlist_window_shifts_non_retained_max.is_none());
                    let suite_vlist_small_scroll_window_shifts_prefetch_max =
                        is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
                            .then_some(0u64)
                            .filter(|_| check_vlist_window_shifts_prefetch_max.is_none());
                    let suite_vlist_small_scroll_window_shifts_escape_max =
                        is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
                            .then_some(0u64)
                            .filter(|_| check_vlist_window_shifts_escape_max.is_none());
                    let suite_vlist_policy_key_stable = components_gallery_suite
                        && script_requires_retained_vlist_reconcile_gate
                        && !check_vlist_policy_key_stable;
                    let suite_windowed_rows_offset_changes_min =
                        ui_gallery_script_requires_windowed_rows_offset_changes_gate(&src)
                            .then_some(1u64)
                            .filter(|_| check_windowed_rows_offset_changes_min.is_none());
                    let suite_windowed_rows_visible_start_changes_repainted =
                        ui_gallery_script_requires_windowed_rows_visible_start_repaint_gate(&src)
                            && !check_windowed_rows_visible_start_changes_repainted;
                    let suite_pixels_changed_test_id =
                        ui_gallery_script_pixels_changed_test_id(&src)
                            .filter(|_| check_pixels_changed_test_id.is_none());
                    let suite_ui_gallery_code_editor_torture_marker_present =
                        ui_gallery_script_requires_code_editor_torture_marker_present_gate(&src)
                            && !check_ui_gallery_code_editor_torture_marker_present;
                    let suite_ui_gallery_code_editor_torture_undo_redo =
                        ui_gallery_script_requires_code_editor_torture_undo_redo_gate(&src)
                            && !check_ui_gallery_code_editor_torture_undo_redo;
                    let suite_ui_gallery_code_editor_torture_geom_fallbacks_low =
                        ui_gallery_script_requires_code_editor_torture_geom_fallbacks_low_gate(
                            &src,
                        ) && !check_ui_gallery_code_editor_torture_geom_fallbacks_low;
                    let suite_ui_gallery_code_editor_torture_read_only_blocks_edits =
                        ui_gallery_script_requires_code_editor_torture_read_only_blocks_edits_gate(
                            &src,
                        ) && !check_ui_gallery_code_editor_torture_read_only_blocks_edits;
                    let suite_ui_gallery_markdown_editor_source_read_only_blocks_edits =
                        ui_gallery_script_requires_markdown_editor_source_read_only_blocks_edits_gate(
                            &src,
                        ) && !check_ui_gallery_markdown_editor_source_read_only_blocks_edits;
                    let suite_ui_gallery_markdown_editor_source_disabled_blocks_edits =
                        ui_gallery_script_requires_markdown_editor_source_disabled_blocks_edits_gate(
                            &src,
                        )
                            && !check_ui_gallery_markdown_editor_source_disabled_blocks_edits;
                    let suite_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable =
                        ui_gallery_script_requires_markdown_editor_source_soft_wrap_toggle_stable_gate(
                            &src,
                        ) && !check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable;
                    let suite_ui_gallery_markdown_editor_source_word_boundary =
                        ui_gallery_script_requires_markdown_editor_source_word_boundary_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_word_boundary;
                    let suite_ui_gallery_web_ime_bridge_enabled =
                        ui_gallery_script_requires_web_ime_bridge_enabled_gate(&src)
                            && !check_ui_gallery_web_ime_bridge_enabled;
                    let suite_ui_gallery_markdown_editor_source_line_boundary_triple_click =
                        ui_gallery_script_requires_markdown_editor_source_line_boundary_triple_click_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_line_boundary_triple_click;
                    let suite_ui_gallery_markdown_editor_source_a11y_composition =
                        ui_gallery_script_requires_markdown_editor_source_a11y_composition_gate(
                            &src,
                        ) && !check_ui_gallery_markdown_editor_source_a11y_composition;
                    let suite_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap =
                        ui_gallery_script_requires_markdown_editor_source_a11y_composition_soft_wrap_gate(
                            &src,
                        ) && !check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap;
                    let suite_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable =
                        ui_gallery_script_requires_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable;
                    let suite_ui_gallery_markdown_editor_source_folds_toggle_stable =
                        ui_gallery_script_requires_markdown_editor_source_folds_toggle_stable_gate(
                            &src,
                        ) && !check_ui_gallery_markdown_editor_source_folds_toggle_stable;
                    let suite_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds =
                        ui_gallery_script_requires_markdown_editor_source_folds_clamp_selection_out_of_folds_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds;
                    let suite_ui_gallery_markdown_editor_source_folds_placeholder_present =
                        ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_folds_placeholder_present;
                    let suite_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap =
                        ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_under_soft_wrap_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap;
                    let suite_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit =
                        ui_gallery_script_requires_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit;
                    let suite_ui_gallery_markdown_editor_source_inlays_toggle_stable =
                        ui_gallery_script_requires_markdown_editor_source_inlays_toggle_stable_gate(
                            &src,
                        ) && !check_ui_gallery_markdown_editor_source_inlays_toggle_stable;
                    let suite_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable =
                        ui_gallery_script_requires_markdown_editor_source_inlays_caret_navigation_stable_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable;
                    let suite_ui_gallery_markdown_editor_source_inlays_present =
                        ui_gallery_script_requires_markdown_editor_source_inlays_present_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_inlays_present;
                    let suite_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap =
                        ui_gallery_script_requires_markdown_editor_source_inlays_present_under_soft_wrap_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap;
                    let suite_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit =
                        ui_gallery_script_requires_markdown_editor_source_inlays_absent_under_inline_preedit_gate(&src)
                            && !check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit;
                    let suite_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit =
                        ui_gallery_script_requires_code_editor_torture_folds_placeholder_absent_under_inline_preedit_gate(&src)
                            && !check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit;
                    let suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped =
                        ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_gate(&src)
                            && !check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped;
                    let suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations =
                        ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_gate(&src)
                            && !check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations;
                    let suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed =
                        ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_gate(&src)
                            && !check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed;
                    let suite_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed =
                        ui_gallery_script_requires_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_gate(&src)
                            && !check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed;
                    let suite_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed =
                        ui_gallery_script_requires_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_gate(&src)
                            && !check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed;
                    let suite_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll =
                        ui_gallery_script_requires_code_editor_torture_composed_preedit_stable_after_wheel_scroll_gate(&src)
                            && !check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll;
                    let suite_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection =
                        ui_gallery_script_requires_code_editor_torture_composed_preedit_cancels_on_drag_selection_gate(&src)
                            && !check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection;
                    let suite_ui_gallery_code_editor_torture_folds_placeholder_present =
                        ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_gate(&src)
                            && !check_ui_gallery_code_editor_torture_folds_placeholder_present;
                    let suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap =
                        ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_soft_wrap_gate(&src)
                            && !check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap;
                    let suite_ui_gallery_code_editor_torture_inlays_present =
                        ui_gallery_script_requires_code_editor_torture_inlays_present_gate(&src)
                            && !check_ui_gallery_code_editor_torture_inlays_present;
                    let suite_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit =
                        ui_gallery_script_requires_code_editor_torture_inlays_absent_under_inline_preedit_gate(&src)
                            && !check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit;
                    let suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped =
                        ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_gate(&src)
                            && !check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped;
                    let suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations =
                        ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_gate(&src)
                            && !check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations;
                    let suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed =
                        ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_gate(&src)
                            && !check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed;
                    let suite_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap =
                        ui_gallery_script_requires_code_editor_torture_inlays_present_under_soft_wrap_gate(&src)
                            && !check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap;
                    let suite_ui_gallery_code_editor_word_boundary =
                        ui_gallery_script_requires_code_editor_word_boundary_gate(&src)
                            && !check_ui_gallery_code_editor_word_boundary;
                    let suite_ui_gallery_code_editor_a11y_selection =
                        ui_gallery_script_requires_code_editor_a11y_selection_gate(&src)
                            && !check_ui_gallery_code_editor_a11y_selection;
                    let suite_ui_gallery_code_editor_a11y_composition =
                        ui_gallery_script_requires_code_editor_a11y_composition_gate(&src)
                            && !check_ui_gallery_code_editor_a11y_composition;
                    let suite_ui_gallery_code_editor_a11y_selection_wrap =
                        ui_gallery_script_requires_code_editor_a11y_selection_wrap_gate(&src)
                            && !check_ui_gallery_code_editor_a11y_selection_wrap;
                    let suite_ui_gallery_code_editor_a11y_composition_wrap =
                        ui_gallery_script_requires_code_editor_a11y_composition_wrap_gate(&src)
                            && !check_ui_gallery_code_editor_a11y_composition_wrap;
                    let suite_ui_gallery_code_editor_a11y_composition_wrap_scroll =
                        ui_gallery_script_requires_code_editor_a11y_composition_wrap_scroll_gate(
                            &src,
                        ) && !check_ui_gallery_code_editor_a11y_composition_wrap_scroll;
                    let suite_ui_gallery_code_editor_a11y_composition_drag =
                        ui_gallery_script_requires_code_editor_a11y_composition_drag_gate(&src)
                            && !check_ui_gallery_code_editor_a11y_composition_drag;
                    let script_requires_retained_vlist_keep_alive_reuse_gate =
                        ui_gallery_script_requires_retained_vlist_keep_alive_reuse_gate(&src);
                    let retained_vlist_suite = components_gallery_suite
                        || ai_transcript_suite
                        || vlist_window_boundary_retained_suite;
                    let suite_retained_vlist_reconcile_no_notify_min = (retained_vlist_suite
                        && script_requires_retained_vlist_reconcile_gate)
                        .then_some(1u64)
                        .filter(|_| check_retained_vlist_reconcile_no_notify_min.is_none());
                    let suite_retained_vlist_attach_detach_max = (retained_vlist_suite
                        && script_requires_retained_vlist_reconcile_gate)
                        .then_some(if vlist_window_boundary_retained_suite {
                            64u64
                        } else {
                            256u64
                        })
                        .filter(|_| check_retained_vlist_attach_detach_max.is_none());
                    let suite_retained_vlist_keep_alive_reuse_min = ((components_gallery_suite
                        && script_requires_retained_vlist_keep_alive_reuse_gate)
                        || vlist_window_boundary_retained_suite)
                        .then_some(if vlist_window_boundary_retained_suite {
                            5u64
                        } else {
                            1u64
                        })
                        .filter(|_| check_retained_vlist_keep_alive_reuse_min.is_none());
                    let suite_retained_vlist_keep_alive_budget = ((components_gallery_suite
                        && script_requires_retained_vlist_keep_alive_reuse_gate)
                        || vlist_window_boundary_retained_suite)
                        .then_some((1u64, 0u64))
                        .filter(|_| check_retained_vlist_keep_alive_budget.is_none());
                    let suite_gc_sweep_liveness =
                        builtin_suite == Some(BuiltinSuite::UiGallery) && is_gc_liveness_script;

                    let mut notify_hotspot_file_max_for_script =
                        check_notify_hotspot_file_max.clone();
                    if notify_hotspot_file_max_for_script.is_empty()
                        && builtin_suite == Some(BuiltinSuite::UiGallery)
                        && src
                            .file_name()
                            .and_then(|v| v.to_str())
                            .is_some_and(|v| v == "ui-gallery-virtual-list-torture.json")
                    {
                        notify_hotspot_file_max_for_script.push((
                            "crates/fret-ui/src/declarative/host_widget/event/pressable.rs"
                                .to_string(),
                            0,
                        ));
                    }
                    apply_post_run_checks(
                        &bundle_path,
                        &resolved_out_dir,
                        check_idle_no_paint_min,
                        check_stale_paint_test_id
                            .as_deref()
                            .or(suite_stale_paint_test_id)
                            .or(suite_components_gallery_stale_paint_test_id),
                        check_stale_paint_eps,
                        check_stale_scene_test_id.as_deref(),
                        check_stale_scene_eps,
                        check_pixels_changed_test_id
                            .as_deref()
                            .or(suite_pixels_changed_test_id)
                            .or(suite_default_pixels_changed_test_id),
                        check_ui_gallery_code_editor_torture_marker_present
                            || suite_ui_gallery_code_editor_torture_marker_present,
                        check_ui_gallery_code_editor_torture_undo_redo
                            || suite_ui_gallery_code_editor_torture_undo_redo,
                        check_ui_gallery_code_editor_torture_geom_fallbacks_low
                            || suite_ui_gallery_code_editor_torture_geom_fallbacks_low,
                        check_ui_gallery_code_editor_torture_read_only_blocks_edits
                            || suite_ui_gallery_code_editor_torture_read_only_blocks_edits,
                        check_ui_gallery_markdown_editor_source_read_only_blocks_edits
                            || suite_ui_gallery_markdown_editor_source_read_only_blocks_edits,
                        check_ui_gallery_markdown_editor_source_disabled_blocks_edits
                            || suite_ui_gallery_markdown_editor_source_disabled_blocks_edits,
                        check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable
                            || suite_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable,
                        check_ui_gallery_markdown_editor_source_word_boundary
                            || suite_ui_gallery_markdown_editor_source_word_boundary,
                        check_ui_gallery_web_ime_bridge_enabled
                            || suite_ui_gallery_web_ime_bridge_enabled,
                        check_ui_gallery_markdown_editor_source_line_boundary_triple_click
                            || suite_ui_gallery_markdown_editor_source_line_boundary_triple_click,
                        check_ui_gallery_markdown_editor_source_a11y_composition
                            || suite_ui_gallery_markdown_editor_source_a11y_composition,
                        check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap
                            || suite_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap,
                        check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable
                            || suite_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable,
                        check_ui_gallery_markdown_editor_source_folds_toggle_stable
                            || suite_ui_gallery_markdown_editor_source_folds_toggle_stable,
                        check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds
                            || suite_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds,
                        check_ui_gallery_markdown_editor_source_folds_placeholder_present
                            || suite_ui_gallery_markdown_editor_source_folds_placeholder_present,
                        check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap
                            || suite_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap,
                        check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit
                            || suite_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit,
                        check_ui_gallery_markdown_editor_source_inlays_toggle_stable
                            || suite_ui_gallery_markdown_editor_source_inlays_toggle_stable,
                        check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable
                            || suite_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable,
                        check_ui_gallery_markdown_editor_source_inlays_present
                            || suite_ui_gallery_markdown_editor_source_inlays_present,
                        check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap
                            || suite_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap,
                        check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit
                            || suite_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit,
                        check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit
                            || suite_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
                            || suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations
                            || suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed
                            || suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed,
                        check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed
                            || suite_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed,
                        check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed
                            || suite_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
                        check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll
                            || suite_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll,
                        check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection
                            || suite_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present
                            || suite_ui_gallery_code_editor_torture_folds_placeholder_present,
                        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap
                            || suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap,
                        check_ui_gallery_code_editor_torture_inlays_present
                            || suite_ui_gallery_code_editor_torture_inlays_present,
                        check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit
                            || suite_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit,
                        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped
                            || suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped,
                        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations
                            || suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations,
                        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed
                            || suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed,
                        check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap
                            || suite_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap,
                        check_ui_gallery_code_editor_word_boundary
                            || suite_ui_gallery_code_editor_word_boundary,
                        check_ui_gallery_code_editor_a11y_selection
                            || suite_ui_gallery_code_editor_a11y_selection,
                        check_ui_gallery_code_editor_a11y_composition
                            || suite_ui_gallery_code_editor_a11y_composition,
                        check_ui_gallery_code_editor_a11y_selection_wrap
                            || suite_ui_gallery_code_editor_a11y_selection_wrap,
                        check_ui_gallery_code_editor_a11y_composition_wrap
                            || suite_ui_gallery_code_editor_a11y_composition_wrap,
                        check_ui_gallery_code_editor_a11y_composition_wrap_scroll
                            || suite_ui_gallery_code_editor_a11y_composition_wrap_scroll,
                        check_ui_gallery_code_editor_a11y_composition_drag
                            || suite_ui_gallery_code_editor_a11y_composition_drag,
                        check_semantics_changed_repainted,
                        dump_semantics_changed_repainted_json,
                        check_wheel_scroll_test_id
                            .as_deref()
                            .or(suite_wheel_scroll_test_id),
                        check_wheel_scroll_hit_changes_test_id
                            .as_deref()
                            .or(suite_wheel_scroll_hit_changes_test_id),
                        check_prepaint_actions_min.or(suite_prepaint_actions_min),
                        check_chart_sampling_window_shifts_min
                            .or(suite_chart_sampling_window_shifts_min),
                        check_node_graph_cull_window_shifts_min
                            .or(suite_node_graph_cull_window_shifts_min),
                        check_node_graph_cull_window_shifts_max
                            .or(suite_node_graph_cull_window_shifts_max),
                        check_vlist_visible_range_refreshes_min
                            .or(suite_vlist_visible_range_refreshes_min),
                        check_vlist_visible_range_refreshes_max
                            .or(suite_vlist_visible_range_refreshes_max),
                        check_vlist_window_shifts_explainable
                            || suite_vlist_window_shifts_explainable,
                        check_vlist_window_shifts_have_prepaint_actions
                            || suite_vlist_window_shifts_have_prepaint_actions,
                        vlist_window_shifts_non_retained_max_for_script
                            .or(suite_vlist_window_shifts_non_retained_max)
                            .or(suite_vlist_small_scroll_window_shifts_non_retained_max),
                        check_vlist_window_shifts_prefetch_max
                            .or(suite_vlist_window_shifts_prefetch_max)
                            .or(suite_vlist_small_scroll_window_shifts_prefetch_max),
                        check_vlist_window_shifts_escape_max
                            .or(suite_vlist_window_shifts_escape_max)
                            .or(suite_vlist_small_scroll_window_shifts_escape_max),
                        check_vlist_policy_key_stable || suite_vlist_policy_key_stable,
                        check_windowed_rows_offset_changes_min
                            .or(suite_windowed_rows_offset_changes_min),
                        check_windowed_rows_offset_changes_eps,
                        check_windowed_rows_visible_start_changes_repainted
                            || suite_windowed_rows_visible_start_changes_repainted,
                        check_layout_fast_path_min.or(suite_layout_fast_path_min),
                        check_drag_cache_root_paint_only_test_id.as_deref(),
                        check_hover_layout_max.or(suite_hover_layout_max),
                        check_gc_sweep_liveness || suite_gc_sweep_liveness,
                        &notify_hotspot_file_max_for_script,
                        check_view_cache_reuse_stable_min.or(suite_view_cache_reuse_stable_min),
                        check_view_cache_reuse_min
                            .or(suite_view_cache_reuse_min)
                            .or(suite_components_gallery_view_cache_reuse_min),
                        check_overlay_synthesis_min,
                        check_viewport_input_min.or(suite_viewport_input_min),
                        check_dock_drag_min.or(suite_dock_drag_min),
                        check_viewport_capture_min.or(suite_viewport_capture_min),
                        retained_vlist_gate_for_script
                            .or(suite_retained_vlist_reconcile_no_notify_min),
                        retained_vlist_attach_detach_max_for_script
                            .or(suite_retained_vlist_attach_detach_max),
                        retained_vlist_keep_alive_reuse_min_for_script
                            .or(suite_retained_vlist_keep_alive_reuse_min),
                        retained_vlist_keep_alive_budget_for_script
                            .or(suite_retained_vlist_keep_alive_budget),
                        warmup_frames,
                    )?;
                }

                suite_rows.push(serde_json::json!({
                    "script": script_key,
                    "run_id": result.run_id,
                    "stage": result.stage,
                    "step_index": result.step_index,
                    "reason_code": result.reason_code,
                    "reason": result.reason,
                    "last_bundle_dir": result.last_bundle_dir,
                    "lint": lint_summary,
                    "evidence_highlights": evidence_highlights,
                }));

                if !reuse_process {
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                }
            }

            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
            let payload = serde_json::json!({
                "schema_version": 1,
                "generated_unix_ms": suite_summary_generated_unix_ms,
                "kind": "suite_summary",
                "status": "passed",
                "suite": suite_summary_suite,
                "out_dir": resolved_out_dir.display().to_string(),
                "warmup_frames": warmup_frames,
                "reuse_launch": reuse_launch,
                "wants_screenshots": suite_wants_screenshots,
                "stage_counts": suite_stage_counts,
                "reason_code_counts": suite_reason_code_counts,
                "evidence_aggregate": suite_evidence_agg.as_json(),
                "rows": suite_rows,
            });
            let _ = write_json_value(&suite_summary_path, &payload);
            if !stats_json {
                println!("SUITE-SUMMARY {}", suite_summary_path.display());
            }
            std::process::exit(0);
        }
        "perf-baseline-from-bundles" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if rest.len() < 2 {
                return Err(
                    "missing script path and bundle paths (try: fretboard diag perf-baseline-from-bundles tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json .fret/diag/exports/1234)".to_string(),
                );
            }

            let script_src = resolve_path(&workspace_root, PathBuf::from(&rest[0]));
            let script_key = normalize_repo_relative_path(&workspace_root, &script_src);

            let sort = sort_override.unwrap_or(BundleStatsSort::Time);
            let out_path = perf_baseline_out
                .clone()
                .ok_or_else(|| "missing --perf-baseline-out <path>".to_string())
                .map(|p| resolve_path(&workspace_root, p))?;

            let mut measured_max_top_total_us: u64 = 0;
            let mut measured_max_top_layout_us: u64 = 0;
            let mut measured_max_top_solve_us: u64 = 0;
            let mut measured_max_pointer_move_dispatch_us: u64 = 0;
            let mut measured_max_pointer_move_hit_test_us: u64 = 0;
            let mut measured_max_pointer_move_global_changes: u64 = 0;
            let mut measured_max_run_paint_cache_hit_test_only_replay_allowed_max: u64 = 0;
            let mut measured_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: u64 =
                0;

            let mut worst_bundle: Option<(u64, PathBuf)> = None;

            for raw in rest.iter().skip(1) {
                let bundle_src = resolve_path(&workspace_root, PathBuf::from(raw));
                let bundle_path = resolve_bundle_json_path(&bundle_src);
                if !bundle_path.is_file() {
                    return Err(format!(
                        "bundle path does not contain bundle.json: {}",
                        bundle_src.display()
                    ));
                }

                let report = bundle_stats_from_path(
                    &bundle_path,
                    1,
                    sort,
                    BundleStatsOptions { warmup_frames },
                )?;

                let top = report.top.first();
                let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
                let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
                let top_solve = top.map(|r| r.layout_engine_solve_time_us).unwrap_or(0);

                let pointer_move_dispatch = report.pointer_move_max_dispatch_time_us;
                let pointer_move_hit_test = report.pointer_move_max_hit_test_time_us;
                let pointer_move_global_changes =
                    report.pointer_move_snapshots_with_global_changes as u64;
                let (
                    run_paint_cache_hit_test_only_replay_allowed_max,
                    run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                ) = bundle_paint_cache_hit_test_only_replay_maxes(&bundle_path, warmup_frames)?;

                measured_max_top_total_us = measured_max_top_total_us.max(top_total);
                measured_max_top_layout_us = measured_max_top_layout_us.max(top_layout);
                measured_max_top_solve_us = measured_max_top_solve_us.max(top_solve);
                measured_max_pointer_move_dispatch_us =
                    measured_max_pointer_move_dispatch_us.max(pointer_move_dispatch);
                measured_max_pointer_move_hit_test_us =
                    measured_max_pointer_move_hit_test_us.max(pointer_move_hit_test);
                measured_max_pointer_move_global_changes =
                    measured_max_pointer_move_global_changes.max(pointer_move_global_changes);
                measured_max_run_paint_cache_hit_test_only_replay_allowed_max =
                    measured_max_run_paint_cache_hit_test_only_replay_allowed_max
                        .max(run_paint_cache_hit_test_only_replay_allowed_max);
                measured_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max =
                    measured_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max
                        .max(run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max);

                if top_total > worst_bundle.as_ref().map(|(t, _)| *t).unwrap_or(0) {
                    worst_bundle = Some((top_total, bundle_path));
                }
            }

            let thresholds = serde_json::json!({
                "max_top_total_us": apply_perf_baseline_headroom(measured_max_top_total_us, perf_baseline_headroom_pct),
                "max_top_layout_us": apply_perf_baseline_headroom(measured_max_top_layout_us, perf_baseline_headroom_pct),
                "max_top_solve_us": apply_perf_baseline_headroom(measured_max_top_solve_us, perf_baseline_headroom_pct),
                "max_pointer_move_dispatch_us": apply_perf_baseline_headroom(measured_max_pointer_move_dispatch_us, perf_baseline_headroom_pct),
                "max_pointer_move_hit_test_us": apply_perf_baseline_headroom(measured_max_pointer_move_hit_test_us, perf_baseline_headroom_pct),
                "max_pointer_move_global_changes": apply_perf_baseline_headroom(measured_max_pointer_move_global_changes, perf_baseline_headroom_pct),
                "min_run_paint_cache_hit_test_only_replay_allowed_max": apply_perf_baseline_floor(measured_max_run_paint_cache_hit_test_only_replay_allowed_max, perf_baseline_headroom_pct),
                "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": apply_perf_baseline_headroom(measured_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max, perf_baseline_headroom_pct),
            });

            let measured_max = serde_json::json!({
                "top_total_time_us": measured_max_top_total_us,
                "top_layout_time_us": measured_max_top_layout_us,
                "top_layout_engine_solve_time_us": measured_max_top_solve_us,
                "pointer_move_max_dispatch_time_us": measured_max_pointer_move_dispatch_us,
                "pointer_move_max_hit_test_time_us": measured_max_pointer_move_hit_test_us,
                "pointer_move_snapshots_with_global_changes": measured_max_pointer_move_global_changes,
                "run_paint_cache_hit_test_only_replay_allowed_max": measured_max_run_paint_cache_hit_test_only_replay_allowed_max,
                "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": measured_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            });

            let row = serde_json::json!({
                "script": script_key,
                "thresholds": thresholds,
                "measured_max": measured_max,
                "worst_bundle": worst_bundle.as_ref().map(|(us, p)| serde_json::json!({
                    "top_total_time_us": us,
                    "bundle": normalize_repo_relative_path(&workspace_root, p),
                })),
            });

            let payload = serde_json::json!({
                "schema_version": 1,
                "generated_unix_ms": now_unix_ms(),
                "kind": "perf_baseline",
                "out_path": out_path.display().to_string(),
                "warmup_frames": warmup_frames,
                "sort": sort.as_str(),
                "repeat": (rest.len() - 1) as u64,
                "headroom_pct": perf_baseline_headroom_pct,
                "rows": [row],
            });

            write_json_value(&out_path, &payload)?;
            if !stats_json {
                println!("wrote perf baseline: {}", out_path.display());
            }

            Ok(())
        }
        "perf" => {
            if pack_after_run {
                return Err("--pack is only supported with `diag run`".to_string());
            }
            if rest.is_empty() {
                return Err(
                    "missing suite name or script paths (try: fretboard diag perf ui-gallery)"
                        .to_string(),
                );
            }

            let mut suite_name: Option<String> = None;
            let scripts: Vec<PathBuf> = if rest.len() == 1 {
                let name = rest[0].as_str();
                if let Some(paths) = perf_seed_policy::scripts_for_perf_suite_name(name) {
                    suite_name = Some(name.to_string());
                    paths
                        .iter()
                        .map(|p| resolve_path(&workspace_root, PathBuf::from(*p)))
                        .collect()
                } else if name == "ui-gallery" {
                    suite_name = Some(name.to_string());
                    [
                        "tools/diag-scripts/ui-gallery-overlay-torture.json",
                        "tools/diag-scripts/ui-gallery-dropdown-open-select.json",
                        "tools/diag-scripts/ui-gallery-context-menu-right-click.json",
                        "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json",
                        "tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json",
                        "tools/diag-scripts/ui-gallery-virtual-list-torture.json",
                        "tools/diag-scripts/ui-gallery-material3-tabs-switch-perf.json",
                        "tools/diag-scripts/ui-gallery-view-cache-toggle-perf.json",
                        "tools/diag-scripts/ui-gallery-window-resize-stress.json",
                    ]
                    .into_iter()
                    .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                    .collect()
                } else if name == "ui-gallery-steady" {
                    suite_name = Some(name.to_string());
                    [
                        "tools/diag-scripts/ui-gallery-overlay-torture-steady.json",
                        "tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json",
                        "tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json",
                        "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json",
                        "tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json",
                        "tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json",
                        "tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json",
                        "tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json",
                        "tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json",
                        "tools/diag-scripts/ui-gallery-window-resize-stress-steady.json",
                    ]
                    .into_iter()
                    .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                    .collect()
                } else if name == "ui-resize-probes" {
                    suite_name = Some(name.to_string());
                    [
                        "tools/diag-scripts/ui-gallery-window-resize-stress-steady.json",
                        "tools/diag-scripts/ui-gallery-window-resize-drag-jitter-steady.json",
                    ]
                    .into_iter()
                    .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                    .collect()
                } else if name == "ui-code-editor-resize-probes" {
                    suite_name = Some(name.to_string());
                    ["tools/diag-scripts/ui-gallery-code-editor-window-resize-drag-jitter-steady.json"]
                        .into_iter()
                        .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                        .collect()
                } else if name == "extras-marquee-steady" {
                    suite_name = Some(name.to_string());
                    ["tools/diag-scripts/extras-marquee-steady.json"]
                        .into_iter()
                        .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                        .collect()
                } else {
                    vec![resolve_path(&workspace_root, PathBuf::from(name))]
                }
            } else {
                rest.iter()
                    .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                    .collect()
            };

            let sort = sort_override.unwrap_or(BundleStatsSort::Time);
            let repeat = perf_repeat.max(1) as usize;
            let reuse_process = launch.is_none() || reuse_launch;
            let cli_thresholds = PerfThresholds {
                max_top_total_us,
                max_top_layout_us,
                max_top_solve_us,
                max_pointer_move_dispatch_us,
                max_pointer_move_hit_test_us,
                max_pointer_move_global_changes,
                min_run_paint_cache_hit_test_only_replay_allowed_max,
                max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            };
            let perf_baseline = perf_baseline_path
                .clone()
                .map(|p| resolve_path(&workspace_root, p))
                .map(|p| read_perf_baseline_file(&workspace_root, &p))
                .transpose()?;
            let perf_baseline_out = perf_baseline_out
                .clone()
                .map(|p| resolve_path(&workspace_root, p));

            let seed_policy: Option<ResolvedPerfBaselineSeedPolicy> = if perf_baseline_out.is_some()
            {
                Some(perf_seed_policy::resolve_perf_baseline_seed_policy(
                    &workspace_root,
                    suite_name.as_deref(),
                    &scripts,
                    &perf_baseline_seed_preset_paths,
                    &perf_baseline_seed_specs,
                )?)
            } else {
                None
            };
            let wants_perf_thresholds = cli_thresholds.any() || perf_baseline.is_some();
            let mut child: Option<LaunchedDemo> = None;
            let launched_by_fretboard = reuse_launch && launch.is_some();
            let mut perf_launch_env = launch_env.clone();
            let _ = ensure_env_var(&mut perf_launch_env, "FRET_DIAG_RENDERER_PERF", "1");

            let mut perf_json_rows: Vec<serde_json::Value> = Vec::new();
            let mut perf_threshold_rows: Vec<serde_json::Value> = Vec::new();
            let mut perf_threshold_failures: Vec<serde_json::Value> = Vec::new();
            let mut perf_baseline_rows: Vec<serde_json::Value> = Vec::new();
            let mut overall_worst: Option<(u64, PathBuf, PathBuf)> = None;
            let stats_opts = BundleStatsOptions { warmup_frames };

            if let Some(baseline) = perf_baseline.as_ref() {
                for src in &scripts {
                    let key = normalize_repo_relative_path(&workspace_root, src);
                    if !baseline.thresholds_by_script.contains_key(&key) {
                        return Err(format!(
                            "perf baseline missing entry for script: {key} (baseline={})",
                            baseline.path.display()
                        ));
                    }
                }
            }

            if launched_by_fretboard {
                child = maybe_launch_demo(
                    &launch,
                    &perf_launch_env,
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    false,
                    timeout_ms,
                    poll_ms,
                )?;
            }

            for src in scripts {
                if repeat == 1 {
                    if !reuse_process {
                        child = maybe_launch_demo(
                            &launch,
                            &perf_launch_env,
                            &workspace_root,
                            &resolved_out_dir,
                            &resolved_ready_path,
                            &resolved_exit_path,
                            false,
                            timeout_ms,
                            poll_ms,
                        )?;
                    }

                    if !reuse_process {
                        clear_script_result_files(
                            &resolved_script_result_path,
                            &resolved_script_result_trigger_path,
                        );
                    }

                    let mut result = run_script_and_wait(
                        &src,
                        &resolved_script_path,
                        &resolved_script_trigger_path,
                        &resolved_script_result_path,
                        &resolved_script_result_trigger_path,
                        timeout_ms,
                        poll_ms,
                    );
                    if let Ok(summary) = &result
                        && summary.stage.as_deref() == Some("failed")
                    {
                        if let Some(dir) = wait_for_failure_dump_bundle(
                            &resolved_out_dir,
                            summary,
                            timeout_ms,
                            poll_ms,
                        ) {
                            if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                                if let Ok(summary) = result.as_mut() {
                                    summary.last_bundle_dir = Some(name.to_string());
                                }
                            }
                        }
                    }
                    let result = match result {
                        Ok(v) => v,
                        Err(e) => {
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            return Err(e);
                        }
                    };

                    match result.stage.as_deref() {
                        Some("passed") => {}
                        Some("failed") => {
                            eprintln!(
                                "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                                src.display(),
                                result.run_id,
                                result.step_index.unwrap_or(0),
                                result.reason.as_deref().unwrap_or("unknown"),
                                result.last_bundle_dir.as_deref().unwrap_or("")
                            );
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            std::process::exit(1);
                        }
                        _ => {
                            eprintln!(
                                "unexpected script stage for {}: {:?}",
                                src.display(),
                                result
                            );
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            std::process::exit(1);
                        }
                    }

                    let bundle_dir = result
                        .last_bundle_dir
                        .as_deref()
                        .filter(|s| !s.trim().is_empty())
                        .map(PathBuf::from);

                    let script_key = normalize_repo_relative_path(&workspace_root, &src);

                    let bundle_path: Option<PathBuf> = match bundle_dir {
                        Some(bundle_dir) => {
                            Some(resolve_bundle_json_path(&resolved_out_dir.join(bundle_dir)))
                        }
                        None => read_latest_pointer(&resolved_out_dir)
                            .or_else(|| find_latest_export_dir(&resolved_out_dir))
                            .map(|path| resolve_bundle_json_path(path.as_path())),
                    };

                    if let Some(bundle_path) = bundle_path {
                        let mut report = bundle_stats_from_path(
                            &bundle_path,
                            stats_top.max(1),
                            sort,
                            stats_opts,
                        )?;
                        let mut report_warmup_frames = warmup_frames;
                        if warmup_frames > 0 && report.top.is_empty() {
                            report = bundle_stats_from_path(
                                &bundle_path,
                                stats_top.max(1),
                                sort,
                                BundleStatsOptions::default(),
                            )?;
                            report_warmup_frames = 0;
                        }
                        let top = report.top.first();
                        let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
                        let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
                        let top_solve = top.map(|r| r.layout_engine_solve_time_us).unwrap_or(0);
                        let top_solves = top.map(|r| r.layout_engine_solves).unwrap_or(0);
                        let top_prepaint = top.map(|r| r.prepaint_time_us).unwrap_or(0);
                        let top_paint = top.map(|r| r.paint_time_us).unwrap_or(0);
                        let top_dispatch = top.map(|r| r.dispatch_time_us).unwrap_or(0);
                        let top_hit_test = top.map(|r| r.hit_test_time_us).unwrap_or(0);
                        let top_dispatch_events = top.map(|r| r.dispatch_events).unwrap_or(0);
                        let top_hit_test_queries = top.map(|r| r.hit_test_queries).unwrap_or(0);
                        let pointer_move_frames_present = report.pointer_move_frames_present;
                        let pointer_move_frames_considered =
                            report.pointer_move_frames_considered as u64;
                        let pointer_move_max_dispatch_time_us =
                            report.pointer_move_max_dispatch_time_us;
                        let pointer_move_max_hit_test_time_us =
                            report.pointer_move_max_hit_test_time_us;
                        let pointer_move_snapshots_with_global_changes =
                            report.pointer_move_snapshots_with_global_changes as u64;
                        let (
                            run_paint_cache_hit_test_only_replay_allowed_max,
                            run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        ) = bundle_paint_cache_hit_test_only_replay_maxes(
                            &bundle_path,
                            report_warmup_frames,
                        )?;
                        let top_hit_test_bounds_tree_queries =
                            top.map(|r| r.hit_test_bounds_tree_queries).unwrap_or(0);
                        let top_hit_test_bounds_tree_disabled =
                            top.map(|r| r.hit_test_bounds_tree_disabled).unwrap_or(0);
                        let top_hit_test_bounds_tree_misses =
                            top.map(|r| r.hit_test_bounds_tree_misses).unwrap_or(0);
                        let top_hit_test_bounds_tree_hits =
                            top.map(|r| r.hit_test_bounds_tree_hits).unwrap_or(0);
                        let top_hit_test_bounds_tree_candidate_rejected = top
                            .map(|r| r.hit_test_bounds_tree_candidate_rejected)
                            .unwrap_or(0);
                        let top_frame_arena_capacity_estimate_bytes = top
                            .map(|r| r.frame_arena_capacity_estimate_bytes)
                            .unwrap_or(0);
                        let top_frame_arena_grow_events =
                            top.map(|r| r.frame_arena_grow_events).unwrap_or(0);
                        let top_element_children_vec_pool_reuses =
                            top.map(|r| r.element_children_vec_pool_reuses).unwrap_or(0);
                        let top_element_children_vec_pool_misses =
                            top.map(|r| r.element_children_vec_pool_misses).unwrap_or(0);
                        let top_frame = top.map(|r| r.frame_id).unwrap_or(0);
                        let top_tick = top.map(|r| r.tick_id).unwrap_or(0);
                        let top_view_cache_contained_relayouts =
                            top.map(|r| r.view_cache_contained_relayouts).unwrap_or(0);
                        let top_view_cache_roots_total =
                            top.map(|r| r.view_cache_roots_total).unwrap_or(0);
                        let top_view_cache_roots_reused =
                            top.map(|r| r.view_cache_roots_reused).unwrap_or(0);
                        let top_view_cache_roots_first_mount =
                            top.map(|r| r.view_cache_roots_first_mount).unwrap_or(0);
                        let top_view_cache_roots_node_recreated =
                            top.map(|r| r.view_cache_roots_node_recreated).unwrap_or(0);
                        let top_view_cache_roots_cache_key_mismatch = top
                            .map(|r| r.view_cache_roots_cache_key_mismatch)
                            .unwrap_or(0);
                        let top_view_cache_roots_not_marked_reuse_root = top
                            .map(|r| r.view_cache_roots_not_marked_reuse_root)
                            .unwrap_or(0);
                        let top_view_cache_roots_needs_rerender =
                            top.map(|r| r.view_cache_roots_needs_rerender).unwrap_or(0);
                        let top_view_cache_roots_layout_invalidated = top
                            .map(|r| r.view_cache_roots_layout_invalidated)
                            .unwrap_or(0);
                        let top_view_cache_roots_manual =
                            top.map(|r| r.view_cache_roots_manual).unwrap_or(0);
                        let top_cache_roots_contained_relayout =
                            top.map(|r| r.cache_roots_contained_relayout).unwrap_or(0);
                        let top_set_children_barrier_writes =
                            top.map(|r| r.set_children_barrier_writes).unwrap_or(0);
                        let top_barrier_relayouts_scheduled =
                            top.map(|r| r.barrier_relayouts_scheduled).unwrap_or(0);
                        let top_barrier_relayouts_performed =
                            top.map(|r| r.barrier_relayouts_performed).unwrap_or(0);
                        let top_virtual_list_visible_range_checks = top
                            .map(|r| r.virtual_list_visible_range_checks)
                            .unwrap_or(0);
                        let top_virtual_list_visible_range_refreshes = top
                            .map(|r| r.virtual_list_visible_range_refreshes)
                            .unwrap_or(0);
                        let top_renderer_tick_id = top.map(|r| r.renderer_tick_id).unwrap_or(0);
                        let top_renderer_frame_id = top.map(|r| r.renderer_frame_id).unwrap_or(0);
                        let top_renderer_encode_scene_us =
                            top.map(|r| r.renderer_encode_scene_us).unwrap_or(0);
                        let top_renderer_prepare_text_us =
                            top.map(|r| r.renderer_prepare_text_us).unwrap_or(0);
                        let top_renderer_prepare_svg_us =
                            top.map(|r| r.renderer_prepare_svg_us).unwrap_or(0);
                        let top_renderer_draw_calls =
                            top.map(|r| r.renderer_draw_calls).unwrap_or(0);
                        let top_renderer_pipeline_switches =
                            top.map(|r| r.renderer_pipeline_switches).unwrap_or(0);
                        let top_renderer_bind_group_switches =
                            top.map(|r| r.renderer_bind_group_switches).unwrap_or(0);
                        let top_renderer_scissor_sets =
                            top.map(|r| r.renderer_scissor_sets).unwrap_or(0);
                        let top_renderer_scene_encoding_cache_misses = top
                            .map(|r| r.renderer_scene_encoding_cache_misses)
                            .unwrap_or(0);
                        let top_renderer_text_atlas_upload_bytes =
                            top.map(|r| r.renderer_text_atlas_upload_bytes).unwrap_or(0);
                        let top_renderer_text_atlas_evicted_pages = top
                            .map(|r| r.renderer_text_atlas_evicted_pages)
                            .unwrap_or(0);
                        let top_renderer_svg_upload_bytes =
                            top.map(|r| r.renderer_svg_upload_bytes).unwrap_or(0);
                        let top_renderer_image_upload_bytes =
                            top.map(|r| r.renderer_image_upload_bytes).unwrap_or(0);
                        let top_renderer_svg_raster_cache_misses =
                            top.map(|r| r.renderer_svg_raster_cache_misses).unwrap_or(0);
                        let top_renderer_svg_raster_budget_evictions = top
                            .map(|r| r.renderer_svg_raster_budget_evictions)
                            .unwrap_or(0);
                        let top_renderer_svg_raster_budget_bytes =
                            top.map(|r| r.renderer_svg_raster_budget_bytes).unwrap_or(0);
                        let top_renderer_svg_rasters_live =
                            top.map(|r| r.renderer_svg_rasters_live).unwrap_or(0);
                        let top_renderer_svg_standalone_bytes_live = top
                            .map(|r| r.renderer_svg_standalone_bytes_live)
                            .unwrap_or(0);
                        let top_renderer_svg_mask_atlas_pages_live = top
                            .map(|r| r.renderer_svg_mask_atlas_pages_live)
                            .unwrap_or(0);
                        let top_renderer_svg_mask_atlas_bytes_live = top
                            .map(|r| r.renderer_svg_mask_atlas_bytes_live)
                            .unwrap_or(0);
                        let top_renderer_svg_mask_atlas_used_px =
                            top.map(|r| r.renderer_svg_mask_atlas_used_px).unwrap_or(0);
                        let top_renderer_svg_mask_atlas_capacity_px = top
                            .map(|r| r.renderer_svg_mask_atlas_capacity_px)
                            .unwrap_or(0);
                        let top_renderer_svg_raster_cache_hits =
                            top.map(|r| r.renderer_svg_raster_cache_hits).unwrap_or(0);
                        let top_renderer_svg_mask_atlas_page_evictions = top
                            .map(|r| r.renderer_svg_mask_atlas_page_evictions)
                            .unwrap_or(0);
                        let top_renderer_svg_mask_atlas_entries_evicted = top
                            .map(|r| r.renderer_svg_mask_atlas_entries_evicted)
                            .unwrap_or(0);
                        let top_renderer_intermediate_budget_bytes = top
                            .map(|r| r.renderer_intermediate_budget_bytes)
                            .unwrap_or(0);
                        let top_renderer_intermediate_in_use_bytes = top
                            .map(|r| r.renderer_intermediate_in_use_bytes)
                            .unwrap_or(0);
                        let top_renderer_intermediate_peak_in_use_bytes = top
                            .map(|r| r.renderer_intermediate_peak_in_use_bytes)
                            .unwrap_or(0);
                        let top_renderer_intermediate_release_targets = top
                            .map(|r| r.renderer_intermediate_release_targets)
                            .unwrap_or(0);
                        let top_renderer_intermediate_pool_allocations = top
                            .map(|r| r.renderer_intermediate_pool_allocations)
                            .unwrap_or(0);
                        let top_renderer_intermediate_pool_reuses = top
                            .map(|r| r.renderer_intermediate_pool_reuses)
                            .unwrap_or(0);
                        let top_renderer_intermediate_pool_releases = top
                            .map(|r| r.renderer_intermediate_pool_releases)
                            .unwrap_or(0);
                        let top_renderer_intermediate_pool_evictions = top
                            .map(|r| r.renderer_intermediate_pool_evictions)
                            .unwrap_or(0);
                        let top_renderer_intermediate_pool_free_bytes = top
                            .map(|r| r.renderer_intermediate_pool_free_bytes)
                            .unwrap_or(0);
                        let top_renderer_intermediate_pool_free_textures = top
                            .map(|r| r.renderer_intermediate_pool_free_textures)
                            .unwrap_or(0);

                        if stats_json {
                            perf_json_rows.push(serde_json::json!({
                                "script": script_key.clone(),
                                "sort": sort.as_str(),
                                "top_total_time_us": top_total,
                                "top_layout_time_us": top_layout,
                                "top_layout_engine_solve_time_us": top_solve,
                                "top_layout_engine_solves": top_solves,
                                "top_prepaint_time_us": top_prepaint,
                                "top_paint_time_us": top_paint,
                                "top_dispatch_time_us": top_dispatch,
                                "top_hit_test_time_us": top_hit_test,
                                "top_dispatch_events": top_dispatch_events,
                                "top_hit_test_queries": top_hit_test_queries,
                                "pointer_move_frames_present": pointer_move_frames_present,
                                "pointer_move_frames_considered": pointer_move_frames_considered,
                                "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                                "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                                "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                                "top_hit_test_bounds_tree_queries": top_hit_test_bounds_tree_queries,
                                "top_hit_test_bounds_tree_disabled": top_hit_test_bounds_tree_disabled,
                                "top_hit_test_bounds_tree_misses": top_hit_test_bounds_tree_misses,
                                "top_hit_test_bounds_tree_hits": top_hit_test_bounds_tree_hits,
                                "top_hit_test_bounds_tree_candidate_rejected": top_hit_test_bounds_tree_candidate_rejected,
                                "top_frame_arena_capacity_estimate_bytes": top_frame_arena_capacity_estimate_bytes,
                                "top_frame_arena_grow_events": top_frame_arena_grow_events,
                                "top_element_children_vec_pool_reuses": top_element_children_vec_pool_reuses,
                                "top_element_children_vec_pool_misses": top_element_children_vec_pool_misses,
                                "top_tick_id": top_tick,
                                "top_frame_id": top_frame,
                                "top_view_cache_contained_relayouts": top_view_cache_contained_relayouts,
                                "top_view_cache_roots_total": top_view_cache_roots_total,
                                "top_view_cache_roots_reused": top_view_cache_roots_reused,
                                "top_view_cache_roots_first_mount": top_view_cache_roots_first_mount,
                                "top_view_cache_roots_node_recreated": top_view_cache_roots_node_recreated,
                                "top_view_cache_roots_cache_key_mismatch": top_view_cache_roots_cache_key_mismatch,
                                "top_view_cache_roots_not_marked_reuse_root": top_view_cache_roots_not_marked_reuse_root,
                                "top_view_cache_roots_needs_rerender": top_view_cache_roots_needs_rerender,
                                "top_view_cache_roots_layout_invalidated": top_view_cache_roots_layout_invalidated,
                                "top_view_cache_roots_manual": top_view_cache_roots_manual,
                                "top_cache_roots_contained_relayout": top_cache_roots_contained_relayout,
                                "top_set_children_barrier_writes": top_set_children_barrier_writes,
                                "top_barrier_relayouts_scheduled": top_barrier_relayouts_scheduled,
                                "top_barrier_relayouts_performed": top_barrier_relayouts_performed,
                                "top_virtual_list_visible_range_checks": top_virtual_list_visible_range_checks,
                                "top_virtual_list_visible_range_refreshes": top_virtual_list_visible_range_refreshes,
                                "top_renderer_tick_id": top_renderer_tick_id,
                                "top_renderer_frame_id": top_renderer_frame_id,
                                "top_renderer_encode_scene_us": top_renderer_encode_scene_us,
                                "top_renderer_prepare_text_us": top_renderer_prepare_text_us,
                                "top_renderer_prepare_svg_us": top_renderer_prepare_svg_us,
                                "top_renderer_draw_calls": top_renderer_draw_calls,
                                "top_renderer_pipeline_switches": top_renderer_pipeline_switches,
                                "top_renderer_bind_group_switches": top_renderer_bind_group_switches,
                                "top_renderer_scissor_sets": top_renderer_scissor_sets,
                                "top_renderer_scene_encoding_cache_misses": top_renderer_scene_encoding_cache_misses,
                                "top_renderer_text_atlas_upload_bytes": top_renderer_text_atlas_upload_bytes,
                                "top_renderer_text_atlas_evicted_pages": top_renderer_text_atlas_evicted_pages,
                                "top_renderer_svg_upload_bytes": top_renderer_svg_upload_bytes,
                                "top_renderer_image_upload_bytes": top_renderer_image_upload_bytes,
                                "top_renderer_svg_raster_cache_misses": top_renderer_svg_raster_cache_misses,
                                "top_renderer_svg_raster_budget_evictions": top_renderer_svg_raster_budget_evictions,
                                "top_renderer_svg_raster_budget_bytes": top_renderer_svg_raster_budget_bytes,
                                "top_renderer_svg_rasters_live": top_renderer_svg_rasters_live,
                                "top_renderer_svg_standalone_bytes_live": top_renderer_svg_standalone_bytes_live,
                                "top_renderer_svg_mask_atlas_pages_live": top_renderer_svg_mask_atlas_pages_live,
                                "top_renderer_svg_mask_atlas_bytes_live": top_renderer_svg_mask_atlas_bytes_live,
                                "top_renderer_svg_mask_atlas_used_px": top_renderer_svg_mask_atlas_used_px,
	                                "top_renderer_svg_mask_atlas_capacity_px": top_renderer_svg_mask_atlas_capacity_px,
	                                "top_renderer_svg_raster_cache_hits": top_renderer_svg_raster_cache_hits,
	                                "top_renderer_svg_mask_atlas_page_evictions": top_renderer_svg_mask_atlas_page_evictions,
	                                "top_renderer_svg_mask_atlas_entries_evicted": top_renderer_svg_mask_atlas_entries_evicted,
	                                "top_renderer_intermediate_budget_bytes": top_renderer_intermediate_budget_bytes,
	                                "top_renderer_intermediate_in_use_bytes": top_renderer_intermediate_in_use_bytes,
	                                "top_renderer_intermediate_peak_in_use_bytes": top_renderer_intermediate_peak_in_use_bytes,
	                                "top_renderer_intermediate_release_targets": top_renderer_intermediate_release_targets,
	                                "top_renderer_intermediate_pool_allocations": top_renderer_intermediate_pool_allocations,
	                                "top_renderer_intermediate_pool_reuses": top_renderer_intermediate_pool_reuses,
	                                "top_renderer_intermediate_pool_releases": top_renderer_intermediate_pool_releases,
	                                "top_renderer_intermediate_pool_evictions": top_renderer_intermediate_pool_evictions,
	                                "top_renderer_intermediate_pool_free_bytes": top_renderer_intermediate_pool_free_bytes,
	                                "top_renderer_intermediate_pool_free_textures": top_renderer_intermediate_pool_free_textures,
	                                "bundle": bundle_path.display().to_string(),
	                            }));
                        } else {
                            println!(
                                "PERF {} sort={} top.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{} top.tick={} top.frame={} bundle={}",
                                src.display(),
                                sort.as_str(),
                                top_total,
                                top_layout,
                                top_solve,
                                top_prepaint,
                                top_paint,
                                top_dispatch,
                                top_hit_test,
                                top_tick,
                                top_frame,
                                bundle_path.display(),
                            );
                        }

                        if perf_baseline_out.is_some() {
                            let policy = seed_policy
                                .as_ref()
                                .ok_or_else(|| "internal error: missing seed policy".to_string())?;

                            let p90_total = top_total;
                            let p95_total = top_total;
                            let p90_layout = top_layout;
                            let p95_layout = top_layout;
                            let p90_solve = top_solve;
                            let p95_solve = top_solve;

                            let seed_total =
                                policy.seed_for(&script_key, PerfSeedMetric::TopTotalTimeUs);
                            let seed_layout =
                                policy.seed_for(&script_key, PerfSeedMetric::TopLayoutTimeUs);
                            let seed_solve = policy
                                .seed_for(&script_key, PerfSeedMetric::TopLayoutEngineSolveTimeUs);

                            let seed_total_value = match seed_total {
                                PerfBaselineSeed::Max => top_total,
                                PerfBaselineSeed::P90 => p90_total,
                                PerfBaselineSeed::P95 => p95_total,
                            };
                            let seed_layout_value = match seed_layout {
                                PerfBaselineSeed::Max => top_layout,
                                PerfBaselineSeed::P90 => p90_layout,
                                PerfBaselineSeed::P95 => p95_layout,
                            };
                            let seed_solve_value = match seed_solve {
                                PerfBaselineSeed::Max => top_solve,
                                PerfBaselineSeed::P90 => p90_solve,
                                PerfBaselineSeed::P95 => p95_solve,
                            };

                            let thr_total = apply_perf_baseline_headroom(
                                seed_total_value,
                                perf_baseline_headroom_pct,
                            );
                            let thr_layout = apply_perf_baseline_headroom(
                                seed_layout_value,
                                perf_baseline_headroom_pct,
                            );
                            let thr_solve = apply_perf_baseline_headroom(
                                seed_solve_value,
                                perf_baseline_headroom_pct,
                            );

                            let thr_pointer_move_dispatch = apply_perf_baseline_headroom(
                                pointer_move_max_dispatch_time_us,
                                perf_baseline_headroom_pct,
                            );
                            let thr_pointer_move_hit_test = apply_perf_baseline_headroom(
                                pointer_move_max_hit_test_time_us,
                                perf_baseline_headroom_pct,
                            );
                            let thr_pointer_move_global_changes = apply_perf_baseline_headroom(
                                pointer_move_snapshots_with_global_changes,
                                perf_baseline_headroom_pct,
                            );
                            let thr_min_run_paint_cache_hit_test_only_replay_allowed_max =
                                apply_perf_baseline_floor(
                                    run_paint_cache_hit_test_only_replay_allowed_max,
                                    perf_baseline_headroom_pct,
                                );
                            let thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max =
                                apply_perf_baseline_headroom(
                                    run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                    perf_baseline_headroom_pct,
                                );

                            perf_baseline_rows.push(serde_json::json!({
                                "script": script_key.clone(),
                                "measured_max": {
                                    "top_total_time_us": top_total,
                                    "top_layout_time_us": top_layout,
                                    "top_layout_engine_solve_time_us": top_solve,
                                    "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                                    "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                                    "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                                    "run_paint_cache_hit_test_only_replay_allowed_max": run_paint_cache_hit_test_only_replay_allowed_max,
                                    "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                },
                                "measured_p90": {
                                    "top_total_time_us": p90_total,
                                    "top_layout_time_us": p90_layout,
                                    "top_layout_engine_solve_time_us": p90_solve,
                                },
                                "measured_p95": {
                                    "top_total_time_us": p95_total,
                                    "top_layout_time_us": p95_layout,
                                    "top_layout_engine_solve_time_us": p95_solve,
                                },
                                "threshold_seed": {
                                    "top_total_time_us": seed_total_value,
                                    "top_layout_time_us": seed_layout_value,
                                    "top_layout_engine_solve_time_us": seed_solve_value,
                                },
                                "threshold_seed_source": {
                                    "top_total_time_us": seed_total.as_str(),
                                    "top_layout_time_us": seed_layout.as_str(),
                                    "top_layout_engine_solve_time_us": seed_solve.as_str(),
                                },
                                "thresholds": {
                                    "max_top_total_us": thr_total,
                                    "max_top_layout_us": thr_layout,
                                    "max_top_solve_us": thr_solve,
                                    "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
                                    "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
                                    "max_pointer_move_global_changes": thr_pointer_move_global_changes,
                                    "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_min_run_paint_cache_hit_test_only_replay_allowed_max,
                                    "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                },
                            }));
                        }
                        if wants_perf_thresholds {
                            let baseline_thresholds = perf_baseline
                                .as_ref()
                                .and_then(|b| b.thresholds_by_script.get(&script_key).copied())
                                .unwrap_or_default();
                            let (thr_total, src_total) = resolve_threshold(
                                cli_thresholds.max_top_total_us,
                                baseline_thresholds.max_top_total_us,
                            );
                            let (thr_layout, src_layout) = resolve_threshold(
                                cli_thresholds.max_top_layout_us,
                                baseline_thresholds.max_top_layout_us,
                            );
                            let (thr_solve, src_solve) = resolve_threshold(
                                cli_thresholds.max_top_solve_us,
                                baseline_thresholds.max_top_solve_us,
                            );
                            let (thr_pointer_move_dispatch, src_pointer_move_dispatch) =
                                resolve_threshold(
                                    cli_thresholds.max_pointer_move_dispatch_us,
                                    baseline_thresholds.max_pointer_move_dispatch_us,
                                );
                            let (thr_pointer_move_hit_test, src_pointer_move_hit_test) =
                                resolve_threshold(
                                    cli_thresholds.max_pointer_move_hit_test_us,
                                    baseline_thresholds.max_pointer_move_hit_test_us,
                                );
                            let (thr_pointer_move_global_changes, src_pointer_move_global_changes) =
                                resolve_threshold(
                                    cli_thresholds.max_pointer_move_global_changes,
                                    baseline_thresholds.max_pointer_move_global_changes,
                                );
                            let (
                                thr_paint_cache_hit_test_only_replay_allowed_max,
                                src_paint_cache_hit_test_only_replay_allowed_max,
                            ) = resolve_threshold(
                                cli_thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max,
                                baseline_thresholds
                                    .min_run_paint_cache_hit_test_only_replay_allowed_max,
                            );
                            let (
                                thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            ) = resolve_threshold(
                                cli_thresholds
                                    .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                baseline_thresholds
                                    .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            );
                            let run = serde_json::json!({
                                "run_index": 0,
                                "top_total_time_us": top_total,
                                "top_layout_time_us": top_layout,
                                "top_layout_engine_solve_time_us": top_solve,
                                "top_layout_engine_solves": top_solves,
                                "pointer_move_frames_present": pointer_move_frames_present,
                                "pointer_move_frames_considered": pointer_move_frames_considered,
                                "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                                "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                                "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                                "run_paint_cache_hit_test_only_replay_allowed_max": run_paint_cache_hit_test_only_replay_allowed_max,
                                "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                "top_tick_id": top_tick,
                                "top_frame_id": top_frame,
                                "bundle": bundle_path.display().to_string(),
                            });
                            let row = serde_json::json!({
                                "script": script_key.clone(),
                                "sort": sort.as_str(),
                                "repeat": 1,
                                "runs": [run],
                                "max": {
                                    "top_total_time_us": top_total,
                                    "top_layout_time_us": top_layout,
                                    "top_layout_engine_solve_time_us": top_solve,
                                    "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                                    "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                                    "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                                    "run_paint_cache_hit_test_only_replay_allowed_max": run_paint_cache_hit_test_only_replay_allowed_max,
                                    "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                },
                                "thresholds": {
                                    "max_top_total_us": thr_total,
                                    "max_top_layout_us": thr_layout,
                                    "max_top_solve_us": thr_solve,
                                    "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
                                    "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
                                    "max_pointer_move_global_changes": thr_pointer_move_global_changes,
                                    "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_paint_cache_hit_test_only_replay_allowed_max,
                                    "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                },
                                "threshold_sources": {
                                    "max_top_total_us": src_total,
                                    "max_top_layout_us": src_layout,
                                    "max_top_solve_us": src_solve,
                                    "max_pointer_move_dispatch_us": src_pointer_move_dispatch,
                                    "max_pointer_move_hit_test_us": src_pointer_move_hit_test,
                                    "max_pointer_move_global_changes": src_pointer_move_global_changes,
                                    "min_run_paint_cache_hit_test_only_replay_allowed_max": src_paint_cache_hit_test_only_replay_allowed_max,
                                    "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                },
                            });
                            perf_threshold_rows.push(row);
                            perf_threshold_failures.extend(scan_perf_threshold_failures(
                                script_key.as_str(),
                                sort,
                                cli_thresholds,
                                baseline_thresholds,
                                top_total,
                                top_layout,
                                top_solve,
                                pointer_move_frames_present,
                                pointer_move_max_dispatch_time_us,
                                pointer_move_max_hit_test_time_us,
                                pointer_move_snapshots_with_global_changes,
                                run_paint_cache_hit_test_only_replay_allowed_max,
                                run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            ));
                        }

                        match &overall_worst {
                            Some((prev_us, _, _)) if *prev_us >= top_total => {}
                            _ => overall_worst = Some((top_total, src.clone(), bundle_path)),
                        }
                    } else {
                        if stats_json {
                            perf_json_rows.push(serde_json::json!({
                                "script": script_key.clone(),
                                "sort": sort.as_str(),
                                "error": "no_last_bundle_dir",
                            }));
                        } else {
                            println!(
                                "PERF {} sort={} (no last_bundle_dir recorded)",
                                src.display(),
                                sort.as_str()
                            );
                        }
                    }

                    if !reuse_process {
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                    }
                    continue;
                }

                let mut runs_total: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_layout: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_solve: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_prepaint: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_paint: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_dispatch: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_hit_test: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_pointer_move_dispatch: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_pointer_move_hit_test: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_pointer_move_global_changes: Vec<u64> = Vec::with_capacity(repeat);
                let mut runs_paint_cache_hit_test_only_replay_allowed_max: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut runs_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut runs_json: Vec<serde_json::Value> = Vec::with_capacity(repeat);
                let mut script_worst: Option<(u64, PathBuf)> = None;

                for run_index in 0..repeat {
                    if !reuse_process {
                        child = maybe_launch_demo(
                            &launch,
                            &perf_launch_env,
                            &workspace_root,
                            &resolved_out_dir,
                            &resolved_ready_path,
                            &resolved_exit_path,
                            false,
                            timeout_ms,
                            poll_ms,
                        )?;
                    }

                    if !reuse_process {
                        clear_script_result_files(
                            &resolved_script_result_path,
                            &resolved_script_result_trigger_path,
                        );
                    }

                    let mut result = run_script_and_wait(
                        &src,
                        &resolved_script_path,
                        &resolved_script_trigger_path,
                        &resolved_script_result_path,
                        &resolved_script_result_trigger_path,
                        timeout_ms,
                        poll_ms,
                    );
                    if let Ok(summary) = &result
                        && summary.stage.as_deref() == Some("failed")
                    {
                        if let Some(dir) = wait_for_failure_dump_bundle(
                            &resolved_out_dir,
                            summary,
                            timeout_ms,
                            poll_ms,
                        ) {
                            if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                                if let Ok(summary) = result.as_mut() {
                                    summary.last_bundle_dir = Some(name.to_string());
                                }
                            }
                        }
                    }
                    let result = match result {
                        Ok(v) => v,
                        Err(e) => {
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            return Err(e);
                        }
                    };

                    match result.stage.as_deref() {
                        Some("passed") => {}
                        Some("failed") => {
                            eprintln!(
                                "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                                src.display(),
                                result.run_id,
                                result.step_index.unwrap_or(0),
                                result.reason.as_deref().unwrap_or("unknown"),
                                result.last_bundle_dir.as_deref().unwrap_or("")
                            );
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            std::process::exit(1);
                        }
                        _ => {
                            eprintln!(
                                "unexpected script stage for {}: {:?}",
                                src.display(),
                                result
                            );
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                            std::process::exit(1);
                        }
                    }

                    let bundle_dir = result
                        .last_bundle_dir
                        .as_deref()
                        .filter(|s| !s.trim().is_empty())
                        .map(PathBuf::from);

                    let bundle_path: Option<PathBuf> = match bundle_dir {
                        Some(bundle_dir) => {
                            Some(resolve_bundle_json_path(&resolved_out_dir.join(bundle_dir)))
                        }
                        None => read_latest_pointer(&resolved_out_dir)
                            .or_else(|| find_latest_export_dir(&resolved_out_dir))
                            .map(|path| resolve_bundle_json_path(path.as_path())),
                    };

                    let Some(bundle_path) = bundle_path else {
                        if stats_json {
                            perf_json_rows.push(serde_json::json!({
                                "script": src.display().to_string(),
                                "sort": sort.as_str(),
                                "repeat": repeat,
                                "error": "no_last_bundle_dir",
                            }));
                        } else {
                            println!(
                                "PERF {} sort={} repeat={} (no last_bundle_dir recorded)",
                                src.display(),
                                sort.as_str(),
                                repeat
                            );
                        }
                        if !reuse_process {
                            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                        }
                        break;
                    };

                    let mut report =
                        bundle_stats_from_path(&bundle_path, stats_top.max(1), sort, stats_opts)?;
                    let mut report_warmup_frames = warmup_frames;
                    if warmup_frames > 0 && report.top.is_empty() {
                        report = bundle_stats_from_path(
                            &bundle_path,
                            stats_top.max(1),
                            sort,
                            BundleStatsOptions::default(),
                        )?;
                        report_warmup_frames = 0;
                    }
                    let top = report.top.first();
                    let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
                    let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
                    let top_solve = top.map(|r| r.layout_engine_solve_time_us).unwrap_or(0);
                    let top_solves = top.map(|r| r.layout_engine_solves).unwrap_or(0);
                    let top_prepaint = top.map(|r| r.prepaint_time_us).unwrap_or(0);
                    let top_paint = top.map(|r| r.paint_time_us).unwrap_or(0);
                    let top_dispatch = top.map(|r| r.dispatch_time_us).unwrap_or(0);
                    let top_hit_test = top.map(|r| r.hit_test_time_us).unwrap_or(0);
                    let top_dispatch_events = top.map(|r| r.dispatch_events).unwrap_or(0);
                    let top_hit_test_queries = top.map(|r| r.hit_test_queries).unwrap_or(0);
                    let top_hit_test_bounds_tree_queries =
                        top.map(|r| r.hit_test_bounds_tree_queries).unwrap_or(0);
                    let top_hit_test_bounds_tree_disabled =
                        top.map(|r| r.hit_test_bounds_tree_disabled).unwrap_or(0);
                    let top_hit_test_bounds_tree_misses =
                        top.map(|r| r.hit_test_bounds_tree_misses).unwrap_or(0);
                    let top_hit_test_bounds_tree_hits =
                        top.map(|r| r.hit_test_bounds_tree_hits).unwrap_or(0);
                    let top_hit_test_bounds_tree_candidate_rejected = top
                        .map(|r| r.hit_test_bounds_tree_candidate_rejected)
                        .unwrap_or(0);
                    let top_frame_arena_capacity_estimate_bytes = top
                        .map(|r| r.frame_arena_capacity_estimate_bytes)
                        .unwrap_or(0);
                    let top_frame_arena_grow_events =
                        top.map(|r| r.frame_arena_grow_events).unwrap_or(0);
                    let top_element_children_vec_pool_reuses =
                        top.map(|r| r.element_children_vec_pool_reuses).unwrap_or(0);
                    let top_element_children_vec_pool_misses =
                        top.map(|r| r.element_children_vec_pool_misses).unwrap_or(0);
                    let top_frame = top.map(|r| r.frame_id).unwrap_or(0);
                    let top_tick = top.map(|r| r.tick_id).unwrap_or(0);
                    let top_view_cache_contained_relayouts =
                        top.map(|r| r.view_cache_contained_relayouts).unwrap_or(0);
                    let top_view_cache_roots_total =
                        top.map(|r| r.view_cache_roots_total).unwrap_or(0);
                    let top_view_cache_roots_reused =
                        top.map(|r| r.view_cache_roots_reused).unwrap_or(0);
                    let top_view_cache_roots_first_mount =
                        top.map(|r| r.view_cache_roots_first_mount).unwrap_or(0);
                    let top_view_cache_roots_node_recreated =
                        top.map(|r| r.view_cache_roots_node_recreated).unwrap_or(0);
                    let top_view_cache_roots_cache_key_mismatch = top
                        .map(|r| r.view_cache_roots_cache_key_mismatch)
                        .unwrap_or(0);
                    let top_view_cache_roots_not_marked_reuse_root = top
                        .map(|r| r.view_cache_roots_not_marked_reuse_root)
                        .unwrap_or(0);
                    let top_view_cache_roots_needs_rerender =
                        top.map(|r| r.view_cache_roots_needs_rerender).unwrap_or(0);
                    let top_view_cache_roots_layout_invalidated = top
                        .map(|r| r.view_cache_roots_layout_invalidated)
                        .unwrap_or(0);
                    let top_view_cache_roots_manual =
                        top.map(|r| r.view_cache_roots_manual).unwrap_or(0);
                    let top_cache_roots_contained_relayout =
                        top.map(|r| r.cache_roots_contained_relayout).unwrap_or(0);
                    let top_set_children_barrier_writes =
                        top.map(|r| r.set_children_barrier_writes).unwrap_or(0);
                    let top_barrier_relayouts_scheduled =
                        top.map(|r| r.barrier_relayouts_scheduled).unwrap_or(0);
                    let top_barrier_relayouts_performed =
                        top.map(|r| r.barrier_relayouts_performed).unwrap_or(0);
                    let top_virtual_list_visible_range_checks = top
                        .map(|r| r.virtual_list_visible_range_checks)
                        .unwrap_or(0);
                    let top_virtual_list_visible_range_refreshes = top
                        .map(|r| r.virtual_list_visible_range_refreshes)
                        .unwrap_or(0);
                    let top_renderer_tick_id = top.map(|r| r.renderer_tick_id).unwrap_or(0);
                    let top_renderer_frame_id = top.map(|r| r.renderer_frame_id).unwrap_or(0);
                    let top_renderer_encode_scene_us =
                        top.map(|r| r.renderer_encode_scene_us).unwrap_or(0);
                    let top_renderer_prepare_text_us =
                        top.map(|r| r.renderer_prepare_text_us).unwrap_or(0);
                    let top_renderer_prepare_svg_us =
                        top.map(|r| r.renderer_prepare_svg_us).unwrap_or(0);
                    let top_renderer_draw_calls = top.map(|r| r.renderer_draw_calls).unwrap_or(0);
                    let top_renderer_pipeline_switches =
                        top.map(|r| r.renderer_pipeline_switches).unwrap_or(0);
                    let top_renderer_bind_group_switches =
                        top.map(|r| r.renderer_bind_group_switches).unwrap_or(0);
                    let top_renderer_scissor_sets =
                        top.map(|r| r.renderer_scissor_sets).unwrap_or(0);
                    let top_renderer_scene_encoding_cache_misses = top
                        .map(|r| r.renderer_scene_encoding_cache_misses)
                        .unwrap_or(0);
                    let top_renderer_text_atlas_upload_bytes =
                        top.map(|r| r.renderer_text_atlas_upload_bytes).unwrap_or(0);
                    let top_renderer_text_atlas_evicted_pages = top
                        .map(|r| r.renderer_text_atlas_evicted_pages)
                        .unwrap_or(0);
                    let top_renderer_svg_upload_bytes =
                        top.map(|r| r.renderer_svg_upload_bytes).unwrap_or(0);
                    let top_renderer_image_upload_bytes =
                        top.map(|r| r.renderer_image_upload_bytes).unwrap_or(0);
                    let top_renderer_svg_raster_cache_misses =
                        top.map(|r| r.renderer_svg_raster_cache_misses).unwrap_or(0);
                    let top_renderer_svg_raster_budget_evictions = top
                        .map(|r| r.renderer_svg_raster_budget_evictions)
                        .unwrap_or(0);
                    let top_renderer_svg_raster_budget_bytes =
                        top.map(|r| r.renderer_svg_raster_budget_bytes).unwrap_or(0);
                    let top_renderer_svg_rasters_live =
                        top.map(|r| r.renderer_svg_rasters_live).unwrap_or(0);
                    let top_renderer_svg_standalone_bytes_live = top
                        .map(|r| r.renderer_svg_standalone_bytes_live)
                        .unwrap_or(0);
                    let top_renderer_svg_mask_atlas_pages_live = top
                        .map(|r| r.renderer_svg_mask_atlas_pages_live)
                        .unwrap_or(0);
                    let top_renderer_svg_mask_atlas_bytes_live = top
                        .map(|r| r.renderer_svg_mask_atlas_bytes_live)
                        .unwrap_or(0);
                    let top_renderer_svg_mask_atlas_used_px =
                        top.map(|r| r.renderer_svg_mask_atlas_used_px).unwrap_or(0);
                    let top_renderer_svg_mask_atlas_capacity_px = top
                        .map(|r| r.renderer_svg_mask_atlas_capacity_px)
                        .unwrap_or(0);
                    let top_renderer_svg_raster_cache_hits =
                        top.map(|r| r.renderer_svg_raster_cache_hits).unwrap_or(0);
                    let top_renderer_svg_mask_atlas_page_evictions = top
                        .map(|r| r.renderer_svg_mask_atlas_page_evictions)
                        .unwrap_or(0);
                    let top_renderer_svg_mask_atlas_entries_evicted = top
                        .map(|r| r.renderer_svg_mask_atlas_entries_evicted)
                        .unwrap_or(0);
                    let top_renderer_intermediate_budget_bytes = top
                        .map(|r| r.renderer_intermediate_budget_bytes)
                        .unwrap_or(0);
                    let top_renderer_intermediate_in_use_bytes = top
                        .map(|r| r.renderer_intermediate_in_use_bytes)
                        .unwrap_or(0);
                    let top_renderer_intermediate_peak_in_use_bytes = top
                        .map(|r| r.renderer_intermediate_peak_in_use_bytes)
                        .unwrap_or(0);
                    let top_renderer_intermediate_release_targets = top
                        .map(|r| r.renderer_intermediate_release_targets)
                        .unwrap_or(0);
                    let top_renderer_intermediate_pool_allocations = top
                        .map(|r| r.renderer_intermediate_pool_allocations)
                        .unwrap_or(0);
                    let top_renderer_intermediate_pool_reuses = top
                        .map(|r| r.renderer_intermediate_pool_reuses)
                        .unwrap_or(0);
                    let top_renderer_intermediate_pool_releases = top
                        .map(|r| r.renderer_intermediate_pool_releases)
                        .unwrap_or(0);
                    let top_renderer_intermediate_pool_evictions = top
                        .map(|r| r.renderer_intermediate_pool_evictions)
                        .unwrap_or(0);
                    let top_renderer_intermediate_pool_free_bytes = top
                        .map(|r| r.renderer_intermediate_pool_free_bytes)
                        .unwrap_or(0);
                    let top_renderer_intermediate_pool_free_textures = top
                        .map(|r| r.renderer_intermediate_pool_free_textures)
                        .unwrap_or(0);

                    runs_total.push(top_total);
                    runs_layout.push(top_layout);
                    runs_solve.push(top_solve);
                    runs_prepaint.push(top_prepaint);
                    runs_paint.push(top_paint);
                    runs_dispatch.push(top_dispatch);
                    runs_hit_test.push(top_hit_test);
                    let pointer_move_frames_present = report.pointer_move_frames_present;
                    let pointer_move_frames_considered =
                        report.pointer_move_frames_considered as u64;
                    let pointer_move_max_dispatch_time_us =
                        report.pointer_move_max_dispatch_time_us;
                    let pointer_move_max_hit_test_time_us =
                        report.pointer_move_max_hit_test_time_us;
                    let pointer_move_snapshots_with_global_changes =
                        report.pointer_move_snapshots_with_global_changes as u64;
                    let (
                        run_paint_cache_hit_test_only_replay_allowed_max,
                        run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    ) = bundle_paint_cache_hit_test_only_replay_maxes(
                        &bundle_path,
                        report_warmup_frames,
                    )?;
                    runs_pointer_move_dispatch.push(pointer_move_max_dispatch_time_us);
                    runs_pointer_move_hit_test.push(pointer_move_max_hit_test_time_us);
                    runs_pointer_move_global_changes
                        .push(pointer_move_snapshots_with_global_changes);
                    runs_paint_cache_hit_test_only_replay_allowed_max
                        .push(run_paint_cache_hit_test_only_replay_allowed_max);
                    runs_paint_cache_hit_test_only_replay_rejected_key_mismatch_max
                        .push(run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max);
                    runs_json.push(serde_json::json!({
                        "run_index": run_index,
                        "top_total_time_us": top_total,
                        "top_layout_time_us": top_layout,
                        "top_layout_engine_solve_time_us": top_solve,
                        "top_layout_engine_solves": top_solves,
                        "top_prepaint_time_us": top_prepaint,
                        "top_paint_time_us": top_paint,
                        "top_dispatch_time_us": top_dispatch,
                        "top_hit_test_time_us": top_hit_test,
                        "top_dispatch_events": top_dispatch_events,
                        "top_hit_test_queries": top_hit_test_queries,
                        "pointer_move_frames_present": pointer_move_frames_present,
                        "pointer_move_frames_considered": pointer_move_frames_considered,
                        "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                        "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                        "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                        "run_paint_cache_hit_test_only_replay_allowed_max": run_paint_cache_hit_test_only_replay_allowed_max,
                        "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        "top_hit_test_bounds_tree_queries": top_hit_test_bounds_tree_queries,
                        "top_hit_test_bounds_tree_disabled": top_hit_test_bounds_tree_disabled,
                        "top_hit_test_bounds_tree_misses": top_hit_test_bounds_tree_misses,
                        "top_hit_test_bounds_tree_hits": top_hit_test_bounds_tree_hits,
                        "top_hit_test_bounds_tree_candidate_rejected": top_hit_test_bounds_tree_candidate_rejected,
                        "top_frame_arena_capacity_estimate_bytes": top_frame_arena_capacity_estimate_bytes,
                        "top_frame_arena_grow_events": top_frame_arena_grow_events,
                        "top_element_children_vec_pool_reuses": top_element_children_vec_pool_reuses,
                        "top_element_children_vec_pool_misses": top_element_children_vec_pool_misses,
                        "top_tick_id": top_tick,
                        "top_frame_id": top_frame,
                        "top_view_cache_contained_relayouts": top_view_cache_contained_relayouts,
                        "top_view_cache_roots_total": top_view_cache_roots_total,
                        "top_view_cache_roots_reused": top_view_cache_roots_reused,
                        "top_view_cache_roots_first_mount": top_view_cache_roots_first_mount,
                        "top_view_cache_roots_node_recreated": top_view_cache_roots_node_recreated,
                        "top_view_cache_roots_cache_key_mismatch": top_view_cache_roots_cache_key_mismatch,
                        "top_view_cache_roots_not_marked_reuse_root": top_view_cache_roots_not_marked_reuse_root,
                        "top_view_cache_roots_needs_rerender": top_view_cache_roots_needs_rerender,
                        "top_view_cache_roots_layout_invalidated": top_view_cache_roots_layout_invalidated,
                        "top_view_cache_roots_manual": top_view_cache_roots_manual,
                        "top_cache_roots_contained_relayout": top_cache_roots_contained_relayout,
                        "top_set_children_barrier_writes": top_set_children_barrier_writes,
                        "top_barrier_relayouts_scheduled": top_barrier_relayouts_scheduled,
                        "top_barrier_relayouts_performed": top_barrier_relayouts_performed,
                        "top_virtual_list_visible_range_checks": top_virtual_list_visible_range_checks,
                        "top_virtual_list_visible_range_refreshes": top_virtual_list_visible_range_refreshes,
                        "top_renderer_tick_id": top_renderer_tick_id,
                        "top_renderer_frame_id": top_renderer_frame_id,
                        "top_renderer_encode_scene_us": top_renderer_encode_scene_us,
                        "top_renderer_prepare_text_us": top_renderer_prepare_text_us,
                        "top_renderer_prepare_svg_us": top_renderer_prepare_svg_us,
                        "top_renderer_draw_calls": top_renderer_draw_calls,
                        "top_renderer_pipeline_switches": top_renderer_pipeline_switches,
                        "top_renderer_bind_group_switches": top_renderer_bind_group_switches,
                        "top_renderer_scissor_sets": top_renderer_scissor_sets,
                        "top_renderer_scene_encoding_cache_misses": top_renderer_scene_encoding_cache_misses,
                        "top_renderer_text_atlas_upload_bytes": top_renderer_text_atlas_upload_bytes,
                        "top_renderer_text_atlas_evicted_pages": top_renderer_text_atlas_evicted_pages,
                        "top_renderer_svg_upload_bytes": top_renderer_svg_upload_bytes,
                        "top_renderer_image_upload_bytes": top_renderer_image_upload_bytes,
                        "top_renderer_svg_raster_cache_misses": top_renderer_svg_raster_cache_misses,
                        "top_renderer_svg_raster_budget_evictions": top_renderer_svg_raster_budget_evictions,
                        "top_renderer_svg_raster_budget_bytes": top_renderer_svg_raster_budget_bytes,
                        "top_renderer_svg_rasters_live": top_renderer_svg_rasters_live,
                        "top_renderer_svg_standalone_bytes_live": top_renderer_svg_standalone_bytes_live,
                        "top_renderer_svg_mask_atlas_pages_live": top_renderer_svg_mask_atlas_pages_live,
                        "top_renderer_svg_mask_atlas_bytes_live": top_renderer_svg_mask_atlas_bytes_live,
                        "top_renderer_svg_mask_atlas_used_px": top_renderer_svg_mask_atlas_used_px,
	                        "top_renderer_svg_mask_atlas_capacity_px": top_renderer_svg_mask_atlas_capacity_px,
	                        "top_renderer_svg_raster_cache_hits": top_renderer_svg_raster_cache_hits,
	                        "top_renderer_svg_mask_atlas_page_evictions": top_renderer_svg_mask_atlas_page_evictions,
	                        "top_renderer_svg_mask_atlas_entries_evicted": top_renderer_svg_mask_atlas_entries_evicted,
	                        "top_renderer_intermediate_budget_bytes": top_renderer_intermediate_budget_bytes,
	                        "top_renderer_intermediate_in_use_bytes": top_renderer_intermediate_in_use_bytes,
	                        "top_renderer_intermediate_peak_in_use_bytes": top_renderer_intermediate_peak_in_use_bytes,
	                        "top_renderer_intermediate_release_targets": top_renderer_intermediate_release_targets,
	                        "top_renderer_intermediate_pool_allocations": top_renderer_intermediate_pool_allocations,
	                        "top_renderer_intermediate_pool_reuses": top_renderer_intermediate_pool_reuses,
	                        "top_renderer_intermediate_pool_releases": top_renderer_intermediate_pool_releases,
	                        "top_renderer_intermediate_pool_evictions": top_renderer_intermediate_pool_evictions,
	                        "top_renderer_intermediate_pool_free_bytes": top_renderer_intermediate_pool_free_bytes,
	                        "top_renderer_intermediate_pool_free_textures": top_renderer_intermediate_pool_free_textures,
	                        "bundle": bundle_path.display().to_string(),
	                    }));

                    match &script_worst {
                        Some((prev_us, _)) if *prev_us >= top_total => {}
                        _ => script_worst = Some((top_total, bundle_path.clone())),
                    }

                    match &overall_worst {
                        Some((prev_us, _, _)) if *prev_us >= top_total => {}
                        _ => overall_worst = Some((top_total, src.clone(), bundle_path.clone())),
                    }

                    if !reuse_process {
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                    }
                }

                if runs_total.len() == repeat {
                    if stats_json {
                        let mut top_frame_arena_capacity_estimate_bytes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_frame_arena_grow_events: Vec<u64> = Vec::with_capacity(repeat);
                        let mut top_element_children_vec_pool_reuses: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_element_children_vec_pool_misses: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_view_cache_contained_relayouts: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_view_cache_roots_total: Vec<u64> = Vec::with_capacity(repeat);
                        let mut top_view_cache_roots_reused: Vec<u64> = Vec::with_capacity(repeat);
                        let mut top_view_cache_roots_cache_key_mismatch: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_view_cache_roots_needs_rerender: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_view_cache_roots_layout_invalidated: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_cache_roots_contained_relayout: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_set_children_barrier_writes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_barrier_relayouts_scheduled: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_barrier_relayouts_performed: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_virtual_list_visible_range_checks: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_virtual_list_visible_range_refreshes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_encode_scene_us: Vec<u64> = Vec::with_capacity(repeat);
                        let mut top_renderer_prepare_text_us: Vec<u64> = Vec::with_capacity(repeat);
                        let mut top_renderer_draw_calls: Vec<u64> = Vec::with_capacity(repeat);
                        let mut top_renderer_pipeline_switches: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_bind_group_switches: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_scene_encoding_cache_misses: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_text_atlas_upload_bytes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_text_atlas_evicted_pages: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_svg_upload_bytes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_image_upload_bytes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_svg_raster_cache_misses: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_svg_raster_budget_evictions: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_svg_rasters_live: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_svg_mask_atlas_pages_live: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_svg_mask_atlas_used_px: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_budget_bytes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_in_use_bytes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_peak_in_use_bytes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_release_targets: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_pool_allocations: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_pool_reuses: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_pool_releases: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_pool_evictions: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_pool_free_bytes: Vec<u64> =
                            Vec::with_capacity(repeat);
                        let mut top_renderer_intermediate_pool_free_textures: Vec<u64> =
                            Vec::with_capacity(repeat);
                        for run in &runs_json {
                            top_frame_arena_capacity_estimate_bytes.push(
                                run.get("top_frame_arena_capacity_estimate_bytes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_frame_arena_grow_events.push(
                                run.get("top_frame_arena_grow_events")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_element_children_vec_pool_reuses.push(
                                run.get("top_element_children_vec_pool_reuses")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_element_children_vec_pool_misses.push(
                                run.get("top_element_children_vec_pool_misses")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_view_cache_contained_relayouts.push(
                                run.get("top_view_cache_contained_relayouts")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_view_cache_roots_total.push(
                                run.get("top_view_cache_roots_total")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_view_cache_roots_reused.push(
                                run.get("top_view_cache_roots_reused")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_view_cache_roots_cache_key_mismatch.push(
                                run.get("top_view_cache_roots_cache_key_mismatch")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_view_cache_roots_needs_rerender.push(
                                run.get("top_view_cache_roots_needs_rerender")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_view_cache_roots_layout_invalidated.push(
                                run.get("top_view_cache_roots_layout_invalidated")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_cache_roots_contained_relayout.push(
                                run.get("top_cache_roots_contained_relayout")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_set_children_barrier_writes.push(
                                run.get("top_set_children_barrier_writes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_barrier_relayouts_scheduled.push(
                                run.get("top_barrier_relayouts_scheduled")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_barrier_relayouts_performed.push(
                                run.get("top_barrier_relayouts_performed")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_virtual_list_visible_range_checks.push(
                                run.get("top_virtual_list_visible_range_checks")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_virtual_list_visible_range_refreshes.push(
                                run.get("top_virtual_list_visible_range_refreshes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_encode_scene_us.push(
                                run.get("top_renderer_encode_scene_us")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_prepare_text_us.push(
                                run.get("top_renderer_prepare_text_us")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_draw_calls.push(
                                run.get("top_renderer_draw_calls")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_pipeline_switches.push(
                                run.get("top_renderer_pipeline_switches")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_bind_group_switches.push(
                                run.get("top_renderer_bind_group_switches")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_scene_encoding_cache_misses.push(
                                run.get("top_renderer_scene_encoding_cache_misses")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_text_atlas_upload_bytes.push(
                                run.get("top_renderer_text_atlas_upload_bytes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_text_atlas_evicted_pages.push(
                                run.get("top_renderer_text_atlas_evicted_pages")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_svg_upload_bytes.push(
                                run.get("top_renderer_svg_upload_bytes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_image_upload_bytes.push(
                                run.get("top_renderer_image_upload_bytes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_svg_raster_cache_misses.push(
                                run.get("top_renderer_svg_raster_cache_misses")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_svg_raster_budget_evictions.push(
                                run.get("top_renderer_svg_raster_budget_evictions")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_svg_rasters_live.push(
                                run.get("top_renderer_svg_rasters_live")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_svg_mask_atlas_pages_live.push(
                                run.get("top_renderer_svg_mask_atlas_pages_live")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_svg_mask_atlas_used_px.push(
                                run.get("top_renderer_svg_mask_atlas_used_px")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_budget_bytes.push(
                                run.get("top_renderer_intermediate_budget_bytes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_in_use_bytes.push(
                                run.get("top_renderer_intermediate_in_use_bytes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_peak_in_use_bytes.push(
                                run.get("top_renderer_intermediate_peak_in_use_bytes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_release_targets.push(
                                run.get("top_renderer_intermediate_release_targets")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_pool_allocations.push(
                                run.get("top_renderer_intermediate_pool_allocations")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_pool_reuses.push(
                                run.get("top_renderer_intermediate_pool_reuses")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_pool_releases.push(
                                run.get("top_renderer_intermediate_pool_releases")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_pool_evictions.push(
                                run.get("top_renderer_intermediate_pool_evictions")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_pool_free_bytes.push(
                                run.get("top_renderer_intermediate_pool_free_bytes")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                            top_renderer_intermediate_pool_free_textures.push(
                                run.get("top_renderer_intermediate_pool_free_textures")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                            );
                        }
                        perf_json_rows.push(serde_json::json!({
	                        "script": src.display().to_string(),
	                        "sort": sort.as_str(),
		                        "repeat": repeat,
		                            "runs": runs_json,
		                            "stats": {
		                                "total_time_us": summarize_times_us(&runs_total),
		                                "layout_time_us": summarize_times_us(&runs_layout),
		                                "layout_engine_solve_time_us": summarize_times_us(&runs_solve),
		                                "prepaint_time_us": summarize_times_us(&runs_prepaint),
		                                "paint_time_us": summarize_times_us(&runs_paint),
		                                "dispatch_time_us": summarize_times_us(&runs_dispatch),
		                                "hit_test_time_us": summarize_times_us(&runs_hit_test),
		                                "pointer_move_max_dispatch_time_us": summarize_times_us(&runs_pointer_move_dispatch),
		                                "pointer_move_max_hit_test_time_us": summarize_times_us(&runs_pointer_move_hit_test),
		                                "pointer_move_snapshots_with_global_changes": summarize_times_us(&runs_pointer_move_global_changes),
		                                "top_frame_arena_capacity_estimate_bytes": summarize_times_us(&top_frame_arena_capacity_estimate_bytes),
		                                "top_frame_arena_grow_events": summarize_times_us(&top_frame_arena_grow_events),
		                                "top_element_children_vec_pool_reuses": summarize_times_us(&top_element_children_vec_pool_reuses),
		                                "top_element_children_vec_pool_misses": summarize_times_us(&top_element_children_vec_pool_misses),
	                                "top_view_cache_contained_relayouts": summarize_times_us(&top_view_cache_contained_relayouts),
	                                "top_view_cache_roots_total": summarize_times_us(&top_view_cache_roots_total),
	                                "top_view_cache_roots_reused": summarize_times_us(&top_view_cache_roots_reused),
	                                "top_view_cache_roots_cache_key_mismatch": summarize_times_us(&top_view_cache_roots_cache_key_mismatch),
	                                "top_view_cache_roots_needs_rerender": summarize_times_us(&top_view_cache_roots_needs_rerender),
	                                "top_view_cache_roots_layout_invalidated": summarize_times_us(&top_view_cache_roots_layout_invalidated),
	                                "top_cache_roots_contained_relayout": summarize_times_us(&top_cache_roots_contained_relayout),
	                                "top_set_children_barrier_writes": summarize_times_us(&top_set_children_barrier_writes),
	                                "top_barrier_relayouts_scheduled": summarize_times_us(&top_barrier_relayouts_scheduled),
	                                "top_barrier_relayouts_performed": summarize_times_us(&top_barrier_relayouts_performed),
	                                "top_virtual_list_visible_range_checks": summarize_times_us(&top_virtual_list_visible_range_checks),
	                                "top_virtual_list_visible_range_refreshes": summarize_times_us(&top_virtual_list_visible_range_refreshes),
	                                "top_renderer_encode_scene_us": summarize_times_us(&top_renderer_encode_scene_us),
	                                "top_renderer_prepare_text_us": summarize_times_us(&top_renderer_prepare_text_us),
	                                "top_renderer_draw_calls": summarize_times_us(&top_renderer_draw_calls),
	                                "top_renderer_pipeline_switches": summarize_times_us(&top_renderer_pipeline_switches),
	                                "top_renderer_bind_group_switches": summarize_times_us(&top_renderer_bind_group_switches),
	                                "top_renderer_scene_encoding_cache_misses": summarize_times_us(&top_renderer_scene_encoding_cache_misses),
	                                "top_renderer_text_atlas_upload_bytes": summarize_times_us(&top_renderer_text_atlas_upload_bytes),
	                                "top_renderer_text_atlas_evicted_pages": summarize_times_us(&top_renderer_text_atlas_evicted_pages),
	                                "top_renderer_svg_upload_bytes": summarize_times_us(&top_renderer_svg_upload_bytes),
	                                "top_renderer_image_upload_bytes": summarize_times_us(&top_renderer_image_upload_bytes),
	                                "top_renderer_svg_raster_cache_misses": summarize_times_us(&top_renderer_svg_raster_cache_misses),
	                                "top_renderer_svg_raster_budget_evictions": summarize_times_us(&top_renderer_svg_raster_budget_evictions),
	                                "top_renderer_svg_rasters_live": summarize_times_us(&top_renderer_svg_rasters_live),
	                                "top_renderer_svg_mask_atlas_pages_live": summarize_times_us(&top_renderer_svg_mask_atlas_pages_live),
	                                "top_renderer_svg_mask_atlas_used_px": summarize_times_us(&top_renderer_svg_mask_atlas_used_px),
	                                "top_renderer_intermediate_budget_bytes": summarize_times_us(&top_renderer_intermediate_budget_bytes),
	                                "top_renderer_intermediate_in_use_bytes": summarize_times_us(&top_renderer_intermediate_in_use_bytes),
	                                "top_renderer_intermediate_peak_in_use_bytes": summarize_times_us(&top_renderer_intermediate_peak_in_use_bytes),
	                                "top_renderer_intermediate_release_targets": summarize_times_us(&top_renderer_intermediate_release_targets),
	                                "top_renderer_intermediate_pool_allocations": summarize_times_us(&top_renderer_intermediate_pool_allocations),
	                                "top_renderer_intermediate_pool_reuses": summarize_times_us(&top_renderer_intermediate_pool_reuses),
	                                "top_renderer_intermediate_pool_releases": summarize_times_us(&top_renderer_intermediate_pool_releases),
	                                "top_renderer_intermediate_pool_evictions": summarize_times_us(&top_renderer_intermediate_pool_evictions),
	                                "top_renderer_intermediate_pool_free_bytes": summarize_times_us(&top_renderer_intermediate_pool_free_bytes),
	                                "top_renderer_intermediate_pool_free_textures": summarize_times_us(&top_renderer_intermediate_pool_free_textures),
	                            },
	                            "worst_run": script_worst.as_ref().map(|(us, bundle)| serde_json::json!({
	                                "top_total_time_us": us,
	                                "bundle": bundle.display().to_string(),
	                            })),
	                        }));
                    } else {
                        let total = summarize_times_us(&runs_total);
                        let layout = summarize_times_us(&runs_layout);
                        let solve = summarize_times_us(&runs_solve);
                        let prepaint = summarize_times_us(&runs_prepaint);
                        let paint = summarize_times_us(&runs_paint);
                        let dispatch = summarize_times_us(&runs_dispatch);
                        let hit_test = summarize_times_us(&runs_hit_test);
                        println!(
                            "PERF {} sort={} repeat={} p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{} p95.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{} max.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{}",
                            src.display(),
                            sort.as_str(),
                            repeat,
                            total.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            layout.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            solve.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            prepaint.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            paint.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            dispatch.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            hit_test.get("p50").and_then(|v| v.as_u64()).unwrap_or(0),
                            total.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            layout.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            solve.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            prepaint.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            paint.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            dispatch.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            hit_test.get("p95").and_then(|v| v.as_u64()).unwrap_or(0),
                            total.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            layout.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            solve.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            prepaint.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            paint.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            dispatch.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                            hit_test.get("max").and_then(|v| v.as_u64()).unwrap_or(0),
                        );
                    }

                    let max_total = *runs_total.iter().max().unwrap_or(&0);
                    let max_layout = *runs_layout.iter().max().unwrap_or(&0);
                    let max_solve = *runs_solve.iter().max().unwrap_or(&0);

                    let mut sorted_total = runs_total.clone();
                    sorted_total.sort_unstable();
                    let p90_total = percentile_nearest_rank_sorted(&sorted_total, 0.90);
                    let p95_total = percentile_nearest_rank_sorted(&sorted_total, 0.95);

                    let mut sorted_layout = runs_layout.clone();
                    sorted_layout.sort_unstable();
                    let p90_layout = percentile_nearest_rank_sorted(&sorted_layout, 0.90);
                    let p95_layout = percentile_nearest_rank_sorted(&sorted_layout, 0.95);

                    let mut sorted_solve = runs_solve.clone();
                    sorted_solve.sort_unstable();
                    let p90_solve = percentile_nearest_rank_sorted(&sorted_solve, 0.90);
                    let p95_solve = percentile_nearest_rank_sorted(&sorted_solve, 0.95);
                    let max_pointer_move_dispatch =
                        *runs_pointer_move_dispatch.iter().max().unwrap_or(&0);
                    let max_pointer_move_hit_test =
                        *runs_pointer_move_hit_test.iter().max().unwrap_or(&0);
                    let max_pointer_move_global_changes =
                        *runs_pointer_move_global_changes.iter().max().unwrap_or(&0);
                    let max_run_paint_cache_hit_test_only_replay_allowed_max =
                        *runs_paint_cache_hit_test_only_replay_allowed_max
                            .iter()
                            .max()
                            .unwrap_or(&0);
                    let max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max =
                        *runs_paint_cache_hit_test_only_replay_rejected_key_mismatch_max
                            .iter()
                            .max()
                            .unwrap_or(&0);
                    let pointer_move_frames_present = runs_json.iter().any(|run| {
                        run.get("pointer_move_frames_present")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    });
                    let script_key = normalize_repo_relative_path(&workspace_root, &src);

                    if perf_baseline_out.is_some() {
                        let policy = seed_policy
                            .as_ref()
                            .ok_or_else(|| "internal error: missing seed policy".to_string())?;

                        let seed_total =
                            policy.seed_for(&script_key, PerfSeedMetric::TopTotalTimeUs);
                        let seed_layout =
                            policy.seed_for(&script_key, PerfSeedMetric::TopLayoutTimeUs);
                        let seed_solve = policy
                            .seed_for(&script_key, PerfSeedMetric::TopLayoutEngineSolveTimeUs);

                        let seed_total_value = match seed_total {
                            PerfBaselineSeed::Max => max_total,
                            PerfBaselineSeed::P90 => p90_total,
                            PerfBaselineSeed::P95 => p95_total,
                        };
                        let seed_layout_value = match seed_layout {
                            PerfBaselineSeed::Max => max_layout,
                            PerfBaselineSeed::P90 => p90_layout,
                            PerfBaselineSeed::P95 => p95_layout,
                        };
                        let seed_solve_value = match seed_solve {
                            PerfBaselineSeed::Max => max_solve,
                            PerfBaselineSeed::P90 => p90_solve,
                            PerfBaselineSeed::P95 => p95_solve,
                        };

                        let thr_total = apply_perf_baseline_headroom(
                            seed_total_value,
                            perf_baseline_headroom_pct,
                        );
                        let thr_layout = apply_perf_baseline_headroom(
                            seed_layout_value,
                            perf_baseline_headroom_pct,
                        );
                        let thr_solve = apply_perf_baseline_headroom(
                            seed_solve_value,
                            perf_baseline_headroom_pct,
                        );
                        let thr_pointer_move_dispatch = apply_perf_baseline_headroom(
                            max_pointer_move_dispatch,
                            perf_baseline_headroom_pct,
                        );
                        let thr_pointer_move_hit_test = apply_perf_baseline_headroom(
                            max_pointer_move_hit_test,
                            perf_baseline_headroom_pct,
                        );
                        let thr_pointer_move_global_changes = apply_perf_baseline_headroom(
                            max_pointer_move_global_changes,
                            perf_baseline_headroom_pct,
                        );
                        let thr_min_run_paint_cache_hit_test_only_replay_allowed_max =
                            apply_perf_baseline_floor(
                                max_run_paint_cache_hit_test_only_replay_allowed_max,
                                perf_baseline_headroom_pct,
                            );
                        let thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max =
                            apply_perf_baseline_headroom(
                                max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                                perf_baseline_headroom_pct,
                            );

                        perf_baseline_rows.push(serde_json::json!({
                            "script": script_key.clone(),
                            "measured_max": {
                                "top_total_time_us": max_total,
                                "top_layout_time_us": max_layout,
                                "top_layout_engine_solve_time_us": max_solve,
                                "pointer_move_max_dispatch_time_us": max_pointer_move_dispatch,
                                "pointer_move_max_hit_test_time_us": max_pointer_move_hit_test,
                                "pointer_move_snapshots_with_global_changes": max_pointer_move_global_changes,
                                "run_paint_cache_hit_test_only_replay_allowed_max": max_run_paint_cache_hit_test_only_replay_allowed_max,
                                "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            },
                            "measured_p90": {
                                "top_total_time_us": p90_total,
                                "top_layout_time_us": p90_layout,
                                "top_layout_engine_solve_time_us": p90_solve,
                            },
                            "measured_p95": {
                                "top_total_time_us": p95_total,
                                "top_layout_time_us": p95_layout,
                                "top_layout_engine_solve_time_us": p95_solve,
                            },
                            "threshold_seed": {
                                "top_total_time_us": seed_total_value,
                                "top_layout_time_us": seed_layout_value,
                                "top_layout_engine_solve_time_us": seed_solve_value,
                            },
                            "threshold_seed_source": {
                                "top_total_time_us": seed_total.as_str(),
                                "top_layout_time_us": seed_layout.as_str(),
                                "top_layout_engine_solve_time_us": seed_solve.as_str(),
                            },
                            "thresholds": {
                                "max_top_total_us": thr_total,
                                "max_top_layout_us": thr_layout,
                                "max_top_solve_us": thr_solve,
                                "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
                                "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
                                "max_pointer_move_global_changes": thr_pointer_move_global_changes,
                                "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_min_run_paint_cache_hit_test_only_replay_allowed_max,
                                "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            },
                        }));
                    }

                    if wants_perf_thresholds {
                        let baseline_thresholds = perf_baseline
                            .as_ref()
                            .and_then(|b| b.thresholds_by_script.get(&script_key).copied())
                            .unwrap_or_default();
                        let (thr_total, src_total) = resolve_threshold(
                            cli_thresholds.max_top_total_us,
                            baseline_thresholds.max_top_total_us,
                        );
                        let (thr_layout, src_layout) = resolve_threshold(
                            cli_thresholds.max_top_layout_us,
                            baseline_thresholds.max_top_layout_us,
                        );
                        let (thr_solve, src_solve) = resolve_threshold(
                            cli_thresholds.max_top_solve_us,
                            baseline_thresholds.max_top_solve_us,
                        );
                        let (thr_pointer_move_dispatch, src_pointer_move_dispatch) =
                            resolve_threshold(
                                cli_thresholds.max_pointer_move_dispatch_us,
                                baseline_thresholds.max_pointer_move_dispatch_us,
                            );
                        let (thr_pointer_move_hit_test, src_pointer_move_hit_test) =
                            resolve_threshold(
                                cli_thresholds.max_pointer_move_hit_test_us,
                                baseline_thresholds.max_pointer_move_hit_test_us,
                            );
                        let (thr_pointer_move_global_changes, src_pointer_move_global_changes) =
                            resolve_threshold(
                                cli_thresholds.max_pointer_move_global_changes,
                                baseline_thresholds.max_pointer_move_global_changes,
                            );
                        let (
                            thr_paint_cache_hit_test_only_replay_allowed_max,
                            src_paint_cache_hit_test_only_replay_allowed_max,
                        ) = resolve_threshold(
                            cli_thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max,
                            baseline_thresholds
                                .min_run_paint_cache_hit_test_only_replay_allowed_max,
                        );
                        let (
                            thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        ) = resolve_threshold(
                            cli_thresholds
                                .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            baseline_thresholds
                                .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        );
                        let row = serde_json::json!({
                            "script": script_key.clone(),
                            "sort": sort.as_str(),
                            "repeat": repeat,
                            "runs": runs_json,
                            "max": {
                                "top_total_time_us": max_total,
                                "top_layout_time_us": max_layout,
                                "top_layout_engine_solve_time_us": max_solve,
                                "pointer_move_max_dispatch_time_us": max_pointer_move_dispatch,
                                "pointer_move_max_hit_test_time_us": max_pointer_move_hit_test,
                                "pointer_move_snapshots_with_global_changes": max_pointer_move_global_changes,
                                "run_paint_cache_hit_test_only_replay_allowed_max": max_run_paint_cache_hit_test_only_replay_allowed_max,
                                "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            },
                            "thresholds": {
                                "max_top_total_us": thr_total,
                                "max_top_layout_us": thr_layout,
                                "max_top_solve_us": thr_solve,
                                "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
                                "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
                                "max_pointer_move_global_changes": thr_pointer_move_global_changes,
                                "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_paint_cache_hit_test_only_replay_allowed_max,
                                "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            },
                            "threshold_sources": {
                                "max_top_total_us": src_total,
                                "max_top_layout_us": src_layout,
                                "max_top_solve_us": src_solve,
                                "max_pointer_move_dispatch_us": src_pointer_move_dispatch,
                                "max_pointer_move_hit_test_us": src_pointer_move_hit_test,
                                "max_pointer_move_global_changes": src_pointer_move_global_changes,
                                "min_run_paint_cache_hit_test_only_replay_allowed_max": src_paint_cache_hit_test_only_replay_allowed_max,
                                "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            },
                        });
                        perf_threshold_rows.push(row);
                        perf_threshold_failures.extend(scan_perf_threshold_failures(
                            script_key.as_str(),
                            sort,
                            cli_thresholds,
                            baseline_thresholds,
                            max_total,
                            max_layout,
                            max_solve,
                            pointer_move_frames_present,
                            max_pointer_move_dispatch,
                            max_pointer_move_hit_test,
                            max_pointer_move_global_changes,
                            max_run_paint_cache_hit_test_only_replay_allowed_max,
                            max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        ));
                    }
                }
            }

            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);

            if let Some(test_id) = check_pixels_changed_test_id.as_deref() {
                check_out_dir_for_pixels_changed(&resolved_out_dir, test_id, warmup_frames)?;
            }

            if let Some(path) = perf_baseline_out.as_ref() {
                let out_path = path;
                let policy = seed_policy
                    .as_ref()
                    .ok_or_else(|| "internal error: missing seed policy".to_string())?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": now_unix_ms(),
                    "kind": "perf_baseline",
                    "out_path": out_path.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "sort": sort.as_str(),
                    "repeat": repeat,
                    "headroom_pct": perf_baseline_headroom_pct,
                    "threshold_seed_policy": policy.threshold_seed_policy_json(),
                    "rows": perf_baseline_rows,
                });
                write_json_value(out_path, &payload)?;
                if !stats_json {
                    println!("wrote perf baseline: {}", out_path.display());
                }
            }

            let mut perf_threshold_failure: Option<(usize, PathBuf)> = None;
            if wants_perf_thresholds {
                let out_path = resolved_out_dir.join("check.perf_thresholds.json");
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": now_unix_ms(),
                    "kind": "perf_thresholds",
                    "out_dir": resolved_out_dir.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "thresholds": {
                        "max_top_total_us": cli_thresholds.max_top_total_us,
                        "max_top_layout_us": cli_thresholds.max_top_layout_us,
                        "max_top_solve_us": cli_thresholds.max_top_solve_us,
                        "max_pointer_move_dispatch_us": cli_thresholds.max_pointer_move_dispatch_us,
                        "max_pointer_move_hit_test_us": cli_thresholds.max_pointer_move_hit_test_us,
                        "max_pointer_move_global_changes": cli_thresholds.max_pointer_move_global_changes,
                        "min_run_paint_cache_hit_test_only_replay_allowed_max": cli_thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max,
                        "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": cli_thresholds.max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    },
                    "baseline": perf_baseline.as_ref().map(|b| serde_json::json!({
                        "path": b.path.display().to_string(),
                        "scripts": b.thresholds_by_script.len(),
                    })),
                    "rows": perf_threshold_rows,
                    "failures": perf_threshold_failures,
                });
                let _ = write_json_value(&out_path, &payload);
                if !perf_threshold_failures.is_empty() {
                    perf_threshold_failure =
                        Some((perf_threshold_failures.len(), out_path.clone()));
                }
            }

            if launched_by_fretboard {
                stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
            }

            if stats_json {
                let worst = overall_worst.as_ref().map(|(us, src, bundle)| {
                    serde_json::json!({
                        "script": src.display().to_string(),
                        "top_total_time_us": us,
                        "bundle": bundle.display().to_string(),
                    })
                });
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "sort": sort.as_str(),
                    "repeat": repeat,
                    "rows": perf_json_rows,
                    "worst_overall": worst,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                );
            } else if let Some((us, src, bundle)) = overall_worst {
                println!(
                    "PERF worst overall: {} us={} bundle={}",
                    src.display(),
                    us,
                    bundle.display()
                );
            }

            if let Some((failures, evidence)) = perf_threshold_failure {
                eprintln!(
                    "PERF threshold gate failed (failures={}, evidence={})",
                    failures,
                    evidence.display()
                );
                std::process::exit(1);
            }

            std::process::exit(0);
        }
        "stats" => {
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing bundle path (try: fretboard diag stats ./target/fret-diag/1234/bundle.json)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let src = resolve_path(&workspace_root, PathBuf::from(src));
            let bundle_path = resolve_bundle_json_path(&src);
            let mut report = bundle_stats_from_path(
                &bundle_path,
                stats_top,
                sort_override.unwrap_or(BundleStatsSort::Invalidation),
                BundleStatsOptions { warmup_frames },
            )?;
            if warmup_frames > 0 && report.top.is_empty() {
                report = bundle_stats_from_path(
                    &bundle_path,
                    stats_top,
                    sort_override.unwrap_or(BundleStatsSort::Invalidation),
                    BundleStatsOptions::default(),
                )?;
            }

            if stats_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report.to_json())
                        .unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                report.print_human(&bundle_path);
            }
            if let Some(test_id) = check_stale_paint_test_id.as_deref() {
                check_bundle_for_stale_paint(&bundle_path, test_id, check_stale_paint_eps)?;
            }
            if let Some(test_id) = check_stale_scene_test_id.as_deref() {
                check_bundle_for_stale_scene(&bundle_path, test_id, check_stale_scene_eps)?;
            }
            if let Some(min) = check_idle_no_paint_min {
                let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
                let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
                check_bundle_for_idle_no_paint_min(&bundle_path, out_dir, min, warmup_frames)?;
            }
            if let Some(test_id) = check_pixels_changed_test_id.as_deref() {
                let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
                let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
                check_out_dir_for_pixels_changed(out_dir, test_id, warmup_frames)?;
            }
            if check_semantics_changed_repainted {
                check_bundle_for_semantics_changed_repainted(
                    &bundle_path,
                    warmup_frames,
                    dump_semantics_changed_repainted_json,
                )?;
            }
            if let Some(test_id) = check_wheel_scroll_test_id.as_deref() {
                check_bundle_for_wheel_scroll(bundle_path.as_path(), test_id, warmup_frames)?;
            }
            if let Some(test_id) = check_wheel_scroll_hit_changes_test_id.as_deref() {
                check_bundle_for_wheel_scroll_hit_changes(
                    bundle_path.as_path(),
                    test_id,
                    warmup_frames,
                )?;
            }
            if let Some(test_id) = check_drag_cache_root_paint_only_test_id.as_deref() {
                check_bundle_for_drag_cache_root_paint_only(&bundle_path, test_id, warmup_frames)?;
            }
            if let Some(max_allowed) = check_hover_layout_max {
                check_report_for_hover_layout_invalidations(&report, max_allowed)?;
            }
            if check_gc_sweep_liveness {
                check_bundle_for_gc_sweep_liveness(bundle_path.as_path(), warmup_frames)?;
            }
            for (file, max) in &check_notify_hotspot_file_max {
                check_bundle_for_notify_hotspot_file_max(
                    bundle_path.as_path(),
                    file.as_str(),
                    *max,
                    warmup_frames,
                )?;
            }
            if let Some(min) = check_view_cache_reuse_stable_min
                && min > 0
            {
                let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
                let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
                check_bundle_for_view_cache_reuse_stable_min(
                    bundle_path.as_path(),
                    out_dir,
                    min,
                    warmup_frames,
                )?;
            }
            if let Some(min) = check_view_cache_reuse_min
                && min > 0
            {
                check_bundle_for_view_cache_reuse_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_overlay_synthesis_min
                && min > 0
            {
                check_bundle_for_overlay_synthesis_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_viewport_input_min
                && min > 0
            {
                check_bundle_for_viewport_input_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_dock_drag_min
                && min > 0
            {
                check_bundle_for_dock_drag_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_viewport_capture_min
                && min > 0
            {
                check_bundle_for_viewport_capture_min(bundle_path.as_path(), min, warmup_frames)?;
            }
            if let Some(min) = check_retained_vlist_reconcile_no_notify_min
                && min > 0
            {
                check_bundle_for_retained_vlist_reconcile_no_notify_min(
                    bundle_path.as_path(),
                    min,
                    warmup_frames,
                )?;
            }
            if let Some(max_delta) = check_retained_vlist_attach_detach_max {
                check_bundle_for_retained_vlist_attach_detach_max(
                    bundle_path.as_path(),
                    max_delta,
                    warmup_frames,
                )?;
            }
            if let Some(min) = check_retained_vlist_keep_alive_reuse_min
                && min > 0
            {
                check_bundle_for_retained_vlist_keep_alive_reuse_min(
                    bundle_path.as_path(),
                    min,
                    warmup_frames,
                )?;
            }
            Ok(())
        }
        "matrix" => {
            let Some(target) = rest.first().cloned() else {
                return Err(
                    "missing matrix target (try: fretboard diag matrix ui-gallery)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }
            if target != "ui-gallery" {
                return Err(format!("unknown matrix target: {target}"));
            }

            let Some(launch) = &launch else {
                return Err(
                    "diag matrix requires --launch to run uncached/cached variants (for env control)"
                        .to_string(),
                );
            };

            let scripts: Vec<PathBuf> = ui_gallery_suite_scripts()
                .into_iter()
                .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                .collect();

            let compare_opts = CompareOptions {
                warmup_frames,
                eps_px: compare_eps_px,
                ignore_bounds: compare_ignore_bounds,
                ignore_scene_fingerprint: compare_ignore_scene_fingerprint,
            };

            // In matrix mode, treat `--check-view-cache-reuse-min 0` as "disabled".
            let reuse_gate = match check_view_cache_reuse_min {
                Some(0) => None,
                Some(v) => Some(v),
                None => Some(1),
            };

            // In matrix mode, treat `--check-view-cache-reuse-stable-min 0` as "disabled".
            let reuse_stable_gate = match check_view_cache_reuse_stable_min {
                Some(0) => None,
                Some(v) => Some(v),
                None => None,
            };

            // In matrix mode, treat `--check-overlay-synthesis-min 0` as "disabled".
            //
            // Default behavior:
            //
            // - If the caller enables shell reuse (`FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`), also
            //   enable a minimal overlay synthesis gate by default. This helps ensure the
            //   cached-synthesis seam is actually exercised (rather than "view cache enabled but
            //   overlay producers always rerendered").
            // - Otherwise, leave the gate off by default to avoid forcing overlay-specific
            //   assumptions onto non-overlay scripts (e.g. virtual-list torture).
            let mut matrix_base_env = launch_env.clone();
            let _ = ensure_env_var(&mut matrix_base_env, "FRET_DIAG_RENDERER_PERF", "1");

            let shell_reuse_enabled = matrix_base_env.iter().any(|(k, v)| {
                (k.as_str() == "FRET_UI_GALLERY_VIEW_CACHE_SHELL")
                    && !v.trim().is_empty()
                    && (v.as_str() != "0")
            });
            let overlay_synthesis_gate = match check_overlay_synthesis_min {
                Some(0) => None,
                Some(v) => Some(v),
                None => shell_reuse_enabled.then_some(1),
            };

            // In matrix mode, treat `--check-viewport-input-min 0` as "disabled".
            let viewport_input_gate = match check_viewport_input_min {
                Some(0) => None,
                Some(v) => Some(v),
                None => None,
            };

            let uncached_out_dir = resolved_out_dir.join("uncached");
            let cached_out_dir = resolved_out_dir.join("cached");

            let uncached_paths =
                ResolvedScriptPaths::for_out_dir(&workspace_root, &uncached_out_dir);
            let cached_paths = ResolvedScriptPaths::for_out_dir(&workspace_root, &cached_out_dir);

            let uncached_env = matrix_launch_env(&matrix_base_env, false)?;
            let cached_env = matrix_launch_env(&matrix_base_env, true)?;

            let uncached_bundles = run_script_suite_collect_bundles(
                &scripts,
                &uncached_paths,
                launch,
                &uncached_env,
                &workspace_root,
                timeout_ms,
                poll_ms,
                warmup_frames,
                None,
                None,
                None,
                None,
                viewport_input_gate,
                viewport_input_gate
                    .map(|_| ui_gallery_script_requires_viewport_input_gate as fn(&Path) -> bool),
                None,
                None,
            )?;
            let cached_bundles = run_script_suite_collect_bundles(
                &scripts,
                &cached_paths,
                launch,
                &cached_env,
                &workspace_root,
                timeout_ms,
                poll_ms,
                warmup_frames,
                reuse_stable_gate,
                reuse_gate,
                overlay_synthesis_gate,
                overlay_synthesis_gate.map(|_| {
                    ui_gallery_script_requires_overlay_synthesis_gate as fn(&Path) -> bool
                }),
                viewport_input_gate,
                viewport_input_gate
                    .map(|_| ui_gallery_script_requires_viewport_input_gate as fn(&Path) -> bool),
                None,
                None,
            )?;

            let mut ok = true;
            let mut comparisons: Vec<(PathBuf, CompareReport)> = Vec::new();
            for (idx, script) in scripts.iter().enumerate() {
                let a = uncached_bundles.get(idx).cloned().ok_or_else(|| {
                    format!("missing uncached bundle for script: {}", script.display())
                })?;
                let b = cached_bundles.get(idx).cloned().ok_or_else(|| {
                    format!("missing cached bundle for script: {}", script.display())
                })?;
                let report = compare_bundles(&a, &b, compare_opts)?;
                ok &= report.ok;
                comparisons.push((script.clone(), report));
            }

            if stats_json {
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "ok": ok,
                    "out_dir_uncached": uncached_paths.out_dir.display().to_string(),
                    "out_dir_cached": cached_paths.out_dir.display().to_string(),
                    "options": {
                        "warmup_frames": compare_opts.warmup_frames,
                        "eps_px": compare_opts.eps_px,
                        "ignore_bounds": compare_opts.ignore_bounds,
                        "ignore_scene_fingerprint": compare_opts.ignore_scene_fingerprint,
                        "check_view_cache_reuse_min": reuse_gate,
                        "check_view_cache_reuse_stable_min": reuse_stable_gate,
                        "check_overlay_synthesis_min": overlay_synthesis_gate,
                        "check_viewport_input_min": viewport_input_gate,
                    },
                    "comparisons": comparisons.iter().map(|(script, report)| serde_json::json!({
                        "script": script.display().to_string(),
                        "report": report.to_json(),
                    })).collect::<Vec<_>>(),
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                );
                if !ok {
                    std::process::exit(1);
                }
                Ok(())
            } else if ok {
                println!(
                    "matrix: ok (scripts={}, warmup_frames={}, check_view_cache_reuse_min={:?}, check_view_cache_reuse_stable_min={:?}, check_overlay_synthesis_min={:?}, check_viewport_input_min={:?})",
                    scripts.len(),
                    warmup_frames,
                    reuse_gate,
                    reuse_stable_gate,
                    overlay_synthesis_gate,
                    viewport_input_gate
                );
                Ok(())
            } else {
                println!(
                    "matrix: failed (scripts={}, warmup_frames={}, check_view_cache_reuse_min={:?}, check_view_cache_reuse_stable_min={:?}, check_overlay_synthesis_min={:?}, check_viewport_input_min={:?})",
                    scripts.len(),
                    warmup_frames,
                    reuse_gate,
                    reuse_stable_gate,
                    overlay_synthesis_gate,
                    viewport_input_gate
                );
                for (script, report) in comparisons {
                    if report.ok {
                        continue;
                    }
                    println!("\nscript: {}", script.display());
                    report.print_human();
                }
                Err("matrix compare failed".to_string())
            }
        }
        "compare" => {
            let Some(a_src) = rest.first().cloned() else {
                return Err(
                    "missing bundle A path (try: fretboard diag compare ./a/bundle.json ./b/bundle.json)".to_string(),
                );
            };
            let Some(b_src) = rest.get(1).cloned() else {
                return Err(
                    "missing bundle B path (try: fretboard diag compare ./a/bundle.json ./b/bundle.json)".to_string(),
                );
            };
            if rest.len() != 2 {
                return Err(format!("unexpected arguments: {}", rest[2..].join(" ")));
            }

            let a_src = resolve_path(&workspace_root, PathBuf::from(a_src));
            let b_src = resolve_path(&workspace_root, PathBuf::from(b_src));
            let a_bundle_path = resolve_bundle_json_path(&a_src);
            let b_bundle_path = resolve_bundle_json_path(&b_src);

            let report = compare_bundles(
                &a_bundle_path,
                &b_bundle_path,
                CompareOptions {
                    warmup_frames,
                    eps_px: compare_eps_px,
                    ignore_bounds: compare_ignore_bounds,
                    ignore_scene_fingerprint: compare_ignore_scene_fingerprint,
                },
            )?;

            if stats_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report.to_json())
                        .unwrap_or_else(|_| "{}".to_string())
                );
                if !report.ok {
                    std::process::exit(1);
                }
                Ok(())
            } else if report.ok {
                report.print_human();
                Ok(())
            } else {
                Err(report.to_human_error())
            }
        }
        "inspect" => {
            let Some(action) = rest.first().cloned() else {
                return Err(
                    "missing inspect action (try: fretboard diag inspect on|off|toggle|status)"
                        .to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            match action.as_str() {
                "status" => {
                    let cfg = read_inspect_config(&resolved_inspect_path);
                    let (enabled, consume_clicks) = match cfg {
                        Some(c) => (c.enabled, c.consume_clicks),
                        None => (false, true),
                    };
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "enabled": enabled,
                        "consume_clicks": consume_clicks,
                        "inspect_path": resolved_inspect_path.display().to_string(),
                        "inspect_trigger_path": resolved_inspect_trigger_path.display().to_string(),
                    });
                    println!(
                        "{}",
                        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
                    );
                    Ok(())
                }
                "on" | "off" | "toggle" => {
                    let prev = read_inspect_config(&resolved_inspect_path);
                    let prev_enabled = prev.as_ref().map(|c| c.enabled).unwrap_or(false);
                    let prev_consume_clicks =
                        prev.as_ref().map(|c| c.consume_clicks).unwrap_or(true);

                    let next_enabled = match action.as_str() {
                        "on" => true,
                        "off" => false,
                        "toggle" => !prev_enabled,
                        _ => unreachable!(),
                    };
                    let next_consume_clicks = inspect_consume_clicks.unwrap_or(prev_consume_clicks);

                    write_inspect_config(
                        &resolved_inspect_path,
                        InspectConfigV1 {
                            schema_version: 1,
                            enabled: next_enabled,
                            consume_clicks: next_consume_clicks,
                        },
                    )?;
                    touch(&resolved_inspect_trigger_path)?;
                    println!("{}", resolved_inspect_trigger_path.display());
                    Ok(())
                }
                other => Err(format!("unknown inspect action: {other}")),
            }
        }
        "pick-arm" => {
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            touch(&resolved_pick_trigger_path)?;
            println!("{}", resolved_pick_trigger_path.display());
            Ok(())
        }
        "pick" => {
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            let result = run_pick_and_wait(
                &resolved_pick_trigger_path,
                &resolved_pick_result_path,
                &resolved_pick_result_trigger_path,
                timeout_ms,
                poll_ms,
            )?;
            report_pick_result_and_exit(&result)
        }
        "pick-script" => {
            if !rest.is_empty() {
                return Err(format!("unexpected arguments: {}", rest.join(" ")));
            }
            let result = run_pick_and_wait(
                &resolved_pick_trigger_path,
                &resolved_pick_result_path,
                &resolved_pick_result_trigger_path,
                timeout_ms,
                poll_ms,
            )?;

            let Some(selector) = result.selector.clone() else {
                return Err("pick succeeded but no selector was returned".to_string());
            };

            write_pick_script(&selector, &resolved_pick_script_out)?;
            println!("{}", resolved_pick_script_out.display());
            Ok(())
        }
        "pick-apply" => {
            let Some(script) = rest.first().cloned() else {
                return Err(
                    "missing script path (try: fretboard diag pick-apply ./script.json --ptr /steps/0/target)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }
            let Some(ptr) = pick_apply_pointer.as_deref() else {
                return Err("missing --ptr (example: --ptr /steps/0/target)".to_string());
            };

            let result = run_pick_and_wait(
                &resolved_pick_trigger_path,
                &resolved_pick_result_path,
                &resolved_pick_result_trigger_path,
                timeout_ms,
                poll_ms,
            )?;

            let Some(selector) = result.selector.clone() else {
                return Err("pick succeeded but no selector was returned".to_string());
            };

            let script_path = resolve_path(&workspace_root, PathBuf::from(script));
            let out_path = pick_apply_out
                .map(|p| resolve_path(&workspace_root, p))
                .unwrap_or_else(|| script_path.clone());

            apply_pick_to_script(&script_path, &out_path, ptr, selector)?;
            println!("{}", out_path.display());
            Ok(())
        }
        other => Err(format!("unknown diag subcommand: {other}")),
    }
}

fn resolve_bundle_root_dir(path: &Path) -> Result<PathBuf, String> {
    if path.is_dir() {
        return Ok(path.to_path_buf());
    }
    let Some(parent) = path.parent() else {
        return Err(format!("invalid bundle path: {}", path.display()));
    };
    Ok(parent.to_path_buf())
}

fn default_pack_out_path(out_dir: &Path, bundle_dir: &Path) -> PathBuf {
    let name = bundle_dir
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");
    if bundle_dir.starts_with(out_dir) {
        out_dir.join("share").join(format!("{name}.zip"))
    } else {
        bundle_dir.with_extension("zip")
    }
}

fn default_triage_out_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("triage.json")
}

fn default_lint_out_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("check.lint.json")
}

fn pack_bundle_dir_to_zip(
    bundle_dir: &Path,
    out_path: &Path,
    include_root_artifacts: bool,
    include_triage: bool,
    include_screenshots: bool,
    include_renderdoc: bool,
    include_tracy: bool,
    artifacts_root: &Path,
    stats_top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> Result<(), String> {
    if !bundle_dir.is_dir() {
        return Err(format!(
            "bundle_dir is not a directory: {}",
            bundle_dir.display()
        ));
    }

    let bundle_json = bundle_dir.join("bundle.json");
    if !bundle_json.is_file() {
        return Err(format!(
            "bundle_dir does not contain bundle.json: {}",
            bundle_dir.display()
        ));
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let bundle_name = bundle_dir
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    zip_add_dir(
        &mut zip,
        bundle_dir,
        bundle_dir,
        bundle_name,
        out_path,
        options,
    )?;

    // Repro workflow helper: if a repro summary exists next to the bundle output root, include it.
    let repro_summary = artifacts_root.join("repro.summary.json");
    if repro_summary.is_file() {
        let dst = format!("{bundle_name}/_root/repro.summary.json");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&repro_summary).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
    }

    if include_root_artifacts {
        let root_prefix = format!("{bundle_name}/_root");
        zip_add_root_artifacts(&mut zip, artifacts_root, &root_prefix, options)?;
    }

    if include_renderdoc {
        let renderdoc_dir = artifacts_root.join("renderdoc");
        if renderdoc_dir.is_dir() {
            let renderdoc_prefix = format!("{bundle_name}/_root/renderdoc");
            zip_add_dir_filtered(
                &mut zip,
                &renderdoc_dir,
                &renderdoc_dir,
                &renderdoc_prefix,
                options,
                &["rdc", "json", "png", "txt", "md", "csv"],
            )?;
        }
    }

    if include_tracy {
        let tracy_dir = artifacts_root.join("tracy");
        if tracy_dir.is_dir() {
            let tracy_prefix = format!("{bundle_name}/_root/tracy");
            zip_add_dir_filtered(
                &mut zip,
                &tracy_dir,
                &tracy_dir,
                &tracy_prefix,
                options,
                &["tracy", "txt", "md", "json"],
            )?;
        }
    }

    if include_screenshots {
        let screenshots_dir = artifacts_root.join("screenshots").join(bundle_name);
        if screenshots_dir.is_dir() {
            let screenshots_prefix = format!("{bundle_name}/_root/screenshots");
            zip_add_screenshots(&mut zip, &screenshots_dir, &screenshots_prefix, options)?;
        }
    }

    if include_triage {
        use std::io::Write;

        let report = bundle_stats_from_path(
            &bundle_json,
            stats_top,
            sort,
            BundleStatsOptions { warmup_frames },
        )?;
        let payload = triage_json_from_stats(&bundle_json, &report, sort, warmup_frames);
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        let dst = format!("{bundle_name}/_root/triage.json");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn triage_json_from_stats(
    bundle_path: &Path,
    report: &BundleStatsReport,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> serde_json::Value {
    use serde_json::json;

    let generated_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64);

    let file_size_bytes = std::fs::metadata(bundle_path).ok().map(|m| m.len());

    let worst = report.top.first().map(|row| {
        json!({
            "window": row.window,
            "tick_id": row.tick_id,
            "frame_id": row.frame_id,
            "timestamp_unix_ms": row.timestamp_unix_ms,
            "total_time_us": row.total_time_us,
            "layout_time_us": row.layout_time_us,
            "prepaint_time_us": row.prepaint_time_us,
            "paint_time_us": row.paint_time_us,
            "invalidation_walk_calls": row.invalidation_walk_calls,
            "invalidation_walk_nodes": row.invalidation_walk_nodes,
            "cache_roots": row.cache_roots,
            "cache_roots_reused": row.cache_roots_reused,
            "cache_replayed_ops": row.cache_replayed_ops,
            "top_invalidation_walks": row.top_invalidation_walks.iter().take(10).map(|w| {
                json!({
                    "root_node": w.root_node,
                    "root_element": w.root_element,
                    "walked_nodes": w.walked_nodes,
                    "kind": w.kind,
                    "source": w.source,
                    "detail": w.detail,
                    "truncated_at": w.truncated_at,
                    "root_role": w.root_role,
                    "root_test_id": w.root_test_id,
                })
            }).collect::<Vec<_>>(),
            "top_cache_roots": row.top_cache_roots.iter().take(10).map(|r| {
                json!({
                    "root_node": r.root_node,
                    "element": r.element,
                    "reused": r.reused,
                    "contained_layout": r.contained_layout,
                    "paint_replayed_ops": r.paint_replayed_ops,
                    "reuse_reason": r.reuse_reason,
                    "root_role": r.root_role,
                    "root_test_id": r.root_test_id,
                })
            }).collect::<Vec<_>>(),
            "top_layout_engine_solves": row.top_layout_engine_solves.iter().take(4).map(|s| {
                json!({
                    "root_node": s.root_node,
                    "solve_time_us": s.solve_time_us,
                    "measure_calls": s.measure_calls,
                    "measure_cache_hits": s.measure_cache_hits,
                    "measure_time_us": s.measure_time_us,
                    "root_role": s.root_role,
                    "root_test_id": s.root_test_id,
                    "top_measures": s.top_measures.iter().take(10).map(|m| {
                        json!({
                            "node": m.node,
                            "measure_time_us": m.measure_time_us,
                            "calls": m.calls,
                            "cache_hits": m.cache_hits,
                            "element": m.element,
                            "element_kind": m.element_kind,
                            "role": m.role,
                            "test_id": m.test_id,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        })
    });

    json!({
        "schema_version": 1,
        "generated_unix_ms": generated_unix_ms,
        "bundle": {
            "bundle_path": bundle_path.display().to_string(),
            "bundle_dir": bundle_path.parent().map(|p| p.display().to_string()),
            "bundle_file_size_bytes": file_size_bytes,
        },
        "params": {
            "sort": sort.as_str(),
            "top": report.top.len(),
            "warmup_frames": warmup_frames,
        },
        "stats": report.to_json(),
        "worst": worst,
    })
}

fn zip_add_root_artifacts(
    zip: &mut zip::ZipWriter<std::fs::File>,
    artifacts_root: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<(), String> {
    let candidates = [
        "evidence.index.json",
        "script.json",
        "script.result.json",
        "pick.result.json",
        "screenshots.result.json",
        "triage.json",
        "picked.script.json",
        "check.semantics_changed_repainted.json",
        "check.pixels_changed.json",
        "check.idle_no_paint.json",
        "check.perf_thresholds.json",
        "check.redraw_hitches.json",
        "check.resource_footprint.json",
        "check.view_cache_reuse_stable.json",
        "resource.footprint.json",
        "redraw_hitches.log",
        "renderdoc.captures.json",
        "tracy.note.md",
    ];

    for name in candidates {
        let src = artifacts_root.join(name);
        if !src.is_file() {
            continue;
        }
        let dst = format!("{zip_prefix}/{name}");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&src).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn metadata_mtime_unix_ms(meta: &std::fs::Metadata) -> Option<u64> {
    let modified = meta.modified().ok()?;
    let dur = modified
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .ok()?;
    Some(dur.as_millis().min(u64::MAX as u128) as u64)
}

fn json_file_summary(path: &Path) -> Option<serde_json::Value> {
    let v = read_json_value(path)?;
    let schema_version = v.get("schema_version").and_then(|v| v.as_u64());
    let kind = v
        .get("kind")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let failures_len = v
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u64);
    let ok = failures_len.map(|n| n == 0);

    Some(serde_json::json!({
        "schema_version": schema_version,
        "kind": kind,
        "ok": ok,
        "failures_len": failures_len,
    }))
}

fn resource_footprint_summary(path: &Path) -> Option<serde_json::Value> {
    let v = read_json_value(path)?;
    let pid = v.get("pid").and_then(|v| v.as_u64());
    let wall_time_ms = v.get("wall_time_ms").and_then(|v| v.as_u64());
    let killed = v.get("killed").and_then(|v| v.as_bool());
    let note = v
        .get("note")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let cpu_avg_pct_total_cores = v
        .get("cpu")
        .and_then(|v| v.get("avg_cpu_percent_total_cores"))
        .and_then(|v| v.as_f64());
    let cpu_usage_pct_avg = v
        .get("cpu")
        .and_then(|v| v.get("usage_percent_avg"))
        .and_then(|v| v.as_f64());

    let working_set_bytes = v
        .get("memory")
        .and_then(|v| v.get("working_set_bytes"))
        .and_then(|v| v.as_u64());
    let peak_working_set_bytes = v
        .get("memory")
        .and_then(|v| v.get("peak_working_set_bytes"))
        .and_then(|v| v.as_u64());

    Some(serde_json::json!({
        "pid": pid,
        "wall_time_ms": wall_time_ms,
        "killed": killed,
        "note": note,
        "cpu_avg_percent_total_cores": cpu_avg_pct_total_cores,
        "cpu_usage_percent_avg": cpu_usage_pct_avg,
        "working_set_bytes": working_set_bytes,
        "peak_working_set_bytes": peak_working_set_bytes,
    }))
}

fn write_evidence_index(
    artifacts_root: &Path,
    summary_path: &Path,
    summary_json: Option<&serde_json::Value>,
) -> Result<PathBuf, String> {
    let out_path = artifacts_root.join("evidence.index.json");

    let mut entries: Vec<serde_json::Value> = Vec::new();
    let mut checks: Vec<serde_json::Value> = Vec::new();

    let mut add_file = |name: &str, rel: &str| {
        let path = artifacts_root.join(rel);
        let meta = std::fs::metadata(&path).ok();
        let exists = meta.is_some();
        let size_bytes = meta.as_ref().map(|m| m.len());
        let mtime_unix_ms = meta.as_ref().and_then(metadata_mtime_unix_ms);
        let json = if exists && rel.ends_with(".json") {
            json_file_summary(&path)
        } else {
            None
        };

        entries.push(serde_json::json!({
            "name": name,
            "rel_path": rel,
            "kind": "file",
            "exists": exists,
            "size_bytes": size_bytes,
            "mtime_unix_ms": mtime_unix_ms,
            "json": json,
        }));
    };

    add_file("repro.summary", "repro.summary.json");
    add_file("repro.zip", "repro.zip");
    add_file("resource.footprint", "resource.footprint.json");
    add_file("redraw_hitches", "redraw_hitches.log");
    add_file("renderdoc.captures", "renderdoc.captures.json");
    add_file("tracy.note", "tracy.note.md");
    add_file("script", "script.json");
    add_file("script.result", "script.result.json");
    add_file("pick.result", "pick.result.json");
    add_file("screenshots.result", "screenshots.result.json");
    add_file(
        "check.semantics_changed_repainted",
        "check.semantics_changed_repainted.json",
    );
    add_file("check.idle_no_paint", "check.idle_no_paint.json");
    add_file("check.pixels_changed", "check.pixels_changed.json");
    add_file("check.perf_thresholds", "check.perf_thresholds.json");
    add_file("check.redraw_hitches", "check.redraw_hitches.json");
    add_file("check.resource_footprint", "check.resource_footprint.json");
    add_file(
        "check.view_cache_reuse_stable",
        "check.view_cache_reuse_stable.json",
    );

    for e in entries.iter() {
        let Some(rel) = e.get("rel_path").and_then(|v| v.as_str()) else {
            continue;
        };
        if !rel.starts_with("check.") || !rel.ends_with(".json") {
            continue;
        }
        if e.get("exists").and_then(|v| v.as_bool()) != Some(true) {
            continue;
        }
        let name = e.get("name").cloned().unwrap_or(serde_json::Value::Null);
        let json = e.get("json").cloned().unwrap_or(serde_json::Value::Null);
        let ok = json.get("ok").cloned().unwrap_or(serde_json::Value::Null);
        checks.push(serde_json::json!({
            "name": name,
            "file": rel,
            "ok": ok,
            "summary": json,
        }));
    }

    let footprint = artifacts_root.join("resource.footprint.json");
    let resources = serde_json::json!({
        "process_footprint": if footprint.is_file() {
            resource_footprint_summary(&footprint)
        } else {
            None
        },
    });

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "out_dir": artifacts_root.display().to_string(),
        "summary_file": summary_path.file_name().and_then(|s| s.to_str()).unwrap_or("repro.summary.json"),
        "summary": summary_json.cloned(),
        "entries": entries,
        "checks": checks,
        "resources": resources,
    });

    let _ = write_json_value(&out_path, &payload);
    Ok(out_path)
}

#[derive(Debug, Clone)]
struct ReproZipBundle {
    prefix: String,
    bundle_json: PathBuf,
    source_script: PathBuf,
}

fn repro_zip_prefix_for_script(item: &ReproPackItem, idx: usize) -> String {
    let stem = item
        .script_path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("script");
    let safe = zip_safe_component(stem);
    format!("{:02}-{safe}", idx.saturating_add(1))
}

fn zip_safe_component(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        let keep = ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.');
        if keep {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "bundle".to_string()
    } else {
        trimmed.to_string()
    }
}

fn pack_repro_zip_multi(
    out_path: &Path,
    include_root_artifacts: bool,
    include_triage: bool,
    include_screenshots: bool,
    include_renderdoc: bool,
    include_tracy: bool,
    artifacts_root: &Path,
    summary_path: &Path,
    bundles: &[ReproZipBundle],
    stats_top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> Result<(), String> {
    use std::io::Write;

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Always include a machine-readable repro summary.
    if summary_path.is_file() {
        let bytes = std::fs::read(summary_path).map_err(|e| e.to_string())?;
        zip.start_file("_root/repro.summary.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    // Include script sources for offline triage.
    for (idx, item) in bundles.iter().enumerate() {
        let bytes = std::fs::read(&item.source_script).map_err(|e| e.to_string())?;
        let name = item
            .source_script
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("script.json");
        let safe = zip_safe_component(name);
        let dst = format!("_root/scripts/{:02}-{safe}", idx.saturating_add(1));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    if include_root_artifacts {
        zip_add_root_artifacts(&mut zip, artifacts_root, "_root", options)?;
    }

    if include_renderdoc {
        let renderdoc_dir = artifacts_root.join("renderdoc");
        if renderdoc_dir.is_dir() {
            zip_add_dir_filtered(
                &mut zip,
                &renderdoc_dir,
                &renderdoc_dir,
                "_root/renderdoc",
                options,
                &["rdc", "json", "png", "txt", "md", "csv"],
            )?;
        }
    }

    if include_tracy {
        let tracy_dir = artifacts_root.join("tracy");
        if tracy_dir.is_dir() {
            zip_add_dir_filtered(
                &mut zip,
                &tracy_dir,
                &tracy_dir,
                "_root/tracy",
                options,
                &["tracy", "txt", "md", "json"],
            )?;
        }
    }

    for item in bundles {
        let bundle_dir = resolve_bundle_root_dir(&item.bundle_json)?;
        zip_add_dir(
            &mut zip,
            &bundle_dir,
            &bundle_dir,
            &item.prefix,
            out_path,
            options,
        )?;

        if include_screenshots {
            let bundle_name = bundle_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let screenshots_dir = artifacts_root.join("screenshots").join(bundle_name);
            if screenshots_dir.is_dir() {
                let screenshots_prefix = format!("{}/_root/screenshots", item.prefix);
                zip_add_screenshots(&mut zip, &screenshots_dir, &screenshots_prefix, options)?;
            }
        }

        if include_triage {
            let report = bundle_stats_from_path(
                &item.bundle_json,
                stats_top,
                sort,
                BundleStatsOptions { warmup_frames },
            )?;
            let payload = triage_json_from_stats(&item.bundle_json, &report, sort, warmup_frames);
            let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
            let dst = format!("{}/_root/triage.json", item.prefix);
            zip.start_file(dst, options).map_err(|e| e.to_string())?;
            zip.write_all(&bytes).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn zip_add_screenshots(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<(), String> {
    zip_add_screenshot_dir(zip, dir, dir, zip_prefix, options)
}

fn zip_add_screenshot_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base_dir: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            zip_add_screenshot_dir(zip, &path, base_dir, zip_prefix, options)?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        // Keep this conservative to avoid exploding zip sizes accidentally.
        let should_include = matches!(ext.as_str(), "png") || name == "manifest.json";
        if !should_include {
            continue;
        }

        let rel = path
            .strip_prefix(base_dir)
            .map_err(|_| "failed to compute zip relative path".to_string())?;

        let dst = format!("{}/{}", zip_prefix, zip_name(rel));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_add_dir_filtered(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base_dir: &Path,
    zip_prefix: &str,
    options: FileOptions,
    allowed_exts: &[&str],
) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            zip_add_dir_filtered(zip, &path, base_dir, zip_prefix, options, allowed_exts)?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if ext.is_empty() {
            continue;
        }
        if !allowed_exts
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(ext.as_str()))
        {
            continue;
        }

        let rel = path
            .strip_prefix(base_dir)
            .map_err(|_| "failed to compute zip relative path".to_string())?;

        let dst = format!("{}/{}", zip_prefix, zip_name(rel));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_add_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base_dir: &Path,
    prefix: &str,
    out_path: &Path,
    options: FileOptions,
) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path == out_path {
            continue;
        }

        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            zip_add_dir(zip, &path, base_dir, prefix, out_path, options)?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let rel = path
            .strip_prefix(base_dir)
            .map_err(|_| "failed to compute zip relative path".to_string())?;

        let name = format!("{}/{}", prefix, zip_name(rel));
        zip.start_file(name, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_name(path: &Path) -> String {
    let mut out = String::new();
    for (i, c) in path.components().enumerate() {
        if i > 0 {
            out.push('/');
        }
        out.push_str(&c.as_os_str().to_string_lossy());
    }
    out
}

fn parse_bool(s: &str) -> Result<bool, ()> {
    match s {
        "1" | "true" | "True" | "TRUE" => Ok(true),
        "0" | "false" | "False" | "FALSE" => Ok(false),
        _ => Err(()),
    }
}

#[derive(Debug, Clone)]
struct InspectConfigV1 {
    schema_version: u32,
    enabled: bool,
    consume_clicks: bool,
}

fn read_inspect_config(path: &Path) -> Option<InspectConfigV1> {
    let bytes = std::fs::read(path).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("schema_version")?.as_u64()? != 1 {
        return None;
    }
    let enabled = v.get("enabled")?.as_bool()?;
    let consume_clicks = v
        .get("consume_clicks")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Some(InspectConfigV1 {
        schema_version: 1,
        enabled,
        consume_clicks,
    })
}

fn write_inspect_config(path: &Path, cfg: InspectConfigV1) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let v = serde_json::json!({
        "schema_version": cfg.schema_version,
        "enabled": cfg.enabled,
        "consume_clicks": cfg.consume_clicks,
    });
    let bytes = serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

fn resolve_path(workspace_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    }
}

fn normalize_host_path_separators(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        return PathBuf::from(path.to_string_lossy().replace('/', "\\"));
    }
    #[cfg(not(windows))]
    {
        path
    }
}

fn expand_script_inputs(workspace_root: &Path, inputs: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut set: BTreeSet<PathBuf> = BTreeSet::new();

    for input in inputs {
        let resolved = resolve_path(workspace_root, PathBuf::from(input));

        // Directory input: treat as recursive `**/*.json` to support suite-like workflows.
        if resolved.is_dir() {
            let mut pattern = resolved.to_string_lossy().to_string();
            pattern = pattern.replace('\\', "/");
            if !pattern.ends_with('/') {
                pattern.push('/');
            }
            pattern.push_str("**/*.json");

            let mut any = false;
            for entry in glob::glob(&pattern).map_err(|e| e.to_string())? {
                let path = entry.map_err(|e| e.to_string())?;
                set.insert(normalize_host_path_separators(path));
                any = true;
            }
            if !any {
                return Err(format!(
                    "script input matched no files: {input} ({pattern})"
                ));
            }
            continue;
        }

        // Wildcard input: expand via glob. (PowerShell doesn't always expand globs for child args.)
        if input.contains('*') || input.contains('?') || input.contains('[') {
            let mut pattern = resolved.to_string_lossy().to_string();
            pattern = pattern.replace('\\', "/");

            let mut any = false;
            for entry in glob::glob(&pattern).map_err(|e| e.to_string())? {
                let path = entry.map_err(|e| e.to_string())?;
                set.insert(normalize_host_path_separators(path));
                any = true;
            }
            if !any {
                return Err(format!(
                    "script input matched no files: {input} ({pattern})"
                ));
            }
            continue;
        }

        set.insert(resolved);
    }

    Ok(set.into_iter().collect())
}

fn resolve_bundle_json_path(path: &Path) -> PathBuf {
    if !path.is_dir() {
        return path.to_path_buf();
    }

    let direct = path.join("bundle.json");
    if direct.is_file() {
        return direct;
    }

    if let Some(dir) = read_latest_pointer(path).or_else(|| find_latest_export_dir(path)) {
        let nested = dir.join("bundle.json");
        if nested.is_file() {
            return nested;
        }
    }

    direct
}

fn bundle_paint_cache_hit_test_only_replay_maxes(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(u64, u64), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok((0, 0));
    }

    let mut allowed_max: u64 = 0;
    let mut rejected_key_mismatch_max: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }

            let stats = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.as_object());
            let Some(stats) = stats else {
                continue;
            };

            let allowed = stats
                .get("paint_cache_hit_test_only_replay_allowed")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let rejected = stats
                .get("paint_cache_hit_test_only_replay_rejected_key_mismatch")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            allowed_max = allowed_max.max(allowed);
            rejected_key_mismatch_max = rejected_key_mismatch_max.max(rejected);
        }
    }

    Ok((allowed_max, rejected_key_mismatch_max))
}

fn wait_for_bundle_json_from_script_result(
    out_dir: &Path,
    result: &ScriptResultSummary,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.min(5_000).max(250));
    while Instant::now() < deadline {
        let dir = result
            .last_bundle_dir
            .as_deref()
            .and_then(|s| (!s.trim().is_empty()).then_some(s.trim()))
            .map(PathBuf::from)
            .map(|p| if p.is_absolute() { p } else { out_dir.join(p) })
            .or_else(|| read_latest_pointer(out_dir))
            .or_else(|| find_latest_export_dir(out_dir));
        if let Some(dir) = dir {
            let bundle_path = resolve_bundle_json_path(&dir);
            if bundle_path.is_file() {
                return Some(bundle_path);
            }
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
    None
}

fn wait_for_bundle_json_in_dir(
    bundle_dir: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.min(5_000).max(250));
    let bundle_path = resolve_bundle_json_path(bundle_dir);
    while Instant::now() < deadline {
        if bundle_path.is_file() {
            return Some(bundle_path.clone());
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
    None
}

fn ui_gallery_suite_scripts() -> [&'static str; 55] {
    [
        "tools/diag-scripts/ui-gallery-overlay-torture.json",
        "tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json",
        "tools/diag-scripts/ui-gallery-popover-dialog-escape-underlay.json",
        "tools/diag-scripts/ui-gallery-portal-geometry-scroll-clamp.json",
        "tools/diag-scripts/ui-gallery-dropdown-open-select.json",
        "tools/diag-scripts/ui-gallery-dropdown-submenu-underlay-dismiss.json",
        "tools/diag-scripts/ui-gallery-context-menu-right-click.json",
        "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json",
        "tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json",
        "tools/diag-scripts/ui-gallery-slider-set-value.json",
        "tools/diag-scripts/ui-gallery-shadcn-extras-screenshots.json",
        "tools/diag-scripts/ui-gallery-shadcn-extras-kanban-dnd.json",
        "tools/diag-scripts/ui-gallery-hover-layout-torture.json",
        "tools/diag-scripts/ui-gallery-material3-tabs-switch-perf.json",
        "tools/diag-scripts/ui-gallery-view-cache-toggle-perf.json",
        "tools/diag-scripts/ui-gallery-table-smoke.json",
        "tools/diag-scripts/ui-gallery-data-table-smoke.json",
        "tools/diag-scripts/ui-gallery-virtual-list-torture.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-scroll-stability.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-soft-wrap-geom-fallback-baseline.json",
        "tools/diag-scripts/ui-gallery-code-view-scroll-refresh-pixels-changed.json",
        "tools/diag-scripts/ui-gallery-code-view-torture-wheel-scroll-hit-changes.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-read-only-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-read-only-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-soft-wrap-toggle-stability-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-soft-wrap-editing-selection-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-inlays-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-a11y-composition-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-folds-placeholder-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-folds-clamp-selection-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-folds-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-folds-soft-wrap-inline-preedit-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-caret-navigation-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-soft-wrap-inline-preedit-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-placeholder-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-with-decorations-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-with-decorations-composed-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-with-decorations-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-with-decorations-composed-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-word-boundary-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-word-boundary-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-word-boundary-soft-wrap-double-click-baseline.json",
    ]
}

fn ui_gallery_code_editor_suite_scripts() -> [&'static str; 43] {
    [
        "tools/diag-scripts/ui-gallery-code-editor-torture-scroll-stability.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-soft-wrap-geom-fallback-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-read-only-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-read-only-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-soft-wrap-toggle-stability-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-soft-wrap-editing-selection-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-inlays-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-a11y-composition-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-folds-placeholder-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-folds-clamp-selection-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-folds-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-folds-soft-wrap-inline-preedit-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-caret-navigation-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-soft-wrap-inline-preedit-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-placeholder-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-with-decorations-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-with-decorations-composed-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-with-decorations-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-with-decorations-composed-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-word-boundary-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-word-boundary-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-word-boundary-soft-wrap-double-click-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-a11y-selection-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-a11y-composition-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-a11y-composition-drag-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-a11y-selection-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-a11y-composition-soft-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-a11y-selection-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-a11y-composition-wrap-baseline.json",
        "tools/diag-scripts/ui-gallery-code-editor-a11y-composition-wrap-scroll-baseline.json",
    ]
}

fn ui_gallery_overlay_steady_suite_scripts() -> [&'static str; 4] {
    [
        "tools/diag-scripts/ui-gallery-overlay-torture-steady.json",
        "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json",
        "tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json",
        "tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json",
    ]
}

fn ui_gallery_date_picker_suite_scripts() -> [&'static str; 1] {
    ["tools/diag-scripts/ui-gallery-date-picker-range-roving-skips-disabled.json"]
}

fn ui_gallery_text_ime_suite_scripts() -> [&'static str; 1] {
    ["tools/diag-scripts/ui-gallery-input-ime-tab-suppressed.json"]
}

fn ui_gallery_combobox_suite_scripts() -> [&'static str; 6] {
    [
        "tools/diag-scripts/ui-gallery-combobox-open-select-focus-restore.json",
        "tools/diag-scripts/ui-gallery-combobox-keyboard-commit-apple.json",
        "tools/diag-scripts/ui-gallery-combobox-typeahead-commit-banana.json",
        "tools/diag-scripts/ui-gallery-combobox-escape-dismiss-focus-restore.json",
        "tools/diag-scripts/ui-gallery-combobox-dismiss-outside-press.json",
        "tools/diag-scripts/ui-gallery-combobox-roving-skips-disabled.json",
    ]
}

fn ui_gallery_select_suite_scripts() -> [&'static str; 10] {
    [
        "tools/diag-scripts/ui-gallery-select-commit-and-label-update-bundle.json",
        "tools/diag-scripts/ui-gallery-select-keyboard-commit-apple.json",
        "tools/diag-scripts/ui-gallery-select-typeahead-commit-banana.json",
        "tools/diag-scripts/ui-gallery-select-disabled-item-no-commit.json",
        "tools/diag-scripts/ui-gallery-select-roving-skips-disabled-orange.json",
        "tools/diag-scripts/ui-gallery-select-dismiss-outside-press.json",
        "tools/diag-scripts/ui-gallery-select-escape-dismiss-focus-restore.json",
        "tools/diag-scripts/ui-gallery-select-open-jitter-click-stable-v2.json",
        "tools/diag-scripts/ui-gallery-select-wheel-scroll.json",
        "tools/diag-scripts/ui-gallery-select-wheel-up-from-bottom.json",
    ]
}

fn ui_gallery_shadcn_conformance_suite_scripts() -> [&'static str; 13] {
    [
        "tools/diag-scripts/ui-gallery-alert-dialog-least-destructive-initial-focus.json",
        "tools/diag-scripts/ui-gallery-card-description-no-early-wrap.json",
        "tools/diag-scripts/ui-gallery-dialog-docs-order-smoke.json",
        "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json",
        "tools/diag-scripts/ui-gallery-dropdown-open-select.json",
        "tools/diag-scripts/ui-gallery-dropdown-submenu-underlay-dismiss.json",
        "tools/diag-scripts/ui-gallery-popover-click-through-outside-press-focus-underlay.json",
        "tools/diag-scripts/ui-gallery-popover-dialog-escape-underlay.json",
        "tools/diag-scripts/ui-gallery-tooltip-hovercard-scroll-clamp.json",
        "tools/diag-scripts/ui-gallery-tooltip-repeat-hover.json",
        "tools/diag-scripts/ui-gallery-combobox-open-select-focus-restore.json",
        "tools/diag-scripts/ui-gallery-button-with-icon-non-overlap.json",
        "tools/diag-scripts/ui-gallery-input-group-text-non-overlap.json",
    ]
}

fn ui_gallery_layout_suite_scripts() -> [&'static str; 6] {
    [
        "tools/diag-scripts/ui-gallery-layout-sweep-core.json",
        "tools/diag-scripts/ui-gallery-layout-sweep-extended.json",
        "tools/diag-scripts/ui-gallery-layout-sweep-extended-chrome.json",
        "tools/diag-scripts/ui-gallery-layout-sweep-torture.json",
        "tools/diag-scripts/ui-gallery-chrome-torture-layout.json",
        "tools/diag-scripts/ui-gallery-hover-layout-torture.json",
    ]
}

fn docking_arbitration_suite_scripts() -> [&'static str; 2] {
    [
        "tools/diag-scripts/docking-arbitration-demo-split-viewports.json",
        "tools/diag-scripts/docking-arbitration-demo-modal-dock-drag-viewport-capture.json",
    ]
}

fn docking_arbitration_script_default_gates(
    script: &Path,
) -> (Option<u64>, Option<u64>, Option<u64>) {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return (None, None, None);
    };

    match name {
        "docking-arbitration-demo-split-viewports.json" => (Some(1), None, None),
        "docking-arbitration-demo-modal-dock-drag-viewport-capture.json" => {
            (Some(1), Some(1), Some(1))
        }
        _ => (None, None, None),
    }
}

fn ui_gallery_script_requires_retained_vlist_reconcile_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-ai-transcript-torture-scroll.json"
            | "ui-gallery-virtual-list-window-boundary-scroll-retained.json"
            | "ui-gallery-tree-window-boundary-scroll-retained.json"
            | "ui-gallery-data-table-window-boundary-scroll-retained.json"
            | "ui-gallery-table-retained-window-boundary-scroll.json"
            | "components-gallery-file-tree-window-boundary-scroll.json"
            | "components-gallery-file-tree-window-boundary-bounce.json"
            | "components-gallery-table-window-boundary-scroll.json"
            | "components-gallery-table-window-boundary-bounce.json"
    )
}

fn ui_gallery_script_requires_retained_vlist_keep_alive_reuse_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "components-gallery-file-tree-window-boundary-bounce.json"
            | "components-gallery-table-window-boundary-bounce.json"
            | "ui-gallery-data-table-window-boundary-bounce-keep-alive.json"
            | "ui-gallery-inspector-torture-bounce-keep-alive.json"
            | "workspace-shell-demo-file-tree-bounce-keep-alive.json"
    )
}

fn ui_gallery_script_requires_overlay_synthesis_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    // These scripts are expected to exercise the cached overlay synthesis seam when view-cache
    // shell reuse is enabled.
    matches!(
        name,
        "ui-gallery-overlay-torture.json"
            | "ui-gallery-modal-barrier-underlay-block.json"
            | "ui-gallery-popover-dialog-escape-underlay.json"
            | "ui-gallery-portal-geometry-scroll-clamp.json"
            | "ui-gallery-dropdown-open-select.json"
            | "ui-gallery-dropdown-submenu-underlay-dismiss.json"
            | "ui-gallery-context-menu-right-click.json"
            | "ui-gallery-dialog-escape-focus-restore.json"
            | "ui-gallery-menubar-keyboard-nav.json"
    )
}

fn ui_gallery_script_requires_viewport_input_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    // Viewport input forwarding is only expected in scripts that explicitly exercise viewport
    // panels / docking viewport tooling scenarios.
    name.contains("viewport") || name.contains("dock")
}

fn ui_gallery_script_requires_windowed_rows_offset_changes_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-scroll-stability.json"
            | "ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json"
    )
}

fn ui_gallery_script_requires_windowed_rows_visible_start_repaint_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(name, "ui-gallery-code-editor-torture-scroll-stability.json")
}

fn ui_gallery_script_pixels_changed_test_id(script: &Path) -> Option<&'static str> {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return None;
    };

    match name {
        "ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json" => {
            Some("ui-gallery-code-editor-torture-root")
        }
        "ui-gallery-code-view-scroll-refresh-pixels-changed.json" => {
            Some("ui-gallery-code-view-root")
        }
        _ => None,
    }
}

fn ui_gallery_script_wheel_scroll_hit_changes_test_id(script: &Path) -> Option<&'static str> {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return None;
    };

    match name {
        "ui-gallery-select-wheel-scroll.json" => Some("select-scroll-viewport"),
        "ui-gallery-select-wheel-up-from-bottom.json" => Some("select-scroll-viewport"),
        "ui-gallery-code-view-torture-wheel-scroll-hit-changes.json" => {
            Some("ui-gallery-code-view-root")
        }
        _ => None,
    }
}

fn ui_gallery_script_requires_code_editor_torture_marker_present_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_undo_redo_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_geom_fallbacks_low_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-soft-wrap-geom-fallback-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_read_only_blocks_edits_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-read-only-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_read_only_blocks_edits_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-read-only-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_disabled_blocks_edits_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-disabled-baseline.json"
            | "ui-gallery-markdown-editor-source-disabled-inject-preedit-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_soft_wrap_toggle_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-soft-wrap-toggle-stability-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_word_boundary_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-word-boundary-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-inlays-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-soft-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_web_ime_bridge_enabled_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-web-markdown-editor-source-ime-bridge-attach-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_line_boundary_triple_click_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-line-boundary-triple-click-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_a11y_composition_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-a11y-composition-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_a11y_composition_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-a11y-composition-soft-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-soft-wrap-editing-selection-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_folds_toggle_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-placeholder-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_folds_clamp_selection_out_of_folds_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-clamp-selection-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-placeholder-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_under_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-soft-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-soft-wrap-inline-preedit-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_inlays_present_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-inlays-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_inlays_toggle_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_inlays_caret_navigation_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-caret-navigation-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_inlays_present_under_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-soft-wrap-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-soft-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_markdown_editor_source_inlays_absent_under_inline_preedit_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-soft-wrap-inline-preedit-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-placeholder-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-soft-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_absent_under_inline_preedit_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-inline-preedit-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-with-decorations-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-with-decorations-composed-baseline.json"
            | "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_composed_preedit_stable_after_wheel_scroll_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_composed_preedit_cancels_on_drag_selection_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-drag-select-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_inlays_present_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(name, "ui-gallery-code-editor-torture-inlays-baseline.json")
}

fn ui_gallery_script_requires_code_editor_torture_inlays_absent_under_inline_preedit_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-inline-preedit-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-with-decorations-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-with-decorations-composed-baseline.json"
            | "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_torture_inlays_present_under_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-soft-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_word_boundary_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-word-boundary-baseline.json"
            | "ui-gallery-code-editor-word-boundary-soft-wrap-baseline.json"
            | "ui-gallery-code-editor-word-boundary-soft-wrap-double-click-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_a11y_selection_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-selection-baseline.json"
            | "ui-gallery-code-editor-a11y-selection-soft-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_a11y_composition_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-composition-baseline.json"
            | "ui-gallery-code-editor-a11y-composition-soft-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_a11y_selection_wrap_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-selection-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_a11y_composition_wrap_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-composition-wrap-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_a11y_composition_wrap_scroll_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-composition-wrap-scroll-baseline.json"
    )
}

fn ui_gallery_script_requires_code_editor_a11y_composition_drag_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-composition-drag-baseline.json"
    )
}

fn script_requests_screenshots(script: &Path) -> bool {
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

fn gate_required_capabilities(
    out_path: &Path,
    required: &[String],
    available: &[String],
) -> Result<(), String> {
    gate_required_capabilities_with_source(out_path, required, available, "filesystem")
}

fn gate_required_capabilities_with_source(
    out_path: &Path,
    required: &[String],
    available: &[String],
    source: &str,
) -> Result<(), String> {
    let available_set: std::collections::HashSet<&str> =
        available.iter().map(|s| s.as_str()).collect();
    let mut missing: Vec<String> = required
        .iter()
        .filter(|c| !available_set.contains(c.as_str()))
        .cloned()
        .collect();
    missing.sort();
    missing.dedup();

    if missing.is_empty() {
        return Ok(());
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "status": "failed",
        "source": source,
        "required": required,
        "available": available,
        "missing": missing,
    });
    let _ = write_json_value(out_path, &payload);

    Err(format!(
        "missing required diagnostics capabilities: {} (see {})",
        missing.join(", "),
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

        let err = gate_required_capabilities(&check_path, &required, &available).unwrap_err();
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
            "unknown --devtools-session-id: {want} (known: {known})"
        ));
    }

    if list.sessions.len() == 1 {
        return Ok(list.sessions[0].session_id.clone());
    }

    let web_apps = list
        .sessions
        .iter()
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

fn run_script_over_devtools_ws(
    out_dir: &Path,
    ws_url: &str,
    token: &str,
    session_id: Option<&str>,
    script_json: serde_json::Value,
    dump_bundle: bool,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(UiScriptResultV1, Option<PathBuf>), String> {
    use crate::transport::{
        ClientKindV1, DevtoolsWsClientConfig, ToolingDiagClient, WsDiagTransportConfig,
    };

    fn wait_for_message<T>(
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

    let sessions = wait_for_message(&devtools, timeout_ms, poll_ms, |msg| {
        if msg.r#type != "session.list" {
            return None;
        }
        serde_json::from_value::<DevtoolsSessionListV1>(msg.payload).ok()
    })?;

    let selected_session_id = devtools_select_session_id(&sessions, session_id)?;
    devtools.set_default_session_id(Some(selected_session_id.clone()));

    let required_caps = script_required_capabilities_value(&script_json);
    if !required_caps.is_empty() {
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

        gate_required_capabilities_with_source(
            &out_dir.join("check.capabilities.json"),
            &required_caps,
            &available_caps,
            "devtools_ws",
        )?;
    }

    devtools.script_run_value(None, script_json);
    let result = wait_for_message(&devtools, timeout_ms, poll_ms, |msg| {
        if msg.r#type != "script.result" || msg.session_id.as_deref() != Some(&selected_session_id)
        {
            return None;
        }
        serde_json::from_value::<UiScriptResultV1>(msg.payload).ok()
    })?;

    if !dump_bundle {
        return Ok((result, None));
    }

    devtools.bundle_dump(None, Some("diag-run"));
    let dumped = wait_for_message(&devtools, timeout_ms, poll_ms, |msg| {
        if msg.r#type != "bundle.dumped" || msg.session_id.as_deref() != Some(&selected_session_id)
        {
            return None;
        }
        serde_json::from_value::<DevtoolsBundleDumpedV1>(msg.payload).ok()
    })?;

    let export_dir_name = devtools_sanitize_export_dir_name(&dumped.dir);
    let export_dir = out_dir.join(&export_dir_name);
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
    let bundle = dumped.bundle.clone().ok_or_else(|| {
        "bundle.dumped did not include an embedded bundle payload (set diagnostics to embed bundles)"
            .to_string()
    })?;
    let bundle_path = export_dir.join("bundle.json");
    write_json_value(&bundle_path, &bundle)?;
    let dumped_path = export_dir.join("bundle.dumped.json");
    write_json_value(
        &dumped_path,
        &serde_json::to_value(&dumped).unwrap_or_else(|_| serde_json::json!({})),
    )?;
    let _ = std::fs::write(out_dir.join("latest.txt"), export_dir_name.as_bytes());

    Ok((result, Some(bundle_path)))
}

fn run_script_suite_collect_bundles(
    scripts: &[PathBuf],
    paths: &ResolvedScriptPaths,
    launch: &[String],
    launch_env: &[(String, String)],
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
    )?;

    let mut required_caps: Vec<String> = Vec::new();
    for src in scripts {
        required_caps.extend(script_required_capabilities(src));
    }
    required_caps.sort();
    required_caps.dedup();
    if !required_caps.is_empty() {
        let available_caps = read_filesystem_capabilities(&paths.out_dir);
        if let Err(e) = gate_required_capabilities(
            &paths.out_dir.join("check.capabilities.json"),
            &required_caps,
            &available_caps,
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
        {
            if let Some(dir) =
                wait_for_failure_dump_bundle(&paths.out_dir, summary, timeout_ms, poll_ms)
            {
                if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                    if let Ok(summary) = result.as_mut() {
                        summary.last_bundle_dir = Some(name.to_string());
                    }
                }
            }
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

        let bundle_path =
            wait_for_bundle_json_from_script_result(&paths.out_dir, &result, timeout_ms, poll_ms)
                .ok_or_else(|| {
                format!(
                    "script passed but no bundle.json was found (required for matrix): {}",
                    src.display()
                )
            })?;

        if let Some(min) = check_view_cache_reuse_stable_min
            && min > 0
        {
            check_bundle_for_view_cache_reuse_stable_min(
                &bundle_path,
                &paths.out_dir,
                min,
                warmup_frames,
            )?;
        }
        if let Some(min) = check_view_cache_reuse_min
            && min > 0
        {
            check_bundle_for_view_cache_reuse_min(&bundle_path, min, warmup_frames)?;
        }
        if let Some(min) = check_overlay_synthesis_min
            && min > 0
        {
            let should_gate = overlay_synthesis_gate_predicate
                .map(|pred| pred(src))
                .unwrap_or(true);
            if should_gate {
                check_bundle_for_overlay_synthesis_min(&bundle_path, min, warmup_frames)?;
            }
        }
        if let Some(min) = check_viewport_input_min
            && min > 0
        {
            let should_gate = viewport_input_gate_predicate
                .map(|pred| pred(src))
                .unwrap_or(true);
            if should_gate {
                check_bundle_for_viewport_input_min(&bundle_path, min, warmup_frames)?;
            }
        }
        if let Some(min) = check_dock_drag_min
            && min > 0
        {
            check_bundle_for_dock_drag_min(&bundle_path, min, warmup_frames)?;
        }
        if let Some(min) = check_viewport_capture_min
            && min > 0
        {
            check_bundle_for_viewport_capture_min(&bundle_path, min, warmup_frames)?;
        }

        bundle_paths.push(bundle_path);
    }

    let _ = stop_launched_demo(&mut child, &paths.exit_path, poll_ms);
    Ok(bundle_paths)
}

fn apply_post_run_checks(
    bundle_path: &Path,
    out_dir: &Path,
    check_idle_no_paint_min: Option<u64>,
    check_stale_paint_test_id: Option<&str>,
    check_stale_paint_eps: f32,
    check_stale_scene_test_id: Option<&str>,
    check_stale_scene_eps: f32,
    check_pixels_changed_test_id: Option<&str>,
    check_ui_gallery_code_editor_torture_marker_present: bool,
    check_ui_gallery_code_editor_torture_undo_redo: bool,
    check_ui_gallery_code_editor_torture_geom_fallbacks_low: bool,
    check_ui_gallery_code_editor_torture_read_only_blocks_edits: bool,
    check_ui_gallery_markdown_editor_source_read_only_blocks_edits: bool,
    check_ui_gallery_markdown_editor_source_disabled_blocks_edits: bool,
    check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable: bool,
    check_ui_gallery_markdown_editor_source_word_boundary: bool,
    check_ui_gallery_web_ime_bridge_enabled: bool,
    check_ui_gallery_markdown_editor_source_line_boundary_triple_click: bool,
    check_ui_gallery_markdown_editor_source_a11y_composition: bool,
    check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap: bool,
    check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable: bool,
    check_ui_gallery_markdown_editor_source_folds_toggle_stable: bool,
    check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds: bool,
    check_ui_gallery_markdown_editor_source_folds_placeholder_present: bool,
    check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap: bool,
    check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit: bool,
    check_ui_gallery_markdown_editor_source_inlays_toggle_stable: bool,
    check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable: bool,
    check_ui_gallery_markdown_editor_source_inlays_present: bool,
    check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap: bool,
    check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit: bool,
    check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit: bool,
    check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped: bool,
    check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations: bool,
    check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed: bool,
    check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed: bool,
    check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed: bool,
    check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll: bool,
    check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection: bool,
    check_ui_gallery_code_editor_torture_folds_placeholder_present: bool,
    check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap: bool,
    check_ui_gallery_code_editor_torture_inlays_present: bool,
    check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit: bool,
    check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped: bool,
    check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations: bool,
    check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed: bool,
    check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap: bool,
    check_ui_gallery_code_editor_word_boundary: bool,
    check_ui_gallery_code_editor_a11y_selection: bool,
    check_ui_gallery_code_editor_a11y_composition: bool,
    check_ui_gallery_code_editor_a11y_selection_wrap: bool,
    check_ui_gallery_code_editor_a11y_composition_wrap: bool,
    check_ui_gallery_code_editor_a11y_composition_wrap_scroll: bool,
    check_ui_gallery_code_editor_a11y_composition_drag: bool,
    check_semantics_changed_repainted: bool,
    dump_semantics_changed_repainted_json: bool,
    check_wheel_scroll_test_id: Option<&str>,
    check_wheel_scroll_hit_changes_test_id: Option<&str>,
    check_prepaint_actions_min: Option<u64>,
    check_chart_sampling_window_shifts_min: Option<u64>,
    check_node_graph_cull_window_shifts_min: Option<u64>,
    check_node_graph_cull_window_shifts_max: Option<u64>,
    check_vlist_visible_range_refreshes_min: Option<u64>,
    check_vlist_visible_range_refreshes_max: Option<u64>,
    check_vlist_window_shifts_explainable: bool,
    check_vlist_window_shifts_have_prepaint_actions: bool,
    check_vlist_window_shifts_non_retained_max: Option<u64>,
    check_vlist_window_shifts_prefetch_max: Option<u64>,
    check_vlist_window_shifts_escape_max: Option<u64>,
    check_vlist_policy_key_stable: bool,
    check_windowed_rows_offset_changes_min: Option<u64>,
    check_windowed_rows_offset_changes_eps: f32,
    check_windowed_rows_visible_start_changes_repainted: bool,
    check_layout_fast_path_min: Option<u64>,
    check_drag_cache_root_paint_only_test_id: Option<&str>,
    check_hover_layout_max: Option<u32>,
    check_gc_sweep_liveness: bool,
    check_notify_hotspot_file_max: &[(String, u64)],
    check_view_cache_reuse_stable_min: Option<u64>,
    check_view_cache_reuse_min: Option<u64>,
    check_overlay_synthesis_min: Option<u64>,
    check_viewport_input_min: Option<u64>,
    check_dock_drag_min: Option<u64>,
    check_viewport_capture_min: Option<u64>,
    check_retained_vlist_reconcile_no_notify_min: Option<u64>,
    check_retained_vlist_attach_detach_max: Option<u64>,
    check_retained_vlist_keep_alive_reuse_min: Option<u64>,
    check_retained_vlist_keep_alive_budget: Option<(u64, u64)>,
    warmup_frames: u64,
) -> Result<(), String> {
    // Prefer the most recent export directory recorded by the diagnostics runtime.
    //
    // `script.result.json` currently reports the last "auto dump" directory (e.g. `press_key`),
    // but scripts typically emit explicit `capture_bundle` exports that include additional frames
    // after the triggering input. Post-run gates should run on the latest export to avoid
    // sampling before the UI has produced updated semantics.
    //
    // Note: the runtime may update `latest.txt` slightly after writing `script.result.json`.
    // Poll briefly to avoid sampling too early.
    let bundle_path_for_checks = {
        fn parse_leading_ts(name: &str) -> Option<u64> {
            let digits: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
            if digits.is_empty() {
                return None;
            }
            digits.parse::<u64>().ok()
        }

        fn normalize_bundle_path(p: std::path::PathBuf) -> std::path::PathBuf {
            if p.extension().is_some_and(|ext| ext == "json") {
                p
            } else {
                p.join("bundle.json")
            }
        }

        fn path_ts(p: &std::path::Path) -> Option<u64> {
            let dir = p.parent()?;
            let name = dir.file_name()?.to_string_lossy();
            parse_leading_ts(&name)
        }

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let mut best: Option<std::path::PathBuf> = None;

        loop {
            let from_latest = compare::read_latest_pointer(out_dir).map(normalize_bundle_path);
            let from_scan = compare::find_latest_export_dir(out_dir)
                .map(|dir| normalize_bundle_path(dir.join("bundle.json")));

            let candidate = match (from_latest, from_scan) {
                (Some(a), Some(b)) => match (path_ts(&a), path_ts(&b)) {
                    (Some(ta), Some(tb)) => {
                        if tb >= ta {
                            Some(b)
                        } else {
                            Some(a)
                        }
                    }
                    (None, Some(_)) => Some(b),
                    (Some(_), None) => Some(a),
                    (None, None) => Some(b),
                },
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            }
            .filter(|p| p.is_file());

            if let Some(path) = candidate {
                best = Some(path.clone());

                let is_auto_dump = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|v| v.to_string_lossy().contains("script-step-"))
                    .unwrap_or(false);
                if !is_auto_dump {
                    break path;
                }
            }

            if std::time::Instant::now() >= deadline {
                break best.unwrap_or_else(|| bundle_path.to_path_buf());
            }

            std::thread::sleep(std::time::Duration::from_millis(25));
        }
    };
    let bundle_path = bundle_path_for_checks.as_path();

    if let Some(test_id) = check_stale_paint_test_id {
        check_bundle_for_stale_paint(bundle_path, test_id, check_stale_paint_eps)?;
    }
    if let Some(test_id) = check_stale_scene_test_id {
        check_bundle_for_stale_scene(bundle_path, test_id, check_stale_scene_eps)?;
    }
    if let Some(min) = check_idle_no_paint_min {
        check_bundle_for_idle_no_paint_min(bundle_path, out_dir, min, warmup_frames)?;
    }
    if let Some(test_id) = check_pixels_changed_test_id {
        check_out_dir_for_pixels_changed(out_dir, test_id, warmup_frames)?;
    }
    if check_ui_gallery_code_editor_torture_marker_present {
        check_bundle_for_ui_gallery_code_editor_torture_marker_present(bundle_path, warmup_frames)?;
    }
    if check_ui_gallery_code_editor_torture_undo_redo {
        check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_geom_fallbacks_low {
        check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_read_only_blocks_edits {
        check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_read_only_blocks_edits {
        check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_disabled_blocks_edits {
        check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable {
        check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_word_boundary {
        check_bundle_for_ui_gallery_markdown_editor_source_word_boundary(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_web_ime_bridge_enabled {
        check_bundle_for_ui_gallery_web_ime_bridge_enabled(bundle_path, warmup_frames)?;
    }
    if check_ui_gallery_markdown_editor_source_line_boundary_triple_click {
        check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_a11y_composition {
        check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap {
        check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable {
        check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_folds_toggle_stable {
        check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds {
        check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_folds_placeholder_present {
        check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap {
        check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit {
        check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_inlays_toggle_stable {
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable {
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_inlays_present {
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_present(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap {
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit {
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit {
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
    {
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations
    {
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed
    {
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed
    {
        check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed
    {
        check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll {
        check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection {
        check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_folds_placeholder_present {
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap {
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_inlays_present {
        check_bundle_for_ui_gallery_code_editor_torture_inlays_present(bundle_path, warmup_frames)?;
    }
    if check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit {
        check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped {
        check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations {
        check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed {
        check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap {
        check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_word_boundary {
        check_bundle_for_ui_gallery_code_editor_word_boundary(bundle_path, warmup_frames)?;
    }
    if check_ui_gallery_code_editor_a11y_selection {
        check_bundle_for_ui_gallery_code_editor_a11y_selection(bundle_path, warmup_frames)?;
    }
    if check_ui_gallery_code_editor_a11y_composition {
        check_bundle_for_ui_gallery_code_editor_a11y_composition(bundle_path, warmup_frames)?;
    }
    if check_ui_gallery_code_editor_a11y_selection_wrap {
        check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap(bundle_path, warmup_frames)?;
    }
    if check_ui_gallery_code_editor_a11y_composition_wrap {
        check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap(bundle_path, warmup_frames)?;
    }
    if check_ui_gallery_code_editor_a11y_composition_wrap_scroll {
        check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_scroll(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_code_editor_a11y_composition_drag {
        check_bundle_for_ui_gallery_code_editor_a11y_composition_drag(bundle_path, warmup_frames)?;
    }
    if check_semantics_changed_repainted {
        check_bundle_for_semantics_changed_repainted(
            bundle_path,
            warmup_frames,
            dump_semantics_changed_repainted_json,
        )?;
    }
    if let Some(test_id) = check_wheel_scroll_test_id {
        check_bundle_for_wheel_scroll(bundle_path, test_id, warmup_frames)?;
    }
    if let Some(test_id) = check_wheel_scroll_hit_changes_test_id {
        check_bundle_for_wheel_scroll_hit_changes(bundle_path, test_id, warmup_frames)?;
    }
    if let Some(min) = check_prepaint_actions_min {
        check_bundle_for_prepaint_actions_min(bundle_path, out_dir, min, warmup_frames)?;
    }
    if let Some(min) = check_chart_sampling_window_shifts_min {
        check_bundle_for_chart_sampling_window_shifts_min(
            bundle_path,
            out_dir,
            min,
            warmup_frames,
        )?;
    }
    if let Some(min) = check_node_graph_cull_window_shifts_min {
        check_bundle_for_node_graph_cull_window_shifts_min(
            bundle_path,
            out_dir,
            min,
            warmup_frames,
        )?;
    }
    if let Some(max) = check_node_graph_cull_window_shifts_max {
        check_bundle_for_node_graph_cull_window_shifts_max(
            bundle_path,
            out_dir,
            max,
            warmup_frames,
        )?;
    }
    if let Some(min_total_refreshes) = check_vlist_visible_range_refreshes_min {
        check_bundle_for_vlist_visible_range_refreshes_min(
            bundle_path,
            out_dir,
            min_total_refreshes,
            warmup_frames,
        )?;
    }
    if let Some(max_total_refreshes) = check_vlist_visible_range_refreshes_max {
        check_bundle_for_vlist_visible_range_refreshes_max(
            bundle_path,
            out_dir,
            max_total_refreshes,
            warmup_frames,
        )?;
    }
    if check_vlist_window_shifts_explainable {
        check_bundle_for_vlist_window_shifts_explainable(bundle_path, out_dir, warmup_frames)?;
    }
    if check_vlist_window_shifts_have_prepaint_actions {
        check_bundle_for_vlist_window_shifts_have_prepaint_actions(
            bundle_path,
            out_dir,
            warmup_frames,
        )?;
    }
    if let Some(max_total_non_retained_shifts) = check_vlist_window_shifts_non_retained_max {
        check_bundle_for_vlist_window_shifts_non_retained_max(
            bundle_path,
            out_dir,
            max_total_non_retained_shifts,
            warmup_frames,
        )?;
    }
    if let Some(max_total_prefetch_shifts) = check_vlist_window_shifts_prefetch_max {
        check_bundle_for_vlist_window_shifts_kind_max(
            bundle_path,
            out_dir,
            "prefetch",
            max_total_prefetch_shifts,
            warmup_frames,
        )?;
    }
    if let Some(max_total_escape_shifts) = check_vlist_window_shifts_escape_max {
        check_bundle_for_vlist_window_shifts_kind_max(
            bundle_path,
            out_dir,
            "escape",
            max_total_escape_shifts,
            warmup_frames,
        )?;
    }
    if check_vlist_policy_key_stable {
        check_bundle_for_vlist_policy_key_stable(bundle_path, out_dir, warmup_frames)?;
    }
    if let Some(min_total_offset_changes) = check_windowed_rows_offset_changes_min {
        check_bundle_for_windowed_rows_offset_changes_min(
            bundle_path,
            out_dir,
            min_total_offset_changes,
            warmup_frames,
            check_windowed_rows_offset_changes_eps,
        )?;
    }
    if check_windowed_rows_visible_start_changes_repainted {
        check_bundle_for_windowed_rows_visible_start_changes_repainted(
            bundle_path,
            out_dir,
            warmup_frames,
        )?;
    }
    if let Some(min_frames) = check_layout_fast_path_min {
        check_bundle_for_layout_fast_path_min(bundle_path, out_dir, min_frames, warmup_frames)?;
    }
    if let Some(test_id) = check_drag_cache_root_paint_only_test_id {
        check_bundle_for_drag_cache_root_paint_only(bundle_path, test_id, warmup_frames)?;
    }
    if let Some(max_allowed) = check_hover_layout_max {
        let report = bundle_stats_from_path(
            bundle_path,
            1,
            BundleStatsSort::Invalidation,
            BundleStatsOptions { warmup_frames },
        )?;
        check_report_for_hover_layout_invalidations(&report, max_allowed)?;
    }
    if let Some(min) = check_view_cache_reuse_stable_min
        && min > 0
    {
        check_bundle_for_view_cache_reuse_stable_min(bundle_path, out_dir, min, warmup_frames)?;
    }
    if let Some(min) = check_view_cache_reuse_min
        && min > 0
    {
        check_bundle_for_view_cache_reuse_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_overlay_synthesis_min
        && min > 0
    {
        check_bundle_for_overlay_synthesis_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_viewport_input_min
        && min > 0
    {
        check_bundle_for_viewport_input_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_dock_drag_min
        && min > 0
    {
        check_bundle_for_dock_drag_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_viewport_capture_min
        && min > 0
    {
        check_bundle_for_viewport_capture_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_retained_vlist_reconcile_no_notify_min
        && min > 0
    {
        check_bundle_for_retained_vlist_reconcile_no_notify_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(max_delta) = check_retained_vlist_attach_detach_max {
        check_bundle_for_retained_vlist_attach_detach_max(bundle_path, max_delta, warmup_frames)?;
    }
    if let Some(min) = check_retained_vlist_keep_alive_reuse_min
        && min > 0
    {
        check_bundle_for_retained_vlist_keep_alive_reuse_min(bundle_path, min, warmup_frames)?;
    }
    if let Some((min_max_pool_len_after, max_total_evicted_items)) =
        check_retained_vlist_keep_alive_budget
    {
        check_bundle_for_retained_vlist_keep_alive_budget(
            bundle_path,
            min_max_pool_len_after,
            max_total_evicted_items,
            warmup_frames,
        )?;
    }
    if check_gc_sweep_liveness {
        check_bundle_for_gc_sweep_liveness(bundle_path, warmup_frames)?;
    }
    for (file, max) in check_notify_hotspot_file_max {
        check_bundle_for_notify_hotspot_file_max(bundle_path, file.as_str(), *max, warmup_frames)?;
    }
    Ok(())
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
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

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
            "pixels changed check requires screenshots results under {} (set FRET_DIAG_SCREENSHOTS=1 and add capture_screenshot steps): {}",
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

        let bundle_json_path = out_dir.join(&bundle_dir_name).join("bundle.json");
        if !bundle_json_path.is_file() {
            continue;
        }

        let bundle = if let Some(b) = bundles_cache.get(&bundle_dir_name) {
            b.clone()
        } else {
            let bytes = std::fs::read(&bundle_json_path).map_err(|e| e.to_string())?;
            let bundle: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            bundles_cache.insert(bundle_dir_name.clone(), bundle.clone());
            bundle
        };

        let bounds =
            match find_semantics_bounds_for_test_id(&bundle, window, tick_id, frame_id, test_id) {
                Some(r) => r,
                None => match find_semantics_bounds_for_test_id_latest(&bundle, window, test_id) {
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

    let nodes = snap
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())?;

    let node = nodes
        .iter()
        .find(|n| n.get("test_id").and_then(|v| v.as_str()) == Some(test_id))?;

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
    window: u64,
    test_id: &str,
) -> Option<RectF> {
    let windows = bundle.get("windows").and_then(|v| v.as_array())?;
    let w = windows
        .iter()
        .find(|w| w.get("window").and_then(|v| v.as_u64()) == Some(window))?;
    let snaps = w.get("snapshots").and_then(|v| v.as_array())?;

    let snap = snaps.iter().max_by_key(|s| {
        s.get("timestamp_unix_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
    })?;

    let nodes = snap
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())?;

    let node = nodes
        .iter()
        .find(|n| n.get("test_id").and_then(|v| v.as_str()) == Some(test_id))?;

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
mod tests {
    use super::*;
    use crate::compare::compare_bundles_json;
    use crate::stats::{
        bundle_stats_from_json_with_options, check_bundle_for_dock_drag_min_json,
        check_bundle_for_gc_sweep_liveness, check_bundle_for_overlay_synthesis_min_json,
        check_bundle_for_retained_vlist_attach_detach_max_json,
        check_bundle_for_retained_vlist_keep_alive_budget_json,
        check_bundle_for_retained_vlist_reconcile_no_notify_min_json,
        check_bundle_for_semantics_changed_repainted_json, check_bundle_for_stale_scene_json,
        check_bundle_for_ui_gallery_code_editor_a11y_composition_drag_json,
        check_bundle_for_ui_gallery_code_editor_a11y_composition_json,
        check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_json,
        check_bundle_for_ui_gallery_code_editor_a11y_selection_json,
        check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap_json,
        check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json,
        check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json,
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json,
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json,
        check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json,
        check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json,
        check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json,
        check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json,
        check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_json,
        check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap_json,
        check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits_json,
        check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json,
        check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json,
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json,
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json,
        check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click_json,
        check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json,
        check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_json,
        check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json,
        check_bundle_for_ui_gallery_markdown_editor_source_word_boundary_json,
        check_bundle_for_ui_gallery_web_ime_bridge_enabled_json,
        check_bundle_for_view_cache_reuse_min_json, check_bundle_for_viewport_capture_min_json,
        check_bundle_for_viewport_input_min_json, check_bundle_for_vlist_window_shifts_explainable,
        check_bundle_for_wheel_scroll_hit_changes_json,
        check_bundle_for_windowed_rows_offset_changes_min,
        check_bundle_for_windowed_rows_visible_start_changes_repainted_json, json_pointer_set,
        scan_semantics_changed_repainted_json,
    };
    use fret_diag_protocol::{DevtoolsSessionDescriptorV1, DevtoolsSessionListV1};
    use serde_json::json;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn stale_scene_check_fails_when_label_changes_without_scene_change() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "scene_fingerprint": 7,
                            "debug": { "semantics": { "nodes": [
                                { "id": 1, "test_id": "search", "bounds": { "y": 0.0 }, "label": "hello" }
                            ]}}
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "scene_fingerprint": 7,
                            "debug": { "semantics": { "nodes": [
                                { "id": 1, "test_id": "search", "bounds": { "y": 0.0 }, "label": "world" }
                            ]}}
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_stale_scene_json(&bundle, Path::new("bundle.json"), "search", 0.5)
                .unwrap_err();
        assert!(err.contains("stale scene suspected"));
    }

    #[test]
    fn semantics_repaint_check_fails_when_semantics_fingerprint_changes_without_scene_change() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 100,
                            "debug": { "stats": { "paint_nodes_performed": 0, "paint_cache_replayed_ops": 0 } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 101,
                            "debug": { "stats": { "paint_nodes_performed": 0, "paint_cache_replayed_ops": 0 } }
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_semantics_changed_repainted_json(&bundle, Path::new("bundle.json"), 0)
                .unwrap_err();
        assert!(err.contains("missing repaint suspected"));
    }

    #[test]
    fn semantics_repaint_scan_includes_semantics_diff_detail_when_available() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 100,
                            "debug": { "semantics": { "nodes": [
                                { "id": 1, "test_id": "search", "role": "textbox", "label": "hello", "bounds": { "x": 0.0, "y": 0.0, "w": 10.0, "h": 10.0 } }
                            ]}}
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 101,
                            "debug": {
                                "stats": { "paint_nodes_performed": 0, "paint_cache_replayed_ops": 0 },
                                "semantics": { "nodes": [
                                    { "id": 1, "test_id": "search", "role": "textbox", "label": "world", "bounds": { "x": 0.0, "y": 0.0, "w": 10.0, "h": 10.0 } }
                                ]}
                            }
                        }
                    ]
                }
            ]
        });

        let scan = scan_semantics_changed_repainted_json(&bundle, 0);
        assert_eq!(scan.findings.len(), 1);
        assert!(scan.findings[0].get("semantics_diff").is_some());
        assert_eq!(
            scan.findings[0]
                .get("semantics_diff")
                .and_then(|v: &serde_json::Value| v.get("counts"))
                .and_then(|v: &serde_json::Value| v.get("changed"))
                .and_then(|v: &serde_json::Value| v.as_u64()),
            Some(1)
        );
    }

    #[test]
    fn semantics_repaint_check_passes_when_scene_fingerprint_changes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "scene_fingerprint": 7,
                            "semantics_fingerprint": 100
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "scene_fingerprint": 8,
                            "semantics_fingerprint": 101
                        }
                    ]
                }
            ]
        });

        check_bundle_for_semantics_changed_repainted_json(&bundle, Path::new("bundle.json"), 0)
            .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_torture_composed_preedit_a11y_toggle_gate_passes() {
        let out_dir =
            tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_a11y_toggle_pass");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");

        let snapshot = |frame_id: u64, folds: bool, inlays: bool| {
            json!({
                "tick_id": frame_id,
                "frame_id": frame_id,
                "app_snapshot": {
                    "kind": "fret_ui_gallery",
                    "selected_page": "code_editor_torture",
                    "code_editor": {
                        "soft_wrap_cols": 80,
                        "folds_fixture": folds,
                        "inlays_fixture": inlays,
                        "torture": {
                            "preedit_active": true,
                            "allow_decorations_under_inline_preedit": true,
                            "compose_inline_preedit": true
                        }
                    }
                },
                "debug": {
                    "semantics": {
                        "nodes": [
                            {
                                "id": 10,
                                "role": "text_field",
                                "value": "----ab----",
                                "text_selection": [6, 6],
                                "text_composition": [4, 6]
                            },
                            {
                                "id": 11,
                                "parent": 10,
                                "test_id": "ui-gallery-code-editor-torture-viewport"
                            }
                        ]
                    }
                }
            })
        };

        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        snapshot(10, true, true),
                        snapshot(11, false, true),
                        snapshot(12, false, false),
                        snapshot(13, true, true)
                    ]
                }
            ]
        });

        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
            &bundle_path,
            0,
        )
        .unwrap();

        assert!(
            out_dir
                .join("check.ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed.json")
                .is_file()
        );
    }

    #[test]
    fn ui_gallery_code_editor_torture_composed_preedit_a11y_toggle_gate_fails_on_mismatched_preedit_text()
     {
        let out_dir =
            tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_a11y_toggle_fail");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 10,
                            "frame_id": 10,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "folds_fixture": true,
                                    "inlays_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "allow_decorations_under_inline_preedit": true,
                                        "compose_inline_preedit": true
                                    }
                                }
                            },
                            "debug": {
                                "semantics": {
                                    "nodes": [
                                        {
                                            "id": 10,
                                            "role": "text_field",
                                            "value": "----ac----",
                                            "text_selection": [6, 6],
                                            "text_composition": [4, 6]
                                        },
                                        {
                                            "id": 11,
                                            "parent": 10,
                                            "test_id": "ui-gallery-code-editor-torture-viewport"
                                        }
                                    ]
                                }
                            }
                        },
                        {
                            "tick_id": 11,
                            "frame_id": 11,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "folds_fixture": false,
                                    "inlays_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "allow_decorations_under_inline_preedit": true,
                                        "compose_inline_preedit": true
                                    }
                                }
                            },
                            "debug": {
                                "semantics": {
                                    "nodes": [
                                        {
                                            "id": 10,
                                            "role": "text_field",
                                            "value": "----ac----",
                                            "text_selection": [6, 6],
                                            "text_composition": [4, 6]
                                        },
                                        {
                                            "id": 11,
                                            "parent": 10,
                                            "test_id": "ui-gallery-code-editor-torture-viewport"
                                        }
                                    ]
                                }
                            }
                        },
                        {
                            "tick_id": 12,
                            "frame_id": 12,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "folds_fixture": false,
                                    "inlays_fixture": false,
                                    "torture": {
                                        "preedit_active": true,
                                        "allow_decorations_under_inline_preedit": true,
                                        "compose_inline_preedit": true
                                    }
                                }
                            },
                            "debug": {
                                "semantics": {
                                    "nodes": [
                                        {
                                            "id": 10,
                                            "role": "text_field",
                                            "value": "----ac----",
                                            "text_selection": [6, 6],
                                            "text_composition": [4, 6]
                                        },
                                        {
                                            "id": 11,
                                            "parent": 10,
                                            "test_id": "ui-gallery-code-editor-torture-viewport"
                                        }
                                    ]
                                }
                            }
                        },
                        {
                            "tick_id": 13,
                            "frame_id": 13,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "folds_fixture": true,
                                    "inlays_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "allow_decorations_under_inline_preedit": true,
                                        "compose_inline_preedit": true
                                    }
                                }
                            },
                            "debug": {
                                "semantics": {
                                    "nodes": [
                                        {
                                            "id": 10,
                                            "role": "text_field",
                                            "value": "----ac----",
                                            "text_selection": [6, 6],
                                            "text_composition": [4, 6]
                                        },
                                        {
                                            "id": 11,
                                            "parent": 10,
                                            "test_id": "ui-gallery-code-editor-torture-viewport"
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        assert!(
            check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
                &bundle_path,
                0,
            )
            .is_err()
        );
    }

    #[test]
    fn ui_gallery_code_editor_torture_composed_preedit_wheel_gate_passes_when_stable() {
        let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_wheel_pass");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");

        let snapshot = |frame_id: u64, rev: u64| {
            json!({
                "tick_id": frame_id,
                "frame_id": frame_id,
                "app_snapshot": {
                    "kind": "fret_ui_gallery",
                    "selected_page": "code_editor_torture",
                    "code_editor": {
                        "soft_wrap_cols": 80,
                        "torture": {
                            "preedit_active": true,
                            "allow_decorations_under_inline_preedit": true,
                            "compose_inline_preedit": true,
                            "buffer_revision": rev,
                            "text_len_bytes": 123,
                            "selection": { "anchor": 4, "caret": 4 }
                        }
                    }
                },
                "debug": {
                    "semantics": {
                        "nodes": [
                            {
                                "id": 10,
                                "role": "text_field",
                                "value": "zzab",
                                "text_selection": [4, 4],
                                "text_composition": [2, 4]
                            },
                            {
                                "id": 11,
                                "parent": 10,
                                "test_id": "ui-gallery-code-editor-torture-viewport"
                            }
                        ]
                    }
                }
            })
        };

        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "events": [
                        { "kind": "pointer.wheel", "frame_id": 10 }
                    ],
                    "snapshots": [
                        snapshot(9, 1),
                        snapshot(10, 1)
                    ]
                }
            ]
        });

        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();

        assert!(
            out_dir
                .join("check.ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll.json")
                .is_file()
        );
    }

    #[test]
    fn ui_gallery_code_editor_torture_composed_preedit_wheel_gate_fails_when_buffer_changes() {
        let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_wheel_fail");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");

        let snapshot = |frame_id: u64, rev: u64| {
            json!({
                "tick_id": frame_id,
                "frame_id": frame_id,
                "app_snapshot": {
                    "kind": "fret_ui_gallery",
                    "selected_page": "code_editor_torture",
                    "code_editor": {
                        "soft_wrap_cols": 80,
                        "torture": {
                            "preedit_active": true,
                            "allow_decorations_under_inline_preedit": true,
                            "compose_inline_preedit": true,
                            "buffer_revision": rev,
                            "text_len_bytes": 123
                        }
                    }
                },
                "debug": {
                    "semantics": {
                        "nodes": [
                            {
                                "id": 10,
                                "role": "text_field",
                                "value": "zzab",
                                "text_selection": [4, 4],
                                "text_composition": [2, 4]
                            },
                            {
                                "id": 11,
                                "parent": 10,
                                "test_id": "ui-gallery-code-editor-torture-viewport"
                            }
                        ]
                    }
                }
            })
        };

        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "events": [
                        { "kind": "pointer.wheel", "frame_id": 10 }
                    ],
                    "snapshots": [
                        snapshot(9, 1),
                        snapshot(10, 2)
                    ]
                }
            ]
        });

        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json(
                &bundle,
                &bundle_path,
                0,
            )
            .unwrap_err();
        assert!(err.contains("wheel gate failed"));
    }

    #[test]
    fn ui_gallery_code_editor_torture_composed_preedit_drag_select_gate_passes() {
        let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_drag_pass");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");

        let snapshot = |frame_id: u64,
                        preedit_active: bool,
                        rev: u64,
                        anchor: u64,
                        caret: u64,
                        text_composition: serde_json::Value| {
            json!({
                "tick_id": frame_id,
                "frame_id": frame_id,
                "app_snapshot": {
                    "kind": "fret_ui_gallery",
                    "selected_page": "code_editor_torture",
                    "code_editor": {
                        "soft_wrap_cols": 80,
                        "torture": {
                            "preedit_active": preedit_active,
                            "allow_decorations_under_inline_preedit": true,
                            "compose_inline_preedit": true,
                            "buffer_revision": rev,
                            "text_len_bytes": 123,
                            "selection": { "anchor": anchor, "caret": caret }
                        }
                    }
                },
                "debug": {
                    "semantics": {
                        "nodes": [
                            {
                                "id": 10,
                                "role": "text_field",
                                "value": "zzab",
                                "text_selection": [anchor, caret],
                                "text_composition": text_composition
                            },
                            {
                                "id": 11,
                                "parent": 10,
                                "test_id": "ui-gallery-code-editor-torture-viewport"
                            }
                        ]
                    }
                }
            })
        };

        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        snapshot(10, true, 1, 4, 4, json!([2,4])),
                        snapshot(11, false, 1, 0, 4, serde_json::Value::Null)
                    ]
                }
            ]
        });

        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();

        assert!(
            out_dir
                .join("check.ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection.json")
                .is_file()
        );
    }

    #[test]
    fn ui_gallery_code_editor_torture_composed_preedit_drag_select_gate_fails_when_buffer_changes()
    {
        let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_drag_fail");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 10,
                            "frame_id": 10,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "torture": {
                                        "preedit_active": true,
                                        "allow_decorations_under_inline_preedit": true,
                                        "compose_inline_preedit": true,
                                        "buffer_revision": 1,
                                        "text_len_bytes": 123,
                                        "selection": { "anchor": 4, "caret": 4 }
                                    }
                                }
                            },
                            "debug": {
                                "semantics": {
                                    "nodes": [
                                        {
                                            "id": 10,
                                            "role": "text_field",
                                            "value": "zzab",
                                            "text_selection": [4, 4],
                                            "text_composition": [2, 4]
                                        },
                                        {
                                            "id": 11,
                                            "parent": 10,
                                            "test_id": "ui-gallery-code-editor-torture-viewport"
                                        }
                                    ]
                                }
                            }
                        },
                        {
                            "tick_id": 11,
                            "frame_id": 11,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "torture": {
                                        "preedit_active": false,
                                        "allow_decorations_under_inline_preedit": true,
                                        "compose_inline_preedit": true,
                                        "buffer_revision": 2,
                                        "text_len_bytes": 123,
                                        "selection": { "anchor": 0, "caret": 4 }
                                    }
                                }
                            },
                            "debug": {
                                "semantics": {
                                    "nodes": [
                                        {
                                            "id": 10,
                                            "role": "text_field",
                                            "value": "zzab",
                                            "text_selection": [0, 4],
                                            "text_composition": null
                                        },
                                        {
                                            "id": 11,
                                            "parent": 10,
                                            "test_id": "ui-gallery-code-editor-torture-viewport"
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json(
                &bundle,
                &bundle_path,
                0,
            )
            .unwrap_err();
        assert!(err.contains("drag-select gate failed"));
    }

    #[test]
    fn bundle_stats_sums_and_sorts_top_by_invalidation_nodes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "changed_models": [],
                            "changed_globals": [],
                            "debug": { "stats": {
                                "invalidation_walk_calls": 2,
                                "invalidation_walk_nodes": 10,
                                "model_change_invalidation_roots": 1,
                                "global_change_invalidation_roots": 0
                            } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "changed_models": [123],
                            "changed_globals": ["TypeId(0x0)"],
                            "debug": { "stats": {
                                "invalidation_walk_calls": 5,
                                "invalidation_walk_nodes": 7,
                                "model_change_invalidation_roots": 2,
                                "global_change_invalidation_roots": 1
                            } }
                        }
                    ]
                }
            ]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            1,
            BundleStatsSort::Invalidation,
            BundleStatsOptions::default(),
        )
        .unwrap();
        assert_eq!(report.windows, 1);
        assert_eq!(report.snapshots, 2);
        assert_eq!(report.snapshots_with_model_changes, 1);
        assert_eq!(report.snapshots_with_global_changes, 1);
        assert_eq!(report.sum_invalidation_walk_calls, 7);
        assert_eq!(report.sum_invalidation_walk_nodes, 17);
        assert_eq!(report.max_invalidation_walk_calls, 5);
        assert_eq!(report.max_invalidation_walk_nodes, 10);
        assert_eq!(report.top.len(), 1);
        assert_eq!(report.top[0].invalidation_walk_nodes, 10);
        assert_eq!(report.top[0].tick_id, 1);
    }

    #[test]
    fn bundle_stats_extracts_top_invalidation_walks_with_semantics() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "changed_models": [],
                            "changed_globals": [],
                            "debug": {
                                "stats": {
                                    "invalidation_walk_calls": 1,
                                    "invalidation_walk_nodes": 42,
                                    "model_change_invalidation_roots": 0,
                                    "global_change_invalidation_roots": 0
                                },
                                "invalidation_walks": [
                                    { "root_node": 42, "kind": "paint", "source": "other", "walked_nodes": 10 },
                                    { "root_node": 43, "kind": "layout", "source": "other", "walked_nodes": 20, "root_element": 9 }
                                ],
                                "semantics": {
                                    "nodes": [
                                        { "id": 43, "role": "button", "test_id": "todo-add" }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            1,
            BundleStatsSort::Invalidation,
            BundleStatsOptions::default(),
        )
        .unwrap();
        assert_eq!(report.top.len(), 1);
        assert_eq!(report.top[0].top_invalidation_walks.len(), 2);
        assert_eq!(report.top[0].top_invalidation_walks[0].root_node, 43);
        assert_eq!(
            report.top[0].top_invalidation_walks[0]
                .root_test_id
                .as_deref(),
            Some("todo-add")
        );
        assert_eq!(
            report.top[0].top_invalidation_walks[0].root_role.as_deref(),
            Some("button")
        );
        assert_eq!(
            report.top[0].top_invalidation_walks[0].root_element,
            Some(9)
        );
    }

    #[test]
    fn perf_percentile_nearest_rank_is_stable() {
        let values = vec![10u64, 20, 30, 40, 50, 60, 70];
        let mut sorted = values.clone();
        sorted.sort_unstable();
        assert_eq!(percentile_nearest_rank_sorted(&sorted, 0.50), 40);
        assert_eq!(percentile_nearest_rank_sorted(&sorted, 0.95), 70);
        assert_eq!(
            summarize_times_us(&values),
            json!({"min":10,"p50":40,"p95":70,"max":70})
        );
    }

    #[test]
    fn bundle_stats_tracks_hover_declarative_layout_invalidations() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "changed_models": [],
                            "changed_globals": [],
                            "debug": {
                                "stats": {
                                    "invalidation_walk_calls": 1,
                                    "invalidation_walk_nodes": 1,
                                    "model_change_invalidation_roots": 0,
                                    "global_change_invalidation_roots": 0,
                                    "hover_declarative_layout_invalidations": 0
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "changed_models": [],
                            "changed_globals": [],
                            "debug": {
                                "stats": {
                                    "invalidation_walk_calls": 2,
                                    "invalidation_walk_nodes": 10,
                                    "model_change_invalidation_roots": 0,
                                    "global_change_invalidation_roots": 0,
                                    "hover_declarative_layout_invalidations": 2
                                },
                                "hover_declarative_invalidation_hotspots": [
                                    { "node": 43, "layout": 2, "hit_test": 0, "paint": 0 }
                                ],
                                "semantics": {
                                    "nodes": [
                                        { "id": 43, "role": "button", "test_id": "hover-offender" }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = bundle_stats_from_json_with_options(
            &bundle,
            1,
            BundleStatsSort::Invalidation,
            BundleStatsOptions::default(),
        )
        .unwrap();

        assert_eq!(report.sum_hover_layout_invalidations, 2);
        assert_eq!(report.max_hover_layout_invalidations, 2);
        assert_eq!(report.snapshots_with_hover_layout_invalidations, 1);
        assert_eq!(report.top.len(), 1);
        assert_eq!(report.top[0].tick_id, 2);
        assert_eq!(report.top[0].hover_declarative_layout_invalidations, 2);
        assert_eq!(report.top[0].top_hover_declarative_invalidations.len(), 1);
        assert_eq!(
            report.top[0].top_hover_declarative_invalidations[0].node,
            43
        );
        assert_eq!(
            report.top[0].top_hover_declarative_invalidations[0]
                .test_id
                .as_deref(),
            Some("hover-offender")
        );
    }

    #[test]
    fn json_pointer_set_updates_object_field() {
        let mut v = json!({
            "steps": [
                { "type": "click", "target": { "kind": "node_id", "node": 1 } }
            ]
        });
        json_pointer_set(
            &mut v,
            "/steps/0/target",
            json!({"kind":"test_id","id":"x"}),
        )
        .unwrap();
        assert_eq!(v["steps"][0]["target"]["kind"], "test_id");
    }

    #[test]
    fn json_pointer_set_updates_predicate_target() {
        let mut v = json!({
            "steps": [
                { "type": "wait_until", "predicate": { "kind": "exists", "target": { "kind": "node_id", "node": 1 } }, "timeout_frames": 10 }
            ]
        });
        json_pointer_set(
            &mut v,
            "/steps/0/predicate/target",
            json!({"kind":"test_id","id":"open"}),
        )
        .unwrap();
        assert_eq!(v["steps"][0]["predicate"]["target"]["id"], "open");
    }

    #[test]
    fn check_bundle_for_view_cache_reuse_min_counts_reused_cache_roots() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "cache_roots": [
                                    { "root": 1, "reused": true },
                                    { "root": 2, "reused": false }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "cache_roots": [
                                    { "root": 3, "reused": true }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        check_bundle_for_view_cache_reuse_min_json(&bundle, Path::new("bundle.json"), 2, 0)
            .expect("expected reuse>=2");
    }

    #[test]
    fn check_bundle_for_view_cache_reuse_min_respects_warmup_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "cache_roots": [
                                    { "root": 1, "reused": true }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "cache_roots": [
                                    { "root": 2, "reused": true }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_view_cache_reuse_min_json(&bundle, Path::new("bundle.json"), 2, 1)
                .expect_err("expected reuse<2 due to warmup");
        assert!(err.contains("expected at least 2 view-cache reuse events"));
        assert!(err.contains("got 1"));
    }

    #[test]
    fn view_cache_reuse_stable_check_passes_when_tail_streak_meets_min() {
        let out_dir = tmp_out_dir("view_cache_reuse_stable_pass");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 1, "reused": false }] } },
                    { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 2, "reused": true }] } },
                    { "tick_id": 3, "frame_id": 3, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 5 }, "cache_roots": [] } },
                    { "tick_id": 4, "frame_id": 4, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 3, "reused": true }] } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_view_cache_reuse_stable_min(&bundle_path, &out_dir, 3, 0).unwrap();
        assert!(out_dir.join("check.view_cache_reuse_stable.json").is_file());
    }

    #[test]
    fn view_cache_reuse_stable_check_fails_when_tail_streak_is_too_small() {
        let out_dir = tmp_out_dir("view_cache_reuse_stable_fail");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 1, "reused": true }] } },
                    { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 2, "reused": false }] } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_view_cache_reuse_stable_min(&bundle_path, &out_dir, 2, 0).unwrap_err();
        assert!(err.contains("view-cache reuse stable gate failed"));
        assert!(out_dir.join("check.view_cache_reuse_stable.json").is_file());
    }

    #[test]
    fn check_bundle_for_overlay_synthesis_min_counts_synthesized_events() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "overlay_synthesis": [
                                    { "kind": "popover", "id": 101, "source": "cached_declaration", "outcome": "synthesized" },
                                    { "kind": "tooltip", "id": 202, "source": "cached_declaration", "outcome": "suppressed_missing_trigger" }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "overlay_synthesis": [
                                    { "kind": "tooltip", "id": 303, "source": "cached_declaration", "outcome": "synthesized" }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        check_bundle_for_overlay_synthesis_min_json(&bundle, Path::new("bundle.json"), 2, 0)
            .expect("expected synthesized>=2");
    }

    #[test]
    fn check_bundle_for_overlay_synthesis_min_respects_warmup_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "overlay_synthesis": [
                                    { "kind": "tooltip", "id": 1, "source": "cached_declaration", "outcome": "synthesized" }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "stats": { "view_cache_active": true },
                                "overlay_synthesis": [
                                    { "kind": "hover", "id": 2, "source": "cached_declaration", "outcome": "suppressed_trigger_not_live_in_current_frame" }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_overlay_synthesis_min_json(&bundle, Path::new("bundle.json"), 1, 1)
                .expect_err("expected synthesized<1 due to warmup");
        assert!(err.contains("expected at least 1 overlay synthesis events"));
        assert!(err.contains("got 0"));
        assert!(err.contains("suppressions=["));
    }

    #[test]
    fn check_bundle_for_retained_vlist_reconcile_no_notify_min_passes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "stats": { "retained_virtual_list_reconciles": 1 },
                            "dirty_views": [{ "root_node": 1, "source": "notify" }]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "stats": { "retained_virtual_list_reconciles": 2 },
                            "retained_virtual_list_reconciles": [
                                { "node": 10, "element": 20, "prev_items": 1, "next_items": 2, "preserved_items": 1, "attached_items": 1, "detached_items": 0 },
                                { "node": 11, "element": 21, "prev_items": 2, "next_items": 3, "preserved_items": 2, "attached_items": 1, "detached_items": 0 }
                            ],
                            "dirty_views": []
                        }
                    }
                ]
            }]
        });

        check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
            &bundle,
            Path::new("bundle.json"),
            1,
            1,
        )
        .expect("expected reconcile>=1 without notify dirtiness");
    }

    #[test]
    fn check_bundle_for_retained_vlist_reconcile_no_notify_min_fails_on_notify_dirty_view() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 1,
                    "debug": {
                        "stats": { "retained_virtual_list_reconciles": 1 },
                        "dirty_views": [
                            { "root_node": 123, "source": "notify", "detail": "notify_call" }
                        ]
                    }
                }]
            }]
        });

        let err = check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
            &bundle,
            Path::new("bundle.json"),
            1,
            0,
        )
        .expect_err("expected notify offenders");
        assert!(err.contains(
            "retained virtual-list reconcile should not require notify-based dirty views"
        ));
        assert!(err.contains("source=notify"));
    }

    #[test]
    fn check_bundle_for_retained_vlist_reconcile_no_notify_min_fails_when_missing_reconciles() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": { "stats": { "retained_virtual_list_reconciles": 0 } }
                }]
            }]
        });

        let err = check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
            &bundle,
            Path::new("bundle.json"),
            1,
            0,
        )
        .expect_err("expected missing reconcile events");
        assert!(err.contains("expected at least 1 retained virtual-list reconcile events"));
        assert!(err.contains("got 0"));
    }

    #[test]
    fn check_bundle_for_retained_vlist_attach_detach_max_passes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": {
                        "stats": {
                            "retained_virtual_list_reconciles": 1,
                            "retained_virtual_list_attached_items": 12,
                            "retained_virtual_list_detached_items": 13
                        },
                        "retained_virtual_list_reconciles": [
                            { "node": 10, "element": 20, "prev_items": 1, "next_items": 2, "preserved_items": 1, "attached_items": 12, "detached_items": 13 }
                        ]
                    }
                }]
            }]
        });

        check_bundle_for_retained_vlist_attach_detach_max_json(
            &bundle,
            Path::new("bundle.json"),
            25,
            0,
        )
        .expect("expected delta<=25");
    }

    #[test]
    fn check_bundle_for_retained_vlist_attach_detach_max_fails_when_exceeded() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": {
                        "stats": {
                            "retained_virtual_list_reconciles": 1,
                            "retained_virtual_list_attached_items": 20,
                            "retained_virtual_list_detached_items": 21
                        }
                    }
                }]
            }]
        });

        let err = check_bundle_for_retained_vlist_attach_detach_max_json(
            &bundle,
            Path::new("bundle.json"),
            40,
            0,
        )
        .expect_err("expected delta>40 to fail");
        assert!(err.contains("attach/detach delta exceeded"));
        assert!(err.contains("delta=41"));
    }

    #[test]
    fn check_bundle_for_retained_vlist_attach_detach_max_fails_when_missing_reconciles() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": { "stats": { "retained_virtual_list_reconciles": 0 } }
                }]
            }]
        });

        let err = check_bundle_for_retained_vlist_attach_detach_max_json(
            &bundle,
            Path::new("bundle.json"),
            10,
            0,
        )
        .expect_err("expected missing reconcile events");
        assert!(err.contains("expected at least 1 retained virtual-list reconcile event"));
    }

    #[test]
    fn check_bundle_for_retained_vlist_keep_alive_budget_passes() {
        let out_dir = tmp_out_dir("retained_vlist_keep_alive_budget_pass");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": {
                        "retained_virtual_list_reconciles": [
                            { "keep_alive_pool_len_after": 128, "evicted_keep_alive_items": 0 }
                        ]
                    }
                }]
            }]
        });

        check_bundle_for_retained_vlist_keep_alive_budget_json(&bundle, &bundle_path, 1, 0, 0)
            .expect("expected keep-alive budget to pass");
        assert!(
            out_dir
                .join("check.retained_vlist_keep_alive_budget.json")
                .is_file()
        );
    }

    #[test]
    fn check_bundle_for_retained_vlist_keep_alive_budget_fails_when_evicted() {
        let out_dir = tmp_out_dir("retained_vlist_keep_alive_budget_fail_evicted");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "debug": {
                        "retained_virtual_list_reconciles": [
                            { "keep_alive_pool_len_after": 64, "evicted_keep_alive_items": 1 }
                        ]
                    }
                }]
            }]
        });

        let err =
            check_bundle_for_retained_vlist_keep_alive_budget_json(&bundle, &bundle_path, 1, 0, 0)
                .expect_err("expected eviction budget to fail");
        assert!(err.contains("keep-alive budget violated"));
        assert!(err.contains("total_evicted_items=1"));
    }

    #[test]
    fn check_bundle_for_viewport_input_min_counts_events() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "viewport_input": [
                                    { "target": 1, "pointer_id": 0, "pointer_type": "mouse", "cursor_px": {"x": 1.0, "y": 2.0}, "uv": [0.0, 0.0], "target_px": [0, 0], "kind": { "type": "pointer_down", "button": "left", "modifiers": {}, "click_count": 1 } }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "viewport_input": [
                                    { "target": 1, "pointer_id": 0, "pointer_type": "mouse", "cursor_px": {"x": 2.0, "y": 3.0}, "uv": [0.1, 0.1], "target_px": [10, 10], "kind": { "type": "pointer_move", "buttons": {"left": true, "right": false, "middle": false}, "modifiers": {} } }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        check_bundle_for_viewport_input_min_json(&bundle, Path::new("bundle.json"), 2, 0)
            .expect("expected viewport_input>=2");
    }

    #[test]
    fn check_bundle_for_viewport_input_min_respects_warmup_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "viewport_input": [
                                    { "target": 1, "pointer_id": 0, "pointer_type": "mouse", "cursor_px": {"x": 1.0, "y": 2.0}, "uv": [0.0, 0.0], "target_px": [0, 0], "kind": { "type": "pointer_down", "button": "left", "modifiers": {}, "click_count": 1 } }
                                ]
                            }
                        },
                        {
                            "frame_id": 1,
                            "debug": {
                                "viewport_input": []
                            }
                        }
                    ]
                }
            ]
        });

        let err = check_bundle_for_viewport_input_min_json(&bundle, Path::new("bundle.json"), 1, 1)
            .expect_err("expected viewport input < 1 due to warmup");
        assert!(err.contains("expected at least 1 viewport input events"));
        assert!(err.contains("got 0"));
    }

    #[test]
    fn check_bundle_for_dock_drag_min_counts_active_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "docking_interaction": {
                                    "dock_drag": { "pointer_id": 0, "source_window": 1, "current_window": 1, "dragging": true, "cross_window_hover": false },
                                    "viewport_capture": null
                                }
                            }
                        }
                    ]
                }
            ]
        });

        check_bundle_for_dock_drag_min_json(&bundle, Path::new("bundle.json"), 1, 0)
            .expect("expected dock_drag>=1");
    }

    #[test]
    fn check_bundle_for_viewport_capture_min_respects_warmup_frames() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 0,
                            "debug": {
                                "docking_interaction": {
                                    "dock_drag": null,
                                    "viewport_capture": { "pointer_id": 0, "target": 2 }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let err =
            check_bundle_for_viewport_capture_min_json(&bundle, Path::new("bundle.json"), 1, 1)
                .expect_err("expected viewport_capture<1 due to warmup");
        assert!(err.contains("expected at least 1 snapshots with an active viewport capture"));
        assert!(err.contains("got 0"));
    }

    #[test]
    fn compare_bundles_passes_when_test_id_semantics_match() {
        let a = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "scene_fingerprint": 42,
                    "debug": {
                        "semantics": {
                            "roots": [{ "root": 1, "visible": true, "blocks_underlay_input": false, "hit_testable": true, "z_index": 0 }],
                            "nodes": [{
                                "id": 1,
                                "role": "button",
                                "bounds": { "x": 1.0, "y": 2.0, "w": 3.0, "h": 4.0 },
                                "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                "actions": { "focus": true, "invoke": true, "set_value": false, "set_text_selection": false },
                                "test_id": "ok"
                            }]
                        }
                    }
                }]
            }]
        });
        let b = a.clone();
        let report = compare_bundles_json(
            &a,
            Path::new("a/bundle.json"),
            &b,
            Path::new("b/bundle.json"),
            CompareOptions {
                warmup_frames: 0,
                eps_px: 0.5,
                ignore_bounds: false,
                ignore_scene_fingerprint: false,
            },
        )
        .unwrap();
        assert!(report.ok);
        assert!(report.diffs.is_empty());
    }

    #[test]
    fn compare_bundles_reports_role_mismatch_for_test_id() {
        let a = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "scene_fingerprint": 42,
                    "debug": {
                        "semantics": {
                            "roots": [{ "root": 1, "visible": true, "blocks_underlay_input": false, "hit_testable": true, "z_index": 0 }],
                            "nodes": [{
                                "id": 1,
                                "role": "button",
                                "bounds": { "x": 1.0, "y": 2.0, "w": 3.0, "h": 4.0 },
                                "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                "actions": { "focus": true, "invoke": true, "set_value": false, "set_text_selection": false },
                                "test_id": "t"
                            }]
                        }
                    }
                }]
            }]
        });
        let mut b = a.clone();
        b["windows"][0]["snapshots"][0]["debug"]["semantics"]["nodes"][0]["role"] =
            serde_json::Value::from("menuitem");

        let report = compare_bundles_json(
            &a,
            Path::new("a/bundle.json"),
            &b,
            Path::new("b/bundle.json"),
            CompareOptions {
                warmup_frames: 0,
                eps_px: 0.5,
                ignore_bounds: false,
                ignore_scene_fingerprint: false,
            },
        )
        .unwrap();
        assert!(!report.ok);
        assert!(report.diffs.iter().any(|d| d.kind == "node_field_mismatch"
            && d.key.as_deref() == Some("t")
            && d.field == Some("role")));
    }

    fn tmp_out_dir(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fretboard_test_{label}_pid{}_{}",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn layout_fast_path_min_check_passes() {
        let out_dir = tmp_out_dir("layout_fast_path_min_pass");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "frame_id": 0, "debug": { "stats": { "layout_fast_path_taken": false } } },
                    { "frame_id": 1, "debug": { "stats": { "layout_fast_path_taken": true } } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_layout_fast_path_min(&bundle_path, &out_dir, 1, 0)
            .expect("expected layout fast-path >= 1");
        assert!(out_dir.join("check.layout_fast_path_min.json").is_file());
    }

    #[test]
    fn layout_fast_path_min_check_fails_when_missing() {
        let out_dir = tmp_out_dir("layout_fast_path_min_fail");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "frame_id": 0, "debug": { "stats": { "layout_fast_path_taken": false } } },
                    { "frame_id": 1, "debug": { "stats": { "layout_fast_path_taken": false } } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_layout_fast_path_min(&bundle_path, &out_dir, 1, 0)
            .expect_err("expected fast-path < 1");
        assert!(err.contains("layout fast-path gate failed"));
        assert!(out_dir.join("check.layout_fast_path_min.json").is_file());
    }

    #[test]
    fn vlist_policy_key_stable_check_passes() {
        let out_dir = tmp_out_dir("vlist_policy_key_stable_pass");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 1,
                        "debug": { "virtual_list_windows": [{ "node": 10, "element": 20, "policy_key": 7 }] }
                    },
                    {
                        "frame_id": 2,
                        "debug": { "virtual_list_windows": [{ "node": 10, "element": 20, "policy_key": 7 }] }
                    }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_vlist_policy_key_stable(&bundle_path, &out_dir, 0)
            .expect("expected stable vlist policy_key");
        assert!(out_dir.join("check.vlist_policy_key_stable.json").is_file());
    }

    #[test]
    fn vlist_policy_key_stable_check_fails_when_changed() {
        let out_dir = tmp_out_dir("vlist_policy_key_stable_fail");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 1,
                        "debug": { "virtual_list_windows": [{ "node": 10, "element": 20, "policy_key": 7 }] }
                    },
                    {
                        "frame_id": 2,
                        "debug": { "virtual_list_windows": [{ "node": 10, "element": 20, "policy_key": 9 }] }
                    }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_vlist_policy_key_stable(&bundle_path, &out_dir, 0)
            .expect_err("expected unstable vlist policy_key");
        assert!(err.contains("vlist policy-key stability gate failed"));
        assert!(out_dir.join("check.vlist_policy_key_stable.json").is_file());
    }

    #[test]
    fn windowed_rows_offset_changes_min_check_passes() {
        let out_dir = tmp_out_dir("windowed_rows_offset_changes_min_pass");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "frame_id": 0, "tick_id": 0, "debug": { "scroll_handle_changes": [], "windowed_rows_surfaces": [] } },
                    {
                        "frame_id": 1,
                        "tick_id": 1,
                        "debug": {
                            "scroll_handle_changes": [{ "offset_changed": true }],
                            "windowed_rows_surfaces": [
                                {
                                    "callsite_id": 7,
                                    "offset_y": 0.0,
                                    "location": { "file": "x.rs", "line": 1, "column": 1 }
                                }
                            ]
                        }
                    },
                    {
                        "frame_id": 2,
                        "tick_id": 2,
                        "debug": {
                            "scroll_handle_changes": [{ "offset_changed": true }],
                            "windowed_rows_surfaces": [
                                {
                                    "callsite_id": 7,
                                    "offset_y": 10.0,
                                    "location": { "file": "x.rs", "line": 1, "column": 1 }
                                }
                            ]
                        }
                    }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_windowed_rows_offset_changes_min(&bundle_path, &out_dir, 1, 0, 0.5)
            .expect("expected windowed rows offset changes >= 1");
        assert!(
            out_dir
                .join("check.windowed_rows_offset_changes_min.json")
                .is_file()
        );
    }

    #[test]
    fn windowed_rows_offset_changes_min_check_fails_when_offset_is_stable() {
        let out_dir = tmp_out_dir("windowed_rows_offset_changes_min_fail");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 1,
                        "tick_id": 1,
                        "debug": {
                            "scroll_handle_changes": [{ "offset_changed": true }],
                            "windowed_rows_surfaces": [
                                {
                                    "callsite_id": 7,
                                    "offset_y": 0.0,
                                    "location": { "file": "x.rs", "line": 1, "column": 1 }
                                }
                            ]
                        }
                    },
                    {
                        "frame_id": 2,
                        "tick_id": 2,
                        "debug": {
                            "scroll_handle_changes": [{ "offset_changed": true }],
                            "windowed_rows_surfaces": [
                                {
                                    "callsite_id": 7,
                                    "offset_y": 0.0,
                                    "location": { "file": "x.rs", "line": 1, "column": 1 }
                                }
                            ]
                        }
                    }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_windowed_rows_offset_changes_min(&bundle_path, &out_dir, 1, 0, 0.5)
                .expect_err("expected offset changes < 1");
        assert!(err.contains("total_offset_changes"));
        assert!(
            out_dir
                .join("check.windowed_rows_offset_changes_min.json")
                .is_file()
        );
    }

    #[test]
    fn windowed_rows_visible_start_repaint_gate_passes_when_scene_fingerprint_changes() {
        let out_dir = tmp_out_dir("windowed_rows_visible_start_repaint_pass");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 1,
                        "tick_id": 1,
                        "scene_fingerprint": 1,
                        "debug": {
                            "scroll_handle_changes": [{ "offset_changed": true }],
                            "windowed_rows_surfaces": [
                                {
                                    "callsite_id": 7,
                                    "offset_y": 0.0,
                                    "visible_start": 0,
                                    "location": { "file": "x.rs", "line": 1, "column": 1 }
                                }
                            ]
                        }
                    },
                    {
                        "frame_id": 2,
                        "tick_id": 2,
                        "scene_fingerprint": 2,
                        "debug": {
                            "scroll_handle_changes": [{ "offset_changed": true }],
                            "windowed_rows_surfaces": [
                                {
                                    "callsite_id": 7,
                                    "offset_y": 10.0,
                                    "visible_start": 10,
                                    "location": { "file": "x.rs", "line": 1, "column": 1 }
                                }
                            ]
                        }
                    }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
            &bundle,
            &bundle_path,
            &out_dir,
            0,
        )
        .expect("expected repaint on visible_start changes");
        assert!(
            out_dir
                .join("check.windowed_rows_visible_start_changes_repainted.json")
                .is_file()
        );
    }

    #[test]
    fn windowed_rows_visible_start_repaint_gate_fails_when_scene_fingerprint_is_stale() {
        let out_dir = tmp_out_dir("windowed_rows_visible_start_repaint_fail");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 1,
                        "tick_id": 1,
                        "scene_fingerprint": 1,
                        "debug": {
                            "scroll_handle_changes": [{ "offset_changed": true }],
                            "windowed_rows_surfaces": [
                                {
                                    "callsite_id": 7,
                                    "offset_y": 0.0,
                                    "visible_start": 0,
                                    "location": { "file": "x.rs", "line": 1, "column": 1 }
                                }
                            ]
                        }
                    },
                    {
                        "frame_id": 2,
                        "tick_id": 2,
                        "scene_fingerprint": 1,
                        "debug": {
                            "scroll_handle_changes": [{ "offset_changed": true }],
                            "windowed_rows_surfaces": [
                                {
                                    "callsite_id": 7,
                                    "offset_y": 10.0,
                                    "visible_start": 10,
                                    "location": { "file": "x.rs", "line": 1, "column": 1 }
                                }
                            ]
                        }
                    }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
            &bundle,
            &bundle_path,
            &out_dir,
            0,
        )
        .expect_err("expected stale fingerprint failure");
        assert!(err.contains("windowed rows repaint gate failed"));
        assert!(
            out_dir
                .join("check.windowed_rows_visible_start_changes_repainted.json")
                .is_file()
        );
    }

    #[test]
    fn wheel_scroll_hit_changes_check_passes_when_offset_changes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "events": [{ "kind": "pointer.wheel", "frame_id": 1 }],
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "hit_test": { "hit": 2 },
                            "semantics": { "nodes": [
                                { "id": 1, "test_id": "root" },
                                { "id": 2, "parent": 1 }
                            ]},
                            "virtual_list_windows": [{ "offset": 0.0 }]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "hit_test": { "hit": 2 },
                            "semantics": { "nodes": [
                                { "id": 1, "test_id": "root" },
                                { "id": 2, "parent": 1 }
                            ]},
                            "virtual_list_windows": [{ "offset": 12.0 }]
                        }
                    }
                ]
            }]
        });

        check_bundle_for_wheel_scroll_hit_changes_json(
            &bundle,
            Path::new("bundle.json"),
            "root",
            0,
        )
        .expect("expected wheel scroll to change offset");
    }

    #[test]
    fn wheel_scroll_hit_changes_check_fails_when_hit_and_offset_are_stable() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "events": [{ "kind": "pointer.wheel", "frame_id": 1 }],
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "hit_test": { "hit": 2 },
                            "semantics": { "nodes": [
                                { "id": 1, "test_id": "root" },
                                { "id": 2, "parent": 1 }
                            ]},
                            "virtual_list_windows": [{ "offset": 0.0 }]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "hit_test": { "hit": 2 },
                            "semantics": { "nodes": [
                                { "id": 1, "test_id": "root" },
                                { "id": 2, "parent": 1 }
                            ]},
                            "virtual_list_windows": [{ "offset": 0.0 }]
                        }
                    }
                ]
            }]
        });

        let err = check_bundle_for_wheel_scroll_hit_changes_json(
            &bundle,
            Path::new("bundle.json"),
            "root",
            0,
        )
        .expect_err("expected wheel scroll check to fail when stable");
        assert!(err.contains("wheel scroll hit-change check failed"));
        assert!(err.contains("error=hit_did_not_change"));
    }

    fn write_png_solid(path: &std::path::Path, w: u32, h: u32, rgba: [u8; 4]) {
        let _ = std::fs::create_dir_all(
            path.parent()
                .expect("png output must have a parent directory"),
        );
        let mut img = image::RgbaImage::new(w, h);
        for p in img.pixels_mut() {
            *p = image::Rgba(rgba);
        }
        img.save(path).expect("png save should succeed");
    }

    fn write_bundle_with_bounds(
        out_dir: &std::path::Path,
        bundle_dir_name: &str,
        window: u64,
        tick_id: u64,
        frame_id: u64,
        test_id: &str,
        bounds: RectF,
    ) {
        let path = out_dir.join(bundle_dir_name).join("bundle.json");
        let _ = std::fs::create_dir_all(
            path.parent()
                .expect("bundle output must have a parent directory"),
        );

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": window,
                "snapshots": [{
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "debug": {
                        "semantics": { "nodes": [{
                            "id": 1,
                            "test_id": test_id,
                            "bounds": { "x": bounds.x, "y": bounds.y, "w": bounds.w, "h": bounds.h }
                        }]}
                    }
                }]
            }]
        });

        std::fs::write(&path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");
    }

    #[test]
    fn gc_sweep_liveness_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("gc_sweep_liveness_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "removed_subtrees": [{
                            "root": 10,
                            "unreachable_from_liveness_roots": false,
                            "reachable_from_layer_roots": true,
                            "reachable_from_view_cache_roots": true,
                            "root_layer_visible": true,
                            "liveness_layer_roots_len": 2,
                            "view_cache_reuse_roots_len": 1,
                            "view_cache_reuse_root_nodes_len": 1,
                            "root_element_path": "root[demo].overlay",
                            "trigger_element_path": "root[demo].trigger"
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_gc_sweep_liveness(&bundle_path, 0).unwrap_err();
        assert!(err.contains("GC sweep liveness violation"));

        let evidence_path = bundle_dir.join("check.gc_sweep_liveness.json");
        assert!(
            evidence_path.is_file(),
            "expected gc sweep liveness evidence JSON to be written"
        );

        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("gc_sweep_liveness")
        );
        assert_eq!(
            evidence
                .get("removed_subtrees_offenders")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert!(
            evidence
                .get("offender_samples")
                .and_then(|v| v.as_array())
                .is_some_and(|a| !a.is_empty()),
            "expected offender_samples to be populated"
        );
    }

    #[test]
    fn gc_sweep_liveness_fails_on_keep_alive_mismatch_under_reuse() {
        let out_dir = tmp_out_dir("gc_sweep_liveness_keep_alive_mismatch");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "removed_subtrees": [{
                            "root": 10,
                            "unreachable_from_liveness_roots": true,
                            "reachable_from_layer_roots": false,
                            "reachable_from_view_cache_roots": false,
                            "root_layer_visible": false,
                            "view_cache_reuse_roots_len": 1,
                            "trigger_element_in_view_cache_keep_alive": true,
                            "root_element_path": "root[demo].overlay",
                            "trigger_element_path": "root[demo].trigger"
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_gc_sweep_liveness(&bundle_path, 0).unwrap_err();
        assert!(err.contains("GC sweep liveness violation"));

        let evidence_path = bundle_dir.join("check.gc_sweep_liveness.json");
        assert!(
            evidence_path.is_file(),
            "expected gc sweep liveness evidence JSON to be written"
        );

        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("gc_sweep_liveness")
        );
        assert!(
            evidence
                .get("offender_taxonomy_counts")
                .and_then(|v| v.get("keep_alive_liveness_mismatch"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0,
            "expected keep_alive_liveness_mismatch to be counted"
        );
    }

    #[test]
    fn notify_hotspots_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("notify_hotspots_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "notify_requests": [{
                            "frame_id": 1,
                            "caller_node": 100,
                            "target_view": 200,
                            "file": "crates/fret-ui/src/declarative/host_widget/event/pressable.rs",
                            "line": 123,
                            "column": 9
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_notify_hotspot_file_max(&bundle_path, "pressable.rs", 0, 0)
            .unwrap_err();
        assert!(err.contains("notify hotspot file budget exceeded"));

        let evidence_path = bundle_dir.join("check.notify_hotspots.json");
        assert!(
            evidence_path.is_file(),
            "expected notify hotspots evidence JSON to be written"
        );
        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("notify_hotspots")
        );
    }

    #[test]
    fn gc_sweep_liveness_fails_on_unmapped_view_cache_reuse_roots() {
        let out_dir = tmp_out_dir("gc_sweep_liveness_reuse_roots_unmapped");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "removed_subtrees": [{
                            "root": 10,
                            "unreachable_from_liveness_roots": true,
                            "reachable_from_layer_roots": false,
                            "reachable_from_view_cache_roots": false,
                            "root_layer_visible": false,
                            "view_cache_reuse_roots_len": 1,
                            "view_cache_reuse_root_nodes_len": 0,
                            "root_element_path": "root[demo].overlay",
                            "trigger_element_path": "root[demo].trigger"
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_gc_sweep_liveness(&bundle_path, 0).unwrap_err();
        assert!(err.contains("GC sweep liveness violation"));

        let evidence_path = bundle_dir.join("check.gc_sweep_liveness.json");
        assert!(
            evidence_path.is_file(),
            "expected gc sweep liveness evidence JSON to be written"
        );

        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("gc_sweep_liveness")
        );
        assert!(
            evidence
                .get("offender_taxonomy_counts")
                .and_then(|v| v.get("reuse_roots_unmapped"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0,
            "expected reuse_roots_unmapped to be counted"
        );
    }

    #[test]
    fn vlist_window_shifts_explainable_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("vlist_window_shifts_explainable_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "virtual_list_windows": [{
                            "node": 10,
                            "element": 1,
                            "window_mismatch": true,
                            "window_shift_kind": "escape"
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_vlist_window_shifts_explainable(&bundle_path, &bundle_dir, 0)
            .unwrap_err();
        assert!(err.contains("vlist window-shift explainability gate failed"));

        let evidence_path = bundle_dir.join("check.vlist_window_shifts_explainable.json");
        assert!(
            evidence_path.is_file(),
            "expected vlist window-shift evidence JSON to be written"
        );

        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("vlist_window_shifts_explainable")
        );
        assert_eq!(evidence.get("offenders").and_then(|v| v.as_u64()), Some(1));
        assert!(
            evidence
                .get("samples")
                .and_then(|v| v.as_array())
                .is_some_and(|a| !a.is_empty()),
            "expected samples to be populated"
        );
    }

    #[test]
    fn vlist_window_shifts_non_retained_max_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("vlist_window_shifts_non_retained_max_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "stats": {
                            "virtual_list_window_shifts_total": 1,
                            "virtual_list_window_shifts_non_retained": 1
                        },
                        "virtual_list_window_shift_samples": [{
                            "frame_id": 1,
                            "source": "prepaint",
                            "node": 10,
                            "element": 1,
                            "window_shift_kind": "escape",
                            "window_shift_reason": "scroll_offset",
                            "window_shift_apply_mode": "non_retained_rerender",
                            "window_shift_invalidation_detail": "scroll_handle_escape_window_update"
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_vlist_window_shifts_non_retained_max(&bundle_path, &bundle_dir, 0, 0)
                .unwrap_err();
        assert!(err.contains("vlist non-retained window-shift gate failed"));

        let evidence_path = bundle_dir.join("check.vlist_window_shifts_non_retained_max.json");
        assert!(
            evidence_path.is_file(),
            "expected vlist non-retained window-shift evidence JSON to be written"
        );

        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("vlist_window_shifts_non_retained_max")
        );
        assert_eq!(
            evidence
                .get("total_non_retained_shifts")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
    }

    #[test]
    fn vlist_window_shifts_kind_max_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("vlist_window_shifts_kind_max_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "virtual_list_windows": [{
                            "source": "prepaint",
                            "node": 10,
                            "element": 1,
                            "window_mismatch": false,
                            "window_shift_kind": "prefetch",
                            "window_shift_reason": "scroll_offset",
                            "window_shift_apply_mode": "non_retained_rerender"
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_vlist_window_shifts_kind_max(
            &bundle_path,
            &bundle_dir,
            "prefetch",
            0,
            0,
        )
        .unwrap_err();
        assert!(err.contains("vlist window-shift kind gate failed"));

        let evidence_path = bundle_dir.join("check.vlist_window_shifts_prefetch_max.json");
        assert!(
            evidence_path.is_file(),
            "expected vlist window-shift kind evidence JSON to be written"
        );

        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("vlist_window_shifts_prefetch_max")
        );
        assert_eq!(
            evidence.get("total_kind_shifts").and_then(|v| v.as_u64()),
            Some(1)
        );
    }

    #[test]
    fn vlist_window_shifts_explainable_accepts_viewport_resize_detail() {
        let out_dir = tmp_out_dir("vlist_window_shifts_explainable_viewport_resize");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "virtual_list_windows": [{
                            "node": 10,
                            "element": 1,
                            "window_mismatch": true,
                            "window_shift_kind": "escape",
                            "window_shift_reason": "viewport_resize",
                            "window_shift_apply_mode": "non_retained_rerender",
                            "window_shift_invalidation_detail": "scroll_handle_viewport_resize_window_update"
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_vlist_window_shifts_explainable(&bundle_path, &bundle_dir, 0)
            .expect("expected gate to accept viewport resize mapping");
    }

    #[test]
    fn vlist_window_shifts_explainable_accepts_items_revision_detail() {
        let out_dir = tmp_out_dir("vlist_window_shifts_explainable_items_revision");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "virtual_list_windows": [{
                            "node": 10,
                            "element": 1,
                            "window_mismatch": true,
                            "window_shift_kind": "escape",
                            "window_shift_reason": "items_revision",
                            "window_shift_apply_mode": "non_retained_rerender",
                            "window_shift_invalidation_detail": "scroll_handle_items_revision_window_update"
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_vlist_window_shifts_explainable(&bundle_path, &bundle_dir, 0)
            .expect("expected gate to accept items revision mapping");
    }

    #[test]
    fn prepaint_actions_min_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("prepaint_actions_min_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "prepaint_actions": []
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_prepaint_actions_min(&bundle_path, &bundle_dir, 1, 0).unwrap_err();
        assert!(err.contains("prepaint actions"));

        let evidence_path = bundle_dir.join("check.prepaint_actions_min.json");
        assert!(
            evidence_path.is_file(),
            "expected prepaint actions min evidence JSON to be written"
        );
        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("prepaint_actions_min")
        );
    }

    #[test]
    fn chart_sampling_window_shifts_min_accepts_matching_action() {
        let out_dir = tmp_out_dir("chart_sampling_window_shifts_min_ok");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "prepaint_actions": [{
                            "node": 10,
                            "kind": "chart_sampling_window_shift",
                            "chart_sampling_window_key": 123
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_chart_sampling_window_shifts_min(&bundle_path, &bundle_dir, 1, 0)
            .expect("expected gate to accept chart sampling action");

        let evidence_path = bundle_dir.join("check.chart_sampling_window_shifts_min.json");
        assert!(
            evidence_path.is_file(),
            "expected chart sampling window shifts evidence JSON to be written"
        );
    }

    #[test]
    fn chart_sampling_window_shifts_min_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("chart_sampling_window_shifts_min_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "prepaint_actions": []
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_chart_sampling_window_shifts_min(&bundle_path, &bundle_dir, 1, 0)
                .unwrap_err();
        assert!(err.contains("chart sampling window shift"));

        let evidence_path = bundle_dir.join("check.chart_sampling_window_shifts_min.json");
        assert!(
            evidence_path.is_file(),
            "expected chart sampling window shifts evidence JSON to be written"
        );
        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("chart_sampling_window_shifts_min")
        );
    }

    #[test]
    fn node_graph_cull_window_shifts_min_accepts_matching_action() {
        let out_dir = tmp_out_dir("node_graph_cull_window_shifts_min_ok");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "prepaint_actions": [{
                            "node": 10,
                            "kind": "node_graph_cull_window_shift",
                            "node_graph_cull_window_key": 456
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_node_graph_cull_window_shifts_min(&bundle_path, &bundle_dir, 1, 0)
            .expect("expected gate to accept node graph cull action");

        let evidence_path = bundle_dir.join("check.node_graph_cull_window_shifts_min.json");
        assert!(
            evidence_path.is_file(),
            "expected node graph cull window shifts evidence JSON to be written"
        );
    }

    #[test]
    fn node_graph_cull_window_shifts_min_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("node_graph_cull_window_shifts_min_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "prepaint_actions": []
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_node_graph_cull_window_shifts_min(&bundle_path, &bundle_dir, 1, 0)
                .unwrap_err();
        assert!(err.contains("node graph cull window shift"));

        let evidence_path = bundle_dir.join("check.node_graph_cull_window_shifts_min.json");
        assert!(
            evidence_path.is_file(),
            "expected node graph cull window shifts evidence JSON to be written"
        );
        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("node_graph_cull_window_shifts_min")
        );
    }

    #[test]
    fn node_graph_cull_window_shifts_max_accepts_when_under_budget() {
        let out_dir = tmp_out_dir("node_graph_cull_window_shifts_max_ok");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "prepaint_actions": [{
                            "node": 10,
                            "kind": "node_graph_cull_window_shift",
                            "node_graph_cull_window_key": 456
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_node_graph_cull_window_shifts_max(&bundle_path, &bundle_dir, 1, 0)
            .expect("expected max gate to accept actions under budget");

        let evidence_path = bundle_dir.join("check.node_graph_cull_window_shifts_max.json");
        assert!(
            evidence_path.is_file(),
            "expected node graph cull window shifts max evidence JSON to be written"
        );
    }

    #[test]
    fn node_graph_cull_window_shifts_max_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("node_graph_cull_window_shifts_max_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "prepaint_actions": [{
                            "node": 10,
                            "kind": "node_graph_cull_window_shift",
                            "node_graph_cull_window_key": 456
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err =
            check_bundle_for_node_graph_cull_window_shifts_max(&bundle_path, &bundle_dir, 0, 0)
                .unwrap_err();
        assert!(err.contains("node graph cull window shift"));

        let evidence_path = bundle_dir.join("check.node_graph_cull_window_shifts_max.json");
        assert!(
            evidence_path.is_file(),
            "expected node graph cull window shifts max evidence JSON to be written"
        );
        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("node_graph_cull_window_shifts_max")
        );
    }

    #[test]
    fn vlist_window_shifts_have_prepaint_actions_accepts_matching_action() {
        let out_dir = tmp_out_dir("vlist_window_shifts_have_prepaint_actions_ok");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "virtual_list_windows": [{
                            "node": 10,
                            "element": 1,
                            "source": "prepaint",
                            "window_shift_kind": "escape",
                            "window_shift_reason": "viewport_resize"
                        }],
                        "prepaint_actions": [{
                            "kind": "virtual_list_window_shift",
                            "node": 10,
                            "element": 1,
                            "virtual_list_window_shift_kind": "escape",
                            "virtual_list_window_shift_reason": "viewport_resize",
                            "frame_id": 1
                        }]
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_vlist_window_shifts_have_prepaint_actions(&bundle_path, &bundle_dir, 0)
            .expect("expected vlist shift prepaint-action gate to pass");
    }

    #[test]
    fn vlist_window_shifts_have_prepaint_actions_writes_evidence_json_on_failure() {
        let out_dir = tmp_out_dir("vlist_window_shifts_have_prepaint_actions_evidence");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_dir = out_dir.join("run");
        let _ = std::fs::create_dir_all(&bundle_dir);
        let bundle_path = bundle_dir.join("bundle.json");

        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "debug": {
                        "virtual_list_windows": [{
                            "node": 10,
                            "element": 1,
                            "source": "prepaint",
                            "window_shift_kind": "escape",
                            "window_shift_reason": "items_revision"
                        }],
                        "prepaint_actions": []
                    }
                }]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_vlist_window_shifts_have_prepaint_actions(
            &bundle_path,
            &bundle_dir,
            0,
        )
        .unwrap_err();
        assert!(err.contains("vlist window-shift prepaint-action gate failed"));

        let evidence_path = bundle_dir.join("check.vlist_window_shifts_have_prepaint_actions.json");
        assert!(
            evidence_path.is_file(),
            "expected vlist shift prepaint-action evidence JSON to be written"
        );
        let evidence: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
        assert_eq!(
            evidence.get("kind").and_then(|v| v.as_str()),
            Some("vlist_window_shifts_have_prepaint_actions")
        );
    }

    #[test]
    fn idle_no_paint_check_passes_when_tail_streak_meets_min() {
        let out_dir = tmp_out_dir("idle_no_paint_pass");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "prepaint_time_us": 1, "paint_time_us": 1, "paint_nodes_performed": 1 } } },
                    { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                    { "tick_id": 3, "frame_id": 3, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                    { "tick_id": 4, "frame_id": 4, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        check_bundle_for_idle_no_paint_min(&bundle_path, &out_dir, 3, 0).unwrap();
        assert!(out_dir.join("check.idle_no_paint.json").is_file());
    }

    #[test]
    fn idle_no_paint_check_fails_when_tail_streak_is_too_small() {
        let out_dir = tmp_out_dir("idle_no_paint_fail");
        let _ = std::fs::create_dir_all(&out_dir);

        let bundle_path = out_dir.join("bundle.json");
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                    { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                    { "tick_id": 3, "frame_id": 3, "debug": { "stats": { "prepaint_time_us": 1, "paint_time_us": 1, "paint_nodes_performed": 1 } } }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
            .expect("bundle.json write should succeed");

        let err = check_bundle_for_idle_no_paint_min(&bundle_path, &out_dir, 2, 0).unwrap_err();
        assert!(err.contains("idle no-paint gate failed"));
        assert!(out_dir.join("check.idle_no_paint.json").is_file());
    }

    #[test]
    fn pixels_changed_check_passes_when_region_hash_changes() {
        let out_dir = tmp_out_dir("pixels_changed_pass");
        let _ = std::fs::create_dir_all(out_dir.join("screenshots"));

        let window = 1u64;
        let test_id = "root";
        let bounds = RectF {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        };

        write_bundle_with_bounds(&out_dir, "b0", window, 10, 10, test_id, bounds);
        write_bundle_with_bounds(&out_dir, "b1", window, 20, 20, test_id, bounds);

        write_png_solid(
            &out_dir.join("screenshots").join("b0").join("shot0.png"),
            10,
            10,
            [0, 0, 0, 255],
        );
        write_png_solid(
            &out_dir.join("screenshots").join("b1").join("shot1.png"),
            10,
            10,
            [255, 0, 0, 255],
        );

        let result = json!({
            "schema_version": 1,
            "completed": [
                { "bundle_dir_name": "b0", "file": "shot0.png", "window": window, "tick_id": 10, "frame_id": 10, "scale_factor": 1.0 },
                { "bundle_dir_name": "b1", "file": "shot1.png", "window": window, "tick_id": 20, "frame_id": 20, "scale_factor": 1.0 }
            ]
        });
        std::fs::write(
            out_dir.join("screenshots.result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .expect("screenshots.result.json write should succeed");

        check_out_dir_for_pixels_changed(&out_dir, test_id, 0).unwrap();
        assert!(out_dir.join("check.pixels_changed.json").is_file());
    }

    #[test]
    fn pixels_changed_check_fails_when_region_hash_is_unchanged() {
        let out_dir = tmp_out_dir("pixels_changed_fail");
        let _ = std::fs::create_dir_all(out_dir.join("screenshots"));

        let window = 1u64;
        let test_id = "root";
        let bounds = RectF {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        };

        write_bundle_with_bounds(&out_dir, "b0", window, 10, 10, test_id, bounds);
        write_bundle_with_bounds(&out_dir, "b1", window, 20, 20, test_id, bounds);

        write_png_solid(
            &out_dir.join("screenshots").join("b0").join("shot0.png"),
            10,
            10,
            [0, 0, 0, 255],
        );
        write_png_solid(
            &out_dir.join("screenshots").join("b1").join("shot1.png"),
            10,
            10,
            [0, 0, 0, 255],
        );

        let result = json!({
            "schema_version": 1,
            "completed": [
                { "bundle_dir_name": "b0", "file": "shot0.png", "window": window, "tick_id": 10, "frame_id": 10, "scale_factor": 1.0 },
                { "bundle_dir_name": "b1", "file": "shot1.png", "window": window, "tick_id": 20, "frame_id": 20, "scale_factor": 1.0 }
            ]
        });
        std::fs::write(
            out_dir.join("screenshots.result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .expect("screenshots.result.json write should succeed");

        let err = check_out_dir_for_pixels_changed(&out_dir, test_id, 0).unwrap_err();
        assert!(err.contains("pixels unchanged suspected"));
        assert!(out_dir.join("check.pixels_changed.json").is_file());
    }

    #[test]
    fn perf_threshold_scan_passes_when_under_limits() {
        let failures = scan_perf_threshold_failures(
            "script.json",
            BundleStatsSort::Time,
            PerfThresholds {
                max_top_total_us: Some(100),
                max_top_layout_us: Some(80),
                max_top_solve_us: Some(50),
                max_pointer_move_dispatch_us: Some(2000),
                max_pointer_move_hit_test_us: Some(1500),
                max_pointer_move_global_changes: Some(0),
                min_run_paint_cache_hit_test_only_replay_allowed_max: None,
                max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: None,
            },
            PerfThresholds::default(),
            99,
            79,
            49,
            true,
            1999,
            1499,
            0,
            0,
            0,
        );
        assert!(failures.is_empty());
    }

    #[test]
    fn perf_threshold_scan_reports_each_exceeded_metric() {
        let failures = scan_perf_threshold_failures(
            "script.json",
            BundleStatsSort::Time,
            PerfThresholds {
                max_top_total_us: Some(100),
                max_top_layout_us: Some(80),
                max_top_solve_us: Some(50),
                max_pointer_move_dispatch_us: Some(2000),
                max_pointer_move_hit_test_us: Some(1500),
                max_pointer_move_global_changes: Some(0),
                min_run_paint_cache_hit_test_only_replay_allowed_max: None,
                max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: None,
            },
            PerfThresholds::default(),
            101,
            81,
            51,
            true,
            2001,
            1501,
            1,
            0,
            0,
        );
        assert_eq!(failures.len(), 6);
        let metrics: Vec<String> = failures
            .iter()
            .filter_map(|v| {
                v.get("metric")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            })
            .collect();
        assert!(metrics.contains(&"top_total_time_us".to_string()));
        assert!(metrics.contains(&"top_layout_time_us".to_string()));
        assert!(metrics.contains(&"top_layout_engine_solve_time_us".to_string()));
        assert!(metrics.contains(&"pointer_move_max_dispatch_time_us".to_string()));
        assert!(metrics.contains(&"pointer_move_max_hit_test_time_us".to_string()));
        assert!(metrics.contains(&"pointer_move_snapshots_with_global_changes".to_string()));
    }

    #[test]
    fn perf_baseline_headroom_rounds_up() {
        assert_eq!(apply_perf_baseline_headroom(100, 20), 120);
        assert_eq!(apply_perf_baseline_headroom(101, 20), 122);
        assert_eq!(apply_perf_baseline_headroom(0, 20), 0);
    }

    #[test]
    fn perf_baseline_parse_reads_script_thresholds() {
        let out_dir = tmp_out_dir("perf_baseline_parse");
        let _ = std::fs::create_dir_all(&out_dir);
        let path = out_dir.join("perf.baseline.json");

        let v = json!({
            "schema_version": 1,
            "kind": "perf_baseline",
            "rows": [{
                "script": "tools/diag-scripts/ui-gallery-overlay-torture.json",
                "thresholds": {
                    "max_top_total_us": 25000,
                    "max_top_layout_us": 15000,
                    "max_top_solve_us": 8000
                }
            }]
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&v).unwrap())
            .expect("baseline write should succeed");

        let baseline = read_perf_baseline_file(Path::new("."), &path).unwrap();
        let t = baseline
            .thresholds_by_script
            .get("tools/diag-scripts/ui-gallery-overlay-torture.json")
            .copied()
            .unwrap();
        assert_eq!(t.max_top_total_us, Some(25_000));
        assert_eq!(t.max_top_layout_us, Some(15_000));
        assert_eq!(t.max_top_solve_us, Some(8_000));
    }

    #[test]
    fn redraw_hitch_gate_fails_when_log_missing() {
        let out_dir = tmp_out_dir("redraw_hitch_gate_missing");
        let _ = std::fs::create_dir_all(&out_dir);

        let r = check_redraw_hitches_max_total_ms(&out_dir, 16).unwrap();
        assert!(r.failures > 0);

        let v = read_json_value(&out_dir.join("check.redraw_hitches.json")).unwrap();
        assert_eq!(
            v.get("kind").and_then(|v| v.as_str()),
            Some("redraw_hitches_thresholds")
        );
        assert_eq!(
            v.get("observed")
                .and_then(|v| v.get("present"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn redraw_hitch_gate_passes_under_threshold() {
        let out_dir = tmp_out_dir("redraw_hitch_gate_pass");
        let _ = std::fs::create_dir_all(&out_dir);
        let log = out_dir.join("redraw_hitches.log");
        std::fs::write(
            &log,
            "\
[1] [thread=ThreadId(1)] redraw hitch window=AppWindowId(1v1) total_ms=10 prepare_ms=Some(0) render_ms=Some(10) record_ms=Some(0) present_ms=Some(0) scene_ops=1 bounds=Rect {} scale_factor=1.0\n\
[2] [thread=ThreadId(1)] redraw hitch window=AppWindowId(1v1) total_ms=12 prepare_ms=Some(0) render_ms=Some(12) record_ms=Some(0) present_ms=Some(0) scene_ops=1 bounds=Rect {} scale_factor=1.0\n",
        )
        .unwrap();

        let r = check_redraw_hitches_max_total_ms(&out_dir, 20).unwrap();
        assert_eq!(r.failures, 0);

        let v = read_json_value(&out_dir.join("check.redraw_hitches.json")).unwrap();
        assert_eq!(
            v.get("failures")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );
        assert_eq!(
            v.get("observed")
                .and_then(|v| v.get("records"))
                .and_then(|v| v.as_u64()),
            Some(2)
        );
    }

    #[test]
    fn redraw_hitch_gate_fails_over_threshold() {
        let out_dir = tmp_out_dir("redraw_hitch_gate_fail");
        let _ = std::fs::create_dir_all(&out_dir);
        let log = out_dir.join("redraw_hitches.log");
        std::fs::write(
            &log,
            "\
[1] [thread=ThreadId(1)] redraw hitch window=AppWindowId(1v1) total_ms=30 prepare_ms=Some(0) render_ms=Some(29) record_ms=Some(0) present_ms=Some(1) scene_ops=1 bounds=Rect {} scale_factor=1.0\n",
        )
        .unwrap();

        let r = check_redraw_hitches_max_total_ms(&out_dir, 20).unwrap();
        assert_eq!(r.failures, 1);

        let v = read_json_value(&out_dir.join("check.redraw_hitches.json")).unwrap();
        let failures = v.get("failures").and_then(|v| v.as_array()).unwrap();
        assert_eq!(
            failures
                .iter()
                .any(|f| f.get("kind").and_then(|v| v.as_str()) == Some("max_total_ms")),
            true
        );
    }

    #[test]
    fn redraw_hitch_gate_parses_tick_and_frame_ids() {
        let out_dir = tmp_out_dir("redraw_hitch_gate_tick_frame");
        let _ = std::fs::create_dir_all(&out_dir);
        let log = out_dir.join("redraw_hitches.log");
        std::fs::write(
            &log,
            "\
[1] [thread=ThreadId(1)] redraw hitch window=AppWindowId(1v1) tick_id=7 frame_id=9 total_ms=30 prepare_ms=Some(0) render_ms=Some(29) record_ms=Some(0) present_ms=Some(1) scene_ops=1 bounds=Rect {} scale_factor=1.0\n",
        )
        .unwrap();

        let _ = check_redraw_hitches_max_total_ms(&out_dir, 10).unwrap();
        let v = read_json_value(&out_dir.join("check.redraw_hitches.json")).unwrap();
        let top = v.get("top").and_then(|v| v.as_array()).unwrap();
        assert_eq!(
            top.first()
                .and_then(|t| t.get("tick_id"))
                .and_then(|v| v.as_u64()),
            Some(7)
        );
        assert_eq!(
            top.first()
                .and_then(|t| t.get("frame_id"))
                .and_then(|v| v.as_u64()),
            Some(9)
        );
    }

    #[test]
    fn ui_gallery_code_editor_undo_redo_gate_passes_on_marker_toggle_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "code_editor": { "torture": {
                                    "marker_present": false,
                                    "text_len_bytes": 10,
                                    "selection": { "anchor": 0, "caret": 0 }
                                }}
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "code_editor": { "torture": {
                                    "marker_present": true,
                                    "text_len_bytes": 30,
                                    "selection": { "anchor": 0, "caret": 5 }
                                }}
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "code_editor": { "torture": {
                                    "marker_present": false,
                                    "text_len_bytes": 10,
                                    "selection": { "anchor": 0, "caret": 5 }
                                }}
                            }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "code_editor": { "torture": {
                                    "marker_present": true,
                                    "text_len_bytes": 30,
                                    "selection": { "anchor": 0, "caret": 5 }
                                }}
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_undo_redo_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_undo_redo_gate_fails_without_redo() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "code_editor": { "torture": {
                                    "marker_present": true,
                                    "text_len_bytes": 30,
                                    "selection": { "anchor": 0, "caret": 5 }
                                }}
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "code_editor": { "torture": {
                                    "marker_present": false,
                                    "text_len_bytes": 10,
                                    "selection": { "anchor": 0, "caret": 5 }
                                }}
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_undo_redo_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("undo/redo gate failed"));
    }

    #[test]
    fn ui_gallery_code_editor_read_only_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": { "torture": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 1,
                                    "text_len_bytes": 5
                                }}
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": { "torture": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": { "torture": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": { "torture": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 5,
                            "frame_id": 5,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": { "torture": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_read_only_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_read_only_gate_fails_when_mutated() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": { "torture": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 1,
                                    "text_len_bytes": 5
                                }}
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": { "torture": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": { "torture": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": { "torture": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 3,
                                    "text_len_bytes": 7
                                }}
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_read_only_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("read-only gate failed"));
    }

    #[test]
    fn ui_gallery_markdown_editor_read_only_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 1,
                                    "text_len_bytes": 5
                                }}
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 5,
                            "frame_id": 5,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_read_only_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_disabled_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 7,
                                    "text_len_bytes": 42,
                                    "selection": { "caret": 3 }
                                }}
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [3,3] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": false, "editable": false },
                                    "buffer_revision": 7,
                                    "text_len_bytes": 42,
                                    "selection": { "caret": 3 }
                                }}
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": false }, "text_selection": [3,3] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": false, "editable": false },
                                    "buffer_revision": 7,
                                    "text_len_bytes": 42,
                                    "selection": { "caret": 3 }
                                }}
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": false }, "text_selection": [3,3] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": false, "editable": false },
                                    "buffer_revision": 7,
                                    "text_len_bytes": 42,
                                    "selection": { "caret": 3 }
                                }}
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": false }, "text_selection": [3,3] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_disabled_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_read_only_gate_fails_when_mutated() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 1,
                                    "text_len_bytes": 5
                                }}
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 6
                                }}
                            }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": false },
                                    "buffer_revision": 3,
                                    "text_len_bytes": 7
                                }}
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_read_only_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("read-only gate failed"));
    }

    #[test]
    fn ui_gallery_markdown_editor_soft_wrap_toggle_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_soft_wrap_toggle_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_soft_wrap_toggle_gate_fails_when_caret_moves() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 6 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 6 }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_soft_wrap_toggle_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("soft-wrap toggle gate failed"));
    }

    #[test]
    fn ui_gallery_markdown_editor_folds_toggle_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": true,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_folds_toggle_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_folds_toggle_gate_fails_when_rev_changes() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": true,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 3,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 3,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_folds_toggle_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("folds toggle gate failed"));
    }

    #[test]
    fn ui_gallery_markdown_editor_folds_clamp_selection_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 },
                                        "folds": {
                                            "fixture_span_line0": { "start": 3, "end": 9 }
                                        }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": true,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 3 },
                                        "folds": {
                                            "fixture_span_line0": { "start": 3, "end": 9 },
                                            "line0_placeholder_present": true
                                        }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_folds_clamp_selection_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_folds_clamp_selection_gate_fails_when_caret_stays_inside() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 },
                                        "folds": {
                                            "fixture_span_line0": { "start": 3, "end": 9 }
                                        }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": true,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 },
                                        "folds": {
                                            "fixture_span_line0": { "start": 3, "end": 9 },
                                            "line0_placeholder_present": true
                                        }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_folds_clamp_selection_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err =
            check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json(
                &bundle,
                &bundle_path,
                0,
            )
            .unwrap_err();
        assert!(err.contains("clamp-selection gate failed"));
    }

    #[test]
    fn ui_gallery_markdown_editor_inlays_toggle_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "inlays_fixture": false,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "inlays_fixture": true,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "inlays_fixture": false,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_inlays_toggle_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_inlays_toggle_gate_fails_when_caret_moves() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "inlays_fixture": false,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 5 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "inlays_fixture": true,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 6 }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "inlays_fixture": false,
                                    "markdown_editor_source": {
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 10,
                                        "selection": { "caret": 6 }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_inlays_toggle_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("inlays toggle gate failed"));
    }

    #[test]
    fn ui_gallery_markdown_editor_inlays_caret_navigation_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "inlays_fixture": false,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 5,
                                        "selection": { "anchor": 2, "caret": 2 },
                                        "inlays": { "enabled": false, "line0_present": false }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "inlays_fixture": true,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 5,
                                        "selection": { "anchor": 2, "caret": 2 },
                                        "inlays": { "enabled": true, "line0_present": true }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "inlays_fixture": true,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 5,
                                        "selection": { "anchor": 3, "caret": 3 },
                                        "inlays": { "enabled": true, "line0_present": true }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "inlays_fixture": true,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 5,
                                        "selection": { "anchor": 2, "caret": 2 },
                                        "inlays": { "enabled": true, "line0_present": true }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_inlays_caret_navigation_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_inlays_caret_navigation_gate_fails_when_caret_does_not_move() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "inlays_fixture": false,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 5,
                                        "selection": { "anchor": 2, "caret": 2 },
                                        "inlays": { "enabled": false, "line0_present": false }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "inlays_fixture": true,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 5,
                                        "selection": { "anchor": 2, "caret": 2 },
                                        "inlays": { "enabled": true, "line0_present": true }
                                    }
                                }
                            }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": {
                                    "soft_wrap_cols": null,
                                    "folds_fixture": false,
                                    "inlays_fixture": true,
                                    "markdown_editor_source": {
                                        "preedit_active": false,
                                        "interaction": { "enabled": true, "editable": true },
                                        "buffer_revision": 2,
                                        "text_len_bytes": 5,
                                        "selection": { "anchor": 2, "caret": 2 },
                                        "inlays": { "enabled": true, "line0_present": true }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_inlays_caret_navigation_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err =
            check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json(
                &bundle,
                &bundle_path,
                0,
            )
            .unwrap_err();
        assert!(err.contains("caret-navigation gate failed"));
    }

    #[test]
    fn ui_gallery_markdown_editor_word_boundary_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [5,5] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_word_boundary_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_word_boundary_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_web_ime_bridge_gate_passes_when_enabled_and_cursor_area_seen() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": { "kind": "fret_ui_gallery", "selected_page": "markdown_editor_source" },
                            "debug": { "web_ime_bridge": {
                                "enabled": true,
                                "mount_kind": "body",
                                "position_mode": "fixed",
                                "textarea_has_focus": true,
                                "cursor_area_set_seen": 1,
                                "last_cursor_area": { "origin": { "x": 0.0, "y": 0.0 }, "size": { "width": 1.0, "height": 1.0 } }
                            } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_web_ime_bridge_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_web_ime_bridge_enabled_json(&bundle, &bundle_path, 0).unwrap();
    }

    #[test]
    fn ui_gallery_web_ime_bridge_gate_fails_when_disabled() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": { "kind": "fret_ui_gallery", "selected_page": "markdown_editor_source" },
                            "debug": { "web_ime_bridge": {
                                "enabled": false,
                                "mount_kind": "body",
                                "position_mode": "fixed",
                                "textarea_has_focus": false,
                                "cursor_area_set_seen": 0,
                                "last_cursor_area": null
                            } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_web_ime_bridge_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_web_ime_bridge_enabled_json(&bundle, &bundle_path, 0)
            .unwrap_err();
        assert!(err.contains("ui-gallery web-ime bridge gate failed"));
    }

    #[test]
    fn devtools_sanitize_export_dir_name_takes_file_name() {
        assert_eq!(
            devtools_sanitize_export_dir_name("1700000-bundle"),
            "1700000-bundle"
        );
        assert_eq!(devtools_sanitize_export_dir_name("a/b/c"), "c");
        assert_eq!(devtools_sanitize_export_dir_name(""), "bundle");
    }

    #[test]
    fn devtools_select_session_id_prefers_single_web_app_when_multiple() {
        let list = DevtoolsSessionListV1 {
            sessions: vec![
                DevtoolsSessionDescriptorV1 {
                    session_id: "s-native".to_string(),
                    client_kind: "native_app".to_string(),
                    client_version: "1".to_string(),
                    capabilities: Vec::new(),
                },
                DevtoolsSessionDescriptorV1 {
                    session_id: "s-web".to_string(),
                    client_kind: "web_app".to_string(),
                    client_version: "1".to_string(),
                    capabilities: Vec::new(),
                },
            ],
        };
        assert_eq!(
            devtools_select_session_id(&list, None).unwrap(),
            "s-web".to_string()
        );
    }

    #[test]
    fn devtools_select_session_id_requires_explicit_when_ambiguous() {
        let list = DevtoolsSessionListV1 {
            sessions: vec![
                DevtoolsSessionDescriptorV1 {
                    session_id: "s1".to_string(),
                    client_kind: "native_app".to_string(),
                    client_version: "1".to_string(),
                    capabilities: Vec::new(),
                },
                DevtoolsSessionDescriptorV1 {
                    session_id: "s2".to_string(),
                    client_kind: "native_app".to_string(),
                    client_version: "1".to_string(),
                    capabilities: Vec::new(),
                },
            ],
        };
        let err = devtools_select_session_id(&list, None).unwrap_err();
        assert!(err.contains("multiple DevTools sessions available"));
    }

    #[test]
    fn ui_gallery_markdown_editor_line_boundary_triple_click_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "hello\nworld\n", "text_selection": [0,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "hello\nworld\n", "text_selection": [0,6] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir =
            tmp_out_dir("ui_gallery_markdown_editor_line_boundary_triple_click_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_a11y_composition_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [4,4], "text_composition": [2,4] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_a11y_composition_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_a11y_composition_soft_wrap_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "soft_wrap_cols": 80 }
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "soft_wrap_cols": 80 }
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [4,4], "text_composition": [2,4] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "soft_wrap_cols": 80 }
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir =
            tmp_out_dir("ui_gallery_markdown_editor_a11y_composition_soft_wrap_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_markdown_editor_soft_wrap_editing_gate_passes_on_sequence() {
        let value_a = "a".repeat(100);
        let value_b = "a".repeat(101);

        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "soft_wrap_cols": 80 }
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_a, "text_selection": [0,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "soft_wrap_cols": 80 }
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_a, "text_selection": [80,80] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "soft_wrap_cols": 80 }
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_b, "text_selection": [81,81] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "soft_wrap_cols": 80 }
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_b, "text_selection": [0,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 5,
                            "frame_id": 5,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "markdown_editor_source",
                                "code_editor": { "soft_wrap_cols": 80 }
                            },
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_b, "text_selection": [80,80] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_markdown_editor_soft_wrap_editing_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_a11y_selection_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,5] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [11,11] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [11,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_selection_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_a11y_selection_json(&bundle, &bundle_path, 0)
            .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_a11y_selection_gate_fails_without_select_all() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,5] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [11,11] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_selection_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err =
            check_bundle_for_ui_gallery_code_editor_a11y_selection_json(&bundle, &bundle_path, 0)
                .unwrap_err();
        assert!(err.contains("a11y-selection gate failed"));
    }

    #[test]
    fn ui_gallery_code_editor_a11y_composition_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [4,4], "text_composition": [2,4] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_composition_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_a11y_composition_json(&bundle, &bundle_path, 0)
            .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_a11y_composition_gate_fails_without_preedit() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_composition_gate_fails");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err =
            check_bundle_for_ui_gallery_code_editor_a11y_composition_json(&bundle, &bundle_path, 0)
                .unwrap_err();
        assert!(err.contains("a11y-composition gate failed"));
    }

    #[test]
    fn ui_gallery_code_editor_a11y_selection_wrap_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "<redacted len=200>", "text_selection": [0,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "<redacted len=200>", "text_selection": [80,80] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "<redacted len=200>", "text_selection": [200,200] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 4,
                            "frame_id": 4,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "<redacted len=200>", "text_selection": [200,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_selection_wrap_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap_json(&bundle, &bundle_path, 0)
            .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_a11y_composition_wrap_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [78,78] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [80,80], "text_composition": [78,80] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [78,78] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_composition_wrap_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_a11y_composition_drag_gate_passes_on_sequence() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [78,78] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-drag-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 2,
                            "frame_id": 2,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [80,80], "text_composition": [78,80] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-drag-gate-viewport", "parent": 2 }
                            ] } }
                        },
                        {
                            "tick_id": 3,
                            "frame_id": 3,
                            "debug": { "semantics": { "nodes": [
                                { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [10,0] },
                                { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-drag-gate-viewport", "parent": 2 }
                            ] } }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_composition_drag_gate_passes");
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_a11y_composition_drag_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_folds_inline_preedit_unwrapped_gate_passes_when_placeholder_present()
    {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "folds_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "folds": { "line0_placeholder_present": true }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir(
            "ui_gallery_code_editor_folds_inline_preedit_unwrapped_gate_passes_when_present",
        );
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_folds_inline_preedit_unwrapped_gate_fails_when_placeholder_absent() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "folds_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "folds": { "line0_placeholder_present": false }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir(
            "ui_gallery_code_editor_folds_inline_preedit_unwrapped_gate_fails_when_absent",
        );
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("gate failed"));
    }

    #[test]
    fn ui_gallery_code_editor_folds_inline_preedit_with_decorations_gate_passes_when_placeholder_present()
     {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "folds_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "allow_decorations_under_inline_preedit": true,
                                        "folds": { "line0_placeholder_present": true }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir(
            "ui_gallery_code_editor_folds_inline_preedit_with_decorations_gate_passes_when_present",
        );
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_folds_inline_preedit_with_decorations_gate_fails_when_placeholder_absent()
     {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "folds_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "allow_decorations_under_inline_preedit": true,
                                        "folds": { "line0_placeholder_present": false }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir(
            "ui_gallery_code_editor_folds_inline_preedit_with_decorations_gate_fails_when_absent",
        );
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("gate failed"));
    }

    #[test]
    fn ui_gallery_code_editor_inlays_inline_preedit_unwrapped_gate_passes_when_inlay_present() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "inlays_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "inlays": { "line0_inlay_present": true }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir(
            "ui_gallery_code_editor_inlays_inline_preedit_unwrapped_gate_passes_when_present",
        );
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_inlays_inline_preedit_unwrapped_gate_fails_when_inlay_absent() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "inlays_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "inlays": { "line0_inlay_present": false }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir(
            "ui_gallery_code_editor_inlays_inline_preedit_unwrapped_gate_fails_when_absent",
        );
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("gate failed"));
    }

    #[test]
    fn ui_gallery_code_editor_inlays_inline_preedit_with_decorations_gate_passes_when_inlay_present()
     {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "inlays_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "allow_decorations_under_inline_preedit": true,
                                        "inlays": { "line0_inlay_present": true }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir(
            "ui_gallery_code_editor_inlays_inline_preedit_with_decorations_gate_passes_when_present",
        );
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap();
    }

    #[test]
    fn ui_gallery_code_editor_inlays_inline_preedit_with_decorations_gate_fails_when_inlay_absent()
    {
        let bundle = json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "tick_id": 1,
                            "frame_id": 1,
                            "app_snapshot": {
                                "kind": "fret_ui_gallery",
                                "selected_page": "code_editor_torture",
                                "code_editor": {
                                    "soft_wrap_cols": 80,
                                    "inlays_fixture": true,
                                    "torture": {
                                        "preedit_active": true,
                                        "allow_decorations_under_inline_preedit": true,
                                        "inlays": { "line0_inlay_present": false }
                                    }
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let out_dir = tmp_out_dir(
            "ui_gallery_code_editor_inlays_inline_preedit_with_decorations_gate_fails_when_absent",
        );
        let _ = std::fs::create_dir_all(&out_dir);
        let bundle_path = out_dir.join("bundle.json");
        let err = check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
        assert!(err.contains("gate failed"));
    }
}
