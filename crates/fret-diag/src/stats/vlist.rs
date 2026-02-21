use std::path::Path;

use super::wheel_scroll::first_wheel_frame_id_for_window;
use crate::util::{now_unix_ms, write_json_value};

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_max(
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

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_min(
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

pub(super) fn check_bundle_for_vlist_policy_key_stable(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_policy_key_stable_json(&bundle, bundle_path, out_dir, warmup_frames)
}

pub(super) fn check_bundle_for_vlist_policy_key_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
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
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_policy_key_stable",
        "bundle_json": bundle_path.display().to_string(),
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

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
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
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_visible_range_refreshes_min",
        "bundle_json": bundle_path.display().to_string(),
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

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
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
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_visible_range_refreshes_max",
        "bundle_json": bundle_path.display().to_string(),
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
