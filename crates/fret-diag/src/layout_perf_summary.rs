use std::path::Path;

use serde_json::{Value, json};

pub(crate) const DEFAULT_LAYOUT_PERF_SUMMARY_TOP: usize = 8;

pub(crate) fn layout_perf_summary_v1_from_bundle_path_strict(
    bundle_path: &Path,
    bundle_dir: &Path,
    warmup_frames: u64,
    top: usize,
) -> Result<Value, String> {
    let report = crate::stats::bundle_stats_from_path(
        bundle_path,
        1,
        crate::stats::BundleStatsSort::Time,
        crate::stats::BundleStatsOptions { warmup_frames },
    )?;

    let stats = report.to_json();
    if stats
        .get("derived_from_frames_index")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return Err(format!(
            "layout perf summary requires full bundle stats, but this bundle is too large and stats were derived from frames.index.json.\n  bundle_artifact: {}\n  hint: ensure a compact `bundle.schema2.json` is available (try: fretboard diag doctor --fix-schema2 <bundle_dir> --warmup-frames {warmup_frames})",
            bundle_path.display()
        ));
    }

    layout_perf_summary_v1_from_stats_json(
        &stats,
        bundle_path.display().to_string(),
        bundle_dir.display().to_string(),
        warmup_frames,
        top,
    )
}

pub(crate) fn layout_perf_summary_v1_from_bundle_path_best_effort(
    bundle_path: &Path,
    bundle_dir: &Path,
    warmup_frames: u64,
    top: usize,
) -> Value {
    match layout_perf_summary_v1_from_bundle_path_strict(
        bundle_path,
        bundle_dir,
        warmup_frames,
        top,
    ) {
        Ok(v) => v,
        Err(err) => json!({
            "schema_version": 1,
            "kind": "layout_perf_summary",
            "bundle_artifact": bundle_path.display().to_string(),
            "bundle_dir": bundle_dir.display().to_string(),
            "warmup_frames": warmup_frames,
            "top": top,
            "error": {
                "code": "layout_perf_summary.unavailable",
                "message": err,
            }
        }),
    }
}

pub(crate) fn human_layout_perf_summary_v1(summary: &Value) -> String {
    fn u64_field(v: &Value, key: &str) -> u64 {
        v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
    }
    fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
        v.get(key).and_then(|v| v.as_str()).unwrap_or("")
    }

    let mut out = String::new();
    out.push_str("layout_perf_summary:\n");
    out.push_str(&format!(
        "  bundle_artifact: {}\n",
        str_field(summary, "bundle_artifact")
    ));
    out.push_str(&format!(
        "  bundle_dir: {}\n",
        str_field(summary, "bundle_dir")
    ));
    out.push_str(&format!(
        "  warmup_frames: {}\n",
        u64_field(summary, "warmup_frames")
    ));

    if let Some(err) = summary.get("error") {
        out.push_str(&format!(
            "  error: {}\n",
            err.get("message").and_then(|v| v.as_str()).unwrap_or("")
        ));
        return out;
    }

    let frame = summary.get("frame").unwrap_or(&Value::Null);
    out.push_str(&format!(
        "  frame: window={} tick_id={} frame_id={} window_snapshot_seq={} ts_unix_ms={}\n",
        u64_field(frame, "window"),
        u64_field(frame, "tick_id"),
        u64_field(frame, "frame_id"),
        frame
            .get("window_snapshot_seq")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string()),
        frame
            .get("timestamp_unix_ms")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string()),
    ));

    let stats = summary.get("stats").unwrap_or(&Value::Null);
    out.push_str(&format!(
        "  stats: total_us={} layout_us={} solve_us={} solves={}\n",
        u64_field(stats, "total_time_us"),
        u64_field(stats, "layout_time_us"),
        u64_field(stats, "layout_engine_solve_time_us"),
        u64_field(stats, "layout_engine_solves"),
    ));

    fn print_list_header(out: &mut String, name: &str, items: usize, clipped: bool) {
        out.push_str(&format!(
            "  {name}: items={}{}\n",
            items,
            if clipped { " (clipped)" } else { "" }
        ));
    }

    let clipped = summary.get("clipped").unwrap_or(&Value::Null);

    let solves = summary
        .get("top_layout_engine_solves")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v.as_slice());
    print_list_header(
        &mut out,
        "top_layout_engine_solves",
        solves.len(),
        clipped
            .get("top_layout_engine_solves")
            .and_then(|v| v.get("clipped"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    );
    for s in solves.iter().take(8) {
        out.push_str(&format!(
            "    - solve_us={} root_kind={} root_path={} root_test_id={}\n",
            u64_field(s, "solve_time_us"),
            s.get("root_element_kind")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            s.get("root_element_path")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            s.get("root_test_id").and_then(|v| v.as_str()).unwrap_or(""),
        ));
    }

    let hotspots = summary
        .get("layout_hotspots")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v.as_slice());
    print_list_header(
        &mut out,
        "layout_hotspots",
        hotspots.len(),
        clipped
            .get("layout_hotspots")
            .and_then(|v| v.get("clipped"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    );
    for h in hotspots.iter().take(8) {
        out.push_str(&format!(
            "    - layout_us={} inclusive_us={} kind={} path={} test_id={}\n",
            u64_field(h, "layout_time_us"),
            u64_field(h, "inclusive_time_us"),
            h.get("element_kind").and_then(|v| v.as_str()).unwrap_or(""),
            h.get("element_path").and_then(|v| v.as_str()).unwrap_or(""),
            h.get("test_id").and_then(|v| v.as_str()).unwrap_or(""),
        ));
    }

    let measures = summary
        .get("widget_measure_hotspots")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v.as_slice());
    print_list_header(
        &mut out,
        "widget_measure_hotspots",
        measures.len(),
        clipped
            .get("widget_measure_hotspots")
            .and_then(|v| v.get("clipped"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    );
    for h in measures.iter().take(8) {
        out.push_str(&format!(
            "    - measure_us={} inclusive_us={} kind={} path={} test_id={}\n",
            u64_field(h, "measure_time_us"),
            u64_field(h, "inclusive_time_us"),
            h.get("element_kind").and_then(|v| v.as_str()).unwrap_or(""),
            h.get("element_path").and_then(|v| v.as_str()).unwrap_or(""),
            h.get("test_id").and_then(|v| v.as_str()).unwrap_or(""),
        ));
    }

    out
}

pub(crate) fn layout_perf_summary_v1_from_stats_json(
    stats: &Value,
    bundle_artifact: String,
    bundle_dir: String,
    warmup_frames: u64,
    top: usize,
) -> Result<Value, String> {
    let top_rows = stats
        .get("top")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let Some(row) = top_rows.first() else {
        return Err(
            "bundle stats report has no top rows (did the bundle contain snapshots?)".to_string(),
        );
    };

    let mut clipped = serde_json::Map::new();

    let top_layout_engine_solves =
        clip_array_field(row, "top_layout_engine_solves", top, &mut clipped);
    let layout_hotspots = clip_array_field(row, "layout_hotspots", top, &mut clipped);
    let widget_measure_hotspots =
        clip_array_field(row, "widget_measure_hotspots", top, &mut clipped);

    let frame = json!({
        "window": row.get("window").and_then(|v| v.as_u64()).unwrap_or(0),
        "tick_id": row.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
        "frame_id": row.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
        "window_snapshot_seq": row.get("window_snapshot_seq").and_then(|v| v.as_u64()),
        "timestamp_unix_ms": row.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
    });

    let stats_out = json!({
        "total_time_us": row.get("total_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
        "layout_time_us": row.get("layout_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
        "layout_engine_solve_time_us": row.get("layout_engine_solve_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
        "layout_engine_solves": row.get("layout_engine_solves").and_then(|v| v.as_u64()).unwrap_or(0),
    });

    Ok(json!({
        "schema_version": 1,
        "kind": "layout_perf_summary",
        "bundle_artifact": bundle_artifact,
        "bundle_dir": bundle_dir,
        "warmup_frames": warmup_frames,
        "top": top,
        "frame": frame,
        "stats": stats_out,
        "top_layout_engine_solves": top_layout_engine_solves,
        "layout_hotspots": layout_hotspots,
        "widget_measure_hotspots": widget_measure_hotspots,
        "clipped": clipped,
    }))
}

fn clip_array_field(
    row: &Value,
    key: &str,
    top: usize,
    clipped: &mut serde_json::Map<String, Value>,
) -> Value {
    let Some(arr) = row.get(key).and_then(|v| v.as_array()) else {
        return Value::Array(Vec::new());
    };

    let clipped_here = arr.len() > top;
    if clipped_here {
        clipped.insert(
            key.to_string(),
            json!({
                "clipped": true,
                "kept": top,
                "dropped": arr.len().saturating_sub(top),
            }),
        );
    } else {
        clipped.insert(
            key.to_string(),
            json!({
                "clipped": false,
                "kept": arr.len(),
                "dropped": 0,
            }),
        );
    }

    Value::Array(arr.iter().take(top).cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_extracts_layout_lists_from_stats_json() {
        let bundle = json!({
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "events": [],
                "snapshots": [{
                    "schema_version": 1,
                    "tick_id": 7,
                    "frame_id": 11,
                    "window": 1,
                    "window_snapshot_seq": 3,
                    "timestamp_unix_ms": 123,
                    "debug": {
                        "stats": {
                            "total_time_us": 10_000,
                            "layout_time_us": 4_000,
                            "layout_engine_solves": 2,
                            "layout_engine_solve_time_us": 2_500,
                            "prepaint_time_us": 0,
                            "paint_time_us": 0,
                            "dispatch_time_us": 0,
                            "hit_test_time_us": 0,
                        },
                        "layout_engine_solves": [{
                            "root_node": 10,
                            "root_element": 42,
                            "root_element_kind": "row",
                            "root_element_path": "root/row",
                            "solve_time_us": 2_500,
                            "measure_calls": 4,
                            "measure_cache_hits": 1,
                            "measure_time_us": 1_200,
                            "top_measures": [{
                                "node": 11,
                                "measure_time_us": 500,
                                "calls": 2,
                                "cache_hits": 1,
                                "element": 43,
                                "element_kind": "text",
                                "top_children": [{
                                    "child": 12,
                                    "measure_time_us": 200,
                                    "calls": 1,
                                    "element": 44,
                                    "element_kind": "icon",
                                }]
                            }]
                        }],
                        "layout_hotspots": [{
                            "node": 100,
                            "element": 50,
                            "element_kind": "vlist",
                            "element_path": "root/vlist",
                            "widget_type": "virtual_list",
                            "layout_time_us": 800,
                            "inclusive_time_us": 900,
                        }],
                        "widget_measure_hotspots": [{
                            "node": 200,
                            "element": 51,
                            "element_kind": "text",
                            "element_path": "root/text",
                            "widget_type": "text",
                            "measure_time_us": 700,
                            "inclusive_time_us": 700,
                        }],
                    }
                }]
            }]
        });

        let report = crate::stats::bundle_stats_from_json_with_options(
            &bundle,
            1,
            crate::stats::BundleStatsSort::Time,
            crate::stats::BundleStatsOptions { warmup_frames: 0 },
        )
        .expect("bundle stats");

        let stats = report.to_json();
        let summary = layout_perf_summary_v1_from_stats_json(
            &stats,
            "bundle.schema2.json".to_string(),
            "bundle_dir".to_string(),
            0,
            8,
        )
        .expect("summary");

        assert_eq!(
            summary.get("kind").and_then(|v| v.as_str()),
            Some("layout_perf_summary")
        );
        assert_eq!(
            summary
                .get("frame")
                .and_then(|v| v.get("frame_id"))
                .and_then(|v| v.as_u64()),
            Some(11)
        );
        assert_eq!(
            summary
                .get("stats")
                .and_then(|v| v.get("layout_engine_solve_time_us"))
                .and_then(|v| v.as_u64()),
            Some(2_500)
        );

        let solves = summary
            .get("top_layout_engine_solves")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(solves.len(), 1);
        assert_eq!(
            solves[0].get("root_element_kind").and_then(|v| v.as_str()),
            Some("row")
        );

        let hotspots = summary
            .get("layout_hotspots")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(hotspots.len(), 1);
        assert_eq!(
            hotspots[0].get("element_kind").and_then(|v| v.as_str()),
            Some("vlist")
        );
    }

    #[test]
    fn summary_clips_arrays_by_top() {
        let stats = json!({
            "top": [{
                "window": 1,
                "tick_id": 1,
                "frame_id": 1,
                "layout_engine_solves": 1,
                "layout_engine_solve_time_us": 1,
                "layout_time_us": 1,
                "total_time_us": 1,
                "top_layout_engine_solves": [{ "solve_time_us": 1 }, { "solve_time_us": 2 }],
                "layout_hotspots": [{ "layout_time_us": 1 }, { "layout_time_us": 2 }],
                "widget_measure_hotspots": [{ "measure_time_us": 1 }, { "measure_time_us": 2 }],
            }]
        });

        let summary = layout_perf_summary_v1_from_stats_json(
            &stats,
            "bundle".to_string(),
            "dir".to_string(),
            0,
            1,
        )
        .expect("summary");

        assert_eq!(
            summary
                .get("top_layout_engine_solves")
                .and_then(|v| v.as_array())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            summary
                .get("clipped")
                .and_then(|v| v.get("top_layout_engine_solves"))
                .and_then(|v| v.get("clipped"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
