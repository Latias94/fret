use std::path::Path;

use crate::util::{now_unix_ms, write_json_value};

pub(crate) fn check_bundle_for_layout_fast_path_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_layout_fast_path_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_frames,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_layout_fast_path_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut fast_path_frames: u64 = 0;

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

            let taken = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("layout_fast_path_taken"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if taken {
                fast_path_frames = fast_path_frames.saturating_add(1);
            }
        }
    }

    let out_path = out_dir.join("check.layout_fast_path_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "layout_fast_path_min",
        "bundle_artifact": bundle_path.display().to_string(),
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_frames": min_frames,
        "examined_snapshots": examined_snapshots,
        "fast_path_frames": fast_path_frames,
    });
    write_json_value(&out_path, &payload)?;

    if examined_snapshots == 0 {
        return Err(format!(
            "layout fast-path gate requires snapshots after warmup, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if fast_path_frames >= min_frames {
        return Ok(());
    }

    Err(format!(
        "layout fast-path gate failed (expected at least {min_frames} frames to take the fast-path after warmup, got {fast_path_frames}; examined_snapshots={examined_snapshots}, warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

pub(crate) fn check_bundle_for_prepaint_actions_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_snapshots: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_prepaint_actions_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_snapshots,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_prepaint_actions_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_snapshots: u64,
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
    let mut snapshots_with_actions: u64 = 0;
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
            let actions = s
                .get("debug")
                .and_then(|v| v.get("prepaint_actions"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if actions.is_empty() {
                continue;
            }

            snapshots_with_actions = snapshots_with_actions.saturating_add(1);
            if samples.len() < 32 {
                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "actions_len": actions.len(),
                }));
            }
        }
    }

    let out_path = out_dir.join("check.prepaint_actions_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "prepaint_actions_min",
        "bundle_artifact": bundle_path.display().to_string(),
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_snapshots": min_snapshots,
        "snapshots_with_actions": snapshots_with_actions,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if min_snapshots > 0 && snapshots_with_actions < min_snapshots {
        return Err(format!(
            "expected prepaint actions to be recorded in at least min_snapshots={min_snapshots}, but snapshots_with_actions={snapshots_with_actions} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_chart_sampling_window_shifts_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_chart_sampling_window_shifts_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_actions,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_chart_sampling_window_shifts_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
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
    let mut snapshots_examined: u64 = 0;
    let mut total_actions: u64 = 0;
    let mut unique_keys: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
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
            snapshots_examined = snapshots_examined.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let actions = s
                .get("debug")
                .and_then(|v| v.get("prepaint_actions"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if actions.is_empty() {
                continue;
            }

            for a in actions {
                let kind = a.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                if kind != "chart_sampling_window_shift" {
                    continue;
                }
                total_actions = total_actions.saturating_add(1);

                let key = a
                    .get("chart_sampling_window_key")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if key != 0 {
                    unique_keys.insert(key);
                }

                if samples.len() < 32 {
                    samples.push(serde_json::json!({
                        "window": window_id,
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                        "chart_sampling_window_key": key,
                    }));
                }
            }
        }
    }

    let out_path = out_dir.join("check.chart_sampling_window_shifts_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "chart_sampling_window_shifts_min",
        "bundle_artifact": bundle_path.display().to_string(),
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_actions": min_actions,
        "snapshots_examined": snapshots_examined,
        "total_actions": total_actions,
        "unique_keys": unique_keys.into_iter().collect::<Vec<u64>>(),
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if min_actions > 0 && total_actions < min_actions {
        return Err(format!(
            "expected chart sampling window shift actions to be recorded at least min_actions={min_actions}, but total_actions={total_actions} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_node_graph_cull_window_shifts_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_node_graph_cull_window_shifts_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_actions,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_node_graph_cull_window_shifts_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_actions: u64,
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
    let mut snapshots_examined: u64 = 0;
    let mut total_actions: u64 = 0;
    let mut unique_keys: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
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
            snapshots_examined = snapshots_examined.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let actions = s
                .get("debug")
                .and_then(|v| v.get("prepaint_actions"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if actions.is_empty() {
                continue;
            }

            for a in actions {
                let kind = a.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                if kind != "node_graph_cull_window_shift" {
                    continue;
                }
                total_actions = total_actions.saturating_add(1);

                let key = a
                    .get("node_graph_cull_window_key")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if key != 0 {
                    unique_keys.insert(key);
                }

                if samples.len() < 32 {
                    samples.push(serde_json::json!({
                        "window": window_id,
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                        "node_graph_cull_window_key": key,
                    }));
                }
            }
        }
    }

    let out_path = out_dir.join("check.node_graph_cull_window_shifts_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "node_graph_cull_window_shifts_min",
        "bundle_artifact": bundle_path.display().to_string(),
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_actions": min_actions,
        "snapshots_examined": snapshots_examined,
        "total_actions": total_actions,
        "unique_keys": unique_keys.into_iter().collect::<Vec<u64>>(),
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if min_actions > 0 && total_actions < min_actions {
        return Err(format!(
            "expected node graph cull window shift actions to be recorded at least min_actions={min_actions}, but total_actions={total_actions} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_node_graph_cull_window_shifts_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_actions: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_node_graph_cull_window_shifts_max_json(
        &bundle,
        bundle_path,
        out_dir,
        max_actions,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_node_graph_cull_window_shifts_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_actions: u64,
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
    let mut snapshots_examined: u64 = 0;
    let mut total_actions: u64 = 0;
    let mut unique_keys: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
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
            snapshots_examined = snapshots_examined.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let actions = s
                .get("debug")
                .and_then(|v| v.get("prepaint_actions"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if actions.is_empty() {
                continue;
            }

            for a in actions {
                let kind = a.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                if kind != "node_graph_cull_window_shift" {
                    continue;
                }
                total_actions = total_actions.saturating_add(1);

                let key = a
                    .get("node_graph_cull_window_key")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if key != 0 {
                    unique_keys.insert(key);
                }

                if samples.len() < 32 {
                    samples.push(serde_json::json!({
                        "window": window_id,
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                        "node_graph_cull_window_key": key,
                    }));
                }
            }
        }
    }

    let out_path = out_dir.join("check.node_graph_cull_window_shifts_max.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "node_graph_cull_window_shifts_max",
        "bundle_artifact": bundle_path.display().to_string(),
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_actions": max_actions,
        "snapshots_examined": snapshots_examined,
        "total_actions": total_actions,
        "unique_keys": unique_keys.into_iter().collect::<Vec<u64>>(),
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if total_actions > max_actions {
        return Err(format!(
            "expected node graph cull window shift actions to stay under max_actions={max_actions}, but total_actions={total_actions} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}
