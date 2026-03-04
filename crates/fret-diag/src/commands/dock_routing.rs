use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::resolve;
use super::sidecars;

pub(crate) fn cmd_dock_routing(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle artifact path (try: fretboard diag dock-routing <bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let src = resolve::maybe_resolve_base_or_session_out_dir_to_latest_bundle_dir(&src);

    let (dock_routing_path, bundle_path) = if src.is_file()
        && src
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "dock.routing.json")
    {
        if sidecars::try_read_sidecar_json_v1(&src, "dock_routing", warmup_frames).is_some() {
            let bundle_path = sidecars::adjacent_bundle_path_for_sidecar(&src);
            if let Some(bundle_path) = bundle_path
                .clone()
                .filter(|p| p.is_file())
            {
                // Prefer regenerating from the bundle artifact when possible so the report can
                // evolve (bounded evidence keys) without requiring users to manually delete
                // existing `dock.routing.json` files.
                let dock_routing_path =
                    crate::bundle_index::ensure_dock_routing_json(&bundle_path, warmup_frames)?;
                (dock_routing_path, Some(bundle_path))
            } else {
                (src.clone(), bundle_path)
            }
        } else {
            return Err(format!(
                "invalid dock.routing.json (expected schema_version=1 warmup_frames={warmup_frames})\n  dock_routing: {}",
                src.display()
            ));
        }
    } else if src.is_dir() {
        let bundle_path = crate::resolve_bundle_artifact_path(&src);
        if bundle_path.is_file() {
            let dock_routing_path =
                crate::bundle_index::ensure_dock_routing_json(&bundle_path, warmup_frames)?;
            (dock_routing_path, Some(bundle_path))
        } else {
            let direct = src.join("dock.routing.json");
            if direct.is_file()
                && sidecars::try_read_sidecar_json_v1(&direct, "dock_routing", warmup_frames)
                    .is_some()
            {
                (direct, None)
            } else {
                let root = src.join("_root").join("dock.routing.json");
                if root.is_file()
                    && sidecars::try_read_sidecar_json_v1(&root, "dock_routing", warmup_frames)
                        .is_some()
                {
                    (root, None)
                } else {
                    return Err(format!(
                        "missing bundle artifact (expected bundle.json or bundle.schema2.json) under: {}",
                        src.display()
                    ));
                }
            }
        }
    } else {
        let bundle_path = crate::resolve_bundle_artifact_path(&src);
        let dock_routing_path =
            crate::bundle_index::ensure_dock_routing_json(&bundle_path, warmup_frames)?;
        (dock_routing_path, Some(bundle_path))
    };

    let routing: Value =
        serde_json::from_slice(&std::fs::read(&dock_routing_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&routing).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    print_dock_routing_report(&routing, &dock_routing_path, bundle_path.as_deref());
    Ok(())
}

fn print_dock_routing_report(routing: &Value, routing_path: &Path, bundle_path: Option<&Path>) {
    fn u64_field(v: &Value, key: &str) -> u64 {
        v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
    }

    fn scale_factor_x1000_string_obj(
        v: &serde_json::Map<String, Value>,
        key: &str,
    ) -> Option<String> {
        let sf = v.get(key).and_then(|v| v.as_u64()).unwrap_or(0);
        if sf == 0 {
            return None;
        }
        Some(format!("{:.3}", (sf as f64) / 1000.0))
    }

    fn point_xy_string_obj(v: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
        let p = v.get(key).and_then(|v| v.as_object())?;
        let x = p.get("x").and_then(|v| v.as_f64())?;
        let y = p.get("y").and_then(|v| v.as_f64())?;
        Some(format!("{x:.1},{y:.1}"))
    }

    fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
        v.get(key).and_then(|v| v.as_str()).unwrap_or("")
    }

    println!("dock_routing:");
    println!("  dock_routing_json: {}", routing_path.display());
    if let Some(bundle_path) = bundle_path {
        println!("  bundle_artifact: {}", bundle_path.display());
    }
    println!("  bundle: {}", str_field(routing, "bundle"));
    println!("  warmup_frames: {}", u64_field(routing, "warmup_frames"));
    println!("  entries_total: {}", u64_field(routing, "entries_total"));

    let Some(entries) = routing.get("entries").and_then(|v| v.as_array()) else {
        return;
    };
    if entries.is_empty() {
        return;
    }

    let mut windows: HashSet<u64> = HashSet::new();
    for e in entries {
        if let Some(w) = e.get("window").and_then(|v| v.as_u64()) {
            windows.insert(w);
        }
    }
    println!("  windows_touched_total: {}", windows.len());

    println!("  entries (most recent first):");
    let max = 12usize;
    for e in entries.iter().rev().take(max) {
        let window = e.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let frame_id = e
            .get("frame_id")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());
        let hover = e
            .get("ui_window_hover_detection")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut parts: Vec<String> = Vec::new();

        if let Some(drag) = e.get("dock_drag").and_then(|v| v.as_object()) {
            let src = drag
                .get("source_window")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cur = drag
                .get("current_window")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let sf_cur =
                scale_factor_x1000_string_obj(drag, "current_window_scale_factor_x1000");
            let dragging = drag
                .get("dragging")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let cross = drag
                .get("cross_window_hover")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let under = drag
                .get("window_under_cursor_source")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let pos = point_xy_string_obj(drag, "position");
            let start = point_xy_string_obj(drag, "start_position");
            let grab = point_xy_string_obj(drag, "cursor_grab_offset");
            let follow = drag.get("follow_window").and_then(|v| v.as_u64());
            let scr_raw = point_xy_string_obj(drag, "cursor_screen_pos_raw_physical_px");
            let scr_raw_present = scr_raw.is_some();
            let scr_used = point_xy_string_obj(drag, "cursor_screen_pos_used_physical_px");
            let clamped = drag
                .get("cursor_screen_pos_was_clamped")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let override_active = drag
                .get("cursor_override_active")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let origin = point_xy_string_obj(drag, "current_window_client_origin_screen_physical_px");
            let origin_platform = drag
                .get("current_window_client_origin_source_platform")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let sf_runner =
                scale_factor_x1000_string_obj(drag, "current_window_scale_factor_x1000_from_runner");
            let sf_moving = scale_factor_x1000_string_obj(drag, "moving_window_scale_factor_x1000");

            let mut drag_parts: Vec<String> = Vec::new();
            drag_parts.push(format!("src={src}"));
            drag_parts.push(format!("cur={cur}"));
            if let Some(pos) = pos {
                drag_parts.push(format!("pos=({pos})"));
            }
            if let Some(start) = start {
                drag_parts.push(format!("start=({start})"));
            }
            if let Some(grab) = grab {
                drag_parts.push(format!("grab=({grab})"));
            }
            if let Some(follow) = follow {
                drag_parts.push(format!("follow={follow}"));
            }
            if let Some(scr_raw) = scr_raw.as_ref() {
                drag_parts.push(format!("scr=({scr_raw})"));
            }
            if let Some(scr_used) = scr_used.as_ref() {
                if clamped {
                    drag_parts.push(format!("scr_used=({scr_used})"));
                } else if !scr_raw_present {
                    drag_parts.push(format!("scr_used=({scr_used})"));
                }
            }
            if override_active {
                drag_parts.push("override=1".to_string());
            }
            if clamped {
                drag_parts.push("clamped=1".to_string());
            }
            if let Some(origin) = origin {
                drag_parts.push(format!("origin=({origin})"));
            }
            if origin_platform {
                drag_parts.push("origin_src=platform".to_string());
            }
            if let Some(sf_runner) = sf_runner {
                drag_parts.push(format!("sf_run={sf_runner}"));
            }
            if let Some(sf_cur) = sf_cur {
                drag_parts.push(format!("sf_cur={sf_cur}"));
            }
            if let Some(sf_moving) = sf_moving {
                drag_parts.push(format!("sf_move={sf_moving}"));
            }
            drag_parts.push(format!("dragging={dragging}"));
            drag_parts.push(format!("cross={cross}"));
            if !under.is_empty() {
                drag_parts.push(format!("under={under}"));
            }
            parts.push(format!("drag {}", drag_parts.join(" ")));
        }

        if let Some(drop) = e.get("dock_drop_resolve").and_then(|v| v.as_object()) {
            let source = drop.get("source").and_then(|v| v.as_str()).unwrap_or("");
            let resolved_zone = drop
                .get("resolved")
                .and_then(|v| v.get("zone"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            parts.push(format!("drop source={} zone={}", source, resolved_zone));
        }

        println!(
            "    - window={} frame={} hover_detection={} {}",
            window,
            frame_id,
            hover,
            parts.join(" | ")
        );
    }
    if entries.len() > max {
        println!("    - ... ({} more)", entries.len() - max);
    }
}
