use super::*;

#[derive(Debug, Clone)]
pub(crate) struct PerfBaselineFromBundlesContext {
    pub pack_after_run: bool,
    pub rest: Vec<String>,
    pub workspace_root: PathBuf,
    pub sort_override: Option<BundleStatsSort>,
    pub perf_baseline_out: Option<PathBuf>,
    pub perf_baseline_headroom_pct: u32,
    pub warmup_frames: u64,
    pub stats_json: bool,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_perf_baseline_from_bundles(
    ctx: PerfBaselineFromBundlesContext,
) -> Result<(), String> {
    let PerfBaselineFromBundlesContext {
        pack_after_run,
        rest,
        workspace_root,
        sort_override,
        perf_baseline_out,
        perf_baseline_headroom_pct,
        warmup_frames,
        stats_json,
    } = ctx;

    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    if rest.len() < 2 {
        return Err(
            "missing script path and bundle artifact paths (try: fretboard diag perf-baseline-from-bundles tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json .fret/diag/exports/1234)".to_string(),
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
    let mut measured_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: u64 = 0;
    let mut measured_max_renderer_encode_scene_us: u64 = 0;
    let mut measured_max_renderer_upload_us: u64 = 0;
    let mut measured_max_renderer_record_passes_us: u64 = 0;
    let mut measured_max_renderer_encoder_finish_us: u64 = 0;
    let mut measured_max_renderer_prepare_text_us: u64 = 0;
    let mut measured_max_renderer_prepare_svg_us: u64 = 0;

    let mut worst_bundle: Option<(u64, PathBuf)> = None;

    for raw in rest.iter().skip(1) {
        let bundle_src = resolve_path(&workspace_root, PathBuf::from(raw));
        let bundle_path = resolve_bundle_artifact_path(&bundle_src);
        if !bundle_path.is_file() {
            return Err(format!(
                "path does not contain a bundle artifact (bundle.json or bundle.schema2.json): {}",
                bundle_src.display()
            ));
        }

        let report =
            bundle_stats_from_path(&bundle_path, 1, sort, BundleStatsOptions { warmup_frames })?;

        let top = report.top.first();
        let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
        let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
        let top_solve = top.map(|r| r.layout_engine_solve_time_us).unwrap_or(0);

        let pointer_move_dispatch = report.pointer_move_max_dispatch_time_us;
        let pointer_move_hit_test = report.pointer_move_max_hit_test_time_us;
        let pointer_move_global_changes = report.pointer_move_snapshots_with_global_changes as u64;
        let (
            run_paint_cache_hit_test_only_replay_allowed_max,
            run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        ) = bundle_paint_cache_hit_test_only_replay_maxes(&bundle_path, warmup_frames)?;
        let renderer_encode_scene_us = report.max_renderer_encode_scene_us;
        let renderer_upload_us = report.max_renderer_upload_us;
        let renderer_record_passes_us = report.max_renderer_record_passes_us;
        let renderer_encoder_finish_us = report.max_renderer_encoder_finish_us;
        let renderer_prepare_text_us = report.max_renderer_prepare_text_us;
        let renderer_prepare_svg_us = report.max_renderer_prepare_svg_us;

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
        measured_max_renderer_encode_scene_us =
            measured_max_renderer_encode_scene_us.max(renderer_encode_scene_us);
        measured_max_renderer_upload_us = measured_max_renderer_upload_us.max(renderer_upload_us);
        measured_max_renderer_record_passes_us =
            measured_max_renderer_record_passes_us.max(renderer_record_passes_us);
        measured_max_renderer_encoder_finish_us =
            measured_max_renderer_encoder_finish_us.max(renderer_encoder_finish_us);
        measured_max_renderer_prepare_text_us =
            measured_max_renderer_prepare_text_us.max(renderer_prepare_text_us);
        measured_max_renderer_prepare_svg_us =
            measured_max_renderer_prepare_svg_us.max(renderer_prepare_svg_us);

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
        "max_renderer_encode_scene_us": apply_perf_baseline_headroom(measured_max_renderer_encode_scene_us, perf_baseline_headroom_pct),
        "max_renderer_upload_us": apply_perf_baseline_headroom(measured_max_renderer_upload_us, perf_baseline_headroom_pct),
        "max_renderer_record_passes_us": apply_perf_baseline_headroom(measured_max_renderer_record_passes_us, perf_baseline_headroom_pct),
        "max_renderer_encoder_finish_us": apply_perf_baseline_headroom(measured_max_renderer_encoder_finish_us, perf_baseline_headroom_pct),
        "max_renderer_prepare_text_us": apply_perf_baseline_headroom(measured_max_renderer_prepare_text_us, perf_baseline_headroom_pct),
        "max_renderer_prepare_svg_us": apply_perf_baseline_headroom(measured_max_renderer_prepare_svg_us, perf_baseline_headroom_pct),
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
        "renderer_encode_scene_us": measured_max_renderer_encode_scene_us,
        "renderer_upload_us": measured_max_renderer_upload_us,
        "renderer_record_passes_us": measured_max_renderer_record_passes_us,
        "renderer_encoder_finish_us": measured_max_renderer_encoder_finish_us,
        "renderer_prepare_text_us": measured_max_renderer_prepare_text_us,
        "renderer_prepare_svg_us": measured_max_renderer_prepare_svg_us,
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
