use std::path::Path;

use serde_json::{Value, json};

use crate::frames_index::TriageLiteMetric;

const FRAMES_INDEX_KIND: &str = "frames_index";
const FRAMES_INDEX_SCHEMA_VERSION: u64 = 1;

fn col_index(columns: &[Value], name: &str) -> Option<usize> {
    columns
        .iter()
        .position(|c| c.as_str().is_some_and(|s| s == name))
}

fn row_u64(row: &[Value], idx: Option<usize>) -> Option<u64> {
    let idx = idx?;
    row.get(idx)?.as_u64()
}

fn row_metric(
    row: &[Value],
    metric: TriageLiteMetric,
    idx_total: Option<usize>,
    idx_layout: Option<usize>,
    idx_paint: Option<usize>,
) -> u64 {
    match metric {
        TriageLiteMetric::TotalTimeUs => row_u64(row, idx_total).unwrap_or(0),
        TriageLiteMetric::LayoutTimeUs => row_u64(row, idx_layout).unwrap_or(0),
        TriageLiteMetric::PaintTimeUs => row_u64(row, idx_paint).unwrap_or(0),
    }
}

#[derive(Debug, Clone)]
struct Entry {
    metric: u64,
    window: u64,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    timestamp_unix_ms: Option<u64>,
    total_time_us: Option<u64>,
    layout_time_us: Option<u64>,
    paint_time_us: Option<u64>,
    semantics_fingerprint: Option<u64>,
    semantics_source_tag: Option<u64>,
}

pub(crate) fn hotspots_lite_json_from_frames_index(
    bundle_path: &Path,
    frames_index_path: &Path,
    frames_index: &Value,
    warmup_frames: u64,
    top: usize,
    metric: TriageLiteMetric,
) -> Result<Value, String> {
    if frames_index.get("kind").and_then(|v| v.as_str()) != Some(FRAMES_INDEX_KIND) {
        return Err("invalid frames.index.json: kind mismatch".to_string());
    }
    if frames_index.get("schema_version").and_then(|v| v.as_u64())
        != Some(FRAMES_INDEX_SCHEMA_VERSION)
    {
        return Err("invalid frames.index.json: schema_version mismatch".to_string());
    }
    if frames_index.get("warmup_frames").and_then(|v| v.as_u64()) != Some(warmup_frames) {
        return Err("invalid frames.index.json: warmup_frames mismatch".to_string());
    }

    let columns = frames_index
        .get("columns")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing columns".to_string())?;

    let idx_frame_id = col_index(columns, "frame_id");
    let idx_seq = col_index(columns, "window_snapshot_seq");
    let idx_ts = col_index(columns, "timestamp_unix_ms");
    let idx_total = col_index(columns, "total_time_us");
    let idx_layout = col_index(columns, "layout_time_us");
    let idx_paint = col_index(columns, "paint_time_us");
    let idx_fp = col_index(columns, "semantics_fingerprint");
    let idx_sem_tag = col_index(columns, "semantics_source_tag");

    let windows = frames_index
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing windows".to_string())?;

    let top = top.max(1).min(5000);
    let metric_name = match metric {
        TriageLiteMetric::TotalTimeUs => "total_time_us",
        TriageLiteMetric::LayoutTimeUs => "layout_time_us",
        TriageLiteMetric::PaintTimeUs => "paint_time_us",
    };

    let mut best: Vec<Entry> = Vec::new();

    let mut frames_index_rows_total: u64 = 0;
    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let rows = w
            .get("rows")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        frames_index_rows_total = frames_index_rows_total.saturating_add(rows.len() as u64);

        for row in rows {
            let Some(row) = row.as_array() else {
                continue;
            };
            let score = row_metric(row, metric, idx_total, idx_layout, idx_paint);
            if score == 0 {
                continue;
            }

            let e = Entry {
                metric: score,
                window: window_id,
                frame_id: row_u64(row, idx_frame_id),
                window_snapshot_seq: row_u64(row, idx_seq),
                timestamp_unix_ms: row_u64(row, idx_ts),
                total_time_us: row_u64(row, idx_total),
                layout_time_us: row_u64(row, idx_layout),
                paint_time_us: row_u64(row, idx_paint),
                semantics_fingerprint: row_u64(row, idx_fp),
                semantics_source_tag: row_u64(row, idx_sem_tag),
            };

            if best.len() < top {
                best.push(e);
                continue;
            }
            let mut min_idx: usize = 0;
            for i in 1..best.len() {
                if best[i].metric < best[min_idx].metric {
                    min_idx = i;
                }
            }
            if e.metric > best[min_idx].metric {
                best[min_idx] = e;
            }
        }
    }

    best.sort_by(|a, b| b.metric.cmp(&a.metric));

    let results: Vec<Value> = best
        .into_iter()
        .map(|e| {
            let mut suggestions: Vec<String> = Vec::new();
            if let Some(fid) = e.frame_id {
                suggestions.push(format!(
                    "fretboard-dev diag slice {} --test-id <test_id> --window {} --frame-id {} --warmup-frames {}",
                    bundle_path.display(),
                    e.window,
                    fid,
                    warmup_frames
                ));
            } else if let Some(seq) = e.window_snapshot_seq {
                suggestions.push(format!(
                    "fretboard-dev diag slice {} --test-id <test_id> --window {} --snapshot-seq {} --warmup-frames {}",
                    bundle_path.display(),
                    e.window,
                    seq,
                    warmup_frames
                ));
            }

            json!({
                "window": e.window,
                "frame_id": e.frame_id,
                "window_snapshot_seq": e.window_snapshot_seq,
                "timestamp_unix_ms": e.timestamp_unix_ms,
                "metric": { metric_name: e.metric },
                "stats": {
                    "total_time_us": e.total_time_us,
                    "layout_time_us": e.layout_time_us,
                    "paint_time_us": e.paint_time_us,
                },
                "semantics": {
                    "fingerprint": e.semantics_fingerprint,
                    "source_tag": e.semantics_source_tag,
                },
                "suggestions": suggestions,
            })
        })
        .collect();

    Ok(json!({
        "schema_version": 1,
        "kind": "diag.hotspots_lite",
        "bundle": bundle_path.display().to_string(),
        "generated_unix_ms": crate::util::now_unix_ms(),
        "warmup_frames": warmup_frames,
        "source": {
            "kind": FRAMES_INDEX_KIND,
            "schema_version": FRAMES_INDEX_SCHEMA_VERSION,
            "path": frames_index_path.display().to_string(),
        },
        "metric": {
            "name": metric_name,
            "top": top,
        },
        "frames_index_rows_total": frames_index_rows_total,
        "notes": [
            "hotspots_lite is derived from frames.index.json to avoid materializing bundle artifacts in memory.",
            "This report identifies slow frames (perf hotspots), not JSON subtree size hotspots.",
        ],
        "results": results,
    }))
}
