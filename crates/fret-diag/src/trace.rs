use std::path::Path;

use serde_json::Value;
use std::collections::BTreeSet;

use crate::perf_keys;

const REAL_SPAN_EXTENSION_KEY_V1: &str = "fret.perf.spans.v1";
const MAX_REAL_SPANS_PER_SNAPSHOT: usize = 512;
const MAX_TRACE_LABEL_BYTES: usize = 160;

const LAYOUT_SYNTHETIC_SUBPHASE_KEYS: &[perf_keys::PerfKey] = &[
    perf_keys::LAYOUT_COLLECT_ROOTS_TIME_US,
    perf_keys::LAYOUT_INVALIDATE_SCROLL_HANDLE_BINDINGS_TIME_US,
    perf_keys::LAYOUT_EXPAND_VIEW_CACHE_INVALIDATIONS_TIME_US,
    perf_keys::LAYOUT_REQUEST_BUILD_ROOTS_TIME_US,
    perf_keys::LAYOUT_REQUEST_BUILD_ROOTS_TAKE_ENGINE_TIME_US,
    perf_keys::LAYOUT_REQUEST_BUILD_ROOTS_PHASE1_TIME_US,
    perf_keys::LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_TIME_US,
    perf_keys::LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_CLEAN_GEOMETRY_PROOF_TIME_US,
    perf_keys::LAYOUT_REQUEST_BUILD_ROOTS_PHASE2_COMPUTE_TIME_US,
    perf_keys::LAYOUT_REQUEST_BUILD_ROOTS_PUT_ENGINE_TIME_US,
    perf_keys::LAYOUT_ENGINE_SOLVE_TIME_US,
    perf_keys::LAYOUT_ROOTS_TIME_US,
    perf_keys::LAYOUT_ROOTS_APPLY_TIME_US,
    perf_keys::LAYOUT_ROOTS_FLUSH_VIEWPORT_TIME_US,
    perf_keys::LAYOUT_PENDING_BARRIER_RELAYOUTS_TIME_US,
    perf_keys::LAYOUT_VIEW_CACHE_TIME_US,
    perf_keys::LAYOUT_REPAIR_VIEW_CACHE_BOUNDS_TIME_US,
    perf_keys::LAYOUT_CONTAINED_VIEW_CACHE_ROOTS_TIME_US,
    perf_keys::LAYOUT_OBSERVATION_RECORD_TIME_US,
    perf_keys::LAYOUT_FOCUS_REPAIR_TIME_US,
    perf_keys::LAYOUT_SEMANTICS_REFRESH_TIME_US,
    perf_keys::LAYOUT_DEFERRED_CLEANUP_TIME_US,
];

const PAINT_SYNTHETIC_SUBPHASE_KEYS: &[perf_keys::PerfKey] = &[
    perf_keys::PAINT_INPUT_CONTEXT_TIME_US,
    perf_keys::PAINT_SCROLL_HANDLE_INVALIDATION_TIME_US,
    perf_keys::PAINT_COLLECT_ROOTS_TIME_US,
    perf_keys::PAINT_RECORD_VISUAL_BOUNDS_TIME_US,
    perf_keys::PAINT_CACHE_KEY_TIME_US,
    perf_keys::PAINT_CACHE_HIT_CHECK_TIME_US,
    perf_keys::PAINT_CACHE_REPLAY_TIME_US,
    perf_keys::PAINT_CACHE_BOUNDS_TRANSLATE_TIME_US,
    perf_keys::PAINT_WIDGET_TIME_US,
    perf_keys::PAINT_TEXT_PREPARE_TIME_US,
    perf_keys::PAINT_PUBLISH_TEXT_INPUT_SNAPSHOT_TIME_US,
    perf_keys::PAINT_OBSERVATION_RECORD_TIME_US,
];

pub(crate) fn write_chrome_trace_from_bundle_path(
    bundle_path: &Path,
    out_path: &Path,
) -> Result<(), String> {
    let trace = chrome_trace_json_from_bundle_path(bundle_path)?;
    write_chrome_trace_value(out_path, &trace)
}

pub(crate) fn chrome_trace_json_from_bundle_path(bundle_path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    chrome_trace_json_from_bundle_value(&bundle)
}

pub(crate) fn write_chrome_trace_value(out_path: &Path, trace: &Value) -> Result<(), String> {
    write_json_value_compact(out_path, trace)
}

fn write_json_value_compact(path: &Path, v: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec(v).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

fn chrome_trace_json_from_bundle_value(bundle: &Value) -> Result<Value, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    let source_bundle_schema_version = bundle.get("schema_version").and_then(|v| v.as_u64());

    let mut events: Vec<Value> = Vec::new();
    let pid: u32 = 1;
    let mut fallback_frame_start_us: u64 = 0;
    let mut real_span_extension_keys: BTreeSet<String> = BTreeSet::new();
    let mut real_span_event_count: u64 = 0;

    for w in windows {
        let window_id_u64 = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let tid = (window_id_u64.min(u32::MAX as u64)) as u32;

        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let (frame_start_us, frame_end_us_hint) =
                snapshot_frame_window_us(s, fallback_frame_start_us);
            fallback_frame_start_us = fallback_frame_start_us.saturating_add(16_000);

            let stats = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.as_object());

            let total_time_us = perf_keys::read_u64(stats, perf_keys::TOTAL_TIME_US);
            let layout_time_us = perf_keys::read_u64(stats, perf_keys::LAYOUT_TIME_US);
            let prepaint_time_us = perf_keys::read_u64(stats, perf_keys::PREPAINT_TIME_US);
            let paint_time_us = perf_keys::read_u64(stats, perf_keys::PAINT_TIME_US);
            let dispatch_time_us = perf_keys::read_u64(stats, perf_keys::DISPATCH_TIME_US);
            let hit_test_time_us = perf_keys::read_u64(stats, perf_keys::HIT_TEST_TIME_US);
            let ui_thread_cpu_time_us =
                perf_keys::read_u64(stats, perf_keys::UI_THREAD_CPU_TIME_US);
            let ui_thread_cpu_cycle_time_delta_cycles =
                perf_keys::read_u64(stats, perf_keys::UI_THREAD_CPU_CYCLE_TIME_DELTA_CYCLES);
            let ui_thread_cpu_cycle_time_total_cycles =
                perf_keys::read_u64(stats, perf_keys::UI_THREAD_CPU_CYCLE_TIME_TOTAL_CYCLES);

            let phase_sum_us = dispatch_time_us
                .saturating_add(hit_test_time_us)
                .saturating_add(layout_time_us)
                .saturating_add(prepaint_time_us)
                .saturating_add(paint_time_us);
            let frame_dur_us = total_time_us.max(phase_sum_us);
            let frame_ts_us = if frame_dur_us > 0 {
                frame_start_us.min(frame_end_us_hint.saturating_sub(frame_dur_us))
            } else {
                frame_start_us
            };
            if frame_dur_us == 0 {
                real_span_event_count =
                    real_span_event_count.saturating_add(push_real_span_extension_events(
                        &mut events,
                        pid,
                        tid,
                        frame_ts_us,
                        tick_id,
                        frame_id,
                        window_id_u64,
                        s,
                        &mut real_span_extension_keys,
                    ));
                continue;
            }

            let frame_wall_us = frame_end_us_hint.saturating_sub(frame_start_us);
            let cpu_pct_denom_us = if frame_wall_us > 0 {
                frame_wall_us
            } else {
                frame_dur_us
            };

            events.push(chrome_x(
                perf_keys::TOTAL_TIME_US.trace_event_name(),
                perf_keys::TOTAL_TIME_US.trace_category_name(),
                pid,
                tid,
                frame_ts_us,
                frame_dur_us,
                serde_json::json!({
                    "window": window_id_u64,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "ui_thread_cpu_time_us": ui_thread_cpu_time_us,
                    "ui_thread_cpu_cycle_time_delta_cycles": ui_thread_cpu_cycle_time_delta_cycles,
                    "ui_thread_cpu_cycle_time_total_cycles": ui_thread_cpu_cycle_time_total_cycles,
                    "ui_thread_cpu_pct_of_wall": if frame_dur_us > 0 {
                        (ui_thread_cpu_time_us as f64) * 100.0 / (cpu_pct_denom_us as f64)
                    } else {
                        0.0
                    },
                }),
            ));

            let mut remaining = frame_dur_us;
            let mut cursor = frame_ts_us;
            cursor = push_phase(
                &mut events,
                pid,
                tid,
                cursor,
                &mut remaining,
                perf_keys::DISPATCH_TIME_US.trace_event_name(),
                perf_keys::DISPATCH_TIME_US.trace_category_name(),
                dispatch_time_us,
            );
            cursor = push_phase(
                &mut events,
                pid,
                tid,
                cursor,
                &mut remaining,
                perf_keys::HIT_TEST_TIME_US.trace_event_name(),
                perf_keys::HIT_TEST_TIME_US.trace_category_name(),
                hit_test_time_us,
            );

            let layout_ts = cursor;
            let layout_dur_us = layout_time_us.min(remaining);
            cursor = push_phase(
                &mut events,
                pid,
                tid,
                cursor,
                &mut remaining,
                perf_keys::LAYOUT_TIME_US.trace_event_name(),
                perf_keys::LAYOUT_TIME_US.trace_category_name(),
                layout_time_us,
            );
            if layout_dur_us > 0 {
                push_registered_subphases(
                    &mut events,
                    pid,
                    tid,
                    layout_ts,
                    layout_dur_us,
                    stats,
                    LAYOUT_SYNTHETIC_SUBPHASE_KEYS,
                );
            }
            push_frame_arg_phase(
                &mut events,
                pid,
                tid,
                layout_ts,
                layout_dur_us,
                stats,
                perf_keys::LAYOUT_OBSERVATION_RECORD_TIME_US,
                tick_id,
                frame_id,
            );

            cursor = push_phase(
                &mut events,
                pid,
                tid,
                cursor,
                &mut remaining,
                perf_keys::PREPAINT_TIME_US.trace_event_name(),
                perf_keys::PREPAINT_TIME_US.trace_category_name(),
                prepaint_time_us,
            );

            let paint_ts = cursor;
            let paint_dur_us = paint_time_us.min(remaining);
            cursor = push_phase(
                &mut events,
                pid,
                tid,
                cursor,
                &mut remaining,
                perf_keys::PAINT_TIME_US.trace_event_name(),
                perf_keys::PAINT_TIME_US.trace_category_name(),
                paint_time_us,
            );
            if paint_dur_us > 0 {
                push_registered_subphases(
                    &mut events,
                    pid,
                    tid,
                    paint_ts,
                    paint_dur_us,
                    stats,
                    PAINT_SYNTHETIC_SUBPHASE_KEYS,
                );
            }
            push_frame_arg_phase(
                &mut events,
                pid,
                tid,
                paint_ts,
                paint_dur_us,
                stats,
                perf_keys::PAINT_OBSERVATION_RECORD_TIME_US,
                tick_id,
                frame_id,
            );
            push_frame_arg_phase(
                &mut events,
                pid,
                tid,
                paint_ts,
                paint_dur_us,
                stats,
                perf_keys::PAINT_TEXT_PREPARE_TIME_US,
                tick_id,
                frame_id,
            );

            if remaining > 0 {
                let desired = remaining;
                let _ = push_phase(
                    &mut events,
                    pid,
                    tid,
                    cursor,
                    &mut remaining,
                    "other",
                    "other",
                    desired,
                );
            }

            real_span_event_count =
                real_span_event_count.saturating_add(push_real_span_extension_events(
                    &mut events,
                    pid,
                    tid,
                    frame_ts_us,
                    tick_id,
                    frame_id,
                    window_id_u64,
                    s,
                    &mut real_span_extension_keys,
                ));
        }
    }

    let real_spans_included = real_span_event_count > 0;
    let trace_source = if real_spans_included {
        crate::perf_schema::PERF_TRACE_SOURCE_BUNDLE_SYNTHETIC_PHASES_WITH_EXTENSION_SPANS
    } else {
        crate::perf_schema::PERF_TRACE_SOURCE_BUNDLE_SYNTHETIC_PHASES
    };

    Ok(serde_json::json!({
        "schema_version": crate::perf_schema::PERF_TRACE_SCHEMA_VERSION,
        "kind": crate::perf_schema::PERF_TRACE_CHROME_KIND,
        "schema_policy": crate::perf_schema::schema_policy_json(),
        "source_bundle_schema_version": source_bundle_schema_version,
        "trace_source": trace_source,
        "real_spans_included": real_spans_included,
        "real_span_extension_keys": real_span_extension_keys.into_iter().collect::<Vec<_>>(),
        "real_span_event_count": real_span_event_count,
        "registered_perf_keys": perf_keys::trace_exported_frame_keys_json(),
        "displayTimeUnit": "ms",
        "traceEvents": events,
    }))
}

fn chrome_x(
    name: &str,
    cat: &str,
    pid: u32,
    tid: u32,
    ts_us: u64,
    dur_us: u64,
    args: Value,
) -> Value {
    serde_json::json!({
        "name": name,
        "cat": cat,
        "ph": "X",
        "ts": ts_us,
        "dur": dur_us,
        "pid": pid,
        "tid": tid,
        "args": args,
    })
}

#[allow(clippy::too_many_arguments)]
fn push_real_span_extension_events(
    events: &mut Vec<Value>,
    pid: u32,
    default_tid: u32,
    frame_ts_us: u64,
    tick_id: u64,
    frame_id: u64,
    window_id: u64,
    snapshot: &Value,
    extension_keys: &mut BTreeSet<String>,
) -> u64 {
    let Some(payload) = snapshot
        .pointer("/debug/extensions")
        .and_then(|extensions| extensions.get(REAL_SPAN_EXTENSION_KEY_V1))
    else {
        return 0;
    };
    if payload
        .get("_clipped")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return 0;
    }
    let Some(spans) = payload.get("spans").and_then(|v| v.as_array()) else {
        return 0;
    };

    let mut pushed = 0_u64;
    for (span_index, span) in spans.iter().take(MAX_REAL_SPANS_PER_SNAPSHOT).enumerate() {
        let Some(dur_us) = span.get("dur_us").and_then(|v| v.as_u64()) else {
            continue;
        };
        if dur_us == 0 {
            continue;
        }

        let start_us = span
            .get("start_us")
            .or_else(|| span.get("ts_us"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let name = bounded_trace_label(span.get("name").and_then(|v| v.as_str()), "fret.real_span");
        let cat = bounded_trace_label(
            span.get("cat")
                .or_else(|| span.get("category"))
                .and_then(|v| v.as_str()),
            "real_span",
        );
        let tid = span
            .get("tid")
            .and_then(|v| v.as_u64())
            .map(|tid| (tid.min(u32::MAX as u64)) as u32)
            .unwrap_or(default_tid);
        let span_args = span.get("args").cloned().unwrap_or(Value::Null);

        events.push(chrome_x(
            &name,
            &cat,
            pid,
            tid,
            frame_ts_us.saturating_add(start_us),
            dur_us,
            serde_json::json!({
                "source": "debug.extensions",
                "extension_key": REAL_SPAN_EXTENSION_KEY_V1,
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "span_index": span_index,
                "span_args": span_args,
            }),
        ));
        pushed = pushed.saturating_add(1);
    }

    if pushed > 0 {
        extension_keys.insert(REAL_SPAN_EXTENSION_KEY_V1.to_string());
    }
    pushed
}

fn bounded_trace_label(value: Option<&str>, fallback: &str) -> String {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return fallback.to_string();
    };
    if value.len() <= MAX_TRACE_LABEL_BYTES {
        return value.to_string();
    }

    let mut end = MAX_TRACE_LABEL_BYTES;
    while !value.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    format!("{}...", &value[..end])
}

#[allow(clippy::too_many_arguments)]
fn push_phase(
    events: &mut Vec<Value>,
    pid: u32,
    tid: u32,
    cursor_us: u64,
    remaining_us: &mut u64,
    name: &'static str,
    cat: &'static str,
    desired_us: u64,
) -> u64 {
    if desired_us == 0 || *remaining_us == 0 {
        return cursor_us;
    }
    let dur = desired_us.min(*remaining_us);
    *remaining_us = remaining_us.saturating_sub(dur);
    events.push(chrome_x(name, cat, pid, tid, cursor_us, dur, Value::Null));
    cursor_us.saturating_add(dur)
}

#[allow(clippy::too_many_arguments)]
fn push_frame_arg_phase(
    events: &mut Vec<Value>,
    pid: u32,
    tid: u32,
    ts_us: u64,
    max_dur_us: u64,
    stats: Option<&serde_json::Map<String, Value>>,
    key: perf_keys::PerfKey,
    tick_id: u64,
    frame_id: u64,
) {
    let desired_us = perf_keys::read_u64(stats, key);
    if desired_us == 0 || max_dur_us == 0 {
        return;
    }

    events.push(chrome_x(
        key.trace_event_name(),
        key.trace_category_name(),
        pid,
        tid,
        ts_us,
        desired_us.min(max_dur_us),
        serde_json::json!({
            "tick_id": tick_id,
            "frame_id": frame_id,
        }),
    ));
}

fn push_registered_subphases(
    events: &mut Vec<Value>,
    pid: u32,
    tid: u32,
    parent_ts_us: u64,
    parent_dur_us: u64,
    stats: Option<&serde_json::Map<String, Value>>,
    phases: &[perf_keys::PerfKey],
) {
    let mut cursor = parent_ts_us;
    let mut remaining = parent_dur_us;
    for key in phases {
        let desired_us = perf_keys::read_u64(stats, *key);
        cursor = push_phase(
            events,
            pid,
            tid,
            cursor,
            &mut remaining,
            key.trace_event_name(),
            key.trace_category_name(),
            desired_us,
        );
        if remaining == 0 {
            break;
        }
    }
}

fn snapshot_frame_window_us(s: &Value, fallback_start_us: u64) -> (u64, u64) {
    let ts_unix_ms = s
        .get("timestamp_unix_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let mut end_us_hint = ts_unix_ms.saturating_mul(1_000);

    if let Some(frame_clock) = s.get("frame_clock") {
        let now_ms = frame_clock
            .get("now_monotonic_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let delta_ms = frame_clock
            .get("delta_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        end_us_hint = now_ms.saturating_mul(1_000);
        let start_us = now_ms.saturating_sub(delta_ms).saturating_mul(1_000);
        return (start_us, end_us_hint);
    }

    if end_us_hint == 0 {
        end_us_hint = fallback_start_us.saturating_add(16_000);
    }
    let start_us = end_us_hint.saturating_sub(16_000);
    (start_us, end_us_hint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn trace_event_names(trace: &Value) -> BTreeSet<&str> {
        trace
            .get("traceEvents")
            .and_then(|v| v.as_array())
            .expect("trace events")
            .iter()
            .filter_map(|e| e.get("name").and_then(|v| v.as_str()))
            .collect()
    }

    #[test]
    fn chrome_trace_includes_trace_events() {
        let bundle = serde_json::json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "window": 1,
                    "timestamp_unix_ms": 123,
                    "frame_clock": { "now_monotonic_ms": 1000, "delta_ms": 16 },
                    "debug": { "stats": {
                        "total_time_us": 1000,
                        "layout_time_us": 400,
                        "layout_collect_roots_time_us": 50,
                        "layout_engine_solve_time_us": 100,
                        "prepaint_time_us": 100,
                        "paint_time_us": 500
                    } }
                }]
            }]
        });

        let trace = chrome_trace_json_from_bundle_value(&bundle).expect("trace");
        assert_eq!(
            trace.get("kind").and_then(|v| v.as_str()),
            Some(crate::perf_schema::PERF_TRACE_CHROME_KIND)
        );
        assert_eq!(
            trace.get("schema_version").and_then(|v| v.as_u64()),
            Some(crate::perf_schema::PERF_TRACE_SCHEMA_VERSION as u64)
        );
        assert_eq!(
            trace
                .get("schema_policy")
                .and_then(|v| v.get("compatibility"))
                .and_then(|v| v.as_str()),
            Some("additive_only")
        );
        assert_eq!(
            trace
                .get("source_bundle_schema_version")
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            trace.get("trace_source").and_then(|v| v.as_str()),
            Some(crate::perf_schema::PERF_TRACE_SOURCE_BUNDLE_SYNTHETIC_PHASES)
        );
        assert_eq!(
            trace.get("real_spans_included").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            trace.get("real_span_event_count").and_then(|v| v.as_u64()),
            Some(0)
        );
        let registered_perf_keys = trace
            .get("registered_perf_keys")
            .and_then(|v| v.as_array())
            .expect("registered perf key metadata");
        assert!(registered_perf_keys.iter().any(|key| {
            key.get("key").and_then(|v| v.as_str()) == Some("total_time_us")
                && key.get("unit").and_then(|v| v.as_str()) == Some("us")
                && key.get("trace_event").and_then(|v| v.as_str()) == Some("fret.frame")
        }));
        assert!(
            trace
                .get("traceEvents")
                .and_then(|v| v.as_array())
                .is_some()
        );
        let names = trace_event_names(&trace);
        assert!(names.contains(&"fret.frame"));
        assert!(names.contains(&"layout.collect_roots"));
        assert!(names.contains(&"layout.engine_solve"));
    }

    #[test]
    fn chrome_trace_merges_real_span_extension_events() {
        let bundle = serde_json::json!({
            "schema_version": 2,
            "windows": [{
                "window": 7,
                "snapshots": [{
                    "tick_id": 3,
                    "frame_id": 5,
                    "window": 7,
                    "frame_clock": { "now_monotonic_ms": 1000, "delta_ms": 16 },
                    "debug": {
                        "stats": {
                            "total_time_us": 1000,
                            "layout_time_us": 400,
                            "paint_time_us": 600
                        },
                        "extensions": {
                            "fret.perf.spans.v1": {
                                "schema_version": "v1",
                                "spans": [{
                                    "name": "fret.ui.view",
                                    "cat": "ui.real",
                                    "start_us": 7,
                                    "dur_us": 42,
                                    "args": { "phase": "view" }
                                }]
                            }
                        }
                    }
                }]
            }]
        });

        let trace = chrome_trace_json_from_bundle_value(&bundle).expect("trace");
        assert_eq!(
            trace.get("trace_source").and_then(|v| v.as_str()),
            Some(
                crate::perf_schema::PERF_TRACE_SOURCE_BUNDLE_SYNTHETIC_PHASES_WITH_EXTENSION_SPANS
            )
        );
        assert_eq!(
            trace.get("real_spans_included").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            trace.get("real_span_event_count").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            trace
                .get("real_span_extension_keys")
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_str()),
            Some("fret.perf.spans.v1")
        );

        let real = trace
            .get("traceEvents")
            .and_then(|v| v.as_array())
            .expect("trace events")
            .iter()
            .find(|event| event.get("name").and_then(|v| v.as_str()) == Some("fret.ui.view"))
            .expect("real span event");
        assert_eq!(real.get("cat").and_then(|v| v.as_str()), Some("ui.real"));
        assert_eq!(real.get("ts").and_then(|v| v.as_u64()), Some(984_007));
        assert_eq!(real.get("dur").and_then(|v| v.as_u64()), Some(42));
        assert_eq!(
            real.pointer("/args/extension_key").and_then(|v| v.as_str()),
            Some("fret.perf.spans.v1")
        );
        assert_eq!(
            real.pointer("/args/span_args/phase")
                .and_then(|v| v.as_str()),
            Some("view")
        );
    }

    #[test]
    fn chrome_trace_keeps_real_span_extension_when_synthetic_stats_are_zero() {
        let bundle = serde_json::json!({
            "schema_version": 2,
            "windows": [{
                "window": 7,
                "snapshots": [{
                    "tick_id": 3,
                    "frame_id": 5,
                    "window": 7,
                    "frame_clock": { "now_monotonic_ms": 1000, "delta_ms": 16 },
                    "debug": {
                        "stats": {
                            "total_time_us": 0,
                            "layout_time_us": 0,
                            "paint_time_us": 0
                        },
                        "extensions": {
                            "fret.perf.spans.v1": {
                                "schema_version": "v1",
                                "spans": [{
                                    "name": "fret.ui.view",
                                    "cat": "ui.driver",
                                    "start_us": 7,
                                    "dur_us": 42,
                                    "args": { "phase": "view" }
                                }]
                            }
                        }
                    }
                }]
            }]
        });

        let trace = chrome_trace_json_from_bundle_value(&bundle).expect("trace");
        assert_eq!(
            trace.get("trace_source").and_then(|v| v.as_str()),
            Some(
                crate::perf_schema::PERF_TRACE_SOURCE_BUNDLE_SYNTHETIC_PHASES_WITH_EXTENSION_SPANS
            )
        );
        assert_eq!(
            trace.get("real_spans_included").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            trace.get("real_span_event_count").and_then(|v| v.as_u64()),
            Some(1)
        );
        let names = trace_event_names(&trace);
        assert!(names.contains(&"fret.ui.view"));
        assert!(!names.contains(&"fret.frame"));
    }

    #[test]
    fn chrome_trace_synthetic_ui_subphases_cover_registered_timing_events() {
        let mut stats = serde_json::Map::new();
        for key in perf_keys::TRACE_EXPORTED_FRAME_KEYS {
            if matches!(key.kind, perf_keys::PerfKeyKind::Timing) {
                stats.insert(key.key.to_string(), Value::from(10_u64));
            }
        }
        stats.insert("total_time_us".to_string(), Value::from(2_000_u64));
        stats.insert("layout_time_us".to_string(), Value::from(900_u64));
        stats.insert("paint_time_us".to_string(), Value::from(900_u64));
        stats.insert("dispatch_time_us".to_string(), Value::from(50_u64));
        stats.insert("hit_test_time_us".to_string(), Value::from(50_u64));
        stats.insert("prepaint_time_us".to_string(), Value::from(50_u64));

        let bundle = serde_json::json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "tick_id": 1,
                    "frame_id": 1,
                    "window": 1,
                    "frame_clock": { "now_monotonic_ms": 1000, "delta_ms": 16 },
                    "debug": { "stats": stats }
                }]
            }]
        });

        let trace = chrome_trace_json_from_bundle_value(&bundle).expect("trace");
        let names = trace_event_names(&trace);
        let top_level = [
            perf_keys::LAYOUT_TIME_US.key,
            perf_keys::PAINT_TIME_US.key,
            perf_keys::PREPAINT_TIME_US.key,
            perf_keys::DISPATCH_TIME_US.key,
            perf_keys::HIT_TEST_TIME_US.key,
            perf_keys::TOTAL_TIME_US.key,
            perf_keys::UI_THREAD_CPU_TIME_US.key,
        ];

        let missing: Vec<&str> = perf_keys::TRACE_EXPORTED_FRAME_KEYS
            .iter()
            .filter(|key| {
                matches!(key.kind, perf_keys::PerfKeyKind::Timing)
                    && matches!(key.trace_category_name(), "layout" | "paint")
                    && !top_level.contains(&key.key)
            })
            .filter_map(|key| {
                let event = key.trace_event_name();
                (!names.contains(event)).then_some(event)
            })
            .collect();
        assert!(
            missing.is_empty(),
            "trace-exported layout/paint timing keys missing synthetic events: {missing:?}"
        );
    }
}
