use std::path::{Path, PathBuf};

use fret_diag_protocol::{UiScriptResultV1, UiScriptStageV1};

use crate::util::{now_unix_ms, write_json_value};

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
    write_run_id_manifest_json(out_dir, run_id, result);
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

fn stage_as_str(stage: &UiScriptStageV1) -> &'static str {
    match stage {
        UiScriptStageV1::Queued => "queued",
        UiScriptStageV1::Running => "running",
        UiScriptStageV1::Passed => "passed",
        UiScriptStageV1::Failed => "failed",
    }
}

pub(crate) fn write_run_id_manifest_json(out_dir: &Path, run_id: u64, result: &UiScriptResultV1) {
    let dir = run_id_artifact_dir(out_dir, run_id);
    let path = dir.join("manifest.json");

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "run_id": run_id,
        "paths": {
            "script_result_json": "script.result.json",
            "bundle_json": "bundle.json",
        },
        "script_result": {
            "stage": stage_as_str(&result.stage),
            "reason_code": result.reason_code,
            "updated_unix_ms": result.updated_unix_ms,
        },
        "last_bundle_dir": result.last_bundle_dir,
        "last_bundle_artifact": result.last_bundle_artifact,
    });

    let _ = write_json_value(&path, &payload);
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use fret_diag_protocol::{UiScriptEvidenceV1, UiScriptStageV1};

    use super::*;

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        root
    }

    #[test]
    fn write_run_id_script_result_also_writes_manifest_json() {
        let root = make_temp_dir("fret-diag-run-artifacts-manifest");
        let run_id = 7u64;
        let result = UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: now_unix_ms(),
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(UiScriptEvidenceV1::default()),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };

        write_run_id_script_result(&root, run_id, &result);

        let manifest_path = root.join(run_id.to_string()).join("manifest.json");
        let bytes = std::fs::read(&manifest_path).expect("read manifest.json");
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("parse manifest.json");
        assert_eq!(parsed.get("run_id").and_then(|v| v.as_u64()), Some(run_id));
        assert_eq!(
            parsed
                .get("script_result")
                .and_then(|v| v.get("stage"))
                .and_then(|v| v.as_str()),
            Some("passed")
        );
        assert_eq!(
            parsed
                .get("paths")
                .and_then(|v| v.get("script_result_json"))
                .and_then(|v| v.as_str()),
            Some("script.result.json")
        );
    }
}
