use std::collections::HashMap;
use std::path::Path;

use fret_diag_protocol::{UiScriptResultV1, UiScriptStageV1, UiSelectorV1};

use super::AiPacketAnchorsV1;

pub(super) fn try_read_script_result_v1(dir: &Path) -> Option<UiScriptResultV1> {
    let path = dir.join("script.result.json");
    let bytes = std::fs::read(path).ok()?;
    let parsed = serde_json::from_slice::<UiScriptResultV1>(&bytes).ok()?;
    (parsed.schema_version == 1).then_some(parsed)
}

pub(super) fn pick_candidate_test_ids_for_failed_step(script: &UiScriptResultV1) -> Vec<String> {
    if !matches!(script.stage, UiScriptStageV1::Failed) {
        return Vec::new();
    }
    let Some(step_index) = script.step_index else {
        return Vec::new();
    };
    let Some(evidence) = script.evidence.as_ref() else {
        return Vec::new();
    };

    let mut scores: HashMap<String, u32> = HashMap::new();
    let mut bump = |id: &str, w: u32| {
        let id = id.trim();
        if id.is_empty() {
            return;
        }
        let entry = scores.entry(id.to_string()).or_insert(0);
        *entry = entry.saturating_add(w);
    };

    for entry in evidence
        .selector_resolution_trace
        .iter()
        .filter(|e| e.step_index == step_index)
    {
        if let UiSelectorV1::TestId { id } = &entry.selector {
            bump(id, 100);
        }
        for cand in &entry.candidates {
            if let Some(id) = cand.test_id.as_deref() {
                bump(id, 30);
            }
        }
    }

    for entry in evidence
        .hit_test_trace
        .iter()
        .filter(|e| e.step_index == step_index)
    {
        if let Some(id) = entry.intended_test_id.as_deref() {
            bump(id, 90);
        }
        if let Some(id) = entry.hit_semantics_test_id.as_deref() {
            bump(id, 80);
        }
        if let Some(id) = entry.pointer_capture_test_id.as_deref() {
            bump(id, 60);
        }
        if let Some(id) = entry.pointer_occlusion_test_id.as_deref() {
            bump(id, 60);
        }
    }

    for entry in evidence
        .click_stable_trace
        .iter()
        .filter(|e| e.step_index == step_index)
    {
        let hit = &entry.hit_test;
        if let Some(id) = hit.intended_test_id.as_deref() {
            bump(id, 80);
        }
        if let Some(id) = hit.hit_semantics_test_id.as_deref() {
            bump(id, 70);
        }
        if let Some(id) = hit.pointer_capture_test_id.as_deref() {
            bump(id, 50);
        }
        if let Some(id) = hit.pointer_occlusion_test_id.as_deref() {
            bump(id, 50);
        }
    }

    for entry in evidence
        .focus_trace
        .iter()
        .filter(|e| e.step_index == step_index)
    {
        if let Some(id) = entry.expected_test_id.as_deref() {
            bump(id, 90);
        }
        if let Some(id) = entry.focused_test_id.as_deref() {
            bump(id, 70);
        }
    }

    let mut items: Vec<(String, u32)> = scores.into_iter().collect();
    items.sort_by(|(a_id, a), (b_id, b)| b.cmp(a).then_with(|| a_id.cmp(b_id)));

    items
        .into_iter()
        .filter_map(|(id, _)| (!id.is_empty()).then_some(id))
        .take(6)
        .collect()
}

pub(super) fn write_packet_anchors_if_possible(dir: &Path) -> Result<(), String> {
    let script_path = dir.join("script.result.json");
    let index_path = dir.join("bundle.index.json");
    if !script_path.is_file() || !index_path.is_file() {
        return Ok(());
    }

    let script = match super::budget::read_json(&script_path) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    if script.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        return Ok(());
    }

    let stage = script
        .get("stage")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let failed_step_index = if stage == "failed" {
        script
            .get("step_index")
            .and_then(|v| v.as_u64())
            .map(|v| v.min(u32::MAX as u64) as u32)
    } else {
        None
    };

    let idx = match super::budget::read_json(&index_path) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    if idx.get("kind").and_then(|v| v.as_str()) != Some("bundle_index") {
        return Ok(());
    }

    let mut anchors = AiPacketAnchorsV1 {
        failed_step_index,
        failed_window: None,
        failed_frame_id: None,
        failed_window_snapshot_seq: None,
    };

    if let Some(step_index) = failed_step_index {
        if let Some(steps) = idx
            .get("script")
            .and_then(|v| v.get("steps"))
            .and_then(|v| v.as_array())
        {
            if let Some(step) = steps.iter().find(|s| {
                s.get("step_index")
                    .and_then(|v| v.as_u64())
                    .is_some_and(|v| v == step_index as u64)
            }) {
                anchors.failed_window = step.get("window").and_then(|v| v.as_u64());
                anchors.failed_frame_id = step.get("frame_id").and_then(|v| v.as_u64());
                anchors.failed_window_snapshot_seq =
                    step.get("window_snapshot_seq").and_then(|v| v.as_u64());
            }
        }
    }

    let payload = serde_json::json!({
        "kind": "ai_packet_anchors",
        "schema_version": 1,
        "failed_step_index": anchors.failed_step_index,
        "failed_snapshot": if anchors.failed_window.is_some()
            || anchors.failed_frame_id.is_some()
            || anchors.failed_window_snapshot_seq.is_some()
        {
            serde_json::json!({
                "window": anchors.failed_window,
                "frame_id": anchors.failed_frame_id,
                "window_snapshot_seq": anchors.failed_window_snapshot_seq,
            })
        } else {
            serde_json::Value::Null
        },
    });

    let bytes = serde_json::to_vec_pretty(&payload).unwrap_or_else(|_| b"{}".to_vec());
    std::fs::write(dir.join("anchors.json"), bytes).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_test_ids_are_ranked_and_deduped_for_failed_step() {
        let v = serde_json::json!({
            "schema_version": 1,
            "run_id": 7,
            "updated_unix_ms": 0,
            "window": null,
            "stage": "failed",
            "step_index": 3,
            "reason_code": null,
            "reason": null,
            "evidence": {
                "selector_resolution_trace": [{
                    "step_index": 3,
                    "selector": { "kind": "test_id", "id": "primary" },
                    "candidates": [
                        { "node_id": 1, "role": "button", "test_id": "secondary" }
                    ]
                }],
                "hit_test_trace": [{
                    "step_index": 3,
                    "selector": { "kind": "test_id", "id": "primary" },
                    "position": { "x_px": 0.0, "y_px": 0.0 },
                    "intended_test_id": "primary",
                    "hit_semantics_test_id": "secondary"
                }],
                "focus_trace": [{
                    "step_index": 3,
                    "expected_test_id": "focus"
                }]
            },
            "last_bundle_dir": null
        });

        let script: UiScriptResultV1 = serde_json::from_value(v).expect("parse script.result");
        let ids = pick_candidate_test_ids_for_failed_step(&script);
        assert_eq!(ids.get(0).map(String::as_str), Some("primary"));
        assert_eq!(ids.get(1).map(String::as_str), Some("secondary"));
        assert_eq!(ids.get(2).map(String::as_str), Some("focus"));
    }

    #[test]
    fn candidate_test_ids_are_empty_when_not_failed() {
        let script = UiScriptResultV1 {
            schema_version: 1,
            run_id: 1,
            updated_unix_ms: 0,
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        assert!(pick_candidate_test_ids_for_failed_step(&script).is_empty());
    }
}

