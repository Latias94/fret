use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::json_bundle::{
    SemanticsResolver, pick_last_snapshot_with_semantics_after_warmup, snapshot_frame_id,
    snapshot_window_snapshot_seq,
};

fn slice_test_id_payload_from_bundle(
    bundle_path: &Path,
    bundle: &serde_json::Value,
    semantics: &SemanticsResolver<'_>,
    warmup_frames: u64,
    test_id: &str,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    window_id: Option<u64>,
    max_matches: usize,
    max_ancestors: usize,
) -> Result<serde_json::Value, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    struct Picked<'a> {
        window: u64,
        snapshot: &'a serde_json::Value,
    }

    fn snapshot_has_test_id(
        semantics: &SemanticsResolver<'_>,
        snapshot: &serde_json::Value,
        target: &str,
    ) -> bool {
        let nodes = semantics.nodes(snapshot).unwrap_or(&[]);
        nodes.iter().any(|n| {
            n.get("test_id")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.trim() == target)
        })
    }

    let mut picked: Option<Picked<'_>> = None;
    for w in windows {
        let w_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        if let Some(req) = window_id
            && req != w_id
        {
            continue;
        }
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        let snapshot = if let Some(req_frame) = frame_id {
            snaps.iter().find(|s| snapshot_frame_id(s) == req_frame)
        } else if let Some(req_seq) = window_snapshot_seq {
            snaps
                .iter()
                .find(|s| snapshot_window_snapshot_seq(s) == Some(req_seq))
        } else {
            pick_last_snapshot_with_semantics_after_warmup(snaps, warmup_frames)
        };
        let Some(snapshot) = snapshot else {
            continue;
        };
        if semantics.nodes(snapshot).is_none() {
            if frame_id.is_some() || window_snapshot_seq.is_some() {
                return Err("selected snapshot has no exported semantics (try a different --frame-id/--snapshot-seq, or ensure semantics export is enabled)".to_string());
            }
            continue;
        }
        if snapshot_has_test_id(semantics, snapshot, test_id) {
            picked = Some(Picked {
                window: w_id,
                snapshot,
            });
            break;
        }
        if picked.is_none() {
            picked = Some(Picked {
                window: w_id,
                snapshot,
            });
        }
    }

    let Some(picked) = picked else {
        return Err("bundle.json contains no windows".to_string());
    };

    let snapshot = picked.snapshot;
    let frame_id = snapshot_frame_id(snapshot);
    let snapshot_seq = snapshot_window_snapshot_seq(snapshot);
    let ts = snapshot
        .get("timestamp_unix_ms")
        .and_then(|v| v.as_u64())
        .or_else(|| snapshot.get("timestamp_ms").and_then(|v| v.as_u64()));
    let window_bounds = snapshot.get("window_bounds").cloned();

    let nodes = semantics
        .nodes(snapshot)
        .ok_or_else(|| "bundle snapshot missing semantics nodes".to_string())?;

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
        "window": picked.window,
        "frame_id": frame_id,
        "window_snapshot_seq": snapshot_seq,
        "timestamp_unix_ms": ts,
        "window_bounds": window_bounds,
        "test_id": test_id,
        "matches": match_payloads,
        "stats": stats,
    }))
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
}

fn sanitize_test_id_for_filename(test_id: &str) -> String {
    let mut out = String::with_capacity(test_id.len().min(80));
    for ch in test_id.chars().take(80) {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.trim().is_empty() {
        "test_id".to_string()
    } else {
        out
    }
}

fn resolve_bundle_json_path_or_latest(
    bundle_arg: Option<&str>,
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    if let Some(s) = bundle_arg {
        let src = crate::resolve_path(workspace_root, PathBuf::from(s));
        return Ok(crate::resolve_bundle_json_path(&src));
    }
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(crate::resolve_bundle_json_path(&latest))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_slice(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    slice_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let mut bundle_arg: Option<String> = None;
    let mut test_id: Option<String> = None;
    let mut frame_id: Option<u64> = None;
    let mut window_snapshot_seq: Option<u64> = None;
    let mut window_id: Option<u64> = None;
    let mut max_matches: usize = 20;
    let mut max_ancestors: usize = 64;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--test-id" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --test-id".to_string());
                };
                test_id = Some(v);
                i += 1;
            }
            "--frame-id" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --frame-id".to_string());
                };
                frame_id = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --frame-id (expected u64)".to_string())?,
                );
                i += 1;
            }
            "--snapshot-seq" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --snapshot-seq".to_string());
                };
                window_snapshot_seq =
                    Some(v.parse::<u64>().map_err(|_| {
                        "invalid value for --snapshot-seq (expected u64)".to_string()
                    })?);
                i += 1;
            }
            "--window" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --window".to_string());
                };
                window_id = Some(
                    v.parse::<u64>()
                        .map_err(|_| "invalid value for --window (expected u64)".to_string())?,
                );
                i += 1;
            }
            "--max-matches" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --max-matches".to_string());
                };
                max_matches = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --max-matches (expected usize)".to_string())?;
                i += 1;
            }
            "--max-ancestors" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --max-ancestors".to_string());
                };
                max_ancestors = v.parse::<usize>().map_err(|_| {
                    "invalid value for --max-ancestors (expected usize)".to_string()
                })?;
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for slice: {other}"));
            }
            other => {
                if bundle_arg.is_none() && looks_like_path(other) {
                    bundle_arg = Some(other.to_string());
                } else if bundle_arg.is_none() {
                    // allow positional bundle arg if it exists as a path
                    let p = crate::resolve_path(workspace_root, PathBuf::from(other));
                    if p.is_file() || p.is_dir() {
                        bundle_arg = Some(other.to_string());
                    } else if test_id.is_none() {
                        test_id = Some(other.to_string());
                    } else {
                        return Err(format!("unexpected argument: {other}"));
                    }
                } else if test_id.is_none() {
                    test_id = Some(other.to_string());
                } else {
                    return Err(format!("unexpected argument: {other}"));
                }
                i += 1;
            }
        }
    }

    let test_id = test_id.ok_or_else(|| {
        "missing --test-id (try: fretboard diag slice --test-id <test_id>)".to_string()
    })?;

    let bundle_path =
        resolve_bundle_json_path_or_latest(bundle_arg.as_deref(), workspace_root, out_dir)?;
    let bytes = std::fs::read(&bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let semantics = SemanticsResolver::new(&bundle);
    let payload = slice_test_id_payload_from_bundle(
        &bundle_path,
        &bundle,
        &semantics,
        warmup_frames,
        &test_id,
        frame_id,
        window_snapshot_seq,
        window_id,
        max_matches,
        max_ancestors,
    )?;

    let out = slice_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| {
            let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
            dir.join(format!(
                "slice.{}.json",
                sanitize_test_id_for_filename(test_id.as_str())
            ))
        });

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

    if stats_json {
        println!("{pretty}");
    } else {
        println!("{}", out.display());
    }
    Ok(())
}
