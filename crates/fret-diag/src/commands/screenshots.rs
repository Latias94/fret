use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::resolve;

pub(crate) fn cmd_screenshots(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle artifact path (try: fretboard diag screenshots <out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let resolved = resolve_screenshots_manifest_path(&src)
        .or_else(|| {
            let start_dir = if src.is_dir() {
                src.to_path_buf()
            } else {
                src.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
            };
            let bundle_dir =
                resolve::resolve_base_or_session_out_dir_to_latest_bundle_dir_or_self(&start_dir);
            resolve_screenshots_manifest_path(&bundle_dir)
        })
        .or_else(|| {
            let start = if src.is_dir() {
                src.as_path()
            } else {
                src.parent().unwrap_or_else(|| Path::new("."))
            };
            let out_dir = resolve::find_nearest_script_result_json_preferring_evidence(start)
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))?;
            let bundle_dir =
                resolve::resolve_latest_bundle_dir_from_base_or_session_out_dir(&out_dir, None)
                    .ok()
                    .map(|(bundle_dir, _session_id, _source)| bundle_dir)?;
            resolve_screenshots_manifest_path(&bundle_dir)
        });

    let Some((screenshots_dir, manifest_path)) = resolved else {
        return Err(format!(
            "unable to locate screenshots manifest for: {}\n  tried: <bundle_dir>/_root/screenshots/manifest.json, <bundle_dir_parent>/screenshots/<bundle_name>/manifest.json, direct manifest paths, and (for out dirs) <out_dir>/script.result.json (last_bundle_dir) or <out_dir>/latest.txt (and for base dirs: <base_dir>/sessions/<latest>/...)",
            src.display()
        ));
    };

    let manifest: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    print_screenshots_report(&manifest, &screenshots_dir, &manifest_path);
    Ok(())
}

pub(crate) fn resolve_screenshots_manifest_path(src: &Path) -> Option<(PathBuf, PathBuf)> {
    let try_dir = |dir: &Path| -> Option<(PathBuf, PathBuf)> {
        let manifest = dir.join("manifest.json");
        if manifest.is_file() {
            return Some((dir.to_path_buf(), manifest));
        }
        None
    };

    if src.is_dir() {
        // Packed zip extraction layout: <bundle_dir>/_root/screenshots/manifest.json
        if let Some(res) = try_dir(&src.join("_root").join("screenshots")) {
            return Some(res);
        }
        // Direct screenshots dir: <screenshots_dir>/manifest.json
        if let Some(res) = try_dir(src) {
            return Some(res);
        }
        // Runtime filesystem layout: <out_dir>/screenshots/<bundle_dir_name>/manifest.json
        if let Some(bundle_name) = src.file_name().and_then(|s| s.to_str()) {
            if let Some(parent) = src.parent() {
                if let Some(res) = try_dir(&parent.join("screenshots").join(bundle_name)) {
                    return Some(res);
                }
            }
        }
        return None;
    }

    if src.is_file() {
        let name = src.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name == "manifest.json" {
            if let Some(parent) = src.parent() {
                return Some((parent.to_path_buf(), src.to_path_buf()));
            }
        }

        if name.ends_with(".png") {
            if let Some(parent) = src.parent() {
                if let Some(res) = try_dir(parent) {
                    return Some(res);
                }
            }
        }

        // Bundle artifact -> bundle dir
        if let Some(bundle_dir) = src.parent() {
            if let Some(res) = resolve_screenshots_manifest_path(bundle_dir) {
                return Some(res);
            }
        }
    }

    None
}

fn print_screenshots_report(manifest: &Value, screenshots_dir: &Path, manifest_path: &Path) {
    fn u64_field(v: &Value, key: &str) -> u64 {
        v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
    }

    println!("screenshots:");
    println!("  screenshots_dir: {}", screenshots_dir.display());
    println!("  manifest_json: {}", manifest_path.display());

    let schema = u64_field(manifest, "schema_version");
    let generated = u64_field(manifest, "generated_unix_ms");
    println!("  schema_version: {}", schema);
    println!("  generated_unix_ms: {}", generated);

    let Some(images) = manifest.get("images").and_then(|v| v.as_array()) else {
        println!("  images_total: 0");
        return;
    };
    println!("  images_total: {}", images.len());

    let mut windows: HashSet<u64> = HashSet::new();
    for img in images {
        if let Some(w) = img.get("window").and_then(|v| v.as_u64()) {
            windows.insert(w);
        }
    }
    println!("  windows_total: {}", windows.len());

    if images.is_empty() {
        return;
    }

    println!("  images:");
    let max = 12usize;
    for img in images.iter().rev().take(max) {
        let file = img.get("file").and_then(|v| v.as_str()).unwrap_or("");
        let window = img.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let tick_id = img.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let frame_id = img.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let scale = img
            .get("scale_factor")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let w_px = img.get("width_px").and_then(|v| v.as_u64()).unwrap_or(0);
        let h_px = img.get("height_px").and_then(|v| v.as_u64()).unwrap_or(0);
        let path = screenshots_dir.join(file);
        let exists = if path.is_file() { "ok" } else { "missing" };

        println!(
            "    - {exists} window={window} tick={tick_id} frame={frame_id} scale={scale:.2} size={w_px}x{h_px} file={file}",
        );
    }
    if images.len() > max {
        println!("    - ... ({} more)", images.len() - max);
    }
}
