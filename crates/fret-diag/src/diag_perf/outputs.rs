use super::*;
use crate::perf_hint_gate::PerfHintGateOptions;

pub(crate) fn write_perf_baseline_json(
    out_path: &Path,
    warmup_frames: u64,
    sort: BundleStatsSort,
    repeat: usize,
    headroom_pct: u32,
    threshold_seed_policy: serde_json::Value,
    rows: &[serde_json::Value],
    stats_json: bool,
) -> Result<(), String> {
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "perf_baseline",
        "out_path": out_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "sort": sort.as_str(),
        "repeat": repeat,
        "headroom_pct": headroom_pct,
        "threshold_seed_policy": threshold_seed_policy,
        "rows": rows,
    });
    write_json_value(out_path, &payload)?;
    if !stats_json {
        println!("wrote perf baseline: {}", out_path.display());
    }
    Ok(())
}

pub(crate) fn write_perf_thresholds_json(
    out_dir: &Path,
    warmup_frames: u64,
    observed_aggregate: PerfThresholdAggregate,
    prewarm_scripts: &[PathBuf],
    prelude_scripts: &[PathBuf],
    prelude_each_run: bool,
    cli_thresholds: &PerfThresholds,
    baseline_summary: Option<serde_json::Value>,
    rows: &[serde_json::Value],
    failures: &[serde_json::Value],
) -> PathBuf {
    let out_path = out_dir.join("check.perf_thresholds.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "perf_thresholds",
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "observed_aggregate": observed_aggregate.as_str(),
        "suite_hooks": {
            "prewarm": prewarm_scripts.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "prelude": prelude_scripts.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "prelude_each_run": prelude_each_run,
        },
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
        "baseline": baseline_summary,
        "rows": rows,
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);
    out_path
}

pub(crate) fn write_perf_hints_json(
    out_dir: &Path,
    warmup_frames: u64,
    perf_hint_gate_opts: &PerfHintGateOptions,
    rows: &[serde_json::Value],
    failures: &[serde_json::Value],
) -> PathBuf {
    let out_path = out_dir.join("check.perf_hints.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "perf_hints",
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_severity": perf_hint_gate_opts.min_severity.as_str(),
        "deny": perf_hint_gate_opts.deny_codes.iter().cloned().collect::<Vec<_>>(),
        "rows": rows,
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);
    out_path
}
