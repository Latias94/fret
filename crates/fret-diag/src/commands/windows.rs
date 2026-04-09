use std::path::{Path, PathBuf};

use serde_json::Value;

use super::resolve;
use super::sidecars;

fn bounds_component(bounds: &Value, logical_key: &str, physical_key: &str) -> f64 {
    bounds
        .get(logical_key)
        .and_then(|v| v.as_f64())
        .or_else(|| bounds.get(physical_key).and_then(|v| v.as_f64()))
        .unwrap_or(0.0)
}

fn format_window_bounds(bounds: &Value) -> String {
    if bounds.is_object() {
        let x = bounds_component(bounds, "x", "x_px");
        let y = bounds_component(bounds, "y", "y_px");
        let ww = bounds_component(bounds, "w", "w_px");
        let hh = bounds_component(bounds, "h", "h_px");
        format!("x={x:.1} y={y:.1} w={ww:.1} h={hh:.1}")
    } else {
        "null".to_string()
    }
}

pub(crate) fn cmd_windows(
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
            "missing bundle artifact path (try: fretboard-dev diag windows <bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));

    let (window_map_path, bundle_path) = if src.is_file()
        && src
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "window.map.json")
    {
        if sidecars::try_read_sidecar_json_v1(&src, "window_map", warmup_frames).is_some() {
            let bundle_path = sidecars::adjacent_bundle_path_for_sidecar(&src);
            (src.clone(), bundle_path)
        } else {
            return Err(format!(
                "invalid window.map.json (expected schema_version=1 warmup_frames={warmup_frames})\n  window_map: {}",
                src.display()
            ));
        }
    } else if src.is_dir() {
        let resolved = resolve::resolve_bundle_ref(&src)?;
        let bundle_dir = resolved.bundle_dir;
        let bundle_path = resolved.bundle_artifact;

        let direct = bundle_dir.join("window.map.json");
        if direct.is_file()
            && sidecars::try_read_sidecar_json_v1(&direct, "window_map", warmup_frames).is_some()
        {
            (direct, None)
        } else {
            let root = bundle_dir.join("_root").join("window.map.json");
            if root.is_file()
                && sidecars::try_read_sidecar_json_v1(&root, "window_map", warmup_frames).is_some()
            {
                (root, None)
            } else {
                let window_map_path =
                    crate::bundle_index::ensure_window_map_json(&bundle_path, warmup_frames)?;
                (window_map_path, Some(bundle_path))
            }
        }
    } else {
        let resolved = resolve::resolve_bundle_ref(&src)?;
        let bundle_path = resolved.bundle_artifact;
        let window_map_path =
            crate::bundle_index::ensure_window_map_json(&bundle_path, warmup_frames)?;
        (window_map_path, Some(bundle_path))
    };

    let map: Value =
        serde_json::from_slice(&std::fs::read(&window_map_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    print_windows_report(&map, &window_map_path, bundle_path.as_deref());
    Ok(())
}

fn print_windows_report(window_map: &Value, window_map_path: &Path, bundle_path: Option<&Path>) {
    fn u64_field(v: &Value, key: &str) -> u64 {
        v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
    }

    fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
        v.get(key).and_then(|v| v.as_str()).unwrap_or("")
    }

    println!("window_map:");
    println!("  window_map_json: {}", window_map_path.display());
    if let Some(bundle_path) = bundle_path {
        println!("  bundle_artifact: {}", bundle_path.display());
    }
    println!("  bundle: {}", str_field(window_map, "bundle"));
    println!(
        "  warmup_frames: {}",
        u64_field(window_map, "warmup_frames")
    );
    println!(
        "  windows_total: {}",
        u64_field(window_map, "windows_total")
    );

    let Some(windows) = window_map.get("windows").and_then(|v| v.as_array()) else {
        return;
    };
    if windows.is_empty() {
        return;
    }

    println!("  windows:");
    let max = 12usize;
    for w in windows.iter().take(max) {
        let window = u64_field(w, "window");
        let snapshots_total = u64_field(w, "snapshots_total");
        let events_total = u64_field(w, "events_total");
        let last_seen = w.get("last_seen").unwrap_or(&Value::Null);
        let last_frame = last_seen
            .get("frame_id")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());
        let hover = last_seen
            .get("ui_window_hover_detection")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let docking = last_seen
            .get("docking_interaction_present")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let bounds = last_seen.get("window_bounds").unwrap_or(&Value::Null);
        let bounds_str = format_window_bounds(bounds);
        println!(
            "    - window={} snapshots={} events={} last_frame={} hover_detection={} docking_present={} bounds={}",
            window, snapshots_total, events_total, last_frame, hover, docking, bounds_str
        );
    }
    if windows.len() > max {
        println!("    - ... ({} more)", windows.len() - max);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::format_window_bounds;

    #[test]
    fn windows_report_supports_logical_bounds_keys() {
        let bounds = json!({ "x": 1.0, "y": 2.0, "w": 300.0, "h": 200.0 });
        assert_eq!(format_window_bounds(&bounds), "x=1.0 y=2.0 w=300.0 h=200.0");
    }

    #[test]
    fn windows_report_supports_physical_bounds_keys() {
        let bounds = json!({ "x_px": 1.0, "y_px": 2.0, "w_px": 300.0, "h_px": 200.0 });
        assert_eq!(format_window_bounds(&bounds), "x=1.0 y=2.0 w=300.0 h=200.0");
    }
}
