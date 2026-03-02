use super::*;
use crate::commands::resolve;
use crate::stats;

mod check_support;

use check_support::{
    STATS_LITE_SUPPORTED_CHECKS, StatsLiteCheckKind, stats_lite_support_for,
    stats_lite_support_matrix_json_value,
};

#[derive(Debug, Clone)]
pub(crate) struct StatsCmdContext {
    pub rest: Vec<String>,
    pub stats_diff: Option<(PathBuf, PathBuf)>,
    pub workspace_root: PathBuf,
    pub sort_override: Option<BundleStatsSort>,
    pub stats_top: usize,
    pub stats_json: bool,
    pub stats_lite_checks_json: bool,
    pub stats_verbose: bool,
    pub warmup_frames: u64,
    pub check_stale_paint_test_id: Option<String>,
    pub check_stale_paint_eps: f32,
    pub check_stale_scene_test_id: Option<String>,
    pub check_stale_scene_eps: f32,
    pub check_idle_no_paint_min: Option<u64>,
    pub check_pixels_changed_test_id: Option<String>,
    pub check_pixels_unchanged_test_id: Option<String>,
    pub check_semantics_changed_repainted: bool,
    pub dump_semantics_changed_repainted_json: bool,
    pub check_wheel_scroll_test_id: Option<String>,
    pub check_wheel_scroll_hit_changes_test_id: Option<String>,
    pub check_drag_cache_root_paint_only_test_id: Option<String>,
    pub check_hover_layout_max: Option<u32>,
    pub check_gc_sweep_liveness: bool,
    pub check_notify_hotspot_file_max: Vec<(String, u64)>,
    pub check_view_cache_reuse_stable_min: Option<u64>,
    pub check_view_cache_reuse_min: Option<u64>,
    pub check_overlay_synthesis_min: Option<u64>,
    pub check_viewport_input_min: Option<u64>,
    pub check_dock_drag_min: Option<u64>,
    pub check_viewport_capture_min: Option<u64>,
    pub check_retained_vlist_reconcile_no_notify_min: Option<u64>,
    pub check_retained_vlist_attach_detach_max: Option<u64>,
    pub check_retained_vlist_keep_alive_reuse_min: Option<u64>,
}

pub(crate) fn cmd_stats(ctx: StatsCmdContext) -> Result<(), String> {
    let StatsCmdContext {
        rest,
        stats_diff,
        workspace_root,
        sort_override,
        stats_top,
        stats_json,
        stats_lite_checks_json,
        stats_verbose,
        warmup_frames,
        check_stale_paint_test_id,
        check_stale_paint_eps,
        check_stale_scene_test_id,
        check_stale_scene_eps,
        check_idle_no_paint_min,
        check_pixels_changed_test_id,
        check_pixels_unchanged_test_id,
        check_semantics_changed_repainted,
        dump_semantics_changed_repainted_json,
        check_wheel_scroll_test_id,
        check_wheel_scroll_hit_changes_test_id,
        check_drag_cache_root_paint_only_test_id,
        check_hover_layout_max,
        check_gc_sweep_liveness,
        check_notify_hotspot_file_max,
        check_view_cache_reuse_stable_min,
        check_view_cache_reuse_min,
        check_overlay_synthesis_min,
        check_viewport_input_min,
        check_dock_drag_min,
        check_viewport_capture_min,
        check_retained_vlist_reconcile_no_notify_min,
        check_retained_vlist_attach_detach_max,
        check_retained_vlist_keep_alive_reuse_min,
    } = ctx;

    if stats_lite_checks_json {
        if stats_diff.is_some() {
            return Err("--stats-lite-checks-json cannot be combined with --diff".to_string());
        }
        if !rest.is_empty() {
            return Err(format!("unexpected arguments: {}", rest.join(" ")));
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&stats_lite_support_matrix_json_value())
                .unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(());
    }

    if let Some((a, b)) = stats_diff {
        if !rest.is_empty() {
            return Err(format!("unexpected arguments: {}", rest.join(" ")));
        }
        let a = resolve_path(&workspace_root, a);
        let b = resolve_path(&workspace_root, b);
        let a = resolve::maybe_resolve_base_or_session_out_dir_to_latest_bundle_dir(&a);
        let b = resolve::maybe_resolve_base_or_session_out_dir_to_latest_bundle_dir(&b);
        let a_bundle_path =
            prefer_schema2_sibling_for_bundle_json_path(&resolve_bundle_artifact_path(&a));
        let b_bundle_path =
            prefer_schema2_sibling_for_bundle_json_path(&resolve_bundle_artifact_path(&b));
        let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);
        let report = bundle_stats_diff_from_paths(
            &a_bundle_path,
            &b_bundle_path,
            stats_top,
            sort,
            BundleStatsOptions { warmup_frames },
        )?;
        if stats_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report.to_json())
                    .unwrap_or_else(|_| "{}".to_string())
            );
        } else {
            report.print_human();
        }
        return Ok(());
    }

    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle artifact path (try: fretboard diag stats <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = resolve_path(&workspace_root, PathBuf::from(src));
    let src = resolve::maybe_resolve_base_or_session_out_dir_to_latest_bundle_dir(&src);
    let bundle_path = resolve_bundle_artifact_path(&src);
    let bundle_path = prefer_schema2_sibling_for_bundle_json_path(&bundle_path);
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
            serde_json::to_string_pretty(&report.to_json()).unwrap_or_else(|_| "{}".to_string())
        );
    } else {
        if stats_verbose {
            report.print_human(&bundle_path);
        } else {
            report.print_human_brief(&bundle_path);
        }
    }

    let derived_from_frames_index = report.derived_from_frames_index();

    fn ensure_check_supported_in_stats_mode(
        derived_from_frames_index: bool,
        check_name: &str,
        bundle_path: &Path,
        warmup_frames: u64,
    ) -> Result<(), String> {
        if !derived_from_frames_index {
            return Ok(());
        }

        if stats_lite_support_for(check_name).is_some() {
            return Ok(());
        }

        fn stats_lite_supported_checks_table() -> String {
            let mut rows: Vec<(StatsLiteCheckKind, &'static str, &'static str)> =
                STATS_LITE_SUPPORTED_CHECKS
                    .iter()
                    .map(|c| (c.kind, c.check_name, c.note))
                    .collect();
            rows.sort_by(|a, b| a.1.cmp(b.1));

            let mut out = String::new();
            out.push_str("  stats-lite supported checks:\n");
            for (kind, name, note) in rows {
                let kind = kind.as_str();
                out.push_str(&format!("    - {name} ({kind}): {note}\n"));
            }
            out
        }

        Err(format!(
            "`diag stats --{check_name}` is not stats-lite compatible yet, but this run used a stats-lite report derived from frames.index.json (bundle too large to materialize).\n\
  bundle: {}\n\
{}\n\
  hint: regenerate schema2 + sidecars, then re-run the check:\n\
    - fretboard diag doctor --fix-schema2 <bundle_dir> --warmup-frames {warmup_frames}\n\
    - fretboard diag index <bundle_dir> --warmup-frames {warmup_frames}\n\
    - fretboard diag stats <bundle_dir> --warmup-frames {warmup_frames} --{check_name} ...",
            bundle_path.display(),
            stats_lite_supported_checks_table(),
        ))
    }
    if let Some(test_id) = check_stale_paint_test_id.as_deref() {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-stale-paint",
            &bundle_path,
            warmup_frames,
        )?;
        if derived_from_frames_index {
            stats::check_bundle_for_stale_paint_streaming(
                &bundle_path,
                test_id,
                check_stale_paint_eps,
            )?;
        } else {
            stats::check_bundle_for_stale_paint(&bundle_path, test_id, check_stale_paint_eps)?;
        }
    }
    if let Some(test_id) = check_stale_scene_test_id.as_deref() {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-stale-scene",
            &bundle_path,
            warmup_frames,
        )?;
        if derived_from_frames_index {
            stats::check_bundle_for_stale_scene_streaming(
                &bundle_path,
                test_id,
                check_stale_scene_eps,
            )?;
        } else {
            stats::check_bundle_for_stale_scene(&bundle_path, test_id, check_stale_scene_eps)?;
        }
    }
    if let Some(min) = check_idle_no_paint_min {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-idle-no-paint-min",
            &bundle_path,
            warmup_frames,
        )?;
        let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
        let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
        if derived_from_frames_index {
            stats::check_frames_index_for_idle_no_paint_min(
                &bundle_path,
                out_dir,
                min,
                warmup_frames,
            )?;
        } else {
            stats::check_bundle_for_idle_no_paint_min(&bundle_path, out_dir, min, warmup_frames)?;
        }
    }
    if let Some(test_id) = check_pixels_changed_test_id.as_deref() {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-pixels-changed",
            &bundle_path,
            warmup_frames,
        )?;
        let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
        let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
        stats::check_out_dir_for_pixels_changed(out_dir, test_id, warmup_frames)?;
    }
    if let Some(test_id) = check_pixels_unchanged_test_id.as_deref() {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-pixels-unchanged",
            &bundle_path,
            warmup_frames,
        )?;
        let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
        let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
        stats::check_out_dir_for_pixels_unchanged(out_dir, test_id, warmup_frames)?;
    }
    if check_semantics_changed_repainted {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-semantics-changed-repainted",
            &bundle_path,
            warmup_frames,
        )?;
        if derived_from_frames_index {
            stats::check_bundle_for_semantics_changed_repainted_streaming(
                &bundle_path,
                warmup_frames,
                dump_semantics_changed_repainted_json,
            )?;
        } else {
            stats::check_bundle_for_semantics_changed_repainted(
                &bundle_path,
                warmup_frames,
                dump_semantics_changed_repainted_json,
            )?;
        }
    }
    if let Some(test_id) = check_wheel_scroll_test_id.as_deref() {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-wheel-scroll",
            &bundle_path,
            warmup_frames,
        )?;
        stats::check_bundle_for_wheel_scroll(bundle_path.as_path(), test_id, warmup_frames)?;
    }
    if let Some(test_id) = check_wheel_scroll_hit_changes_test_id.as_deref() {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-wheel-scroll-hit-changes",
            &bundle_path,
            warmup_frames,
        )?;
        stats::check_bundle_for_wheel_scroll_hit_changes(
            bundle_path.as_path(),
            test_id,
            warmup_frames,
        )?;
    }
    if let Some(test_id) = check_drag_cache_root_paint_only_test_id.as_deref() {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-drag-cache-root-paint-only",
            &bundle_path,
            warmup_frames,
        )?;
        stats::check_bundle_for_drag_cache_root_paint_only(&bundle_path, test_id, warmup_frames)?;
    }
    if let Some(max_allowed) = check_hover_layout_max {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-hover-layout-max",
            &bundle_path,
            warmup_frames,
        )?;
        check_report_for_hover_layout_invalidations(&report, max_allowed)?;
    }
    if check_gc_sweep_liveness {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-gc-sweep-liveness",
            &bundle_path,
            warmup_frames,
        )?;
        if derived_from_frames_index {
            stats::check_bundle_for_gc_sweep_liveness_streaming(
                bundle_path.as_path(),
                warmup_frames,
            )?;
        } else {
            stats::check_bundle_for_gc_sweep_liveness(bundle_path.as_path(), warmup_frames)?;
        }
    }
    if !check_notify_hotspot_file_max.is_empty() {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-notify-hotspot-file-max",
            &bundle_path,
            warmup_frames,
        )?;
    }
    for (file, max) in &check_notify_hotspot_file_max {
        stats::check_bundle_for_notify_hotspot_file_max(
            bundle_path.as_path(),
            file.as_str(),
            *max,
            warmup_frames,
        )?;
    }
    if let Some(min) = check_view_cache_reuse_stable_min
        && min > 0
    {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-view-cache-reuse-stable-min",
            &bundle_path,
            warmup_frames,
        )?;
        let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;
        let out_dir = bundle_dir.parent().unwrap_or_else(|| Path::new("."));
        if derived_from_frames_index {
            stats::check_frames_index_for_view_cache_reuse_stable_min(
                bundle_path.as_path(),
                out_dir,
                min,
                warmup_frames,
            )?;
        } else {
            stats::check_bundle_for_view_cache_reuse_stable_min(
                bundle_path.as_path(),
                out_dir,
                min,
                warmup_frames,
            )?;
        }
    }
    if let Some(min) = check_view_cache_reuse_min
        && min > 0
    {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-view-cache-reuse-min",
            &bundle_path,
            warmup_frames,
        )?;
        if derived_from_frames_index {
            stats::check_frames_index_for_view_cache_reuse_min(
                bundle_path.as_path(),
                min,
                warmup_frames,
            )?;
        } else {
            stats::check_bundle_for_view_cache_reuse_min(
                bundle_path.as_path(),
                min,
                warmup_frames,
            )?;
        }
    }
    if let Some(min) = check_overlay_synthesis_min
        && min > 0
    {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-overlay-synthesis-min",
            &bundle_path,
            warmup_frames,
        )?;
        if derived_from_frames_index {
            stats::check_frames_index_for_overlay_synthesis_min(
                bundle_path.as_path(),
                min,
                warmup_frames,
            )?;
        } else {
            stats::check_bundle_for_overlay_synthesis_min(
                bundle_path.as_path(),
                min,
                warmup_frames,
            )?;
        }
    }
    if let Some(min) = check_viewport_input_min
        && min > 0
    {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-viewport-input-min",
            &bundle_path,
            warmup_frames,
        )?;
        if derived_from_frames_index {
            stats::check_frames_index_for_viewport_input_min(
                bundle_path.as_path(),
                min,
                warmup_frames,
            )?;
        } else {
            stats::check_bundle_for_viewport_input_min(bundle_path.as_path(), min, warmup_frames)?;
        }
    }
    if let Some(min) = check_dock_drag_min
        && min > 0
    {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-dock-drag-min",
            &bundle_path,
            warmup_frames,
        )?;
        if derived_from_frames_index {
            stats::check_frames_index_for_dock_drag_min(bundle_path.as_path(), min, warmup_frames)?;
        } else {
            stats::check_bundle_for_dock_drag_min(bundle_path.as_path(), min, warmup_frames)?;
        }
    }
    if let Some(min) = check_viewport_capture_min
        && min > 0
    {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-viewport-capture-min",
            &bundle_path,
            warmup_frames,
        )?;
        if derived_from_frames_index {
            stats::check_frames_index_for_viewport_capture_min(
                bundle_path.as_path(),
                min,
                warmup_frames,
            )?;
        } else {
            stats::check_bundle_for_viewport_capture_min(
                bundle_path.as_path(),
                min,
                warmup_frames,
            )?;
        }
    }
    if let Some(min) = check_retained_vlist_reconcile_no_notify_min
        && min > 0
    {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-retained-vlist-reconcile-no-notify-min",
            &bundle_path,
            warmup_frames,
        )?;
        stats::check_bundle_for_retained_vlist_reconcile_no_notify_min(
            bundle_path.as_path(),
            min,
            warmup_frames,
        )?;
    }
    if let Some(max_delta) = check_retained_vlist_attach_detach_max {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-retained-vlist-attach-detach-max",
            &bundle_path,
            warmup_frames,
        )?;
        stats::check_bundle_for_retained_vlist_attach_detach_max(
            bundle_path.as_path(),
            max_delta,
            warmup_frames,
        )?;
    }
    if let Some(min) = check_retained_vlist_keep_alive_reuse_min
        && min > 0
    {
        ensure_check_supported_in_stats_mode(
            derived_from_frames_index,
            "check-retained-vlist-keep-alive-reuse-min",
            &bundle_path,
            warmup_frames,
        )?;
        stats::check_bundle_for_retained_vlist_keep_alive_reuse_min(
            bundle_path.as_path(),
            min,
            warmup_frames,
        )?;
    }
    Ok(())
}
