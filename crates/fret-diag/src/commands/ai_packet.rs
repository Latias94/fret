use std::path::{Path, PathBuf};

use super::doctor;

use crate::frames_index::TriageLiteMetric;
use crate::stats::{BundleStatsOptions, BundleStatsSort, bundle_stats_from_path};

mod anchors;
mod budget;
mod fs;
mod slices;

#[derive(Debug, Clone)]
struct AiPacketBudgetConfig {
    soft_total_bytes: u64,
    hard_total_bytes: u64,

    max_bundle_meta_bytes: u64,
    max_bundle_schema2_bytes: u64,
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
            max_bundle_schema2_bytes: 8 * 1024 * 1024,
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
        resolve_bundle_artifact_path_or_latest(bundle_arg.as_deref(), workspace_root, out_dir)?;
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

    fs::copy_file_named(&meta_path, &packet_dir, "bundle.meta.json")?;
    fs::copy_file_named(&test_ids_index_path, &packet_dir, "test_ids.index.json")?;
    fs::copy_file_named(&bundle_index_path, &packet_dir, "bundle.index.json")?;
    fs::copy_file_named(&frames_index_path, &packet_dir, "frames.index.json")?;

    fs::copy_bundle_schema2_if_present(&bundle_path, &bundle_dir, &packet_dir)?;

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
        budget::write_json_compact(&packet_dir.join("triage.lite.json"), &triage_lite)?;

        let hotspots_lite = crate::hotspots_lite::hotspots_lite_json_from_frames_index(
            &bundle_path,
            &frames_index_path,
            &frames_index,
            warmup_frames,
            50,
            TriageLiteMetric::TotalTimeUs,
        )?;
        budget::write_json_compact(&packet_dir.join("hotspots.lite.json"), &hotspots_lite)?;
    }

    fs::copy_if_present(
        &bundle_dir.join("script.result.json"),
        &packet_dir,
        "script.result.json",
    )?;
    fs::copy_if_present(
        &bundle_dir.join("manifest.json"),
        &packet_dir,
        "manifest.json",
    )?;

    crate::util::write_json_value(
        &packet_dir.join("doctor.json"),
        &doctor::doctor_report_json(&bundle_path, warmup_frames),
    )?;

    anchors::write_packet_anchors_if_possible(&packet_dir)?;

    let mut failed_step_slices_report: Option<AiPacketFailedStepSlicesReportV1> = None;
    if test_id.is_none() {
        failed_step_slices_report = slices::write_anchor_slices_if_possible(
            &bundle_path,
            warmup_frames,
            &packet_dir,
            AiPacketBudgetConfig::default().max_slice_bytes,
        )?;
    }

    if include_triage {
        let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);
        match bundle_stats_from_path(
            &bundle_path,
            stats_top,
            sort,
            BundleStatsOptions { warmup_frames },
        ) {
            Ok(report) => {
                let payload =
                    crate::triage_json_from_stats(&bundle_path, &report, sort, warmup_frames);
                crate::util::write_json_value(&packet_dir.join("triage.json"), &payload)?;
            }
            Err(err) => {
                eprintln!("ai-packet: failed to generate triage.json: {err}");
                crate::util::write_json_value(
                    &packet_dir.join("triage.error.json"),
                    &serde_json::json!({
                        "schema_version": 1,
                        "kind": "diag.ai_packet_note",
                        "bundle": bundle_path.display().to_string(),
                        "warmup_frames": warmup_frames,
                        "message": "Failed to generate triage.json; falling back to triage.lite.json.",
                        "error": err,
                        "suggestions": [
                            format!("fretboard diag triage {} --warmup-frames {}", bundle_path.display(), warmup_frames),
                            format!("fretboard diag triage --lite {} --warmup-frames {}", bundle_path.display(), warmup_frames),
                        ],
                    }),
                )?;
            }
        }
    }

    if let Some(test_id) = &test_id {
        let budget_cfg = AiPacketBudgetConfig::default();
        let payload = slices::build_slice_payload_with_budget(
            &bundle_path,
            warmup_frames,
            test_id.as_str(),
            budget_cfg.max_slice_bytes,
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
    let enforce_res = budget::enforce_ai_packet_budgets(&packet_dir, &mut report);
    budget::write_packet_budget_report(&packet_dir, &report)?;
    enforce_res?;

    println!("{}", packet_dir.display());
    Ok(())
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
}

fn resolve_bundle_artifact_path_or_latest(
    bundle_arg: Option<&str>,
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    if let Some(s) = bundle_arg {
        let src = crate::resolve_path(workspace_root, PathBuf::from(s));
        return Ok(crate::resolve_bundle_artifact_path(&src));
    }
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(crate::resolve_bundle_artifact_path(&latest))
}
