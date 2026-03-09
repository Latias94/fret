use std::path::{Path, PathBuf};

use super::resolve;
use super::sidecars;

use crate::lint::{LintOptions, LintReport, lint_bundle_from_path};
use crate::stats::{BundleStatsOptions, BundleStatsSort, bundle_stats_from_path};

struct PackCommandRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    out_dir: &'a Path,
    pack_out: Option<PathBuf>,
    pack_ai_only: bool,
}

struct PreparedPackCommand {
    bundle_dir: PathBuf,
    artifacts_root: PathBuf,
    out: PathBuf,
}

struct PackAiPacketEnsureRequest<'a> {
    bundle_dir: &'a Path,
    pack_include_triage: bool,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
}

struct RequiredBundleArtifactRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    missing_hint: &'a str,
}

struct LintCommandRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    lint_out: Option<PathBuf>,
}

struct PreparedLintCommand {
    bundle_path: PathBuf,
    out: PathBuf,
}

struct TriageCommandRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    triage_out: Option<PathBuf>,
}

struct PreparedTriageCommand {
    bundle_path: PathBuf,
    out: PathBuf,
    lite: bool,
    metric: crate::frames_index::TriageLiteMetric,
}

struct TestIdsCommandRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    test_ids_out: Option<PathBuf>,
}

struct PreparedTestIdsCommand {
    bundle_path: PathBuf,
    out: PathBuf,
}

struct MetaCommandRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    meta_out: Option<PathBuf>,
    stats_json: bool,
    meta_report: bool,
}

struct PreparedMetaCommand {
    canonical_path: PathBuf,
    out: PathBuf,
    display_mode: ArtifactDisplayMode,
}

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
    let PreparedPackCommand {
        bundle_dir,
        artifacts_root,
        out,
    } = prepare_cmd_pack(PackCommandRequest {
        rest,
        workspace_root,
        out_dir,
        pack_out,
        pack_ai_only,
    })?;

    if ensure_ai_packet || pack_ai_only {
        ensure_ai_packet_dir_for_pack(PackAiPacketEnsureRequest {
            bundle_dir: &bundle_dir,
            pack_include_triage,
            stats_top,
            sort_override,
            warmup_frames,
        });
    }

    if pack_ai_only {
        require_ai_packet_dir_for_pack_ai_only(&bundle_dir)?;
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

fn prepare_cmd_pack(request: PackCommandRequest<'_>) -> Result<PreparedPackCommand, String> {
    if request.rest.len() > 1 {
        return Err(format!(
            "unexpected arguments: {}",
            request.rest[1..].join(" ")
        ));
    }

    let source = request
        .rest
        .first()
        .map(|src| crate::resolve_path(request.workspace_root, PathBuf::from(src)))
        .unwrap_or_default();
    let resolved =
        resolve::resolve_bundle_input_or_latest(&source, request.out_dir).map_err(|error| {
            if request.rest.is_empty() {
                format!(
                    "{} (try: fretboard diag pack ./target/fret-diag/<timestamp>)",
                    error
                )
            } else {
                error
            }
        })?;
    let out = request
        .pack_out
        .map(|path| crate::resolve_path(request.workspace_root, path))
        .unwrap_or_else(|| {
            default_pack_output_path(&resolved.bundle_dir, request.out_dir, request.pack_ai_only)
        });

    Ok(PreparedPackCommand {
        bundle_dir: resolved.bundle_dir,
        artifacts_root: resolved.artifacts_root,
        out,
    })
}

fn pack_ai_packet_dir(bundle_dir: &Path) -> PathBuf {
    bundle_dir.join("ai.packet")
}

fn ensure_ai_packet_dir_for_pack(request: PackAiPacketEnsureRequest<'_>) {
    let packet_dir = pack_ai_packet_dir(request.bundle_dir);
    if packet_dir.is_dir() {
        return;
    }
    if let Err(err) = super::ai_packet::ensure_ai_packet_dir_best_effort(
        None,
        request.bundle_dir,
        &packet_dir,
        request.pack_include_triage,
        request.stats_top,
        request.sort_override,
        request.warmup_frames,
        None,
    ) {
        eprintln!("ai-packet: failed to generate ai.packet: {err}");
    }
}

fn require_ai_packet_dir_for_pack_ai_only(bundle_dir: &Path) -> Result<PathBuf, String> {
    let packet_dir = pack_ai_packet_dir(bundle_dir);
    if packet_dir.is_dir() {
        Ok(packet_dir)
    } else {
        Err(format!(
            "--ai-only requires ai.packet under the bundle dir (tip: fretboard diag ai-packet {} --packet-out {})",
            bundle_dir.display(),
            packet_dir.display()
        ))
    }
}

fn default_pack_output_path(bundle_dir: &Path, out_dir: &Path, pack_ai_only: bool) -> PathBuf {
    if pack_ai_only {
        let name = bundle_dir
            .file_name()
            .and_then(|segment| segment.to_str())
            .filter(|segment| !segment.trim().is_empty())
            .unwrap_or("bundle");
        if bundle_dir.starts_with(out_dir) {
            out_dir.join("share").join(format!("{name}.ai.zip"))
        } else {
            bundle_dir.with_extension("ai.zip")
        }
    } else {
        crate::default_pack_out_path(out_dir, bundle_dir)
    }
}

fn resolve_required_bundle_artifact(
    request: RequiredBundleArtifactRequest<'_>,
) -> Result<PathBuf, String> {
    let Some(src) = request.rest.first().cloned() else {
        return Err(request.missing_hint.to_string());
    };
    if request.rest.len() != 1 {
        return Err(format!(
            "unexpected arguments: {}",
            request.rest[1..].join(" ")
        ));
    }

    let src = crate::resolve_path(request.workspace_root, PathBuf::from(src));
    let resolved = resolve::resolve_bundle_ref(&src)?;
    Ok(resolved.bundle_artifact)
}

fn emit_path_or_json_output(out: &Path, stats_json: bool) -> Result<(), String> {
    if stats_json {
        println!("{}", read_artifact_output_text(out)?);
    } else {
        println!("{}", out.display());
    }
    Ok(())
}

fn ensure_and_emit_bundle_artifact_output<F>(
    bundle_path: &Path,
    warmup_frames: u64,
    stats_json: bool,
    ensure_artifact_output: F,
) -> Result<(), String>
where
    F: FnOnce(&Path, u64) -> Result<PathBuf, String>,
{
    let out = ensure_artifact_output(bundle_path, warmup_frames)?;
    emit_path_or_json_output(&out, stats_json)
}

fn read_artifact_output_text(out: &Path) -> Result<String, String> {
    std::fs::read_to_string(out).map_err(|e| e.to_string())
}

fn read_artifact_output_json(out: &Path) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(out).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn json_artifact_payload_to_pretty_text(payload: &serde_json::Value) -> String {
    serde_json::to_string_pretty(payload).unwrap_or_else(|_| "{}".to_string())
}

fn emit_existing_artifact_output_if_present(out: &Path, stats_json: bool) -> Result<bool, String> {
    if out.is_file() {
        emit_path_or_json_output(out, stats_json)?;
        Ok(true)
    } else {
        Ok(false)
    }
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
    fn pack_ai_packet_dir_appends_ai_packet_child() {
        let bundle_dir = Path::new("captures/demo-bundle");

        let packet_dir = pack_ai_packet_dir(bundle_dir);

        assert_eq!(packet_dir, PathBuf::from("captures/demo-bundle/ai.packet"));
    }

    #[test]
    fn require_ai_packet_dir_for_pack_ai_only_accepts_existing_dir() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-pack-ai-packet-dir-ok-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("bundle");
        let packet_dir = bundle_dir.join("ai.packet");
        std::fs::create_dir_all(&packet_dir).expect("create ai.packet dir");

        let resolved =
            require_ai_packet_dir_for_pack_ai_only(&bundle_dir).expect("resolve ai.packet dir");

        assert_eq!(resolved, packet_dir);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn require_ai_packet_dir_for_pack_ai_only_errors_when_missing() {
        let bundle_dir = PathBuf::from("captures/demo-bundle");

        let err =
            require_ai_packet_dir_for_pack_ai_only(&bundle_dir).expect_err("missing ai.packet");

        assert!(err.contains("--ai-only requires ai.packet under the bundle dir"));
        assert!(err.contains(&pack_ai_packet_dir(&bundle_dir).display().to_string()));
    }

    #[test]
    fn default_pack_output_path_uses_share_dir_for_ai_only_under_out_dir() {
        let out_dir = PathBuf::from("target/fret-diag/session-1");
        let bundle_dir = out_dir.join("123-ui-gallery");

        let out = default_pack_output_path(&bundle_dir, &out_dir, true);

        assert_eq!(out, out_dir.join("share").join("123-ui-gallery.ai.zip"));
    }

    #[test]
    fn default_pack_output_path_uses_bundle_extension_for_external_ai_only() {
        let out_dir = PathBuf::from("target/fret-diag/session-1");
        let bundle_dir = PathBuf::from("captures/123-ui-gallery");

        let out = default_pack_output_path(&bundle_dir, &out_dir, true);

        assert_eq!(out, PathBuf::from("captures/123-ui-gallery.ai.zip"));
    }

    #[test]
    fn resolve_required_bundle_artifact_rejects_missing_input() {
        let err = resolve_required_bundle_artifact(RequiredBundleArtifactRequest {
            rest: &[],
            workspace_root: Path::new("workspace-root"),
            missing_hint: "missing bundle",
        })
        .expect_err("missing input");

        assert_eq!(err, "missing bundle");
    }

    #[test]
    fn resolve_required_bundle_artifact_rejects_extra_args() {
        let err = resolve_required_bundle_artifact(RequiredBundleArtifactRequest {
            rest: &["a".to_string(), "b".to_string()],
            workspace_root: Path::new("workspace-root"),
            missing_hint: "missing bundle",
        })
        .expect_err("extra args");

        assert!(err.contains("unexpected arguments: b"));
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
    fn meta_report_lines_include_base_summary_fields() {
        let meta = serde_json::json!({
            "bundle": "bundle.json",
            "warmup_frames": 3,
            "windows_total": 2,
            "snapshots_total": 5,
            "snapshots_with_semantics_total": 4,
            "snapshots_with_inline_semantics_total": 1,
            "snapshots_with_table_semantics_total": 3,
            "semantics_table_entries_total": 8,
            "semantics_table_unique_keys_total": 6,
            "windows": [],
        });

        let lines = meta_report_lines(&meta, Path::new("out/bundle.meta.json"));

        assert_eq!(lines.first().map(String::as_str), Some("bundle_meta:"));
        assert!(
            lines
                .iter()
                .any(|line| line == "  meta_json: out/bundle.meta.json"),
            "missing meta path: {lines:?}"
        );
        assert!(lines.iter().any(|line| line == "  bundle: bundle.json"));
        assert!(lines.iter().any(|line| line == "  warmup_frames: 3"));
        assert!(lines.iter().any(|line| line == "  windows_total: 2"));
        assert!(lines.iter().any(|line| {
            line == "  semantics: resolved=4 inline=1 table=3 table_entries=8 table_unique_keys=6"
        }));
    }

    #[test]
    fn meta_report_lines_truncate_windows_after_six_rows() {
        let windows = (0..8)
            .map(|index| {
                serde_json::json!({
                    "window": index,
                    "snapshots_total": index + 10,
                    "considered_frame_id": index + 100,
                    "snapshots_with_semantics_total": index + 1,
                    "snapshots_with_inline_semantics_total": index + 2,
                    "snapshots_with_table_semantics_total": index + 3,
                    "semantics_table_entries_total": index + 4,
                    "semantics_table_unique_keys_total": index + 5,
                })
            })
            .collect::<Vec<_>>();
        let meta = serde_json::json!({
            "bundle": "bundle.json",
            "warmup_frames": 0,
            "windows_total": 8,
            "snapshots_total": 8,
            "snapshots_with_semantics_total": 8,
            "snapshots_with_inline_semantics_total": 8,
            "snapshots_with_table_semantics_total": 8,
            "semantics_table_entries_total": 8,
            "semantics_table_unique_keys_total": 8,
            "windows": windows,
        });

        let lines = meta_report_lines(&meta, Path::new("out/bundle.meta.json"));

        assert!(lines.iter().any(|line| line == "  windows:"));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("window=0 snapshots=10 considered_frame=100"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("window=5 snapshots=15 considered_frame=105"))
        );
        assert!(lines.iter().any(|line| line == "    - ... (2 more)"));
    }

    #[test]
    fn default_triage_out_path_uses_lite_sidecar_name() {
        let bundle_path = Path::new("captures/demo/bundle.json");

        let out = default_triage_out_path(bundle_path, true);

        assert_eq!(out, PathBuf::from("captures/demo/triage.lite.json"));
    }

    #[test]
    fn prepare_cmd_triage_defaults_to_lite_out_for_metric_request() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-triage-lite-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.json"), b"{}" as &[u8]).expect("write bundle json");

        let prepared = prepare_cmd_triage(TriageCommandRequest {
            rest: &[
                "--metric".to_string(),
                "paint".to_string(),
                "captures/demo/bundle.json".to_string(),
            ],
            workspace_root: &root,
            triage_out: None,
        })
        .expect("prepare triage command");

        assert!(prepared.lite);
        assert!(matches!(
            prepared.metric,
            crate::frames_index::TriageLiteMetric::PaintTimeUs
        ));
        assert_eq!(prepared.bundle_path, bundle_dir.join("bundle.json"));
        assert_eq!(prepared.out, bundle_dir.join("triage.lite.json"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prepare_cmd_triage_uses_custom_out_when_provided() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-triage-custom-out-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.json"), b"{}" as &[u8]).expect("write bundle json");

        let prepared = prepare_cmd_triage(TriageCommandRequest {
            rest: &["captures/demo/bundle.json".to_string()],
            workspace_root: &root,
            triage_out: Some(PathBuf::from("exports/triage.json")),
        })
        .expect("prepare triage command");

        assert!(!prepared.lite);
        assert_eq!(prepared.out, root.join("exports").join("triage.json"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prepare_cmd_test_ids_defaults_to_default_out_path() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-test-ids-default-out-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.json"), b"{}" as &[u8]).expect("write bundle json");

        let prepared = prepare_cmd_test_ids(TestIdsCommandRequest {
            rest: &["captures/demo/bundle.json".to_string()],
            workspace_root: &root,
            test_ids_out: None,
        })
        .expect("prepare test ids command");

        assert_eq!(prepared.bundle_path, bundle_dir.join("bundle.json"));
        assert_eq!(
            prepared.out,
            crate::default_test_ids_out_path(&bundle_dir.join("bundle.json"))
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prepare_cmd_test_ids_uses_custom_out_when_provided() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-test-ids-custom-out-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.json"), b"{}" as &[u8]).expect("write bundle json");

        let prepared = prepare_cmd_test_ids(TestIdsCommandRequest {
            rest: &["captures/demo/bundle.json".to_string()],
            workspace_root: &root,
            test_ids_out: Some(PathBuf::from("exports/test-ids.json")),
        })
        .expect("prepare test ids command");

        assert_eq!(prepared.out, root.join("exports").join("test-ids.json"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn append_triage_tooling_warnings_inserts_warning_array_for_bundle_dir() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-triage-tooling-warnings-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(crate::session::SESSIONS_DIRNAME))
            .expect("create sessions dir");
        let bundle_dir = root.join("123-bundle");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");

        let bundle_path = bundle_dir.join("bundle.json");
        let mut payload = serde_json::json!({ "kind": "triage" });
        append_triage_tooling_warnings(&mut payload, &bundle_path);

        let warnings = payload
            .get("tooling_warnings")
            .and_then(|value| value.as_array())
            .expect("tooling warnings array");
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings[0]
                .get("code")
                .and_then(|value| value.as_str())
                .unwrap_or(""),
            "diag.concurrency.base_dir_contains_sessions_but_bundle_not_in_session"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn lint_exit_required_only_when_error_issues_exist() {
        assert!(!lint_exit_required(0));
        assert!(lint_exit_required(1));
        assert!(lint_exit_required(3));
    }

    #[test]
    fn write_lint_report_output_returns_false_when_no_error_issues() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-write-lint-report-ok-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let out = root.join("lint.json");
        let report = LintReport {
            error_issues: 0,
            payload: serde_json::json!({ "kind": "lint", "error_issues": 0 }),
        };

        let should_exit =
            write_lint_report_output(&out, &report, false).expect("write lint report output");

        assert!(!should_exit);
        assert!(out.is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn write_lint_report_output_returns_true_when_error_issues_exist() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-write-lint-report-exit-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let out = root.join("lint.json");
        let report = LintReport {
            error_issues: 2,
            payload: serde_json::json!({ "kind": "lint", "error_issues": 2 }),
        };

        let should_exit =
            write_lint_report_output(&out, &report, false).expect("write lint report output");

        assert!(should_exit);
        assert!(out.is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn artifact_output_materialization_is_noop_when_out_matches_canonical() {
        let canonical = Path::new("captures/demo/test-ids.json");

        assert!(artifact_output_materialization_is_noop(
            canonical, canonical
        ));
    }

    #[test]
    fn materialize_canonical_artifact_output_keeps_existing_out_file() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-materialize-artifact-existing-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let canonical = root.join("canonical.json");
        let out = root.join("existing.json");
        std::fs::write(&canonical, b"canonical").expect("write canonical");
        std::fs::write(&out, b"existing").expect("write existing out");

        materialize_canonical_artifact_output(&canonical, &out)
            .expect("materialize canonical output");

        assert_eq!(
            std::fs::read_to_string(&out).expect("read existing out"),
            "existing"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn materialize_canonical_artifact_output_copies_to_nested_out_path() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-materialize-artifact-copy-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let canonical = root.join("canonical.json");
        let out = root.join("nested").join("copy.json");
        std::fs::write(&canonical, b"canonical").expect("write canonical");

        materialize_canonical_artifact_output(&canonical, &out)
            .expect("materialize canonical output");

        assert_eq!(
            std::fs::read_to_string(&out).expect("read copied out"),
            "canonical"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_and_emit_bundle_artifact_output_returns_ok_for_existing_artifact() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-ensure-emit-artifact-ok-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let out = root.join("artifact.json");
        std::fs::write(&out, b"{}" as &[u8]).expect("write artifact out");

        let result =
            ensure_and_emit_bundle_artifact_output(Path::new("bundle.json"), 0, false, |_, _| {
                Ok(out.clone())
            });

        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_and_emit_bundle_artifact_output_propagates_ensure_error() {
        let result =
            ensure_and_emit_bundle_artifact_output(Path::new("bundle.json"), 0, false, |_, _| {
                Err("boom".to_string())
            });

        assert_eq!(result.expect_err("expected ensure error"), "boom");
    }

    #[test]
    fn ensure_and_emit_bundle_artifact_output_propagates_emit_error() {
        let missing = PathBuf::from("missing-artifact.json");

        let result = ensure_and_emit_bundle_artifact_output(
            Path::new("bundle.json"),
            0,
            true,
            move |_, _| Ok(missing.clone()),
        );

        assert!(result.is_err());
    }

    #[test]
    fn emit_existing_artifact_output_if_present_returns_true_for_existing_file() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-emit-existing-artifact-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let artifact = root.join("artifact.json");
        std::fs::write(&artifact, b"{}" as &[u8]).expect("write artifact");

        let emitted =
            emit_existing_artifact_output_if_present(&artifact, false).expect("emit existing file");

        assert!(emitted);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn read_artifact_output_text_reads_existing_file() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-read-artifact-text-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let artifact = root.join("artifact.json");
        std::fs::write(&artifact, b"{\n  \"ok\": true\n}").expect("write artifact");

        let text = read_artifact_output_text(&artifact).expect("read artifact text");

        assert!(text.contains("\"ok\": true"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn read_artifact_output_json_parses_existing_json() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-read-artifact-json-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let artifact = root.join("artifact.json");
        std::fs::write(&artifact, b"{\"kind\":\"triage\"}").expect("write artifact");

        let json = read_artifact_output_json(&artifact).expect("read artifact json");

        assert_eq!(
            json.get("kind").and_then(|value| value.as_str()),
            Some("triage")
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn json_artifact_payload_to_pretty_text_formats_pretty_json() {
        let text = json_artifact_payload_to_pretty_text(&serde_json::json!({
            "kind": "lint",
            "error_issues": 1,
        }));

        assert!(text.starts_with('{'));
        assert!(text.contains("\n  \"kind\": \"lint\""));
    }

    #[test]
    fn resolve_artifact_display_mode_rejects_meta_report_with_json() {
        let err = resolve_artifact_display_mode(true, true).expect_err("expected invalid mode");
        assert!(
            err.contains("--meta-report cannot be combined with --json"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn resolve_meta_artifact_paths_from_meta_sidecar_keeps_valid_sidecar() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-meta-sidecar-valid-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        let sidecar = root.join("bundle.meta.json");
        std::fs::write(
            &sidecar,
            serde_json::to_vec(&serde_json::json!({
                "kind": "bundle_meta",
                "schema_version": 1,
                "warmup_frames": 0,
            }))
            .unwrap(),
        )
        .expect("write meta sidecar");

        let paths = resolve_meta_artifact_paths_from_meta_sidecar(&sidecar, 0)
            .expect("resolve meta sidecar");

        assert_eq!(paths.canonical_path, sidecar);
        assert_eq!(paths.default_out, paths.canonical_path);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_meta_artifact_paths_from_meta_sidecar_falls_back_to_adjacent_bundle() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-meta-sidecar-fallback-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        let bundle_path = root.join("bundle.json");
        let sidecar = root.join("bundle.meta.json");
        std::fs::write(
            &bundle_path,
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 2,
                "run_id": "run-1",
                "windows": [],
                "warmup_frames": 0,
            }))
            .unwrap(),
        )
        .expect("write bundle json");
        std::fs::write(
            &sidecar,
            serde_json::to_vec(&serde_json::json!({
                "kind": "bundle_meta",
                "schema_version": 1,
                "warmup_frames": 99,
            }))
            .unwrap(),
        )
        .expect("write invalid meta sidecar");

        let paths =
            resolve_meta_artifact_paths_from_meta_sidecar(&sidecar, 0).expect("fallback to bundle");

        assert_eq!(paths.default_out, root.join("bundle.meta.json"));
        assert_eq!(paths.canonical_path, root.join("bundle.meta.json"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_meta_artifact_paths_from_bundle_dir_prefers_root_meta_sidecar() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-meta-bundle-dir-root-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let bundle_dir = root.join("bundle-dir");
        let root_meta = bundle_dir.join("_root").join("bundle.meta.json");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root_meta.parent().expect("root meta parent"))
            .expect("create root meta dir");
        std::fs::write(
            &root_meta,
            serde_json::to_vec(&serde_json::json!({
                "kind": "bundle_meta",
                "schema_version": 1,
                "warmup_frames": 0,
            }))
            .unwrap(),
        )
        .expect("write root meta sidecar");

        let paths = resolve_meta_artifact_paths_from_bundle_dir(&bundle_dir, 0)
            .expect("resolve bundle dir");

        assert_eq!(paths.canonical_path, root_meta);
        assert_eq!(paths.default_out, paths.canonical_path);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prepare_cmd_meta_defaults_to_resolved_default_out() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-meta-default-out-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        let bundle_path = bundle_dir.join("bundle.json");
        std::fs::write(
            &bundle_path,
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 2,
                "run_id": "run-1",
                "windows": [],
                "warmup_frames": 0,
            }))
            .unwrap(),
        )
        .expect("write bundle json");

        let prepared = prepare_cmd_meta(
            MetaCommandRequest {
                rest: &["captures/demo/bundle.json".to_string()],
                workspace_root: &root,
                meta_out: None,
                stats_json: false,
                meta_report: false,
            },
            0,
        )
        .expect("prepare meta command");

        assert_eq!(prepared.out, crate::default_meta_out_path(&bundle_path));
        assert!(matches!(prepared.display_mode, ArtifactDisplayMode::Path));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prepare_cmd_meta_uses_custom_out_and_meta_report_mode() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-meta-custom-out-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        let sidecar = root.join("bundle.meta.json");
        std::fs::write(
            &sidecar,
            serde_json::to_vec(&serde_json::json!({
                "kind": "bundle_meta",
                "schema_version": 1,
                "warmup_frames": 0,
            }))
            .unwrap(),
        )
        .expect("write meta sidecar");

        let prepared = prepare_cmd_meta(
            MetaCommandRequest {
                rest: &["bundle.meta.json".to_string()],
                workspace_root: &root,
                meta_out: Some(PathBuf::from("exports/meta.json")),
                stats_json: false,
                meta_report: true,
            },
            0,
        )
        .expect("prepare meta command");

        assert_eq!(prepared.canonical_path, sidecar);
        assert_eq!(prepared.out, root.join("exports").join("meta.json"));
        assert!(matches!(
            prepared.display_mode,
            ArtifactDisplayMode::MetaReport
        ));
        let _ = std::fs::remove_dir_all(&root);
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
        build_triage_lite_payload(bundle_path, metric, stats_top, warmup_frames)?
    } else {
        build_triage_full_payload(bundle_path, stats_top, sort_override, warmup_frames)?
    };

    append_triage_tooling_warnings(&mut payload, bundle_path);
    Ok(payload)
}

fn build_triage_lite_payload(
    bundle_path: &Path,
    metric: crate::frames_index::TriageLiteMetric,
    stats_top: usize,
    warmup_frames: u64,
) -> Result<serde_json::Value, String> {
    let (index_path, frames_index) = resolve_triage_frames_index(bundle_path, warmup_frames)?;
    crate::frames_index::triage_lite_json_from_frames_index(
        bundle_path,
        &index_path,
        &frames_index,
        warmup_frames,
        stats_top,
        metric,
    )
}

fn resolve_triage_frames_index(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(PathBuf, serde_json::Value), String> {
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

    Ok((index_path, frames_index))
}

fn build_triage_full_payload(
    bundle_path: &Path,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
) -> Result<serde_json::Value, String> {
    let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);
    let report = bundle_stats_from_path(
        bundle_path,
        stats_top,
        sort,
        BundleStatsOptions { warmup_frames },
    )?;
    Ok(crate::triage_json_from_stats(
        bundle_path,
        &report,
        sort,
        warmup_frames,
    ))
}

fn append_triage_tooling_warnings(payload: &mut serde_json::Value, bundle_path: &Path) {
    let Some(bundle_dir) = bundle_path.parent() else {
        return;
    };
    let warnings = crate::tooling_warnings::tooling_warnings_for_bundle_dir(bundle_dir);
    if warnings.is_empty() {
        return;
    }
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(
            "tooling_warnings".to_string(),
            serde_json::Value::Array(warnings),
        );
    }
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
    let pretty = json_artifact_payload_to_pretty_text(payload);
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

    if meta_sidecar_source_is_direct(&src) {
        resolve_meta_artifact_paths_from_meta_sidecar(&src, warmup_frames)
    } else if src.is_dir() {
        resolve_meta_artifact_paths_from_bundle_dir(&src, warmup_frames)
    } else {
        build_meta_artifact_paths_from_bundle_path(
            &crate::resolve_bundle_artifact_path(&src),
            warmup_frames,
        )
    }
}

fn meta_sidecar_source_is_direct(src: &Path) -> bool {
    src.is_file()
        && src
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "bundle.meta.json")
}

fn build_meta_artifact_paths_from_bundle_path(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<MetaArtifactPaths, String> {
    let canonical_path = crate::bundle_index::ensure_bundle_meta_json(bundle_path, warmup_frames)?;
    let default_out = crate::default_meta_out_path(bundle_path);
    Ok(MetaArtifactPaths {
        canonical_path,
        default_out,
    })
}

fn resolve_meta_artifact_paths_from_meta_sidecar(
    src: &Path,
    warmup_frames: u64,
) -> Result<MetaArtifactPaths, String> {
    if sidecars::try_read_sidecar_json_v1(src, "bundle_meta", warmup_frames).is_some() {
        return Ok(MetaArtifactPaths {
            canonical_path: src.to_path_buf(),
            default_out: src.to_path_buf(),
        });
    }

    let Some(bundle_path) = sidecars::adjacent_bundle_path_for_sidecar(src) else {
        return Err(format!(
            "invalid bundle.meta.json (expected schema_version=1 warmup_frames={warmup_frames}) and no adjacent bundle artifact was found to regenerate it\n  meta: {}",
            src.display()
        ));
    };
    build_meta_artifact_paths_from_bundle_path(&bundle_path, warmup_frames)
}

fn resolve_meta_artifact_paths_from_bundle_dir(
    src: &Path,
    warmup_frames: u64,
) -> Result<MetaArtifactPaths, String> {
    let direct = src.join("bundle.meta.json");
    if direct.is_file()
        && sidecars::try_read_sidecar_json_v1(&direct, "bundle_meta", warmup_frames).is_some()
    {
        return Ok(MetaArtifactPaths {
            canonical_path: direct.clone(),
            default_out: direct,
        });
    }

    let root = src.join("_root").join("bundle.meta.json");
    if root.is_file()
        && sidecars::try_read_sidecar_json_v1(&root, "bundle_meta", warmup_frames).is_some()
    {
        return Ok(MetaArtifactPaths {
            canonical_path: root.clone(),
            default_out: root,
        });
    }

    build_meta_artifact_paths_from_bundle_path(
        &crate::resolve_bundle_artifact_path(src),
        warmup_frames,
    )
}

fn materialize_canonical_artifact_output(canonical_path: &Path, out: &Path) -> Result<(), String> {
    if artifact_output_materialization_is_noop(canonical_path, out) {
        return Ok(());
    }

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::copy(canonical_path, out).map_err(|e| e.to_string())?;
    Ok(())
}

fn artifact_output_materialization_is_noop(canonical_path: &Path, out: &Path) -> bool {
    out.is_file() || out == canonical_path
}

fn emit_artifact_output(out: &Path, display_mode: ArtifactDisplayMode) -> Result<(), String> {
    match display_mode {
        ArtifactDisplayMode::Path => {
            println!("{}", out.display());
            Ok(())
        }
        ArtifactDisplayMode::Json => {
            println!("{}", read_artifact_output_text(out)?);
            Ok(())
        }
        ArtifactDisplayMode::MetaReport => {
            let meta = read_artifact_output_json(out)?;
            print_meta_report(&meta, out);
            Ok(())
        }
    }
}

fn prepare_cmd_lint(request: LintCommandRequest<'_>) -> Result<PreparedLintCommand, String> {
    let bundle_path = resolve_required_bundle_artifact(RequiredBundleArtifactRequest {
        rest: request.rest,
        workspace_root: request.workspace_root,
        missing_hint: "missing bundle artifact path (try: fretboard diag lint <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)",
    })?;
    let out = request
        .lint_out
        .map(|path| crate::resolve_path(request.workspace_root, path))
        .unwrap_or_else(|| crate::default_lint_out_path(&bundle_path));

    Ok(PreparedLintCommand { bundle_path, out })
}

fn prepare_cmd_triage(request: TriageCommandRequest<'_>) -> Result<PreparedTriageCommand, String> {
    let parsed = parse_triage_request(request.rest, request.workspace_root)?;
    let resolved = resolve::resolve_bundle_ref(&parsed.source)?;
    let bundle_path = resolved.bundle_artifact;
    let out = request
        .triage_out
        .map(|path| crate::resolve_path(request.workspace_root, path))
        .unwrap_or_else(|| default_triage_out_path(&bundle_path, parsed.lite));

    Ok(PreparedTriageCommand {
        bundle_path,
        out,
        lite: parsed.lite,
        metric: parsed.metric,
    })
}

fn prepare_cmd_test_ids(
    request: TestIdsCommandRequest<'_>,
) -> Result<PreparedTestIdsCommand, String> {
    let bundle_path = resolve_required_bundle_artifact(RequiredBundleArtifactRequest {
        rest: request.rest,
        workspace_root: request.workspace_root,
        missing_hint: "missing bundle artifact path (try: fretboard diag test-ids <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)",
    })?;
    let out = request
        .test_ids_out
        .map(|path| crate::resolve_path(request.workspace_root, path))
        .unwrap_or_else(|| crate::default_test_ids_out_path(&bundle_path));

    Ok(PreparedTestIdsCommand { bundle_path, out })
}

fn prepare_cmd_meta(
    request: MetaCommandRequest<'_>,
    warmup_frames: u64,
) -> Result<PreparedMetaCommand, String> {
    let parsed = parse_meta_request(
        request.rest,
        request.workspace_root,
        request.stats_json,
        request.meta_report,
    )?;
    let paths = resolve_meta_artifact_paths(&parsed.source, warmup_frames)?;
    let out = request
        .meta_out
        .map(|path| crate::resolve_path(request.workspace_root, path))
        .unwrap_or(paths.default_out);

    Ok(PreparedMetaCommand {
        canonical_path: paths.canonical_path,
        out,
        display_mode: parsed.display_mode,
    })
}

fn lint_exit_required(error_issues: u64) -> bool {
    error_issues > 0
}

fn write_lint_report_output(
    out: &Path,
    report: &LintReport,
    stats_json: bool,
) -> Result<bool, String> {
    write_json_artifact_output(out, &report.payload, stats_json)?;
    Ok(lint_exit_required(report.error_issues))
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

    let PreparedTriageCommand {
        bundle_path,
        out,
        lite,
        metric,
    } = prepare_cmd_triage(TriageCommandRequest {
        rest,
        workspace_root,
        triage_out,
    })?;
    let payload = build_triage_payload(
        &bundle_path,
        lite,
        metric,
        stats_top,
        sort_override,
        warmup_frames,
    )?;
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
    let PreparedLintCommand { bundle_path, out } = prepare_cmd_lint(LintCommandRequest {
        rest,
        workspace_root,
        lint_out,
    })?;

    let report = lint_bundle_from_path(
        &bundle_path,
        warmup_frames,
        LintOptions {
            all_test_ids_bounds: lint_all_test_ids_bounds,
            eps_px: lint_eps_px,
        },
    )?;

    if write_lint_report_output(&out, &report, stats_json)? {
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
    let PreparedTestIdsCommand { bundle_path, out } =
        prepare_cmd_test_ids(TestIdsCommandRequest {
            rest,
            workspace_root,
            test_ids_out,
        })?;

    if emit_existing_artifact_output_if_present(&out, stats_json)? {
        return Ok(());
    }

    let canonical =
        crate::bundle_index::ensure_test_ids_json(&bundle_path, warmup_frames, max_test_ids)?;
    materialize_canonical_artifact_output(&canonical, &out)?;

    emit_path_or_json_output(&out, stats_json)?;
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
    let bundle_path = resolve_required_bundle_artifact(RequiredBundleArtifactRequest {
        rest,
        workspace_root,
        missing_hint: "missing bundle artifact path (try: fretboard diag test-ids-index <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)",
    })?;
    ensure_and_emit_bundle_artifact_output(
        &bundle_path,
        warmup_frames,
        stats_json,
        crate::bundle_index::ensure_test_ids_index_json,
    )
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
    let bundle_path = resolve_required_bundle_artifact(RequiredBundleArtifactRequest {
        rest,
        workspace_root,
        missing_hint: "missing bundle artifact path (try: fretboard diag frames-index <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)",
    })?;
    ensure_and_emit_bundle_artifact_output(
        &bundle_path,
        warmup_frames,
        stats_json,
        crate::frames_index::ensure_frames_index_json,
    )
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

    let PreparedMetaCommand {
        canonical_path,
        out,
        display_mode,
    } = prepare_cmd_meta(
        MetaCommandRequest {
            rest,
            workspace_root,
            meta_out,
            stats_json,
            meta_report,
        },
        warmup_frames,
    )?;
    materialize_canonical_artifact_output(&canonical_path, &out)?;
    emit_artifact_output(&out, display_mode)
}

fn print_meta_report(meta: &serde_json::Value, meta_path: &Path) {
    for line in meta_report_lines(meta, meta_path) {
        println!("{line}");
    }
}

fn meta_report_u64_field(value: &serde_json::Value, key: &str) -> u64 {
    value.get(key).and_then(|entry| entry.as_u64()).unwrap_or(0)
}

fn meta_report_str_field<'a>(value: &'a serde_json::Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(|entry| entry.as_str())
        .unwrap_or("")
}

fn meta_report_lines(meta: &serde_json::Value, meta_path: &Path) -> Vec<String> {
    let mut lines = vec![
        "bundle_meta:".to_string(),
        format!("  meta_json: {}", meta_path.display()),
        format!("  bundle: {}", meta_report_str_field(meta, "bundle")),
        format!(
            "  warmup_frames: {}",
            meta_report_u64_field(meta, "warmup_frames")
        ),
        format!(
            "  windows_total: {}",
            meta_report_u64_field(meta, "windows_total")
        ),
        format!(
            "  snapshots_total: {}",
            meta_report_u64_field(meta, "snapshots_total")
        ),
        format!(
            "  semantics: resolved={} inline={} table={} table_entries={} table_unique_keys={}",
            meta_report_u64_field(meta, "snapshots_with_semantics_total"),
            meta_report_u64_field(meta, "snapshots_with_inline_semantics_total"),
            meta_report_u64_field(meta, "snapshots_with_table_semantics_total"),
            meta_report_u64_field(meta, "semantics_table_entries_total"),
            meta_report_u64_field(meta, "semantics_table_unique_keys_total"),
        ),
    ];

    let Some(windows) = meta.get("windows").and_then(|value| value.as_array()) else {
        return lines;
    };
    if windows.is_empty() {
        return lines;
    }

    lines.push("  windows:".to_string());
    let max = 6usize;
    for window in windows.iter().take(max) {
        let considered_frame_id = window
            .get("considered_frame_id")
            .and_then(|value| value.as_u64())
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_string());
        lines.push(format!(
            "    - window={} snapshots={} considered_frame={} semantics(resolved/inline/table)={}/{}/{} table(entries/keys)={}/{}",
            meta_report_u64_field(window, "window"),
            meta_report_u64_field(window, "snapshots_total"),
            considered_frame_id,
            meta_report_u64_field(window, "snapshots_with_semantics_total"),
            meta_report_u64_field(window, "snapshots_with_inline_semantics_total"),
            meta_report_u64_field(window, "snapshots_with_table_semantics_total"),
            meta_report_u64_field(window, "semantics_table_entries_total"),
            meta_report_u64_field(window, "semantics_table_unique_keys_total"),
        ));
    }
    if windows.len() > max {
        lines.push(format!("    - ... ({} more)", windows.len() - max));
    }
    lines
}
