use std::path::{Path, PathBuf};

use serde_json::Value;

pub(crate) fn try_read_sidecar_json_v1(
    path: &Path,
    kind: &str,
    warmup_frames: u64,
) -> Option<Value> {
    let bytes = std::fs::read(path).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("kind").and_then(|v| v.as_str()) != Some(kind) {
        return None;
    }
    if v.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        return None;
    }
    if v.get("warmup_frames").and_then(|v| v.as_u64()) != Some(warmup_frames) {
        return None;
    }
    Some(v)
}

pub(crate) fn adjacent_bundle_json_path_for_sidecar(path: &Path) -> Option<PathBuf> {
    let parent = path.parent()?;

    // Common layouts:
    // - <bundle_dir>/bundle.json + <bundle_dir>/bundle.index.json
    // - <bundle_dir>/_root/bundle.json (packed zip extraction)
    let bundle = crate::resolve_bundle_json_path(parent);
    if bundle.is_file() {
        return Some(bundle);
    }

    // Best-effort: if the sidecar is under `_root/`, also try the bundle dir.
    if parent.file_name().and_then(|s| s.to_str()) == Some("_root") {
        if let Some(grandparent) = parent.parent() {
            let bundle = crate::resolve_bundle_json_path(grandparent);
            if bundle.is_file() {
                return Some(bundle);
            }
        }
    }

    None
}
