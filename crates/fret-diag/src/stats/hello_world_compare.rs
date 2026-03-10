use std::path::Path;

use crate::util::{now_unix_ms, write_json_value};

const INTERNAL_REPORT_BASENAME: &str = "hello_world_compare.internal_gpu.json";
const EVIDENCE_BASENAME: &str = "check.hello_world_compare_idle_present.json";
const COMMAND_PALETTE_SERVICE: &str = "fret_bootstrap::ui_app_driver::CommandPaletteService";

#[derive(Debug, Clone)]
struct IdlePresentSample {
    offset_secs: f64,
    runner_present_total: u64,
    runner_frame_drive_total: u64,
    redraw_request_total: u64,
    global_change_batch_count: u64,
    top_global_name: Option<String>,
    command_palette_global_count: u64,
}

pub(crate) fn check_out_dir_for_hello_world_compare_idle_present_max_delta(
    out_dir: &Path,
    max_delta: u64,
) -> Result<(), String> {
    let report_path = out_dir.join(INTERNAL_REPORT_BASENAME);
    if !report_path.is_file() {
        return Err(format!(
            "hello_world_compare idle-present gate requires an internal report at {} (set FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH or FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_TO_DIAG_DIR=1)",
            report_path.display()
        ));
    }

    let bytes = std::fs::read(&report_path).map_err(|err| err.to_string())?;
    let root: serde_json::Value = serde_json::from_slice(&bytes).map_err(|err| err.to_string())?;
    let samples = parse_idle_present_samples(&root)?;

    let first_present_count = samples.first().map(|sample| sample.runner_present_total);
    let last_present_count = samples.last().map(|sample| sample.runner_present_total);
    let present_delta = match (first_present_count, last_present_count) {
        (Some(first), Some(last)) => last.saturating_sub(first),
        _ => 0,
    };
    let command_palette_global_count_total = samples
        .iter()
        .map(|sample| sample.command_palette_global_count)
        .sum::<u64>();

    let evidence_path = out_dir.join(EVIDENCE_BASENAME);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "hello_world_compare_idle_present",
        "out_dir": out_dir.display().to_string(),
        "internal_report": report_path.display().to_string(),
        "max_delta": max_delta,
        "requested_runtime": root.get("requested_runtime").cloned().unwrap_or(serde_json::Value::Null),
        "samples": samples.iter().map(|sample| serde_json::json!({
            "offset_secs": sample.offset_secs,
            "runner_present_total": sample.runner_present_total,
            "runner_frame_drive_total": sample.runner_frame_drive_total,
            "redraw_request_total": sample.redraw_request_total,
            "global_change_batch_count": sample.global_change_batch_count,
            "top_global_name": sample.top_global_name,
            "command_palette_global_count": sample.command_palette_global_count,
        })).collect::<Vec<_>>(),
        "summary": {
            "sample_count": samples.len(),
            "first_present_count": first_present_count,
            "last_present_count": last_present_count,
            "present_delta": present_delta,
            "max_present_count": samples.iter().map(|sample| sample.runner_present_total).max().unwrap_or(0),
            "first_offset_secs": samples.first().map(|sample| sample.offset_secs),
            "last_offset_secs": samples.last().map(|sample| sample.offset_secs),
            "command_palette_global_count_total": command_palette_global_count_total,
        },
        "pass": samples.len() >= 2 && present_delta <= max_delta,
    });
    let _ = write_json_value(&evidence_path, &payload);

    if samples.len() < 2 {
        return Err(format!(
            "hello_world_compare idle-present gate requires at least 2 samples (got {})\n  internal_report: {}\n  evidence: {}",
            samples.len(),
            report_path.display(),
            evidence_path.display()
        ));
    }

    if present_delta <= max_delta {
        return Ok(());
    }

    Err(format!(
        "hello_world_compare idle-present gate failed (present_delta={} > max_delta={})\n  internal_report: {}\n  evidence: {}",
        present_delta,
        max_delta,
        report_path.display(),
        evidence_path.display()
    ))
}

fn parse_idle_present_samples(root: &serde_json::Value) -> Result<Vec<IdlePresentSample>, String> {
    let raw_samples = root
        .get("samples")
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            "invalid hello_world_compare internal report: missing samples array".to_string()
        })?;

    let mut samples = Vec::with_capacity(raw_samples.len());
    for entry in raw_samples {
        let runtime = entry
            .get("runtime")
            .and_then(|value| value.as_object())
            .ok_or_else(|| {
                "invalid hello_world_compare internal report: sample missing runtime".to_string()
            })?;
        let global_change_globals = runtime
            .get("global_changes")
            .and_then(|value| value.get("globals"))
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        let command_palette_global_count = global_change_globals
            .iter()
            .filter_map(|global| {
                let name = global.get("name").and_then(|value| value.as_str())?;
                (name == COMMAND_PALETTE_SERVICE).then(|| {
                    global
                        .get("count")
                        .and_then(|value| value.as_u64())
                        .unwrap_or(0)
                })
            })
            .sum::<u64>();
        let top_global_name = global_change_globals
            .first()
            .and_then(|value| value.get("name"))
            .and_then(|value| value.as_str())
            .map(str::to_string);

        samples.push(IdlePresentSample {
            offset_secs: entry
                .get("offset_secs")
                .and_then(|value| value.as_f64())
                .unwrap_or(0.0),
            runner_present_total: runtime
                .get("runner_present")
                .and_then(|value| value.get("total_present_count"))
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
            runner_frame_drive_total: runtime
                .get("runner_frame_drive")
                .and_then(|value| value.get("total_event_count"))
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
            redraw_request_total: runtime
                .get("redraw_requests")
                .and_then(|value| value.get("total_request_count"))
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
            global_change_batch_count: runtime
                .get("global_changes")
                .and_then(|value| value.get("batch_count"))
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
            top_global_name,
            command_palette_global_count,
        });
    }

    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_tmp_dir(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "fret_diag_hello_world_compare_{label}_{}_{}",
            std::process::id(),
            now_unix_ms()
        ));
        std::fs::create_dir_all(&path).expect("create tmp dir");
        path
    }

    fn write_report(out_dir: &Path, samples: &[serde_json::Value]) {
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "hello_world_compare_internal_gpu_timeline",
            "samples": samples,
        });
        std::fs::write(
            out_dir.join(INTERNAL_REPORT_BASENAME),
            serde_json::to_vec_pretty(&payload).expect("serialize report"),
        )
        .expect("write report");
    }

    #[test]
    fn idle_present_gate_passes_when_present_count_is_flat() {
        let out_dir = unique_tmp_dir("pass");
        write_report(
            &out_dir,
            &[
                serde_json::json!({
                    "offset_secs": 2.0,
                    "runtime": {
                        "runner_present": { "total_present_count": 5 },
                        "runner_frame_drive": { "total_event_count": 10 },
                        "redraw_requests": { "total_request_count": 0 },
                        "global_changes": {
                            "batch_count": 3,
                            "globals": [{ "name": "fret_runtime::font_catalog::TextFontStackKey", "count": 2 }]
                        }
                    }
                }),
                serde_json::json!({
                    "offset_secs": 3.0,
                    "runtime": {
                        "runner_present": { "total_present_count": 5 },
                        "runner_frame_drive": { "total_event_count": 10 },
                        "redraw_requests": { "total_request_count": 0 },
                        "global_changes": { "batch_count": 3, "globals": [] }
                    }
                }),
            ],
        );

        check_out_dir_for_hello_world_compare_idle_present_max_delta(&out_dir, 0).unwrap();
        assert!(out_dir.join(EVIDENCE_BASENAME).is_file());
    }

    #[test]
    fn idle_present_gate_fails_when_present_count_keeps_growing() {
        let out_dir = unique_tmp_dir("fail");
        write_report(
            &out_dir,
            &[
                serde_json::json!({
                    "offset_secs": 2.0,
                    "runtime": {
                        "runner_present": { "total_present_count": 98 },
                        "runner_frame_drive": { "total_event_count": 100 },
                        "redraw_requests": { "total_request_count": 97 },
                        "global_changes": {
                            "batch_count": 98,
                            "globals": [{ "name": COMMAND_PALETTE_SERVICE, "count": 97 }]
                        }
                    }
                }),
                serde_json::json!({
                    "offset_secs": 4.0,
                    "runtime": {
                        "runner_present": { "total_present_count": 335 },
                        "runner_frame_drive": { "total_event_count": 337 },
                        "redraw_requests": { "total_request_count": 334 },
                        "global_changes": {
                            "batch_count": 335,
                            "globals": [{ "name": COMMAND_PALETTE_SERVICE, "count": 334 }]
                        }
                    }
                }),
            ],
        );

        let err =
            check_out_dir_for_hello_world_compare_idle_present_max_delta(&out_dir, 1).unwrap_err();
        assert!(err.contains("present_delta=237 > max_delta=1"), "{err}");
        assert!(out_dir.join(EVIDENCE_BASENAME).is_file());
    }
}
