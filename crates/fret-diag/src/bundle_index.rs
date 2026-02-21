use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::json_bundle::{
    SemanticsResolver, pick_last_snapshot_with_resolved_semantics_after_warmup, snapshot_frame_id,
    snapshot_semantics_fingerprint, snapshot_semantics_nodes, snapshot_window_snapshot_seq,
};
use crate::test_id_bloom::{
    TEST_ID_BLOOM_V1_K, TEST_ID_BLOOM_V1_M_BITS, TEST_ID_BLOOM_V1_SCHEMA_VERSION, TestIdBloomV1,
};

const TEST_ID_BLOOM_TAIL_SNAPSHOTS_PER_WINDOW: usize = 64;
const TEST_ID_BLOOM_MAX_SEMANTICS_KEYS_PER_WINDOW: usize = 512;

pub(crate) fn semantics_bloom_index_from_bundle_index_json(
    idx: &Value,
) -> HashMap<(u64, u64, u8), TestIdBloomV1> {
    let mut out: HashMap<(u64, u64, u8), TestIdBloomV1> = HashMap::new();
    let Some(blooms) = idx.get("semantics_blooms") else {
        return out;
    };
    let windows = blooms
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v.as_slice());

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let items = w
            .get("items")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v.as_slice());
        for it in items {
            let Some(fp) = it.get("semantics_fingerprint").and_then(|v| v.as_u64()) else {
                continue;
            };
            let source = it
                .get("semantics_source")
                .and_then(|v| v.as_str())
                .unwrap_or("none");
            let source_tag = match source {
                "inline" => 0u8,
                "table" => 1u8,
                _ => continue,
            };
            let Some(hex) = it.get("test_id_bloom_hex").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(bloom) = TestIdBloomV1::from_hex(hex) else {
                continue;
            };
            out.insert((window_id, fp, source_tag), bloom);
        }
    }
    out
}

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

pub(crate) fn default_bundle_index_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("bundle.index.json")
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

pub(crate) fn ensure_bundle_index_json(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<PathBuf, String> {
    let out = default_bundle_index_path(bundle_path);
    if out.is_file() && read_json_value(&out).is_some() {
        return Ok(out);
    }
    let payload = build_bundle_index_payload(bundle_path, warmup_frames)?;
    write_pretty_json(&out, &payload)?;
    Ok(out)
}

fn build_bundle_meta_payload(bundle_path: &Path, warmup_frames: u64) -> Result<Value, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    build_bundle_meta_payload_from_json(&bundle, &bundle_path.display().to_string(), warmup_frames)
}

fn build_bundle_index_payload(bundle_path: &Path, warmup_frames: u64) -> Result<Value, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    build_bundle_index_payload_from_json(&bundle, &bundle_path.display().to_string(), warmup_frames)
}

fn build_bundle_meta_payload_from_json(
    bundle: &Value,
    bundle_label: &str,
    warmup_frames: u64,
) -> Result<Value, String> {
    let semantics = SemanticsResolver::new(&bundle);
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut out_windows: Vec<Value> = Vec::new();
    let mut total_snapshots: u64 = 0;
    let mut total_unique_test_ids: HashSet<String> = HashSet::new();
    let mut total_snapshots_with_semantics: u64 = 0;
    let mut total_snapshots_with_inline_semantics: u64 = 0;
    let mut total_snapshots_with_table_semantics: u64 = 0;
    let mut total_unique_semantics_fingerprints: HashSet<u64> = HashSet::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        total_snapshots = total_snapshots.saturating_add(snaps.len() as u64);

        let mut window_snapshots_with_semantics: u64 = 0;
        let mut window_snapshots_with_inline_semantics: u64 = 0;
        let mut window_snapshots_with_table_semantics: u64 = 0;
        let mut window_unique_semantics_fingerprints: HashSet<u64> = HashSet::new();
        for s in snaps {
            let inline_nodes = snapshot_semantics_nodes(s).is_some();
            if inline_nodes {
                window_snapshots_with_inline_semantics =
                    window_snapshots_with_inline_semantics.saturating_add(1);
                total_snapshots_with_inline_semantics =
                    total_snapshots_with_inline_semantics.saturating_add(1);
            }

            let resolved_nodes = semantics.nodes(s).is_some();
            if resolved_nodes {
                window_snapshots_with_semantics = window_snapshots_with_semantics.saturating_add(1);
                total_snapshots_with_semantics = total_snapshots_with_semantics.saturating_add(1);
                if !inline_nodes {
                    window_snapshots_with_table_semantics =
                        window_snapshots_with_table_semantics.saturating_add(1);
                    total_snapshots_with_table_semantics =
                        total_snapshots_with_table_semantics.saturating_add(1);
                }
            }
            if let Some(fp) = s.get("semantics_fingerprint").and_then(|v| v.as_u64()) {
                window_unique_semantics_fingerprints.insert(fp);
                total_unique_semantics_fingerprints.insert(fp);
            }
        }

        let mut window_semantics_table_entries_total: u64 = 0;
        for e in semantics.table_entries() {
            if e.get("window").and_then(|v| v.as_u64()) == Some(window_id) {
                window_semantics_table_entries_total =
                    window_semantics_table_entries_total.saturating_add(1);
            }
        }
        let window_semantics_table_unique_keys_total =
            semantics.table_unique_keys_total_for_window(window_id) as u64;

        let Some(snapshot) = pick_last_snapshot_with_resolved_semantics_after_warmup(
            snaps,
            warmup_frames,
            &semantics,
        ) else {
            out_windows.push(json!({
                "window": window_id,
                "snapshots_total": snaps.len(),
                "snapshots_with_semantics_total": window_snapshots_with_semantics,
                "snapshots_with_inline_semantics_total": window_snapshots_with_inline_semantics,
                "snapshots_with_table_semantics_total": window_snapshots_with_table_semantics,
                "unique_semantics_fingerprints_total": window_unique_semantics_fingerprints.len(),
                "semantics_table_entries_total": window_semantics_table_entries_total,
                "semantics_table_unique_keys_total": window_semantics_table_unique_keys_total,
                "considered_frame_id": null,
                "considered_timestamp_unix_ms": null,
                "semantics_nodes_total": 0,
                "test_id_nodes_total": 0,
                "unique_test_ids_total": 0,
                "duplicate_test_ids_total": 0,
            }));
            continue;
        };

        let frame_id = snapshot_frame_id(snapshot);
        let ts = snapshot
            .get("timestamp_unix_ms")
            .and_then(|v| v.as_u64())
            .or_else(|| snapshot.get("timestamp_ms").and_then(|v| v.as_u64()));

        let nodes = semantics.nodes(snapshot).unwrap_or(&[]);

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
            "snapshots_with_inline_semantics_total": window_snapshots_with_inline_semantics,
            "snapshots_with_table_semantics_total": window_snapshots_with_table_semantics,
            "unique_semantics_fingerprints_total": window_unique_semantics_fingerprints.len(),
            "semantics_table_entries_total": window_semantics_table_entries_total,
            "semantics_table_unique_keys_total": window_semantics_table_unique_keys_total,
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
        "bundle": bundle_label,
        "warmup_frames": warmup_frames,
        "windows_total": windows.len(),
        "snapshots_total": total_snapshots,
        "total_unique_test_ids": total_unique_test_ids.len(),
        "snapshots_with_semantics_total": total_snapshots_with_semantics,
        "snapshots_with_inline_semantics_total": total_snapshots_with_inline_semantics,
        "snapshots_with_table_semantics_total": total_snapshots_with_table_semantics,
        "semantics_table_entries_total": semantics.table_entries_total(),
        "semantics_table_unique_keys_total": semantics.table_unique_keys_total(),
        "unique_semantics_fingerprints_total": total_unique_semantics_fingerprints.len(),
        "windows": out_windows,
    }))
}

fn build_bundle_index_payload_from_json(
    bundle: &Value,
    bundle_label: &str,
    warmup_frames: u64,
) -> Result<Value, String> {
    let semantics = SemanticsResolver::new(bundle);
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut windows_out: Vec<Value> = Vec::new();
    let mut total_snapshots: u64 = 0;

    let mut bloom_cache: HashMap<(u64, u64, u8), String> = HashMap::new();
    let mut semantics_blooms_windows: Vec<Value> = Vec::new();
    let mut semantics_blooms_keys_total: u64 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        total_snapshots = total_snapshots.saturating_add(snaps.len() as u64);

        let mut snapshots_out: Vec<Value> = Vec::with_capacity(snaps.len());

        let mut first_frame_id: Option<u64> = None;
        let mut last_frame_id: Option<u64> = None;
        let mut first_timestamp_unix_ms: Option<u64> = None;
        let mut last_timestamp_unix_ms: Option<u64> = None;

        let bloom_start_idx = snaps
            .len()
            .saturating_sub(TEST_ID_BLOOM_TAIL_SNAPSHOTS_PER_WINDOW);

        for (idx, s) in snaps.iter().enumerate() {
            let frame_id = snapshot_frame_id(s);
            let window_snapshot_seq = snapshot_window_snapshot_seq(s);
            let ts = s
                .get("timestamp_unix_ms")
                .and_then(|v| v.as_u64())
                .or_else(|| s.get("timestamp_ms").and_then(|v| v.as_u64()));

            first_frame_id.get_or_insert(frame_id);
            last_frame_id = Some(frame_id);
            if first_timestamp_unix_ms.is_none() {
                first_timestamp_unix_ms = ts;
            }
            last_timestamp_unix_ms = ts.or(last_timestamp_unix_ms);

            let semantics_fingerprint = snapshot_semantics_fingerprint(s);
            let has_inline_semantics = snapshot_semantics_nodes(s).is_some();
            let has_resolved_semantics = semantics.nodes(s).is_some();
            let semantics_source = if has_inline_semantics {
                "inline"
            } else if has_resolved_semantics {
                "table"
            } else {
                "none"
            };

            let test_id_bloom_hex = if idx >= bloom_start_idx
                && (semantics_source == "inline" || semantics_source == "table")
                && has_resolved_semantics
            {
                if let Some(fp) = semantics_fingerprint {
                    let source_tag = if semantics_source == "inline" {
                        0u8
                    } else {
                        1u8
                    };
                    let key = (window_id, fp, source_tag);
                    if let Some(hex) = bloom_cache.get(&key) {
                        Some(hex.clone())
                    } else {
                        let nodes = semantics.nodes(s).unwrap_or(&[]);
                        let mut bloom = TestIdBloomV1::new();
                        for n in nodes {
                            if let Some(tid) = n.get("test_id").and_then(|v| v.as_str()) {
                                bloom.add(tid);
                            }
                        }
                        let hex = bloom.to_hex();
                        bloom_cache.insert(key, hex.clone());
                        Some(hex)
                    }
                } else {
                    let nodes = semantics.nodes(s).unwrap_or(&[]);
                    let mut bloom = TestIdBloomV1::new();
                    for n in nodes {
                        if let Some(tid) = n.get("test_id").and_then(|v| v.as_str()) {
                            bloom.add(tid);
                        }
                    }
                    Some(bloom.to_hex())
                }
            } else {
                None
            };

            snapshots_out.push(json!({
                "window_snapshot_seq": window_snapshot_seq,
                "frame_id": frame_id,
                "timestamp_unix_ms": ts,
                "is_warmup": (idx as u64) < warmup_frames,
                "semantics_fingerprint": semantics_fingerprint,
                "semantics_source": semantics_source,
                "has_semantics": has_resolved_semantics,
                "test_id_bloom_hex": test_id_bloom_hex,
            }));
        }

        let mut keys_seen: HashSet<(u64, u8)> = HashSet::new();
        let mut items: Vec<Value> = Vec::new();
        for s in snaps.iter().rev() {
            if items.len() >= TEST_ID_BLOOM_MAX_SEMANTICS_KEYS_PER_WINDOW {
                break;
            }
            let Some(fp) = snapshot_semantics_fingerprint(s) else {
                continue;
            };
            let has_inline_semantics = snapshot_semantics_nodes(s).is_some();
            let has_resolved_semantics = semantics.nodes(s).is_some();
            if !has_resolved_semantics {
                continue;
            }
            let semantics_source = if has_inline_semantics {
                "inline"
            } else {
                "table"
            };
            let source_tag = if semantics_source == "inline" {
                0u8
            } else {
                1u8
            };
            if !keys_seen.insert((fp, source_tag)) {
                continue;
            }

            let key = (window_id, fp, source_tag);
            let hex = if let Some(hex) = bloom_cache.get(&key) {
                hex.clone()
            } else {
                let nodes = semantics.nodes(s).unwrap_or(&[]);
                let mut bloom = TestIdBloomV1::new();
                for n in nodes {
                    if let Some(tid) = n.get("test_id").and_then(|v| v.as_str()) {
                        bloom.add(tid);
                    }
                }
                let hex = bloom.to_hex();
                bloom_cache.insert(key, hex.clone());
                hex
            };

            items.push(json!({
                "semantics_fingerprint": fp,
                "semantics_source": semantics_source,
                "test_id_bloom_hex": hex,
            }));
        }
        if !items.is_empty() {
            semantics_blooms_keys_total =
                semantics_blooms_keys_total.saturating_add(items.len() as u64);
            semantics_blooms_windows.push(json!({
                "window": window_id,
                "items": items,
            }));
        }

        windows_out.push(json!({
            "window": window_id,
            "snapshots_total": snaps.len(),
            "first_frame_id": first_frame_id,
            "last_frame_id": last_frame_id,
            "first_timestamp_unix_ms": first_timestamp_unix_ms,
            "last_timestamp_unix_ms": last_timestamp_unix_ms,
            "snapshots": snapshots_out,
        }));
    }

    Ok(json!({
        "schema_version": 1,
        "kind": "bundle_index",
        "bundle": bundle_label,
        "warmup_frames": warmup_frames,
        "windows_total": windows.len(),
        "snapshots_total": total_snapshots,
        "test_id_bloom": {
            "schema_version": TEST_ID_BLOOM_V1_SCHEMA_VERSION,
            "m_bits": TEST_ID_BLOOM_V1_M_BITS,
            "k": TEST_ID_BLOOM_V1_K,
            "tail_snapshots_per_window": TEST_ID_BLOOM_TAIL_SNAPSHOTS_PER_WINDOW,
            "computed_from": "resolved_semantics_nodes",
            "semantics_source": "resolved",
            "covers_semantics_sources": ["inline", "table"],
        },
        "semantics_blooms": {
            "schema_version": 1,
            "m_bits": TEST_ID_BLOOM_V1_M_BITS,
            "k": TEST_ID_BLOOM_V1_K,
            "computed_from": "resolved_semantics_nodes",
            "max_keys_per_window": TEST_ID_BLOOM_MAX_SEMANTICS_KEYS_PER_WINDOW,
            "keys_total": semantics_blooms_keys_total,
            "windows": semantics_blooms_windows,
        },
        "windows": windows_out,
    }))
}

fn build_test_ids_payload(
    bundle_path: &Path,
    warmup_frames: u64,
    max_test_ids: usize,
) -> Result<Value, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let semantics = SemanticsResolver::new(&bundle);
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
        let Some(snapshot) = pick_last_snapshot_with_resolved_semantics_after_warmup(
            snaps,
            warmup_frames,
            &semantics,
        ) else {
            continue;
        };

        let frame_id = snapshot_frame_id(snapshot);
        let nodes = semantics.nodes(snapshot).unwrap_or(&[]);

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
    build_test_ids_index_payload_from_json(
        &bundle,
        &bundle_path.display().to_string(),
        warmup_frames,
    )
}

fn build_test_ids_index_payload_from_json(
    bundle: &Value,
    bundle_label: &str,
    warmup_frames: u64,
) -> Result<Value, String> {
    let semantics = SemanticsResolver::new(&bundle);
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
        let Some(snapshot) = pick_last_snapshot_with_resolved_semantics_after_warmup(
            snaps,
            warmup_frames,
            &semantics,
        ) else {
            continue;
        };

        let frame_id = snapshot_frame_id(snapshot);
        let nodes = semantics.nodes(snapshot).unwrap_or(&[]);

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
        "bundle": bundle_label,
        "warmup_frames": warmup_frames,
        "max_unique_test_ids_budget": MAX_UNIQUE_TEST_IDS_BUDGET,
        "truncated": truncated,
        "total_unique_test_ids": total_unique.len(),
        "windows": windows_out,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_v2_bundle() -> Value {
        json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "frame_id": 0, "window": 1, "semantics_fingerprint": 10, "debug": {} },
                    { "frame_id": 1, "window": 1, "semantics_fingerprint": 10, "debug": {} }
                ]
            }],
            "tables": {
                "semantics": {
                    "entries": [{
                        "window": 1,
                        "semantics_fingerprint": 10,
                        "semantics": {
                            "nodes": [
                                { "id": 1, "test_id": "a" },
                                { "id": 2, "test_id": "b" }
                            ]
                        }
                    }]
                }
            }
        })
    }

    fn sample_v2_bundle_with_inline_semantics() -> Value {
        json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "window": 1,
                        "window_snapshot_seq": 100,
                        "semantics_fingerprint": 10,
                        "debug": { "semantics": { "nodes": [{ "id": 1, "test_id": "a" }] } }
                    },
                    { "frame_id": 1, "window": 1, "window_snapshot_seq": 101, "semantics_fingerprint": 10, "debug": {} }
                ]
            }],
            "tables": {
                "semantics": {
                    "entries": [{
                        "window": 1,
                        "semantics_fingerprint": 10,
                        "semantics": {
                            "nodes": [
                                { "id": 1, "test_id": "a" },
                                { "id": 2, "test_id": "b" }
                            ]
                        }
                    }]
                }
            }
        })
    }

    #[test]
    fn bundle_meta_counts_semantics_via_table() {
        let bundle = sample_v2_bundle();
        let meta = build_bundle_meta_payload_from_json(&bundle, "bundle.json", 0).unwrap();

        assert_eq!(meta["kind"].as_str(), Some("bundle_meta"));
        assert_eq!(meta["snapshots_total"].as_u64(), Some(2));
        assert_eq!(meta["snapshots_with_semantics_total"].as_u64(), Some(2));
        assert_eq!(
            meta["snapshots_with_inline_semantics_total"].as_u64(),
            Some(0)
        );
        assert_eq!(
            meta["snapshots_with_table_semantics_total"].as_u64(),
            Some(2)
        );
        assert_eq!(meta["semantics_table_entries_total"].as_u64(), Some(1));
        assert_eq!(meta["semantics_table_unique_keys_total"].as_u64(), Some(1));
        assert_eq!(
            meta["unique_semantics_fingerprints_total"].as_u64(),
            Some(1)
        );

        let windows = meta["windows"].as_array().unwrap();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0]["window"].as_u64(), Some(1));
        assert_eq!(windows[0]["considered_frame_id"].as_u64(), Some(1));
        assert_eq!(windows[0]["semantics_nodes_total"].as_u64(), Some(2));
        assert_eq!(windows[0]["unique_test_ids_total"].as_u64(), Some(2));
        assert_eq!(
            windows[0]["snapshots_with_inline_semantics_total"].as_u64(),
            Some(0)
        );
        assert_eq!(
            windows[0]["snapshots_with_table_semantics_total"].as_u64(),
            Some(2)
        );
        assert_eq!(
            windows[0]["semantics_table_entries_total"].as_u64(),
            Some(1)
        );
        assert_eq!(
            windows[0]["semantics_table_unique_keys_total"].as_u64(),
            Some(1)
        );
    }

    #[test]
    fn bundle_index_records_snapshot_seq_and_semantics_source() {
        let bundle = sample_v2_bundle_with_inline_semantics();
        let idx = build_bundle_index_payload_from_json(&bundle, "bundle.json", 1).unwrap();

        assert_eq!(idx["kind"].as_str(), Some("bundle_index"));
        assert_eq!(idx["snapshots_total"].as_u64(), Some(2));
        assert_eq!(
            idx["test_id_bloom"]["schema_version"].as_u64(),
            Some(crate::test_id_bloom::TEST_ID_BLOOM_V1_SCHEMA_VERSION)
        );
        assert_eq!(
            idx["test_id_bloom"]["m_bits"].as_u64(),
            Some(crate::test_id_bloom::TEST_ID_BLOOM_V1_M_BITS as u64)
        );
        assert_eq!(
            idx["test_id_bloom"]["k"].as_u64(),
            Some(crate::test_id_bloom::TEST_ID_BLOOM_V1_K as u64)
        );
        assert_eq!(
            idx["test_id_bloom"]["semantics_source"].as_str(),
            Some("resolved")
        );

        let windows = idx["windows"].as_array().unwrap();
        assert_eq!(windows.len(), 1);
        let snaps = windows[0]["snapshots"].as_array().unwrap();
        assert_eq!(snaps.len(), 2);

        assert_eq!(snaps[0]["window_snapshot_seq"].as_u64(), Some(100));
        assert_eq!(snaps[0]["is_warmup"].as_bool(), Some(true));
        assert_eq!(snaps[0]["semantics_source"].as_str(), Some("inline"));
        let hex = snaps[0]["test_id_bloom_hex"].as_str().expect("bloom hex");
        let bloom = crate::test_id_bloom::TestIdBloomV1::from_hex(hex).expect("decode bloom");
        assert!(bloom.might_contain("a"));
        assert!(!bloom.might_contain("zzzz-not-present"));

        assert_eq!(snaps[1]["window_snapshot_seq"].as_u64(), Some(101));
        assert_eq!(snaps[1]["is_warmup"].as_bool(), Some(false));
        assert_eq!(snaps[1]["semantics_source"].as_str(), Some("table"));
        let hex = snaps[1]["test_id_bloom_hex"].as_str().expect("bloom hex");
        let bloom = crate::test_id_bloom::TestIdBloomV1::from_hex(hex).expect("decode bloom");
        assert!(bloom.might_contain("a"));
        assert!(bloom.might_contain("b"));

        let blooms = idx["semantics_blooms"]
            .as_object()
            .expect("semantics_blooms");
        assert_eq!(blooms["schema_version"].as_u64(), Some(1));
        let b_windows = blooms["windows"].as_array().expect("windows");
        assert_eq!(b_windows.len(), 1);
        let items = b_windows[0]["items"].as_array().expect("items");
        assert!(!items.is_empty());
        let any_hex = items[0]["test_id_bloom_hex"].as_str().expect("hex");
        let any = crate::test_id_bloom::TestIdBloomV1::from_hex(any_hex).expect("decode");
        assert!(any.might_contain("a"));
    }

    #[test]
    fn test_ids_index_uses_table_semantics() {
        let bundle = sample_v2_bundle();
        let idx = build_test_ids_index_payload_from_json(&bundle, "bundle.json", 0).unwrap();

        assert_eq!(idx["kind"].as_str(), Some("test_ids_index"));
        assert_eq!(idx["total_unique_test_ids"].as_u64(), Some(2));

        let windows = idx["windows"].as_array().unwrap();
        assert_eq!(windows.len(), 1);
        let items = windows[0]["items"].as_array().unwrap();
        let mut got: Vec<(String, u64)> = items
            .iter()
            .filter_map(|v| {
                Some((
                    v.get("test_id")?.as_str()?.to_string(),
                    v.get("count")?.as_u64()?,
                ))
            })
            .collect();
        got.sort();
        assert_eq!(got, vec![("a".to_string(), 1), ("b".to_string(), 1)]);
    }
}
