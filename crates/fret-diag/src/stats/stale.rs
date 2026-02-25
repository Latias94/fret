use std::path::Path;

use crate::util::{now_unix_ms, write_json_value};

use super::semantics::{
    semantics_diff_detail, semantics_diff_summary, semantics_node_fields_for_test_id,
    semantics_node_y_for_test_id,
};

pub(crate) fn check_bundle_for_stale_paint(
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_stale_paint_json(&bundle, bundle_path, test_id, eps)
}

pub(crate) fn check_bundle_for_stale_paint_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut suspicious: Vec<String> = Vec::new();
    let mut missing_scene_fingerprint = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_y: Option<f64> = None;
        let mut prev_fp: Option<u64> = None;
        for s in snaps {
            let y = semantics_node_y_for_test_id(&semantics, s, test_id);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }
            let (Some(y), Some(fp)) = (y, fp) else {
                prev_y = y;
                prev_fp = fp;
                continue;
            };

            if let (Some(prev_y), Some(prev_fp)) = (prev_y, prev_fp)
                && (y - prev_y).abs() >= eps as f64
                && fp == prev_fp
            {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let paint_nodes_performed = s
                    .get("debug")
                    .and_then(|v| v.get("stats"))
                    .and_then(|v| v.get("paint_nodes_performed"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let paint_replayed_ops = s
                    .get("debug")
                    .and_then(|v| v.get("stats"))
                    .and_then(|v| v.get("paint_cache_replayed_ops"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                suspicious.push(format!(
                    "window={window_id} tick={tick_id} frame={frame_id} test_id={test_id} delta_y={:.2} scene_fingerprint=0x{:016x} paint_nodes_performed={paint_nodes_performed} paint_cache_replayed_ops={paint_replayed_ops}",
                    y - prev_y,
                    fp
                ));
                if suspicious.len() >= 8 {
                    break;
                }
            }

            prev_y = Some(y);
            prev_fp = Some(fp);
        }
    }

    if missing_scene_fingerprint {
        return Err(format!(
            "stale paint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale paint suspected (semantics bounds moved but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in suspicious {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(crate) fn check_bundle_for_stale_scene(
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_stale_scene_json(&bundle, bundle_path, test_id, eps)
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SemanticsChangedRepaintedScan {
    missing_scene_fingerprint: bool,
    missing_semantics_fingerprint: bool,
    suspicious_lines: Vec<String>,
    pub(crate) findings: Vec<serde_json::Value>,
}

pub(crate) fn check_bundle_for_semantics_changed_repainted(
    bundle_path: &Path,
    warmup_frames: u64,
    dump_json: bool,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let scan = scan_semantics_changed_repainted_json(&bundle, warmup_frames);
    if dump_json && !scan.findings.is_empty() {
        let out_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
        let out_path = out_dir.join("check.semantics_changed_repainted.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "semantics_changed_repainted",
            "bundle_json": bundle_path.display().to_string(),
            "warmup_frames": warmup_frames,
            "findings": scan.findings,
        });
        let _ = write_json_value(&out_path, &payload);
    }

    check_bundle_for_semantics_changed_repainted_json(&bundle, bundle_path, warmup_frames)
}

pub(crate) fn check_bundle_for_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let scan = scan_semantics_changed_repainted_json(bundle, warmup_frames);

    if scan.missing_scene_fingerprint {
        return Err(format!(
            "semantics repaint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.missing_semantics_fingerprint {
        return Err(format!(
            "semantics repaint check requires `semantics_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.suspicious_lines.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "missing repaint suspected (semantics fingerprint changed but scene fingerprint did not)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in scan.suspicious_lines {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(crate) fn scan_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    warmup_frames: u64,
) -> SemanticsChangedRepaintedScan {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    if windows.is_empty() {
        return SemanticsChangedRepaintedScan::default();
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);
    let mut scan = SemanticsChangedRepaintedScan::default();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_scene_fingerprint: Option<u64> = None;
        let mut prev_semantics_fingerprint: Option<u64> = None;
        let mut prev_tick_id: u64 = 0;
        let mut prev_frame_id: u64 = 0;
        let mut prev_snapshot: Option<&serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let scene_fingerprint = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if scene_fingerprint.is_none() {
                scan.missing_scene_fingerprint = true;
            }

            let semantics_fingerprint = s.get("semantics_fingerprint").and_then(|v| v.as_u64());
            if semantics_fingerprint.is_none() {
                scan.missing_semantics_fingerprint = true;
            }

            let (Some(scene_fingerprint), Some(semantics_fingerprint)) =
                (scene_fingerprint, semantics_fingerprint)
            else {
                prev_scene_fingerprint = None;
                prev_semantics_fingerprint = None;
                prev_tick_id = tick_id;
                prev_frame_id = frame_id;
                prev_snapshot = Some(s);
                continue;
            };

            if let (Some(prev_scene), Some(prev_sem)) =
                (prev_scene_fingerprint, prev_semantics_fingerprint)
            {
                let semantics_changed = semantics_fingerprint != prev_sem;
                let scene_unchanged = scene_fingerprint == prev_scene;
                if semantics_changed && scene_unchanged {
                    let paint_nodes_performed = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_nodes_performed"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let paint_cache_replayed_ops = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_cache_replayed_ops"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    let diff_detail = prev_snapshot
                        .map(|prev| semantics_diff_detail(&semantics, prev, s))
                        .unwrap_or(serde_json::Value::Null);

                    scan.findings.push(serde_json::json!({
                        "window": window_id,
                        "prev": {
                            "tick_id": prev_tick_id,
                            "frame_id": prev_frame_id,
                            "scene_fingerprint": prev_scene,
                            "semantics_fingerprint": prev_sem,
                        },
                        "now": {
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "scene_fingerprint": scene_fingerprint,
                            "semantics_fingerprint": semantics_fingerprint,
                        },
                        "paint_nodes_performed": paint_nodes_performed,
                        "paint_cache_replayed_ops": paint_cache_replayed_ops,
                        "semantics_diff": diff_detail,
                    }));

                    let mut detail = String::new();
                    if let Some(prev) = prev_snapshot {
                        let diff = semantics_diff_summary(&semantics, prev, s);
                        if !diff.is_empty() {
                            detail.push(' ');
                            detail.push_str(&diff);
                        }
                    }

                    scan.suspicious_lines.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} prev_tick={prev_tick_id} prev_frame={prev_frame_id} semantics_fingerprint=0x{semantics_fingerprint:016x} prev_semantics_fingerprint=0x{prev_sem:016x} scene_fingerprint=0x{scene_fingerprint:016x} paint_nodes_performed={paint_nodes_performed} paint_cache_replayed_ops={paint_cache_replayed_ops}{detail}"
                    ));
                    if scan.suspicious_lines.len() >= 8 {
                        break;
                    }
                }
            }

            prev_scene_fingerprint = Some(scene_fingerprint);
            prev_semantics_fingerprint = Some(semantics_fingerprint);
            prev_tick_id = tick_id;
            prev_frame_id = frame_id;
            prev_snapshot = Some(s);
        }
    }

    scan
}

#[derive(Debug, Clone)]
struct IdleNoPaintWindowReport {
    window: u64,
    examined_snapshots: u64,
    idle_frames_total: u64,
    paint_frames_total: u64,
    idle_streak_max: u64,
    idle_streak_tail: u64,
    last_paint: Option<serde_json::Value>,
}

fn snapshot_is_idle_no_paint(snapshot: &serde_json::Value) -> bool {
    let stats = snapshot.get("debug").and_then(|v| v.get("stats"));
    let prepaint_time_us = stats
        .and_then(|v| v.get("prepaint_time_us"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let paint_time_us = stats
        .and_then(|v| v.get("paint_time_us"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let paint_nodes_performed = stats
        .and_then(|v| v.get("paint_nodes_performed"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    prepaint_time_us == 0 && paint_time_us == 0 && paint_nodes_performed == 0
}

pub(crate) fn check_bundle_for_idle_no_paint_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_idle_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

    let mut reports: Vec<IdleNoPaintWindowReport> = Vec::new();
    let mut failures: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut examined_snapshots: u64 = 0;
        let mut idle_frames_total: u64 = 0;
        let mut paint_frames_total: u64 = 0;
        let mut idle_streak: u64 = 0;
        let mut idle_streak_max: u64 = 0;
        let mut last_paint: Option<serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let is_idle = snapshot_is_idle_no_paint(s);
            if is_idle {
                idle_frames_total = idle_frames_total.saturating_add(1);
                idle_streak = idle_streak.saturating_add(1);
                idle_streak_max = idle_streak_max.max(idle_streak);
            } else {
                paint_frames_total = paint_frames_total.saturating_add(1);
                idle_streak = 0;

                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let stats = s.get("debug").and_then(|v| v.get("stats"));
                let prepaint_time_us = stats
                    .and_then(|v| v.get("prepaint_time_us"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let paint_time_us = stats
                    .and_then(|v| v.get("paint_time_us"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let paint_nodes_performed = stats
                    .and_then(|v| v.get("paint_nodes_performed"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                last_paint = Some(serde_json::json!({
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "prepaint_time_us": prepaint_time_us,
                    "paint_time_us": paint_time_us,
                    "paint_nodes_performed": paint_nodes_performed,
                }));
            }
        }

        reports.push(IdleNoPaintWindowReport {
            window,
            examined_snapshots,
            idle_frames_total,
            paint_frames_total,
            idle_streak_max,
            idle_streak_tail: idle_streak,
            last_paint: last_paint.clone(),
        });

        let mut fail_reason: Option<&'static str> = None;
        if min_idle_frames > 0 && examined_snapshots < min_idle_frames {
            fail_reason = Some("insufficient_snapshots");
        } else if min_idle_frames > 0 && idle_streak < min_idle_frames {
            fail_reason = Some("idle_tail_streak_too_small");
        }

        if let Some(reason) = fail_reason {
            failures.push(serde_json::json!({
                "window": window,
                "reason": reason,
                "examined_snapshots": examined_snapshots,
                "idle_streak_tail": idle_streak,
                "idle_streak_max": idle_streak_max,
                "idle_frames_total": idle_frames_total,
                "paint_frames_total": paint_frames_total,
                "last_paint": last_paint,
            }));
        }
    }

    let out_path = out_dir.join("check.idle_no_paint.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "idle_no_paint",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_idle_frames": min_idle_frames,
        "windows": reports.iter().map(|r| serde_json::json!({
            "window": r.window,
            "examined_snapshots": r.examined_snapshots,
            "idle_frames_total": r.idle_frames_total,
            "paint_frames_total": r.paint_frames_total,
            "idle_streak_max": r.idle_streak_max,
            "idle_streak_tail": r.idle_streak_tail,
            "last_paint": r.last_paint,
        })).collect::<Vec<_>>(),
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    if payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|v| v.is_empty())
        .unwrap_or(true)
    {
        return Ok(());
    }

    Err(format!(
        "idle no-paint gate failed (min_idle_frames={min_idle_frames}, warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

pub(crate) fn check_bundle_for_stale_scene_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut suspicious: Vec<String> = Vec::new();
    let mut missing_scene_fingerprint = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_y: Option<f64> = None;
        let mut prev_label: Option<String> = None;
        let mut prev_value: Option<String> = None;
        let mut prev_fp: Option<u64> = None;

        for s in snaps {
            let (y, label, value) = semantics_node_fields_for_test_id(&semantics, s, test_id);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }

            let Some(fp) = fp else {
                prev_y = y;
                prev_label = label;
                prev_value = value;
                prev_fp = None;
                continue;
            };

            if let (Some(prev_fp), Some(prev_y)) = (prev_fp, prev_y) {
                let moved = y
                    .zip(Some(prev_y))
                    .is_some_and(|(y, prev_y)| (y - prev_y).abs() >= eps as f64);
                let label_changed = label.as_deref() != prev_label.as_deref();
                let value_changed = value.as_deref() != prev_value.as_deref();
                let changed = moved || label_changed || value_changed;

                if changed && fp == prev_fp {
                    let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let label_len_prev = prev_label.as_deref().map(|s| s.len()).unwrap_or(0);
                    let label_len_now = label.as_deref().map(|s| s.len()).unwrap_or(0);
                    let value_len_prev = prev_value.as_deref().map(|s| s.len()).unwrap_or(0);
                    let value_len_now = value.as_deref().map(|s| s.len()).unwrap_or(0);
                    let delta_y = y
                        .zip(Some(prev_y))
                        .map(|(y, prev_y)| y - prev_y)
                        .unwrap_or(0.0);
                    suspicious.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} test_id={test_id} changed={{moved={moved} label={label_changed} value={value_changed}}} delta_y={delta_y:.2} label_len={label_len_prev}->{label_len_now} value_len={value_len_prev}->{value_len_now} scene_fingerprint=0x{fp:016x}",
                    ));
                    if suspicious.len() >= 8 {
                        break;
                    }
                }
            }

            prev_y = y;
            prev_label = label;
            prev_value = value;
            prev_fp = Some(fp);
        }
    }

    if missing_scene_fingerprint {
        return Err(format!(
            "stale scene check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale scene suspected (semantics changed but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in suspicious {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}
