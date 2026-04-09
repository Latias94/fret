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

enum PackExecutionMode {
    AiOnly,
    Bundle {
        include_root_artifacts: bool,
        include_triage: bool,
        include_screenshots: bool,
        schema2_only: bool,
        stats_top: usize,
        sort: BundleStatsSort,
        warmup_frames: u64,
    },
}

struct PackExecutionPlan {
    bundle_dir: PathBuf,
    artifacts_root: PathBuf,
    out: PathBuf,
    mode: PackExecutionMode,
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

struct EnsuredBundleArtifactCommandRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    missing_hint: &'a str,
    warmup_frames: u64,
    stats_json: bool,
}

struct RequiredBundleArtifactOutputRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    missing_hint: &'a str,
    custom_out: Option<PathBuf>,
    default_out: fn(&Path) -> PathBuf,
}

struct PreparedBundleArtifactOutput {
    bundle_path: PathBuf,
    out: PathBuf,
}

struct EnsuredBundleArtifactPlan {
    bundle_path: PathBuf,
    warmup_frames: u64,
    display_mode: ArtifactDisplayMode,
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

struct LintReportOutput {
    presentation: ArtifactOutputPresentation,
    exit_required: bool,
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

struct TriageCommandOutput {
    presentation: ArtifactOutputPresentation,
}

struct TriageExecutionOptions {
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
    stats_json: bool,
}

struct TriageExecutionPlan {
    payload: TriagePayloadPlan,
    out: PathBuf,
    stats_json: bool,
}

#[derive(Debug)]
struct PackCommandOutput {
    presentation: ArtifactOutputPresentation,
}

#[derive(Debug)]
struct EnsuredBundleArtifactOutput {
    presentation: ArtifactOutputPresentation,
}

#[derive(Debug)]
struct GeneratedArtifactOutput {
    presentation: ArtifactOutputPresentation,
}

struct ArtifactMaterializationPlan {
    out: PathBuf,
    display_mode: ArtifactDisplayMode,
    mode: ArtifactMaterializationMode,
}

#[derive(Debug, PartialEq, Eq)]
enum ArtifactMaterializationMode {
    ReuseExistingOut,
    ReuseCanonicalOut,
    CopyCanonical { canonical_path: PathBuf },
}

enum TriagePayloadMode {
    Lite {
        metric: crate::frames_index::TriageLiteMetric,
    },
    Full {
        sort: BundleStatsSort,
    },
}

struct TriagePayloadPlan {
    bundle_path: PathBuf,
    stats_top: usize,
    warmup_frames: u64,
    mode: TriagePayloadMode,
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

struct TestIdsExecutionPlan {
    bundle_path: PathBuf,
    out: PathBuf,
    warmup_frames: u64,
    max_test_ids: usize,
    display_mode: ArtifactDisplayMode,
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

struct MetaExecutionPlan {
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
    let prepared = prepare_cmd_pack(PackCommandRequest {
        rest,
        workspace_root,
        out_dir,
        pack_out,
        pack_ai_only,
    })?;

    if ensure_ai_packet || pack_ai_only {
        ensure_ai_packet_dir_for_pack(PackAiPacketEnsureRequest {
            bundle_dir: &prepared.bundle_dir,
            pack_include_triage,
            stats_top,
            sort_override,
            warmup_frames,
        });
    }
    let plan = build_pack_execution_plan(
        prepared,
        PackExecutionOptions {
            pack_ai_only,
            pack_include_root_artifacts,
            pack_include_triage,
            pack_include_screenshots,
            pack_schema2_only,
            stats_top,
            sort_override,
            warmup_frames,
        },
    );
    let output = build_pack_command_output(&plan)?;
    emit_artifact_output_presentation(output.presentation);
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
                    "{} (try: fretboard-dev diag pack ./target/fret-diag/<timestamp>)",
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

struct PackExecutionOptions {
    pack_ai_only: bool,
    pack_include_root_artifacts: bool,
    pack_include_triage: bool,
    pack_include_screenshots: bool,
    pack_schema2_only: bool,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
}

fn build_pack_execution_plan(
    prepared: PreparedPackCommand,
    options: PackExecutionOptions,
) -> PackExecutionPlan {
    let mode = if options.pack_ai_only {
        PackExecutionMode::AiOnly
    } else {
        PackExecutionMode::Bundle {
            include_root_artifacts: options.pack_include_root_artifacts,
            include_triage: options.pack_include_triage,
            include_screenshots: options.pack_include_screenshots,
            schema2_only: options.pack_schema2_only,
            stats_top: options.stats_top,
            sort: options
                .sort_override
                .unwrap_or(BundleStatsSort::Invalidation),
            warmup_frames: options.warmup_frames,
        }
    };
    PackExecutionPlan {
        bundle_dir: prepared.bundle_dir,
        artifacts_root: prepared.artifacts_root,
        out: prepared.out,
        mode,
    }
}

fn execute_pack_execution_plan(plan: &PackExecutionPlan) -> Result<(), String> {
    match &plan.mode {
        PackExecutionMode::AiOnly => {
            require_ai_packet_dir_for_pack_ai_only(&plan.bundle_dir)?;
            crate::pack_ai_packet_dir_to_zip(&plan.bundle_dir, &plan.out, &plan.artifacts_root)
        }
        PackExecutionMode::Bundle {
            include_root_artifacts,
            include_triage,
            include_screenshots,
            schema2_only,
            stats_top,
            sort,
            warmup_frames,
        } => crate::pack_bundle_dir_to_zip(
            &plan.bundle_dir,
            &plan.out,
            *include_root_artifacts,
            *include_triage,
            *include_screenshots,
            *schema2_only,
            false,
            false,
            &plan.artifacts_root,
            *stats_top,
            *sort,
            *warmup_frames,
        ),
    }
}

fn build_pack_command_output(plan: &PackExecutionPlan) -> Result<PackCommandOutput, String> {
    execute_pack_execution_plan(plan)?;
    Ok(PackCommandOutput {
        presentation: ArtifactOutputPresentation::Path(plan.out.clone()),
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
            "--ai-only requires ai.packet under the bundle dir (tip: fretboard-dev diag ai-packet {} --packet-out {})",
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

fn resolve_command_out_path<F>(
    workspace_root: &Path,
    custom_out: Option<PathBuf>,
    default_out: F,
) -> PathBuf
where
    F: FnOnce() -> PathBuf,
{
    custom_out
        .map(|path| crate::resolve_path(workspace_root, path))
        .unwrap_or_else(default_out)
}

fn prepare_required_bundle_artifact_output(
    request: RequiredBundleArtifactOutputRequest<'_>,
) -> Result<PreparedBundleArtifactOutput, String> {
    let bundle_path = resolve_required_bundle_artifact(RequiredBundleArtifactRequest {
        rest: request.rest,
        workspace_root: request.workspace_root,
        missing_hint: request.missing_hint,
    })?;
    let out = resolve_command_out_path(request.workspace_root, request.custom_out, || {
        (request.default_out)(&bundle_path)
    });
    Ok(PreparedBundleArtifactOutput { bundle_path, out })
}

fn build_ensured_bundle_artifact_plan(
    bundle_path: &Path,
    warmup_frames: u64,
    stats_json: bool,
) -> EnsuredBundleArtifactPlan {
    EnsuredBundleArtifactPlan {
        bundle_path: bundle_path.to_path_buf(),
        warmup_frames,
        display_mode: artifact_display_mode_for_stats_json(stats_json),
    }
}

fn build_required_bundle_artifact_plan(
    request: EnsuredBundleArtifactCommandRequest<'_>,
) -> Result<EnsuredBundleArtifactPlan, String> {
    let bundle_path = resolve_required_bundle_artifact(RequiredBundleArtifactRequest {
        rest: request.rest,
        workspace_root: request.workspace_root,
        missing_hint: request.missing_hint,
    })?;
    Ok(build_ensured_bundle_artifact_plan(
        &bundle_path,
        request.warmup_frames,
        request.stats_json,
    ))
}

fn build_ensured_bundle_artifact_output<F>(
    plan: EnsuredBundleArtifactPlan,
    ensure_artifact_output: F,
) -> Result<EnsuredBundleArtifactOutput, String>
where
    F: FnOnce(&Path, u64) -> Result<PathBuf, String>,
{
    let out = ensure_artifact_output(&plan.bundle_path, plan.warmup_frames)?;
    let presentation = build_artifact_output_presentation(&out, plan.display_mode)?;
    Ok(EnsuredBundleArtifactOutput { presentation })
}

fn build_required_bundle_artifact_output<F>(
    request: EnsuredBundleArtifactCommandRequest<'_>,
    ensure_artifact_output: F,
) -> Result<EnsuredBundleArtifactOutput, String>
where
    F: FnOnce(&Path, u64) -> Result<PathBuf, String>,
{
    let plan = build_required_bundle_artifact_plan(request)?;
    build_ensured_bundle_artifact_output(plan, ensure_artifact_output)
}

fn read_artifact_output_text(out: &Path) -> Result<String, String> {
    std::fs::read_to_string(out).map_err(|e| e.to_string())
}

fn read_artifact_output_json(out: &Path) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(out).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn artifact_display_mode_for_stats_json(stats_json: bool) -> ArtifactDisplayMode {
    if stats_json {
        ArtifactDisplayMode::Json
    } else {
        ArtifactDisplayMode::Path
    }
}

fn json_artifact_payload_to_pretty_text(payload: &serde_json::Value) -> String {
    serde_json::to_string_pretty(payload).unwrap_or_else(|_| "{}".to_string())
}

fn build_generated_artifact_output<F>(
    out: &Path,
    display_mode: ArtifactDisplayMode,
    reuse_existing_out: bool,
    resolve_canonical_path: F,
) -> Result<GeneratedArtifactOutput, String>
where
    F: FnOnce() -> Result<PathBuf, String>,
{
    let plan = build_artifact_materialization_plan(
        out,
        display_mode,
        reuse_existing_out,
        resolve_canonical_path,
    )?;
    execute_artifact_materialization_plan(&plan)?;
    let presentation = build_artifact_output_presentation(&plan.out, plan.display_mode)?;
    Ok(GeneratedArtifactOutput { presentation })
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
    fn build_pack_execution_plan_uses_ai_only_mode_when_requested() {
        let plan = build_pack_execution_plan(
            PreparedPackCommand {
                bundle_dir: PathBuf::from("captures/demo"),
                artifacts_root: PathBuf::from("target/fret-diag"),
                out: PathBuf::from("target/fret-diag/share/demo.ai.zip"),
            },
            PackExecutionOptions {
                pack_ai_only: true,
                pack_include_root_artifacts: true,
                pack_include_triage: true,
                pack_include_screenshots: true,
                pack_schema2_only: true,
                stats_top: 99,
                sort_override: Some(BundleStatsSort::Time),
                warmup_frames: 7,
            },
        );

        assert!(matches!(plan.mode, PackExecutionMode::AiOnly));
    }

    #[test]
    fn build_pack_execution_plan_defaults_bundle_sort_to_invalidation() {
        let plan = build_pack_execution_plan(
            PreparedPackCommand {
                bundle_dir: PathBuf::from("captures/demo"),
                artifacts_root: PathBuf::from("target/fret-diag"),
                out: PathBuf::from("target/fret-diag/share/demo.zip"),
            },
            PackExecutionOptions {
                pack_ai_only: false,
                pack_include_root_artifacts: true,
                pack_include_triage: false,
                pack_include_screenshots: true,
                pack_schema2_only: false,
                stats_top: 12,
                sort_override: None,
                warmup_frames: 5,
            },
        );

        match plan.mode {
            PackExecutionMode::AiOnly => panic!("expected bundle mode"),
            PackExecutionMode::Bundle {
                include_root_artifacts,
                include_triage,
                include_screenshots,
                schema2_only,
                stats_top,
                sort,
                warmup_frames,
            } => {
                assert!(include_root_artifacts);
                assert!(!include_triage);
                assert!(include_screenshots);
                assert!(!schema2_only);
                assert_eq!(stats_top, 12);
                assert_eq!(sort, BundleStatsSort::Invalidation);
                assert_eq!(warmup_frames, 5);
            }
        }
    }

    #[test]
    fn build_pack_command_output_uses_path_presentation() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-pack-output-presentation-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("bundle");
        let packet_dir = bundle_dir.join("ai.packet");
        std::fs::create_dir_all(&packet_dir).expect("create ai.packet dir");
        std::fs::write(packet_dir.join("bundle.meta.json"), b"{}").expect("write bundle meta");

        let out = root.join("share").join("bundle.ai.zip");
        let plan = PackExecutionPlan {
            bundle_dir,
            artifacts_root: root.clone(),
            out: out.clone(),
            mode: PackExecutionMode::AiOnly,
        };

        let output = build_pack_command_output(&plan).expect("build pack output");

        match output.presentation {
            ArtifactOutputPresentation::Path(path) => assert_eq!(path, out),
            other => panic!("expected path presentation, got {other:?}"),
        }

        let _ = std::fs::remove_dir_all(&root);
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
    fn resolve_command_out_path_prefers_custom_out_under_workspace_root() {
        let out = resolve_command_out_path(
            Path::new("workspace-root"),
            Some(PathBuf::from("exports/triage.json")),
            || PathBuf::from("default/triage.json"),
        );

        assert_eq!(out, PathBuf::from("workspace-root/exports/triage.json"));
    }

    #[test]
    fn resolve_command_out_path_uses_default_when_custom_out_missing() {
        let out = resolve_command_out_path(Path::new("workspace-root"), None, || {
            PathBuf::from("captures/demo/triage.json")
        });

        assert_eq!(out, PathBuf::from("captures/demo/triage.json"));
    }

    #[test]
    fn prepare_required_bundle_artifact_output_uses_default_out_builder() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-required-bundle-output-default-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        let bundle_path = bundle_dir.join("bundle.json");
        std::fs::write(&bundle_path, b"{}" as &[u8]).expect("write bundle");

        let prepared =
            prepare_required_bundle_artifact_output(RequiredBundleArtifactOutputRequest {
                rest: &["captures/demo/bundle.json".to_string()],
                workspace_root: &root,
                missing_hint: "missing bundle",
                custom_out: None,
                default_out: crate::default_test_ids_out_path,
            })
            .expect("prepare required bundle output");

        assert_eq!(prepared.bundle_path, bundle_path);
        assert_eq!(
            prepared.out,
            crate::default_test_ids_out_path(&prepared.bundle_path)
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prepare_required_bundle_artifact_output_uses_custom_out_when_provided() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-required-bundle-output-custom-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.json"), b"{}" as &[u8]).expect("write bundle");

        let prepared =
            prepare_required_bundle_artifact_output(RequiredBundleArtifactOutputRequest {
                rest: &["captures/demo/bundle.json".to_string()],
                workspace_root: &root,
                missing_hint: "missing bundle",
                custom_out: Some(PathBuf::from("exports/test-ids.json")),
                default_out: crate::default_test_ids_out_path,
            })
            .expect("prepare required bundle output");

        assert_eq!(prepared.out, root.join("exports").join("test-ids.json"));
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
    fn prepare_cmd_lint_defaults_to_default_out_path() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-lint-default-out-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.json"), b"{}" as &[u8]).expect("write bundle json");

        let prepared = prepare_cmd_lint(LintCommandRequest {
            rest: &["captures/demo/bundle.json".to_string()],
            workspace_root: &root,
            lint_out: None,
        })
        .expect("prepare lint command");

        assert_eq!(prepared.bundle_path, bundle_dir.join("bundle.json"));
        assert_eq!(
            prepared.out,
            crate::default_lint_out_path(&bundle_dir.join("bundle.json"))
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prepare_cmd_lint_uses_custom_out_when_provided() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-prepare-lint-custom-out-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.json"), b"{}" as &[u8]).expect("write bundle json");

        let prepared = prepare_cmd_lint(LintCommandRequest {
            rest: &["captures/demo/bundle.json".to_string()],
            workspace_root: &root,
            lint_out: Some(PathBuf::from("exports/lint.json")),
        })
        .expect("prepare lint command");

        assert_eq!(prepared.out, root.join("exports").join("lint.json"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_triage_command_output_uses_path_presentation_when_not_printing_json() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-triage-command-output-path-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.json"), b"{}" as &[u8]).expect("write bundle json");
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
            .expect("serialize frames index"),
        )
        .expect("write frames index");

        let output = build_triage_command_output(build_triage_execution_plan(
            PreparedTriageCommand {
                bundle_path: bundle_dir.join("bundle.json"),
                out: bundle_dir.join("triage.lite.json"),
                lite: true,
                metric: crate::frames_index::TriageLiteMetric::TotalTimeUs,
            },
            TriageExecutionOptions {
                stats_top: 5,
                sort_override: None,
                warmup_frames: 0,
                stats_json: false,
            },
        ))
        .expect("build triage output");

        assert_eq!(
            output.presentation,
            ArtifactOutputPresentation::Path(bundle_dir.join("triage.lite.json"))
        );
        assert!(bundle_dir.join("triage.lite.json").is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_triage_command_output_uses_text_presentation_when_printing_json() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-triage-command-output-text-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.json"), b"{}" as &[u8]).expect("write bundle json");
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
            .expect("serialize frames index"),
        )
        .expect("write frames index");

        let output = build_triage_command_output(build_triage_execution_plan(
            PreparedTriageCommand {
                bundle_path: bundle_dir.join("bundle.json"),
                out: bundle_dir.join("triage.lite.json"),
                lite: true,
                metric: crate::frames_index::TriageLiteMetric::TotalTimeUs,
            },
            TriageExecutionOptions {
                stats_top: 5,
                sort_override: None,
                warmup_frames: 0,
                stats_json: true,
            },
        ))
        .expect("build triage output");

        match output.presentation {
            ArtifactOutputPresentation::Text(text) => {
                assert!(text.contains("\"kind\": \"triage_lite\""));
            }
            ArtifactOutputPresentation::Path(_) => panic!("expected text presentation"),
            ArtifactOutputPresentation::Lines(_) => panic!("expected text presentation"),
        }
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
    fn build_test_ids_execution_plan_keeps_prepared_fields() {
        let plan = build_test_ids_execution_plan(
            PreparedTestIdsCommand {
                bundle_path: PathBuf::from("captures/demo/bundle.json"),
                out: PathBuf::from("exports/test-ids.json"),
            },
            7,
            42,
            true,
        );

        assert_eq!(plan.bundle_path, PathBuf::from("captures/demo/bundle.json"));
        assert_eq!(plan.out, PathBuf::from("exports/test-ids.json"));
        assert_eq!(plan.warmup_frames, 7);
        assert_eq!(plan.max_test_ids, 42);
        assert!(matches!(plan.display_mode, ArtifactDisplayMode::Json));
    }

    #[test]
    fn build_test_ids_command_output_uses_path_presentation() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-test-ids-command-output-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(
            bundle_dir.join("bundle.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 2,
                "run_id": "run-1",
                "windows": [],
                "warmup_frames": 0,
            }))
            .expect("serialize bundle json"),
        )
        .expect("write bundle json");

        let out = bundle_dir.join("test-ids.json");
        let output = build_test_ids_command_output(TestIdsExecutionPlan {
            bundle_path: bundle_dir.join("bundle.json"),
            out: out.clone(),
            warmup_frames: 0,
            max_test_ids: 8,
            display_mode: ArtifactDisplayMode::Path,
        })
        .expect("build test ids command output");

        assert_eq!(
            output.presentation,
            ArtifactOutputPresentation::Path(out.clone())
        );
        assert!(out.is_file());
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
    fn build_lint_report_output_returns_false_when_no_error_issues() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-lint-report-ok-{}-{}",
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

        let output =
            build_lint_report_output(&out, &report, false).expect("build lint report output");

        assert!(!output.exit_required);
        assert!(out.is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_lint_report_output_returns_true_when_error_issues_exist() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-lint-report-exit-{}-{}",
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

        let output =
            build_lint_report_output(&out, &report, false).expect("build lint report output");

        assert!(output.exit_required);
        assert!(out.is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_artifact_materialization_plan_reuses_canonical_out_when_paths_match() {
        let canonical = Path::new("captures/demo/test-ids.json");

        let plan = build_artifact_materialization_plan(
            canonical,
            ArtifactDisplayMode::Path,
            false,
            || Ok(canonical.to_path_buf()),
        )
        .expect("build materialization plan");

        assert_eq!(plan.out, canonical);
        assert!(matches!(
            plan.mode,
            ArtifactMaterializationMode::ReuseCanonicalOut
        ));
    }

    #[test]
    fn build_artifact_materialization_plan_reuses_existing_out_when_allowed() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-materialization-plan-existing-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let canonical = root.join("canonical.json");
        let out = root.join("existing.json");
        std::fs::write(&canonical, b"canonical").expect("write canonical");
        std::fs::write(&out, b"existing").expect("write existing out");

        let plan =
            build_artifact_materialization_plan(&out, ArtifactDisplayMode::Path, true, || {
                Err("should not resolve canonical".to_string())
            })
            .expect("build materialization plan");

        assert_eq!(plan.out, out);
        assert!(matches!(
            plan.mode,
            ArtifactMaterializationMode::ReuseExistingOut
        ));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_artifact_materialization_plan_keeps_existing_out_file() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-execute-materialization-existing-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let out = root.join("existing.json");
        std::fs::write(&out, b"existing").expect("write existing out");

        execute_artifact_materialization_plan(&ArtifactMaterializationPlan {
            out: out.clone(),
            display_mode: ArtifactDisplayMode::Path,
            mode: ArtifactMaterializationMode::ReuseExistingOut,
        })
        .expect("execute materialization plan");

        assert_eq!(
            std::fs::read_to_string(&out).expect("read existing out"),
            "existing"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_artifact_materialization_plan_copies_to_nested_out_path() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-execute-materialization-copy-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let canonical = root.join("canonical.json");
        let out = root.join("nested").join("copy.json");
        std::fs::write(&canonical, b"canonical").expect("write canonical");

        execute_artifact_materialization_plan(&ArtifactMaterializationPlan {
            out: out.clone(),
            display_mode: ArtifactDisplayMode::Path,
            mode: ArtifactMaterializationMode::CopyCanonical {
                canonical_path: canonical,
            },
        })
        .expect("execute materialization plan");

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

        let result = build_ensured_bundle_artifact_output(
            build_ensured_bundle_artifact_plan(Path::new("bundle.json"), 0, false),
            |_, _| Ok(out.clone()),
        );

        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ensure_and_emit_bundle_artifact_output_propagates_ensure_error() {
        let result = build_ensured_bundle_artifact_output(
            build_ensured_bundle_artifact_plan(Path::new("bundle.json"), 0, false),
            |_, _| Err("boom".to_string()),
        );

        assert_eq!(result.expect_err("expected ensure error"), "boom");
    }

    #[test]
    fn ensure_and_emit_bundle_artifact_output_propagates_emit_error() {
        let missing = PathBuf::from("missing-artifact.json");

        let result = build_ensured_bundle_artifact_output(
            build_ensured_bundle_artifact_plan(Path::new("bundle.json"), 0, true),
            move |_, _| Ok(missing.clone()),
        );

        assert!(result.is_err());
    }

    #[test]
    fn build_ensured_bundle_artifact_output_uses_path_presentation_by_default() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-ensured-bundle-artifact-output-path-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let artifact = root.join("frames.index.json");
        std::fs::write(&artifact, b"{}" as &[u8]).expect("write artifact");

        let output = build_ensured_bundle_artifact_output(
            build_ensured_bundle_artifact_plan(Path::new("bundle.json"), 0, false),
            |_, _| Ok(artifact.clone()),
        )
        .expect("build ensured bundle artifact output");

        assert_eq!(
            output.presentation,
            ArtifactOutputPresentation::Path(artifact.clone())
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_ensured_bundle_artifact_output_uses_text_presentation_when_json_requested() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-ensured-bundle-artifact-output-text-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let artifact = root.join("frames.index.json");
        std::fs::write(&artifact, b"{\n  \"kind\": \"frames_index\"\n}" as &[u8])
            .expect("write artifact");

        let output = build_ensured_bundle_artifact_output(
            build_ensured_bundle_artifact_plan(Path::new("bundle.json"), 0, true),
            |_, _| Ok(artifact.clone()),
        )
        .expect("build ensured bundle artifact output");

        match output.presentation {
            ArtifactOutputPresentation::Text(text) => {
                assert!(text.contains("\"kind\": \"frames_index\""));
            }
            ArtifactOutputPresentation::Path(_) => panic!("expected text presentation"),
            ArtifactOutputPresentation::Lines(_) => panic!("expected text presentation"),
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_required_bundle_artifact_output_propagates_missing_hint() {
        let err = build_required_bundle_artifact_output(
            EnsuredBundleArtifactCommandRequest {
                rest: &[],
                workspace_root: Path::new("workspace-root"),
                missing_hint: "missing bundle",
                warmup_frames: 0,
                stats_json: false,
            },
            |_, _| Ok(PathBuf::from("artifact.json")),
        )
        .expect_err("expected missing input error");

        assert_eq!(err, "missing bundle");
    }

    #[test]
    fn build_ensured_bundle_artifact_plan_maps_stats_json_to_display_mode() {
        let plan = build_ensured_bundle_artifact_plan(Path::new("bundle.json"), 7, true);

        assert_eq!(plan.bundle_path, PathBuf::from("bundle.json"));
        assert_eq!(plan.warmup_frames, 7);
        assert!(matches!(plan.display_mode, ArtifactDisplayMode::Json));
    }

    #[test]
    fn build_required_bundle_artifact_plan_resolves_input_before_building_plan() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-required-bundle-artifact-plan-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        let bundle_path = bundle_dir.join("bundle.json");
        std::fs::write(&bundle_path, b"{}" as &[u8]).expect("write bundle");

        let plan = build_required_bundle_artifact_plan(EnsuredBundleArtifactCommandRequest {
            rest: &["captures/demo/bundle.json".to_string()],
            workspace_root: &root,
            missing_hint: "missing bundle",
            warmup_frames: 3,
            stats_json: false,
        })
        .expect("build required bundle artifact plan");

        assert_eq!(plan.bundle_path, bundle_path);
        assert_eq!(plan.warmup_frames, 3);
        assert!(matches!(plan.display_mode, ArtifactDisplayMode::Path));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_required_bundle_artifact_output_resolves_input_before_building_output() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-required-bundle-artifact-output-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bundle_dir = root.join("captures").join("demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        let bundle_path = bundle_dir.join("bundle.json");
        let artifact = bundle_dir.join("frames.index.json");
        std::fs::write(&bundle_path, b"{}" as &[u8]).expect("write bundle");
        std::fs::write(&artifact, b"{}" as &[u8]).expect("write artifact");

        let output = build_required_bundle_artifact_output(
            EnsuredBundleArtifactCommandRequest {
                rest: &["captures/demo/bundle.json".to_string()],
                workspace_root: &root,
                missing_hint: "missing bundle",
                warmup_frames: 0,
                stats_json: false,
            },
            |resolved_bundle_path, _| {
                assert_eq!(resolved_bundle_path, bundle_path.as_path());
                Ok(artifact.clone())
            },
        )
        .expect("build required bundle artifact output");

        assert_eq!(
            output.presentation,
            ArtifactOutputPresentation::Path(artifact.clone())
        );
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
    fn artifact_display_mode_for_stats_json_maps_to_path_or_json() {
        assert!(matches!(
            artifact_display_mode_for_stats_json(false),
            ArtifactDisplayMode::Path
        ));
        assert!(matches!(
            artifact_display_mode_for_stats_json(true),
            ArtifactDisplayMode::Json
        ));
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
    fn build_json_write_output_presentation_uses_text_when_printing_json() {
        let presentation = build_json_write_output_presentation(
            Path::new("out/triage.json"),
            "{\n  \"kind\": \"triage\"\n}".to_string(),
            true,
        );

        assert_eq!(
            presentation,
            ArtifactOutputPresentation::Text("{\n  \"kind\": \"triage\"\n}".to_string())
        );
    }

    #[test]
    fn build_json_write_output_presentation_uses_path_when_not_printing_json() {
        let presentation = build_json_write_output_presentation(
            Path::new("out/triage.json"),
            "{\n  \"kind\": \"triage\"\n}".to_string(),
            false,
        );

        assert_eq!(
            presentation,
            ArtifactOutputPresentation::Path(PathBuf::from("out/triage.json"))
        );
    }

    #[test]
    fn write_json_artifact_file_returns_text_presentation_when_requested() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-write-json-artifact-file-text-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        let out = root.join("triage.json");

        let presentation =
            write_json_artifact_file(&out, &serde_json::json!({ "kind": "triage" }), true)
                .expect("write json artifact file");

        assert!(matches!(presentation, ArtifactOutputPresentation::Text(_)));
        assert!(out.is_file());
        let _ = std::fs::remove_dir_all(&root);
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
    fn build_artifact_output_presentation_returns_meta_report_lines() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-artifact-output-presentation-meta-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        let meta_path = root.join("bundle.meta.json");
        std::fs::write(
            &meta_path,
            serde_json::to_vec(&serde_json::json!({
                "bundle": "bundle.json",
                "warmup_frames": 0,
                "windows_total": 0,
                "snapshots_total": 0,
                "snapshots_with_semantics_total": 0,
                "snapshots_with_inline_semantics_total": 0,
                "snapshots_with_table_semantics_total": 0,
                "semantics_table_entries_total": 0,
                "semantics_table_unique_keys_total": 0,
                "windows": [],
            }))
            .expect("serialize meta"),
        )
        .expect("write meta json");

        let presentation =
            build_artifact_output_presentation(&meta_path, ArtifactDisplayMode::MetaReport)
                .expect("build meta report presentation");

        match presentation {
            ArtifactOutputPresentation::Lines(lines) => {
                assert_eq!(lines.first().map(String::as_str), Some("bundle_meta:"));
            }
            other => panic!("expected lines presentation, got {other:?}"),
        }

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_lint_report_output_returns_path_and_exit_flag() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-lint-report-output-{}-{}",
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

        let output =
            build_lint_report_output(&out, &report, false).expect("build lint report output");

        assert_eq!(
            output.presentation,
            ArtifactOutputPresentation::Path(out.clone())
        );
        assert!(output.exit_required);
        assert!(out.is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_generated_artifact_output_reuses_existing_out_when_allowed() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-generated-artifact-output-reuse-existing-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let out = root.join("existing.json");
        std::fs::write(&out, b"{\"kind\":\"existing\"}").expect("write existing out");

        let output = build_generated_artifact_output(&out, ArtifactDisplayMode::Path, true, || {
            Err("should not resolve canonical".to_string())
        })
        .expect("build generated artifact output");

        assert_eq!(
            output.presentation,
            ArtifactOutputPresentation::Path(out.clone())
        );
        assert_eq!(
            std::fs::read_to_string(&out).expect("read existing out"),
            "{\"kind\":\"existing\"}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_generated_artifact_output_copies_missing_out() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-generated-artifact-output-copy-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let canonical = root.join("canonical.json");
        let out = root.join("nested").join("copy.json");
        std::fs::write(&canonical, b"{\"kind\":\"canonical\"}").expect("write canonical");

        let output =
            build_generated_artifact_output(&out, ArtifactDisplayMode::Path, false, || {
                Ok(canonical.clone())
            })
            .expect("build generated artifact output");

        assert_eq!(
            output.presentation,
            ArtifactOutputPresentation::Path(out.clone())
        );
        assert_eq!(
            std::fs::read_to_string(&out).expect("read copied out"),
            "{\"kind\":\"canonical\"}"
        );
        let _ = std::fs::remove_dir_all(&root);
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
    fn find_bundle_dir_meta_sidecar_path_prefers_direct_over_root() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-meta-bundle-dir-direct-over-root-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let bundle_dir = root.join("bundle-dir");
        let direct_meta = bundle_dir.join("bundle.meta.json");
        let root_meta = bundle_dir.join("_root").join("bundle.meta.json");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root_meta.parent().expect("root meta parent"))
            .expect("create root meta dir");
        std::fs::write(
            &direct_meta,
            serde_json::to_vec(&serde_json::json!({
                "kind": "bundle_meta",
                "schema_version": 1,
                "warmup_frames": 0,
            }))
            .unwrap(),
        )
        .expect("write direct meta sidecar");
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

        let path =
            find_bundle_dir_meta_sidecar_path(&bundle_dir, 0).expect("find preferred sidecar");

        assert_eq!(path, direct_meta);
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

    #[test]
    fn build_meta_execution_plan_keeps_prepared_fields() {
        let plan = build_meta_execution_plan(PreparedMetaCommand {
            canonical_path: PathBuf::from("captures/demo/bundle.meta.json"),
            out: PathBuf::from("exports/meta.json"),
            display_mode: ArtifactDisplayMode::MetaReport,
        });

        assert_eq!(
            plan.canonical_path,
            PathBuf::from("captures/demo/bundle.meta.json")
        );
        assert_eq!(plan.out, PathBuf::from("exports/meta.json"));
        assert!(matches!(plan.display_mode, ArtifactDisplayMode::MetaReport));
    }

    #[test]
    fn build_meta_command_output_uses_path_presentation() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-build-meta-command-output-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        let canonical = root.join("bundle.meta.json");
        let out = root.join("exports").join("meta.json");
        std::fs::write(
            &canonical,
            serde_json::to_vec(&serde_json::json!({
                "bundle": "bundle.json",
                "warmup_frames": 0,
                "windows_total": 0,
                "snapshots_total": 0,
                "snapshots_with_semantics_total": 0,
                "snapshots_with_inline_semantics_total": 0,
                "snapshots_with_table_semantics_total": 0,
                "semantics_table_entries_total": 0,
                "semantics_table_unique_keys_total": 0,
                "windows": [],
            }))
            .expect("serialize meta"),
        )
        .expect("write canonical meta");

        let output = build_meta_command_output(MetaExecutionPlan {
            canonical_path: canonical,
            out: out.clone(),
            display_mode: ArtifactDisplayMode::Path,
        })
        .expect("build meta command output");

        assert_eq!(
            output.presentation,
            ArtifactOutputPresentation::Path(out.clone())
        );
        assert!(out.is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn meta_report_summary_lines_include_semantics_summary() {
        let meta = serde_json::json!({
            "bundle": "bundle.json",
            "warmup_frames": 1,
            "windows_total": 2,
            "snapshots_total": 5,
            "snapshots_with_semantics_total": 4,
            "snapshots_with_inline_semantics_total": 1,
            "snapshots_with_table_semantics_total": 3,
            "semantics_table_entries_total": 8,
            "semantics_table_unique_keys_total": 6,
        });

        let lines = meta_report_summary_lines(&meta, Path::new("captures/bundle.meta.json"));

        assert_eq!(lines[0], "bundle_meta:");
        assert!(lines.iter().any(|line| line == "  bundle: bundle.json"));
        assert!(lines.iter().any(|line| {
            line == "  semantics: resolved=4 inline=1 table=3 table_entries=8 table_unique_keys=6"
        }));
    }

    #[test]
    fn meta_report_window_line_uses_null_when_considered_frame_missing() {
        let line = meta_report_window_line(&serde_json::json!({
            "window": 3,
            "snapshots_total": 9,
            "snapshots_with_semantics_total": 7,
            "snapshots_with_inline_semantics_total": 2,
            "snapshots_with_table_semantics_total": 5,
            "semantics_table_entries_total": 11,
            "semantics_table_unique_keys_total": 4,
        }));

        assert!(line.contains("window=3"));
        assert!(line.contains("considered_frame=null"));
    }

    #[test]
    fn append_meta_report_window_lines_truncates_after_six_entries() {
        let windows = (0..8)
            .map(|index| {
                serde_json::json!({
                    "window": index,
                    "snapshots_total": index + 1,
                    "considered_frame_id": index + 100,
                    "snapshots_with_semantics_total": index + 2,
                    "snapshots_with_inline_semantics_total": index + 3,
                    "snapshots_with_table_semantics_total": index + 4,
                    "semantics_table_entries_total": index + 5,
                    "semantics_table_unique_keys_total": index + 6,
                })
            })
            .collect::<Vec<_>>();
        let mut lines = Vec::new();

        append_meta_report_window_lines(&mut lines, &windows);

        assert_eq!(lines.first().map(String::as_str), Some("  windows:"));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("window=0 snapshots=1"))
        );
        assert!(lines.iter().any(|line| line == "    - ... (2 more)"));
    }

    #[test]
    fn build_triage_payload_mode_prefers_lite_metric_when_lite_requested() {
        let mode = build_triage_payload_mode(
            true,
            crate::frames_index::TriageLiteMetric::PaintTimeUs,
            Some(BundleStatsSort::Time),
        );

        match mode {
            TriagePayloadMode::Lite { metric } => {
                assert!(matches!(
                    metric,
                    crate::frames_index::TriageLiteMetric::PaintTimeUs
                ));
            }
            TriagePayloadMode::Full { .. } => panic!("expected lite mode"),
        }
    }

    #[test]
    fn build_triage_payload_mode_defaults_full_sort_to_invalidation() {
        let mode = build_triage_payload_mode(
            false,
            crate::frames_index::TriageLiteMetric::TotalTimeUs,
            None,
        );

        match mode {
            TriagePayloadMode::Lite { .. } => panic!("expected full mode"),
            TriagePayloadMode::Full { sort } => {
                assert_eq!(sort, BundleStatsSort::Invalidation);
            }
        }
    }

    #[test]
    fn build_triage_execution_plan_keeps_lite_metric_and_output_mode() {
        let plan = build_triage_execution_plan(
            PreparedTriageCommand {
                bundle_path: PathBuf::from("captures/demo/bundle.json"),
                out: PathBuf::from("captures/demo/triage.lite.json"),
                lite: true,
                metric: crate::frames_index::TriageLiteMetric::LayoutTimeUs,
            },
            TriageExecutionOptions {
                stats_top: 17,
                sort_override: Some(BundleStatsSort::Time),
                warmup_frames: 9,
                stats_json: true,
            },
        );

        assert_eq!(plan.out, PathBuf::from("captures/demo/triage.lite.json"));
        assert!(plan.stats_json);
        assert_eq!(
            plan.payload.bundle_path,
            PathBuf::from("captures/demo/bundle.json")
        );
        assert_eq!(plan.payload.stats_top, 17);
        assert_eq!(plan.payload.warmup_frames, 9);
        match plan.payload.mode {
            TriagePayloadMode::Lite { metric } => {
                assert!(matches!(
                    metric,
                    crate::frames_index::TriageLiteMetric::LayoutTimeUs
                ));
            }
            TriagePayloadMode::Full { .. } => panic!("expected lite mode"),
        }
    }

    #[test]
    fn build_triage_execution_plan_defaults_full_sort_to_invalidation() {
        let plan = build_triage_execution_plan(
            PreparedTriageCommand {
                bundle_path: PathBuf::from("captures/demo/bundle.json"),
                out: PathBuf::from("captures/demo/triage.json"),
                lite: false,
                metric: crate::frames_index::TriageLiteMetric::PaintTimeUs,
            },
            TriageExecutionOptions {
                stats_top: 8,
                sort_override: None,
                warmup_frames: 3,
                stats_json: false,
            },
        );

        match plan.payload.mode {
            TriagePayloadMode::Lite { .. } => panic!("expected full mode"),
            TriagePayloadMode::Full { sort } => {
                assert_eq!(sort, BundleStatsSort::Invalidation);
            }
        }
    }

    #[test]
    fn finalize_triage_payload_keeps_payload_without_bundle_dir() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-finalize-triage-payload-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        let bundle_path = root.join("bundle.json");
        let payload = serde_json::json!({ "kind": "triage" });

        let finalized = finalize_triage_payload(payload.clone(), &bundle_path);

        assert_eq!(finalized, payload);
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

#[derive(Debug, PartialEq, Eq)]
enum ArtifactOutputPresentation {
    Path(PathBuf),
    Text(String),
    Lines(Vec<String>),
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
            "missing bundle artifact path (try: fretboard-dev diag triage <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
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

fn build_triage_payload_mode(
    lite: bool,
    metric: crate::frames_index::TriageLiteMetric,
    sort_override: Option<BundleStatsSort>,
) -> TriagePayloadMode {
    if lite {
        TriagePayloadMode::Lite { metric }
    } else {
        TriagePayloadMode::Full {
            sort: sort_override.unwrap_or(BundleStatsSort::Invalidation),
        }
    }
}

fn build_triage_payload_with_plan(plan: TriagePayloadPlan) -> Result<serde_json::Value, String> {
    match plan.mode {
        TriagePayloadMode::Lite { metric } => build_triage_lite_payload(
            &plan.bundle_path,
            metric,
            plan.stats_top,
            plan.warmup_frames,
        ),
        TriagePayloadMode::Full { sort } => {
            build_triage_full_payload(&plan.bundle_path, plan.stats_top, sort, plan.warmup_frames)
        }
    }
}

fn finalize_triage_payload(
    mut payload: serde_json::Value,
    bundle_path: &Path,
) -> serde_json::Value {
    append_triage_tooling_warnings(&mut payload, bundle_path);
    payload
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
            "frames.index.json is missing or invalid (tip: fretboard-dev diag frames-index {} --warmup-frames {})",
            bundle_path.display(),
            warmup_frames
        )
    })?;

    Ok((index_path, frames_index))
}

fn build_triage_full_payload(
    bundle_path: &Path,
    stats_top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> Result<serde_json::Value, String> {
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

fn write_json_artifact_file(
    out: &Path,
    payload: &serde_json::Value,
    print_json: bool,
) -> Result<ArtifactOutputPresentation, String> {
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = json_artifact_payload_to_pretty_text(payload);
    std::fs::write(out, pretty.as_bytes()).map_err(|e| e.to_string())?;
    Ok(build_json_write_output_presentation(
        out, pretty, print_json,
    ))
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
            "missing bundle artifact path (try: fretboard-dev diag meta <bundle_dir|bundle.json|bundle.schema2.json>)"
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

fn valid_bundle_meta_sidecar_path(path: &Path, warmup_frames: u64) -> Option<PathBuf> {
    (path.is_file()
        && sidecars::try_read_sidecar_json_v1(path, "bundle_meta", warmup_frames).is_some())
    .then(|| path.to_path_buf())
}

fn build_meta_artifact_paths_for_existing_sidecar(path: PathBuf) -> MetaArtifactPaths {
    MetaArtifactPaths {
        canonical_path: path.clone(),
        default_out: path,
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
    if let Some(path) = valid_bundle_meta_sidecar_path(src, warmup_frames) {
        return Ok(build_meta_artifact_paths_for_existing_sidecar(path));
    }

    let Some(bundle_path) = sidecars::adjacent_bundle_path_for_sidecar(src) else {
        return Err(format!(
            "invalid bundle.meta.json (expected schema_version=1 warmup_frames={warmup_frames}) and no adjacent bundle artifact was found to regenerate it\n  meta: {}",
            src.display()
        ));
    };
    build_meta_artifact_paths_from_bundle_path(&bundle_path, warmup_frames)
}

fn find_bundle_dir_meta_sidecar_path(src: &Path, warmup_frames: u64) -> Option<PathBuf> {
    valid_bundle_meta_sidecar_path(&src.join("bundle.meta.json"), warmup_frames).or_else(|| {
        valid_bundle_meta_sidecar_path(&src.join("_root").join("bundle.meta.json"), warmup_frames)
    })
}

fn resolve_meta_artifact_paths_from_bundle_dir(
    src: &Path,
    warmup_frames: u64,
) -> Result<MetaArtifactPaths, String> {
    if let Some(path) = find_bundle_dir_meta_sidecar_path(src, warmup_frames) {
        return Ok(build_meta_artifact_paths_for_existing_sidecar(path));
    }

    build_meta_artifact_paths_from_bundle_path(
        &crate::resolve_bundle_artifact_path(src),
        warmup_frames,
    )
}

fn build_artifact_materialization_plan<F>(
    out: &Path,
    display_mode: ArtifactDisplayMode,
    reuse_existing_out: bool,
    resolve_canonical_path: F,
) -> Result<ArtifactMaterializationPlan, String>
where
    F: FnOnce() -> Result<PathBuf, String>,
{
    let mode = if reuse_existing_out && out.is_file() {
        ArtifactMaterializationMode::ReuseExistingOut
    } else {
        let canonical_path = resolve_canonical_path()?;
        if out == canonical_path {
            ArtifactMaterializationMode::ReuseCanonicalOut
        } else {
            ArtifactMaterializationMode::CopyCanonical { canonical_path }
        }
    };

    Ok(ArtifactMaterializationPlan {
        out: out.to_path_buf(),
        display_mode,
        mode,
    })
}

fn execute_artifact_materialization_plan(plan: &ArtifactMaterializationPlan) -> Result<(), String> {
    match &plan.mode {
        ArtifactMaterializationMode::ReuseExistingOut
        | ArtifactMaterializationMode::ReuseCanonicalOut => Ok(()),
        ArtifactMaterializationMode::CopyCanonical { canonical_path } => {
            if let Some(parent) = plan.out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::copy(canonical_path, &plan.out).map_err(|e| e.to_string())?;
            Ok(())
        }
    }
}

fn build_json_write_output_presentation(
    out: &Path,
    pretty: String,
    print_json: bool,
) -> ArtifactOutputPresentation {
    if print_json {
        ArtifactOutputPresentation::Text(pretty)
    } else {
        ArtifactOutputPresentation::Path(out.to_path_buf())
    }
}

fn build_artifact_output_presentation(
    out: &Path,
    display_mode: ArtifactDisplayMode,
) -> Result<ArtifactOutputPresentation, String> {
    match display_mode {
        ArtifactDisplayMode::Path => Ok(ArtifactOutputPresentation::Path(out.to_path_buf())),
        ArtifactDisplayMode::Json => Ok(ArtifactOutputPresentation::Text(
            read_artifact_output_text(out)?,
        )),
        ArtifactDisplayMode::MetaReport => {
            let meta = read_artifact_output_json(out)?;
            Ok(ArtifactOutputPresentation::Lines(meta_report_lines(
                &meta, out,
            )))
        }
    }
}

fn emit_artifact_output_presentation(presentation: ArtifactOutputPresentation) {
    match presentation {
        ArtifactOutputPresentation::Path(path) => println!("{}", path.display()),
        ArtifactOutputPresentation::Text(text) => println!("{text}"),
        ArtifactOutputPresentation::Lines(lines) => {
            for line in lines {
                println!("{line}");
            }
        }
    }
}

fn prepare_cmd_lint(request: LintCommandRequest<'_>) -> Result<PreparedLintCommand, String> {
    let PreparedBundleArtifactOutput { bundle_path, out } =
        prepare_required_bundle_artifact_output(RequiredBundleArtifactOutputRequest {
            rest: request.rest,
            workspace_root: request.workspace_root,
            missing_hint: "missing bundle artifact path (try: fretboard-dev diag lint <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)",
            custom_out: request.lint_out,
            default_out: crate::default_lint_out_path,
        })?;

    Ok(PreparedLintCommand { bundle_path, out })
}

fn prepare_cmd_triage(request: TriageCommandRequest<'_>) -> Result<PreparedTriageCommand, String> {
    let parsed = parse_triage_request(request.rest, request.workspace_root)?;
    let resolved = resolve::resolve_bundle_ref(&parsed.source)?;
    let bundle_path = resolved.bundle_artifact;
    let out = resolve_command_out_path(request.workspace_root, request.triage_out, || {
        default_triage_out_path(&bundle_path, parsed.lite)
    });

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
    let PreparedBundleArtifactOutput { bundle_path, out } =
        prepare_required_bundle_artifact_output(RequiredBundleArtifactOutputRequest {
            rest: request.rest,
            workspace_root: request.workspace_root,
            missing_hint: "missing bundle artifact path (try: fretboard-dev diag test-ids <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)",
            custom_out: request.test_ids_out,
            default_out: crate::default_test_ids_out_path,
        })?;

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
    let out = resolve_command_out_path(request.workspace_root, request.meta_out, || {
        paths.default_out.clone()
    });

    Ok(PreparedMetaCommand {
        canonical_path: paths.canonical_path,
        out,
        display_mode: parsed.display_mode,
    })
}

fn lint_exit_required(error_issues: u64) -> bool {
    error_issues > 0
}

fn build_lint_report_output(
    out: &Path,
    report: &LintReport,
    stats_json: bool,
) -> Result<LintReportOutput, String> {
    let presentation = write_json_artifact_file(out, &report.payload, stats_json)?;
    Ok(LintReportOutput {
        presentation,
        exit_required: lint_exit_required(report.error_issues),
    })
}

fn build_triage_execution_plan(
    prepared: PreparedTriageCommand,
    options: TriageExecutionOptions,
) -> TriageExecutionPlan {
    TriageExecutionPlan {
        payload: TriagePayloadPlan {
            bundle_path: prepared.bundle_path,
            stats_top: options.stats_top,
            warmup_frames: options.warmup_frames,
            mode: build_triage_payload_mode(prepared.lite, prepared.metric, options.sort_override),
        },
        out: prepared.out,
        stats_json: options.stats_json,
    }
}

fn build_triage_command_output(plan: TriageExecutionPlan) -> Result<TriageCommandOutput, String> {
    let bundle_path = plan.payload.bundle_path.clone();
    let payload = build_triage_payload_with_plan(plan.payload)?;
    let payload = finalize_triage_payload(payload, &bundle_path);
    let presentation = write_json_artifact_file(&plan.out, &payload, plan.stats_json)?;
    Ok(TriageCommandOutput { presentation })
}

fn build_meta_execution_plan(prepared: PreparedMetaCommand) -> MetaExecutionPlan {
    MetaExecutionPlan {
        canonical_path: prepared.canonical_path,
        out: prepared.out,
        display_mode: prepared.display_mode,
    }
}

fn build_test_ids_execution_plan(
    prepared: PreparedTestIdsCommand,
    warmup_frames: u64,
    max_test_ids: usize,
    stats_json: bool,
) -> TestIdsExecutionPlan {
    TestIdsExecutionPlan {
        bundle_path: prepared.bundle_path,
        out: prepared.out,
        warmup_frames,
        max_test_ids,
        display_mode: artifact_display_mode_for_stats_json(stats_json),
    }
}

fn build_test_ids_command_output(
    plan: TestIdsExecutionPlan,
) -> Result<GeneratedArtifactOutput, String> {
    let TestIdsExecutionPlan {
        bundle_path,
        out,
        warmup_frames,
        max_test_ids,
        display_mode,
    } = plan;
    build_generated_artifact_output(&out, display_mode, true, || {
        crate::bundle_index::ensure_test_ids_json(&bundle_path, warmup_frames, max_test_ids)
    })
}

fn build_meta_command_output(plan: MetaExecutionPlan) -> Result<GeneratedArtifactOutput, String> {
    let MetaExecutionPlan {
        canonical_path,
        out,
        display_mode,
    } = plan;
    build_generated_artifact_output(&out, display_mode, false, || Ok(canonical_path))
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

    let prepared = prepare_cmd_triage(TriageCommandRequest {
        rest,
        workspace_root,
        triage_out,
    })?;
    let plan = build_triage_execution_plan(
        prepared,
        TriageExecutionOptions {
            stats_top,
            sort_override,
            warmup_frames,
            stats_json,
        },
    );
    let output = build_triage_command_output(plan)?;
    emit_artifact_output_presentation(output.presentation);
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

    let output = build_lint_report_output(&out, &report, stats_json)?;
    emit_artifact_output_presentation(output.presentation);
    if output.exit_required {
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
    let prepared = prepare_cmd_test_ids(TestIdsCommandRequest {
        rest,
        workspace_root,
        test_ids_out,
    })?;
    let plan = build_test_ids_execution_plan(prepared, warmup_frames, max_test_ids, stats_json);
    let output = build_test_ids_command_output(plan)?;
    emit_artifact_output_presentation(output.presentation);
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
    let output = build_required_bundle_artifact_output(
        EnsuredBundleArtifactCommandRequest {
            rest,
            workspace_root,
            missing_hint: "missing bundle artifact path (try: fretboard-dev diag test-ids-index <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)",
            warmup_frames,
            stats_json,
        },
        crate::bundle_index::ensure_test_ids_index_json,
    )?;
    emit_artifact_output_presentation(output.presentation);
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
    let output = build_required_bundle_artifact_output(
        EnsuredBundleArtifactCommandRequest {
            rest,
            workspace_root,
            missing_hint: "missing bundle artifact path (try: fretboard-dev diag frames-index <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)",
            warmup_frames,
            stats_json,
        },
        crate::frames_index::ensure_frames_index_json,
    )?;
    emit_artifact_output_presentation(output.presentation);
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

    let prepared = prepare_cmd_meta(
        MetaCommandRequest {
            rest,
            workspace_root,
            meta_out,
            stats_json,
            meta_report,
        },
        warmup_frames,
    )?;
    let plan = build_meta_execution_plan(prepared);
    let output = build_meta_command_output(plan)?;
    emit_artifact_output_presentation(output.presentation);
    Ok(())
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

fn meta_report_summary_lines(meta: &serde_json::Value, meta_path: &Path) -> Vec<String> {
    vec![
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
    ]
}

fn meta_report_window_considered_frame_id(window: &serde_json::Value) -> String {
    window
        .get("considered_frame_id")
        .and_then(|value| value.as_u64())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn meta_report_window_line(window: &serde_json::Value) -> String {
    format!(
        "    - window={} snapshots={} considered_frame={} semantics(resolved/inline/table)={}/{}/{} table(entries/keys)={}/{}",
        meta_report_u64_field(window, "window"),
        meta_report_u64_field(window, "snapshots_total"),
        meta_report_window_considered_frame_id(window),
        meta_report_u64_field(window, "snapshots_with_semantics_total"),
        meta_report_u64_field(window, "snapshots_with_inline_semantics_total"),
        meta_report_u64_field(window, "snapshots_with_table_semantics_total"),
        meta_report_u64_field(window, "semantics_table_entries_total"),
        meta_report_u64_field(window, "semantics_table_unique_keys_total"),
    )
}

fn append_meta_report_window_lines(lines: &mut Vec<String>, windows: &[serde_json::Value]) {
    if windows.is_empty() {
        return;
    }

    lines.push("  windows:".to_string());
    let max = 6usize;
    for window in windows.iter().take(max) {
        lines.push(meta_report_window_line(window));
    }
    if windows.len() > max {
        lines.push(format!("    - ... ({} more)", windows.len() - max));
    }
}

fn meta_report_lines(meta: &serde_json::Value, meta_path: &Path) -> Vec<String> {
    let mut lines = meta_report_summary_lines(meta, meta_path);

    let Some(windows) = meta.get("windows").and_then(|value| value.as_array()) else {
        return lines;
    };
    append_meta_report_window_lines(&mut lines, windows);
    lines
}
