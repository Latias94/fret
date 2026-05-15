use super::*;

#[derive(Clone, Copy)]
pub(super) struct RendererMetricEvidence<'a> {
    pub encode_scene: Option<&'a (u64, PathBuf, u64)>,
    pub upload: Option<&'a (u64, PathBuf, u64)>,
    pub record_passes: Option<&'a (u64, PathBuf, u64)>,
    pub encoder_finish: Option<&'a (u64, PathBuf, u64)>,
    pub prepare_text: Option<&'a (u64, PathBuf, u64)>,
    pub prepare_svg: Option<&'a (u64, PathBuf, u64)>,
    pub instance_bytes: Option<&'a (u64, PathBuf, u64)>,
    pub encode_scene_text_ops: Option<&'a (u64, PathBuf, u64)>,
}

#[derive(Clone, Copy)]
struct MetricEvidence<'a> {
    bundle: &'a Path,
    run_index: u64,
    peak_sort: Option<BundleStatsSort>,
}

impl<'a> RendererMetricEvidence<'a> {
    fn evidence_for_metric(&self, metric: &str) -> Option<MetricEvidence<'a>> {
        let (entry, peak_sort) = match metric {
            "renderer_encode_scene_us" => (
                self.encode_scene,
                Some(BundleStatsSort::RendererEncodeScene),
            ),
            "renderer_upload_us" => (self.upload, Some(BundleStatsSort::RendererUpload)),
            "renderer_record_passes_us" => (
                self.record_passes,
                Some(BundleStatsSort::RendererRecordPasses),
            ),
            "renderer_encoder_finish_us" => (
                self.encoder_finish,
                Some(BundleStatsSort::RendererEncoderFinish),
            ),
            "renderer_prepare_text_us" => (
                self.prepare_text,
                Some(BundleStatsSort::RendererPrepareText),
            ),
            "renderer_prepare_svg_us" => (self.prepare_svg, None),
            "renderer_instance_bytes" => (self.instance_bytes, None),
            "renderer_encode_scene_text_ops" => (self.encode_scene_text_ops, None),
            _ => return None,
        };
        entry.map(|(_us, bundle, run_index)| MetricEvidence {
            bundle: bundle.as_path(),
            run_index: *run_index,
            peak_sort,
        })
    }
}

fn renderer_peak_sort(metric: &str) -> Option<BundleStatsSort> {
    match metric {
        "renderer_encode_scene_us" => Some(BundleStatsSort::RendererEncodeScene),
        "renderer_upload_us" => Some(BundleStatsSort::RendererUpload),
        "renderer_record_passes_us" => Some(BundleStatsSort::RendererRecordPasses),
        "renderer_encoder_finish_us" => Some(BundleStatsSort::RendererEncoderFinish),
        "renderer_prepare_text_us" => Some(BundleStatsSort::RendererPrepareText),
        _ => None,
    }
}

fn attach_failure_run_context(
    failure: &mut serde_json::Value,
    run_row: serde_json::Value,
    evidence_bundle: &Path,
    evidence_run_index: u64,
    warmup_frames: u64,
    peak_sort: Option<BundleStatsSort>,
) {
    if let Some(obj) = failure.as_object_mut() {
        obj.insert(
            "evidence_bundle".to_string(),
            serde_json::Value::String(evidence_bundle.display().to_string()),
        );
        obj.insert(
            "evidence_run_index".to_string(),
            serde_json::Value::Number(serde_json::Number::from(evidence_run_index)),
        );
        obj.insert("evidence_run".to_string(), run_row);

        if let Some(sort) = peak_sort
            && let Ok(report) = bundle_stats_from_path(
                evidence_bundle,
                1,
                sort,
                BundleStatsOptions { warmup_frames },
            )
        {
            let peak = triage_json_from_stats(evidence_bundle, &report, sort, warmup_frames)
                .get("worst")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            obj.insert(
                "evidence_peak".to_string(),
                serde_json::json!({
                    "metric_sort": sort.as_str(),
                    "worst": peak,
                }),
            );
        }
    }
}

fn attach_single_run_failure_context(
    failures: &mut [serde_json::Value],
    run_row: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) {
    let run_row = run_row.clone();
    for failure in failures.iter_mut() {
        let metric = failure
            .get("metric")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        attach_failure_run_context(
            failure,
            run_row.clone(),
            bundle_path,
            0,
            warmup_frames,
            renderer_peak_sort(metric),
        );
    }
}

fn attach_repeat_run_failure_context<'a>(
    failures: &mut [serde_json::Value],
    runs_json: &[serde_json::Value],
    warmup_frames: u64,
    renderer_evidence: &RendererMetricEvidence<'a>,
) {
    for failure in failures.iter_mut() {
        let metric = failure
            .get("metric")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let renderer_metric_evidence = renderer_evidence.evidence_for_metric(metric);
        let run_index = renderer_metric_evidence
            .map(|e| e.run_index)
            .or_else(|| failure.get("evidence_run_index").and_then(|v| v.as_u64()));
        let Some(run_index) = run_index else {
            continue;
        };
        let Some(run_row) = runs_json.get(run_index as usize).cloned() else {
            continue;
        };

        if let Some(metric_evidence) = renderer_metric_evidence {
            attach_failure_run_context(
                failure,
                run_row,
                metric_evidence.bundle,
                metric_evidence.run_index,
                warmup_frames,
                metric_evidence.peak_sort,
            );
        } else if let Some(obj) = failure.as_object_mut() {
            obj.insert("evidence_run".to_string(), run_row);
        }
    }
}

pub(super) struct SingleRunThresholdInputs<'a> {
    pub script_key: &'a str,
    pub sort: BundleStatsSort,
    pub perf_threshold_agg: PerfThresholdAggregate,
    pub cli_thresholds: PerfThresholds,
    pub baseline_thresholds: PerfThresholds,
    pub warmup_frames: u64,

    pub top_total: u64,
    pub top_layout: u64,
    pub top_solve: u64,
    pub top_solves: u64,
    pub top_tick: u64,
    pub top_frame: u64,

    pub frame_p95_total_time_us: u64,
    pub frame_p95_layout_time_us: u64,
    pub frame_p95_layout_engine_solve_time_us: u64,

    pub pointer_move_frames_present: bool,
    pub pointer_move_frames_considered: u64,
    pub pointer_move_max_dispatch_time_us: u64,
    pub pointer_move_max_hit_test_time_us: u64,
    pub pointer_move_snapshots_with_global_changes: u64,

    pub run_paint_cache_hit_test_only_replay_allowed_max: u64,
    pub run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: u64,

    pub max_renderer_encode_scene_us: u64,
    pub max_renderer_upload_us: u64,
    pub max_renderer_record_passes_us: u64,
    pub max_renderer_encoder_finish_us: u64,
    pub max_renderer_prepare_text_us: u64,
    pub max_renderer_prepare_svg_us: u64,
    pub max_renderer_instance_bytes: u64,
    pub max_renderer_encode_scene_text_ops: u64,

    pub bundle_path: &'a Path,

    pub thr_total: Option<u64>,
    pub src_total: Option<&'static str>,
    pub thr_layout: Option<u64>,
    pub src_layout: Option<&'static str>,
    pub thr_solve: Option<u64>,
    pub src_solve: Option<&'static str>,
    pub thr_frame_p95_total: Option<u64>,
    pub src_frame_p95_total: Option<&'static str>,
    pub thr_frame_p95_layout: Option<u64>,
    pub src_frame_p95_layout: Option<&'static str>,
    pub thr_frame_p95_solve: Option<u64>,
    pub src_frame_p95_solve: Option<&'static str>,
    pub thr_pointer_move_dispatch: Option<u64>,
    pub src_pointer_move_dispatch: Option<&'static str>,
    pub thr_pointer_move_hit_test: Option<u64>,
    pub src_pointer_move_hit_test: Option<&'static str>,
    pub thr_pointer_move_global_changes: Option<u64>,
    pub src_pointer_move_global_changes: Option<&'static str>,
    pub thr_paint_cache_hit_test_only_replay_allowed_max: Option<u64>,
    pub src_paint_cache_hit_test_only_replay_allowed_max: Option<&'static str>,
    pub thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Option<u64>,
    pub src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Option<&'static str>,
}

pub(super) fn push_single_run_threshold_row_and_failures(
    perf_threshold_rows: &mut Vec<serde_json::Value>,
    perf_threshold_failures: &mut Vec<serde_json::Value>,
    input: SingleRunThresholdInputs<'_>,
) {
    let SingleRunThresholdInputs {
        script_key,
        sort,
        perf_threshold_agg,
        cli_thresholds,
        baseline_thresholds,
        warmup_frames,
        top_total,
        top_layout,
        top_solve,
        top_solves,
        top_tick,
        top_frame,
        frame_p95_total_time_us,
        frame_p95_layout_time_us,
        frame_p95_layout_engine_solve_time_us,
        pointer_move_frames_present,
        pointer_move_frames_considered,
        pointer_move_max_dispatch_time_us,
        pointer_move_max_hit_test_time_us,
        pointer_move_snapshots_with_global_changes,
        run_paint_cache_hit_test_only_replay_allowed_max,
        run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        max_renderer_encode_scene_us,
        max_renderer_upload_us,
        max_renderer_record_passes_us,
        max_renderer_encoder_finish_us,
        max_renderer_prepare_text_us,
        max_renderer_prepare_svg_us,
        max_renderer_instance_bytes,
        max_renderer_encode_scene_text_ops,
        bundle_path,
        thr_total,
        src_total,
        thr_layout,
        src_layout,
        thr_solve,
        src_solve,
        thr_frame_p95_total,
        src_frame_p95_total,
        thr_frame_p95_layout,
        src_frame_p95_layout,
        thr_frame_p95_solve,
        src_frame_p95_solve,
        thr_pointer_move_dispatch,
        src_pointer_move_dispatch,
        thr_pointer_move_hit_test,
        src_pointer_move_hit_test,
        thr_pointer_move_global_changes,
        src_pointer_move_global_changes,
        thr_paint_cache_hit_test_only_replay_allowed_max,
        src_paint_cache_hit_test_only_replay_allowed_max,
        thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
    } = input;

    let (thr_pointer_move_dispatch, src_pointer_move_dispatch) = if pointer_move_frames_present {
        (thr_pointer_move_dispatch, src_pointer_move_dispatch)
    } else {
        (None, None)
    };
    let (thr_pointer_move_hit_test, src_pointer_move_hit_test) = if pointer_move_frames_present {
        (thr_pointer_move_hit_test, src_pointer_move_hit_test)
    } else {
        (None, None)
    };
    let (thr_pointer_move_global_changes, src_pointer_move_global_changes) =
        if pointer_move_frames_present {
            (
                thr_pointer_move_global_changes,
                src_pointer_move_global_changes,
            )
        } else {
            (None, None)
        };

    let (thr_renderer_encode_scene, src_renderer_encode_scene) = resolve_threshold(
        cli_thresholds.max_renderer_encode_scene_us,
        baseline_thresholds.max_renderer_encode_scene_us,
    );
    let (thr_renderer_upload, src_renderer_upload) = resolve_threshold(
        cli_thresholds.max_renderer_upload_us,
        baseline_thresholds.max_renderer_upload_us,
    );
    let (thr_renderer_record_passes, src_renderer_record_passes) = resolve_threshold(
        cli_thresholds.max_renderer_record_passes_us,
        baseline_thresholds.max_renderer_record_passes_us,
    );
    let (thr_renderer_encoder_finish, src_renderer_encoder_finish) = resolve_threshold(
        cli_thresholds.max_renderer_encoder_finish_us,
        baseline_thresholds.max_renderer_encoder_finish_us,
    );
    let (thr_renderer_prepare_text, src_renderer_prepare_text) = resolve_threshold(
        cli_thresholds.max_renderer_prepare_text_us,
        baseline_thresholds.max_renderer_prepare_text_us,
    );
    let (thr_renderer_prepare_svg, src_renderer_prepare_svg) = resolve_threshold(
        cli_thresholds.max_renderer_prepare_svg_us,
        baseline_thresholds.max_renderer_prepare_svg_us,
    );
    let (thr_renderer_instance_bytes, src_renderer_instance_bytes) = resolve_threshold(
        cli_thresholds.max_renderer_instance_bytes,
        baseline_thresholds.max_renderer_instance_bytes,
    );
    let (thr_renderer_encode_scene_text_ops, src_renderer_encode_scene_text_ops) =
        resolve_threshold(
            cli_thresholds.max_renderer_encode_scene_text_ops,
            baseline_thresholds.max_renderer_encode_scene_text_ops,
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
        "top_renderer_instance_bytes": max_renderer_instance_bytes,
        "top_renderer_encode_scene_text_ops": max_renderer_encode_scene_text_ops,
        "top_tick_id": top_tick,
        "top_frame_id": top_frame,
        "bundle": bundle_path.display().to_string(),
    });
    let row = serde_json::json!({
        "script": script_key,
        "sort": sort.as_str(),
        "repeat": 1,
        "runs": [run],
        "observed_aggregate": perf_threshold_agg.as_str(),
        "observed": {
            "top_total_time_us": top_total,
            "top_layout_time_us": top_layout,
            "top_layout_engine_solve_time_us": top_solve,
            "renderer_encode_scene_us": max_renderer_encode_scene_us,
            "renderer_upload_us": max_renderer_upload_us,
            "renderer_record_passes_us": max_renderer_record_passes_us,
            "renderer_encoder_finish_us": max_renderer_encoder_finish_us,
            "renderer_prepare_text_us": max_renderer_prepare_text_us,
            "renderer_prepare_svg_us": max_renderer_prepare_svg_us,
            "renderer_instance_bytes": max_renderer_instance_bytes,
            "renderer_encode_scene_text_ops": max_renderer_encode_scene_text_ops,
        },
        "worst_run": {
            "top_total_time_us": top_total,
            "bundle": bundle_path.display().to_string(),
            "trace_chrome": bundle_path
                .parent()
                .map(|dir| dir.join("trace.chrome.json"))
                .filter(|p| p.is_file())
                .map(|p| p.display().to_string()),
        },
        "max": {
            "top_total_time_us": top_total,
            "top_layout_time_us": top_layout,
            "top_layout_engine_solve_time_us": top_solve,
            "frame_p95_total_time_us": frame_p95_total_time_us,
            "frame_p95_layout_time_us": frame_p95_layout_time_us,
            "frame_p95_layout_engine_solve_time_us": frame_p95_layout_engine_solve_time_us,
            "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
            "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
            "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
            "run_paint_cache_hit_test_only_replay_allowed_max": run_paint_cache_hit_test_only_replay_allowed_max,
            "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            "renderer_encode_scene_us": max_renderer_encode_scene_us,
            "renderer_upload_us": max_renderer_upload_us,
            "renderer_record_passes_us": max_renderer_record_passes_us,
            "renderer_encoder_finish_us": max_renderer_encoder_finish_us,
            "renderer_prepare_text_us": max_renderer_prepare_text_us,
            "renderer_prepare_svg_us": max_renderer_prepare_svg_us,
            "renderer_instance_bytes": max_renderer_instance_bytes,
            "renderer_encode_scene_text_ops": max_renderer_encode_scene_text_ops,
        },
        "p50": {
            "top_total_time_us": top_total,
            "top_layout_time_us": top_layout,
            "top_layout_engine_solve_time_us": top_solve,
            "frame_p95_total_time_us": frame_p95_total_time_us,
            "frame_p95_layout_time_us": frame_p95_layout_time_us,
            "frame_p95_layout_engine_solve_time_us": frame_p95_layout_engine_solve_time_us,
            "renderer_encode_scene_us": max_renderer_encode_scene_us,
            "renderer_upload_us": max_renderer_upload_us,
            "renderer_record_passes_us": max_renderer_record_passes_us,
            "renderer_encoder_finish_us": max_renderer_encoder_finish_us,
            "renderer_prepare_text_us": max_renderer_prepare_text_us,
            "renderer_prepare_svg_us": max_renderer_prepare_svg_us,
            "renderer_instance_bytes": max_renderer_instance_bytes,
            "renderer_encode_scene_text_ops": max_renderer_encode_scene_text_ops,
        },
        "p95": {
            "top_total_time_us": top_total,
            "top_layout_time_us": top_layout,
            "top_layout_engine_solve_time_us": top_solve,
            "frame_p95_total_time_us": frame_p95_total_time_us,
            "frame_p95_layout_time_us": frame_p95_layout_time_us,
            "frame_p95_layout_engine_solve_time_us": frame_p95_layout_engine_solve_time_us,
            "renderer_encode_scene_us": max_renderer_encode_scene_us,
            "renderer_upload_us": max_renderer_upload_us,
            "renderer_record_passes_us": max_renderer_record_passes_us,
            "renderer_encoder_finish_us": max_renderer_encoder_finish_us,
            "renderer_prepare_text_us": max_renderer_prepare_text_us,
            "renderer_prepare_svg_us": max_renderer_prepare_svg_us,
            "renderer_instance_bytes": max_renderer_instance_bytes,
            "renderer_encode_scene_text_ops": max_renderer_encode_scene_text_ops,
        },
        "thresholds": {
            "max_top_total_us": thr_total,
            "max_top_layout_us": thr_layout,
            "max_top_solve_us": thr_solve,
            "max_frame_p95_total_us": thr_frame_p95_total,
            "max_frame_p95_layout_us": thr_frame_p95_layout,
            "max_frame_p95_solve_us": thr_frame_p95_solve,
            "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
            "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
            "max_pointer_move_global_changes": thr_pointer_move_global_changes,
            "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_paint_cache_hit_test_only_replay_allowed_max,
            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            "max_renderer_encode_scene_us": thr_renderer_encode_scene,
            "max_renderer_upload_us": thr_renderer_upload,
            "max_renderer_record_passes_us": thr_renderer_record_passes,
            "max_renderer_encoder_finish_us": thr_renderer_encoder_finish,
            "max_renderer_prepare_text_us": thr_renderer_prepare_text,
            "max_renderer_prepare_svg_us": thr_renderer_prepare_svg,
            "max_renderer_instance_bytes": thr_renderer_instance_bytes,
            "max_renderer_encode_scene_text_ops": thr_renderer_encode_scene_text_ops,
        },
        "threshold_sources": {
            "max_top_total_us": src_total,
            "max_top_layout_us": src_layout,
            "max_top_solve_us": src_solve,
            "max_frame_p95_total_us": src_frame_p95_total,
            "max_frame_p95_layout_us": src_frame_p95_layout,
            "max_frame_p95_solve_us": src_frame_p95_solve,
            "max_pointer_move_dispatch_us": src_pointer_move_dispatch,
            "max_pointer_move_hit_test_us": src_pointer_move_hit_test,
            "max_pointer_move_global_changes": src_pointer_move_global_changes,
            "min_run_paint_cache_hit_test_only_replay_allowed_max": src_paint_cache_hit_test_only_replay_allowed_max,
            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            "max_renderer_encode_scene_us": src_renderer_encode_scene,
            "max_renderer_upload_us": src_renderer_upload,
            "max_renderer_record_passes_us": src_renderer_record_passes,
            "max_renderer_encoder_finish_us": src_renderer_encoder_finish,
            "max_renderer_prepare_text_us": src_renderer_prepare_text,
            "max_renderer_prepare_svg_us": src_renderer_prepare_svg,
            "max_renderer_instance_bytes": src_renderer_instance_bytes,
            "max_renderer_encode_scene_text_ops": src_renderer_encode_scene_text_ops,
        },
    });

    perf_threshold_rows.push(row);
    let mut failures = scan_perf_threshold_failures(
        script_key,
        sort,
        perf_threshold_agg,
        cli_thresholds,
        baseline_thresholds,
        top_total,
        top_total,
        top_total,
        top_layout,
        top_layout,
        top_layout,
        top_solve,
        top_solve,
        top_solve,
        frame_p95_total_time_us,
        frame_p95_total_time_us,
        frame_p95_total_time_us,
        frame_p95_layout_time_us,
        frame_p95_layout_time_us,
        frame_p95_layout_time_us,
        frame_p95_layout_engine_solve_time_us,
        frame_p95_layout_engine_solve_time_us,
        frame_p95_layout_engine_solve_time_us,
        pointer_move_frames_present,
        pointer_move_max_dispatch_time_us,
        pointer_move_max_hit_test_time_us,
        pointer_move_snapshots_with_global_changes,
        run_paint_cache_hit_test_only_replay_allowed_max,
        run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        max_renderer_encode_scene_us,
        max_renderer_encode_scene_us,
        max_renderer_encode_scene_us,
        max_renderer_upload_us,
        max_renderer_upload_us,
        max_renderer_upload_us,
        max_renderer_record_passes_us,
        max_renderer_record_passes_us,
        max_renderer_record_passes_us,
        max_renderer_encoder_finish_us,
        max_renderer_encoder_finish_us,
        max_renderer_encoder_finish_us,
        max_renderer_prepare_text_us,
        max_renderer_prepare_text_us,
        max_renderer_prepare_text_us,
        max_renderer_prepare_svg_us,
        max_renderer_prepare_svg_us,
        max_renderer_prepare_svg_us,
        max_renderer_instance_bytes,
        max_renderer_instance_bytes,
        max_renderer_instance_bytes,
        max_renderer_encode_scene_text_ops,
        max_renderer_encode_scene_text_ops,
        max_renderer_encode_scene_text_ops,
        Some(bundle_path),
        Some(0),
        None,
        None,
        None,
        None,
    );
    attach_single_run_failure_context(&mut failures, &run, bundle_path, warmup_frames);
    perf_threshold_failures.extend(failures);
}

pub(super) struct RepeatThresholdInputs<'a> {
    pub script_key: &'a str,
    pub sort: BundleStatsSort,
    pub repeat: usize,
    pub runs_json: &'a Vec<serde_json::Value>,
    pub perf_threshold_agg: PerfThresholdAggregate,
    pub cli_thresholds: PerfThresholds,
    pub baseline_thresholds: PerfThresholds,
    pub warmup_frames: u64,
    pub renderer_evidence: RendererMetricEvidence<'a>,

    pub observed_total: u64,
    pub max_total: u64,
    pub p95_total: u64,
    pub sorted_total: &'a Vec<u64>,
    pub p90_total: u64,

    pub observed_layout: u64,
    pub max_layout: u64,
    pub p95_layout: u64,
    pub sorted_layout: &'a Vec<u64>,
    pub p90_layout: u64,

    pub observed_solve: u64,
    pub max_solve: u64,
    pub p95_solve: u64,
    pub sorted_solve: &'a Vec<u64>,
    pub p90_solve: u64,

    pub observed_frame_p95_total: u64,
    pub max_frame_p95_total: u64,
    pub p95_frame_p95_total: u64,
    pub sorted_frame_p95_total: &'a Vec<u64>,
    pub p90_frame_p95_total: u64,

    pub observed_frame_p95_layout: u64,
    pub max_frame_p95_layout: u64,
    pub p95_frame_p95_layout: u64,
    pub sorted_frame_p95_layout: &'a Vec<u64>,
    pub p90_frame_p95_layout: u64,

    pub observed_frame_p95_solve: u64,
    pub max_frame_p95_solve: u64,
    pub p95_frame_p95_solve: u64,
    pub sorted_frame_p95_solve: &'a Vec<u64>,
    pub p90_frame_p95_solve: u64,

    pub pointer_move_frames_present: bool,
    pub max_pointer_move_dispatch: u64,
    pub max_pointer_move_hit_test: u64,
    pub max_pointer_move_global_changes: u64,

    pub max_run_paint_cache_hit_test_only_replay_allowed_max: u64,
    pub max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: u64,

    pub observed_renderer_encode_scene_us: u64,
    pub max_renderer_encode_scene_us: u64,
    pub p95_renderer_encode_scene_us: u64,

    pub observed_renderer_upload_us: u64,
    pub max_renderer_upload_us: u64,
    pub p95_renderer_upload_us: u64,

    pub observed_renderer_record_passes_us: u64,
    pub max_renderer_record_passes_us: u64,
    pub p95_renderer_record_passes_us: u64,

    pub observed_renderer_encoder_finish_us: u64,
    pub max_renderer_encoder_finish_us: u64,
    pub p95_renderer_encoder_finish_us: u64,

    pub observed_renderer_prepare_text_us: u64,
    pub max_renderer_prepare_text_us: u64,
    pub p95_renderer_prepare_text_us: u64,

    pub observed_renderer_prepare_svg_us: u64,
    pub max_renderer_prepare_svg_us: u64,
    pub p95_renderer_prepare_svg_us: u64,
    pub observed_renderer_instance_bytes: u64,
    pub max_renderer_instance_bytes: u64,
    pub p95_renderer_instance_bytes: u64,
    pub observed_renderer_encode_scene_text_ops: u64,
    pub max_renderer_encode_scene_text_ops: u64,
    pub p95_renderer_encode_scene_text_ops: u64,

    pub script_worst: &'a Option<(u64, PathBuf, u64)>,
    pub script_worst_layout: &'a Option<(u64, PathBuf, u64)>,
    pub script_worst_solve: &'a Option<(u64, PathBuf, u64)>,

    pub thr_total: Option<u64>,
    pub src_total: Option<&'static str>,
    pub thr_layout: Option<u64>,
    pub src_layout: Option<&'static str>,
    pub thr_solve: Option<u64>,
    pub src_solve: Option<&'static str>,
    pub thr_frame_p95_total: Option<u64>,
    pub src_frame_p95_total: Option<&'static str>,
    pub thr_frame_p95_layout: Option<u64>,
    pub src_frame_p95_layout: Option<&'static str>,
    pub thr_frame_p95_solve: Option<u64>,
    pub src_frame_p95_solve: Option<&'static str>,
    pub thr_pointer_move_dispatch: Option<u64>,
    pub src_pointer_move_dispatch: Option<&'static str>,
    pub thr_pointer_move_hit_test: Option<u64>,
    pub src_pointer_move_hit_test: Option<&'static str>,
    pub thr_pointer_move_global_changes: Option<u64>,
    pub src_pointer_move_global_changes: Option<&'static str>,
    pub thr_paint_cache_hit_test_only_replay_allowed_max: Option<u64>,
    pub src_paint_cache_hit_test_only_replay_allowed_max: Option<&'static str>,
    pub thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Option<u64>,
    pub src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Option<&'static str>,
}

pub(super) fn push_repeat_threshold_row_and_failures(
    perf_threshold_rows: &mut Vec<serde_json::Value>,
    perf_threshold_failures: &mut Vec<serde_json::Value>,
    input: RepeatThresholdInputs<'_>,
) {
    let RepeatThresholdInputs {
        script_key,
        sort,
        repeat,
        runs_json,
        perf_threshold_agg,
        cli_thresholds,
        baseline_thresholds,
        warmup_frames,
        renderer_evidence,
        observed_total,
        max_total,
        p95_total,
        sorted_total,
        p90_total,
        observed_layout,
        max_layout,
        p95_layout,
        sorted_layout,
        p90_layout,
        observed_solve,
        max_solve,
        p95_solve,
        sorted_solve,
        p90_solve,
        observed_frame_p95_total,
        max_frame_p95_total,
        p95_frame_p95_total,
        sorted_frame_p95_total,
        p90_frame_p95_total,
        observed_frame_p95_layout,
        max_frame_p95_layout,
        p95_frame_p95_layout,
        sorted_frame_p95_layout,
        p90_frame_p95_layout,
        observed_frame_p95_solve,
        max_frame_p95_solve,
        p95_frame_p95_solve,
        sorted_frame_p95_solve,
        p90_frame_p95_solve,
        pointer_move_frames_present,
        max_pointer_move_dispatch,
        max_pointer_move_hit_test,
        max_pointer_move_global_changes,
        max_run_paint_cache_hit_test_only_replay_allowed_max,
        max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        observed_renderer_encode_scene_us,
        max_renderer_encode_scene_us,
        p95_renderer_encode_scene_us,
        observed_renderer_upload_us,
        max_renderer_upload_us,
        p95_renderer_upload_us,
        observed_renderer_record_passes_us,
        max_renderer_record_passes_us,
        p95_renderer_record_passes_us,
        observed_renderer_encoder_finish_us,
        max_renderer_encoder_finish_us,
        p95_renderer_encoder_finish_us,
        observed_renderer_prepare_text_us,
        max_renderer_prepare_text_us,
        p95_renderer_prepare_text_us,
        observed_renderer_prepare_svg_us,
        max_renderer_prepare_svg_us,
        p95_renderer_prepare_svg_us,
        observed_renderer_instance_bytes,
        max_renderer_instance_bytes,
        p95_renderer_instance_bytes,
        observed_renderer_encode_scene_text_ops,
        max_renderer_encode_scene_text_ops,
        p95_renderer_encode_scene_text_ops,
        script_worst,
        script_worst_layout,
        script_worst_solve,
        thr_total,
        src_total,
        thr_layout,
        src_layout,
        thr_solve,
        src_solve,
        thr_frame_p95_total,
        src_frame_p95_total,
        thr_frame_p95_layout,
        src_frame_p95_layout,
        thr_frame_p95_solve,
        src_frame_p95_solve,
        thr_pointer_move_dispatch,
        src_pointer_move_dispatch,
        thr_pointer_move_hit_test,
        src_pointer_move_hit_test,
        thr_pointer_move_global_changes,
        src_pointer_move_global_changes,
        thr_paint_cache_hit_test_only_replay_allowed_max,
        src_paint_cache_hit_test_only_replay_allowed_max,
        thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
    } = input;

    let (thr_pointer_move_dispatch, src_pointer_move_dispatch) = if pointer_move_frames_present {
        (thr_pointer_move_dispatch, src_pointer_move_dispatch)
    } else {
        (None, None)
    };
    let (thr_pointer_move_hit_test, src_pointer_move_hit_test) = if pointer_move_frames_present {
        (thr_pointer_move_hit_test, src_pointer_move_hit_test)
    } else {
        (None, None)
    };
    let (thr_pointer_move_global_changes, src_pointer_move_global_changes) =
        if pointer_move_frames_present {
            (
                thr_pointer_move_global_changes,
                src_pointer_move_global_changes,
            )
        } else {
            (None, None)
        };

    let (thr_renderer_encode_scene, src_renderer_encode_scene) = resolve_threshold(
        cli_thresholds.max_renderer_encode_scene_us,
        baseline_thresholds.max_renderer_encode_scene_us,
    );
    let (thr_renderer_upload, src_renderer_upload) = resolve_threshold(
        cli_thresholds.max_renderer_upload_us,
        baseline_thresholds.max_renderer_upload_us,
    );
    let (thr_renderer_record_passes, src_renderer_record_passes) = resolve_threshold(
        cli_thresholds.max_renderer_record_passes_us,
        baseline_thresholds.max_renderer_record_passes_us,
    );
    let (thr_renderer_encoder_finish, src_renderer_encoder_finish) = resolve_threshold(
        cli_thresholds.max_renderer_encoder_finish_us,
        baseline_thresholds.max_renderer_encoder_finish_us,
    );
    let (thr_renderer_prepare_text, src_renderer_prepare_text) = resolve_threshold(
        cli_thresholds.max_renderer_prepare_text_us,
        baseline_thresholds.max_renderer_prepare_text_us,
    );
    let (thr_renderer_prepare_svg, src_renderer_prepare_svg) = resolve_threshold(
        cli_thresholds.max_renderer_prepare_svg_us,
        baseline_thresholds.max_renderer_prepare_svg_us,
    );
    let (thr_renderer_instance_bytes, src_renderer_instance_bytes) = resolve_threshold(
        cli_thresholds.max_renderer_instance_bytes,
        baseline_thresholds.max_renderer_instance_bytes,
    );
    let (thr_renderer_encode_scene_text_ops, src_renderer_encode_scene_text_ops) =
        resolve_threshold(
            cli_thresholds.max_renderer_encode_scene_text_ops,
            baseline_thresholds.max_renderer_encode_scene_text_ops,
        );

    let row = serde_json::json!({
        "script": script_key,
        "sort": sort.as_str(),
        "repeat": repeat,
        "runs": runs_json,
        "observed_aggregate": perf_threshold_agg.as_str(),
        "observed": {
            "top_total_time_us": observed_total,
            "top_layout_time_us": observed_layout,
            "top_layout_engine_solve_time_us": observed_solve,
            "frame_p95_total_time_us": observed_frame_p95_total,
            "frame_p95_layout_time_us": observed_frame_p95_layout,
            "frame_p95_layout_engine_solve_time_us": observed_frame_p95_solve,
            "renderer_encode_scene_us": observed_renderer_encode_scene_us,
            "renderer_upload_us": observed_renderer_upload_us,
            "renderer_record_passes_us": observed_renderer_record_passes_us,
            "renderer_encoder_finish_us": observed_renderer_encoder_finish_us,
            "renderer_prepare_text_us": observed_renderer_prepare_text_us,
            "renderer_prepare_svg_us": observed_renderer_prepare_svg_us,
            "renderer_instance_bytes": observed_renderer_instance_bytes,
            "renderer_encode_scene_text_ops": observed_renderer_encode_scene_text_ops,
        },
        "worst_run": script_worst.as_ref().map(|(us, bundle, run_index)| serde_json::json!({
            "top_total_time_us": us,
            "bundle": bundle.display().to_string(),
            "run_index": run_index,
            "trace_chrome": bundle
                .parent()
                .map(|dir| dir.join("trace.chrome.json"))
                .filter(|p| p.is_file())
                .map(|p| p.display().to_string()),
        })),
        "max": {
            "top_total_time_us": max_total,
            "top_layout_time_us": max_layout,
            "top_layout_engine_solve_time_us": max_solve,
            "frame_p95_total_time_us": max_frame_p95_total,
            "frame_p95_layout_time_us": max_frame_p95_layout,
            "frame_p95_layout_engine_solve_time_us": max_frame_p95_solve,
            "pointer_move_max_dispatch_time_us": max_pointer_move_dispatch,
            "pointer_move_max_hit_test_time_us": max_pointer_move_hit_test,
            "pointer_move_snapshots_with_global_changes": max_pointer_move_global_changes,
            "run_paint_cache_hit_test_only_replay_allowed_max": max_run_paint_cache_hit_test_only_replay_allowed_max,
            "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            "renderer_encode_scene_us": max_renderer_encode_scene_us,
            "renderer_upload_us": max_renderer_upload_us,
            "renderer_record_passes_us": max_renderer_record_passes_us,
            "renderer_encoder_finish_us": max_renderer_encoder_finish_us,
            "renderer_prepare_text_us": max_renderer_prepare_text_us,
            "renderer_prepare_svg_us": max_renderer_prepare_svg_us,
            "renderer_instance_bytes": max_renderer_instance_bytes,
            "renderer_encode_scene_text_ops": max_renderer_encode_scene_text_ops,
        },
        "p50": {
            "top_total_time_us": percentile_nearest_rank_sorted(sorted_total, 0.50),
            "top_layout_time_us": percentile_nearest_rank_sorted(sorted_layout, 0.50),
            "top_layout_engine_solve_time_us": percentile_nearest_rank_sorted(sorted_solve, 0.50),
            "frame_p95_total_time_us": percentile_nearest_rank_sorted(sorted_frame_p95_total, 0.50),
            "frame_p95_layout_time_us": percentile_nearest_rank_sorted(sorted_frame_p95_layout, 0.50),
            "frame_p95_layout_engine_solve_time_us": percentile_nearest_rank_sorted(sorted_frame_p95_solve, 0.50),
        },
        "p90": {
            "top_total_time_us": p90_total,
            "top_layout_time_us": p90_layout,
            "top_layout_engine_solve_time_us": p90_solve,
            "frame_p95_total_time_us": p90_frame_p95_total,
            "frame_p95_layout_time_us": p90_frame_p95_layout,
            "frame_p95_layout_engine_solve_time_us": p90_frame_p95_solve,
        },
        "p95": {
            "top_total_time_us": p95_total,
            "top_layout_time_us": p95_layout,
            "top_layout_engine_solve_time_us": p95_solve,
            "frame_p95_total_time_us": p95_frame_p95_total,
            "frame_p95_layout_time_us": p95_frame_p95_layout,
            "frame_p95_layout_engine_solve_time_us": p95_frame_p95_solve,
            "renderer_encode_scene_us": p95_renderer_encode_scene_us,
            "renderer_upload_us": p95_renderer_upload_us,
            "renderer_record_passes_us": p95_renderer_record_passes_us,
            "renderer_encoder_finish_us": p95_renderer_encoder_finish_us,
            "renderer_prepare_text_us": p95_renderer_prepare_text_us,
            "renderer_prepare_svg_us": p95_renderer_prepare_svg_us,
            "renderer_instance_bytes": p95_renderer_instance_bytes,
            "renderer_encode_scene_text_ops": p95_renderer_encode_scene_text_ops,
        },
        "thresholds": {
            "max_top_total_us": thr_total,
            "max_top_layout_us": thr_layout,
            "max_top_solve_us": thr_solve,
            "max_frame_p95_total_us": thr_frame_p95_total,
            "max_frame_p95_layout_us": thr_frame_p95_layout,
            "max_frame_p95_solve_us": thr_frame_p95_solve,
            "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
            "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
            "max_pointer_move_global_changes": thr_pointer_move_global_changes,
            "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_paint_cache_hit_test_only_replay_allowed_max,
            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            "max_renderer_encode_scene_us": thr_renderer_encode_scene,
            "max_renderer_upload_us": thr_renderer_upload,
            "max_renderer_record_passes_us": thr_renderer_record_passes,
            "max_renderer_encoder_finish_us": thr_renderer_encoder_finish,
            "max_renderer_prepare_text_us": thr_renderer_prepare_text,
            "max_renderer_prepare_svg_us": thr_renderer_prepare_svg,
            "max_renderer_instance_bytes": thr_renderer_instance_bytes,
            "max_renderer_encode_scene_text_ops": thr_renderer_encode_scene_text_ops,
        },
        "threshold_sources": {
            "max_top_total_us": src_total,
            "max_top_layout_us": src_layout,
            "max_top_solve_us": src_solve,
            "max_frame_p95_total_us": src_frame_p95_total,
            "max_frame_p95_layout_us": src_frame_p95_layout,
            "max_frame_p95_solve_us": src_frame_p95_solve,
            "max_pointer_move_dispatch_us": src_pointer_move_dispatch,
            "max_pointer_move_hit_test_us": src_pointer_move_hit_test,
            "max_pointer_move_global_changes": src_pointer_move_global_changes,
            "min_run_paint_cache_hit_test_only_replay_allowed_max": src_paint_cache_hit_test_only_replay_allowed_max,
            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            "max_renderer_encode_scene_us": src_renderer_encode_scene,
            "max_renderer_upload_us": src_renderer_upload,
            "max_renderer_record_passes_us": src_renderer_record_passes,
            "max_renderer_encoder_finish_us": src_renderer_encoder_finish,
            "max_renderer_prepare_text_us": src_renderer_prepare_text,
            "max_renderer_prepare_svg_us": src_renderer_prepare_svg,
            "max_renderer_instance_bytes": src_renderer_instance_bytes,
            "max_renderer_encode_scene_text_ops": src_renderer_encode_scene_text_ops,
        },
    });

    perf_threshold_rows.push(row);
    let mut failures = scan_perf_threshold_failures(
        script_key,
        sort,
        perf_threshold_agg,
        cli_thresholds,
        baseline_thresholds,
        observed_total,
        max_total,
        p95_total,
        observed_layout,
        max_layout,
        p95_layout,
        observed_solve,
        max_solve,
        p95_solve,
        observed_frame_p95_total,
        max_frame_p95_total,
        p95_frame_p95_total,
        observed_frame_p95_layout,
        max_frame_p95_layout,
        p95_frame_p95_layout,
        observed_frame_p95_solve,
        max_frame_p95_solve,
        p95_frame_p95_solve,
        pointer_move_frames_present,
        max_pointer_move_dispatch,
        max_pointer_move_hit_test,
        max_pointer_move_global_changes,
        max_run_paint_cache_hit_test_only_replay_allowed_max,
        max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        observed_renderer_encode_scene_us,
        max_renderer_encode_scene_us,
        p95_renderer_encode_scene_us,
        observed_renderer_upload_us,
        max_renderer_upload_us,
        p95_renderer_upload_us,
        observed_renderer_record_passes_us,
        max_renderer_record_passes_us,
        p95_renderer_record_passes_us,
        observed_renderer_encoder_finish_us,
        max_renderer_encoder_finish_us,
        p95_renderer_encoder_finish_us,
        observed_renderer_prepare_text_us,
        max_renderer_prepare_text_us,
        p95_renderer_prepare_text_us,
        observed_renderer_prepare_svg_us,
        max_renderer_prepare_svg_us,
        p95_renderer_prepare_svg_us,
        observed_renderer_instance_bytes,
        max_renderer_instance_bytes,
        p95_renderer_instance_bytes,
        observed_renderer_encode_scene_text_ops,
        max_renderer_encode_scene_text_ops,
        p95_renderer_encode_scene_text_ops,
        script_worst
            .as_ref()
            .map(|(_us, bundle, _run)| bundle.as_path()),
        script_worst.as_ref().map(|(_us, _bundle, run)| *run),
        script_worst_layout
            .as_ref()
            .map(|(_us, bundle, _run)| bundle.as_path()),
        script_worst_layout.as_ref().map(|(_us, _bundle, run)| *run),
        script_worst_solve
            .as_ref()
            .map(|(_us, bundle, _run)| bundle.as_path()),
        script_worst_solve.as_ref().map(|(_us, _bundle, run)| *run),
    );
    attach_repeat_run_failure_context(&mut failures, runs_json, warmup_frames, &renderer_evidence);
    perf_threshold_failures.extend(failures);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perf_thresholds_with_pointer_move_limits() -> PerfThresholds {
        PerfThresholds {
            max_top_total_us: Some(100),
            max_top_layout_us: None,
            max_top_solve_us: None,
            max_frame_p95_total_us: None,
            max_frame_p95_layout_us: None,
            max_frame_p95_solve_us: None,
            max_pointer_move_dispatch_us: Some(10),
            max_pointer_move_hit_test_us: Some(20),
            max_pointer_move_global_changes: Some(0),
            min_run_paint_cache_hit_test_only_replay_allowed_max: None,
            max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: None,
            max_renderer_encode_scene_us: None,
            max_renderer_upload_us: None,
            max_renderer_record_passes_us: None,
            max_renderer_encoder_finish_us: None,
            max_renderer_prepare_text_us: None,
            max_renderer_prepare_svg_us: None,
            max_renderer_instance_bytes: None,
            max_renderer_encode_scene_text_ops: None,
        }
    }

    #[test]
    fn repeat_failure_context_uses_metric_specific_renderer_run() {
        let upload_worst = (1032, PathBuf::from("bundle-upload.json"), 2);
        let renderer_evidence = RendererMetricEvidence {
            encode_scene: None,
            upload: Some(&upload_worst),
            record_passes: None,
            encoder_finish: None,
            prepare_text: None,
            prepare_svg: None,
            instance_bytes: None,
            encode_scene_text_ops: None,
        };
        let runs_json = vec![
            serde_json::json!({
                "run_index": 0,
                "bundle": "bundle-total.json",
                "top_frame_id": 10
            }),
            serde_json::json!({
                "run_index": 1,
                "bundle": "bundle-other.json",
                "top_frame_id": 20
            }),
            serde_json::json!({
                "run_index": 2,
                "bundle": "bundle-upload.json",
                "top_frame_id": 30
            }),
        ];
        let mut failures = vec![serde_json::json!({
            "metric": "renderer_upload_us",
            "evidence_bundle": "bundle-total.json",
            "evidence_run_index": 0,
        })];

        attach_repeat_run_failure_context(&mut failures, &runs_json, 0, &renderer_evidence);

        assert_eq!(
            failures[0].get("evidence_bundle").and_then(|v| v.as_str()),
            Some("bundle-upload.json")
        );
        assert_eq!(
            failures[0]
                .get("evidence_run_index")
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            failures[0]
                .pointer("/evidence_run/top_frame_id")
                .and_then(|v| v.as_u64()),
            Some(30)
        );
    }

    #[test]
    fn repeat_failure_context_keeps_existing_non_renderer_evidence() {
        let upload_worst = (1032, PathBuf::from("bundle-upload.json"), 2);
        let renderer_evidence = RendererMetricEvidence {
            encode_scene: None,
            upload: Some(&upload_worst),
            record_passes: None,
            encoder_finish: None,
            prepare_text: None,
            prepare_svg: None,
            instance_bytes: None,
            encode_scene_text_ops: None,
        };
        let runs_json = vec![
            serde_json::json!({
                "run_index": 0,
                "bundle": "bundle-total.json",
                "top_frame_id": 10
            }),
            serde_json::json!({
                "run_index": 1,
                "bundle": "bundle-layout.json",
                "top_frame_id": 20
            }),
        ];
        let mut failures = vec![serde_json::json!({
            "metric": "top_layout_time_us",
            "evidence_bundle": "bundle-layout.json",
            "evidence_run_index": 1,
        })];

        attach_repeat_run_failure_context(&mut failures, &runs_json, 0, &renderer_evidence);

        assert_eq!(
            failures[0].get("evidence_bundle").and_then(|v| v.as_str()),
            Some("bundle-layout.json")
        );
        assert_eq!(
            failures[0]
                .get("evidence_run_index")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            failures[0]
                .pointer("/evidence_run/top_frame_id")
                .and_then(|v| v.as_u64()),
            Some(20)
        );
    }

    #[test]
    fn single_threshold_row_omits_pointer_move_thresholds_when_frames_are_absent() {
        let mut rows = Vec::new();
        let mut failures = Vec::new();
        let bundle = Path::new("target/no-pointer-move/bundle.schema2.json");

        push_single_run_threshold_row_and_failures(
            &mut rows,
            &mut failures,
            SingleRunThresholdInputs {
                script_key: "tools/diag-scripts/no-pointer-move.json",
                sort: BundleStatsSort::Time,
                perf_threshold_agg: PerfThresholdAggregate::Max,
                cli_thresholds: perf_thresholds_with_pointer_move_limits(),
                baseline_thresholds: PerfThresholds::default(),
                warmup_frames: 0,
                top_total: 90,
                top_layout: 0,
                top_solve: 0,
                top_solves: 0,
                top_tick: 0,
                top_frame: 0,
                frame_p95_total_time_us: 0,
                frame_p95_layout_time_us: 0,
                frame_p95_layout_engine_solve_time_us: 0,
                pointer_move_frames_present: false,
                pointer_move_frames_considered: 0,
                pointer_move_max_dispatch_time_us: 99,
                pointer_move_max_hit_test_time_us: 99,
                pointer_move_snapshots_with_global_changes: 3,
                run_paint_cache_hit_test_only_replay_allowed_max: 0,
                run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: 0,
                max_renderer_encode_scene_us: 0,
                max_renderer_upload_us: 0,
                max_renderer_record_passes_us: 0,
                max_renderer_encoder_finish_us: 0,
                max_renderer_prepare_text_us: 0,
                max_renderer_prepare_svg_us: 0,
                max_renderer_instance_bytes: 0,
                max_renderer_encode_scene_text_ops: 0,
                bundle_path: bundle,
                thr_total: Some(100),
                src_total: Some("cli"),
                thr_layout: None,
                src_layout: None,
                thr_solve: None,
                src_solve: None,
                thr_frame_p95_total: None,
                src_frame_p95_total: None,
                thr_frame_p95_layout: None,
                src_frame_p95_layout: None,
                thr_frame_p95_solve: None,
                src_frame_p95_solve: None,
                thr_pointer_move_dispatch: Some(10),
                src_pointer_move_dispatch: Some("cli"),
                thr_pointer_move_hit_test: Some(20),
                src_pointer_move_hit_test: Some("cli"),
                thr_pointer_move_global_changes: Some(0),
                src_pointer_move_global_changes: Some("cli"),
                thr_paint_cache_hit_test_only_replay_allowed_max: None,
                src_paint_cache_hit_test_only_replay_allowed_max: None,
                thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: None,
                src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: None,
            },
        );

        assert!(failures.is_empty());
        assert_eq!(rows.len(), 1);
        assert!(rows[0]["thresholds"]["max_pointer_move_dispatch_us"].is_null());
        assert!(rows[0]["thresholds"]["max_pointer_move_hit_test_us"].is_null());
        assert!(rows[0]["thresholds"]["max_pointer_move_global_changes"].is_null());
        assert!(rows[0]["threshold_sources"]["max_pointer_move_dispatch_us"].is_null());
        assert!(rows[0]["max"]["pointer_move_max_dispatch_time_us"].as_u64() > Some(0));
    }

    #[test]
    fn repeat_threshold_row_omits_pointer_move_thresholds_when_frames_are_absent() {
        let mut rows = Vec::new();
        let mut failures = Vec::new();
        let runs_json = vec![serde_json::json!({
            "run_index": 0,
            "pointer_move_frames_present": false,
            "bundle": "target/no-pointer-move/bundle.schema2.json",
        })];
        let script_worst = Some((
            90,
            PathBuf::from("target/no-pointer-move/bundle.schema2.json"),
            0,
        ));

        push_repeat_threshold_row_and_failures(
            &mut rows,
            &mut failures,
            RepeatThresholdInputs {
                script_key: "tools/diag-scripts/no-pointer-move.json",
                sort: BundleStatsSort::Time,
                repeat: 1,
                runs_json: &runs_json,
                perf_threshold_agg: PerfThresholdAggregate::Max,
                cli_thresholds: perf_thresholds_with_pointer_move_limits(),
                baseline_thresholds: PerfThresholds::default(),
                warmup_frames: 0,
                renderer_evidence: RendererMetricEvidence {
                    encode_scene: None,
                    upload: None,
                    record_passes: None,
                    encoder_finish: None,
                    prepare_text: None,
                    prepare_svg: None,
                    instance_bytes: None,
                    encode_scene_text_ops: None,
                },
                observed_total: 90,
                max_total: 90,
                p95_total: 90,
                sorted_total: &vec![90],
                p90_total: 90,
                observed_layout: 0,
                max_layout: 0,
                p95_layout: 0,
                sorted_layout: &vec![0],
                p90_layout: 0,
                observed_solve: 0,
                max_solve: 0,
                p95_solve: 0,
                sorted_solve: &vec![0],
                p90_solve: 0,
                observed_frame_p95_total: 0,
                max_frame_p95_total: 0,
                p95_frame_p95_total: 0,
                sorted_frame_p95_total: &vec![0],
                p90_frame_p95_total: 0,
                observed_frame_p95_layout: 0,
                max_frame_p95_layout: 0,
                p95_frame_p95_layout: 0,
                sorted_frame_p95_layout: &vec![0],
                p90_frame_p95_layout: 0,
                observed_frame_p95_solve: 0,
                max_frame_p95_solve: 0,
                p95_frame_p95_solve: 0,
                sorted_frame_p95_solve: &vec![0],
                p90_frame_p95_solve: 0,
                pointer_move_frames_present: false,
                max_pointer_move_dispatch: 99,
                max_pointer_move_hit_test: 99,
                max_pointer_move_global_changes: 3,
                max_run_paint_cache_hit_test_only_replay_allowed_max: 0,
                max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: 0,
                observed_renderer_encode_scene_us: 0,
                max_renderer_encode_scene_us: 0,
                p95_renderer_encode_scene_us: 0,
                observed_renderer_upload_us: 0,
                max_renderer_upload_us: 0,
                p95_renderer_upload_us: 0,
                observed_renderer_record_passes_us: 0,
                max_renderer_record_passes_us: 0,
                p95_renderer_record_passes_us: 0,
                observed_renderer_encoder_finish_us: 0,
                max_renderer_encoder_finish_us: 0,
                p95_renderer_encoder_finish_us: 0,
                observed_renderer_prepare_text_us: 0,
                max_renderer_prepare_text_us: 0,
                p95_renderer_prepare_text_us: 0,
                observed_renderer_prepare_svg_us: 0,
                max_renderer_prepare_svg_us: 0,
                p95_renderer_prepare_svg_us: 0,
                observed_renderer_instance_bytes: 0,
                max_renderer_instance_bytes: 0,
                p95_renderer_instance_bytes: 0,
                observed_renderer_encode_scene_text_ops: 0,
                max_renderer_encode_scene_text_ops: 0,
                p95_renderer_encode_scene_text_ops: 0,
                script_worst: &script_worst,
                script_worst_layout: &None,
                script_worst_solve: &None,
                thr_total: Some(100),
                src_total: Some("cli"),
                thr_layout: None,
                src_layout: None,
                thr_solve: None,
                src_solve: None,
                thr_frame_p95_total: None,
                src_frame_p95_total: None,
                thr_frame_p95_layout: None,
                src_frame_p95_layout: None,
                thr_frame_p95_solve: None,
                src_frame_p95_solve: None,
                thr_pointer_move_dispatch: Some(10),
                src_pointer_move_dispatch: Some("cli"),
                thr_pointer_move_hit_test: Some(20),
                src_pointer_move_hit_test: Some("cli"),
                thr_pointer_move_global_changes: Some(0),
                src_pointer_move_global_changes: Some("cli"),
                thr_paint_cache_hit_test_only_replay_allowed_max: None,
                src_paint_cache_hit_test_only_replay_allowed_max: None,
                thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: None,
                src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: None,
            },
        );

        assert!(failures.is_empty());
        assert_eq!(rows.len(), 1);
        assert!(rows[0]["thresholds"]["max_pointer_move_dispatch_us"].is_null());
        assert!(rows[0]["thresholds"]["max_pointer_move_hit_test_us"].is_null());
        assert!(rows[0]["thresholds"]["max_pointer_move_global_changes"].is_null());
        assert!(rows[0]["threshold_sources"]["max_pointer_move_dispatch_us"].is_null());
        assert_eq!(rows[0]["max"]["pointer_move_max_dispatch_time_us"], 99);
    }
}
