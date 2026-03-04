use std::path::{Path, PathBuf};

use super::util::{now_unix_ms, read_json_value, write_json_value};

pub(super) struct ResourceFootprintThresholds {
    pub(super) max_working_set_bytes: Option<u64>,
    pub(super) max_peak_working_set_bytes: Option<u64>,
    pub(super) max_macos_physical_footprint_peak_bytes: Option<u64>,
    pub(super) max_macos_owned_unmapped_memory_dirty_bytes: Option<u64>,
    pub(super) max_cpu_avg_percent_total_cores: Option<f64>,
}

impl ResourceFootprintThresholds {
    pub(super) fn any(&self) -> bool {
        self.max_working_set_bytes.is_some()
            || self.max_peak_working_set_bytes.is_some()
            || self.max_macos_physical_footprint_peak_bytes.is_some()
            || self.max_macos_owned_unmapped_memory_dirty_bytes.is_some()
            || self.max_cpu_avg_percent_total_cores.is_some()
    }
}

#[derive(Debug, Clone)]
pub(super) struct ResourceFootprintGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Debug, Clone)]
pub(super) struct WgpuMetalAllocatedSizeGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Debug, Clone)]
pub(super) struct RenderTextAtlasBytesGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

pub(super) fn check_wgpu_metal_current_allocated_size_threshold(
    out_dir: &Path,
    bundle_path: Option<&Path>,
    max_wgpu_metal_current_allocated_size_bytes: u64,
) -> Result<WgpuMetalAllocatedSizeGateResult, String> {
    let out_path = out_dir.join("check.wgpu_metal_allocated_size.json");

    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let (tick_id, frame_id, present_flag, bytes_value) = if let Some(v) = v.as_ref() {
        let windows = v.get("windows").and_then(|v| v.as_array());
        let first_window = windows.and_then(|w| w.first());
        let snapshots = first_window
            .and_then(|w| w.get("snapshots"))
            .and_then(|v| v.as_array());
        let last_snapshot = snapshots.and_then(|s| s.last());
        let stats = last_snapshot
            .and_then(|s| s.get("debug"))
            .and_then(|d| d.get("stats"))
            .and_then(|v| v.as_object());

        let tick_id = last_snapshot
            .and_then(|s| s.get("tick_id"))
            .and_then(|v| v.as_u64());
        let frame_id = last_snapshot
            .and_then(|s| s.get("frame_id"))
            .and_then(|v| v.as_u64());
        let present_flag = stats
            .and_then(|o| o.get("wgpu_metal_current_allocated_size_present"))
            .and_then(|v| v.as_bool());
        let bytes_value = stats
            .and_then(|o| o.get("wgpu_metal_current_allocated_size_bytes"))
            .and_then(|v| v.as_u64());
        (tick_id, frame_id, present_flag, bytes_value)
    } else {
        (None, None, None, None)
    };

    let missing_reason = if bundle_present {
        "missing_field"
    } else {
        "missing_bundle"
    };

    let mut failures: Vec<serde_json::Value> = Vec::new();
    match (present_flag, bytes_value) {
        (Some(true), Some(observed)) if observed > max_wgpu_metal_current_allocated_size_bytes => {
            failures.push(serde_json::json!({
                "kind": "wgpu_metal_current_allocated_size_bytes",
                "threshold": max_wgpu_metal_current_allocated_size_bytes,
                "observed": observed,
                "reason": "exceeded",
            }));
        }
        (Some(true), Some(_)) => {}
        (Some(true), None) => failures.push(serde_json::json!({
            "kind": "wgpu_metal_current_allocated_size_bytes",
            "threshold": max_wgpu_metal_current_allocated_size_bytes,
            "observed": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].debug.stats.wgpu_metal_current_allocated_size_bytes",
        })),
        (Some(false), _) | (None, _) => failures.push(serde_json::json!({
            "kind": "wgpu_metal_current_allocated_size_bytes",
            "threshold": max_wgpu_metal_current_allocated_size_bytes,
            "observed": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].debug.stats.wgpu_metal_current_allocated_size_bytes",
        })),
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "wgpu_metal_current_allocated_size_threshold",
        "out_dir": out_dir.display().to_string(),
        "bundle_file": bundle_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>"),
        "thresholds": {
            "max_wgpu_metal_current_allocated_size_bytes": max_wgpu_metal_current_allocated_size_bytes,
        },
        "observed": {
            "bundle_present": bundle_present,
            "tick_id": tick_id,
            "frame_id": frame_id,
            "wgpu_metal_current_allocated_size_present": present_flag,
            "wgpu_metal_current_allocated_size_bytes": bytes_value,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(WgpuMetalAllocatedSizeGateResult {
        evidence_path: out_path,
        failures,
    })
}

pub(super) fn check_render_text_atlas_bytes_live_estimate_total_threshold(
    out_dir: &Path,
    bundle_path: Option<&Path>,
    max_render_text_atlas_bytes_live_estimate_total: u64,
) -> Result<RenderTextAtlasBytesGateResult, String> {
    let out_path = out_dir.join("check.render_text_atlas_bytes.json");

    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let (tick_id, frame_id, present_flag, mask_bytes, color_bytes, subpixel_bytes, total_bytes) =
        if let Some(v) = v.as_ref() {
            let windows = v.get("windows").and_then(|v| v.as_array());
            let first_window = windows.and_then(|w| w.first());
            let snapshots = first_window
                .and_then(|w| w.get("snapshots"))
                .and_then(|v| v.as_array());
            let last_snapshot = snapshots.and_then(|s| s.last());

            let tick_id = last_snapshot
                .and_then(|s| s.get("tick_id"))
                .and_then(|v| v.as_u64());
            let frame_id = last_snapshot
                .and_then(|s| s.get("frame_id"))
                .and_then(|v| v.as_u64());

            let render_text = last_snapshot
                .and_then(|s| s.get("resource_caches"))
                .and_then(|v| v.get("render_text"))
                .and_then(|v| v.as_object());

            let rt_atlas = |k: &str| {
                render_text
                    .and_then(|o| o.get(k))
                    .and_then(|v| v.as_object())
            };
            let atlas_u64 = |atlas: Option<&serde_json::Map<String, serde_json::Value>>,
                             k: &str| {
                atlas.and_then(|o| o.get(k)).and_then(|v| v.as_u64())
            };
            let sat_mul_u64 = |a: u64, b: u64| -> u64 {
                ((a as u128) * (b as u128)).min(u64::MAX as u128) as u64
            };
            let atlas_bytes = |atlas: Option<&serde_json::Map<String, serde_json::Value>>,
                               bpp: u64|
             -> Option<u64> {
                let w = atlas_u64(atlas, "width")?;
                let h = atlas_u64(atlas, "height")?;
                let pages = atlas_u64(atlas, "pages")?;
                Some(sat_mul_u64(sat_mul_u64(sat_mul_u64(w, h), pages), bpp))
            };

            let mask_atlas = rt_atlas("mask_atlas");
            let color_atlas = rt_atlas("color_atlas");
            let subpixel_atlas = rt_atlas("subpixel_atlas");

            let mask_bytes = atlas_bytes(mask_atlas, 1);
            let color_bytes = atlas_bytes(color_atlas, 4);
            let subpixel_bytes = atlas_bytes(subpixel_atlas, 4);
            let total_bytes = match (mask_bytes, color_bytes, subpixel_bytes) {
                (Some(a), Some(b), Some(c)) => Some(a.saturating_add(b).saturating_add(c)),
                _ => None,
            };

            (
                tick_id,
                frame_id,
                Some(render_text.is_some()),
                mask_bytes,
                color_bytes,
                subpixel_bytes,
                total_bytes,
            )
        } else {
            (None, None, None, None, None, None, None)
        };

    let missing_reason = if bundle_present {
        "missing_field"
    } else {
        "missing_bundle"
    };

    let mut failures: Vec<serde_json::Value> = Vec::new();
    match (present_flag, total_bytes) {
        (Some(true), Some(observed)) if observed > max_render_text_atlas_bytes_live_estimate_total => {
            failures.push(serde_json::json!({
                "kind": "render_text_atlas_bytes_live_estimate_total",
                "threshold": max_render_text_atlas_bytes_live_estimate_total,
                "observed": observed,
                "reason": "exceeded",
            }));
        }
        (Some(true), Some(_)) => {}
        (Some(true), None) => failures.push(serde_json::json!({
            "kind": "render_text_atlas_bytes_live_estimate_total",
            "threshold": max_render_text_atlas_bytes_live_estimate_total,
            "observed": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].resource_caches.render_text.*_atlas.{width,height,pages}",
        })),
        (Some(false), _) | (None, _) => failures.push(serde_json::json!({
            "kind": "render_text_atlas_bytes_live_estimate_total",
            "threshold": max_render_text_atlas_bytes_live_estimate_total,
            "observed": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].resource_caches.render_text",
        })),
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "render_text_atlas_bytes_threshold",
        "out_dir": out_dir.display().to_string(),
        "bundle_file": bundle_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>"),
        "thresholds": {
            "max_render_text_atlas_bytes_live_estimate_total": max_render_text_atlas_bytes_live_estimate_total,
        },
        "observed": {
            "bundle_present": bundle_present,
            "tick_id": tick_id,
            "frame_id": frame_id,
            "render_text_present": present_flag,
            "render_text_mask_atlas_bytes_live_estimate": mask_bytes,
            "render_text_color_atlas_bytes_live_estimate": color_bytes,
            "render_text_subpixel_atlas_bytes_live_estimate": subpixel_bytes,
            "render_text_atlas_bytes_live_estimate_total": total_bytes,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(RenderTextAtlasBytesGateResult {
        evidence_path: out_path,
        failures,
    })
}

pub(super) fn check_resource_footprint_thresholds(
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

    let macos_physical_footprint_peak_bytes = v
        .as_ref()
        .and_then(|v| v.get("macos_vmmap"))
        .and_then(|v| v.get("physical_footprint_peak_bytes"))
        .and_then(|v| v.as_u64());
    let macos_owned_unmapped_memory_dirty_bytes = v
        .as_ref()
        .and_then(|v| v.get("macos_vmmap"))
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("owned_unmapped_memory_dirty_bytes"))
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

    if let Some(thr) = thresholds.max_macos_physical_footprint_peak_bytes {
        match macos_physical_footprint_peak_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_physical_footprint_peak_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_physical_footprint_peak_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "macos_vmmap.physical_footprint_peak_bytes",
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_owned_unmapped_memory_dirty_bytes {
        match macos_owned_unmapped_memory_dirty_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_owned_unmapped_memory_dirty_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_owned_unmapped_memory_dirty_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "macos_vmmap.regions.owned_unmapped_memory_dirty_bytes",
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
            "max_macos_physical_footprint_peak_bytes": thresholds.max_macos_physical_footprint_peak_bytes,
            "max_macos_owned_unmapped_memory_dirty_bytes": thresholds.max_macos_owned_unmapped_memory_dirty_bytes,
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
            "macos_physical_footprint_peak_bytes": macos_physical_footprint_peak_bytes,
            "macos_owned_unmapped_memory_dirty_bytes": macos_owned_unmapped_memory_dirty_bytes,
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
pub(super) struct RedrawHitchesGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
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

pub(super) fn check_redraw_hitches_max_total_ms(
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
