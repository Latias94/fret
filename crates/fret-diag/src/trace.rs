use std::path::Path;

use serde_json::Value;

pub(crate) fn write_chrome_trace_from_bundle_path(
    bundle_path: &Path,
    out_path: &Path,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let trace = chrome_trace_json_from_bundle_value(&bundle)?;
    write_json_value_compact(out_path, &trace)
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

    let mut events: Vec<Value> = Vec::new();
    let pid: u32 = 1;
    let mut fallback_frame_start_us: u64 = 0;

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

            let total_time_us = stats
                .and_then(|m| m.get("total_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_time_us = stats
                .and_then(|m| m.get("layout_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let prepaint_time_us = stats
                .and_then(|m| m.get("prepaint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_time_us = stats
                .and_then(|m| m.get("paint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let dispatch_time_us = stats
                .and_then(|m| m.get("dispatch_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let hit_test_time_us = stats
                .and_then(|m| m.get("hit_test_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let ui_thread_cpu_time_us = stats
                .and_then(|m| m.get("ui_thread_cpu_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let ui_thread_cpu_cycle_time_delta_cycles = stats
                .and_then(|m| m.get("ui_thread_cpu_cycle_time_delta_cycles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let ui_thread_cpu_cycle_time_total_cycles = stats
                .and_then(|m| m.get("ui_thread_cpu_cycle_time_total_cycles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let layout_obs_record_time_us = stats
                .and_then(|m| m.get("layout_observation_record_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_collect_roots_time_us = stats
                .and_then(|m| m.get("layout_collect_roots_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_invalidate_scroll_handle_bindings_time_us = stats
                .and_then(|m| m.get("layout_invalidate_scroll_handle_bindings_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_expand_view_cache_invalidations_time_us = stats
                .and_then(|m| m.get("layout_expand_view_cache_invalidations_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_request_build_roots_time_us = stats
                .and_then(|m| m.get("layout_request_build_roots_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_roots_time_us = stats
                .and_then(|m| m.get("layout_roots_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_view_cache_time_us = stats
                .and_then(|m| m.get("layout_view_cache_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_engine_solve_time_us = stats
                .and_then(|m| m.get("layout_engine_solve_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_obs_record_time_us = stats
                .and_then(|m| m.get("paint_observation_record_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_text_prepare_time_us = stats
                .and_then(|m| m.get("paint_text_prepare_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_record_visual_bounds_time_us = stats
                .and_then(|m| m.get("paint_record_visual_bounds_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_cache_key_time_us = stats
                .and_then(|m| m.get("paint_cache_key_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_cache_hit_check_time_us = stats
                .and_then(|m| m.get("paint_cache_hit_check_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_cache_replay_time_us = stats
                .and_then(|m| m.get("paint_cache_replay_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_cache_bounds_translate_time_us = stats
                .and_then(|m| m.get("paint_cache_bounds_translate_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_widget_time_us = stats
                .and_then(|m| m.get("paint_widget_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let phase_sum_us = dispatch_time_us
                .saturating_add(hit_test_time_us)
                .saturating_add(layout_time_us)
                .saturating_add(prepaint_time_us)
                .saturating_add(paint_time_us);
            let frame_dur_us = total_time_us.max(phase_sum_us);
            if frame_dur_us == 0 {
                continue;
            }

            let frame_wall_us = frame_end_us_hint.saturating_sub(frame_start_us);
            let cpu_pct_denom_us = if frame_wall_us > 0 {
                frame_wall_us
            } else {
                frame_dur_us
            };

            let frame_ts_us = frame_start_us.min(frame_end_us_hint.saturating_sub(frame_dur_us));

            events.push(chrome_x(
                "fret.frame",
                "frame",
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
                "dispatch",
                "dispatch",
                dispatch_time_us,
            );
            cursor = push_phase(
                &mut events,
                pid,
                tid,
                cursor,
                &mut remaining,
                "hit_test",
                "hit_test",
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
                "layout",
                "layout",
                layout_time_us,
            );
            if layout_dur_us > 0 {
                push_subphases(
                    &mut events,
                    pid,
                    tid,
                    layout_ts,
                    layout_dur_us,
                    "layout",
                    &[
                        ("layout.collect_roots", layout_collect_roots_time_us),
                        (
                            "layout.invalidate_scroll_bindings",
                            layout_invalidate_scroll_handle_bindings_time_us,
                        ),
                        (
                            "layout.expand_view_cache_invalidations",
                            layout_expand_view_cache_invalidations_time_us,
                        ),
                        (
                            "layout.request_build_roots",
                            layout_request_build_roots_time_us,
                        ),
                        ("layout.engine_solve", layout_engine_solve_time_us),
                        ("layout.roots", layout_roots_time_us),
                        ("layout.view_cache", layout_view_cache_time_us),
                    ],
                );
            }
            if layout_obs_record_time_us > 0 && layout_time_us > 0 {
                let dur = layout_obs_record_time_us.min(layout_dur_us);
                events.push(chrome_x(
                    "layout.obs_record",
                    "layout",
                    pid,
                    tid,
                    layout_ts,
                    dur,
                    serde_json::json!({
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                    }),
                ));
            }

            cursor = push_phase(
                &mut events,
                pid,
                tid,
                cursor,
                &mut remaining,
                "prepaint",
                "prepaint",
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
                "paint",
                "paint",
                paint_time_us,
            );
            if paint_dur_us > 0 {
                push_subphases(
                    &mut events,
                    pid,
                    tid,
                    paint_ts,
                    paint_dur_us,
                    "paint",
                    &[
                        (
                            "paint.record_visual_bounds",
                            paint_record_visual_bounds_time_us,
                        ),
                        ("paint.cache_key", paint_cache_key_time_us),
                        ("paint.cache_hit_check", paint_cache_hit_check_time_us),
                        ("paint.cache_replay", paint_cache_replay_time_us),
                        (
                            "paint.cache_bounds_translate",
                            paint_cache_bounds_translate_time_us,
                        ),
                        ("paint.widget", paint_widget_time_us),
                        ("paint.text_prepare", paint_text_prepare_time_us),
                        ("paint.obs_record", paint_obs_record_time_us),
                    ],
                );
            }
            if paint_obs_record_time_us > 0 && paint_time_us > 0 {
                let dur = paint_obs_record_time_us.min(paint_dur_us);
                events.push(chrome_x(
                    "paint.obs_record",
                    "paint",
                    pid,
                    tid,
                    paint_ts,
                    dur,
                    serde_json::json!({
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                    }),
                ));
            }
            if paint_text_prepare_time_us > 0 && paint_time_us > 0 {
                let dur = paint_text_prepare_time_us.min(paint_dur_us);
                events.push(chrome_x(
                    "paint.text_prepare",
                    "paint",
                    pid,
                    tid,
                    paint_ts,
                    dur,
                    serde_json::json!({
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                    }),
                ));
            }

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
        }
    }

    Ok(serde_json::json!({
        "schema_version": 1,
        "displayTimeUnit": "ms",
        "traceEvents": events,
    }))
}

fn chrome_x(
    name: &'static str,
    cat: &'static str,
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

fn push_subphases(
    events: &mut Vec<Value>,
    pid: u32,
    tid: u32,
    parent_ts_us: u64,
    parent_dur_us: u64,
    cat: &'static str,
    phases: &[(&'static str, u64)],
) {
    let mut cursor = parent_ts_us;
    let mut remaining = parent_dur_us;
    for (name, desired_us) in phases {
        cursor = push_phase(
            events,
            pid,
            tid,
            cursor,
            &mut remaining,
            name,
            cat,
            *desired_us,
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

    #[test]
    fn chrome_trace_includes_trace_events() {
        let bundle = serde_json::json!({
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
        assert!(
            trace
                .get("traceEvents")
                .and_then(|v| v.as_array())
                .is_some()
        );
        let names = trace
            .get("traceEvents")
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|e| e.get("name").and_then(|v| v.as_str()))
            .collect::<Vec<_>>();
        assert!(names.contains(&"fret.frame"));
        assert!(names.contains(&"layout.collect_roots"));
        assert!(names.contains(&"layout.engine_solve"));
    }
}
