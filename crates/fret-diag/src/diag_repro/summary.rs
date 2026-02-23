use super::super::*;
use super::launch::PreparedReproLaunch;

pub(super) fn build_captures_json(
    with_tracy: bool,
    with_renderdoc: bool,
    prepared_launch: &PreparedReproLaunch,
    renderdoc_capture_payload: Option<&serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({
        "tracy": if with_tracy {
            serde_json::json!({
                "requested": true,
                "env_enabled": prepared_launch.tracy_env_enabled,
                "feature_injected": prepared_launch.tracy_feature_injected,
                "note": "Capture is not recorded automatically yet; use the Tracy UI to save a capture."
            })
        } else {
            serde_json::Value::Null
        },
        "renderdoc": if with_renderdoc {
            renderdoc_capture_payload.cloned().unwrap_or_else(|| serde_json::json!({
                "schema_version": 2,
                "generated_unix_ms": now_unix_ms(),
                "capture_dir": "renderdoc",
                "autocapture_after_frames": prepared_launch.renderdoc_autocapture_after_frames,
                "captures": [],
            }))
        } else {
            serde_json::Value::Null
        }
    })
}

pub(super) fn summary_json_with_error(
    summary_json: &serde_json::Value,
    overall_error: Option<&str>,
    overall_reason_code: Option<&str>,
) -> serde_json::Value {
    summary_json
        .as_object()
        .cloned()
        .map(|mut obj| {
            if let Some(err) = overall_error {
                obj.insert(
                    "error".to_string(),
                    serde_json::Value::String(err.to_string()),
                );
            }
            if let Some(code) = overall_reason_code {
                obj.insert(
                    "error_reason_code".to_string(),
                    serde_json::Value::String(code.to_string()),
                );
            }
            serde_json::Value::Object(obj)
        })
        .unwrap_or_else(|| summary_json.clone())
}

pub(super) fn write_summary_and_evidence_best_effort(
    resolved_out_dir: &Path,
    summary_path: &Path,
    summary_json: &serde_json::Value,
) {
    if let Some(parent) = summary_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = write_json_value(summary_path, summary_json);
    let _ = write_evidence_index(resolved_out_dir, summary_path, Some(summary_json));
}
