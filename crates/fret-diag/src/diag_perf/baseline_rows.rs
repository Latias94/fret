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

#[derive(Debug, Clone, Copy)]
pub(crate) struct RendererPayloadMetrics {
    pub instance_bytes: u64,
    pub encode_scene_text_ops: u64,
}

impl RendererPayloadMetrics {
    pub(crate) fn new(instance_bytes: u64, encode_scene_text_ops: u64) -> Self {
        Self {
            instance_bytes,
            encode_scene_text_ops,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RendererSeedSelection {
    pub encode_scene_us: PerfBaselineSeed,
    pub upload_us: PerfBaselineSeed,
    pub record_passes_us: PerfBaselineSeed,
    pub encoder_finish_us: PerfBaselineSeed,
    pub prepare_text_us: PerfBaselineSeed,
    pub prepare_svg_us: PerfBaselineSeed,
}

impl RendererSeedSelection {
    pub(crate) fn new(
        encode_scene_us: PerfBaselineSeed,
        upload_us: PerfBaselineSeed,
        record_passes_us: PerfBaselineSeed,
        encoder_finish_us: PerfBaselineSeed,
        prepare_text_us: PerfBaselineSeed,
        prepare_svg_us: PerfBaselineSeed,
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
    threshold_surface: PerfBaselineThresholdSurface,
    measured_max_top: TopTimesUs,
    measured_p50_top: TopTimesUs,
    measured_p90_top: TopTimesUs,
    measured_p95_top: TopTimesUs,
    measured_max_pointer_move: PointerMoveMetrics,
    measured_max_paint_cache: PaintCacheReplayMetrics,
    measured_max_renderer: RendererTimesUs,
    measured_max_renderer_payload: RendererPayloadMetrics,
    seed_total: PerfBaselineSeed,
    seed_layout: PerfBaselineSeed,
    seed_solve: PerfBaselineSeed,
    seed_total_value: u64,
    seed_layout_value: u64,
    seed_solve_value: u64,
    seed_renderer: RendererSeedSelection,
    seed_renderer_value: RendererTimesUs,
    seed_renderer_payload_value: RendererPayloadMetrics,
    thr_total: u64,
    thr_layout: u64,
    thr_solve: u64,
    thr_pointer_move: PointerMoveMetrics,
    thr_min_hit_test_only_replay_allowed_max: u64,
    thr_max_hit_test_only_replay_rejected_key_mismatch_max: u64,
    thr_renderer: RendererTimesUs,
    thr_renderer_payload: RendererPayloadMetrics,
) {
    let wants_ui_thresholds = threshold_surface.includes_ui();
    let wants_renderer_thresholds = threshold_surface.includes_renderer();
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
            "renderer_instance_bytes": measured_max_renderer_payload.instance_bytes,
            "renderer_encode_scene_text_ops": measured_max_renderer_payload.encode_scene_text_ops,
        },
        "measured_p50": {
            "top_total_time_us": measured_p50_top.total,
            "top_layout_time_us": measured_p50_top.layout,
            "top_layout_engine_solve_time_us": measured_p50_top.solve,
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
            "top_total_time_us": wants_ui_thresholds.then_some(seed_total_value),
            "top_layout_time_us": wants_ui_thresholds.then_some(seed_layout_value),
            "top_layout_engine_solve_time_us": wants_ui_thresholds.then_some(seed_solve_value),
            "renderer_encode_scene_us": wants_renderer_thresholds.then_some(seed_renderer_value.encode_scene_us),
            "renderer_upload_us": wants_renderer_thresholds.then_some(seed_renderer_value.upload_us),
            "renderer_record_passes_us": wants_renderer_thresholds.then_some(seed_renderer_value.record_passes_us),
            "renderer_encoder_finish_us": wants_renderer_thresholds.then_some(seed_renderer_value.encoder_finish_us),
            "renderer_prepare_text_us": wants_renderer_thresholds.then_some(seed_renderer_value.prepare_text_us),
            "renderer_prepare_svg_us": wants_renderer_thresholds.then_some(seed_renderer_value.prepare_svg_us),
            "renderer_instance_bytes": wants_renderer_thresholds.then_some(seed_renderer_payload_value.instance_bytes),
            "renderer_encode_scene_text_ops": wants_renderer_thresholds.then_some(seed_renderer_payload_value.encode_scene_text_ops),
        },
        "threshold_seed_source": {
            "top_total_time_us": wants_ui_thresholds.then_some(seed_total.as_str()),
            "top_layout_time_us": wants_ui_thresholds.then_some(seed_layout.as_str()),
            "top_layout_engine_solve_time_us": wants_ui_thresholds.then_some(seed_solve.as_str()),
            "renderer_encode_scene_us": wants_renderer_thresholds.then_some(seed_renderer.encode_scene_us.as_str()),
            "renderer_upload_us": wants_renderer_thresholds.then_some(seed_renderer.upload_us.as_str()),
            "renderer_record_passes_us": wants_renderer_thresholds.then_some(seed_renderer.record_passes_us.as_str()),
            "renderer_encoder_finish_us": wants_renderer_thresholds.then_some(seed_renderer.encoder_finish_us.as_str()),
            "renderer_prepare_text_us": wants_renderer_thresholds.then_some(seed_renderer.prepare_text_us.as_str()),
            "renderer_prepare_svg_us": wants_renderer_thresholds.then_some(seed_renderer.prepare_svg_us.as_str()),
            "renderer_instance_bytes": wants_renderer_thresholds.then_some(PerfBaselineSeed::Max.as_str()),
            "renderer_encode_scene_text_ops": wants_renderer_thresholds.then_some(PerfBaselineSeed::Max.as_str()),
        },
        "thresholds": {
            "max_top_total_us": wants_ui_thresholds.then_some(thr_total),
            "max_top_layout_us": wants_ui_thresholds.then_some(thr_layout),
            "max_top_solve_us": wants_ui_thresholds.then_some(thr_solve),
            "max_pointer_move_dispatch_us": wants_ui_thresholds.then_some(thr_pointer_move.max_dispatch_time_us),
            "max_pointer_move_hit_test_us": wants_ui_thresholds.then_some(thr_pointer_move.max_hit_test_time_us),
            "max_pointer_move_global_changes": wants_ui_thresholds.then_some(thr_pointer_move.snapshots_with_global_changes),
            "min_run_paint_cache_hit_test_only_replay_allowed_max": wants_ui_thresholds.then_some(thr_min_hit_test_only_replay_allowed_max),
            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": wants_ui_thresholds.then_some(thr_max_hit_test_only_replay_rejected_key_mismatch_max),
            "max_renderer_encode_scene_us": wants_renderer_thresholds.then_some(thr_renderer.encode_scene_us),
            "max_renderer_upload_us": wants_renderer_thresholds.then_some(thr_renderer.upload_us),
            "max_renderer_record_passes_us": wants_renderer_thresholds.then_some(thr_renderer.record_passes_us),
            "max_renderer_encoder_finish_us": wants_renderer_thresholds.then_some(thr_renderer.encoder_finish_us),
            "max_renderer_prepare_text_us": wants_renderer_thresholds.then_some(thr_renderer.prepare_text_us),
            "max_renderer_prepare_svg_us": wants_renderer_thresholds.then_some(thr_renderer.prepare_svg_us),
            "max_renderer_instance_bytes": wants_renderer_thresholds.then_some(thr_renderer_payload.instance_bytes),
            "max_renderer_encode_scene_text_ops": wants_renderer_thresholds.then_some(thr_renderer_payload.encode_scene_text_ops),
        },
    }));
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_perf_baseline_row_repeat(
    rows: &mut Vec<serde_json::Value>,
    script_key: &str,
    threshold_surface: PerfBaselineThresholdSurface,
    measured_max_top: TopTimesUs,
    measured_max_frame_p95: TopTimesUs,
    measured_max_pointer_move: PointerMoveMetrics,
    measured_max_paint_cache: PaintCacheReplayMetrics,
    measured_max_renderer: RendererTimesUs,
    measured_max_renderer_payload: RendererPayloadMetrics,
    measured_p50_top: TopTimesUs,
    measured_p50_frame_p95: TopTimesUs,
    measured_p50_renderer: RendererTimesUs,
    measured_p50_renderer_payload: RendererPayloadMetrics,
    measured_p90_top: TopTimesUs,
    measured_p90_frame_p95: TopTimesUs,
    measured_p90_renderer: RendererTimesUs,
    measured_p90_renderer_payload: RendererPayloadMetrics,
    measured_p95_top: TopTimesUs,
    measured_p95_frame_p95: TopTimesUs,
    measured_p95_renderer: RendererTimesUs,
    measured_p95_renderer_payload: RendererPayloadMetrics,
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
    seed_renderer: RendererSeedSelection,
    seed_renderer_value: RendererTimesUs,
    seed_renderer_payload_value: RendererPayloadMetrics,
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
    thr_renderer_payload: RendererPayloadMetrics,
) {
    let wants_ui_thresholds = threshold_surface.includes_ui();
    let wants_renderer_thresholds = threshold_surface.includes_renderer();
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
            "renderer_instance_bytes": measured_max_renderer_payload.instance_bytes,
            "renderer_encode_scene_text_ops": measured_max_renderer_payload.encode_scene_text_ops,
        },
        "measured_p50": {
            "top_total_time_us": measured_p50_top.total,
            "top_layout_time_us": measured_p50_top.layout,
            "top_layout_engine_solve_time_us": measured_p50_top.solve,
            "frame_p95_total_time_us": measured_p50_frame_p95.total,
            "frame_p95_layout_time_us": measured_p50_frame_p95.layout,
            "frame_p95_layout_engine_solve_time_us": measured_p50_frame_p95.solve,
            "renderer_encode_scene_us": measured_p50_renderer.encode_scene_us,
            "renderer_upload_us": measured_p50_renderer.upload_us,
            "renderer_record_passes_us": measured_p50_renderer.record_passes_us,
            "renderer_encoder_finish_us": measured_p50_renderer.encoder_finish_us,
            "renderer_prepare_text_us": measured_p50_renderer.prepare_text_us,
            "renderer_prepare_svg_us": measured_p50_renderer.prepare_svg_us,
            "renderer_instance_bytes": measured_p50_renderer_payload.instance_bytes,
            "renderer_encode_scene_text_ops": measured_p50_renderer_payload.encode_scene_text_ops,
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
            "renderer_instance_bytes": measured_p90_renderer_payload.instance_bytes,
            "renderer_encode_scene_text_ops": measured_p90_renderer_payload.encode_scene_text_ops,
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
            "renderer_instance_bytes": measured_p95_renderer_payload.instance_bytes,
            "renderer_encode_scene_text_ops": measured_p95_renderer_payload.encode_scene_text_ops,
        },
        "threshold_seed": {
            "top_total_time_us": wants_ui_thresholds.then_some(seed_total_value),
            "top_layout_time_us": wants_ui_thresholds.then_some(seed_layout_value),
            "top_layout_engine_solve_time_us": wants_ui_thresholds.then_some(seed_solve_value),
            "frame_p95_total_time_us": (wants_ui_thresholds && wants_frame_p95_thresholds).then_some(seed_frame_p95_total_value),
            "frame_p95_layout_time_us": (wants_ui_thresholds && wants_frame_p95_thresholds).then_some(seed_frame_p95_layout_value),
            "frame_p95_layout_engine_solve_time_us": (wants_ui_thresholds && wants_frame_p95_thresholds).then_some(seed_frame_p95_solve_value),
            "renderer_encode_scene_us": wants_renderer_thresholds.then_some(seed_renderer_value.encode_scene_us),
            "renderer_upload_us": wants_renderer_thresholds.then_some(seed_renderer_value.upload_us),
            "renderer_record_passes_us": wants_renderer_thresholds.then_some(seed_renderer_value.record_passes_us),
            "renderer_encoder_finish_us": wants_renderer_thresholds.then_some(seed_renderer_value.encoder_finish_us),
            "renderer_prepare_text_us": wants_renderer_thresholds.then_some(seed_renderer_value.prepare_text_us),
            "renderer_prepare_svg_us": wants_renderer_thresholds.then_some(seed_renderer_value.prepare_svg_us),
            "renderer_instance_bytes": wants_renderer_thresholds.then_some(seed_renderer_payload_value.instance_bytes),
            "renderer_encode_scene_text_ops": wants_renderer_thresholds.then_some(seed_renderer_payload_value.encode_scene_text_ops),
        },
        "threshold_seed_source": {
            "top_total_time_us": wants_ui_thresholds.then_some(seed_total.as_str()),
            "top_layout_time_us": wants_ui_thresholds.then_some(seed_layout.as_str()),
            "top_layout_engine_solve_time_us": wants_ui_thresholds.then_some(seed_solve.as_str()),
            "frame_p95_total_time_us": (wants_ui_thresholds && wants_frame_p95_thresholds).then_some(seed_frame_p95_total.as_str()),
            "frame_p95_layout_time_us": (wants_ui_thresholds && wants_frame_p95_thresholds).then_some(seed_frame_p95_layout.as_str()),
            "frame_p95_layout_engine_solve_time_us": (wants_ui_thresholds && wants_frame_p95_thresholds).then_some(seed_frame_p95_solve.as_str()),
            "renderer_encode_scene_us": wants_renderer_thresholds.then_some(seed_renderer.encode_scene_us.as_str()),
            "renderer_upload_us": wants_renderer_thresholds.then_some(seed_renderer.upload_us.as_str()),
            "renderer_record_passes_us": wants_renderer_thresholds.then_some(seed_renderer.record_passes_us.as_str()),
            "renderer_encoder_finish_us": wants_renderer_thresholds.then_some(seed_renderer.encoder_finish_us.as_str()),
            "renderer_prepare_text_us": wants_renderer_thresholds.then_some(seed_renderer.prepare_text_us.as_str()),
            "renderer_prepare_svg_us": wants_renderer_thresholds.then_some(seed_renderer.prepare_svg_us.as_str()),
            "renderer_instance_bytes": wants_renderer_thresholds.then_some(PerfBaselineSeed::Max.as_str()),
            "renderer_encode_scene_text_ops": wants_renderer_thresholds.then_some(PerfBaselineSeed::Max.as_str()),
        },
        "thresholds": {
            "max_top_total_us": (wants_ui_thresholds && !wants_frame_p95_thresholds).then_some(thr_total),
            "max_top_layout_us": (wants_ui_thresholds && !wants_frame_p95_thresholds).then_some(thr_layout),
            "max_top_solve_us": (wants_ui_thresholds && !wants_frame_p95_thresholds).then_some(thr_solve),
            "max_frame_p95_total_us": wants_ui_thresholds.then_some(thr_frame_p95_total).flatten(),
            "max_frame_p95_layout_us": wants_ui_thresholds.then_some(thr_frame_p95_layout).flatten(),
            "max_frame_p95_solve_us": wants_ui_thresholds.then_some(thr_frame_p95_solve).flatten(),
            "max_pointer_move_dispatch_us": wants_ui_thresholds.then_some(thr_pointer_move.max_dispatch_time_us),
            "max_pointer_move_hit_test_us": wants_ui_thresholds.then_some(thr_pointer_move.max_hit_test_time_us),
            "max_pointer_move_global_changes": wants_ui_thresholds.then_some(thr_pointer_move.snapshots_with_global_changes),
            "min_run_paint_cache_hit_test_only_replay_allowed_max": wants_ui_thresholds.then_some(thr_min_hit_test_only_replay_allowed_max),
            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": wants_ui_thresholds.then_some(thr_max_hit_test_only_replay_rejected_key_mismatch_max),
            "max_renderer_encode_scene_us": wants_renderer_thresholds.then_some(thr_renderer.encode_scene_us),
            "max_renderer_upload_us": wants_renderer_thresholds.then_some(thr_renderer.upload_us),
            "max_renderer_record_passes_us": wants_renderer_thresholds.then_some(thr_renderer.record_passes_us),
            "max_renderer_encoder_finish_us": wants_renderer_thresholds.then_some(thr_renderer.encoder_finish_us),
            "max_renderer_prepare_text_us": wants_renderer_thresholds.then_some(thr_renderer.prepare_text_us),
            "max_renderer_prepare_svg_us": wants_renderer_thresholds.then_some(thr_renderer.prepare_svg_us),
            "max_renderer_instance_bytes": wants_renderer_thresholds.then_some(thr_renderer_payload.instance_bytes),
            "max_renderer_encode_scene_text_ops": wants_renderer_thresholds.then_some(thr_renderer_payload.encode_scene_text_ops),
        },
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer(dispatch: u64, hit_test: u64, global_changes: u64) -> PointerMoveMetrics {
        PointerMoveMetrics::new(dispatch, hit_test, global_changes)
    }

    fn paint_cache(allowed: u64, rejected: u64) -> PaintCacheReplayMetrics {
        PaintCacheReplayMetrics::new(allowed, rejected)
    }

    fn renderer(base: u64) -> RendererTimesUs {
        RendererTimesUs::new(base, base + 1, base + 2, base + 3, base + 4, base + 5)
    }

    fn payload(base: u64) -> RendererPayloadMetrics {
        RendererPayloadMetrics::new(base, base + 1)
    }

    fn renderer_seed(seed: PerfBaselineSeed) -> RendererSeedSelection {
        RendererSeedSelection::new(seed, seed, seed, seed, seed, seed)
    }

    #[test]
    fn single_baseline_row_records_measured_p50() {
        let mut rows = Vec::new();

        push_perf_baseline_row_single(
            &mut rows,
            "tools/diag-scripts/example.json",
            PerfBaselineThresholdSurface::All,
            TopTimesUs::new(30, 20, 10),
            TopTimesUs::new(30, 20, 10),
            TopTimesUs::new(30, 20, 10),
            TopTimesUs::new(30, 20, 10),
            pointer(1, 2, 0),
            paint_cache(3, 4),
            renderer(5),
            payload(50),
            PerfBaselineSeed::Max,
            PerfBaselineSeed::Max,
            PerfBaselineSeed::Max,
            30,
            20,
            10,
            renderer_seed(PerfBaselineSeed::Max),
            renderer(5),
            payload(50),
            36,
            24,
            12,
            pointer(2, 3, 0),
            2,
            5,
            renderer(7),
            payload(70),
        );

        assert_eq!(
            rows[0]["measured_p50"],
            serde_json::json!({
                "top_total_time_us": 30,
                "top_layout_time_us": 20,
                "top_layout_engine_solve_time_us": 10,
            })
        );
    }

    #[test]
    fn repeat_baseline_row_records_measured_p50() {
        let mut rows = Vec::new();

        push_perf_baseline_row_repeat(
            &mut rows,
            "tools/diag-scripts/repeat.json",
            PerfBaselineThresholdSurface::All,
            TopTimesUs::new(100, 80, 60),
            TopTimesUs::new(40, 30, 20),
            pointer(10, 11, 0),
            paint_cache(12, 13),
            renderer(1000),
            payload(100),
            TopTimesUs::new(70, 50, 30),
            TopTimesUs::new(25, 20, 15),
            renderer(2000),
            payload(200),
            TopTimesUs::new(90, 70, 50),
            TopTimesUs::new(35, 30, 25),
            renderer(3000),
            payload(300),
            TopTimesUs::new(95, 75, 55),
            TopTimesUs::new(38, 33, 28),
            renderer(4000),
            payload(400),
            PerfBaselineSeed::Max,
            PerfBaselineSeed::Max,
            PerfBaselineSeed::Max,
            PerfBaselineSeed::P95,
            PerfBaselineSeed::P95,
            PerfBaselineSeed::P95,
            100,
            80,
            60,
            38,
            33,
            28,
            renderer_seed(PerfBaselineSeed::Max),
            renderer(1000),
            payload(100),
            true,
            120,
            96,
            72,
            Some(46),
            Some(40),
            Some(34),
            pointer(12, 13, 0),
            10,
            16,
            renderer(5000),
            payload(500),
        );

        assert_eq!(
            rows[0]["measured_p50"],
            serde_json::json!({
                "top_total_time_us": 70,
                "top_layout_time_us": 50,
                "top_layout_engine_solve_time_us": 30,
                "frame_p95_total_time_us": 25,
                "frame_p95_layout_time_us": 20,
                "frame_p95_layout_engine_solve_time_us": 15,
                "renderer_encode_scene_us": 2000,
                "renderer_upload_us": 2001,
                "renderer_record_passes_us": 2002,
                "renderer_encoder_finish_us": 2003,
                "renderer_prepare_text_us": 2004,
                "renderer_prepare_svg_us": 2005,
                "renderer_instance_bytes": 200,
                "renderer_encode_scene_text_ops": 201,
            })
        );
    }

    #[test]
    fn ui_threshold_surface_keeps_renderer_measurements_but_omits_renderer_thresholds() {
        let mut rows = Vec::new();

        push_perf_baseline_row_repeat(
            &mut rows,
            "tools/diag-scripts/repeat.json",
            PerfBaselineThresholdSurface::Ui,
            TopTimesUs::new(100, 80, 60),
            TopTimesUs::new(40, 30, 20),
            pointer(10, 11, 0),
            paint_cache(12, 13),
            renderer(1000),
            payload(100),
            TopTimesUs::new(70, 50, 30),
            TopTimesUs::new(25, 20, 15),
            renderer(2000),
            payload(200),
            TopTimesUs::new(90, 70, 50),
            TopTimesUs::new(35, 30, 25),
            renderer(3000),
            payload(300),
            TopTimesUs::new(95, 75, 55),
            TopTimesUs::new(38, 33, 28),
            renderer(4000),
            payload(400),
            PerfBaselineSeed::Max,
            PerfBaselineSeed::Max,
            PerfBaselineSeed::Max,
            PerfBaselineSeed::P95,
            PerfBaselineSeed::P95,
            PerfBaselineSeed::P95,
            100,
            80,
            60,
            38,
            33,
            28,
            renderer_seed(PerfBaselineSeed::Max),
            renderer(1000),
            payload(100),
            false,
            120,
            96,
            72,
            None,
            None,
            None,
            pointer(12, 13, 0),
            10,
            16,
            renderer(5000),
            payload(500),
        );

        assert_eq!(rows[0]["measured_max"]["renderer_encode_scene_us"], 1000);
        assert_eq!(rows[0]["measured_max"]["renderer_instance_bytes"], 100);
        assert_eq!(rows[0]["measured_p95"]["renderer_prepare_svg_us"], 4005);
        assert_eq!(
            rows[0]["measured_p95"]["renderer_encode_scene_text_ops"],
            401
        );
        assert_eq!(rows[0]["thresholds"]["max_top_total_us"], 120);
        assert!(rows[0]["thresholds"]["max_renderer_encode_scene_us"].is_null());
        assert!(rows[0]["thresholds"]["max_renderer_prepare_svg_us"].is_null());
        assert!(rows[0]["thresholds"]["max_renderer_instance_bytes"].is_null());
    }

    #[test]
    fn renderer_threshold_surface_omits_ui_thresholds() {
        let mut rows = Vec::new();

        push_perf_baseline_row_single(
            &mut rows,
            "tools/diag-scripts/example.json",
            PerfBaselineThresholdSurface::Renderer,
            TopTimesUs::new(30, 20, 10),
            TopTimesUs::new(30, 20, 10),
            TopTimesUs::new(30, 20, 10),
            TopTimesUs::new(30, 20, 10),
            pointer(1, 2, 0),
            paint_cache(3, 4),
            renderer(5),
            payload(50),
            PerfBaselineSeed::Max,
            PerfBaselineSeed::Max,
            PerfBaselineSeed::Max,
            30,
            20,
            10,
            renderer_seed(PerfBaselineSeed::Max),
            renderer(5),
            payload(50),
            36,
            24,
            12,
            pointer(2, 3, 0),
            2,
            5,
            renderer(7),
            payload(70),
        );

        assert_eq!(rows[0]["measured_max"]["top_total_time_us"], 30);
        assert_eq!(rows[0]["measured_max"]["renderer_prepare_svg_us"], 10);
        assert!(rows[0]["thresholds"]["max_top_total_us"].is_null());
        assert!(rows[0]["thresholds"]["max_pointer_move_dispatch_us"].is_null());
        assert_eq!(rows[0]["thresholds"]["max_renderer_encode_scene_us"], 7);
        assert_eq!(rows[0]["thresholds"]["max_renderer_prepare_svg_us"], 12);
        assert_eq!(rows[0]["thresholds"]["max_renderer_instance_bytes"], 70);
    }
}
