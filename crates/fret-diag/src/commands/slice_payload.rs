use std::collections::HashMap;
use std::path::Path;

use crate::json_bundle::{snapshot_frame_id, snapshot_window_snapshot_seq};

pub(super) fn build_test_id_slice_payload_from_snapshot_and_nodes(
    bundle_path: &Path,
    warmup_frames: u64,
    window_id: u64,
    snapshot: &serde_json::Value,
    nodes: &[serde_json::Value],
    test_id: &str,
    max_matches: usize,
    max_ancestors: usize,
) -> Result<serde_json::Value, String> {
    let frame_id = snapshot_frame_id(snapshot);
    let snapshot_seq = snapshot_window_snapshot_seq(snapshot);
    let ts = snapshot
        .get("timestamp_unix_ms")
        .and_then(|v| v.as_u64())
        .or_else(|| snapshot.get("timestamp_ms").and_then(|v| v.as_u64()));
    let window_bounds = snapshot.get("window_bounds").cloned();

    let mut by_id: HashMap<u64, usize> = HashMap::new();
    let mut parent: HashMap<u64, u64> = HashMap::new();

    for (idx, n) in nodes.iter().enumerate() {
        let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        by_id.insert(id, idx);
        if let Some(p) = n.get("parent").and_then(|v| v.as_u64()) {
            parent.entry(id).or_insert(p);
        }
        if let Some(children) = n.get("children").and_then(|v| v.as_array()) {
            for c in children {
                let Some(cid) = c.as_u64() else {
                    continue;
                };
                parent.entry(cid).or_insert(id);
            }
        }
    }

    let mut matches: Vec<u64> = nodes
        .iter()
        .filter_map(|n| {
            let id = n.get("id").and_then(|v| v.as_u64())?;
            let tid = n.get("test_id").and_then(|v| v.as_str())?;
            if tid.trim() == test_id {
                Some(id)
            } else {
                None
            }
        })
        .collect();
    matches.sort_unstable();
    if max_matches > 0 && matches.len() > max_matches {
        matches.truncate(max_matches);
    }

    let mut match_payloads: Vec<serde_json::Value> = Vec::new();
    for id in matches {
        let Some(idx) = by_id.get(&id).copied() else {
            continue;
        };
        let node = nodes[idx].clone();
        let mut ancestors: Vec<serde_json::Value> = Vec::new();
        let mut cur = id;
        for _ in 0..max_ancestors {
            let Some(p) = parent.get(&cur).copied() else {
                break;
            };
            cur = p;
            let Some(pidx) = by_id.get(&cur).copied() else {
                break;
            };
            let pn = &nodes[pidx];
            ancestors.push(serde_json::json!({
                "id": cur,
                "role": pn.get("role").cloned(),
                "test_id": pn.get("test_id").cloned(),
            }));
        }

        match_payloads.push(serde_json::json!({
            "node_id": id,
            "node": node,
            "ancestors": ancestors,
        }));
    }

    let stats = snapshot
        .get("debug")
        .and_then(|v| v.get("stats"))
        .and_then(|v| v.as_object())
        .map(|m| {
            serde_json::json!({
                "total_time_us": m.get("total_time_us").cloned(),
                "layout_time_us": m.get("layout_time_us").cloned(),
                "prepaint_time_us": m.get("prepaint_time_us").cloned(),
                "paint_time_us": m.get("paint_time_us").cloned(),
                "invalidation_walk_calls": m.get("invalidation_walk_calls").cloned(),
                "invalidation_walk_nodes": m.get("invalidation_walk_nodes").cloned(),
            })
        })
        .unwrap_or_else(|| serde_json::json!({}));

    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "slice.test_id",
        "bundle": bundle_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "window": window_id,
        "frame_id": frame_id,
        "window_snapshot_seq": snapshot_seq,
        "timestamp_unix_ms": ts,
        "window_bounds": window_bounds,
        "test_id": test_id,
        "matches": match_payloads,
        "stats": stats,
    }))
}
