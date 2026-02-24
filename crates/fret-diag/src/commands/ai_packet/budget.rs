use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::{AiPacketBudgetConfig, AiPacketBudgetReport};

pub(super) fn write_packet_budget_report(
    dir: &Path,
    report: &AiPacketBudgetReport,
) -> Result<(), String> {
    let mut files_present: Vec<(String, u64)> = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let Some(name) = p
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
        else {
            continue;
        };
        let bytes = file_bytes(&p)?;
        files_present.push((name, bytes));
    }
    files_present.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut payload = serde_json::json!({
        "kind": report.kind,
        "schema_version": report.schema_version,
        "reason_code": report.reason_code,
        "budget": {
            "soft_total_bytes": report.budget.soft_total_bytes,
            "hard_total_bytes": report.budget.hard_total_bytes,
            "max_bundle_meta_bytes": report.budget.max_bundle_meta_bytes,
            "max_bundle_schema2_bytes": report.budget.max_bundle_schema2_bytes,
            "max_bundle_index_bytes": report.budget.max_bundle_index_bytes,
            "max_test_ids_index_bytes": report.budget.max_test_ids_index_bytes,
            "max_frames_index_bytes": report.budget.max_frames_index_bytes,
            "max_slice_bytes": report.budget.max_slice_bytes,
        },
        "bytes_total": report.bytes_total,
        "files_present": files_present.iter().map(|(name, bytes)| serde_json::json!({
            "name": name,
            "bytes": bytes,
        })).collect::<Vec<_>>(),
        "soft_budget_exceeded": report.soft_budget_exceeded,
        "hard_budget_exceeded": report.hard_budget_exceeded,
        "dropped_files": report.dropped_files,
        "clipped_files": report.clipped_files,
    });

    if let Some(v) = report.failed_step_slices.as_ref() {
        if let Some(obj) = payload.as_object_mut() {
            let written = v
                .written
                .iter()
                .map(|w| {
                    serde_json::json!({
                        "file": &w.file,
                        "test_id": &w.test_id,
                        "matches": w.matches,
                    })
                })
                .collect::<Vec<_>>();
            obj.insert(
                "failed_step_slices".to_string(),
                serde_json::json!({
                    "schema_version": v.schema_version,
                    "status": &v.status,
                    "reason_code": &v.reason_code,
                    "failed_step_index": v.failed_step_index,
                    "failed_snapshot": {
                        "window": v.window,
                        "frame_id": v.frame_id,
                        "window_snapshot_seq": v.window_snapshot_seq,
                    },
                    "candidate_test_ids": &v.candidate_test_ids,
                    "attempted_test_ids": &v.attempted_test_ids,
                    "written": written,
                }),
            );
        }
    }

    let bytes = serde_json::to_vec_pretty(&payload).unwrap_or_else(|_| b"{}".to_vec());
    std::fs::write(dir.join("ai.packet.json"), bytes).map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) fn enforce_ai_packet_budgets(
    dir: &Path,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let cfg = report.budget.clone();

    // Always enforce per-file caps first.
    clip_bundle_meta_if_needed(dir, &cfg, report)?;
    drop_bundle_schema2_if_needed(dir, &cfg, report)?;
    clip_bundle_index_if_needed(dir, &cfg, report)?;
    clip_test_ids_index_if_needed(dir, &cfg, report)?;
    clip_frames_index_if_needed(dir, &cfg, report)?;

    // Optional files: if we are above the hard total budget, drop these first.
    let mut total = packet_total_bytes(dir)?;
    if total > cfg.hard_total_bytes {
        drop_if_present(dir, "triage.json", report)?;
        drop_if_present(dir, "hotspots.lite.json", report)?;
        drop_if_present(dir, "triage.lite.json", report)?;
        drop_if_present(dir, "manifest.json", report)?;
        drop_if_present(dir, "bundle.schema2.json", report)?;
        total = packet_total_bytes(dir)?;
    }

    report.bytes_total = total;
    report.soft_budget_exceeded = total > cfg.soft_total_bytes;
    report.hard_budget_exceeded = total > cfg.hard_total_bytes;

    if report.hard_budget_exceeded {
        report.reason_code = Some("tooling.ai_packet.budget.hard_exceeded".to_string());
        return Err(format!(
            "ai packet exceeds hard budget (total_bytes={} > {})",
            total, cfg.hard_total_bytes
        ));
    }

    Ok(())
}

fn file_bytes(path: &Path) -> Result<u64, String> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())
}

fn packet_total_bytes(dir: &Path) -> Result<u64, String> {
    let mut total: u64 = 0;
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if p.is_file() {
            total = total.saturating_add(file_bytes(&p)?);
        }
    }
    Ok(total)
}

fn drop_if_present(
    dir: &Path,
    name: &str,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join(name);
    if path.is_file() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        report.dropped_files.push(name.to_string());
    }
    Ok(())
}

pub(super) fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

pub(super) fn write_json_compact(path: &Path, v: &serde_json::Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(v).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    Ok(())
}

fn clip_bundle_meta_if_needed(
    dir: &Path,
    cfg: &AiPacketBudgetConfig,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join("bundle.meta.json");
    if !path.is_file() {
        return Ok(());
    }
    let bytes = file_bytes(&path)?;
    if bytes <= cfg.max_bundle_meta_bytes {
        return Ok(());
    }
    let v = read_json(&path)?;
    write_json_compact(&path, &v)?;
    let new_bytes = file_bytes(&path)?;
    if new_bytes > cfg.max_bundle_meta_bytes {
        report.reason_code = Some("tooling.ai_packet.budget.bundle_meta_exceeded".to_string());
        return Err(format!(
            "bundle.meta.json exceeds budget (bytes={} > max={})",
            new_bytes, cfg.max_bundle_meta_bytes
        ));
    }
    report.clipped_files.push("bundle.meta.json".to_string());
    Ok(())
}

fn drop_bundle_schema2_if_needed(
    dir: &Path,
    cfg: &AiPacketBudgetConfig,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join("bundle.schema2.json");
    if !path.is_file() {
        return Ok(());
    }

    let bytes = file_bytes(&path)?;
    if bytes <= cfg.max_bundle_schema2_bytes {
        return Ok(());
    }

    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    report
        .dropped_files
        .push("bundle.schema2.json (exceeds max_bundle_schema2_bytes)".to_string());
    Ok(())
}

fn clip_bundle_index_if_needed(
    dir: &Path,
    cfg: &AiPacketBudgetConfig,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join("bundle.index.json");
    if !path.is_file() {
        return Ok(());
    }
    let bytes = file_bytes(&path)?;
    if bytes <= cfg.max_bundle_index_bytes {
        return Ok(());
    }

    let mut v = read_json(&path)?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("bundle_index") {
        return Ok(());
    }

    let mut required_by_window: HashMap<u64, (HashSet<u64>, HashSet<u64>)> = HashMap::new();
    if let Some(steps) = v
        .get("script")
        .and_then(|v| v.get("steps"))
        .and_then(|v| v.as_array())
    {
        for step in steps {
            let Some(window) = step.get("window").and_then(|v| v.as_u64()) else {
                continue;
            };
            let entry = required_by_window.entry(window).or_default();
            if let Some(seq) = step.get("window_snapshot_seq").and_then(|v| v.as_u64()) {
                entry.0.insert(seq);
            }
            if let Some(frame_id) = step.get("frame_id").and_then(|v| v.as_u64()) {
                entry.1.insert(frame_id);
            }
        }
    }

    let mut max_keep: usize = v
        .get("windows")
        .and_then(|v| v.as_array())
        .map(|windows| {
            windows
                .iter()
                .filter_map(|w| {
                    w.get("snapshots")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                })
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    max_keep = max_keep.min(4096).max(8);

    loop {
        let snapshots_total: u64 = {
            let windows = v
                .get_mut("windows")
                .and_then(|v| v.as_array_mut())
                .ok_or_else(|| "invalid bundle.index.json: missing windows".to_string())?;

            for w in windows.iter_mut() {
                let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
                if let Some(snaps) = w.get_mut("snapshots").and_then(|v| v.as_array_mut()) {
                    if snaps.len() > max_keep {
                        let mut required_seq: Option<&HashSet<u64>> = None;
                        let mut required_frame: Option<&HashSet<u64>> = None;
                        if let Some((seq, frame)) = required_by_window.get(&window_id) {
                            required_seq = Some(seq);
                            required_frame = Some(frame);
                        }

                        let mut old: Vec<serde_json::Value> = Vec::new();
                        std::mem::swap(snaps, &mut old);

                        let len = old.len();
                        let start = len.saturating_sub(max_keep);
                        let mut keep: Vec<bool> = vec![false; len];
                        for i in start..len {
                            keep[i] = true;
                        }

                        if required_seq.is_some() || required_frame.is_some() {
                            for (i, s) in old.iter().enumerate() {
                                let seq = s.get("window_snapshot_seq").and_then(|v| v.as_u64());
                                let frame_id = s.get("frame_id").and_then(|v| v.as_u64());
                                let required = seq.is_some_and(|v| {
                                    required_seq.is_some_and(|set| set.contains(&v))
                                }) || frame_id.is_some_and(|v| {
                                    required_frame.is_some_and(|set| set.contains(&v))
                                });
                                if required {
                                    keep[i] = true;
                                }
                            }
                        }

                        *snaps = old
                            .into_iter()
                            .enumerate()
                            .filter_map(|(i, v)| keep.get(i).copied().unwrap_or(false).then_some(v))
                            .collect();
                    }
                }

                let (first_frame_id, first_ts, last_frame_id, last_ts) = w
                    .get("snapshots")
                    .and_then(|v| v.as_array())
                    .map(|snaps| {
                        let first = snaps.first();
                        let last = snaps.last();
                        (
                            first.and_then(|v| v.get("frame_id")).cloned(),
                            first.and_then(|v| v.get("timestamp_unix_ms")).cloned(),
                            last.and_then(|v| v.get("frame_id")).cloned(),
                            last.and_then(|v| v.get("timestamp_unix_ms")).cloned(),
                        )
                    })
                    .unwrap_or((None, None, None, None));

                if let Some(obj) = w.as_object_mut() {
                    if let Some(v) = first_frame_id {
                        obj.insert("first_frame_id".to_string(), v);
                    }
                    if let Some(v) = first_ts {
                        obj.insert("first_timestamp_unix_ms".to_string(), v);
                    }
                    if let Some(v) = last_frame_id {
                        obj.insert("last_frame_id".to_string(), v);
                    }
                    if let Some(v) = last_ts {
                        obj.insert("last_timestamp_unix_ms".to_string(), v);
                    }
                }
            }

            windows
                .iter()
                .filter_map(|w| {
                    w.get("snapshots")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len() as u64)
                })
                .sum()
        };
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "snapshots_total".to_string(),
                serde_json::Value::from(snapshots_total),
            );
            obj.insert(
                "clipped".to_string(),
                serde_json::json!({
                    "schema_version": 1,
                    "max_snapshots_per_window": max_keep,
                    "reason": "bundle_index_exceeds_budget",
                }),
            );
        }

        write_json_compact(&path, &v)?;

        let new_bytes = file_bytes(&path)?;
        if new_bytes <= cfg.max_bundle_index_bytes {
            report.clipped_files.push("bundle.index.json".to_string());
            return Ok(());
        }

        if max_keep <= 8 {
            report.reason_code = Some("tooling.ai_packet.budget.bundle_index_exceeded".to_string());
            return Err(format!(
                "bundle.index.json exceeds budget even after clipping (bytes={} > max={})",
                new_bytes, cfg.max_bundle_index_bytes
            ));
        }

        max_keep = (max_keep * 2 / 3).max(8);
    }
}

fn clip_test_ids_index_if_needed(
    dir: &Path,
    cfg: &AiPacketBudgetConfig,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join("test_ids.index.json");
    if !path.is_file() {
        return Ok(());
    }
    let bytes = file_bytes(&path)?;
    if bytes <= cfg.max_test_ids_index_bytes {
        return Ok(());
    }

    let mut v = read_json(&path)?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("test_ids_index") {
        return Ok(());
    }

    let mut max_keep: usize = v
        .get("windows")
        .and_then(|v| v.as_array())
        .map(|windows| {
            windows
                .iter()
                .filter_map(|w| w.get("items").and_then(|v| v.as_array()).map(|a| a.len()))
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    max_keep = max_keep.min(20000).max(64);

    loop {
        let unique_total: u64 = {
            let windows = v
                .get_mut("windows")
                .and_then(|v| v.as_array_mut())
                .ok_or_else(|| "invalid test_ids.index.json: missing windows".to_string())?;

            for w in windows.iter_mut() {
                let Some(items) = w.get_mut("items").and_then(|v| v.as_array_mut()) else {
                    continue;
                };
                if items.len() > max_keep {
                    items.truncate(max_keep);
                }

                let unique = items.len() as u64;
                let count_sum: u64 = items
                    .iter()
                    .filter_map(|it| it.get("count").and_then(|v| v.as_u64()))
                    .sum();
                if let Some(obj) = w.as_object_mut() {
                    obj.insert(
                        "unique_test_ids_total".to_string(),
                        serde_json::Value::from(unique),
                    );
                    obj.insert(
                        "test_id_nodes_total".to_string(),
                        serde_json::Value::from(count_sum),
                    );
                }
            }

            windows
                .iter()
                .filter_map(|w| w.get("unique_test_ids_total").and_then(|v| v.as_u64()))
                .sum()
        };
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "total_unique_test_ids".to_string(),
                serde_json::Value::from(unique_total),
            );
            obj.insert("truncated".to_string(), serde_json::Value::from(true));
            obj.insert(
                "clipped".to_string(),
                serde_json::json!({
                    "schema_version": 1,
                    "max_items_per_window": max_keep,
                    "reason": "test_ids_index_exceeds_budget",
                }),
            );
        }

        write_json_compact(&path, &v)?;

        let new_bytes = file_bytes(&path)?;
        if new_bytes <= cfg.max_test_ids_index_bytes {
            report.clipped_files.push("test_ids.index.json".to_string());
            return Ok(());
        }

        if max_keep <= 64 {
            report.reason_code =
                Some("tooling.ai_packet.budget.test_ids_index_exceeded".to_string());
            return Err(format!(
                "test_ids.index.json exceeds budget even after clipping (bytes={} > max={})",
                new_bytes, cfg.max_test_ids_index_bytes
            ));
        }

        max_keep = (max_keep * 2 / 3).max(64);
    }
}

fn clip_frames_index_if_needed(
    dir: &Path,
    cfg: &AiPacketBudgetConfig,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join("frames.index.json");
    if !path.is_file() {
        return Ok(());
    }
    let bytes = file_bytes(&path)?;
    if bytes <= cfg.max_frames_index_bytes {
        return Ok(());
    }

    let mut v = read_json(&path)?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("frames_index") {
        return Ok(());
    }

    let mut max_keep: usize = v
        .get("windows")
        .and_then(|v| v.as_array())
        .map(|windows| {
            windows
                .iter()
                .filter_map(|w| w.get("rows").and_then(|v| v.as_array()).map(|a| a.len()))
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    max_keep = max_keep.min(8192).max(64);

    loop {
        let frames_total: u64 = {
            let windows = v
                .get_mut("windows")
                .and_then(|v| v.as_array_mut())
                .ok_or_else(|| "invalid frames.index.json: missing windows".to_string())?;

            for w in windows.iter_mut() {
                let rows_len = {
                    let Some(rows) = w.get_mut("rows").and_then(|v| v.as_array_mut()) else {
                        continue;
                    };
                    if rows.len() > max_keep {
                        let len = rows.len();
                        rows.drain(0..(len - max_keep));
                    }
                    rows.len() as u64
                };
                if let Some(obj) = w.as_object_mut() {
                    obj.insert(
                        "frames_total".to_string(),
                        serde_json::Value::from(rows_len),
                    );
                }
            }

            windows
                .iter()
                .filter_map(|w| {
                    w.get("rows")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len() as u64)
                })
                .sum()
        };

        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "frames_total".to_string(),
                serde_json::Value::from(frames_total),
            );
            obj.insert(
                "clipped".to_string(),
                serde_json::json!({
                    "schema_version": 1,
                    "max_rows_per_window": max_keep,
                    "reason": "frames_index_exceeds_budget",
                }),
            );
        }

        write_json_compact(&path, &v)?;

        let new_bytes = file_bytes(&path)?;
        if new_bytes <= cfg.max_frames_index_bytes {
            report.clipped_files.push("frames.index.json".to_string());
            return Ok(());
        }

        if max_keep <= 32 {
            // frames.index.json is optional: drop it instead of failing the whole packet.
            drop_if_present(dir, "frames.index.json", report)?;
            return Ok(());
        }

        max_keep = (max_keep * 2 / 3).max(32);
    }
}
