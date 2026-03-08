use std::path::{Path, PathBuf};

use super::resolve;
use super::sidecars;

use crate::lint::{LintOptions, lint_bundle_from_path};
use crate::stats::{BundleStatsOptions, BundleStatsSort, bundle_stats_from_path};

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_pack(
    rest: &[String],
    workspace_root: &Path,
    out_dir: &Path,
    pack_out: Option<PathBuf>,
    ensure_ai_packet: bool,
    pack_ai_only: bool,
    pack_include_root_artifacts: bool,
    pack_include_triage: bool,
    pack_include_screenshots: bool,
    pack_schema2_only: bool,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
) -> Result<(), String> {
    if rest.len() > 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let source = rest
        .first()
        .map(|src| crate::resolve_path(workspace_root, PathBuf::from(src)))
        .unwrap_or_default();
    let resolved = resolve::resolve_bundle_input_or_latest(&source, out_dir).map_err(|error| {
        if rest.is_empty() {
            format!(
                "{} (try: fretboard diag pack ./target/fret-diag/<timestamp>)",
                error
            )
        } else {
            error
        }
    })?;
    let bundle_dir = resolved.bundle_dir;
    let out = pack_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| {
            if pack_ai_only {
                let name = bundle_dir
                    .file_name()
                    .and_then(|s| s.to_str())
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or("bundle");
                if bundle_dir.starts_with(out_dir) {
                    out_dir.join("share").join(format!("{name}.ai.zip"))
                } else {
                    bundle_dir.with_extension("ai.zip")
                }
            } else {
                crate::default_pack_out_path(out_dir, &bundle_dir)
            }
        });

    let artifacts_root = resolved.artifacts_root;

    if ensure_ai_packet || pack_ai_only {
        let packet_dir = bundle_dir.join("ai.packet");
        if !packet_dir.is_dir() {
            if let Err(err) = super::ai_packet::ensure_ai_packet_dir_best_effort(
                None,
                &bundle_dir,
                &packet_dir,
                pack_include_triage,
                stats_top,
                sort_override,
                warmup_frames,
                None,
            ) {
                // Best-effort: pack may still succeed if `ai.packet/` is already present,
                // or will fail with a clear `--ai-only requires ai.packet` error.
                eprintln!("ai-packet: failed to generate ai.packet: {err}");
            }
        }
    }

    if pack_ai_only {
        let packet_dir = bundle_dir.join("ai.packet");
        if !packet_dir.is_dir() {
            return Err(format!(
                "--ai-only requires ai.packet under the bundle dir (tip: fretboard diag ai-packet {} --packet-out {})",
                bundle_dir.display(),
                packet_dir.display()
            ));
        }
        crate::pack_ai_packet_dir_to_zip(&bundle_dir, &out, &artifacts_root)?;
        println!("{}", out.display());
        return Ok(());
    }

    crate::pack_bundle_dir_to_zip(
        &bundle_dir,
        &out,
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
        pack_schema2_only,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_ai_only_can_generate_ai_packet_from_sidecars_only() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-pack-ai-only-sidecars-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let bundle_dir = root.join("bundle");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");

        std::fs::write(
            bundle_dir.join("bundle.meta.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind": "bundle_meta",
                "schema_version": 1,
                "warmup_frames": 0,
                "bundle": "bundle.json",
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            bundle_dir.join("test_ids.index.json"),
            b"{\"kind\":\"test_ids_index\",\"schema_version\":1}",
        )
        .unwrap();
        std::fs::write(
            bundle_dir.join("bundle.index.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind": "bundle_index",
                "schema_version": 1,
                "warmup_frames": 0,
                "bundle": "bundle.json",
                "windows": [],
                "script": { "steps": [] },
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            bundle_dir.join("frames.index.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind": "frames_index",
                "schema_version": 1,
                "bundle": "bundle.json",
                "generated_unix_ms": 0,
                "warmup_frames": 0,
                "has_semantics_table": true,
                "columns": ["frame_id", "window_snapshot_seq", "timestamp_unix_ms", "total_time_us", "layout_time_us", "paint_time_us", "semantics_fingerprint", "semantics_source_tag"],
                "windows_total": 0,
                "snapshots_total": 0,
                "frames_total": 0,
                "windows": []
            }))
            .unwrap(),
        )
        .unwrap();

        let out_path = root.join("out.ai.zip");
        cmd_pack(
            &[bundle_dir.to_string_lossy().to_string()],
            &root,
            &root,
            Some(out_path.clone()),
            false,
            true,
            false,
            false,
            false,
            false,
            10,
            None,
            0,
        )
        .expect("pack ai-only zip");

        assert!(out_path.is_file(), "expected output zip");

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

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn parse_triage_request_supports_metric_aliases() {
        let workspace_root = Path::new("workspace-root");
        let request = parse_triage_request(
            &[
                "--metric".to_string(),
                "paint".to_string(),
                "demo/bundle.json".to_string(),
            ],
            workspace_root,
        )
        .expect("parse triage request");

        assert!(request.lite, "--metric should imply lite mode");
        assert!(matches!(
            request.metric,
            crate::frames_index::TriageLiteMetric::PaintTimeUs
        ));
        assert_eq!(
            request.source,
            workspace_root.join(PathBuf::from("demo/bundle.json"))
        );
    }

    #[test]
    fn resolve_artifact_display_mode_rejects_meta_report_with_json() {
        let err = resolve_artifact_display_mode(true, true).expect_err("expected invalid mode");
        assert!(
            err.contains("--meta-report cannot be combined with --json"),
            "unexpected error: {err}"
        );
    }
}

#[derive(Debug)]
struct ParsedTriageRequest {
    source: PathBuf,
    lite: bool,
    metric: crate::frames_index::TriageLiteMetric,
}

#[derive(Debug, Clone, Copy)]
enum ArtifactDisplayMode {
    Path,
    Json,
    MetaReport,
}

#[derive(Debug)]
struct ParsedMetaRequest {
    source: PathBuf,
    display_mode: ArtifactDisplayMode,
}

#[derive(Debug)]
struct MetaArtifactPaths {
    canonical_path: PathBuf,
    default_out: PathBuf,
}

fn parse_triage_metric(raw: &str) -> Result<crate::frames_index::TriageLiteMetric, String> {
    match raw {
        "total" | "total_time_us" => Ok(crate::frames_index::TriageLiteMetric::TotalTimeUs),
        "layout" | "layout_time_us" => Ok(crate::frames_index::TriageLiteMetric::LayoutTimeUs),
        "paint" | "paint_time_us" => Ok(crate::frames_index::TriageLiteMetric::PaintTimeUs),
        other => Err(format!(
            "invalid value for --metric: {other} (expected total|layout|paint)"
        )),
    }
}

fn parse_triage_request(
    rest: &[String],
    workspace_root: &Path,
) -> Result<ParsedTriageRequest, String> {
    let mut lite = false;
    let mut metric = crate::frames_index::TriageLiteMetric::TotalTimeUs;
    let mut positionals: Vec<String> = Vec::new();

    let mut index = 0usize;
    while index < rest.len() {
        match rest[index].as_str() {
            "--lite" | "--frames-index" | "--from-frames-index" => {
                lite = true;
                index += 1;
            }
            "--metric" => {
                lite = true;
                index += 1;
                let Some(value) = rest.get(index) else {
                    return Err(
                        "missing value for --metric (expected total|layout|paint)".to_string()
                    );
                };
                metric = parse_triage_metric(value)?;
                index += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for triage: {other}"));
            }
            other => {
                positionals.push(other.to_string());
                index += 1;
            }
        }
    }

    let Some(source) = positionals.first() else {
        return Err(
            "missing bundle artifact path (try: fretboard diag triage <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if positionals.len() != 1 {
        return Err(format!(
            "unexpected arguments: {}",
            positionals[1..].join(" ")
        ));
    }

    Ok(ParsedTriageRequest {
        source: crate::resolve_path(workspace_root, PathBuf::from(source)),
        lite,
        metric,
    })
}

fn build_triage_payload(
    bundle_path: &Path,
    lite: bool,
    metric: crate::frames_index::TriageLiteMetric,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
) -> Result<serde_json::Value, String> {
    let mut payload = if lite {
        let index_path = crate::frames_index::default_frames_index_path(bundle_path);
        let mut frames_index =
            crate::frames_index::read_frames_index_json_v1(&index_path, warmup_frames);
        if frames_index.is_none() {
            let out = crate::frames_index::ensure_frames_index_json(bundle_path, warmup_frames)?;
            frames_index = crate::frames_index::read_frames_index_json_v1(&out, warmup_frames);
        }
        let frames_index = frames_index.ok_or_else(|| {
            format!(
                "frames.index.json is missing or invalid (tip: fretboard diag frames-index {} --warmup-frames {})",
                bundle_path.display(),
                warmup_frames
            )
        })?;
        crate::frames_index::triage_lite_json_from_frames_index(
            bundle_path,
            &index_path,
            &frames_index,
            warmup_frames,
            stats_top,
            metric,
        )?
    } else {
        let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);
        let report = bundle_stats_from_path(
            bundle_path,
            stats_top,
            sort,
            BundleStatsOptions { warmup_frames },
        )?;
        crate::triage_json_from_stats(bundle_path, &report, sort, warmup_frames)
    };

    if let Some(bundle_dir) = bundle_path.parent() {
        let warnings = crate::tooling_warnings::tooling_warnings_for_bundle_dir(bundle_dir);
        if !warnings.is_empty()
            && let Some(obj) = payload.as_object_mut()
        {
            obj.insert(
                "tooling_warnings".to_string(),
                serde_json::Value::Array(warnings),
            );
        }
    }

    Ok(payload)
}

fn default_triage_out_path(bundle_path: &Path, lite: bool) -> PathBuf {
    if lite {
        let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
        dir.join("triage.lite.json")
    } else {
        crate::default_triage_out_path(bundle_path)
    }
}

fn write_json_artifact_output(
    out: &Path,
    payload: &serde_json::Value,
    print_json: bool,
) -> Result<(), String> {
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(payload).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(out, pretty.as_bytes()).map_err(|e| e.to_string())?;

    if print_json {
        println!("{pretty}");
    } else {
        println!("{}", out.display());
    }
    Ok(())
}

fn resolve_artifact_display_mode(
    stats_json: bool,
    meta_report: bool,
) -> Result<ArtifactDisplayMode, String> {
    if stats_json && meta_report {
        return Err("--meta-report cannot be combined with --json".to_string());
    }

    Ok(if stats_json {
        ArtifactDisplayMode::Json
    } else if meta_report {
        ArtifactDisplayMode::MetaReport
    } else {
        ArtifactDisplayMode::Path
    })
}

fn parse_meta_request(
    rest: &[String],
    workspace_root: &Path,
    stats_json: bool,
    meta_report: bool,
) -> Result<ParsedMetaRequest, String> {
    let display_mode = resolve_artifact_display_mode(stats_json, meta_report)?;
    let Some(source) = rest.first() else {
        return Err(
            "missing bundle artifact path (try: fretboard diag meta <bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    Ok(ParsedMetaRequest {
        source: crate::resolve_path(workspace_root, PathBuf::from(source)),
        display_mode,
    })
}

fn resolve_meta_artifact_paths(
    src: &Path,
    warmup_frames: u64,
) -> Result<MetaArtifactPaths, String> {
    let resolved = resolve::resolve_bundle_ref(src)?;
    let src = resolved.bundle_dir;

    let (canonical_path, default_out) = if src.is_file()
        && src
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "bundle.meta.json")
    {
        if sidecars::try_read_sidecar_json_v1(&src, "bundle_meta", warmup_frames).is_some() {
            (src.clone(), src.clone())
        } else if let Some(bundle_path) = sidecars::adjacent_bundle_path_for_sidecar(&src) {
            let canonical =
                crate::bundle_index::ensure_bundle_meta_json(&bundle_path, warmup_frames)?;
            let out = crate::default_meta_out_path(&bundle_path);
            (canonical, out)
        } else {
            return Err(format!(
                "invalid bundle.meta.json (expected schema_version=1 warmup_frames={warmup_frames}) and no adjacent bundle artifact was found to regenerate it\n  meta: {}",
                src.display()
            ));
        }
    } else if src.is_dir() {
        let direct = src.join("bundle.meta.json");
        if direct.is_file()
            && sidecars::try_read_sidecar_json_v1(&direct, "bundle_meta", warmup_frames).is_some()
        {
            (direct.clone(), direct)
        } else {
            let root = src.join("_root").join("bundle.meta.json");
            if root.is_file()
                && sidecars::try_read_sidecar_json_v1(&root, "bundle_meta", warmup_frames).is_some()
            {
                (root.clone(), root)
            } else {
                let bundle_path = crate::resolve_bundle_artifact_path(&src);
                let canonical =
                    crate::bundle_index::ensure_bundle_meta_json(&bundle_path, warmup_frames)?;
                let out = crate::default_meta_out_path(&bundle_path);
                (canonical, out)
            }
        }
    } else {
        let bundle_path = crate::resolve_bundle_artifact_path(&src);
        let canonical = crate::bundle_index::ensure_bundle_meta_json(&bundle_path, warmup_frames)?;
        let out = crate::default_meta_out_path(&bundle_path);
        (canonical, out)
    };

    Ok(MetaArtifactPaths {
        canonical_path,
        default_out,
    })
}

fn materialize_meta_output(canonical_path: &Path, out: &Path) -> Result<(), String> {
    if out.is_file() || out == canonical_path {
        return Ok(());
    }

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::copy(canonical_path, out).map_err(|e| e.to_string())?;
    Ok(())
}

fn emit_artifact_output(out: &Path, display_mode: ArtifactDisplayMode) -> Result<(), String> {
    match display_mode {
        ArtifactDisplayMode::Path => {
            println!("{}", out.display());
            Ok(())
        }
        ArtifactDisplayMode::Json => {
            println!(
                "{}",
                std::fs::read_to_string(out).map_err(|e| e.to_string())?
            );
            Ok(())
        }
        ArtifactDisplayMode::MetaReport => {
            let meta: serde_json::Value =
                serde_json::from_slice(&std::fs::read(out).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            print_meta_report(&meta, out);
            Ok(())
        }
    }
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

    let request = parse_triage_request(rest, workspace_root)?;
    let resolved = resolve::resolve_bundle_ref(&request.source)?;
    let bundle_path = resolved.bundle_artifact;
    let payload = build_triage_payload(
        &bundle_path,
        request.lite,
        request.metric,
        stats_top,
        sort_override,
        warmup_frames,
    )?;
    let out = triage_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| default_triage_out_path(&bundle_path, request.lite));
    write_json_artifact_output(&out, &payload, stats_json)
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
            "missing bundle artifact path (try: fretboard diag lint <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let resolved = resolve::resolve_bundle_ref(&src)?;
    let bundle_path = resolved.bundle_artifact;

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
            "missing bundle artifact path (try: fretboard diag test-ids <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let resolved = resolve::resolve_bundle_ref(&src)?;
    let bundle_path = resolved.bundle_artifact;

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
pub(crate) fn cmd_test_ids_index(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle artifact path (try: fretboard diag test-ids-index <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let resolved = resolve::resolve_bundle_ref(&src)?;
    let bundle_path = resolved.bundle_artifact;
    let out = crate::bundle_index::ensure_test_ids_index_json(&bundle_path, warmup_frames)?;

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
pub(crate) fn cmd_frames_index(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle artifact path (try: fretboard diag frames-index <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let resolved = resolve::resolve_bundle_ref(&src)?;
    let bundle_path = resolved.bundle_artifact;
    let out = crate::frames_index::ensure_frames_index_json(&bundle_path, warmup_frames)?;

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

    let request = parse_meta_request(rest, workspace_root, stats_json, meta_report)?;
    let paths = resolve_meta_artifact_paths(&request.source, warmup_frames)?;
    let out = meta_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or(paths.default_out);
    materialize_meta_output(&paths.canonical_path, &out)?;
    emit_artifact_output(&out, request.display_mode)
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
