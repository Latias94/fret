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
            (src.clone(), bundle_path)
        } else {
            return Err(format!(
                "invalid dock.routing.json (expected schema_version=1 warmup_frames={warmup_frames})\n  dock_routing: {}",
                src.display()
            ));
        }
    } else if src.is_dir() {
        let direct = src.join("dock.routing.json");
        if direct.is_file()
            && sidecars::try_read_sidecar_json_v1(&direct, "dock_routing", warmup_frames).is_some()
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
                let bundle_path = crate::resolve_bundle_artifact_path(&src);
                let dock_routing_path =
                    crate::bundle_index::ensure_dock_routing_json(&bundle_path, warmup_frames)?;
                (dock_routing_path, Some(bundle_path))
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
            parts.push(format!(
                "drag src={} cur={} dragging={} cross={} under={}",
                src, cur, dragging, cross, under
            ));
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
