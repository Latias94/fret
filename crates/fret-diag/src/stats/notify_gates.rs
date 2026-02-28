use std::path::Path;

use crate::stats::notify_gates_streaming;

pub(crate) fn check_bundle_for_notify_hotspot_file_max(
    bundle_path: &Path,
    file_filter: &str,
    max_count: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    notify_gates_streaming::check_bundle_for_notify_hotspot_file_max_streaming(
        bundle_path,
        file_filter,
        max_count,
        warmup_frames,
    )
}

#[cfg(test)]
use crate::util::{now_unix_ms, write_json_value};

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn check_bundle_for_notify_hotspot_file_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    file_filter: &str,
    max_count: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    fn file_matches(actual: &str, filter: &str) -> bool {
        if filter.is_empty() {
            return false;
        }
        if actual == filter {
            return true;
        }
        let actual_norm = actual.replace('\\', "/");
        let filter_norm = filter.replace('\\', "/");
        actual_norm.ends_with(&filter_norm) || actual_norm.contains(&filter_norm)
    }

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut total_notify_requests: u64 = 0;
    let mut matched_notify_requests: u64 = 0;
    let mut matched_samples: Vec<serde_json::Value> = Vec::new();
    let mut matched_hotspot_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let reqs = s
                .get("debug")
                .and_then(|v| v.get("notify_requests"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);

            for req in reqs {
                total_notify_requests = total_notify_requests.saturating_add(1);

                let file = req.get("file").and_then(|v| v.as_str()).unwrap_or_default();
                let line = req.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                let column = req.get("column").and_then(|v| v.as_u64()).unwrap_or(0);

                let key = format!("{file}:{line}:{column}");
                *matched_hotspot_counts.entry(key).or_insert(0) += 1;

                if file_matches(file, file_filter) {
                    matched_notify_requests = matched_notify_requests.saturating_add(1);
                    if matched_samples.len() < 20 {
                        matched_samples.push(serde_json::json!({
                            "window_id": window_id,
                            "frame_id": frame_id,
                            "caller_node": req.get("caller_node").and_then(|v| v.as_u64()),
                            "target_view": req.get("target_view").and_then(|v| v.as_u64()),
                            "file": file,
                            "line": line,
                            "column": column,
                        }));
                    }
                }
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.notify_hotspots.json");

    let mut top_hotspots: Vec<(String, u64)> = matched_hotspot_counts.into_iter().collect();
    top_hotspots.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top_hotspots: Vec<serde_json::Value> = top_hotspots
        .into_iter()
        .take(30)
        .map(|(key, count)| serde_json::json!({ "key": key, "count": count }))
        .collect();

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "notify_hotspots",
        "bundle_artifact": bundle_path.display().to_string(),
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "file_filter": file_filter,
        "max_count": max_count,
        "total_notify_requests": total_notify_requests,
        "matched_notify_requests": matched_notify_requests,
        "matched_samples": matched_samples,
        "top_hotspots": top_hotspots,
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_notify_requests > max_count {
        return Err(format!(
            "notify hotspot file budget exceeded: file_filter={file_filter} matched_notify_requests={matched_notify_requests} max_count={max_count}\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}
