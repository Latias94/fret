use std::path::Path;

use crate::util::{now_unix_ms, write_json_value};

fn first_scroll_offset_change_frame_id_for_window(
    window: &serde_json::Value,
    warmup_frames: u64,
) -> Option<u64> {
    let snaps = window
        .get("snapshots")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    snaps
        .iter()
        .filter_map(|s| {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64())?;
            if frame_id < warmup_frames {
                return None;
            }
            let changes = s
                .get("debug")
                .and_then(|v| v.get("scroll_handle_changes"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            let any_offset_changed = changes.iter().any(|c| {
                c.get("offset_changed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            });
            any_offset_changed.then_some(frame_id)
        })
        .min()
}

pub(super) fn check_bundle_for_windowed_rows_offset_changes_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_total_offset_changes: u64,
    warmup_frames: u64,
    eps_px: f32,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_windowed_rows_offset_changes_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_total_offset_changes,
        warmup_frames,
        eps_px,
    )
}

pub(super) fn check_bundle_for_windowed_rows_offset_changes_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_total_offset_changes: u64,
    warmup_frames: u64,
    eps_px: f32,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    #[derive(Default)]
    struct SurfaceStats {
        location: Option<serde_json::Value>,
        samples: u64,
        offset_changes: u64,
        visible_start_changes: u64,
        prev_offset_y: Option<f32>,
        prev_visible_start: Option<u64>,
    }

    let mut any_scroll = false;
    let mut examined_snapshots: u64 = 0;
    let mut scroll_offset_changed_events: u64 = 0;
    let mut total_offset_changes: u64 = 0;

    let mut surfaces: std::collections::BTreeMap<(u64, u64), SurfaceStats> =
        std::collections::BTreeMap::new();
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(scroll_frame) = first_scroll_offset_change_frame_id_for_window(w, warmup_frames)
        else {
            continue;
        };
        any_scroll = true;

        let after_frame = scroll_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let scroll_changes = s
                .get("debug")
                .and_then(|v| v.get("scroll_handle_changes"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            for c in scroll_changes {
                if c.get("offset_changed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    scroll_offset_changed_events = scroll_offset_changed_events.saturating_add(1);
                }
            }

            let list = s
                .get("debug")
                .and_then(|v| v.get("windowed_rows_surfaces"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if list.is_empty() {
                continue;
            }

            for entry in list {
                let Some(callsite_id) = entry.get("callsite_id").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Some(offset_y) = entry
                    .get("offset_y")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32)
                else {
                    continue;
                };

                let stats = surfaces.entry((window_id, callsite_id)).or_default();
                stats.samples = stats.samples.saturating_add(1);
                if stats.location.is_none() {
                    stats.location = entry.get("location").cloned();
                }

                if let Some(prev) = stats.prev_offset_y {
                    let delta = offset_y - prev;
                    if delta.abs() >= eps_px {
                        stats.offset_changes = stats.offset_changes.saturating_add(1);
                        total_offset_changes = total_offset_changes.saturating_add(1);

                        if samples.len() < 32 {
                            samples.push(serde_json::json!({
                                "window": window_id,
                                "tick_id": tick_id,
                                "frame_id": frame_id,
                                "callsite_id": callsite_id,
                                "delta_offset_y": delta,
                                "prev_offset_y": prev,
                                "offset_y": offset_y,
                            }));
                        }
                    }
                }
                stats.prev_offset_y = Some(offset_y);

                if let Some(visible_start) = entry.get("visible_start").and_then(|v| v.as_u64()) {
                    if let Some(prev) = stats.prev_visible_start
                        && visible_start != prev
                    {
                        stats.visible_start_changes = stats.visible_start_changes.saturating_add(1);
                    }
                    stats.prev_visible_start = Some(visible_start);
                }
            }
        }
    }

    let out_path = out_dir.join("check.windowed_rows_offset_changes_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "windowed_rows_offset_changes_min",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "eps_px": eps_px,
        "min_total_offset_changes": min_total_offset_changes,
        "any_scroll": any_scroll,
        "examined_snapshots": examined_snapshots,
        "scroll_offset_changed_events": scroll_offset_changed_events,
        "surfaces_seen": surfaces.len(),
        "total_offset_changes": total_offset_changes,
        "surfaces": surfaces.iter().map(|((window, callsite_id), stats)| serde_json::json!({
            "window": window,
            "callsite_id": callsite_id,
            "location": stats.location,
            "samples": stats.samples,
            "offset_changes": stats.offset_changes,
            "visible_start_changes": stats.visible_start_changes,
        })).collect::<Vec<_>>(),
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_scroll {
        return Err(format!(
            "windowed rows offset-change gate requires scroll offset changes after warmup, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if examined_snapshots == 0 {
        return Err(format!(
            "windowed rows offset-change gate requires snapshots after the first scroll change, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if scroll_offset_changed_events == 0 {
        return Err(format!(
            "windowed rows offset-change gate requires debug.scroll_handle_changes events after the first scroll change, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if surfaces.is_empty() {
        return Err(format!(
            "windowed rows offset-change gate requires debug.windowed_rows_surfaces after scroll changes, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if min_total_offset_changes > 0 && total_offset_changes < min_total_offset_changes {
        return Err(format!(
            "expected windowed rows surfaces to observe scroll offset changes, but total_offset_changes={total_offset_changes} was below min_total_offset_changes={min_total_offset_changes} (warmup_frames={warmup_frames}, eps_px={eps_px})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_windowed_rows_visible_start_changes_repainted(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
        &bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
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

    #[derive(Default)]
    struct SurfaceStats {
        location: Option<serde_json::Value>,
        samples: u64,
        visible_start_changes: u64,
        suspicious_visible_start_changes: u64,
        prev_visible_start: Option<u64>,
        prev_scene_fingerprint: Option<u64>,
    }

    let mut any_scroll = false;
    let mut examined_snapshots: u64 = 0;
    let mut scroll_offset_changed_events: u64 = 0;
    let mut total_visible_start_changes: u64 = 0;
    let mut total_suspicious_changes: u64 = 0;
    let mut missing_scene_fingerprint = false;

    let mut surfaces: std::collections::BTreeMap<(u64, u64), SurfaceStats> =
        std::collections::BTreeMap::new();
    let mut suspicious: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(scroll_frame) = first_scroll_offset_change_frame_id_for_window(w, warmup_frames)
        else {
            continue;
        };
        any_scroll = true;

        let after_frame = scroll_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }

            let scroll_changes = s
                .get("debug")
                .and_then(|v| v.get("scroll_handle_changes"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            for c in scroll_changes {
                if c.get("offset_changed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    scroll_offset_changed_events = scroll_offset_changed_events.saturating_add(1);
                }
            }

            let list = s
                .get("debug")
                .and_then(|v| v.get("windowed_rows_surfaces"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if list.is_empty() {
                continue;
            }

            for entry in list {
                let Some(callsite_id) = entry.get("callsite_id").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Some(visible_start) = entry.get("visible_start").and_then(|v| v.as_u64())
                else {
                    continue;
                };
                let Some(fp) = fp else {
                    continue;
                };

                let stats = surfaces.entry((window_id, callsite_id)).or_default();
                stats.samples = stats.samples.saturating_add(1);
                if stats.location.is_none() {
                    stats.location = entry.get("location").cloned();
                }

                if let (Some(prev_start), Some(prev_fp)) =
                    (stats.prev_visible_start, stats.prev_scene_fingerprint)
                    && visible_start != prev_start
                {
                    stats.visible_start_changes = stats.visible_start_changes.saturating_add(1);
                    total_visible_start_changes = total_visible_start_changes.saturating_add(1);
                    if fp == prev_fp {
                        stats.suspicious_visible_start_changes =
                            stats.suspicious_visible_start_changes.saturating_add(1);
                        total_suspicious_changes = total_suspicious_changes.saturating_add(1);
                        if suspicious.len() < 32 {
                            suspicious.push(serde_json::json!({
                                "window": window_id,
                                "tick_id": tick_id,
                                "frame_id": frame_id,
                                "callsite_id": callsite_id,
                                "prev_visible_start": prev_start,
                                "visible_start": visible_start,
                                "scene_fingerprint": fp,
                            }));
                        }
                    }
                }

                stats.prev_visible_start = Some(visible_start);
                stats.prev_scene_fingerprint = Some(fp);
            }
        }
    }

    let out_path = out_dir.join("check.windowed_rows_visible_start_changes_repainted.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "windowed_rows_visible_start_changes_repainted",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "any_scroll": any_scroll,
        "examined_snapshots": examined_snapshots,
        "scroll_offset_changed_events": scroll_offset_changed_events,
        "surfaces_seen": surfaces.len(),
        "total_visible_start_changes": total_visible_start_changes,
        "total_suspicious_changes": total_suspicious_changes,
        "surfaces": surfaces.iter().map(|((window, callsite_id), stats)| serde_json::json!({
            "window": window,
            "callsite_id": callsite_id,
            "location": stats.location,
            "samples": stats.samples,
            "visible_start_changes": stats.visible_start_changes,
            "suspicious_visible_start_changes": stats.suspicious_visible_start_changes,
        })).collect::<Vec<_>>(),
        "suspicious_samples": suspicious,
    });
    write_json_value(&out_path, &payload)?;

    if missing_scene_fingerprint {
        return Err(format!(
            "windowed rows repaint gate requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if !any_scroll {
        return Err(format!(
            "windowed rows repaint gate requires scroll offset changes after warmup, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if examined_snapshots == 0 {
        return Err(format!(
            "windowed rows repaint gate requires snapshots after the first scroll change, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if scroll_offset_changed_events == 0 {
        return Err(format!(
            "windowed rows repaint gate requires debug.scroll_handle_changes events after the first scroll change, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if surfaces.is_empty() {
        return Err(format!(
            "windowed rows repaint gate requires debug.windowed_rows_surfaces after scroll changes, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if total_visible_start_changes == 0 {
        return Err(format!(
            "windowed rows repaint gate requires at least one visible_start change after the first scroll change (otherwise stale paint cannot be evaluated)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if total_suspicious_changes == 0 {
        return Ok(());
    }

    Err(format!(
        "windowed rows repaint gate failed (visible_start changed but scene fingerprint did not; suspected stale paint / stale lines)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}
