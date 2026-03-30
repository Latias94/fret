use std::collections::HashMap;

use fret_diag_protocol::UiScriptResultV1;
use serde_json::{Value, json};

const SCRIPT_STEP_INDEX_SCHEMA_VERSION: u32 = 1;
const SCRIPT_STEP_INDEX_MAX_TIMESTAMP_DELTA_MS: u64 = 2_000;

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

    let windows = idx
        .get("windows")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    let snaps_empty: &[Value] = &[];
    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(snaps_empty);
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
    if let Some(frame_id) = frame_id
        && let Some(info) = by_window_frame.get(&(window, frame_id))
    {
        return (
            info.window_snapshot_seq,
            info.timestamp_unix_ms,
            info.semantics_source.clone(),
            info.semantics_fingerprint,
            Some("frame_id"),
        );
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

pub(super) fn build_script_step_index_payload(
    idx: &Value,
    script_result: &UiScriptResultV1,
) -> Option<Value> {
    let script_result = serde_json::to_value(script_result).ok()?;
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
