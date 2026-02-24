use std::path::Path;

use fret_diag_protocol::UiScriptStageV1;

use super::{AiPacketFailedStepSlicesReportV1, AiPacketWrittenSliceV1};

fn try_read_failed_snapshot_selector_from_anchors(
    dir: &Path,
) -> Option<(u32, u64, Option<u64>, Option<u64>)> {
    let path = dir.join("anchors.json");
    let bytes = std::fs::read(path).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("ai_packet_anchors") {
        return None;
    }
    if v.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        return None;
    }
    let step_index = v
        .get("failed_step_index")
        .and_then(|v| v.as_u64())
        .map(|v| v.min(u32::MAX as u64) as u32)?;
    let snap = v.get("failed_snapshot")?;
    let window = snap.get("window").and_then(|v| v.as_u64())?;
    let frame_id = snap.get("frame_id").and_then(|v| v.as_u64());
    let window_snapshot_seq = snap.get("window_snapshot_seq").and_then(|v| v.as_u64());
    if frame_id.is_none() && window_snapshot_seq.is_none() {
        return None;
    }
    Some((step_index, window, frame_id, window_snapshot_seq))
}

pub(super) fn write_anchor_slices_if_possible(
    bundle_path: &Path,
    warmup_frames: u64,
    packet_dir: &Path,
    max_slice_bytes: u64,
) -> Result<Option<AiPacketFailedStepSlicesReportV1>, String> {
    let Some(script) = super::anchors::try_read_script_result_v1(packet_dir) else {
        return Ok(None);
    };

    let mut report = AiPacketFailedStepSlicesReportV1 {
        schema_version: 1,
        status: "skipped".to_string(),
        ..Default::default()
    };

    report.failed_step_index = script.step_index;

    if !matches!(script.stage, UiScriptStageV1::Failed) {
        report.reason_code = Some("tooling.ai_packet.failed_step_slices.skipped.not_failed".to_string());
        return Ok(Some(report));
    }

    let Some((step_index, window, frame_id, window_snapshot_seq)) =
        try_read_failed_snapshot_selector_from_anchors(packet_dir)
    else {
        report.reason_code = Some(
            "tooling.ai_packet.failed_step_slices.skipped.missing_failed_snapshot_selector"
                .to_string(),
        );
        return Ok(Some(report));
    };

    let candidates = super::anchors::pick_candidate_test_ids_for_failed_step(&script);
    if candidates.is_empty() {
        report.reason_code =
            Some("tooling.ai_packet.failed_step_slices.skipped.no_candidate_test_id".to_string());
        return Ok(Some(report));
    }

    report.failed_step_index = Some(step_index);
    report.window = Some(window);
    report.frame_id = frame_id;
    report.window_snapshot_seq = window_snapshot_seq;
    report.candidate_test_ids = candidates.clone();

    for test_id in candidates {
        if report.written.len() >= 2 {
            break;
        }
        report.attempted_test_ids.push(test_id.clone());

        let (frame_id, window_snapshot_seq) = if window_snapshot_seq.is_some() {
            (None, window_snapshot_seq)
        } else {
            (frame_id, None)
        };

        let payload = match build_slice_payload_with_budget_at_selector(
            bundle_path,
            warmup_frames,
            test_id.as_str(),
            Some(window),
            frame_id,
            window_snapshot_seq,
            max_slice_bytes,
        ) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let matches = payload
            .get("matches")
            .and_then(|v| v.as_array())
            .map(|v| v.len() as u32)
            .unwrap_or(0);
        if matches == 0 {
            continue;
        }

        let stem = crate::util::sanitize_for_filename(&test_id, 80, "test_id");
        let file = format!("slice.failed_step.{step_index}.test_id.{stem}.json");
        crate::util::write_json_value(&packet_dir.join(&file), &payload)?;
        report.written.push(AiPacketWrittenSliceV1 {
            file,
            test_id,
            matches,
        });
    }

    if report.written.is_empty() {
        report.reason_code =
            Some("tooling.ai_packet.failed_step_slices.skipped.no_slices_written".to_string());
        return Ok(Some(report));
    }

    report.status = "written".to_string();
    Ok(Some(report))
}

pub(super) fn build_slice_payload_with_budget(
    bundle_path: &Path,
    warmup_frames: u64,
    test_id: &str,
    max_bytes: u64,
) -> Result<serde_json::Value, String> {
    let configs = [(20usize, 64usize), (10, 32), (5, 16), (3, 8)];
    for (max_matches, max_ancestors) in configs {
        let payload = super::super::slice::build_test_id_slice_payload_from_bundle_path(
            bundle_path,
            warmup_frames,
            test_id,
            None,
            None,
            None,
            max_matches,
            max_ancestors,
        )?;
        let bytes = serde_json::to_vec_pretty(&payload)
            .map(|b| b.len() as u64)
            .unwrap_or(u64::MAX);
        if bytes <= max_bytes {
            return Ok(payload);
        }
    }
    Err(format!(
        "slice payload exceeds max bytes budget (max_bytes={max_bytes})"
    ))
}

fn build_slice_payload_with_budget_at_selector(
    bundle_path: &Path,
    warmup_frames: u64,
    test_id: &str,
    window_id: Option<u64>,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    max_bytes: u64,
) -> Result<serde_json::Value, String> {
    let configs = [(20usize, 64usize), (10, 32), (5, 16), (3, 8)];
    for (max_matches, max_ancestors) in configs {
        let payload = super::super::slice::build_test_id_slice_payload_from_bundle_path(
            bundle_path,
            warmup_frames,
            test_id,
            frame_id,
            window_snapshot_seq,
            window_id,
            max_matches,
            max_ancestors,
        )?;
        let bytes = serde_json::to_vec_pretty(&payload)
            .map(|b| b.len() as u64)
            .unwrap_or(u64::MAX);
        if bytes <= max_bytes {
            return Ok(payload);
        }
    }
    Err(format!(
        "slice payload exceeds max bytes budget (max_bytes={max_bytes})"
    ))
}

