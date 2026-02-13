use std::path::{Path, PathBuf};

use fret_diag_protocol::UiScriptResultV1;

use crate::util::write_json_value;

pub(crate) fn run_id_artifact_dir(out_dir: &Path, run_id: u64) -> PathBuf {
    out_dir.join(run_id.to_string())
}

pub(crate) fn write_run_id_script_result(out_dir: &Path, run_id: u64, result: &UiScriptResultV1) {
    let dir = run_id_artifact_dir(out_dir, run_id);
    let path = dir.join("script.result.json");
    let _ = write_json_value(
        &path,
        &serde_json::to_value(result).unwrap_or_else(|_| serde_json::json!({})),
    );
}

pub(crate) fn write_run_id_bundle_json(out_dir: &Path, run_id: u64, bundle_json_path: &Path) {
    if !bundle_json_path.is_file() {
        return;
    }
    let dir = run_id_artifact_dir(out_dir, run_id);
    let dst = dir.join("bundle.json");
    if let Some(parent) = dst.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Best-effort alias: keep a stable per-run path even when the underlying bundle export directory
    // is timestamp/label-based (filesystem) or message-derived (WS).
    let _ = std::fs::copy(bundle_json_path, &dst);
}

