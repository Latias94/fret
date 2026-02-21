use std::path::{Path, PathBuf};

use crate::lint::{LintOptions, lint_bundle_from_path};
use crate::stats::{BundleStatsOptions, BundleStatsSort, bundle_stats_from_path};

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_pack(
    rest: &[String],
    workspace_root: &Path,
    out_dir: &Path,
    pack_out: Option<PathBuf>,
    pack_include_root_artifacts: bool,
    pack_include_triage: bool,
    pack_include_screenshots: bool,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
) -> Result<(), String> {
    if rest.len() > 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let bundle_dir = match rest.first() {
        Some(src) => {
            let src = crate::resolve_path(workspace_root, PathBuf::from(src));
            crate::resolve_bundle_root_dir(&src)?
        }
        None => crate::read_latest_pointer(out_dir)
            .or_else(|| crate::find_latest_export_dir(out_dir))
            .ok_or_else(|| {
                format!(
                    "no diagnostics bundle found under {} (try: fretboard diag pack ./target/fret-diag/<timestamp>)",
                    out_dir.display()
                )
            })?,
    };

    let bundle_dir = crate::resolve_bundle_root_dir(&bundle_dir)?;
    let out = pack_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| crate::default_pack_out_path(out_dir, &bundle_dir));

    let artifacts_root = if bundle_dir.starts_with(out_dir) {
        out_dir.to_path_buf()
    } else {
        bundle_dir.parent().unwrap_or(out_dir).to_path_buf()
    };

    crate::pack_bundle_dir_to_zip(
        &bundle_dir,
        &out,
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
        false,
        false,
        &artifacts_root,
        stats_top,
        sort_override.unwrap_or(BundleStatsSort::Invalidation),
        warmup_frames,
    )?;
    println!("{}", out.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_triage(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    triage_out: Option<PathBuf>,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle path (try: fretboard diag triage ./target/fret-diag/1234/bundle.json)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let bundle_path = crate::resolve_bundle_json_path(&src);
    let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);

    let report = bundle_stats_from_path(
        &bundle_path,
        stats_top,
        sort,
        BundleStatsOptions { warmup_frames },
    )?;
    let payload = crate::triage_json_from_stats(&bundle_path, &report, sort, warmup_frames);

    let out = triage_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| crate::default_triage_out_path(&bundle_path));

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

    if stats_json {
        println!("{pretty}");
    } else {
        println!("{}", out.display());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_lint(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    lint_out: Option<PathBuf>,
    lint_all_test_ids_bounds: bool,
    lint_eps_px: f32,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle path (try: fretboard diag lint ./target/fret-diag/1234/bundle.json)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let bundle_path = crate::resolve_bundle_json_path(&src);

    let report = lint_bundle_from_path(
        &bundle_path,
        warmup_frames,
        LintOptions {
            all_test_ids_bounds: lint_all_test_ids_bounds,
            eps_px: lint_eps_px,
        },
    )?;

    let out = lint_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| crate::default_lint_out_path(&bundle_path));

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(&report.payload).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

    if stats_json {
        println!("{pretty}");
    } else {
        println!("{}", out.display());
    }

    if report.error_issues > 0 {
        std::process::exit(1);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_test_ids(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    test_ids_out: Option<PathBuf>,
    warmup_frames: u64,
    max_test_ids: usize,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle path (try: fretboard diag test-ids ./target/fret-diag/1234/bundle.json)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let bundle_path = crate::resolve_bundle_json_path(&src);

    let out = test_ids_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| crate::default_test_ids_out_path(&bundle_path));

    if out.is_file() {
        if stats_json {
            println!(
                "{}",
                std::fs::read_to_string(&out).map_err(|e| e.to_string())?
            );
        } else {
            println!("{}", out.display());
        }
        return Ok(());
    }

    let canonical =
        crate::bundle_index::ensure_test_ids_json(&bundle_path, warmup_frames, max_test_ids)?;
    if out != canonical {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::copy(&canonical, &out).map_err(|e| e.to_string())?;
    }

    if stats_json {
        println!(
            "{}",
            std::fs::read_to_string(&out).map_err(|e| e.to_string())?
        );
    } else {
        println!("{}", out.display());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_meta(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    meta_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
    meta_report: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    if stats_json && meta_report {
        return Err("--meta-report cannot be combined with --json".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle path (try: fretboard diag meta ./target/fret-diag/1234/bundle.json)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));

    let (meta_path, default_out) = if src.is_file()
        && src
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "bundle.meta.json")
    {
        (src.clone(), src.clone())
    } else if src.is_dir() {
        let direct = src.join("bundle.meta.json");
        if direct.is_file() {
            (direct.clone(), direct)
        } else {
            let bundle_path = crate::resolve_bundle_json_path(&src);
            let canonical =
                crate::bundle_index::ensure_bundle_meta_json(&bundle_path, warmup_frames)?;
            let out = crate::default_meta_out_path(&bundle_path);
            (canonical, out)
        }
    } else {
        let bundle_path = crate::resolve_bundle_json_path(&src);
        let canonical = crate::bundle_index::ensure_bundle_meta_json(&bundle_path, warmup_frames)?;
        let out = crate::default_meta_out_path(&bundle_path);
        (canonical, out)
    };

    let out = meta_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or(default_out);

    if out.is_file() {
        if stats_json {
            println!(
                "{}",
                std::fs::read_to_string(&out).map_err(|e| e.to_string())?
            );
        } else if meta_report {
            let meta: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&out).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            print_meta_report(&meta, &out);
        } else {
            println!("{}", out.display());
        }
        return Ok(());
    }

    if out != meta_path {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::copy(&meta_path, &out).map_err(|e| e.to_string())?;
    }

    if stats_json {
        println!(
            "{}",
            std::fs::read_to_string(&out).map_err(|e| e.to_string())?
        );
    } else if meta_report {
        let meta: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&out).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        print_meta_report(&meta, &out);
    } else {
        println!("{}", out.display());
    }
    Ok(())
}

fn print_meta_report(meta: &serde_json::Value, meta_path: &Path) {
    fn u64_field(v: &serde_json::Value, key: &str) -> u64 {
        v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
    }

    fn str_field<'a>(v: &'a serde_json::Value, key: &str) -> &'a str {
        v.get(key).and_then(|v| v.as_str()).unwrap_or("")
    }

    println!("bundle_meta:");
    println!("  meta_json: {}", meta_path.display());
    println!("  bundle: {}", str_field(meta, "bundle"));
    println!("  warmup_frames: {}", u64_field(meta, "warmup_frames"));
    println!("  windows_total: {}", u64_field(meta, "windows_total"));
    println!("  snapshots_total: {}", u64_field(meta, "snapshots_total"));
    println!(
        "  semantics: resolved={} inline={} table={} table_entries={} table_unique_keys={}",
        u64_field(meta, "snapshots_with_semantics_total"),
        u64_field(meta, "snapshots_with_inline_semantics_total"),
        u64_field(meta, "snapshots_with_table_semantics_total"),
        u64_field(meta, "semantics_table_entries_total"),
        u64_field(meta, "semantics_table_unique_keys_total"),
    );

    let Some(windows) = meta.get("windows").and_then(|v| v.as_array()) else {
        return;
    };
    if windows.is_empty() {
        return;
    }

    println!("  windows:");
    let max = 6usize;
    for w in windows.iter().take(max) {
        let window = u64_field(w, "window");
        let snapshots_total = u64_field(w, "snapshots_total");
        let sem_resolved = u64_field(w, "snapshots_with_semantics_total");
        let sem_inline = u64_field(w, "snapshots_with_inline_semantics_total");
        let sem_table = u64_field(w, "snapshots_with_table_semantics_total");
        let table_entries = u64_field(w, "semantics_table_entries_total");
        let table_keys = u64_field(w, "semantics_table_unique_keys_total");
        let considered_frame_id = w
            .get("considered_frame_id")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());
        println!(
            "    - window={} snapshots={} considered_frame={} semantics(resolved/inline/table)={}/{}/{} table(entries/keys)={}/{}",
            window,
            snapshots_total,
            considered_frame_id,
            sem_resolved,
            sem_inline,
            sem_table,
            table_entries,
            table_keys,
        );
    }
    if windows.len() > max {
        println!("    - ... ({} more)", windows.len() - max);
    }
}
