use std::path::{Path, PathBuf};

use crate::stats::{BundleStatsOptions, BundleStatsSort, bundle_stats_from_path};

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

    copy_file_named(&meta_path, &packet_dir, "bundle.meta.json")?;
    copy_file_named(&test_ids_index_path, &packet_dir, "test_ids.index.json")?;
    copy_file_named(&bundle_index_path, &packet_dir, "bundle.index.json")?;

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
        let bytes = std::fs::read(&bundle_path).map_err(|e| e.to_string())?;
        let bundle: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let semantics = crate::json_bundle::SemanticsResolver::new(&bundle);
        let payload = super::slice::build_test_id_slice_payload_from_bundle(
            &bundle_path,
            &bundle,
            &semantics,
            warmup_frames,
            test_id.as_str(),
            None,
            None,
            None,
            20,
            64,
        )?;
        let stem = crate::util::sanitize_for_filename(test_id, 80, "test_id");
        crate::util::write_json_value(
            &packet_dir.join(format!("slice.test_id.{stem}.json")),
            &payload,
        )?;
        crate::util::write_json_value(&packet_dir.join(format!("slice.{stem}.json")), &payload)?;
    }

    println!("{}", packet_dir.display());
    Ok(())
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
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
