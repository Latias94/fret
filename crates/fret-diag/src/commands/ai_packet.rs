use std::path::{Path, PathBuf};

use super::doctor;

use crate::stats::{BundleStatsOptions, BundleStatsSort, bundle_stats_from_path};
use std::collections::{HashMap, HashSet};

use fret_diag_protocol::{UiScriptResultV1, UiScriptStageV1, UiSelectorV1};

use crate::frames_index::TriageLiteMetric;

#[derive(Debug, Clone)]
struct AiPacketBudgetConfig {
    soft_total_bytes: u64,
    hard_total_bytes: u64,

    max_bundle_meta_bytes: u64,
    max_bundle_index_bytes: u64,
    max_test_ids_index_bytes: u64,
    max_frames_index_bytes: u64,
    max_slice_bytes: u64,
}

impl Default for AiPacketBudgetConfig {
    fn default() -> Self {
        Self {
            soft_total_bytes: 2 * 1024 * 1024,
            hard_total_bytes: 20 * 1024 * 1024,
            max_bundle_meta_bytes: 128 * 1024,
            max_bundle_index_bytes: 4 * 1024 * 1024,
            max_test_ids_index_bytes: 2 * 1024 * 1024,
            max_frames_index_bytes: 2 * 1024 * 1024,
            max_slice_bytes: 2 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct AiPacketBudgetReport {
    kind: &'static str,
    schema_version: u32,
    budget: AiPacketBudgetConfig,
    bytes_total: u64,
    soft_budget_exceeded: bool,
    hard_budget_exceeded: bool,
    reason_code: Option<String>,
    dropped_files: Vec<String>,
    clipped_files: Vec<String>,
    failed_step_slices: Option<AiPacketFailedStepSlicesReportV1>,
}

#[derive(Debug, Clone)]
struct AiPacketAnchorsV1 {
    failed_step_index: Option<u32>,
    failed_window: Option<u64>,
    failed_frame_id: Option<u64>,
    failed_window_snapshot_seq: Option<u64>,
}

#[derive(Debug, Clone, Default)]
struct AiPacketFailedStepSlicesReportV1 {
    schema_version: u32,
    status: String,
    reason_code: Option<String>,
    failed_step_index: Option<u32>,
    window: Option<u64>,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    candidate_test_ids: Vec<String>,
    attempted_test_ids: Vec<String>,
    written: Vec<AiPacketWrittenSliceV1>,
}

#[derive(Debug, Clone)]
struct AiPacketWrittenSliceV1 {
    file: String,
    test_id: String,
    matches: u32,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_ai_packet(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    packet_out: Option<PathBuf>,
    include_triage: bool,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let mut bundle_arg: Option<String> = None;
    let mut test_id: Option<String> = None;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--test-id" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --test-id".to_string());
                };
                test_id = Some(v);
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for ai-packet: {other}"));
            }
            other => {
                if bundle_arg.is_none() && looks_like_path(other) {
                    bundle_arg = Some(other.to_string());
                } else if bundle_arg.is_none() {
                    let p = crate::resolve_path(workspace_root, PathBuf::from(other));
                    if p.is_file() || p.is_dir() {
                        bundle_arg = Some(other.to_string());
                    } else if test_id.is_none() {
                        test_id = Some(other.to_string());
                    } else {
                        return Err(format!("unexpected argument: {other}"));
                    }
                } else if test_id.is_none() {
                    test_id = Some(other.to_string());
                } else {
                    return Err(format!("unexpected argument: {other}"));
                }
                i += 1;
            }
        }
    }

    let bundle_path =
        resolve_bundle_json_path_or_latest(bundle_arg.as_deref(), workspace_root, out_dir)?;
    let bundle_dir = crate::resolve_bundle_root_dir(&bundle_path)?;

    let packet_dir = packet_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| {
            if let Some(test_id) = &test_id {
                bundle_dir.join(format!(
                    "ai.packet.{}",
                    crate::util::sanitize_for_filename(test_id, 80, "test_id")
                ))
            } else {
                bundle_dir.join("ai.packet")
            }
        });

    if packet_dir.is_file() {
        return Err(format!(
            "--packet-out must be a directory, got file: {}",
            packet_dir.display()
        ));
    }
    std::fs::create_dir_all(&packet_dir).map_err(|e| e.to_string())?;

    let meta_path = crate::bundle_index::ensure_bundle_meta_json(&bundle_path, warmup_frames)?;
    let test_ids_index_path =
        crate::bundle_index::ensure_test_ids_index_json(&bundle_path, warmup_frames)?;
    let bundle_index_path =
        crate::bundle_index::ensure_bundle_index_json(&bundle_path, warmup_frames)?;
    let frames_index_path =
        crate::frames_index::ensure_frames_index_json(&bundle_path, warmup_frames)?;

    copy_file_named(&meta_path, &packet_dir, "bundle.meta.json")?;
    copy_file_named(&test_ids_index_path, &packet_dir, "test_ids.index.json")?;
    copy_file_named(&bundle_index_path, &packet_dir, "bundle.index.json")?;
    copy_file_named(&frames_index_path, &packet_dir, "frames.index.json")?;

    if let Some(frames_index) =
        crate::frames_index::read_frames_index_json_v1(&frames_index_path, warmup_frames)
    {
        let triage_lite = crate::frames_index::triage_lite_json_from_frames_index(
            &bundle_path,
            &frames_index_path,
            &frames_index,
            warmup_frames,
            50,
            TriageLiteMetric::TotalTimeUs,
        )?;
        write_json_compact(&packet_dir.join("triage.lite.json"), &triage_lite)?;

        let hotspots_lite = crate::hotspots_lite::hotspots_lite_json_from_frames_index(
            &bundle_path,
            &frames_index_path,
            &frames_index,
            warmup_frames,
            50,
            TriageLiteMetric::TotalTimeUs,
        )?;
        write_json_compact(&packet_dir.join("hotspots.lite.json"), &hotspots_lite)?;
    }

    copy_if_present(
        &bundle_dir.join("script.result.json"),
        &packet_dir,
        "script.result.json",
    )?;
    copy_if_present(
        &bundle_dir.join("manifest.json"),
        &packet_dir,
        "manifest.json",
    )?;

    crate::util::write_json_value(
        &packet_dir.join("doctor.json"),
        &doctor::doctor_report_json(&bundle_path, warmup_frames),
    )?;

    write_packet_anchors_if_possible(&packet_dir)?;

    let mut failed_step_slices_report: Option<AiPacketFailedStepSlicesReportV1> = None;
    if test_id.is_none() {
        failed_step_slices_report = write_anchor_slices_if_possible(
            &bundle_path,
            warmup_frames,
            &packet_dir,
            AiPacketBudgetConfig::default().max_slice_bytes,
        )?;
    }

    if include_triage {
        let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);
        let report = bundle_stats_from_path(
            &bundle_path,
            stats_top,
            sort,
            BundleStatsOptions { warmup_frames },
        )?;
        let payload = crate::triage_json_from_stats(&bundle_path, &report, sort, warmup_frames);
        crate::util::write_json_value(&packet_dir.join("triage.json"), &payload)?;
    }

    if let Some(test_id) = &test_id {
        let budget = AiPacketBudgetConfig::default();
        let payload = build_slice_payload_with_budget(
            &bundle_path,
            warmup_frames,
            test_id.as_str(),
            budget.max_slice_bytes,
        )?;
        let stem = crate::util::sanitize_for_filename(test_id, 80, "test_id");
        crate::util::write_json_value(
            &packet_dir.join(format!("slice.test_id.{stem}.json")),
            &payload,
        )?;
        crate::util::write_json_value(&packet_dir.join(format!("slice.{stem}.json")), &payload)?;
    }

    let mut report = AiPacketBudgetReport {
        kind: "ai_packet",
        schema_version: 1,
        budget: AiPacketBudgetConfig::default(),
        failed_step_slices: failed_step_slices_report,
        ..Default::default()
    };
    let enforce_res = enforce_ai_packet_budgets(&packet_dir, &mut report);
    write_packet_budget_report(&packet_dir, &report)?;
    enforce_res?;

    println!("{}", packet_dir.display());
    Ok(())
}

fn write_packet_budget_report(dir: &Path, report: &AiPacketBudgetReport) -> Result<(), String> {
    let mut files_present: Vec<(String, u64)> = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let Some(name) = p
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
        else {
            continue;
        };
        let bytes = file_bytes(&p)?;
        files_present.push((name, bytes));
    }
    files_present.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut payload = serde_json::json!({
        "kind": report.kind,
        "schema_version": report.schema_version,
        "reason_code": report.reason_code,
        "budget": {
            "soft_total_bytes": report.budget.soft_total_bytes,
            "hard_total_bytes": report.budget.hard_total_bytes,
            "max_bundle_meta_bytes": report.budget.max_bundle_meta_bytes,
            "max_bundle_index_bytes": report.budget.max_bundle_index_bytes,
            "max_test_ids_index_bytes": report.budget.max_test_ids_index_bytes,
            "max_frames_index_bytes": report.budget.max_frames_index_bytes,
            "max_slice_bytes": report.budget.max_slice_bytes,
        },
        "bytes_total": report.bytes_total,
        "files_present": files_present.iter().map(|(name, bytes)| serde_json::json!({
            "name": name,
            "bytes": bytes,
        })).collect::<Vec<_>>(),
        "soft_budget_exceeded": report.soft_budget_exceeded,
        "hard_budget_exceeded": report.hard_budget_exceeded,
        "dropped_files": report.dropped_files,
        "clipped_files": report.clipped_files,
    });

    if let Some(v) = report.failed_step_slices.as_ref() {
        if let Some(obj) = payload.as_object_mut() {
            let written = v
                .written
                .iter()
                .map(|w| {
                    serde_json::json!({
                        "file": &w.file,
                        "test_id": &w.test_id,
                        "matches": w.matches,
                    })
                })
                .collect::<Vec<_>>();
            obj.insert(
                "failed_step_slices".to_string(),
                serde_json::json!({
                    "schema_version": v.schema_version,
                    "status": &v.status,
                    "reason_code": &v.reason_code,
                    "failed_step_index": v.failed_step_index,
                    "failed_snapshot": {
                        "window": v.window,
                        "frame_id": v.frame_id,
                        "window_snapshot_seq": v.window_snapshot_seq,
                    },
                    "candidate_test_ids": &v.candidate_test_ids,
                    "attempted_test_ids": &v.attempted_test_ids,
                    "written": written,
                }),
            );
        }
    }

    let bytes = serde_json::to_vec_pretty(&payload).unwrap_or_else(|_| b"{}".to_vec());
    std::fs::write(dir.join("ai.packet.json"), bytes).map_err(|e| e.to_string())?;
    Ok(())
}

fn try_read_script_result_v1(dir: &Path) -> Option<UiScriptResultV1> {
    let path = dir.join("script.result.json");
    let bytes = std::fs::read(path).ok()?;
    let parsed = serde_json::from_slice::<UiScriptResultV1>(&bytes).ok()?;
    (parsed.schema_version == 1).then_some(parsed)
}

fn pick_candidate_test_ids_for_failed_step(script: &UiScriptResultV1) -> Vec<String> {
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

fn write_anchor_slices_if_possible(
    bundle_path: &Path,
    warmup_frames: u64,
    packet_dir: &Path,
    max_slice_bytes: u64,
) -> Result<Option<AiPacketFailedStepSlicesReportV1>, String> {
    let Some(script) = try_read_script_result_v1(packet_dir) else {
        return Ok(None);
    };

    let mut report = AiPacketFailedStepSlicesReportV1 {
        schema_version: 1,
        status: "skipped".to_string(),
        ..Default::default()
    };

    report.failed_step_index = script.step_index;

    if !matches!(script.stage, UiScriptStageV1::Failed) {
        report.reason_code =
            Some("tooling.ai_packet.failed_step_slices.skipped.not_failed".to_string());
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

    let candidates = pick_candidate_test_ids_for_failed_step(&script);
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

fn write_packet_anchors_if_possible(dir: &Path) -> Result<(), String> {
    let script_path = dir.join("script.result.json");
    let index_path = dir.join("bundle.index.json");
    if !script_path.is_file() || !index_path.is_file() {
        return Ok(());
    }

    let script = match read_json(&script_path) {
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

    let idx = match read_json(&index_path) {
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

fn file_bytes(path: &Path) -> Result<u64, String> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())
}

fn packet_total_bytes(dir: &Path) -> Result<u64, String> {
    let mut total: u64 = 0;
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if p.is_file() {
            total = total.saturating_add(file_bytes(&p)?);
        }
    }
    Ok(total)
}

fn drop_if_present(
    dir: &Path,
    name: &str,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join(name);
    if path.is_file() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        report.dropped_files.push(name.to_string());
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn write_json_compact(path: &Path, v: &serde_json::Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(v).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    Ok(())
}

fn enforce_ai_packet_budgets(dir: &Path, report: &mut AiPacketBudgetReport) -> Result<(), String> {
    let cfg = report.budget.clone();

    // Always enforce per-file caps first.
    clip_bundle_meta_if_needed(dir, &cfg, report)?;
    clip_bundle_index_if_needed(dir, &cfg, report)?;
    clip_test_ids_index_if_needed(dir, &cfg, report)?;
    clip_frames_index_if_needed(dir, &cfg, report)?;

    // Optional files: if we are above the hard total budget, drop these first.
    let mut total = packet_total_bytes(dir)?;
    if total > cfg.hard_total_bytes {
        drop_if_present(dir, "triage.json", report)?;
        drop_if_present(dir, "hotspots.lite.json", report)?;
        drop_if_present(dir, "triage.lite.json", report)?;
        drop_if_present(dir, "manifest.json", report)?;
        total = packet_total_bytes(dir)?;
    }

    report.bytes_total = total;
    report.soft_budget_exceeded = total > cfg.soft_total_bytes;
    report.hard_budget_exceeded = total > cfg.hard_total_bytes;

    if report.hard_budget_exceeded {
        report.reason_code = Some("tooling.ai_packet.budget.hard_exceeded".to_string());
        return Err(format!(
            "ai packet exceeds hard budget (total_bytes={} > {})",
            total, cfg.hard_total_bytes
        ));
    }

    Ok(())
}

fn clip_bundle_meta_if_needed(
    dir: &Path,
    cfg: &AiPacketBudgetConfig,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join("bundle.meta.json");
    if !path.is_file() {
        return Ok(());
    }
    let bytes = file_bytes(&path)?;
    if bytes <= cfg.max_bundle_meta_bytes {
        return Ok(());
    }
    let v = read_json(&path)?;
    write_json_compact(&path, &v)?;
    let new_bytes = file_bytes(&path)?;
    if new_bytes > cfg.max_bundle_meta_bytes {
        report.reason_code = Some("tooling.ai_packet.budget.bundle_meta_exceeded".to_string());
        return Err(format!(
            "bundle.meta.json exceeds budget (bytes={} > max={})",
            new_bytes, cfg.max_bundle_meta_bytes
        ));
    }
    report.clipped_files.push("bundle.meta.json".to_string());
    Ok(())
}

fn clip_bundle_index_if_needed(
    dir: &Path,
    cfg: &AiPacketBudgetConfig,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join("bundle.index.json");
    if !path.is_file() {
        return Ok(());
    }
    let bytes = file_bytes(&path)?;
    if bytes <= cfg.max_bundle_index_bytes {
        return Ok(());
    }

    let mut v = read_json(&path)?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("bundle_index") {
        return Ok(());
    }

    let mut required_by_window: HashMap<u64, (HashSet<u64>, HashSet<u64>)> = HashMap::new();
    if let Some(steps) = v
        .get("script")
        .and_then(|v| v.get("steps"))
        .and_then(|v| v.as_array())
    {
        for step in steps {
            let Some(window) = step.get("window").and_then(|v| v.as_u64()) else {
                continue;
            };
            let entry = required_by_window.entry(window).or_default();
            if let Some(seq) = step.get("window_snapshot_seq").and_then(|v| v.as_u64()) {
                entry.0.insert(seq);
            }
            if let Some(frame_id) = step.get("frame_id").and_then(|v| v.as_u64()) {
                entry.1.insert(frame_id);
            }
        }
    }

    let mut max_keep: usize = v
        .get("windows")
        .and_then(|v| v.as_array())
        .map(|windows| {
            windows
                .iter()
                .filter_map(|w| {
                    w.get("snapshots")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                })
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    max_keep = max_keep.min(4096).max(8);

    loop {
        let snapshots_total: u64 = {
            let windows = v
                .get_mut("windows")
                .and_then(|v| v.as_array_mut())
                .ok_or_else(|| "invalid bundle.index.json: missing windows".to_string())?;

            for w in windows.iter_mut() {
                let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
                if let Some(snaps) = w.get_mut("snapshots").and_then(|v| v.as_array_mut()) {
                    if snaps.len() > max_keep {
                        let mut required_seq: Option<&HashSet<u64>> = None;
                        let mut required_frame: Option<&HashSet<u64>> = None;
                        if let Some((seq, frame)) = required_by_window.get(&window_id) {
                            required_seq = Some(seq);
                            required_frame = Some(frame);
                        }

                        let mut old: Vec<serde_json::Value> = Vec::new();
                        std::mem::swap(snaps, &mut old);

                        let len = old.len();
                        let start = len.saturating_sub(max_keep);
                        let mut keep: Vec<bool> = vec![false; len];
                        for i in start..len {
                            keep[i] = true;
                        }

                        if required_seq.is_some() || required_frame.is_some() {
                            for (i, s) in old.iter().enumerate() {
                                let seq = s.get("window_snapshot_seq").and_then(|v| v.as_u64());
                                let frame_id = s.get("frame_id").and_then(|v| v.as_u64());
                                let required = seq.is_some_and(|v| {
                                    required_seq.is_some_and(|set| set.contains(&v))
                                }) || frame_id.is_some_and(|v| {
                                    required_frame.is_some_and(|set| set.contains(&v))
                                });
                                if required {
                                    keep[i] = true;
                                }
                            }
                        }

                        *snaps = old
                            .into_iter()
                            .enumerate()
                            .filter_map(|(i, v)| keep.get(i).copied().unwrap_or(false).then_some(v))
                            .collect();
                    }
                }

                let (first_frame_id, first_ts, last_frame_id, last_ts) = w
                    .get("snapshots")
                    .and_then(|v| v.as_array())
                    .map(|snaps| {
                        let first = snaps.first();
                        let last = snaps.last();
                        (
                            first.and_then(|v| v.get("frame_id")).cloned(),
                            first.and_then(|v| v.get("timestamp_unix_ms")).cloned(),
                            last.and_then(|v| v.get("frame_id")).cloned(),
                            last.and_then(|v| v.get("timestamp_unix_ms")).cloned(),
                        )
                    })
                    .unwrap_or((None, None, None, None));

                if let Some(obj) = w.as_object_mut() {
                    if let Some(v) = first_frame_id {
                        obj.insert("first_frame_id".to_string(), v);
                    }
                    if let Some(v) = first_ts {
                        obj.insert("first_timestamp_unix_ms".to_string(), v);
                    }
                    if let Some(v) = last_frame_id {
                        obj.insert("last_frame_id".to_string(), v);
                    }
                    if let Some(v) = last_ts {
                        obj.insert("last_timestamp_unix_ms".to_string(), v);
                    }
                }
            }

            windows
                .iter()
                .filter_map(|w| {
                    w.get("snapshots")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len() as u64)
                })
                .sum()
        };
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "snapshots_total".to_string(),
                serde_json::Value::from(snapshots_total),
            );
            obj.insert(
                "clipped".to_string(),
                serde_json::json!({
                    "schema_version": 1,
                    "max_snapshots_per_window": max_keep,
                    "reason": "bundle_index_exceeds_budget",
                }),
            );
        }

        write_json_compact(&path, &v)?;

        let new_bytes = file_bytes(&path)?;
        if new_bytes <= cfg.max_bundle_index_bytes {
            report.clipped_files.push("bundle.index.json".to_string());
            return Ok(());
        }

        if max_keep <= 8 {
            report.reason_code = Some("tooling.ai_packet.budget.bundle_index_exceeded".to_string());
            return Err(format!(
                "bundle.index.json exceeds budget even after clipping (bytes={} > max={})",
                new_bytes, cfg.max_bundle_index_bytes
            ));
        }

        max_keep = (max_keep * 2 / 3).max(8);
    }
}

fn clip_test_ids_index_if_needed(
    dir: &Path,
    cfg: &AiPacketBudgetConfig,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join("test_ids.index.json");
    if !path.is_file() {
        return Ok(());
    }
    let bytes = file_bytes(&path)?;
    if bytes <= cfg.max_test_ids_index_bytes {
        return Ok(());
    }

    let mut v = read_json(&path)?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("test_ids_index") {
        return Ok(());
    }

    let mut max_keep: usize = v
        .get("windows")
        .and_then(|v| v.as_array())
        .map(|windows| {
            windows
                .iter()
                .filter_map(|w| w.get("items").and_then(|v| v.as_array()).map(|a| a.len()))
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    max_keep = max_keep.min(20000).max(64);

    loop {
        let unique_total: u64 = {
            let windows = v
                .get_mut("windows")
                .and_then(|v| v.as_array_mut())
                .ok_or_else(|| "invalid test_ids.index.json: missing windows".to_string())?;

            for w in windows.iter_mut() {
                let Some(items) = w.get_mut("items").and_then(|v| v.as_array_mut()) else {
                    continue;
                };
                if items.len() > max_keep {
                    items.truncate(max_keep);
                }

                let unique = items.len() as u64;
                let count_sum: u64 = items
                    .iter()
                    .filter_map(|it| it.get("count").and_then(|v| v.as_u64()))
                    .sum();
                if let Some(obj) = w.as_object_mut() {
                    obj.insert(
                        "unique_test_ids_total".to_string(),
                        serde_json::Value::from(unique),
                    );
                    obj.insert(
                        "test_id_nodes_total".to_string(),
                        serde_json::Value::from(count_sum),
                    );
                }
            }

            windows
                .iter()
                .filter_map(|w| w.get("unique_test_ids_total").and_then(|v| v.as_u64()))
                .sum()
        };
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "total_unique_test_ids".to_string(),
                serde_json::Value::from(unique_total),
            );
            obj.insert("truncated".to_string(), serde_json::Value::from(true));
            obj.insert(
                "clipped".to_string(),
                serde_json::json!({
                    "schema_version": 1,
                    "max_items_per_window": max_keep,
                    "reason": "test_ids_index_exceeds_budget",
                }),
            );
        }

        write_json_compact(&path, &v)?;

        let new_bytes = file_bytes(&path)?;
        if new_bytes <= cfg.max_test_ids_index_bytes {
            report.clipped_files.push("test_ids.index.json".to_string());
            return Ok(());
        }

        if max_keep <= 64 {
            report.reason_code =
                Some("tooling.ai_packet.budget.test_ids_index_exceeded".to_string());
            return Err(format!(
                "test_ids.index.json exceeds budget even after clipping (bytes={} > max={})",
                new_bytes, cfg.max_test_ids_index_bytes
            ));
        }

        max_keep = (max_keep * 2 / 3).max(64);
    }
}

fn clip_frames_index_if_needed(
    dir: &Path,
    cfg: &AiPacketBudgetConfig,
    report: &mut AiPacketBudgetReport,
) -> Result<(), String> {
    let path = dir.join("frames.index.json");
    if !path.is_file() {
        return Ok(());
    }
    let bytes = file_bytes(&path)?;
    if bytes <= cfg.max_frames_index_bytes {
        return Ok(());
    }

    let mut v = read_json(&path)?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("frames_index") {
        return Ok(());
    }

    let mut max_keep: usize = v
        .get("windows")
        .and_then(|v| v.as_array())
        .map(|windows| {
            windows
                .iter()
                .filter_map(|w| w.get("rows").and_then(|v| v.as_array()).map(|a| a.len()))
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    max_keep = max_keep.min(8192).max(64);

    loop {
        let frames_total: u64 = {
            let windows = v
                .get_mut("windows")
                .and_then(|v| v.as_array_mut())
                .ok_or_else(|| "invalid frames.index.json: missing windows".to_string())?;

            for w in windows.iter_mut() {
                let rows_len = {
                    let Some(rows) = w.get_mut("rows").and_then(|v| v.as_array_mut()) else {
                        continue;
                    };
                    if rows.len() > max_keep {
                        let len = rows.len();
                        rows.drain(0..(len - max_keep));
                    }
                    rows.len() as u64
                };
                if let Some(obj) = w.as_object_mut() {
                    obj.insert(
                        "frames_total".to_string(),
                        serde_json::Value::from(rows_len),
                    );
                }
            }

            windows
                .iter()
                .filter_map(|w| {
                    w.get("rows")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len() as u64)
                })
                .sum()
        };

        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "frames_total".to_string(),
                serde_json::Value::from(frames_total),
            );
            obj.insert(
                "clipped".to_string(),
                serde_json::json!({
                    "schema_version": 1,
                    "max_rows_per_window": max_keep,
                    "reason": "frames_index_exceeds_budget",
                }),
            );
        }

        write_json_compact(&path, &v)?;

        let new_bytes = file_bytes(&path)?;
        if new_bytes <= cfg.max_frames_index_bytes {
            report.clipped_files.push("frames.index.json".to_string());
            return Ok(());
        }

        if max_keep <= 32 {
            // frames.index.json is optional: drop it instead of failing the whole packet.
            drop_if_present(dir, "frames.index.json", report)?;
            return Ok(());
        }

        max_keep = (max_keep * 2 / 3).max(32);
    }
}

fn build_slice_payload_with_budget(
    bundle_path: &Path,
    warmup_frames: u64,
    test_id: &str,
    max_bytes: u64,
) -> Result<serde_json::Value, String> {
    let configs = [(20usize, 64usize), (10, 32), (5, 16), (3, 8)];
    for (max_matches, max_ancestors) in configs {
        let payload = super::slice::build_test_id_slice_payload_from_bundle_path(
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
        let payload = super::slice::build_test_id_slice_payload_from_bundle_path(
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

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
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

fn resolve_bundle_json_path_or_latest(
    bundle_arg: Option<&str>,
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    if let Some(s) = bundle_arg {
        let src = crate::resolve_path(workspace_root, PathBuf::from(s));
        return Ok(crate::resolve_bundle_json_path(&src));
    }
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(crate::resolve_bundle_json_path(&latest))
}

fn copy_file_named(src: &Path, dir: &Path, name: &str) -> Result<(), String> {
    let dst = dir.join(name);
    std::fs::copy(src, dst).map_err(|e| e.to_string())?;
    Ok(())
}

fn copy_if_present(src: &Path, dir: &Path, name: &str) -> Result<(), String> {
    if src.is_file() {
        copy_file_named(src, dir, name)?;
    }
    Ok(())
}
