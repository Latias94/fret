use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct TopTimesUs {
    pub total: u64,
    pub layout: u64,
    pub solve: u64,
}

impl TopTimesUs {
    pub(crate) fn new(total: u64, layout: u64, solve: u64) -> Self {
        Self {
            total,
            layout,
            solve,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PointerMoveMetrics {
    pub max_dispatch_time_us: u64,
    pub max_hit_test_time_us: u64,
    pub snapshots_with_global_changes: u64,
}

impl PointerMoveMetrics {
    pub(crate) fn new(
        max_dispatch_time_us: u64,
        max_hit_test_time_us: u64,
        snapshots_with_global_changes: u64,
    ) -> Self {
        Self {
            max_dispatch_time_us,
            max_hit_test_time_us,
            snapshots_with_global_changes,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PaintCacheReplayMetrics {
    pub hit_test_only_replay_allowed_max: u64,
    pub hit_test_only_replay_rejected_key_mismatch_max: u64,
}

impl PaintCacheReplayMetrics {
    pub(crate) fn new(
        hit_test_only_replay_allowed_max: u64,
        hit_test_only_replay_rejected_key_mismatch_max: u64,
    ) -> Self {
        Self {
            hit_test_only_replay_allowed_max,
            hit_test_only_replay_rejected_key_mismatch_max,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RendererTimesUs {
    pub encode_scene_us: u64,
    pub upload_us: u64,
    pub record_passes_us: u64,
    pub encoder_finish_us: u64,
    pub prepare_text_us: u64,
    pub prepare_svg_us: u64,
}

impl RendererTimesUs {
    pub(crate) fn new(
        encode_scene_us: u64,
        upload_us: u64,
        record_passes_us: u64,
        encoder_finish_us: u64,
        prepare_text_us: u64,
        prepare_svg_us: u64,
    ) -> Self {
        Self {
            encode_scene_us,
            upload_us,
            record_passes_us,
            encoder_finish_us,
            prepare_text_us,
            prepare_svg_us,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_perf_baseline_row_single(
    rows: &mut Vec<serde_json::Value>,
    script_key: &str,
    measured_max_top: TopTimesUs,
    measured_p90_top: TopTimesUs,
    measured_p95_top: TopTimesUs,
    measured_max_pointer_move: PointerMoveMetrics,
    measured_max_paint_cache: PaintCacheReplayMetrics,
    measured_max_renderer: RendererTimesUs,
    seed_total: PerfBaselineSeed,
    seed_layout: PerfBaselineSeed,
    seed_solve: PerfBaselineSeed,
    seed_total_value: u64,
    seed_layout_value: u64,
    seed_solve_value: u64,
    thr_total: u64,
    thr_layout: u64,
    thr_solve: u64,
    thr_pointer_move: PointerMoveMetrics,
    thr_min_hit_test_only_replay_allowed_max: u64,
    thr_max_hit_test_only_replay_rejected_key_mismatch_max: u64,
    thr_renderer: RendererTimesUs,
) {
    rows.push(serde_json::json!({
        "script": script_key.to_string(),
        "measured_max": {
            "top_total_time_us": measured_max_top.total,
            "top_layout_time_us": measured_max_top.layout,
            "top_layout_engine_solve_time_us": measured_max_top.solve,
            "pointer_move_max_dispatch_time_us": measured_max_pointer_move.max_dispatch_time_us,
            "pointer_move_max_hit_test_time_us": measured_max_pointer_move.max_hit_test_time_us,
            "pointer_move_snapshots_with_global_changes": measured_max_pointer_move.snapshots_with_global_changes,
            "run_paint_cache_hit_test_only_replay_allowed_max": measured_max_paint_cache.hit_test_only_replay_allowed_max,
            "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": measured_max_paint_cache.hit_test_only_replay_rejected_key_mismatch_max,
            "renderer_encode_scene_us": measured_max_renderer.encode_scene_us,
            "renderer_upload_us": measured_max_renderer.upload_us,
            "renderer_record_passes_us": measured_max_renderer.record_passes_us,
            "renderer_encoder_finish_us": measured_max_renderer.encoder_finish_us,
            "renderer_prepare_text_us": measured_max_renderer.prepare_text_us,
            "renderer_prepare_svg_us": measured_max_renderer.prepare_svg_us,
        },
        "measured_p90": {
            "top_total_time_us": measured_p90_top.total,
            "top_layout_time_us": measured_p90_top.layout,
            "top_layout_engine_solve_time_us": measured_p90_top.solve,
        },
        "measured_p95": {
            "top_total_time_us": measured_p95_top.total,
            "top_layout_time_us": measured_p95_top.layout,
            "top_layout_engine_solve_time_us": measured_p95_top.solve,
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
            "max_pointer_move_dispatch_us": thr_pointer_move.max_dispatch_time_us,
            "max_pointer_move_hit_test_us": thr_pointer_move.max_hit_test_time_us,
            "max_pointer_move_global_changes": thr_pointer_move.snapshots_with_global_changes,
            "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_min_hit_test_only_replay_allowed_max,
            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_max_hit_test_only_replay_rejected_key_mismatch_max,
            "max_renderer_encode_scene_us": thr_renderer.encode_scene_us,
            "max_renderer_upload_us": thr_renderer.upload_us,
            "max_renderer_record_passes_us": thr_renderer.record_passes_us,
            "max_renderer_encoder_finish_us": thr_renderer.encoder_finish_us,
            "max_renderer_prepare_text_us": thr_renderer.prepare_text_us,
            "max_renderer_prepare_svg_us": thr_renderer.prepare_svg_us,
        },
    }));
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_perf_baseline_row_repeat(
    rows: &mut Vec<serde_json::Value>,
    script_key: &str,
    measured_max_top: TopTimesUs,
    measured_max_frame_p95: TopTimesUs,
    measured_max_pointer_move: PointerMoveMetrics,
    measured_max_paint_cache: PaintCacheReplayMetrics,
    measured_max_renderer: RendererTimesUs,
    measured_p90_top: TopTimesUs,
    measured_p90_frame_p95: TopTimesUs,
    measured_p90_renderer: RendererTimesUs,
    measured_p95_top: TopTimesUs,
    measured_p95_frame_p95: TopTimesUs,
    measured_p95_renderer: RendererTimesUs,
    seed_total: PerfBaselineSeed,
    seed_layout: PerfBaselineSeed,
    seed_solve: PerfBaselineSeed,
    seed_frame_p95_total: PerfBaselineSeed,
    seed_frame_p95_layout: PerfBaselineSeed,
    seed_frame_p95_solve: PerfBaselineSeed,
    seed_total_value: u64,
    seed_layout_value: u64,
    seed_solve_value: u64,
    seed_frame_p95_total_value: u64,
    seed_frame_p95_layout_value: u64,
    seed_frame_p95_solve_value: u64,
    wants_frame_p95_thresholds: bool,
    thr_total: u64,
    thr_layout: u64,
    thr_solve: u64,
    thr_frame_p95_total: Option<u64>,
    thr_frame_p95_layout: Option<u64>,
    thr_frame_p95_solve: Option<u64>,
    thr_pointer_move: PointerMoveMetrics,
    thr_min_hit_test_only_replay_allowed_max: u64,
    thr_max_hit_test_only_replay_rejected_key_mismatch_max: u64,
    thr_renderer: RendererTimesUs,
) {
    rows.push(serde_json::json!({
        "script": script_key.to_string(),
        "measured_max": {
            "top_total_time_us": measured_max_top.total,
            "top_layout_time_us": measured_max_top.layout,
            "top_layout_engine_solve_time_us": measured_max_top.solve,
            "frame_p95_total_time_us": measured_max_frame_p95.total,
            "frame_p95_layout_time_us": measured_max_frame_p95.layout,
            "frame_p95_layout_engine_solve_time_us": measured_max_frame_p95.solve,
            "pointer_move_max_dispatch_time_us": measured_max_pointer_move.max_dispatch_time_us,
            "pointer_move_max_hit_test_time_us": measured_max_pointer_move.max_hit_test_time_us,
            "pointer_move_snapshots_with_global_changes": measured_max_pointer_move.snapshots_with_global_changes,
            "run_paint_cache_hit_test_only_replay_allowed_max": measured_max_paint_cache.hit_test_only_replay_allowed_max,
            "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": measured_max_paint_cache.hit_test_only_replay_rejected_key_mismatch_max,
            "renderer_encode_scene_us": measured_max_renderer.encode_scene_us,
            "renderer_upload_us": measured_max_renderer.upload_us,
            "renderer_record_passes_us": measured_max_renderer.record_passes_us,
            "renderer_encoder_finish_us": measured_max_renderer.encoder_finish_us,
            "renderer_prepare_text_us": measured_max_renderer.prepare_text_us,
            "renderer_prepare_svg_us": measured_max_renderer.prepare_svg_us,
        },
        "measured_p90": {
            "top_total_time_us": measured_p90_top.total,
            "top_layout_time_us": measured_p90_top.layout,
            "top_layout_engine_solve_time_us": measured_p90_top.solve,
            "frame_p95_total_time_us": measured_p90_frame_p95.total,
            "frame_p95_layout_time_us": measured_p90_frame_p95.layout,
            "frame_p95_layout_engine_solve_time_us": measured_p90_frame_p95.solve,
            "renderer_encode_scene_us": measured_p90_renderer.encode_scene_us,
            "renderer_upload_us": measured_p90_renderer.upload_us,
            "renderer_record_passes_us": measured_p90_renderer.record_passes_us,
            "renderer_encoder_finish_us": measured_p90_renderer.encoder_finish_us,
            "renderer_prepare_text_us": measured_p90_renderer.prepare_text_us,
            "renderer_prepare_svg_us": measured_p90_renderer.prepare_svg_us,
        },
        "measured_p95": {
            "top_total_time_us": measured_p95_top.total,
            "top_layout_time_us": measured_p95_top.layout,
            "top_layout_engine_solve_time_us": measured_p95_top.solve,
            "frame_p95_total_time_us": measured_p95_frame_p95.total,
            "frame_p95_layout_time_us": measured_p95_frame_p95.layout,
            "frame_p95_layout_engine_solve_time_us": measured_p95_frame_p95.solve,
            "renderer_encode_scene_us": measured_p95_renderer.encode_scene_us,
            "renderer_upload_us": measured_p95_renderer.upload_us,
            "renderer_record_passes_us": measured_p95_renderer.record_passes_us,
            "renderer_encoder_finish_us": measured_p95_renderer.encoder_finish_us,
            "renderer_prepare_text_us": measured_p95_renderer.prepare_text_us,
            "renderer_prepare_svg_us": measured_p95_renderer.prepare_svg_us,
        },
        "threshold_seed": {
            "top_total_time_us": seed_total_value,
            "top_layout_time_us": seed_layout_value,
            "top_layout_engine_solve_time_us": seed_solve_value,
            "frame_p95_total_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_total_value),
            "frame_p95_layout_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_layout_value),
            "frame_p95_layout_engine_solve_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_solve_value),
        },
        "threshold_seed_source": {
            "top_total_time_us": seed_total.as_str(),
            "top_layout_time_us": seed_layout.as_str(),
            "top_layout_engine_solve_time_us": seed_solve.as_str(),
            "frame_p95_total_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_total.as_str()),
            "frame_p95_layout_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_layout.as_str()),
            "frame_p95_layout_engine_solve_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_solve.as_str()),
        },
        "thresholds": {
            "max_top_total_us": (!wants_frame_p95_thresholds).then_some(thr_total),
            "max_top_layout_us": (!wants_frame_p95_thresholds).then_some(thr_layout),
            "max_top_solve_us": (!wants_frame_p95_thresholds).then_some(thr_solve),
            "max_frame_p95_total_us": thr_frame_p95_total,
            "max_frame_p95_layout_us": thr_frame_p95_layout,
            "max_frame_p95_solve_us": thr_frame_p95_solve,
            "max_pointer_move_dispatch_us": thr_pointer_move.max_dispatch_time_us,
            "max_pointer_move_hit_test_us": thr_pointer_move.max_hit_test_time_us,
            "max_pointer_move_global_changes": thr_pointer_move.snapshots_with_global_changes,
            "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_min_hit_test_only_replay_allowed_max,
            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_max_hit_test_only_replay_rejected_key_mismatch_max,
            "max_renderer_encode_scene_us": thr_renderer.encode_scene_us,
            "max_renderer_upload_us": thr_renderer.upload_us,
            "max_renderer_record_passes_us": thr_renderer.record_passes_us,
            "max_renderer_encoder_finish_us": thr_renderer.encoder_finish_us,
            "max_renderer_prepare_text_us": thr_renderer.prepare_text_us,
            "max_renderer_prepare_svg_us": thr_renderer.prepare_svg_us,
        },
    }));
}

