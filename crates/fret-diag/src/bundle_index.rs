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

const SCRIPT_STEP_INDEX_SCHEMA_VERSION: u32 = 1;
const SCRIPT_STEP_INDEX_MAX_TIMESTAMP_DELTA_MS: u64 = 2_000;

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

fn try_read_script_result_json(bundle_path: &Path) -> Option<Value> {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let direct = dir.join("script.result.json");
    let from_parent = if dir.file_name().and_then(|s| s.to_str()) == Some("_root") {
        dir.parent().map(|p| p.join("script.result.json"))
    } else {
        None
    };

    let v = if direct.is_file() {
        read_json_value(&direct)?
    } else if let Some(from_parent) = from_parent
        && from_parent.is_file()
    {
        read_json_value(&from_parent)?
    } else {
        return None;
    };
    if v.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        return None;
    }
    // Best-effort sanity check: a script result should have at least a stage.
    if v.get("stage").is_none() {
        return None;
    }
    Some(v)
}

#[derive(Debug, Clone)]
struct IndexSnapshotInfo {
    window_snapshot_seq: Option<u64>,
    timestamp_unix_ms: Option<u64>,
    semantics_source: Option<String>,
    semantics_fingerprint: Option<u64>,
}

fn build_index_snapshot_maps(
    idx: &Value,
) -> (
    HashMap<(u64, u64), IndexSnapshotInfo>,
    HashMap<u64, Vec<(u64, u64, IndexSnapshotInfo)>>,
) {
    let mut by_window_frame: HashMap<(u64, u64), IndexSnapshotInfo> = HashMap::new();
    let mut by_window_ts: HashMap<u64, Vec<(u64, u64, IndexSnapshotInfo)>> = HashMap::new();

    let empty = Vec::new();
    let windows = idx
        .get("windows")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);
    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps_empty = Vec::new();
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .unwrap_or(&snaps_empty);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64());
            let timestamp_unix_ms = s.get("timestamp_unix_ms").and_then(|v| v.as_u64());
            let info = IndexSnapshotInfo {
                window_snapshot_seq: s.get("window_snapshot_seq").and_then(|v| v.as_u64()),
                timestamp_unix_ms,
                semantics_source: s
                    .get("semantics_source")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                semantics_fingerprint: s.get("semantics_fingerprint").and_then(|v| v.as_u64()),
            };
            if let Some(frame_id) = frame_id {
                by_window_frame.insert((window, frame_id), info.clone());
            }
            if let Some(ts) = timestamp_unix_ms {
                by_window_ts
                    .entry(window)
                    .or_default()
                    .push((ts, frame_id.unwrap_or(0), info));
            }
        }
    }

    (by_window_frame, by_window_ts)
}

fn resolve_snapshot_for_event(
    by_window_frame: &HashMap<(u64, u64), IndexSnapshotInfo>,
    by_window_ts: &HashMap<u64, Vec<(u64, u64, IndexSnapshotInfo)>>,
    window: u64,
    frame_id: Option<u64>,
    unix_ms: Option<u64>,
) -> (
    Option<u64>,
    Option<u64>,
    Option<String>,
    Option<u64>,
    Option<&'static str>,
) {
    if let Some(frame_id) = frame_id {
        if let Some(info) = by_window_frame.get(&(window, frame_id)) {
            return (
                info.window_snapshot_seq,
                info.timestamp_unix_ms,
                info.semantics_source.clone(),
                info.semantics_fingerprint,
                Some("frame_id"),
            );
        }
    }

    let Some(unix_ms) = unix_ms else {
        return (None, None, None, None, None);
    };
    let Some(items) = by_window_ts.get(&window) else {
        return (None, None, None, None, None);
    };

    let mut best: Option<(u64, &IndexSnapshotInfo)> = None;
    for (ts, _frame, info) in items {
        let delta = ts.abs_diff(unix_ms);
        match best {
            Some((best_delta, _)) if best_delta <= delta => {}
            _ => best = Some((delta, info)),
        }
    }
    let Some((delta, info)) = best else {
        return (None, None, None, None, None);
    };
    if delta > SCRIPT_STEP_INDEX_MAX_TIMESTAMP_DELTA_MS {
        return (None, None, None, None, None);
    }
    (
        info.window_snapshot_seq,
        info.timestamp_unix_ms,
        info.semantics_source.clone(),
        info.semantics_fingerprint,
        Some("timestamp"),
    )
}

fn build_script_step_index_payload(idx: &Value, script_result: &Value) -> Option<Value> {
    let (by_window_frame, by_window_ts) = build_index_snapshot_maps(idx);

    let default_window = script_result.get("window").and_then(|v| v.as_u64());
    let run_id = script_result.get("run_id").and_then(|v| v.as_u64());
    let updated_unix_ms = script_result
        .get("updated_unix_ms")
        .and_then(|v| v.as_u64());

    let evidence = script_result.get("evidence");
    let events = evidence
        .and_then(|v| v.get("event_log"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    if events.is_empty() {
        return None;
    }

    let event_log_dropped = evidence
        .and_then(|v| v.get("event_log_dropped"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let mut unresolved_events: u64 = 0;
    let mut steps: HashMap<u32, Value> = HashMap::new();

    for e in events {
        let Some(step_index) = e.get("step_index").and_then(|v| v.as_u64()) else {
            continue;
        };
        let step_index_u32 = step_index.min(u32::MAX as u64) as u32;
        let kind = e
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let note = e
            .get("note")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let window = e.get("window").and_then(|v| v.as_u64()).or(default_window);
        let frame_id = e.get("frame_id").and_then(|v| v.as_u64());
        let unix_ms = e.get("unix_ms").and_then(|v| v.as_u64());

        let Some(window) = window else {
            unresolved_events = unresolved_events.saturating_add(1);
            continue;
        };

        let (window_snapshot_seq, timestamp_unix_ms, semantics_source, semantics_fingerprint, mode) =
            resolve_snapshot_for_event(&by_window_frame, &by_window_ts, window, frame_id, unix_ms);

        if window_snapshot_seq.is_none() && timestamp_unix_ms.is_none() {
            unresolved_events = unresolved_events.saturating_add(1);
            continue;
        }

        steps.insert(
            step_index_u32,
            json!({
                "step_index": step_index_u32,
                "kind": kind,
                "note": note,
                "window": window,
                "frame_id": frame_id,
                "window_snapshot_seq": window_snapshot_seq,
                "timestamp_unix_ms": timestamp_unix_ms,
                "semantics_source": semantics_source,
                "semantics_fingerprint": semantics_fingerprint,
                "resolve_mode": mode,
            }),
        );
    }

    if steps.is_empty() {
        return None;
    }

    let mut steps_out: Vec<Value> = steps.into_values().collect();
    steps_out.sort_by_key(|v| v.get("step_index").and_then(|v| v.as_u64()).unwrap_or(0));

    Some(json!({
        "schema_version": SCRIPT_STEP_INDEX_SCHEMA_VERSION,
        "source": "script.result.json",
        "run_id": run_id,
        "window": default_window,
        "updated_unix_ms": updated_unix_ms,
        "events_total": events.len(),
        "event_log_dropped": event_log_dropped,
        "unresolved_events_total": unresolved_events,
        "steps": steps_out,
    }))
}

pub(crate) fn default_bundle_meta_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("bundle.meta.json")
}

pub(crate) fn default_bundle_index_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("bundle.index.json")
}

pub(crate) fn default_window_map_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("window.map.json")
}

pub(crate) fn default_dock_routing_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("dock.routing.json")
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
    let expected_bundle = bundle_path.display().to_string();
    if out.is_file() {
        if let Some(existing) = read_json_value(&out) {
            let kind_ok = existing.get("kind").and_then(|v| v.as_str()) == Some("bundle_meta");
            let schema_ok = existing.get("schema_version").and_then(|v| v.as_u64()) == Some(1u64);
            let warmup_ok =
                existing.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames);
            let bundle_ok =
                existing.get("bundle").and_then(|v| v.as_str()) == Some(&expected_bundle);
            if kind_ok && schema_ok && warmup_ok && bundle_ok {
                return Ok(out);
            }
        }
    }
    let payload = build_bundle_meta_payload(bundle_path, warmup_frames)?;
    write_pretty_json(&out, &payload)?;
    Ok(out)
}

pub(crate) fn ensure_window_map_json(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<PathBuf, String> {
    let out = default_window_map_path(bundle_path);
    let expected_bundle = bundle_path.display().to_string();
    if out.is_file() {
        if let Some(existing) = read_json_value(&out) {
            let kind_ok = existing.get("kind").and_then(|v| v.as_str()) == Some("window_map");
            let schema_ok = existing.get("schema_version").and_then(|v| v.as_u64()) == Some(1u64);
            let warmup_ok =
                existing.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames);
            let bundle_ok =
                existing.get("bundle").and_then(|v| v.as_str()) == Some(&expected_bundle);
            if kind_ok && schema_ok && warmup_ok && bundle_ok {
                return Ok(out);
            }
        }
    }
    let payload = build_window_map_payload(bundle_path, warmup_frames)?;
    write_pretty_json(&out, &payload)?;
    Ok(out)
}

pub(crate) fn ensure_dock_routing_json(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<PathBuf, String> {
    let out = default_dock_routing_path(bundle_path);
    let expected_bundle = bundle_path.display().to_string();
    if out.is_file() {
        if let Some(existing) = read_json_value(&out) {
            let kind_ok = existing.get("kind").and_then(|v| v.as_str()) == Some("dock_routing");
            let schema_ok = existing.get("schema_version").and_then(|v| v.as_u64()) == Some(1u64);
            let warmup_ok =
                existing.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames);
            let bundle_ok =
                existing.get("bundle").and_then(|v| v.as_str()) == Some(&expected_bundle);
            // `dock_routing` is an evolving, "bounded evidence" sidecar. Prefer regenerating when
            // it appears to be missing newer key fields, even if the schema version matches.
            let content_ok = existing
                .get("entries")
                .and_then(|v| v.as_array())
                .is_some_and(|entries| {
                    for e in entries {
                        if let Some(drag) = e.get("dock_drag").and_then(|v| v.as_object()) {
                            return drag.contains_key("kind");
                        }
                    }
                    true
                });
            if kind_ok && schema_ok && warmup_ok && bundle_ok && content_ok {
                return Ok(out);
            }
        }
    }
    let payload = build_dock_routing_payload(bundle_path, warmup_frames)?;
    write_pretty_json(&out, &payload)?;
    Ok(out)
}

pub(crate) fn ensure_test_ids_json(
    bundle_path: &Path,
    warmup_frames: u64,
    max_test_ids: usize,
) -> Result<PathBuf, String> {
    let out = default_test_ids_path(bundle_path);
    let expected_bundle = bundle_path.display().to_string();
    if out.is_file() {
        if let Some(existing) = read_json_value(&out) {
            let kind_ok = existing.get("kind").and_then(|v| v.as_str()) == Some("test_ids");
            let schema_ok = existing.get("schema_version").and_then(|v| v.as_u64()) == Some(1u64);
            let warmup_ok =
                existing.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames);
            let max_ok = existing.get("max_test_ids").and_then(|v| v.as_u64())
                == Some(max_test_ids.min(u64::MAX as usize) as u64);
            let bundle_ok =
                existing.get("bundle").and_then(|v| v.as_str()) == Some(&expected_bundle);
            if kind_ok && schema_ok && warmup_ok && max_ok && bundle_ok {
                return Ok(out);
            }
        }
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
    let expected_bundle = bundle_path.display().to_string();
    if out.is_file() {
        if let Some(existing) = read_json_value(&out) {
            let kind_ok = existing.get("kind").and_then(|v| v.as_str()) == Some("test_ids_index");
            let schema_ok = existing.get("schema_version").and_then(|v| v.as_u64()) == Some(1u64);
            let warmup_ok =
                existing.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames);
            let bundle_ok =
                existing.get("bundle").and_then(|v| v.as_str()) == Some(&expected_bundle);
            if kind_ok && schema_ok && warmup_ok && bundle_ok {
                return Ok(out);
            }
        }
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
    let expected_bundle = bundle_path.display().to_string();
    if out.is_file() {
        if let Some(existing) = read_json_value(&out) {
            let kind_ok = existing.get("kind").and_then(|v| v.as_str()) == Some("bundle_index");
            let schema_ok = existing.get("schema_version").and_then(|v| v.as_u64()) == Some(1u64);
            let warmup_ok =
                existing.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames);
            let bundle_ok =
                existing.get("bundle").and_then(|v| v.as_str()) == Some(&expected_bundle);

            if !kind_ok || !schema_ok || !warmup_ok || !bundle_ok {
                // Fall through and regenerate.
            } else {
                // Best-effort upgrade: older indexes may be missing the additive `script` markers.
                // If we can compute them (script.result.json adjacent), regenerate once.
                let missing_script_markers = existing
                    .get("script")
                    .and_then(|v| v.get("steps"))
                    .is_none();
                if missing_script_markers
                    && let Some(script_result) = try_read_script_result_json(bundle_path)
                    && build_script_step_index_payload(&existing, &script_result).is_some()
                {
                    // Fall through and regenerate.
                } else {
                    return Ok(out);
                }
            }
        } else {
            // Fall through and regenerate.
        }
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
    let mut payload = build_bundle_index_payload_from_json(
        &bundle,
        &bundle_path.display().to_string(),
        warmup_frames,
    )?;

    if let Some(script_result) = try_read_script_result_json(bundle_path)
        && let Some(script_steps) = build_script_step_index_payload(&payload, &script_result)
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert("script".to_string(), script_steps);
    }

    Ok(payload)
}

fn build_window_map_payload(bundle_path: &Path, warmup_frames: u64) -> Result<Value, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    build_window_map_payload_from_json(&bundle, &bundle_path.display().to_string(), warmup_frames)
}

fn build_dock_routing_payload(bundle_path: &Path, warmup_frames: u64) -> Result<Value, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    build_dock_routing_payload_from_json(&bundle, &bundle_path.display().to_string(), warmup_frames)
}

fn hash_str_64(s: &str) -> u64 {
    let mut fp: u64 = 14695981039346656037;
    for b in s.as_bytes() {
        fp ^= *b as u64;
        fp = fp.wrapping_mul(1099511628211);
    }
    fp
}

fn build_dock_routing_payload_from_json(
    bundle: &Value,
    bundle_label: &str,
    warmup_frames: u64,
) -> Result<Value, String> {
    const MAX_ENTRIES: usize = 512;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

    let mut entries: Vec<Value> = Vec::new();
    let mut last_fingerprint_by_window: HashMap<u64, u64> = HashMap::new();

    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v.as_slice());

        for (idx, s) in snaps.iter().enumerate() {
            if (idx as u64) < warmup_frames {
                continue;
            }

            let docking = s
                .get("debug")
                .and_then(|v| v.get("docking_interaction"))
                .and_then(|v| v.as_object());
            let Some(docking) = docking else {
                continue;
            };

            let dock_drag = docking.get("dock_drag").and_then(|v| v.as_object());
            let dock_drop = docking.get("dock_drop_resolve").and_then(|v| v.as_object());

            let drag_interesting = dock_drag.is_some_and(|d| {
                d.get("dragging").and_then(|v| v.as_bool()).unwrap_or(false)
                    || d.get("cross_window_hover")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
            });
            let interesting = drag_interesting || dock_drop.is_some();
            if !interesting {
                continue;
            }

            let hover_detection = s
                .get("caps")
                .and_then(|v| v.get("ui_window_hover_detection"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let mut fp: u64 = 14695981039346656037;
            let mix = |fp: &mut u64, v: u64| {
                *fp ^= v;
                *fp = fp.wrapping_mul(1099511628211);
            };

            mix(&mut fp, window);
            mix(&mut fp, hash_str_64(hover_detection));

            if let Some(d) = dock_drag {
                mix(
                    &mut fp,
                    d.get("pointer_id").and_then(|v| v.as_u64()).unwrap_or(0),
                );
                mix(
                    &mut fp,
                    d.get("source_window").and_then(|v| v.as_u64()).unwrap_or(0),
                );
                mix(
                    &mut fp,
                    d.get("current_window")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                );
                mix(
                    &mut fp,
                    d.get("current_window_scale_factor_x1000")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                );
                mix(
                    &mut fp,
                    hash_str_64(d.get("kind").and_then(|v| v.as_str()).unwrap_or("")),
                );
                mix(
                    &mut fp,
                    d.get("dragging").and_then(|v| v.as_bool()).unwrap_or(false) as u64,
                );
                mix(
                    &mut fp,
                    d.get("cross_window_hover")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false) as u64,
                );
                mix(
                    &mut fp,
                    d.get("transparent_payload_applied")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false) as u64,
                );
                mix(
                    &mut fp,
                    d.get("transparent_payload_mouse_passthrough_applied")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false) as u64,
                );
                mix(
                    &mut fp,
                    hash_str_64(
                        d.get("window_under_cursor_source")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                );
                mix(
                    &mut fp,
                    d.get("moving_window").and_then(|v| v.as_u64()).unwrap_or(0),
                );
                mix(
                    &mut fp,
                    d.get("moving_window_scale_factor_x1000")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                );
                mix(
                    &mut fp,
                    d.get("window_under_moving_window")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                );
                mix(
                    &mut fp,
                    hash_str_64(
                        d.get("window_under_moving_window_source")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                );
            }

            if let Some(d) = dock_drop {
                mix(
                    &mut fp,
                    d.get("pointer_id").and_then(|v| v.as_u64()).unwrap_or(0),
                );
                if let Some(source) = d.get("source") {
                    let label = serde_json::to_string(source).unwrap_or_default();
                    mix(&mut fp, hash_str_64(&label));
                }
                if let Some(r) = d.get("resolved").and_then(|v| v.as_object()) {
                    mix(
                        &mut fp,
                        r.get("layout_root").and_then(|v| v.as_u64()).unwrap_or(0),
                    );
                    mix(&mut fp, r.get("tabs").and_then(|v| v.as_u64()).unwrap_or(0));
                    if let Some(zone) = r.get("zone") {
                        let label = serde_json::to_string(zone).unwrap_or_default();
                        mix(&mut fp, hash_str_64(&label));
                    }
                    mix(
                        &mut fp,
                        r.get("outer").and_then(|v| v.as_bool()).unwrap_or(false) as u64,
                    );
                }
                if let Some(p) = d.get("preview").and_then(|v| v.as_object()) {
                    if let Some(kind) = p.get("kind") {
                        let label = serde_json::to_string(kind).unwrap_or_default();
                        mix(&mut fp, hash_str_64(&label));
                    }
                }
            }

            if last_fingerprint_by_window.get(&window).copied() == Some(fp) {
                continue;
            }
            last_fingerprint_by_window.insert(window, fp);

            let dock_drag_out = dock_drag.map(|d| {
                json!({
                    "pointer_id": d.get("pointer_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    "source_window": d.get("source_window").and_then(|v| v.as_u64()).unwrap_or(0),
                    "current_window": d.get("current_window").and_then(|v| v.as_u64()).unwrap_or(0),
                    "position": d.get("position").cloned().unwrap_or(Value::Null),
                    "start_position": d.get("start_position").cloned().unwrap_or(Value::Null),
                    "cursor_grab_offset": d.get("cursor_grab_offset").cloned().unwrap_or(Value::Null),
                    "follow_window": d.get("follow_window").cloned().unwrap_or(Value::Null),
                    "cursor_screen_pos_raw_physical_px": d.get("cursor_screen_pos_raw_physical_px").cloned().unwrap_or(Value::Null),
                    "cursor_screen_pos_used_physical_px": d.get("cursor_screen_pos_used_physical_px").cloned().unwrap_or(Value::Null),
                    "cursor_screen_pos_was_clamped": d.get("cursor_screen_pos_was_clamped").and_then(|v| v.as_bool()).unwrap_or(false),
                    "cursor_override_active": d.get("cursor_override_active").and_then(|v| v.as_bool()).unwrap_or(false),
                    "current_window_outer_pos_physical_px": d.get("current_window_outer_pos_physical_px").cloned().unwrap_or(Value::Null),
                    "current_window_decoration_offset_physical_px": d.get("current_window_decoration_offset_physical_px").cloned().unwrap_or(Value::Null),
                    "current_window_client_origin_screen_physical_px": d.get("current_window_client_origin_screen_physical_px").cloned().unwrap_or(Value::Null),
                    "current_window_client_origin_source_platform": d.get("current_window_client_origin_source_platform").and_then(|v| v.as_bool()).unwrap_or(false),
                    "current_window_scale_factor_x1000_from_runner": d.get("current_window_scale_factor_x1000_from_runner").cloned().unwrap_or(Value::Null),
                    "current_window_local_pos_from_screen_logical_px": d.get("current_window_local_pos_from_screen_logical_px").cloned().unwrap_or(Value::Null),
                    "current_window_scale_factor_x1000": d.get("current_window_scale_factor_x1000").cloned().unwrap_or(Value::Null),
                    "kind": d.get("kind").cloned().unwrap_or(Value::Null),
                    "dragging": d.get("dragging").and_then(|v| v.as_bool()).unwrap_or(false),
                    "cross_window_hover": d.get("cross_window_hover").and_then(|v| v.as_bool()).unwrap_or(false),
                    "transparent_payload_applied": d.get("transparent_payload_applied").and_then(|v| v.as_bool()).unwrap_or(false),
                    "transparent_payload_mouse_passthrough_applied": d.get("transparent_payload_mouse_passthrough_applied").and_then(|v| v.as_bool()).unwrap_or(false),
                    "window_under_cursor_source": d.get("window_under_cursor_source").cloned().unwrap_or(Value::Null),
                    "moving_window": d.get("moving_window").cloned().unwrap_or(Value::Null),
                    "moving_window_scale_factor_x1000": d.get("moving_window_scale_factor_x1000").cloned().unwrap_or(Value::Null),
                    "window_under_moving_window": d.get("window_under_moving_window").cloned().unwrap_or(Value::Null),
                    "window_under_moving_window_source": d.get("window_under_moving_window_source").cloned().unwrap_or(Value::Null),
                })
            });

            let dock_drop_out = dock_drop.map(|d| {
                json!({
                    "pointer_id": d.get("pointer_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    "source": d.get("source").cloned().unwrap_or(Value::Null),
                    "resolved": d.get("resolved").and_then(|v| v.as_object()).map(|r| json!({
                        "layout_root": r.get("layout_root").and_then(|v| v.as_u64()).unwrap_or(0),
                        "tabs": r.get("tabs").and_then(|v| v.as_u64()).unwrap_or(0),
                        "zone": r.get("zone").cloned().unwrap_or(Value::Null),
                        "outer": r.get("outer").and_then(|v| v.as_bool()).unwrap_or(false),
                    })),
                    "preview": d.get("preview").and_then(|v| v.as_object()).map(|p| json!({
                        "kind": p.get("kind").cloned().unwrap_or(Value::Null),
                    })),
                })
            });

            let timestamp_unix_ms = s
                .get("timestamp_unix_ms")
                .and_then(|v| v.as_u64())
                .or_else(|| s.get("timestamp_ms").and_then(|v| v.as_u64()));

            entries.push(json!({
                "window": window,
                "tick_id": s.get("tick_id").and_then(|v| v.as_u64()),
                "frame_id": s.get("frame_id").and_then(|v| v.as_u64()),
                "timestamp_unix_ms": timestamp_unix_ms,
                "window_snapshot_seq": s.get("window_snapshot_seq").and_then(|v| v.as_u64()),
                "ui_window_hover_detection": hover_detection,
                "dock_drag": dock_drag_out,
                "dock_drop_resolve": dock_drop_out,
            }));

            if entries.len() > MAX_ENTRIES {
                let extra = entries.len().saturating_sub(MAX_ENTRIES);
                entries.drain(0..extra);
            }
        }
    }

    Ok(json!({
        "schema_version": 1,
        "kind": "dock_routing",
        "bundle": bundle_label,
        "warmup_frames": warmup_frames,
        "entries_total": entries.len(),
        "entries": entries,
    }))
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
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

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

fn build_window_map_payload_from_json(
    bundle: &Value,
    bundle_label: &str,
    warmup_frames: u64,
) -> Result<Value, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing windows".to_string())?;

    fn get_rect(v: &Value) -> Option<Value> {
        let x = v.get("x_px").and_then(|v| v.as_f64());
        let y = v.get("y_px").and_then(|v| v.as_f64());
        let w = v.get("w_px").and_then(|v| v.as_f64());
        let h = v.get("h_px").and_then(|v| v.as_f64());
        match (x, y, w, h) {
            (Some(x), Some(y), Some(w), Some(h)) => Some(json!({
                "x_px": x,
                "y_px": y,
                "w_px": w,
                "h_px": h,
            })),
            _ => None,
        }
    }

    fn snapshot_seen(s: &Value) -> Value {
        json!({
            "tick_id": s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
            "frame_id": s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
            "timestamp_unix_ms": s.get("timestamp_unix_ms").and_then(|v| v.as_u64()).unwrap_or(0),
            "window_snapshot_seq": s.get("window_snapshot_seq").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    }

    let mut windows_out: Vec<Value> = Vec::with_capacity(windows.len());

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let events_total = w
            .get("events")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let first = snaps.first();
        let last = snaps.last();

        let last_bounds = last.and_then(|s| s.get("window_bounds")).and_then(get_rect);
        let last_scale_factor = last
            .and_then(|s| s.get("scale_factor"))
            .and_then(|v| v.as_f64());
        let last_primary_pointer_type = last
            .and_then(|s| s.get("primary_pointer_type"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let last_hover_detection = last
            .and_then(|s| s.get("caps"))
            .and_then(|v| v.get("ui_window_hover_detection"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let last_docking_interaction_present = last
            .and_then(|s| s.get("debug"))
            .and_then(|v| v.get("docking_interaction"))
            .is_some();

        windows_out.push(json!({
            "window": window_id,
            "snapshots_total": snaps.len(),
            "events_total": events_total,
            "first_seen": first.map(snapshot_seen),
            "last_seen": last.map(|s| {
                let mut out = snapshot_seen(s);
                if let Some(obj) = out.as_object_mut() {
                    obj.insert("window_bounds".to_string(), last_bounds.clone().unwrap_or(Value::Null));
                    obj.insert(
                        "scale_factor".to_string(),
                        last_scale_factor
                            .map(|v| json!(v))
                            .unwrap_or(Value::Null),
                    );
                    obj.insert(
                        "primary_pointer_type".to_string(),
                        last_primary_pointer_type
                            .clone()
                            .map(|v| json!(v))
                            .unwrap_or(Value::Null),
                    );
                    obj.insert(
                        "ui_window_hover_detection".to_string(),
                        last_hover_detection
                            .clone()
                            .map(|v| json!(v))
                            .unwrap_or(Value::Null),
                    );
                    obj.insert("docking_interaction_present".to_string(), Value::Bool(last_docking_interaction_present));
                }
                out
            }),
        }));
    }

    Ok(json!({
        "schema_version": 1,
        "kind": "window_map",
        "bundle": bundle_label,
        "warmup_frames": warmup_frames,
        "windows_total": windows.len(),
        "windows": windows_out,
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
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

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
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

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
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root(prefix: &str) -> PathBuf {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        std::env::temp_dir().join(format!("{prefix}-{ms}"))
    }

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

    fn sample_v2_bundle_with_window_map_fields() -> Value {
        json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "events": [{ "kind": "noop" }],
                "snapshots": [
                    {
                        "tick_id": 7,
                        "frame_id": 42,
                        "timestamp_unix_ms": 123,
                        "window_snapshot_seq": 100,
                        "window": 1,
                        "scale_factor": 2.0,
                        "window_bounds": { "x_px": 1.0, "y_px": 2.0, "w_px": 300.0, "h_px": 200.0 },
                        "primary_pointer_type": "mouse",
                        "caps": { "ui_window_hover_detection": "platform_win32" },
                        "debug": { "docking_interaction": { "dock_drag": null } }
                    }
                ]
            }]
        })
    }

    fn sample_v2_bundle_with_dock_routing_fields() -> Value {
        json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "events": [{ "kind": "noop" }],
                "snapshots": [
                    {
                        "tick_id": 7,
                        "frame_id": 42,
                        "timestamp_unix_ms": 123,
                        "window_snapshot_seq": 100,
                        "window": 1,
                        "caps": { "ui_window_hover_detection": "platform_win32" },
                        "debug": {
                            "docking_interaction": {
                                "dock_drag": {
                                    "pointer_id": 1,
                                    "source_window": 1,
                                    "current_window": 2,
                                    "position": { "x": 12.0, "y": 34.0 },
                                    "start_position": { "x": 1.0, "y": 2.0 },
                                    "cursor_grab_offset": { "x": 3.0, "y": 4.0 },
                                    "follow_window": 9,
                                    "cursor_screen_pos_raw_physical_px": { "x": 100.0, "y": 200.0 },
                                    "cursor_screen_pos_used_physical_px": { "x": 110.0, "y": 210.0 },
                                    "cursor_screen_pos_was_clamped": true,
                                    "cursor_override_active": true,
                                    "current_window_outer_pos_physical_px": { "x": 1000.0, "y": 2000.0 },
                                    "current_window_decoration_offset_physical_px": { "x": 7.0, "y": 31.0 },
                                    "current_window_client_origin_screen_physical_px": { "x": 1007.0, "y": 2031.0 },
                                    "current_window_client_origin_source_platform": true,
                                    "current_window_scale_factor_x1000_from_runner": 1250,
                                    "current_window_local_pos_from_screen_logical_px": { "x": 8.0, "y": 16.0 },
                                    "current_window_scale_factor_x1000": 1500,
                                    "kind": "dock_panel",
                                    "dragging": true,
                                    "cross_window_hover": true,
                                    "transparent_payload_applied": true,
                                    "transparent_payload_mouse_passthrough_applied": true,
                                    "window_under_cursor_source": "heuristic_rects"
                                    ,
                                    "moving_window": 2,
                                    "moving_window_scale_factor_x1000": 1000,
                                    "window_under_moving_window": 1,
                                    "window_under_moving_window_source": "platform_win32"
                                },
                                "dock_drop_resolve": {
                                    "pointer_id": 1,
                                    "source": "outer_hint_rect",
                                    "resolved": { "layout_root": 11, "tabs": 22, "zone": "left", "outer": false },
                                    "preview": { "kind": { "kind": "wrap_binary" } }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 8,
                        "frame_id": 43,
                        "timestamp_unix_ms": 124,
                        "window_snapshot_seq": 101,
                        "window": 1,
                        "caps": { "ui_window_hover_detection": "platform_win32" },
                        "debug": {
                            "docking_interaction": {
                                "dock_drag": {
                                    "pointer_id": 1,
                                    "source_window": 1,
                                    "current_window": 2,
                                    "current_window_scale_factor_x1000": 1500,
                                    "kind": "dock_panel",
                                    "dragging": true,
                                    "cross_window_hover": true,
                                    "transparent_payload_applied": true,
                                    "transparent_payload_mouse_passthrough_applied": true,
                                    "window_under_cursor_source": "heuristic_rects"
                                    ,
                                    "moving_window": 2,
                                    "moving_window_scale_factor_x1000": 1000,
                                    "window_under_moving_window": 1,
                                    "window_under_moving_window_source": "platform_win32"
                                },
                                "dock_drop_resolve": {
                                    "pointer_id": 1,
                                    "source": "outer_hint_rect",
                                    "resolved": { "layout_root": 11, "tabs": 22, "zone": "left", "outer": false },
                                    "preview": { "kind": { "kind": "wrap_binary" } }
                                }
                            }
                        }
                    }
                ]
            }]
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
    fn window_map_records_last_bounds_and_hover_detection() {
        let bundle = sample_v2_bundle_with_window_map_fields();
        let map = build_window_map_payload_from_json(&bundle, "bundle.json", 0).unwrap();

        assert_eq!(map["kind"].as_str(), Some("window_map"));
        assert_eq!(map["schema_version"].as_u64(), Some(1));
        assert_eq!(map["windows_total"].as_u64(), Some(1));

        let windows = map["windows"].as_array().unwrap();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0]["window"].as_u64(), Some(1));
        assert_eq!(windows[0]["snapshots_total"].as_u64(), Some(1));
        assert_eq!(windows[0]["events_total"].as_u64(), Some(1));

        let last = windows[0]["last_seen"].as_object().unwrap();
        assert_eq!(last.get("frame_id").and_then(|v| v.as_u64()), Some(42));
        assert_eq!(
            last.get("ui_window_hover_detection")
                .and_then(|v| v.as_str()),
            Some("platform_win32")
        );
        assert_eq!(
            last.get("docking_interaction_present")
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        let bounds = last.get("window_bounds").unwrap().as_object().unwrap();
        assert_eq!(bounds.get("x_px").and_then(|v| v.as_f64()), Some(1.0));
        assert_eq!(bounds.get("y_px").and_then(|v| v.as_f64()), Some(2.0));
        assert_eq!(bounds.get("w_px").and_then(|v| v.as_f64()), Some(300.0));
        assert_eq!(bounds.get("h_px").and_then(|v| v.as_f64()), Some(200.0));
    }

    #[test]
    fn dock_routing_dedups_repeated_frames_and_records_key_fields() {
        let bundle = sample_v2_bundle_with_dock_routing_fields();
        let routing = build_dock_routing_payload_from_json(&bundle, "bundle.json", 0).unwrap();

        assert_eq!(routing["kind"].as_str(), Some("dock_routing"));
        assert_eq!(routing["schema_version"].as_u64(), Some(1));
        assert_eq!(routing["entries_total"].as_u64(), Some(1));

        let entries = routing["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        let e0 = entries[0].as_object().unwrap();
        assert_eq!(e0.get("window").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(e0.get("tick_id").and_then(|v| v.as_u64()), Some(7));
        assert_eq!(e0.get("frame_id").and_then(|v| v.as_u64()), Some(42));
        assert_eq!(
            e0.get("ui_window_hover_detection").and_then(|v| v.as_str()),
            Some("platform_win32")
        );

        let drag = e0.get("dock_drag").unwrap().as_object().unwrap();
        assert_eq!(drag.get("dragging").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            drag.get("kind").and_then(|v| v.as_str()),
            Some("dock_panel")
        );
        let pos = drag.get("position").unwrap().as_object().unwrap();
        assert_eq!(pos.get("x").and_then(|v| v.as_f64()), Some(12.0));
        assert_eq!(pos.get("y").and_then(|v| v.as_f64()), Some(34.0));
        let start = drag.get("start_position").unwrap().as_object().unwrap();
        assert_eq!(start.get("x").and_then(|v| v.as_f64()), Some(1.0));
        assert_eq!(start.get("y").and_then(|v| v.as_f64()), Some(2.0));
        let grab = drag.get("cursor_grab_offset").unwrap().as_object().unwrap();
        assert_eq!(grab.get("x").and_then(|v| v.as_f64()), Some(3.0));
        assert_eq!(grab.get("y").and_then(|v| v.as_f64()), Some(4.0));
        assert_eq!(drag.get("follow_window").and_then(|v| v.as_u64()), Some(9));
        let scr_raw = drag
            .get("cursor_screen_pos_raw_physical_px")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(scr_raw.get("x").and_then(|v| v.as_f64()), Some(100.0));
        assert_eq!(scr_raw.get("y").and_then(|v| v.as_f64()), Some(200.0));
        let scr_used = drag
            .get("cursor_screen_pos_used_physical_px")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(scr_used.get("x").and_then(|v| v.as_f64()), Some(110.0));
        assert_eq!(scr_used.get("y").and_then(|v| v.as_f64()), Some(210.0));
        assert_eq!(
            drag.get("cursor_screen_pos_was_clamped")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            drag.get("cursor_override_active").and_then(|v| v.as_bool()),
            Some(true)
        );
        let origin = drag
            .get("current_window_client_origin_screen_physical_px")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(origin.get("x").and_then(|v| v.as_f64()), Some(1007.0));
        assert_eq!(origin.get("y").and_then(|v| v.as_f64()), Some(2031.0));
        assert_eq!(
            drag.get("current_window_client_origin_source_platform")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            drag.get("current_window_scale_factor_x1000_from_runner")
                .and_then(|v| v.as_u64()),
            Some(1250)
        );
        let derived = drag
            .get("current_window_local_pos_from_screen_logical_px")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(derived.get("x").and_then(|v| v.as_f64()), Some(8.0));
        assert_eq!(derived.get("y").and_then(|v| v.as_f64()), Some(16.0));
        assert_eq!(
            drag.get("current_window_scale_factor_x1000")
                .and_then(|v| v.as_u64()),
            Some(1500)
        );
        assert_eq!(
            drag.get("cross_window_hover").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            drag.get("window_under_cursor_source")
                .and_then(|v| v.as_str()),
            Some("heuristic_rects")
        );
        assert_eq!(drag.get("moving_window").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(
            drag.get("moving_window_scale_factor_x1000")
                .and_then(|v| v.as_u64()),
            Some(1000)
        );
        assert_eq!(
            drag.get("window_under_moving_window")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            drag.get("window_under_moving_window_source")
                .and_then(|v| v.as_str()),
            Some("platform_win32")
        );

        let drop = e0.get("dock_drop_resolve").unwrap().as_object().unwrap();
        assert_eq!(
            drop.get("source").and_then(|v| v.as_str()),
            Some("outer_hint_rect")
        );
        let resolved = drop.get("resolved").unwrap().as_object().unwrap();
        assert_eq!(resolved.get("zone").and_then(|v| v.as_str()), Some("left"));
        let preview = drop.get("preview").unwrap().as_object().unwrap();
        let kind = preview.get("kind").unwrap().as_object().unwrap();
        assert_eq!(
            kind.get("kind").and_then(|v| v.as_str()),
            Some("wrap_binary")
        );
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

    #[test]
    fn script_step_index_resolves_step_to_snapshot_selector() {
        let idx = json!({
            "kind": "bundle_index",
            "schema_version": 1,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 10,
                    "window_snapshot_seq": 7,
                    "timestamp_unix_ms": 1000,
                    "semantics_source": "inline",
                    "semantics_fingerprint": 42,
                    "has_semantics": true,
                }]
            }]
        });

        let script = json!({
            "schema_version": 1,
            "run_id": 123,
            "updated_unix_ms": 1005,
            "window": 1,
            "stage": "running",
            "evidence": {
                "event_log": [{
                    "unix_ms": 1000,
                    "kind": "step_end",
                    "step_index": 3,
                    "note": "click",
                    "window": 1,
                    "frame_id": 10
                }]
            }
        });

        let out = build_script_step_index_payload(&idx, &script).expect("script markers");
        let steps = out["steps"].as_array().expect("steps");
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0]["step_index"].as_u64(), Some(3));
        assert_eq!(steps[0]["window"].as_u64(), Some(1));
        assert_eq!(steps[0]["frame_id"].as_u64(), Some(10));
        assert_eq!(steps[0]["window_snapshot_seq"].as_u64(), Some(7));
        assert_eq!(steps[0]["semantics_fingerprint"].as_u64(), Some(42));
        assert_eq!(steps[0]["resolve_mode"].as_str(), Some("frame_id"));
    }

    #[test]
    fn ensure_bundle_meta_json_regenerates_when_bundle_label_mismatch() {
        let root = unique_temp_root("fret-diag-ensure-bundle-meta");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let bundle_path = root.join("bundle.json");
        std::fs::write(&bundle_path, "{\"schema_version\":2,\"windows\":[]}")
            .expect("write bundle.json");

        let meta_path = root.join("bundle.meta.json");
        std::fs::write(
            &meta_path,
            serde_json::to_vec_pretty(&json!({
                "kind": "bundle_meta",
                "schema_version": 1,
                "bundle": "some/other/path/bundle.json",
                "warmup_frames": 0,
                "windows_total": 0,
                "snapshots_total": 0,
                "windows": [],
            }))
            .expect("encode meta"),
        )
        .expect("write bundle.meta.json");

        let out = ensure_bundle_meta_json(&bundle_path, 0).expect("ensure meta");
        assert_eq!(out, meta_path);

        let v = read_json_value(&meta_path).expect("read meta");
        let expected = bundle_path.display().to_string();
        assert_eq!(
            v.get("bundle").and_then(|v| v.as_str()),
            Some(expected.as_str())
        );
    }
}
