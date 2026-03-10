use std::path::Path;

use super::wheel_scroll::first_wheel_frame_id_for_window;
use crate::util::{now_unix_ms, write_json_value};

pub(crate) fn check_bundle_for_vlist_visible_range_refreshes_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_visible_range_refreshes_max_json(
        &bundle,
        bundle_path,
        out_dir,
        max_total_refreshes,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_vlist_visible_range_refreshes_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_visible_range_refreshes_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_total_refreshes,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_vlist_policy_key_stable(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_policy_key_stable_json(&bundle, bundle_path, out_dir, warmup_frames)
}

pub(crate) fn check_bundle_for_vlist_policy_key_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_signal = false;
    let mut examined_snapshots: u64 = 0;
    let mut by_surface: std::collections::BTreeMap<(u64, u64), std::collections::BTreeSet<u64>> =
        std::collections::BTreeMap::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let vlist_windows = s
                .get("debug")
                .and_then(|v| v.get("virtual_list_windows"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if vlist_windows.is_empty() {
                continue;
            }

            any_signal = true;
            for win in vlist_windows {
                let node = win.get("node").and_then(|v| v.as_u64()).unwrap_or(0);
                let element = win.get("element").and_then(|v| v.as_u64()).unwrap_or(0);
                let policy_key = win.get("policy_key").and_then(|v| v.as_u64()).unwrap_or(0);
                by_surface
                    .entry((node, element))
                    .or_default()
                    .insert(policy_key);
            }
        }
    }

    let offenders: Vec<serde_json::Value> = by_surface
        .iter()
        .filter(|(_, keys)| keys.len() > 1)
        .take(64)
        .map(|((node, element), keys)| {
            serde_json::json!({
                "node": node,
                "element": element,
                "policy_keys": keys.iter().copied().collect::<Vec<u64>>(),
            })
        })
        .collect();

    let out_path = out_dir.join("check.vlist_policy_key_stable.json");
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_policy_key_stable",
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "surfaces_seen": by_surface.len(),
        "offenders": offenders,
    });
    write_json_value(&out_path, &payload)?;

    if !any_signal {
        return Err(format!(
            "vlist policy-key stability gate requires debug.virtual_list_windows after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if offenders.is_empty() {
        return Ok(());
    }

    Err(format!(
        "vlist policy-key stability gate failed (expected each vlist surface to keep a stable policy_key after warmup)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

pub(crate) fn check_bundle_for_vlist_visible_range_refreshes_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut total_refreshes: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let refreshes = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("virtual_list_visible_range_refreshes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if refreshes == 0 {
                continue;
            }
            total_refreshes = total_refreshes.saturating_add(refreshes);
            if samples.len() < 32 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "refreshes": refreshes,
                }));
            }
        }
    }

    let out_path = out_dir.join("check.vlist_visible_range_refreshes_min.json");
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_visible_range_refreshes_min",
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_total_refreshes": min_total_refreshes,
        "any_wheel": any_wheel,
        "total_refreshes": total_refreshes,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_wheel {
        return Err(format!(
            "vlist visible-range refresh gate requires wheel events, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if min_total_refreshes > 0 && total_refreshes < min_total_refreshes {
        return Err(format!(
            "expected virtual list visible-range refreshes to occur after wheel events, but total_refreshes={total_refreshes} was below min_total_refreshes={min_total_refreshes} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_vlist_visible_range_refreshes_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut total_refreshes: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let refreshes = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("virtual_list_visible_range_refreshes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if refreshes == 0 {
                continue;
            }
            total_refreshes = total_refreshes.saturating_add(refreshes);
            if samples.len() < 32 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "refreshes": refreshes,
                }));
            }
        }
    }

    let out_path = out_dir.join("check.vlist_visible_range_refreshes_max.json");
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_visible_range_refreshes_max",
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_total_refreshes": max_total_refreshes,
        "any_wheel": any_wheel,
        "total_refreshes": total_refreshes,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_wheel {
        return Err(format!(
            "vlist visible-range refresh gate requires wheel events, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if max_total_refreshes > 0 && total_refreshes > max_total_refreshes {
        return Err(format!(
            "expected virtual list visible-range refreshes to stay under budget after wheel events, but total_refreshes={total_refreshes} exceeded max_total_refreshes={max_total_refreshes} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_vlist_window_shifts_explainable(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_explainable_json(
        &bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_vlist_window_shifts_explainable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_signal = false;
    let mut total_shifts: u64 = 0;
    let mut offenders: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let wheel_frame = first_wheel_frame_id_for_window(w);
        let after_frame = wheel_frame.unwrap_or(warmup_frames).max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let list = s
                .get("debug")
                .and_then(|v| v.get("virtual_list_windows"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if list.is_empty() {
                continue;
            }
            any_signal = true;

            for win in list {
                let mismatch = win
                    .get("window_mismatch")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let kind = win
                    .get("window_shift_kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or(if mismatch { "escape" } else { "none" });
                if kind == "none" {
                    continue;
                }
                total_shifts = total_shifts.saturating_add(1);

                let reason = win.get("window_shift_reason").and_then(|v| v.as_str());
                let mode = win.get("window_shift_apply_mode").and_then(|v| v.as_str());
                let invalidation_detail = win
                    .get("window_shift_invalidation_detail")
                    .and_then(|v| v.as_str());
                if reason.is_some() && mode.is_some() {
                    if mode == Some("non_retained_rerender") {
                        let expected_detail = match reason {
                            Some("scroll_to_item") => {
                                Some("scroll_handle_scroll_to_item_window_update")
                            }
                            Some("viewport_resize") => {
                                Some("scroll_handle_viewport_resize_window_update")
                            }
                            Some("items_revision") => {
                                Some("scroll_handle_items_revision_window_update")
                            }
                            _ => match kind {
                                "escape" => Some("scroll_handle_window_update"),
                                "prefetch" => Some("scroll_handle_prefetch_window_update"),
                                _ => None,
                            },
                        };
                        if invalidation_detail.is_none() {
                            offenders = offenders.saturating_add(1);
                            failures.push(format!(
                                "window={window_id} tick_id={tick_id} frame_id={frame_id} error=missing_shift_invalidation_detail kind={kind} apply_mode={mode:?}"
                            ));
                            if samples.len() < 64 {
                                samples.push(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "kind": kind,
                                    "reason": reason,
                                    "apply_mode": mode,
                                    "invalidation_detail": invalidation_detail,
                                    "expected_invalidation_detail": expected_detail,
                                    "node": win.get("node").and_then(|v| v.as_u64()),
                                    "element": win.get("element").and_then(|v| v.as_u64()),
                                    "policy_key": win.get("policy_key").and_then(|v| v.as_u64()),
                                    "inputs_key": win.get("inputs_key").and_then(|v| v.as_u64()),
                                }));
                            }
                        } else if expected_detail.is_some()
                            && invalidation_detail != expected_detail
                        {
                            offenders = offenders.saturating_add(1);
                            failures.push(format!(
                                "window={window_id} tick_id={tick_id} frame_id={frame_id} error=unexpected_shift_invalidation_detail kind={kind} got={invalidation_detail:?} expected={expected_detail:?}"
                            ));
                            if samples.len() < 64 {
                                samples.push(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "kind": kind,
                                    "reason": reason,
                                    "apply_mode": mode,
                                    "invalidation_detail": invalidation_detail,
                                    "expected_invalidation_detail": expected_detail,
                                    "node": win.get("node").and_then(|v| v.as_u64()),
                                    "element": win.get("element").and_then(|v| v.as_u64()),
                                    "policy_key": win.get("policy_key").and_then(|v| v.as_u64()),
                                    "inputs_key": win.get("inputs_key").and_then(|v| v.as_u64()),
                                }));
                            }
                        }
                    }
                    continue;
                }

                offenders = offenders.saturating_add(1);
                failures.push(format!(
                    "window={window_id} tick_id={tick_id} frame_id={frame_id} error=missing_shift_explainability kind={kind} reason={reason:?} apply_mode={mode:?} invalidation_detail={invalidation_detail:?}"
                ));

                if samples.len() < 64 {
                    samples.push(serde_json::json!({
                        "window": window_id,
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                        "kind": kind,
                        "reason": reason,
                        "apply_mode": mode,
                        "invalidation_detail": invalidation_detail,
                        "node": win.get("node").and_then(|v| v.as_u64()),
                        "element": win.get("element").and_then(|v| v.as_u64()),
                        "policy_key": win.get("policy_key").and_then(|v| v.as_u64()),
                        "inputs_key": win.get("inputs_key").and_then(|v| v.as_u64()),
                        "window_range": win.get("window_range"),
                        "prev_window_range": win.get("prev_window_range"),
                        "render_window_range": win.get("render_window_range"),
                        "deferred_scroll_to_item": win
                            .get("deferred_scroll_to_item")
                            .and_then(|v| v.as_bool()),
                        "deferred_scroll_consumed": win
                            .get("deferred_scroll_consumed")
                            .and_then(|v| v.as_bool()),
                    }));
                }
            }
        }
    }

    let out_path = out_dir.join("check.vlist_window_shifts_explainable.json");
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_window_shifts_explainable",
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "total_shifts": total_shifts,
        "offenders": offenders,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_signal {
        return Err(format!(
            "vlist window-shift explainability gate requires debug.virtual_list_windows after warmup, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if offenders == 0 {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("vlist window-shift explainability gate failed (expected every window shift to have reason + apply_mode)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!("evidence: {}\n", out_path.display()));
    for line in failures.into_iter().take(12) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(crate) fn check_bundle_for_vlist_window_shifts_non_retained_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_total_non_retained_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_non_retained_max_json(
        &bundle,
        bundle_path,
        out_dir,
        max_total_non_retained_shifts,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_vlist_window_shifts_non_retained_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_total_non_retained_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut snapshots_examined: u64 = 0;
    let mut total_non_retained_shifts: u64 = 0;
    let mut total_shifts: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            snapshots_examined = snapshots_examined.saturating_add(1);

            let debug_stats = s.get("debug").and_then(|v| v.get("stats"));
            let window_shifts_total = debug_stats
                .and_then(|v| v.get("virtual_list_window_shifts_total"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let window_shifts_non_retained = debug_stats
                .and_then(|v| v.get("virtual_list_window_shifts_non_retained"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            total_shifts = total_shifts.saturating_add(window_shifts_total);
            if window_shifts_non_retained == 0 {
                continue;
            }
            total_non_retained_shifts =
                total_non_retained_shifts.saturating_add(window_shifts_non_retained);

            if samples.len() < 64 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let shift_samples = s
                    .get("debug")
                    .and_then(|v| v.get("virtual_list_window_shift_samples"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().take(8).cloned().collect::<Vec<_>>())
                    .unwrap_or_default();

                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "non_retained_shifts": window_shifts_non_retained,
                    "window_shifts_total": window_shifts_total,
                    "shift_samples": shift_samples,
                }));
            }
        }
    }

    let out_path = out_dir.join("check.vlist_window_shifts_non_retained_max.json");
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_window_shifts_non_retained_max",
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_total_non_retained_shifts": max_total_non_retained_shifts,
        "snapshots_examined": snapshots_examined,
        "total_window_shifts": total_shifts,
        "total_non_retained_shifts": total_non_retained_shifts,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if total_non_retained_shifts > max_total_non_retained_shifts {
        return Err(format!(
            "vlist non-retained window-shift gate failed: total_non_retained_shifts={total_non_retained_shifts} exceeded max_total_non_retained_shifts={max_total_non_retained_shifts} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_vlist_window_shifts_kind_max(
    bundle_path: &Path,
    out_dir: &Path,
    kind: &str,
    max_total_kind_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_kind_max_json(
        &bundle,
        bundle_path,
        out_dir,
        kind,
        max_total_kind_shifts,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_vlist_window_shifts_kind_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    kind: &str,
    max_total_kind_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let kind = match kind {
        "prefetch" | "escape" => kind,
        _ => {
            return Err(format!(
                "vlist window-shift kind must be one of: prefetch|escape (got: {kind})"
            ));
        }
    };

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut snapshots_examined: u64 = 0;
    let mut total_kind_shifts: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "invalid bundle artifact: missing snapshots".to_string())?;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            snapshots_examined = snapshots_examined.saturating_add(1);

            let shift_entries = s
                .get("debug")
                .and_then(|v| v.get("virtual_list_windows"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter(|w| {
                            w.get("source")
                                .and_then(|v| v.as_str())
                                .is_some_and(|s| s == "prepaint")
                                && w.get("window_shift_kind")
                                    .and_then(|v| v.as_str())
                                    .is_some_and(|k| k == kind)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if shift_entries.is_empty() {
                continue;
            }

            let mut unique_entries: Vec<&serde_json::Value> = Vec::new();
            type VirtualListShiftKey = (
                Option<u64>,
                Option<u64>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<bool>,
            );
            let mut seen_keys: std::collections::HashSet<VirtualListShiftKey> =
                std::collections::HashSet::new();
            for w in shift_entries {
                let key = (
                    w.get("node").and_then(|v| v.as_u64()),
                    w.get("element").and_then(|v| v.as_u64()),
                    w.get("source")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    w.get("window_shift_kind")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    w.get("window_shift_reason")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    w.get("window_shift_apply_mode")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    w.get("window_mismatch").and_then(|v| v.as_bool()),
                );
                if seen_keys.insert(key) {
                    unique_entries.push(w);
                }
            }
            if unique_entries.is_empty() {
                continue;
            }

            total_kind_shifts = total_kind_shifts.saturating_add(unique_entries.len() as u64);

            if samples.len() < 64 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let entries = unique_entries
                    .iter()
                    .take(4)
                    .map(|w| {
                        serde_json::json!({
                            "node": w.get("node").cloned().unwrap_or(serde_json::Value::Null),
                            "element": w.get("element").cloned().unwrap_or(serde_json::Value::Null),
                            "window_shift_kind": w.get("window_shift_kind").cloned().unwrap_or(serde_json::Value::Null),
                            "window_shift_reason": w.get("window_shift_reason").cloned().unwrap_or(serde_json::Value::Null),
                            "window_shift_apply_mode": w.get("window_shift_apply_mode").cloned().unwrap_or(serde_json::Value::Null),
                            "window_mismatch": w.get("window_mismatch").cloned().unwrap_or(serde_json::Value::Null),
                        })
                    })
                    .collect::<Vec<_>>();

                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "kind": kind,
                    "shifts_in_frame": unique_entries.len(),
                    "entries": entries,
                }));
            }
        }
    }

    let out_path = out_dir.join(format!("check.vlist_window_shifts_{kind}_max.json"));
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": format!("vlist_window_shifts_{kind}_max"),
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_total_kind_shifts": max_total_kind_shifts,
        "snapshots_examined": snapshots_examined,
        "total_kind_shifts": total_kind_shifts,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if total_kind_shifts > max_total_kind_shifts {
        return Err(format!(
            "vlist window-shift kind gate failed: total_{kind}_shifts={total_kind_shifts} exceeded max_total_kind_shifts={max_total_kind_shifts} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_vlist_window_shifts_have_prepaint_actions(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_have_prepaint_actions_json(
        &bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_vlist_window_shifts_have_prepaint_actions_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let after_frame = warmup_frames.saturating_add(1);
    let mut offenders: u64 = 0;
    let mut failures: Vec<String> = Vec::new();
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let debug = s.get("debug").unwrap_or(&serde_json::Value::Null);
            let vlist = debug
                .get("virtual_list_windows")
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if vlist.is_empty() {
                continue;
            }
            let actions = debug
                .get("prepaint_actions")
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);

            let shift_actions: Vec<&serde_json::Value> = actions
                .iter()
                .filter(|a| {
                    a.get("kind").and_then(|v| v.as_str()) == Some("virtual_list_window_shift")
                })
                .collect();

            for win in vlist {
                let source = win.get("source").and_then(|v| v.as_str());
                if source != Some("prepaint") {
                    continue;
                }
                let shift_kind = win
                    .get("window_shift_kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("none");
                if shift_kind == "none" {
                    continue;
                }

                let node = win.get("node").and_then(|v| v.as_u64());
                let element = win.get("element").and_then(|v| v.as_u64());
                let shift_reason = win.get("window_shift_reason").and_then(|v| v.as_str());

                let found = shift_actions.iter().any(|a| {
                    let a_node = a.get("node").and_then(|v| v.as_u64());
                    let a_element = a.get("element").and_then(|v| v.as_u64());
                    let a_kind = a
                        .get("virtual_list_window_shift_kind")
                        .and_then(|v| v.as_str());
                    let a_reason = a
                        .get("virtual_list_window_shift_reason")
                        .and_then(|v| v.as_str());

                    a_node == node
                        && a_element == element
                        && a_kind == Some(shift_kind)
                        && (shift_reason.is_none() || a_reason == shift_reason)
                });

                if !found {
                    offenders = offenders.saturating_add(1);
                    failures.push(format!(
                        "window={window_id} tick_id={tick_id} frame_id={frame_id} error=missing_vlist_window_shift_prepaint_action node={node:?} element={element:?} shift_kind={shift_kind} shift_reason={shift_reason:?}"
                    ));
                    if samples.len() < 64 {
                        samples.push(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "node": node,
                            "element": element,
                            "shift_kind": shift_kind,
                            "shift_reason": shift_reason,
                            "available_shift_actions": shift_actions.iter().take(8).map(|a| serde_json::json!({
                                "node": a.get("node").and_then(|v| v.as_u64()),
                                "element": a.get("element").and_then(|v| v.as_u64()),
                                "shift_kind": a.get("virtual_list_window_shift_kind").and_then(|v| v.as_str()),
                                "shift_reason": a.get("virtual_list_window_shift_reason").and_then(|v| v.as_str()),
                            })).collect::<Vec<_>>(),
                        }));
                    }
                }
            }
        }
    }

    let out_path = out_dir.join("check.vlist_window_shifts_have_prepaint_actions.json");
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_window_shifts_have_prepaint_actions",
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "offenders": offenders,
        "failures": failures,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if offenders > 0 {
        return Err(format!(
            "vlist window-shift prepaint-action gate failed: offenders={offenders} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}
