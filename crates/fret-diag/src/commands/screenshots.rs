use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::resolve;

#[derive(Debug, Clone)]
struct ResolvedScreenshots {
    screenshots_dir: PathBuf,
    manifest_path: PathBuf,
    manifest: Value,
}

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
    let Some(resolved) = resolve_screenshots_source(&src) else {
        return Err(format!(
            "unable to locate screenshots manifest for: {}\n  tried: direct screenshot manifests, run-dir/session-dir aggregation, <bundle_dir>/_root/screenshots/manifest.json, <bundle_dir_parent>/screenshots/<bundle_name>/manifest.json, and latest bundle resolution from session/base out dirs",
            src.display()
        ));
    };

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&resolved.manifest).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    print_screenshots_report(
        &resolved.manifest,
        &resolved.screenshots_dir,
        &resolved.manifest_path,
    );
    Ok(())
}

fn resolve_screenshots_source(src: &Path) -> Option<ResolvedScreenshots> {
    if let Some((screenshots_dir, manifest_path)) = resolve_screenshots_manifest_path(src) {
        let manifest = read_json_value(&manifest_path)?;
        let has_images = manifest
            .get("images")
            .and_then(|value| value.as_array())
            .is_some_and(|images| !images.is_empty());
        if has_images {
            return Some(ResolvedScreenshots {
                screenshots_dir,
                manifest_path,
                manifest,
            });
        }
    }

    if let Some(resolved) = resolve_aggregated_screenshots_source(src) {
        return Some(resolved);
    }

    if src.is_dir()
        && let Ok((bundle_dir, _session_id, _source)) =
            resolve::resolve_latest_bundle_dir_from_base_or_session_out_dir(src, None)
        && bundle_dir != src
    {
        return resolve_screenshots_source(&bundle_dir);
    }

    None
}

fn read_json_value(path: &Path) -> Option<Value> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn resolve_aggregated_screenshots_source(src: &Path) -> Option<ResolvedScreenshots> {
    if !src.is_dir() {
        return None;
    }

    if let Some(run_id) = src
        .file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse::<u64>().ok())
        && let Some(session_root) = src.parent().and_then(find_nearest_diag_session_root)
        && let Some(resolved) = aggregate_session_screenshots_for_run(&session_root, run_id)
    {
        return Some(resolved);
    }

    if resolve::looks_like_diag_session_root(src)
        && let Some(run_id) = resolve::try_read_script_result_v1(&src.join("script.result.json"))
            .map(|result| result.run_id)
            .filter(|run_id| *run_id != 0)
        && let Some(resolved) = aggregate_session_screenshots_for_run(src, run_id)
    {
        return Some(resolved);
    }

    if let Some(session_root) = find_nearest_diag_session_root(src)
        && session_root != src
        && let Some(run_id) =
            resolve::try_read_script_result_v1(&session_root.join("script.result.json"))
                .map(|result| result.run_id)
                .filter(|run_id| *run_id != 0)
        && let Some(resolved) = aggregate_session_screenshots_for_run(&session_root, run_id)
    {
        return Some(resolved);
    }

    if src.file_name().and_then(|s| s.to_str()) == Some("screenshots") {
        return aggregate_all_screenshots_under(src);
    }

    let screenshots_root = src.join("screenshots");
    if screenshots_root.is_dir() {
        return aggregate_all_screenshots_under(&screenshots_root);
    }

    None
}

fn find_nearest_diag_session_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if resolve::looks_like_diag_session_root(dir) {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn aggregate_session_screenshots_for_run(
    session_root: &Path,
    run_id: u64,
) -> Option<ResolvedScreenshots> {
    let screenshots_root = session_root.join("screenshots");
    if !screenshots_root.is_dir() {
        return None;
    }

    let screenshots_result = read_json_value(&session_root.join("screenshots.result.json"))?;
    let completed = screenshots_result.get("completed")?.as_array()?;
    let request_prefix = format!("script-run-{run_id}-");

    let screenshot_dirs = completed
        .iter()
        .filter(|entry| {
            entry
                .get("request_id")
                .and_then(|value| value.as_str())
                .is_some_and(|request_id| request_id.starts_with(&request_prefix))
        })
        .filter_map(|entry| {
            entry
                .get("bundle_dir_name")
                .and_then(|value| value.as_str())
                .map(|dir_name| screenshots_root.join(dir_name))
        })
        .collect::<Vec<_>>();

    if screenshot_dirs.is_empty() {
        return None;
    }

    build_aggregated_screenshots(
        screenshots_root,
        screenshot_dirs,
        format!("manifest.run-{run_id}.json"),
    )
}

fn aggregate_all_screenshots_under(screenshots_root: &Path) -> Option<ResolvedScreenshots> {
    let mut screenshot_dirs = std::fs::read_dir(screenshots_root)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir() && path.join("manifest.json").is_file())
        .collect::<Vec<_>>();
    screenshot_dirs.sort();

    if screenshot_dirs.is_empty() {
        return None;
    }

    build_aggregated_screenshots(
        screenshots_root.to_path_buf(),
        screenshot_dirs,
        "manifest.aggregate.json".to_string(),
    )
}

fn build_aggregated_screenshots(
    screenshots_root: PathBuf,
    screenshot_dirs: Vec<PathBuf>,
    manifest_name: String,
) -> Option<ResolvedScreenshots> {
    let mut generated_unix_ms: u64 = 0;
    let mut images: Vec<Value> = Vec::new();

    for screenshot_dir in screenshot_dirs {
        let dir_name = screenshot_dir.file_name()?.to_string_lossy().to_string();
        let manifest = read_json_value(&screenshot_dir.join("manifest.json"))?;
        generated_unix_ms = generated_unix_ms.max(
            manifest
                .get("generated_unix_ms")
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
        );

        let entries = manifest.get("images")?.as_array()?;
        for entry in entries {
            let mut image = entry.as_object()?.clone();
            if let Some(file) = image.get("file").and_then(|value| value.as_str()) {
                image.insert(
                    "file".to_string(),
                    Value::String(format!("{dir_name}/{file}")),
                );
            }
            images.push(Value::Object(image));
        }
    }

    if images.is_empty() {
        return None;
    }

    Some(ResolvedScreenshots {
        screenshots_dir: screenshots_root.clone(),
        manifest_path: screenshots_root.join(manifest_name),
        manifest: json!({
            "schema_version": 1,
            "generated_unix_ms": generated_unix_ms,
            "images": images,
        }),
    })
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
        if let Some(res) = try_dir(&src.join("_root").join("screenshots")) {
            return Some(res);
        }
        if let Some(res) = try_dir(src) {
            return Some(res);
        }
        if let Some(bundle_name) = src.file_name().and_then(|s| s.to_str())
            && let Some(parent) = src.parent()
            && let Some(res) = try_dir(&parent.join("screenshots").join(bundle_name))
        {
            return Some(res);
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

        if name.ends_with(".png")
            && let Some(parent) = src.parent()
            && let Some(res) = try_dir(parent)
        {
            return Some(res);
        }

        if let Some(bundle_dir) = src.parent()
            && let Some(res) = resolve_screenshots_manifest_path(bundle_dir)
        {
            return Some(res);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("{prefix}-{}", crate::util::now_unix_ms()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_json(path: &Path, value: Value) {
        let bytes = serde_json::to_vec_pretty(&value).expect("serialize json");
        std::fs::write(path, bytes).expect("write json");
    }

    fn write_script_result(out_dir: &Path, run_id: u64, last_bundle_dir: &str) {
        write_json(
            &out_dir.join("script.result.json"),
            json!({
                "schema_version": 1,
                "run_id": run_id,
                "updated_unix_ms": 1,
                "window": null,
                "stage": "passed",
                "step_index": null,
                "reason_code": "ok",
                "reason": null,
                "evidence": null,
                "last_bundle_dir": last_bundle_dir,
                "last_bundle_artifact": null,
            }),
        );
    }

    fn write_screenshot_manifest(dir: &Path, file: &str, tick_id: u64) {
        std::fs::create_dir_all(dir).expect("create screenshot dir");
        std::fs::write(dir.join(file), b"png").expect("write png placeholder");
        write_json(
            &dir.join("manifest.json"),
            json!({
                "schema_version": 1,
                "generated_unix_ms": tick_id,
                "images": [
                    {
                        "file": file,
                        "window": 1,
                        "tick_id": tick_id,
                        "frame_id": tick_id,
                        "scale_factor": 1.0,
                        "width_px": 100,
                        "height_px": 50
                    }
                ]
            }),
        );
    }

    #[test]
    fn resolve_screenshots_source_aggregates_run_dir_captures() {
        let session = make_temp_dir("fret-diag-screenshots-run-dir");
        let run_dir = session.join("123");
        let latest_bundle = session.join("123-bundle");
        std::fs::create_dir_all(&run_dir).expect("create run dir");
        std::fs::create_dir_all(&latest_bundle).expect("create bundle dir");
        write_json(
            &run_dir.join("manifest.json"),
            json!({
                "schema_version": 2,
                "generated_unix_ms": 1,
                "files": [
                    {
                        "id": "script_result_json",
                        "path": "script.result.json"
                    }
                ]
            }),
        );

        write_script_result(&session, 123, "123-bundle");
        write_json(
            &session.join("diag.config.json"),
            json!({"schema_version": 1}),
        );
        write_json(
            &session.join("screenshots.result.json"),
            json!({
                "schema_version": 1,
                "completed": [
                    {
                        "request_id": "script-run-123-window-1-step-0008",
                        "bundle_dir_name": "200-default"
                    },
                    {
                        "request_id": "script-run-123-window-1-step-0011",
                        "bundle_dir_name": "201-opal"
                    },
                    {
                        "request_id": "script-run-999-window-1-step-0001",
                        "bundle_dir_name": "999-other"
                    }
                ]
            }),
        );

        let screenshots_root = session.join("screenshots");
        write_screenshot_manifest(&screenshots_root.join("200-default"), "default.png", 8);
        write_screenshot_manifest(&screenshots_root.join("201-opal"), "opal.png", 11);
        write_screenshot_manifest(&screenshots_root.join("999-other"), "other.png", 1);

        let resolved = resolve_screenshots_source(&run_dir).expect("resolve screenshots");
        let images = resolved
            .manifest
            .get("images")
            .and_then(|value| value.as_array())
            .expect("images array");

        assert_eq!(resolved.screenshots_dir, screenshots_root);
        assert_eq!(images.len(), 2);
        assert_eq!(
            images[0].get("file").and_then(|v| v.as_str()),
            Some("200-default/default.png")
        );
        assert_eq!(
            images[1].get("file").and_then(|v| v.as_str()),
            Some("201-opal/opal.png")
        );
    }

    #[test]
    fn resolve_screenshots_source_aggregates_latest_session_run() {
        let base = make_temp_dir("fret-diag-screenshots-base-dir");
        let session = crate::session::session_out_dir(&base, "200-1");
        let latest_bundle = session.join("555-ui-gallery");
        std::fs::create_dir_all(&latest_bundle).expect("create bundle dir");
        write_json(
            &session.join("session.json"),
            json!({
                "schema_version": 1,
                "created_unix_ms": 200,
                "pid": 1,
                "session_id": "200-1"
            }),
        );
        write_script_result(&session, 555, "555-ui-gallery");
        write_json(
            &session.join("diag.config.json"),
            json!({"schema_version": 1}),
        );
        write_json(
            &session.join("screenshots.result.json"),
            json!({
                "schema_version": 1,
                "completed": [
                    {
                        "request_id": "script-run-555-window-1-step-0008",
                        "bundle_dir_name": "600-default"
                    },
                    {
                        "request_id": "script-run-555-window-1-step-0011",
                        "bundle_dir_name": "601-opal"
                    }
                ]
            }),
        );

        let screenshots_root = session.join("screenshots");
        write_screenshot_manifest(&screenshots_root.join("600-default"), "default.png", 8);
        write_screenshot_manifest(&screenshots_root.join("601-opal"), "opal.png", 11);

        let resolved = resolve_screenshots_source(&base).expect("resolve screenshots from base");
        let images = resolved
            .manifest
            .get("images")
            .and_then(|value| value.as_array())
            .expect("images array");

        assert_eq!(resolved.screenshots_dir, screenshots_root);
        assert_eq!(images.len(), 2);
        assert_eq!(
            images[0].get("file").and_then(|v| v.as_str()),
            Some("600-default/default.png")
        );
        assert_eq!(
            images[1].get("file").and_then(|v| v.as_str()),
            Some("601-opal/opal.png")
        );
    }
}
