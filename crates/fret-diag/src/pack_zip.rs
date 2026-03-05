use std::path::{Path, PathBuf};

use zip::write::FileOptions;

use crate::stats::{BundleStatsOptions, BundleStatsSort, bundle_stats_from_path};

pub(crate) fn pack_ai_packet_dir_to_zip(
    bundle_dir: &Path,
    out_path: &Path,
    artifacts_root: &Path,
) -> Result<(), String> {
    if !bundle_dir.is_dir() {
        return Err(format!(
            "bundle_dir is not a directory: {}",
            bundle_dir.display()
        ));
    }

    let packet_dir = bundle_dir.join("ai.packet");
    if !packet_dir.is_dir() {
        return Err(format!(
            "bundle_dir does not contain ai.packet (tip: fretboard diag ai-packet {} --packet-out {})",
            bundle_dir.display(),
            packet_dir.display()
        ));
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let bundle_name = bundle_dir
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Repro workflow helper: if a repro summary exists next to the bundle output root, include it.
    let repro_summary = artifacts_root.join("repro.summary.json");
    if repro_summary.is_file() {
        let dst = format!("{bundle_name}/_root/repro.summary.json");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&repro_summary).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
    }

    // Include script sources when present (small but often essential for agentic triage).
    for name in ["script.json", "picked.script.json"] {
        let src = artifacts_root.join(name);
        if !src.is_file() {
            continue;
        }
        let dst = format!("{bundle_name}/_root/{name}");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&src).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
    }

    // Pack the packet directory under `_root/ai.packet/`.
    let packet_prefix = format!("{bundle_name}/_root/ai.packet");
    zip_add_dir_filtered(
        &mut zip,
        &packet_dir,
        &packet_dir,
        &packet_prefix,
        options,
        &["json", "md", "txt"],
    )?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn pack_repro_ai_zip_multi(
    out_path: &Path,
    artifacts_root: &Path,
    summary_path: &Path,
    bundles: &[ReproZipBundle],
) -> Result<(), String> {
    use std::io::Write;

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Always include a machine-readable repro summary.
    if summary_path.is_file() {
        let bytes = std::fs::read(summary_path).map_err(|e| e.to_string())?;
        zip.start_file("_root/repro.summary.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    // Include script sources for offline triage.
    for (idx, item) in bundles.iter().enumerate() {
        let bytes = std::fs::read(&item.source_script).map_err(|e| e.to_string())?;
        let name = item
            .source_script
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("script.json");
        let safe = zip_safe_component(name);
        let dst = format!("_root/scripts/{:02}-{safe}", idx.saturating_add(1));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    // Include nearby script sources (common for single-script repros).
    for name in ["script.json", "picked.script.json"] {
        let src = artifacts_root.join(name);
        if !src.is_file() {
            continue;
        }
        zip.start_file(format!("_root/{name}"), options)
            .map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&src).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
    }

    // Pack each script prefix's ai.packet under its `_root/`.
    for item in bundles {
        let bundle_dir = crate::resolve_bundle_root_dir(&item.bundle_artifact)?;
        let packet_dir = bundle_dir.join("ai.packet");
        if !packet_dir.is_dir() {
            return Err(format!(
                "missing ai.packet for repro item {} (tip: fretboard diag ai-packet {} --packet-out {})",
                item.prefix,
                bundle_dir.display(),
                packet_dir.display()
            ));
        }

        let zip_prefix = format!("{}/_root/ai.packet", item.prefix);
        zip_add_dir_filtered(
            &mut zip,
            &packet_dir,
            &packet_dir,
            &zip_prefix,
            options,
            &["json", "md", "txt"],
        )?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn pack_bundle_dir_to_zip(
    bundle_dir: &Path,
    out_path: &Path,
    include_root_artifacts: bool,
    include_triage: bool,
    include_screenshots: bool,
    schema2_only: bool,
    include_renderdoc: bool,
    include_tracy: bool,
    artifacts_root: &Path,
    stats_top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> Result<(), String> {
    if !bundle_dir.is_dir() {
        return Err(format!(
            "bundle_dir is not a directory: {}",
            bundle_dir.display()
        ));
    }

    let bundle_artifact = crate::resolve_bundle_artifact_path(bundle_dir);
    if !bundle_artifact.is_file() {
        return Err(format!(
            "bundle_dir does not contain a bundle artifact (bundle.json or bundle.schema2.json): {}",
            bundle_dir.display()
        ));
    }

    if schema2_only {
        if crate::resolve_bundle_schema2_artifact_path_no_materialize(bundle_dir).is_none() {
            return Err(format!(
                "--pack-schema2-only requires bundle.schema2.json (tip: fretboard diag doctor --fix-schema2 {} --warmup-frames {})",
                bundle_dir.display(),
                warmup_frames
            ));
        }
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let bundle_name = bundle_dir
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    zip_add_dir_pack_bundle(
        &mut zip,
        bundle_dir,
        bundle_dir,
        bundle_name,
        out_path,
        options,
        schema2_only,
    )?;

    // Repro workflow helper: if a repro summary exists next to the bundle output root, include it.
    let repro_summary = artifacts_root.join("repro.summary.json");
    if repro_summary.is_file() {
        let dst = format!("{bundle_name}/_root/repro.summary.json");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&repro_summary).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
    }

    if include_root_artifacts {
        let root_prefix = format!("{bundle_name}/_root");
        zip_add_root_artifacts(&mut zip, artifacts_root, &root_prefix, options)?;
    }

    if include_renderdoc {
        let renderdoc_dir = artifacts_root.join("renderdoc");
        if renderdoc_dir.is_dir() {
            let renderdoc_prefix = format!("{bundle_name}/_root/renderdoc");
            zip_add_dir_filtered(
                &mut zip,
                &renderdoc_dir,
                &renderdoc_dir,
                &renderdoc_prefix,
                options,
                &["rdc", "json", "png", "txt", "md", "csv"],
            )?;
        }
    }

    if include_tracy {
        let tracy_dir = artifacts_root.join("tracy");
        if tracy_dir.is_dir() {
            let tracy_prefix = format!("{bundle_name}/_root/tracy");
            zip_add_dir_filtered(
                &mut zip,
                &tracy_dir,
                &tracy_dir,
                &tracy_prefix,
                options,
                &["tracy", "txt", "md", "json"],
            )?;
        }
    }

    if include_screenshots {
        let screenshots_dir = artifacts_root.join("screenshots").join(bundle_name);
        if screenshots_dir.is_dir() {
            let screenshots_prefix = format!("{bundle_name}/_root/screenshots");
            zip_add_screenshots(&mut zip, &screenshots_dir, &screenshots_prefix, options)?;
        }
    }

    if include_root_artifacts || include_triage {
        let meta_path =
            crate::bundle_index::ensure_bundle_meta_json(&bundle_artifact, warmup_frames)?;
        let bundle_index_path =
            crate::bundle_index::ensure_bundle_index_json(&bundle_artifact, warmup_frames)?;
        let test_ids_index_path =
            crate::bundle_index::ensure_test_ids_index_json(&bundle_artifact, warmup_frames)?;
        let test_ids_path =
            crate::bundle_index::ensure_test_ids_json(&bundle_artifact, warmup_frames, 500)?;
        let frames_index_path =
            crate::frames_index::ensure_frames_index_json(&bundle_artifact, warmup_frames)?;
        let window_map_path =
            crate::bundle_index::ensure_window_map_json(&bundle_artifact, warmup_frames)?;
        let dock_routing_path =
            crate::bundle_index::ensure_dock_routing_json(&bundle_artifact, warmup_frames)?;

        for (src, rel) in [
            (meta_path, "bundle.meta.json"),
            (bundle_index_path, "bundle.index.json"),
            (test_ids_index_path, "test_ids.index.json"),
            (test_ids_path, "test_ids.json"),
            (frames_index_path, "frames.index.json"),
            (window_map_path, "window.map.json"),
            (dock_routing_path, "dock.routing.json"),
        ] {
            if src.is_file() {
                let dst = format!("{bundle_name}/_root/{rel}");
                zip.start_file(dst, options).map_err(|e| e.to_string())?;
                let mut f = std::fs::File::open(&src).map_err(|e| e.to_string())?;
                std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
            }
        }

        // Optional layout explainability sidecar produced by scripted diagnostics runs.
        if let Some(parent) = bundle_artifact.parent() {
            let path = parent.join("layout.taffy.v1.json");
            if path.is_file() {
                let dst = format!("{bundle_name}/_root/layout.taffy.v1.json");
                zip.start_file(dst, options).map_err(|e| e.to_string())?;
                let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
                std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
            }
        }
    }

    if include_triage {
        use std::io::Write;

        let report = bundle_stats_from_path(
            &bundle_artifact,
            stats_top,
            sort,
            BundleStatsOptions { warmup_frames },
        )?;
        let payload = crate::triage_json_from_stats(&bundle_artifact, &report, sort, warmup_frames);
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        let dst = format!("{bundle_name}/_root/triage.json");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct ReproZipBundle {
    pub(crate) prefix: String,
    pub(crate) bundle_artifact: PathBuf,
    pub(crate) source_script: PathBuf,
}

pub(crate) fn repro_zip_prefix_for_script(item: &crate::ReproPackItem, idx: usize) -> String {
    let stem = item
        .script_path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("script");
    let safe = zip_safe_component(stem);
    format!("{:02}-{safe}", idx.saturating_add(1))
}

pub(crate) fn zip_safe_component(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        let keep = ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.');
        if keep {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "bundle".to_string()
    } else {
        trimmed.to_string()
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn pack_repro_zip_multi(
    out_path: &Path,
    include_root_artifacts: bool,
    include_triage: bool,
    include_screenshots: bool,
    schema2_only: bool,
    include_renderdoc: bool,
    include_tracy: bool,
    artifacts_root: &Path,
    summary_path: &Path,
    bundles: &[ReproZipBundle],
    stats_top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> Result<(), String> {
    use std::io::Write;

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Always include a machine-readable repro summary.
    if summary_path.is_file() {
        let bytes = std::fs::read(summary_path).map_err(|e| e.to_string())?;
        zip.start_file("_root/repro.summary.json", options)
            .map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    // Include script sources for offline triage.
    for (idx, item) in bundles.iter().enumerate() {
        let bytes = std::fs::read(&item.source_script).map_err(|e| e.to_string())?;
        let name = item
            .source_script
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("script.json");
        let safe = zip_safe_component(name);
        let dst = format!("_root/scripts/{:02}-{safe}", idx.saturating_add(1));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    if include_root_artifacts {
        zip_add_root_artifacts(&mut zip, artifacts_root, "_root", options)?;
    }

    if include_renderdoc {
        let renderdoc_dir = artifacts_root.join("renderdoc");
        if renderdoc_dir.is_dir() {
            zip_add_dir_filtered(
                &mut zip,
                &renderdoc_dir,
                &renderdoc_dir,
                "_root/renderdoc",
                options,
                &["rdc", "json", "png", "txt", "md", "csv"],
            )?;
        }
    }

    if include_tracy {
        let tracy_dir = artifacts_root.join("tracy");
        if tracy_dir.is_dir() {
            zip_add_dir_filtered(
                &mut zip,
                &tracy_dir,
                &tracy_dir,
                "_root/tracy",
                options,
                &["tracy", "txt", "md", "json"],
            )?;
        }
    }

    for item in bundles {
        let bundle_dir = crate::resolve_bundle_root_dir(&item.bundle_artifact)?;
        if schema2_only {
            if crate::resolve_bundle_schema2_artifact_path_no_materialize(&bundle_dir).is_none() {
                return Err(format!(
                    "--pack-schema2-only requires bundle.schema2.json (tip: fretboard diag doctor --fix-schema2 {} --warmup-frames {})",
                    bundle_dir.display(),
                    warmup_frames
                ));
            }
        }
        zip_add_dir_pack_bundle(
            &mut zip,
            &bundle_dir,
            &bundle_dir,
            &item.prefix,
            out_path,
            options,
            schema2_only,
        )?;

        if include_root_artifacts || include_triage {
            let bundle_artifact = crate::resolve_bundle_artifact_path(&bundle_dir);
            let meta_path =
                crate::bundle_index::ensure_bundle_meta_json(&bundle_artifact, warmup_frames)?;
            let bundle_index_path =
                crate::bundle_index::ensure_bundle_index_json(&bundle_artifact, warmup_frames)?;
            let test_ids_index_path =
                crate::bundle_index::ensure_test_ids_index_json(&bundle_artifact, warmup_frames)?;
            let test_ids_path =
                crate::bundle_index::ensure_test_ids_json(&bundle_artifact, warmup_frames, 500)?;
            let frames_index_path =
                crate::frames_index::ensure_frames_index_json(&bundle_artifact, warmup_frames)?;
            let window_map_path =
                crate::bundle_index::ensure_window_map_json(&bundle_artifact, warmup_frames)?;
            let dock_routing_path =
                crate::bundle_index::ensure_dock_routing_json(&bundle_artifact, warmup_frames)?;

            for (src, rel) in [
                (meta_path, "bundle.meta.json"),
                (bundle_index_path, "bundle.index.json"),
                (test_ids_index_path, "test_ids.index.json"),
                (test_ids_path, "test_ids.json"),
                (frames_index_path, "frames.index.json"),
                (window_map_path, "window.map.json"),
                (dock_routing_path, "dock.routing.json"),
            ] {
                if src.is_file() {
                    let dst = format!("{}/_root/{rel}", item.prefix);
                    zip.start_file(dst, options).map_err(|e| e.to_string())?;
                    let mut f = std::fs::File::open(&src).map_err(|e| e.to_string())?;
                    std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
                }
            }
        }

        if include_screenshots {
            let bundle_name = bundle_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let screenshots_dir = artifacts_root.join("screenshots").join(bundle_name);
            if screenshots_dir.is_dir() {
                let screenshots_prefix = format!("{}/_root/screenshots", item.prefix);
                zip_add_screenshots(&mut zip, &screenshots_dir, &screenshots_prefix, options)?;
            }
        }

        if include_triage {
            let report = bundle_stats_from_path(
                &item.bundle_artifact,
                stats_top,
                sort,
                BundleStatsOptions { warmup_frames },
            )?;
            let payload =
                crate::triage_json_from_stats(&item.bundle_artifact, &report, sort, warmup_frames);
            let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
            let dst = format!("{}/_root/triage.json", item.prefix);
            zip.start_file(dst, options).map_err(|e| e.to_string())?;
            zip.write_all(&bytes).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn zip_add_root_artifacts(
    zip: &mut zip::ZipWriter<std::fs::File>,
    artifacts_root: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<(), String> {
    let candidates = [
        "evidence.index.json",
        "script.json",
        "script.result.json",
        "pick.result.json",
        "screenshots.result.json",
        "triage.json",
        "picked.script.json",
        "check.semantics_changed_repainted.json",
        "check.pixels_changed.json",
        "check.idle_no_paint.json",
        "check.perf_thresholds.json",
        "check.perf_hints.json",
        "check.redraw_hitches.json",
        "check.wgpu_metal_allocated_size.json",
        "check.wgpu_hub_counts.json",
        "check.resource_footprint.json",
        "check.view_cache_reuse_stable.json",
        "resource.footprint.json",
        "resource.vmmap_summary.steady.txt",
        "resource.vmmap_regions_sorted.steady.txt",
        "resource.vmmap_summary.txt",
        "redraw_hitches.log",
        "renderdoc.captures.json",
        "tracy.note.md",
    ];

    for name in candidates {
        let src = artifacts_root.join(name);
        if !src.is_file() {
            continue;
        }
        let dst = format!("{zip_prefix}/{name}");
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&src).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_add_screenshots(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<(), String> {
    zip_add_screenshot_dir(zip, dir, dir, zip_prefix, options)
}

fn zip_add_screenshot_dir(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base_dir: &Path,
    zip_prefix: &str,
    options: FileOptions,
) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            zip_add_screenshot_dir(zip, &path, base_dir, zip_prefix, options)?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        // Keep this conservative to avoid exploding zip sizes accidentally.
        let should_include = matches!(ext.as_str(), "png") || name == "manifest.json";
        if !should_include {
            continue;
        }

        let rel = path
            .strip_prefix(base_dir)
            .map_err(|_| "failed to compute zip relative path".to_string())?;

        let dst = format!("{}/{}", zip_prefix, zip_name(rel));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_add_dir_filtered(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base_dir: &Path,
    zip_prefix: &str,
    options: FileOptions,
    allowed_exts: &[&str],
) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            zip_add_dir_filtered(zip, &path, base_dir, zip_prefix, options, allowed_exts)?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if ext.is_empty() {
            continue;
        }
        if !allowed_exts
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(ext.as_str()))
        {
            continue;
        }

        let rel = path
            .strip_prefix(base_dir)
            .map_err(|_| "failed to compute zip relative path".to_string())?;

        let dst = format!("{}/{}", zip_prefix, zip_name(rel));
        zip.start_file(dst, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_add_dir_pack_bundle(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base_dir: &Path,
    prefix: &str,
    out_path: &Path,
    options: FileOptions,
    schema2_only: bool,
) -> Result<(), String> {
    fn rel_starts_with_components(rel: &Path, head: &str, next: &str) -> bool {
        let mut it = rel.components();
        let a = it.next().and_then(|c| c.as_os_str().to_str());
        let b = it.next().and_then(|c| c.as_os_str().to_str());
        a.is_some_and(|v| v.eq_ignore_ascii_case(head))
            && b.is_some_and(|v| v.eq_ignore_ascii_case(next))
    }

    fn should_skip_path(rel: &Path) -> bool {
        if rel
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("bundle.json"))
        {
            let parent = rel.parent();
            let is_root = parent.is_none()
                || parent.is_some_and(|p| p.as_os_str().is_empty())
                || parent.is_some_and(|p| p == Path::new("_root"));
            if is_root {
                return true;
            }
        }

        // Avoid packing the raw bundle.json chunk directory when shipping schema2-only zips.
        if rel_starts_with_components(rel, "chunks", "bundle_json") {
            return true;
        }

        false
    }

    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path == out_path {
            continue;
        }

        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }

        let rel = path
            .strip_prefix(base_dir)
            .map_err(|_| "failed to compute zip relative path".to_string())?;
        if schema2_only && should_skip_path(rel) {
            continue;
        }

        if meta.is_dir() {
            zip_add_dir_pack_bundle(
                zip,
                &path,
                base_dir,
                prefix,
                out_path,
                options,
                schema2_only,
            )?;
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        let name = format!("{}/{}", prefix, zip_name(rel));
        zip.start_file(name, options).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn zip_name(path: &Path) -> String {
    let mut out = String::new();
    for (i, c) in path.components().enumerate() {
        if i > 0 {
            out.push('/');
        }
        out.push_str(&c.as_os_str().to_string_lossy());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root(prefix: &str) -> PathBuf {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        std::env::temp_dir().join(format!("{prefix}-{ms}"))
    }

    #[test]
    fn pack_bundle_dir_to_zip_accepts_schema2_only() {
        let root = unique_temp_root("fret-diag-pack-schema2-only");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let bundle_dir = root.join("bundle");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(
            bundle_dir.join("bundle.schema2.json"),
            "{\"schema_version\":2}",
        )
        .expect("write bundle.schema2.json");
        std::fs::create_dir_all(bundle_dir.join("chunks").join("bundle_json"))
            .expect("create raw bundle.json chunks dir");
        std::fs::write(
            bundle_dir
                .join("chunks")
                .join("bundle_json")
                .join("0000.json"),
            "{\"schema_version\":1}",
        )
        .expect("write chunk");
        std::fs::write(bundle_dir.join("bundle.json"), "{\"schema_version\":1}")
            .expect("write raw bundle.json");

        let artifacts_root = root.join("artifacts");
        std::fs::create_dir_all(&artifacts_root).expect("create artifacts root");

        let out_path = root.join("out.zip");
        pack_bundle_dir_to_zip(
            &bundle_dir,
            &out_path,
            false,
            false,
            false,
            true,
            false,
            false,
            &artifacts_root,
            1,
            BundleStatsSort::Invalidation,
            0,
        )
        .expect("pack schema2-only zip");

        let f = std::fs::File::open(out_path).expect("open out zip");
        let mut zip = zip::ZipArchive::new(f).expect("open zip archive");
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).expect("zip entry").name().to_string())
            .collect();

        assert!(
            !names.iter().any(|n| n.ends_with("/bundle.json")),
            "bundle.json should be skipped when packing schema2-only"
        );
        assert!(
            !names.iter().any(
                |n| n.contains("/chunks/bundle_json/") || n.contains("\\chunks\\bundle_json\\")
            ),
            "raw bundle.json chunks should be skipped when packing schema2-only"
        );
    }

    #[test]
    fn pack_repro_zip_multi_includes_sidecars_under_root() {
        let root = unique_temp_root("fret-diag-pack-repro-multi-sidecars");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let artifacts_root = root.join("artifacts");
        std::fs::create_dir_all(&artifacts_root).expect("create artifacts root");

        let summary_path = artifacts_root.join("repro.summary.json");
        std::fs::write(&summary_path, "{\"schema_version\":1}").expect("write summary");

        let make_bundle = |name: &str| -> (PathBuf, PathBuf) {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).expect("create bundle dir");
            std::fs::write(
                dir.join("bundle.schema2.json"),
                "{\"schema_version\":2,\"windows\":[]}",
            )
            .expect("write bundle.schema2.json");
            let script = root.join(format!("{name}.json"));
            std::fs::write(&script, "{\"schema_version\":1,\"steps\":[]}").expect("write script");
            (dir.join("bundle.schema2.json"), script)
        };

        let (bundle_a, script_a) = make_bundle("a");
        let (bundle_b, script_b) = make_bundle("b");

        let bundles = vec![
            ReproZipBundle {
                prefix: "01-a".to_string(),
                bundle_artifact: bundle_a,
                source_script: script_a,
            },
            ReproZipBundle {
                prefix: "02-b".to_string(),
                bundle_artifact: bundle_b,
                source_script: script_b,
            },
        ];

        let out_path = root.join("repro.zip");
        pack_repro_zip_multi(
            &out_path,
            true,
            false,
            false,
            true,
            false,
            false,
            &artifacts_root,
            &summary_path,
            &bundles,
            1,
            BundleStatsSort::Invalidation,
            0,
        )
        .expect("pack repro zip");

        let f = std::fs::File::open(out_path).expect("open out zip");
        let mut zip = zip::ZipArchive::new(f).expect("open zip archive");
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).expect("zip entry").name().to_string())
            .collect();

        for prefix in ["01-a", "02-b"] {
            assert!(
                names
                    .iter()
                    .any(|n| n == &format!("{prefix}/_root/bundle.meta.json")),
                "{prefix} bundle.meta.json should be included under _root"
            );
            assert!(
                names
                    .iter()
                    .any(|n| n == &format!("{prefix}/_root/bundle.index.json")),
                "{prefix} bundle.index.json should be included under _root"
            );
            assert!(
                names
                    .iter()
                    .any(|n| n == &format!("{prefix}/_root/test_ids.index.json")),
                "{prefix} test_ids.index.json should be included under _root"
            );
            assert!(
                names
                    .iter()
                    .any(|n| n == &format!("{prefix}/_root/test_ids.json")),
                "{prefix} test_ids.json should be included under _root"
            );
            assert!(
                names
                    .iter()
                    .any(|n| n == &format!("{prefix}/_root/frames.index.json")),
                "{prefix} frames.index.json should be included under _root"
            );
        }
    }

    #[test]
    fn pack_ai_packet_dir_to_zip_packs_only_bounded_ai_artifacts() {
        let root = unique_temp_root("fret-diag-pack-ai-only");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let bundle_dir = root.join("bundle");
        let packet_dir = bundle_dir.join("ai.packet");
        std::fs::create_dir_all(&packet_dir).expect("create ai.packet dir");
        std::fs::write(
            packet_dir.join("bundle.meta.json"),
            "{\"schema_version\":1}",
        )
        .expect("write bundle.meta.json");
        std::fs::write(packet_dir.join("doctor.json"), "{\"schema_version\":1}")
            .expect("write doctor.json");
        std::fs::write(bundle_dir.join("bundle.json"), "{\"schema_version\":2}")
            .expect("write raw bundle.json");
        std::fs::write(
            bundle_dir.join("bundle.schema2.json"),
            "{\"schema_version\":2}",
        )
        .expect("write bundle.schema2.json");

        let artifacts_root = root.join("artifacts");
        std::fs::create_dir_all(&artifacts_root).expect("create artifacts root");
        std::fs::write(
            artifacts_root.join("repro.summary.json"),
            "{\"schema_version\":1}",
        )
        .expect("write repro.summary.json");
        std::fs::write(
            artifacts_root.join("script.json"),
            "{\"schema_version\":1,\"steps\":[]}",
        )
        .expect("write script.json");

        let out_path = root.join("ai.zip");
        pack_ai_packet_dir_to_zip(&bundle_dir, &out_path, &artifacts_root)
            .expect("pack ai-only zip");

        let f = std::fs::File::open(out_path).expect("open out zip");
        let mut zip = zip::ZipArchive::new(f).expect("open zip archive");
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).expect("zip entry").name().to_string())
            .collect();

        assert!(
            names
                .iter()
                .any(|n| n.ends_with("/_root/ai.packet/bundle.meta.json")),
            "expected ai.packet/bundle.meta.json in zip"
        );
        assert!(
            names
                .iter()
                .any(|n| n.ends_with("/_root/ai.packet/doctor.json")),
            "expected ai.packet/doctor.json in zip"
        );
        assert!(
            names
                .iter()
                .any(|n| n.ends_with("/_root/repro.summary.json")),
            "expected repro.summary.json in zip"
        );
        assert!(
            names.iter().any(|n| n.ends_with("/_root/script.json")),
            "expected script.json in zip"
        );

        assert!(
            !names.iter().any(|n| n.ends_with("/bundle.json")),
            "ai-only zip must not include bundle.json"
        );
        assert!(
            !names.iter().any(|n| n.ends_with("/bundle.schema2.json")),
            "ai-only zip must not include bundle.schema2.json"
        );
    }

    #[test]
    fn pack_repro_ai_zip_multi_packs_only_ai_packet_and_scripts() {
        let root = unique_temp_root("fret-diag-pack-repro-ai-only");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let artifacts_root = root.join("artifacts");
        std::fs::create_dir_all(&artifacts_root).expect("create artifacts root");

        let summary_path = artifacts_root.join("repro.summary.json");
        std::fs::write(&summary_path, "{\"schema_version\":1}").expect("write summary");

        let make_bundle = |name: &str| -> (PathBuf, PathBuf) {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).expect("create bundle dir");
            std::fs::write(
                dir.join("bundle.schema2.json"),
                "{\"schema_version\":2,\"windows\":[]}",
            )
            .expect("write bundle.schema2.json");

            let packet_dir = dir.join("ai.packet");
            std::fs::create_dir_all(&packet_dir).expect("create ai.packet dir");
            std::fs::write(
                packet_dir.join("doctor.json"),
                "{\"schema_version\":1,\"ok\":true}",
            )
            .expect("write doctor.json");

            let script = root.join(format!("{name}.json"));
            std::fs::write(&script, "{\"schema_version\":1,\"steps\":[]}").expect("write script");
            (dir.join("bundle.schema2.json"), script)
        };

        let (bundle_a, script_a) = make_bundle("a");
        let (bundle_b, script_b) = make_bundle("b");

        let bundles = vec![
            ReproZipBundle {
                prefix: "01-a".to_string(),
                bundle_artifact: bundle_a,
                source_script: script_a,
            },
            ReproZipBundle {
                prefix: "02-b".to_string(),
                bundle_artifact: bundle_b,
                source_script: script_b,
            },
        ];

        let out_path = root.join("repro.ai.zip");
        pack_repro_ai_zip_multi(&out_path, &artifacts_root, &summary_path, &bundles)
            .expect("pack repro.ai zip");

        let f = std::fs::File::open(out_path).expect("open out zip");
        let mut zip = zip::ZipArchive::new(f).expect("open zip archive");
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).expect("zip entry").name().to_string())
            .collect();

        assert!(
            names.iter().any(|n| n == "_root/repro.summary.json"),
            "expected _root/repro.summary.json in zip"
        );
        assert!(
            names.iter().any(|n| n == "_root/scripts/01-a.json"),
            "expected _root/scripts/01-a.json in zip"
        );
        assert!(
            names.iter().any(|n| n == "_root/scripts/02-b.json"),
            "expected _root/scripts/02-b.json in zip"
        );

        for prefix in ["01-a", "02-b"] {
            assert!(
                names
                    .iter()
                    .any(|n| n == &format!("{prefix}/_root/ai.packet/doctor.json")),
                "{prefix} expected ai.packet/doctor.json under _root"
            );
        }

        assert!(
            !names.iter().any(|n| n.ends_with("/bundle.json")),
            "repro.ai zip must not include bundle.json"
        );
        assert!(
            !names.iter().any(|n| n.ends_with("/bundle.schema2.json")),
            "repro.ai zip must not include bundle.schema2.json"
        );
    }
}
