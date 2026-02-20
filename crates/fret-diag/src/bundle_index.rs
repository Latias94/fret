use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::json_bundle::{
    pick_last_snapshot_with_semantics_after_warmup, snapshot_frame_id, snapshot_semantics_nodes,
};

fn read_json_value(path: &Path) -> Option<Value> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn write_pretty_json(path: &Path, payload: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(payload).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

pub(crate) fn default_bundle_meta_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("bundle.meta.json")
}

pub(crate) fn default_test_ids_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("test_ids.json")
}

pub(crate) fn default_test_ids_index_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("test_ids.index.json")
}

pub(crate) fn ensure_bundle_meta_json(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<PathBuf, String> {
    let out = default_bundle_meta_path(bundle_path);
    if out.is_file() && read_json_value(&out).is_some() {
        return Ok(out);
    }
    let payload = build_bundle_meta_payload(bundle_path, warmup_frames)?;
    write_pretty_json(&out, &payload)?;
    Ok(out)
}

pub(crate) fn ensure_test_ids_json(
    bundle_path: &Path,
    warmup_frames: u64,
    max_test_ids: usize,
) -> Result<PathBuf, String> {
    let out = default_test_ids_path(bundle_path);
    if out.is_file() && read_json_value(&out).is_some() {
        return Ok(out);
    }
    let payload = build_test_ids_payload(bundle_path, warmup_frames, max_test_ids)?;
    write_pretty_json(&out, &payload)?;
    Ok(out)
}

pub(crate) fn ensure_test_ids_index_json(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<PathBuf, String> {
    let out = default_test_ids_index_path(bundle_path);
    if out.is_file() && read_json_value(&out).is_some() {
        return Ok(out);
    }
    let payload = build_test_ids_index_payload(bundle_path, warmup_frames)?;
    write_pretty_json(&out, &payload)?;
    Ok(out)
}

fn build_bundle_meta_payload(bundle_path: &Path, warmup_frames: u64) -> Result<Value, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut out_windows: Vec<Value> = Vec::new();
    let mut total_snapshots: u64 = 0;
    let mut total_unique_test_ids: HashSet<String> = HashSet::new();
    let mut total_snapshots_with_semantics: u64 = 0;
    let mut total_unique_semantics_fingerprints: HashSet<u64> = HashSet::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        total_snapshots = total_snapshots.saturating_add(snaps.len() as u64);

        let mut window_snapshots_with_semantics: u64 = 0;
        let mut window_unique_semantics_fingerprints: HashSet<u64> = HashSet::new();
        for s in snaps {
            if snapshot_semantics_nodes(s).is_some() {
                window_snapshots_with_semantics = window_snapshots_with_semantics.saturating_add(1);
                total_snapshots_with_semantics = total_snapshots_with_semantics.saturating_add(1);
            }
            if let Some(fp) = s.get("semantics_fingerprint").and_then(|v| v.as_u64()) {
                window_unique_semantics_fingerprints.insert(fp);
                total_unique_semantics_fingerprints.insert(fp);
            }
        }

        let Some(snapshot) = pick_last_snapshot_with_semantics_after_warmup(snaps, warmup_frames)
        else {
            out_windows.push(json!({
                "window": window_id,
                "snapshots_total": snaps.len(),
                "snapshots_with_semantics_total": window_snapshots_with_semantics,
                "unique_semantics_fingerprints_total": window_unique_semantics_fingerprints.len(),
                "considered_frame_id": null,
                "considered_timestamp_unix_ms": null,
                "semantics_nodes_total": 0,
                "test_id_nodes_total": 0,
                "unique_test_ids_total": 0,
                "duplicate_test_ids_total": 0,
            }));
            continue;
        };

        let frame_id = snapshot
            .get("frame_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let ts = snapshot
            .get("timestamp_unix_ms")
            .and_then(|v| v.as_u64())
            .or_else(|| snapshot.get("timestamp_ms").and_then(|v| v.as_u64()));

        let nodes = snapshot_semantics_nodes(snapshot).unwrap_or(&[]);

        let mut counts: HashMap<&str, u64> = HashMap::new();
        for n in nodes {
            let Some(raw) = n.get("test_id").and_then(|v| v.as_str()) else {
                continue;
            };
            let test_id = raw.trim();
            if test_id.is_empty() {
                continue;
            }
            *counts.entry(test_id).or_insert(0) += 1;
        }

        for k in counts.keys() {
            total_unique_test_ids.insert((*k).to_string());
        }

        let semantics_nodes_total = nodes.len() as u64;
        let test_id_nodes_total: u64 = counts.values().sum();
        let unique_test_ids_total = counts.len() as u64;
        let duplicate_test_ids_total = counts.values().filter(|v| **v > 1).count() as u64;

        out_windows.push(json!({
            "window": window_id,
            "snapshots_total": snaps.len(),
            "snapshots_with_semantics_total": window_snapshots_with_semantics,
            "unique_semantics_fingerprints_total": window_unique_semantics_fingerprints.len(),
            "considered_frame_id": frame_id,
            "considered_timestamp_unix_ms": ts,
            "semantics_nodes_total": semantics_nodes_total,
            "test_id_nodes_total": test_id_nodes_total,
            "unique_test_ids_total": unique_test_ids_total,
            "duplicate_test_ids_total": duplicate_test_ids_total,
        }));
    }

    Ok(json!({
        "schema_version": 1,
        "kind": "bundle_meta",
        "bundle": bundle_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "windows_total": windows.len(),
        "snapshots_total": total_snapshots,
        "total_unique_test_ids": total_unique_test_ids.len(),
        "snapshots_with_semantics_total": total_snapshots_with_semantics,
        "unique_semantics_fingerprints_total": total_unique_semantics_fingerprints.len(),
        "windows": out_windows,
    }))
}

fn build_test_ids_payload(
    bundle_path: &Path,
    warmup_frames: u64,
    max_test_ids: usize,
) -> Result<Value, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut total_unique: HashSet<String> = HashSet::new();
    let mut windows_out: Vec<Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        let Some(snapshot) = pick_last_snapshot_with_semantics_after_warmup(snaps, warmup_frames)
        else {
            continue;
        };

        let frame_id = snapshot
            .get("frame_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let nodes = snapshot_semantics_nodes(snapshot).unwrap_or(&[]);

        let mut counts: HashMap<String, u64> = HashMap::new();
        for n in nodes {
            let Some(raw) = n.get("test_id").and_then(|v| v.as_str()) else {
                continue;
            };
            let test_id = raw.trim();
            if test_id.is_empty() {
                continue;
            }
            *counts.entry(test_id.to_string()).or_insert(0) += 1;
        }

        for k in counts.keys() {
            total_unique.insert(k.clone());
        }

        let semantics_nodes_total = nodes.len() as u64;
        let test_id_nodes_total: u64 = counts.values().sum();
        let unique_test_ids_total = counts.len() as u64;

        let mut items: Vec<(String, u64)> = counts.into_iter().collect();
        items.sort_by(|(a_id, a_count), (b_id, b_count)| {
            b_count.cmp(a_count).then_with(|| a_id.cmp(b_id))
        });

        let duplicates_total = items.iter().filter(|(_id, count)| *count > 1).count() as u64;
        let duplicates_top: Vec<Value> = items
            .iter()
            .filter(|(_id, count)| *count > 1)
            .take(50)
            .map(|(test_id, count)| json!({ "test_id": test_id, "count": count }))
            .collect();

        let mut truncated = false;
        if max_test_ids > 0 && items.len() > max_test_ids {
            items.truncate(max_test_ids);
            truncated = true;
        }

        let unique_test_ids_emitted = items.len() as u64;
        windows_out.push(json!({
            "window": window_id,
            "frame_id": frame_id,
            "semantics_nodes_total": semantics_nodes_total,
            "test_id_nodes_total": test_id_nodes_total,
            "unique_test_ids_total": unique_test_ids_total,
            "unique_test_ids_emitted": unique_test_ids_emitted,
            "duplicate_test_ids_total": duplicates_total,
            "duplicate_test_ids_top": duplicates_top,
            "items": items.into_iter().map(|(test_id, count)| json!({
                "test_id": test_id,
                "count": count,
            })).collect::<Vec<_>>(),
            "truncated": truncated,
        }));
    }

    Ok(json!({
        "schema_version": 1,
        "kind": "test_ids",
        "bundle": bundle_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_test_ids": max_test_ids,
        "total_unique_test_ids": total_unique.len(),
        "windows": windows_out,
    }))
}

fn build_test_ids_index_payload(bundle_path: &Path, warmup_frames: u64) -> Result<Value, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    // Hard safety budget: if we exceed this, something is wrong with `test_id` usage (e.g. per-frame unique ids).
    const MAX_UNIQUE_TEST_IDS_BUDGET: usize = 50_000;

    let mut total_unique: HashSet<String> = HashSet::new();
    let mut windows_out: Vec<Value> = Vec::new();
    let mut truncated: bool = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        let Some(snapshot) = pick_last_snapshot_with_semantics_after_warmup(snaps, warmup_frames)
        else {
            continue;
        };

        let frame_id = snapshot_frame_id(snapshot);
        let nodes = snapshot_semantics_nodes(snapshot).unwrap_or(&[]);

        let mut counts: HashMap<String, u64> = HashMap::new();
        for n in nodes {
            let Some(raw) = n.get("test_id").and_then(|v| v.as_str()) else {
                continue;
            };
            let test_id = raw.trim();
            if test_id.is_empty() {
                continue;
            }
            if !counts.contains_key(test_id)
                && (total_unique.len() + counts.len()) >= MAX_UNIQUE_TEST_IDS_BUDGET
            {
                truncated = true;
                continue;
            }
            *counts.entry(test_id.to_string()).or_insert(0) += 1;
        }

        for k in counts.keys() {
            if total_unique.len() >= MAX_UNIQUE_TEST_IDS_BUDGET {
                truncated = true;
                break;
            }
            total_unique.insert(k.clone());
        }

        let semantics_nodes_total = nodes.len() as u64;
        let test_id_nodes_total: u64 = counts.values().sum();
        let unique_test_ids_total = counts.len() as u64;

        let mut items: Vec<(String, u64)> = counts.into_iter().collect();
        items.sort_by(|(a_id, a_count), (b_id, b_count)| {
            b_count.cmp(a_count).then_with(|| a_id.cmp(b_id))
        });

        let duplicates_total = items.iter().filter(|(_id, count)| *count > 1).count() as u64;
        let duplicates_top: Vec<Value> = items
            .iter()
            .filter(|(_id, count)| *count > 1)
            .take(100)
            .map(|(test_id, count)| json!({ "test_id": test_id, "count": count }))
            .collect();

        windows_out.push(json!({
            "window": window_id,
            "frame_id": frame_id,
            "semantics_nodes_total": semantics_nodes_total,
            "test_id_nodes_total": test_id_nodes_total,
            "unique_test_ids_total": unique_test_ids_total,
            "duplicate_test_ids_total": duplicates_total,
            "duplicate_test_ids_top": duplicates_top,
            "items": items.into_iter().map(|(test_id, count)| json!({
                "test_id": test_id,
                "count": count,
            })).collect::<Vec<_>>(),
        }));
    }

    Ok(json!({
        "schema_version": 1,
        "kind": "test_ids_index",
        "bundle": bundle_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_unique_test_ids_budget": MAX_UNIQUE_TEST_IDS_BUDGET,
        "truncated": truncated,
        "total_unique_test_ids": total_unique.len(),
        "windows": windows_out,
    }))
}
