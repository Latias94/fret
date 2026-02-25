use super::*;

pub(crate) fn push_perf_hint_row(
    rows: &mut Vec<serde_json::Value>,
    script_key: &str,
    sort: BundleStatsSort,
    repeat: usize,
    run_index: usize,
    bundle_path: &Path,
    warmup_frames: u64,
    hints: serde_json::Value,
    unit_costs: serde_json::Value,
    worst: serde_json::Value,
    trace_chrome_json_path: serde_json::Value,
    failures: Vec<serde_json::Value>,
) {
    rows.push(serde_json::json!({
        "script": script_key.to_string(),
        "sort": sort.as_str(),
        "repeat": repeat,
        "run_index": run_index,
        "bundle": bundle_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "hints": hints,
        "unit_costs": unit_costs,
        "worst": worst,
        "trace_chrome_json_path": trace_chrome_json_path,
        "failures": failures,
    }));
}
