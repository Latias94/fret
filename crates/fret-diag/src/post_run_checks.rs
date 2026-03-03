use super::*;
use crate::registry::checks::{CheckRegistry, PostRunCheckContext};
use crate::stats;

pub(crate) fn apply_post_run_checks(
    bundle_path: &Path,
    out_dir: &Path,
    checks: &diag_run::RunChecks,
    warmup_frames: u64,
) -> Result<(), String> {
    let check_ui_gallery_code_editor_torture_read_only_blocks_edits =
        checks.check_ui_gallery_code_editor_torture_read_only_blocks_edits;
    let check_ui_gallery_markdown_editor_source_read_only_blocks_edits =
        checks.check_ui_gallery_markdown_editor_source_read_only_blocks_edits;
    let check_ui_gallery_markdown_editor_source_disabled_blocks_edits =
        checks.check_ui_gallery_markdown_editor_source_disabled_blocks_edits;
    let check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps =
        checks.check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps;
    let check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change =
        checks.check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change;
    let check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change =
        checks.check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change;
    let check_ui_gallery_text_mixed_script_bundled_fallback_conformance =
        checks.check_ui_gallery_text_mixed_script_bundled_fallback_conformance;
    let check_semantics_changed_repainted = checks.check_semantics_changed_repainted;
    let dump_semantics_changed_repainted_json = checks.dump_semantics_changed_repainted_json;
    let check_wheel_scroll_test_id = checks.check_wheel_scroll_test_id.as_deref();
    let check_wheel_scroll_hit_changes_test_id =
        checks.check_wheel_scroll_hit_changes_test_id.as_deref();
    let check_prepaint_actions_min = checks.check_prepaint_actions_min;
    let check_chart_sampling_window_shifts_min = checks.check_chart_sampling_window_shifts_min;
    let check_node_graph_cull_window_shifts_min = checks.check_node_graph_cull_window_shifts_min;
    let check_node_graph_cull_window_shifts_max = checks.check_node_graph_cull_window_shifts_max;
    let check_vlist_visible_range_refreshes_min = checks.check_vlist_visible_range_refreshes_min;
    let check_vlist_visible_range_refreshes_max = checks.check_vlist_visible_range_refreshes_max;
    let check_vlist_window_shifts_explainable = checks.check_vlist_window_shifts_explainable;
    let check_vlist_window_shifts_have_prepaint_actions =
        checks.check_vlist_window_shifts_have_prepaint_actions;
    let check_vlist_window_shifts_non_retained_max =
        checks.check_vlist_window_shifts_non_retained_max;
    let check_vlist_window_shifts_prefetch_max = checks.check_vlist_window_shifts_prefetch_max;
    let check_vlist_window_shifts_escape_max = checks.check_vlist_window_shifts_escape_max;
    let check_vlist_policy_key_stable = checks.check_vlist_policy_key_stable;
    let check_windowed_rows_offset_changes_min = checks.check_windowed_rows_offset_changes_min;
    let check_windowed_rows_offset_changes_eps = checks.check_windowed_rows_offset_changes_eps;
    let check_windowed_rows_visible_start_changes_repainted =
        checks.check_windowed_rows_visible_start_changes_repainted;
    let check_layout_fast_path_min = checks.check_layout_fast_path_min;
    let check_drag_cache_root_paint_only_test_id =
        checks.check_drag_cache_root_paint_only_test_id.as_deref();
    let check_hover_layout_max = checks.check_hover_layout_max;
    let check_view_cache_reuse_stable_min = checks.check_view_cache_reuse_stable_min;
    let check_view_cache_reuse_min = checks.check_view_cache_reuse_min;
    let check_overlay_synthesis_min = checks.check_overlay_synthesis_min;
    let check_viewport_input_min = checks.check_viewport_input_min;
    let check_dock_drag_min = checks.check_dock_drag_min;
    let check_viewport_capture_min = checks.check_viewport_capture_min;
    let check_retained_vlist_reconcile_no_notify_min =
        checks.check_retained_vlist_reconcile_no_notify_min;
    let check_retained_vlist_attach_detach_max = checks.check_retained_vlist_attach_detach_max;
    let check_retained_vlist_keep_alive_reuse_min =
        checks.check_retained_vlist_keep_alive_reuse_min;
    let check_retained_vlist_keep_alive_budget = checks.check_retained_vlist_keep_alive_budget;

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
                crate::resolve_bundle_artifact_path(&p)
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
            let (from_latest, from_scan) = crate::latest::latest_bundle_dir_candidates(out_dir);
            let from_latest = from_latest.map(normalize_bundle_path);
            let from_scan = from_scan.map(|dir| normalize_bundle_path(dir));

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

    if check_ui_gallery_code_editor_torture_read_only_blocks_edits {
        stats::check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_read_only_blocks_edits {
        stats::check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_markdown_editor_source_disabled_blocks_edits {
        stats::check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits(
            bundle_path,
            warmup_frames,
        )?;
    }
    if check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps {
        check_out_dir_for_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps(out_dir)?;
    }
    if check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change {
        check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(out_dir)?;
    }
    if check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change {
        check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(out_dir)?;
    }
    if check_ui_gallery_text_mixed_script_bundled_fallback_conformance {
        check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(out_dir)?;
    }
    if check_semantics_changed_repainted {
        stats::check_bundle_for_semantics_changed_repainted(
            bundle_path,
            warmup_frames,
            dump_semantics_changed_repainted_json,
        )?;
    }
    if let Some(test_id) = check_wheel_scroll_test_id {
        stats::check_bundle_for_wheel_scroll(bundle_path, test_id, warmup_frames)?;
    }
    if let Some(test_id) = check_wheel_scroll_hit_changes_test_id {
        stats::check_bundle_for_wheel_scroll_hit_changes(bundle_path, test_id, warmup_frames)?;
    }
    if let Some(min) = check_prepaint_actions_min {
        stats::check_bundle_for_prepaint_actions_min(bundle_path, out_dir, min, warmup_frames)?;
    }
    if let Some(min) = check_chart_sampling_window_shifts_min {
        stats::check_bundle_for_chart_sampling_window_shifts_min(
            bundle_path,
            out_dir,
            min,
            warmup_frames,
        )?;
    }
    if let Some(min) = check_node_graph_cull_window_shifts_min {
        stats::check_bundle_for_node_graph_cull_window_shifts_min(
            bundle_path,
            out_dir,
            min,
            warmup_frames,
        )?;
    }
    if let Some(max) = check_node_graph_cull_window_shifts_max {
        stats::check_bundle_for_node_graph_cull_window_shifts_max(
            bundle_path,
            out_dir,
            max,
            warmup_frames,
        )?;
    }
    if let Some(min_total_refreshes) = check_vlist_visible_range_refreshes_min {
        stats::check_bundle_for_vlist_visible_range_refreshes_min(
            bundle_path,
            out_dir,
            min_total_refreshes,
            warmup_frames,
        )?;
    }
    if let Some(max_total_refreshes) = check_vlist_visible_range_refreshes_max {
        stats::check_bundle_for_vlist_visible_range_refreshes_max(
            bundle_path,
            out_dir,
            max_total_refreshes,
            warmup_frames,
        )?;
    }
    if check_vlist_window_shifts_explainable {
        stats::check_bundle_for_vlist_window_shifts_explainable(
            bundle_path,
            out_dir,
            warmup_frames,
        )?;
    }
    if check_vlist_window_shifts_have_prepaint_actions {
        stats::check_bundle_for_vlist_window_shifts_have_prepaint_actions(
            bundle_path,
            out_dir,
            warmup_frames,
        )?;
    }
    if let Some(max_total_non_retained_shifts) = check_vlist_window_shifts_non_retained_max {
        stats::check_bundle_for_vlist_window_shifts_non_retained_max(
            bundle_path,
            out_dir,
            max_total_non_retained_shifts,
            warmup_frames,
        )?;
    }
    if let Some(max_total_prefetch_shifts) = check_vlist_window_shifts_prefetch_max {
        stats::check_bundle_for_vlist_window_shifts_kind_max(
            bundle_path,
            out_dir,
            "prefetch",
            max_total_prefetch_shifts,
            warmup_frames,
        )?;
    }
    if let Some(max_total_escape_shifts) = check_vlist_window_shifts_escape_max {
        stats::check_bundle_for_vlist_window_shifts_kind_max(
            bundle_path,
            out_dir,
            "escape",
            max_total_escape_shifts,
            warmup_frames,
        )?;
    }
    if check_vlist_policy_key_stable {
        stats::check_bundle_for_vlist_policy_key_stable(bundle_path, out_dir, warmup_frames)?;
    }
    if let Some(min_total_offset_changes) = check_windowed_rows_offset_changes_min {
        stats::check_bundle_for_windowed_rows_offset_changes_min(
            bundle_path,
            out_dir,
            min_total_offset_changes,
            warmup_frames,
            check_windowed_rows_offset_changes_eps,
        )?;
    }
    if check_windowed_rows_visible_start_changes_repainted {
        stats::check_bundle_for_windowed_rows_visible_start_changes_repainted(
            bundle_path,
            out_dir,
            warmup_frames,
        )?;
    }
    if let Some(min_frames) = check_layout_fast_path_min {
        stats::check_bundle_for_layout_fast_path_min(
            bundle_path,
            out_dir,
            min_frames,
            warmup_frames,
        )?;
    }
    if let Some(test_id) = check_drag_cache_root_paint_only_test_id {
        stats::check_bundle_for_drag_cache_root_paint_only(bundle_path, test_id, warmup_frames)?;
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
        stats::check_bundle_for_view_cache_reuse_stable_min(
            bundle_path,
            out_dir,
            min,
            warmup_frames,
        )?;
    }
    if let Some(min) = check_view_cache_reuse_min
        && min > 0
    {
        stats::check_bundle_for_view_cache_reuse_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_overlay_synthesis_min
        && min > 0
    {
        stats::check_bundle_for_overlay_synthesis_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_viewport_input_min
        && min > 0
    {
        stats::check_bundle_for_viewport_input_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_dock_drag_min
        && min > 0
    {
        stats::check_bundle_for_dock_drag_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_viewport_capture_min
        && min > 0
    {
        stats::check_bundle_for_viewport_capture_min(bundle_path, min, warmup_frames)?;
    }
    if let Some(min) = check_retained_vlist_reconcile_no_notify_min
        && min > 0
    {
        stats::check_bundle_for_retained_vlist_reconcile_no_notify_min(
            bundle_path,
            min,
            warmup_frames,
        )?;
    }
    if let Some(max_delta) = check_retained_vlist_attach_detach_max {
        stats::check_bundle_for_retained_vlist_attach_detach_max(
            bundle_path,
            max_delta,
            warmup_frames,
        )?;
    }
    if let Some(min) = check_retained_vlist_keep_alive_reuse_min
        && min > 0
    {
        stats::check_bundle_for_retained_vlist_keep_alive_reuse_min(
            bundle_path,
            min,
            warmup_frames,
        )?;
    }
    if let Some((min_max_pool_len_after, max_total_evicted_items)) =
        check_retained_vlist_keep_alive_budget
    {
        stats::check_bundle_for_retained_vlist_keep_alive_budget(
            bundle_path,
            min_max_pool_len_after,
            max_total_evicted_items,
            warmup_frames,
        )?;
    }
    CheckRegistry::builtin().apply_post_run_checks(
        PostRunCheckContext {
            bundle_path,
            out_dir,
            warmup_frames,
        },
        checks,
    )?;
    Ok(())
}
