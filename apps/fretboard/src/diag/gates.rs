struct ResourceFootprintThresholds {
    max_working_set_bytes: Option<u64>,
    max_peak_working_set_bytes: Option<u64>,
    max_cpu_avg_percent_total_cores: Option<f64>,
}

impl ResourceFootprintThresholds {
    fn any(&self) -> bool {
        self.max_working_set_bytes.is_some()
            || self.max_peak_working_set_bytes.is_some()
            || self.max_cpu_avg_percent_total_cores.is_some()
    }
}

#[derive(Debug, Clone)]
struct ResourceFootprintGateResult {
    evidence_path: PathBuf,
    failures: usize,
}

fn check_resource_footprint_thresholds(
    out_dir: &Path,
    footprint_path: &Path,
    thresholds: &ResourceFootprintThresholds,
) -> Result<ResourceFootprintGateResult, String> {
    let out_path = out_dir.join("check.resource_footprint.json");
    let v = read_json_value(footprint_path);
    let footprint_present = v.is_some();

    let pid = v
        .as_ref()
        .and_then(|v| v.get("pid"))
        .and_then(|v| v.as_u64());
    let killed = v
        .as_ref()
        .and_then(|v| v.get("killed"))
        .and_then(|v| v.as_bool());
    let wall_time_ms = v
        .as_ref()
        .and_then(|v| v.get("wall_time_ms"))
        .and_then(|v| v.as_u64());
    let logical_cores = v
        .as_ref()
        .and_then(|v| v.get("logical_cores"))
        .and_then(|v| v.as_u64());
    let note = v
        .as_ref()
        .and_then(|v| v.get("note"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let working_set_bytes = v
        .as_ref()
        .and_then(|v| v.get("memory"))
        .and_then(|v| v.get("working_set_bytes"))
        .and_then(|v| v.as_u64());
    let peak_working_set_bytes = v
        .as_ref()
        .and_then(|v| v.get("memory"))
        .and_then(|v| v.get("peak_working_set_bytes"))
        .and_then(|v| v.as_u64());

    let cpu_avg_percent_total_cores = v
        .as_ref()
        .and_then(|v| v.get("cpu"))
        .and_then(|v| v.get("avg_cpu_percent_total_cores"))
        .and_then(|v| v.as_f64());
    let cpu_samples = v
        .as_ref()
        .and_then(|v| v.get("cpu"))
        .and_then(|v| v.get("samples"))
        .and_then(|v| v.as_u64());
    let cpu_collector = v
        .as_ref()
        .and_then(|v| v.get("cpu"))
        .and_then(|v| v.get("collector"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut failures: Vec<serde_json::Value> = Vec::new();
    let missing_reason = if footprint_present {
        "missing_field"
    } else {
        "missing_footprint"
    };

    if let Some(thr) = thresholds.max_working_set_bytes {
        match working_set_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "working_set_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "working_set_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "memory.working_set_bytes",
            })),
        }
    }

    if let Some(thr) = thresholds.max_peak_working_set_bytes {
        match peak_working_set_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "peak_working_set_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "peak_working_set_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "memory.peak_working_set_bytes",
            })),
        }
    }

    if let Some(thr) = thresholds.max_cpu_avg_percent_total_cores {
        match cpu_avg_percent_total_cores {
            Some(observed) => {
                if cpu_samples == Some(0) && cpu_collector.as_deref() == Some("sysinfo") {
                    failures.push(serde_json::json!({
                        "kind": "cpu_avg_percent_total_cores",
                        "threshold": thr,
                        "observed": observed,
                        "reason": "insufficient_samples",
                        "samples": cpu_samples,
                    }));
                } else if observed > thr {
                    failures.push(serde_json::json!({
                        "kind": "cpu_avg_percent_total_cores",
                        "threshold": thr,
                        "observed": observed,
                        "reason": "exceeded",
                    }));
                }
            }
            None => failures.push(serde_json::json!({
                "kind": "cpu_avg_percent_total_cores",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "cpu.avg_cpu_percent_total_cores",
            })),
        }
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "resource_footprint_thresholds",
        "out_dir": out_dir.display().to_string(),
        "footprint_file": footprint_path.file_name().and_then(|s| s.to_str()).unwrap_or("resource.footprint.json"),
        "thresholds": {
            "max_working_set_bytes": thresholds.max_working_set_bytes,
            "max_peak_working_set_bytes": thresholds.max_peak_working_set_bytes,
            "max_cpu_avg_percent_total_cores": thresholds.max_cpu_avg_percent_total_cores,
        },
        "observed": {
            "present": footprint_present,
            "pid": pid,
            "killed": killed,
            "wall_time_ms": wall_time_ms,
            "logical_cores": logical_cores,
            "note": note,
            "cpu_collector": cpu_collector,
            "cpu_samples": cpu_samples,
            "cpu_avg_percent_total_cores": cpu_avg_percent_total_cores,
            "working_set_bytes": working_set_bytes,
            "peak_working_set_bytes": peak_working_set_bytes,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(ResourceFootprintGateResult {
        evidence_path: out_path,
        failures,
    })
}

#[derive(Debug, Clone)]
struct RedrawHitchesGateResult {
    evidence_path: PathBuf,
    failures: usize,
}

#[derive(Debug, Clone)]
struct RedrawHitchRecord {
    line_no: usize,
    ts_unix_ms: Option<u64>,
    tick_id: Option<u64>,
    frame_id: Option<u64>,
    total_ms: u64,
    prepare_ms: Option<u64>,
    render_ms: Option<u64>,
    record_ms: Option<u64>,
    present_ms: Option<u64>,
    scene_ops: Option<u64>,
    line: String,
}

fn check_redraw_hitches_max_total_ms(
    out_dir: &Path,
    max_total_ms: u64,
) -> Result<RedrawHitchesGateResult, String> {
    let log_path = out_dir.join("redraw_hitches.log");
    let out_path = out_dir.join("check.redraw_hitches.json");

    let parse_u64_after = |s: &str, needle: &str| -> Option<u64> {
        let start = s.find(needle)? + needle.len();
        let bytes = s.as_bytes();
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end == start {
            return None;
        }
        s.get(start..end)?.parse::<u64>().ok()
    };

    let parse_opt_u64_dbg = |s: &str, key: &str| -> Option<u64> {
        let needle = format!("{key}=Some(");
        let start = s.find(&needle)? + needle.len();
        let bytes = s.as_bytes();
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end == start {
            return None;
        }
        s.get(start..end)?.parse::<u64>().ok()
    };

    let parse_ts = |s: &str| -> Option<u64> {
        let s = s.strip_prefix('[')?;
        let end = s.find(']')?;
        s.get(0..end)?.parse::<u64>().ok()
    };

    let truncate = |s: &str, max_chars: usize| -> String {
        if s.chars().count() <= max_chars {
            return s.to_string();
        }
        s.chars().take(max_chars).collect()
    };

    let contents = std::fs::read_to_string(&log_path).ok();
    let present = contents.is_some();

    let mut records: Vec<RedrawHitchRecord> = Vec::new();
    if let Some(contents) = contents.as_ref() {
        for (idx, line) in contents.lines().enumerate() {
            let Some(total_ms) = parse_u64_after(line, "total_ms=") else {
                continue;
            };
            records.push(RedrawHitchRecord {
                line_no: idx.saturating_add(1),
                ts_unix_ms: parse_ts(line),
                tick_id: parse_u64_after(line, "tick_id="),
                frame_id: parse_u64_after(line, "frame_id="),
                total_ms,
                prepare_ms: parse_opt_u64_dbg(line, "prepare_ms"),
                render_ms: parse_opt_u64_dbg(line, "render_ms"),
                record_ms: parse_opt_u64_dbg(line, "record_ms"),
                present_ms: parse_opt_u64_dbg(line, "present_ms"),
                scene_ops: parse_u64_after(line, "scene_ops="),
                line: truncate(line, 400),
            });
        }
    }

    let mut totals: Vec<u64> = records.iter().map(|r| r.total_ms).collect();
    totals.sort_unstable();

    let max_observed = totals.last().copied();
    let avg_observed = if totals.is_empty() {
        None
    } else {
        Some(totals.iter().sum::<u64>() as f64 / totals.len() as f64)
    };
    let p95_observed = if totals.is_empty() {
        None
    } else {
        let idx = (totals.len().saturating_sub(1)) * 95 / 100;
        totals.get(idx).copied()
    };

    let mut failures: Vec<serde_json::Value> = Vec::new();
    if !present {
        failures.push(serde_json::json!({
            "kind": "log_file",
            "reason": "missing_log",
            "file": log_path.file_name().and_then(|s| s.to_str()).unwrap_or("redraw_hitches.log"),
        }));
    } else if records.is_empty() {
        failures.push(serde_json::json!({
            "kind": "parse",
            "reason": "no_records",
            "field": "total_ms",
        }));
    }

    if let Some(observed) = max_observed
        && observed > max_total_ms
    {
        failures.push(serde_json::json!({
            "kind": "max_total_ms",
            "threshold": max_total_ms,
            "observed": observed,
            "reason": "exceeded",
        }));
    }

    records.sort_by(|a, b| b.total_ms.cmp(&a.total_ms));
    let top = records
        .iter()
        .take(10)
        .map(|r| {
            serde_json::json!({
                "line_no": r.line_no,
                "ts_unix_ms": r.ts_unix_ms,
                "tick_id": r.tick_id,
                "frame_id": r.frame_id,
                "total_ms": r.total_ms,
                "prepare_ms": r.prepare_ms,
                "render_ms": r.render_ms,
                "record_ms": r.record_ms,
                "present_ms": r.present_ms,
                "scene_ops": r.scene_ops,
                "line": r.line,
            })
        })
        .collect::<Vec<_>>();

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "redraw_hitches_thresholds",
        "out_dir": out_dir.display().to_string(),
        "log_file": log_path.file_name().and_then(|s| s.to_str()).unwrap_or("redraw_hitches.log"),
        "thresholds": {
            "max_total_ms": max_total_ms,
        },
        "observed": {
            "present": present,
            "records": totals.len(),
            "max_total_ms": max_observed,
            "p95_total_ms": p95_observed,
            "avg_total_ms": avg_observed,
        },
        "top": top,
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(RedrawHitchesGateResult {
        evidence_path: out_path,
        failures,
    })
}

