use super::*;

fn json_u64(v: &serde_json::Value, key: &str) -> u64 {
    v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
}

pub(super) fn print_perf_no_last_bundle_dir(src: &Path, sort: BundleStatsSort, repeat: Option<usize>) {
    match repeat {
        Some(repeat) => {
            println!(
                "PERF {} sort={} repeat={} (no last_bundle_dir recorded)",
                src.display(),
                sort.as_str(),
                repeat
            );
        }
        None => {
            println!(
                "PERF {} sort={} (no last_bundle_dir recorded)",
                src.display(),
                sort.as_str()
            );
        }
    }
}

pub(super) fn push_perf_json_no_last_bundle_dir(
    perf_json_rows: &mut Vec<serde_json::Value>,
    script: String,
    sort: BundleStatsSort,
    repeat: Option<usize>,
) {
    let mut obj = serde_json::Map::new();
    obj.insert("script".to_string(), serde_json::Value::String(script));
    obj.insert(
        "sort".to_string(),
        serde_json::Value::String(sort.as_str().to_string()),
    );
    if let Some(repeat) = repeat {
        obj.insert(
            "repeat".to_string(),
            serde_json::Value::Number(serde_json::Number::from(repeat as u64)),
        );
    }
    obj.insert(
        "error".to_string(),
        serde_json::Value::String("no_last_bundle_dir".to_string()),
    );
    perf_json_rows.push(serde_json::Value::Object(obj));
}

pub(super) fn print_perf_repeat_summary(
    src: &Path,
    sort: BundleStatsSort,
    repeat: usize,
    total: &serde_json::Value,
    layout: &serde_json::Value,
    solve: &serde_json::Value,
    prepaint: &serde_json::Value,
    paint: &serde_json::Value,
    dispatch: &serde_json::Value,
    hit_test: &serde_json::Value,
) {
    println!(
        "PERF {} sort={} repeat={} p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{} p95.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{} max.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{}",
        src.display(),
        sort.as_str(),
        repeat,
        json_u64(total, "p50"),
        json_u64(layout, "p50"),
        json_u64(solve, "p50"),
        json_u64(prepaint, "p50"),
        json_u64(paint, "p50"),
        json_u64(dispatch, "p50"),
        json_u64(hit_test, "p50"),
        json_u64(total, "p95"),
        json_u64(layout, "p95"),
        json_u64(solve, "p95"),
        json_u64(prepaint, "p95"),
        json_u64(paint, "p95"),
        json_u64(dispatch, "p95"),
        json_u64(hit_test, "p95"),
        json_u64(total, "max"),
        json_u64(layout, "max"),
        json_u64(solve, "max"),
        json_u64(prepaint, "max"),
        json_u64(paint, "max"),
        json_u64(dispatch, "max"),
        json_u64(hit_test, "max"),
    );
}
