use std::path::{Path, PathBuf};

use clap::error::ErrorKind;

use super::{
    DiagPathOverrides, ResolveDiagCliPathsRequest, ResolvedDiagCliPaths, contracts,
    resolve_diag_cli_paths, workspace_root,
};

enum MigratedDiagCommand {
    Agent(AgentCmdContext),
    AiPacket(AiPacketCmdContext),
    Artifact(ArtifactCmdContext),
    BundleV2(BundleV2CmdContext),
    Campaign(crate::diag_campaign::CampaignCmdContext),
    Compare(crate::diag_compare::CompareCmdContext),
    Config(crate::commands::config::ConfigCmdContext),
    Dashboard(crate::diag_dashboard::DashboardCmdContext),
    DockGraph(DockGraphCmdContext),
    DockRouting(DockRoutingCmdContext),
    Doctor(DoctorCmdContext),
    Extensions(ExtensionsCmdContext),
    FramesIndex(FramesIndexCmdContext),
    Hotspots(HotspotsCmdContext),
    Index(IndexCmdContext),
    Inspect(InspectCmdContext),
    Latest(LatestCmdContext),
    LayoutSidecar(LayoutSidecarCmdContext),
    LayoutPerfSummary(LayoutPerfSummaryCmdContext),
    Lint(LintCmdContext),
    List(ListCmdContext),
    MemorySummary(MemorySummaryCmdContext),
    Meta(MetaCmdContext),
    Matrix(crate::diag_matrix::MatrixCmdContext),
    Pack(PackCmdContext),
    Path(PathCmdContext),
    Perf(crate::diag_perf::PerfCmdContext),
    PerfBaselineFromBundles(crate::diag_perf_baseline::PerfBaselineFromBundlesContext),
    Poke(PokeCmdContext),
    Pick(PickCmdContext),
    PickApply(PickApplyCmdContext),
    PickArm(PickArmCmdContext),
    PickScript(PickScriptCmdContext),
    Query(QueryCmdContext),
    Registry(RegistryCmdContext),
    Resolve(ResolveCmdContext),
    Repro(crate::diag_repro::ReproCmdContext),
    Run(crate::diag_run::RunCmdContext),
    Repeat(crate::diag_repeat::RepeatCmdContext),
    Screenshots(ScreenshotsCmdContext),
    Script(ScriptCmdContext),
    Sessions(SessionsCmdContext),
    Slice(SliceCmdContext),
    Stats(crate::diag_stats::StatsCmdContext),
    Summarize(crate::diag_summarize::SummarizeCmdContext),
    Suite(crate::diag_suite::SuiteCmdContext),
    TestIds(TestIdsCmdContext),
    TestIdsIndex(TestIdsIndexCmdContext),
    Trace(TraceCmdContext),
    Triage(TriageCmdContext),
    Windows(WindowsCmdContext),
}

struct AgentCmdContext {
    bundle_source: Option<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    warmup_frames: u64,
    stats_json: bool,
    out: Option<PathBuf>,
}

struct AiPacketCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    packet_out: Option<PathBuf>,
    include_triage: bool,
    stats_top: usize,
    sort_override: Option<crate::BundleStatsSort>,
    warmup_frames: u64,
}

struct ListCmdContext {
    rest: Vec<String>,
    resolved_out_dir: PathBuf,
    stats_json: bool,
    stats_top_override: Option<usize>,
}

struct SessionsCmdContext {
    rest: Vec<String>,
    resolved_out_dir: PathBuf,
    stats_json: bool,
    top_override: Option<usize>,
}

struct MemorySummaryCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    top_rows: usize,
    stats_json: bool,
    out: Option<PathBuf>,
}

struct DoctorCmdContext {
    rest: Vec<String>,
    resolved_out_dir: PathBuf,
    warmup_frames: u64,
    stats_json: bool,
}

struct ArtifactCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    resolved_script_result_path: PathBuf,
    artifact_lint_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
}

struct BundleV2CmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    bundle_v2_out: Option<PathBuf>,
    stats_json: bool,
}

struct DockGraphCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    stats_json: bool,
}

struct DockRoutingCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    warmup_frames: u64,
    stats_json: bool,
}

struct ExtensionsCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    warmup_frames: u64,
    stats_json: bool,
    out: Option<PathBuf>,
}

struct FramesIndexCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    warmup_frames: u64,
    stats_json: bool,
}

struct HotspotsCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    hotspots_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
}

struct IndexCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    index_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
}

struct InspectCmdContext {
    action: String,
    resolved_inspect_path: PathBuf,
    resolved_inspect_trigger_path: PathBuf,
    inspect_consume_clicks: Option<bool>,
}

struct LatestCmdContext {
    resolved_out_dir: PathBuf,
}

struct LayoutSidecarCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    stats_json: bool,
    out: Option<PathBuf>,
}

struct LayoutPerfSummaryCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    warmup_frames: u64,
    stats_json: bool,
    out: Option<PathBuf>,
}

struct LintCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    lint_out: Option<PathBuf>,
    lint_all_test_ids_bounds: bool,
    lint_eps_px: f32,
    warmup_frames: u64,
    stats_json: bool,
}

struct ScriptCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    resolved_run_context: crate::ResolvedRunContext,
    script_tool_check: bool,
    script_tool_write: bool,
    script_tool_check_out: Option<PathBuf>,
    shrink_out: Option<PathBuf>,
    shrink_any_fail: bool,
    shrink_match_reason_code: Option<String>,
    shrink_match_reason: Option<String>,
    shrink_min_steps: u64,
    shrink_max_iters: u64,
    launch: Option<Vec<String>>,
    launch_env: Vec<(String, String)>,
    timeout_ms: u64,
    poll_ms: u64,
    stats_json: bool,
}

struct MetaCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    meta_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
    meta_report: bool,
}

struct PackCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    pack_out: Option<PathBuf>,
    ensure_ai_packet: bool,
    pack_ai_only: bool,
    pack_include_root_artifacts: bool,
    pack_include_triage: bool,
    pack_include_screenshots: bool,
    pack_schema2_only: bool,
    stats_top: usize,
    sort_override: Option<crate::BundleStatsSort>,
    warmup_frames: u64,
}

struct PathCmdContext {
    resolved_trigger_path: PathBuf,
}

struct PokeCmdContext {
    rest: Vec<String>,
    resolved_out_dir: PathBuf,
    resolved_trigger_path: PathBuf,
    timeout_ms: u64,
    poll_ms: u64,
}

struct PickArmCmdContext {
    resolved_pick_trigger_path: PathBuf,
}

struct PickCmdContext {
    resolved_pick_trigger_path: PathBuf,
    resolved_pick_result_path: PathBuf,
    resolved_pick_result_trigger_path: PathBuf,
    timeout_ms: u64,
    poll_ms: u64,
}

struct PickScriptCmdContext {
    resolved_pick_trigger_path: PathBuf,
    resolved_pick_result_path: PathBuf,
    resolved_pick_result_trigger_path: PathBuf,
    resolved_pick_script_out: PathBuf,
    timeout_ms: u64,
    poll_ms: u64,
}

struct PickApplyCmdContext {
    script: String,
    workspace_root: PathBuf,
    resolved_pick_trigger_path: PathBuf,
    resolved_pick_result_path: PathBuf,
    resolved_pick_result_trigger_path: PathBuf,
    pick_apply_pointer: String,
    pick_apply_out: Option<PathBuf>,
    timeout_ms: u64,
    poll_ms: u64,
}

struct QueryCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    query_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
}

struct ResolveCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    json: bool,
}

struct RegistryCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    stats_json: bool,
}

struct ScreenshotsCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    stats_json: bool,
}

struct SliceCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    slice_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
}

struct TestIdsCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    test_ids_out: Option<PathBuf>,
    warmup_frames: u64,
    max_test_ids: usize,
    stats_json: bool,
}

struct TestIdsIndexCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    warmup_frames: u64,
    stats_json: bool,
}

struct TraceCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    trace_out: Option<PathBuf>,
}

struct TriageCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    triage_out: Option<PathBuf>,
    stats_top: usize,
    sort_override: Option<crate::BundleStatsSort>,
    warmup_frames: u64,
    stats_json: bool,
}

struct WindowsCmdContext {
    rest: Vec<String>,
    workspace_root: PathBuf,
    warmup_frames: u64,
    stats_json: bool,
}

fn clap_error_to_string(err: clap::Error) -> String {
    let rendered = err.to_string();
    rendered
        .strip_prefix("error: ")
        .unwrap_or(rendered.as_str())
        .to_string()
}

fn parse_env_assignments(values: &[String]) -> Result<Vec<(String, String)>, String> {
    values
        .iter()
        .map(|value| {
            let (key, env_value) = value
                .split_once('=')
                .ok_or_else(|| "invalid value for --env (expected KEY=VALUE)".to_string())?;
            let key = key.trim();
            if key.is_empty() {
                return Err("invalid value for --env (empty KEY)".to_string());
            }
            Ok((key.to_string(), env_value.to_string()))
        })
        .collect()
}

fn uses_devtools_transport(args: &contracts::shared::DevtoolsArgs) -> bool {
    args.devtools_ws_url.is_some()
        || args.devtools_token.is_some()
        || args.devtools_session_id.is_some()
}

fn parse_memory_p90_thresholds(values: &[String]) -> Result<Vec<(String, u64)>, String> {
    values
        .iter()
        .map(|value| {
            let value = value.trim();
            let (key, bytes) = value
                .split_once(':')
                .or_else(|| value.split_once('='))
                .ok_or_else(|| {
                    "invalid value for --check-memory-p90-max: expected \"<key>:<bytes>\""
                        .to_string()
                })?;
            let key = key.trim();
            let bytes = bytes.trim();
            if key.is_empty() || bytes.is_empty() {
                return Err(
                    "invalid value for --check-memory-p90-max: expected \"<key>:<bytes>\""
                        .to_string(),
                );
            }
            let bytes = bytes.parse::<u64>().map_err(|_| {
                "invalid value for --check-memory-p90-max: invalid bytes (expected u64)".to_string()
            })?;
            Ok((key.to_string(), bytes))
        })
        .collect()
}

fn push_env_if_missing(env: &mut Vec<(String, String)>, key: &str, value: &str) {
    if env.iter().any(|(existing, _)| existing == key) {
        return;
    }
    env.push((key.to_string(), value.to_string()));
}

fn apply_contract_checks_to_run_checks(
    args: &contracts::shared::ChecksArgs,
) -> crate::diag_run::RunChecks {
    let mut checks = crate::diag_run::RunChecks::default();
    checks.check_stale_paint_test_id = args.check_stale_paint.clone();
    checks.check_stale_paint_eps = args.check_stale_paint_eps;
    checks.check_stale_scene_test_id = args.check_stale_scene.clone();
    checks.check_stale_scene_eps = args.check_stale_scene_eps;
    checks.check_idle_no_paint_min = args.check_idle_no_paint_min;
    checks.check_pixels_changed_test_id = args.check_pixels_changed.clone();
    checks.check_pixels_unchanged_test_id = args.check_pixels_unchanged.clone();
    checks
}

#[derive(Debug, Clone, Copy)]
struct PackPolicy {
    pack_after_run: bool,
    ensure_ai_packet: bool,
    pack_include_root_artifacts: bool,
    pack_include_triage: bool,
    pack_include_screenshots: bool,
    pack_schema2_only: bool,
}

fn pack_policy(args: &contracts::shared::PackArgs) -> PackPolicy {
    PackPolicy {
        pack_after_run: args.pack,
        ensure_ai_packet: args.ai_packet || args.ai_only,
        pack_include_root_artifacts: args.include_root_artifacts || args.include_all,
        pack_include_triage: args.include_triage || args.include_all,
        pack_include_screenshots: args.include_screenshots || args.include_all,
        pack_schema2_only: args.pack_schema2_only,
    }
}

fn ordered_suite_script_inputs(raw_args: &[String]) -> Vec<String> {
    let mut inputs = Vec::new();
    let mut i = 0usize;
    while i < raw_args.len() {
        match raw_args[i].as_str() {
            "--script-dir" | "--glob" => {
                if let Some(value) = raw_args.get(i + 1) {
                    inputs.push(value.clone());
                    i += 2;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    inputs
}

fn resolve_diag_out_dir(
    workspace_root: &Path,
    sub: &str,
    launch: Option<&[String]>,
    out_dir: Option<PathBuf>,
) -> Result<PathBuf, String> {
    let ResolvedDiagCliPaths {
        resolved_out_dir, ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub,
        launch,
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides {
            out_dir,
            ..DiagPathOverrides::default()
        },
    })?;
    Ok(resolved_out_dir)
}

fn looks_like_bundle_source(raw: &str, workspace_root: &Path) -> bool {
    raw.contains('/')
        || raw.contains('\\')
        || raw.ends_with(".json")
        || crate::resolve_path(workspace_root, PathBuf::from(raw)).exists()
}

fn retired_diag_alias_error(args: &[String]) -> Option<String> {
    let sub = args.first()?.as_str();
    match sub {
        "artifacts" => Some("`diag artifacts` was removed; use `diag artifact`.".to_string()),
        "layout_sidecar" => {
            Some("`diag layout_sidecar` was removed; use `diag layout-sidecar`.".to_string())
        }
        "layout_perf_summary" => Some(
            "`diag layout_perf_summary` was removed; use `diag layout-perf-summary`."
                .to_string(),
        ),
        "memory_summary" => Some(
            "`diag memory_summary` was removed; use `diag memory-summary`.".to_string(),
        ),
        "pack" if args.iter().skip(1).any(|arg| arg == "--schema2-only") => Some(
            "`diag pack --schema2-only` was removed; use `diag pack --pack-schema2-only`."
                .to_string(),
        ),
        "query" => match args.get(1).map(String::as_str) {
            Some("test_ids") => {
                Some("`diag query test_ids` was removed; use `diag query test-id`.".to_string())
            }
            Some("snapshot") => {
                Some("`diag query snapshot` was removed; use `diag query snapshots`.".to_string())
            }
            Some("overlay_placement_trace") => Some(
                "`diag query overlay_placement_trace` was removed; use `diag query overlay-placement-trace`."
                    .to_string(),
            ),
            Some("overlay-placement") => Some(
                "`diag query overlay-placement` was removed; use `diag query overlay-placement-trace`."
                    .to_string(),
            ),
            Some("overlay_placement") => Some(
                "`diag query overlay_placement` was removed; use `diag query overlay-placement-trace`."
                    .to_string(),
            ),
            Some("scroll_extents_observation") => Some(
                "`diag query scroll_extents_observation` was removed; use `diag query scroll-extents-observation`."
                    .to_string(),
            ),
            Some("scroll-observation") => Some(
                "`diag query scroll-observation` was removed; use `diag query scroll-extents-observation`."
                    .to_string(),
            ),
            Some("scroll_observation") => Some(
                "`diag query scroll_observation` was removed; use `diag query scroll-extents-observation`."
                    .to_string(),
            ),
            _ => None,
        },
        "triage" if args.iter().skip(1).any(|arg| arg == "--frames-index") => Some(
            "`diag triage --frames-index` was removed; use `diag triage --lite`.".to_string(),
        ),
        "triage" if args.iter().skip(1).any(|arg| arg == "--from-frames-index") => Some(
            "`diag triage --from-frames-index` was removed; use `diag triage --lite`."
                .to_string(),
        ),
        _ => None,
    }
}

fn default_campaign_context(
    rest: Vec<String>,
    workspace_root: &Path,
    resolved_out_dir: PathBuf,
    stats_json: bool,
    stats_top: usize,
    warmup_frames: u64,
) -> crate::diag_campaign::CampaignCmdContext {
    crate::diag_campaign::CampaignCmdContext {
        pack_after_run: false,
        rest,
        suite_script_inputs: Vec::new(),
        suite_prewarm_scripts: Vec::new(),
        suite_prelude_scripts: Vec::new(),
        suite_prelude_each_run: false,
        workspace_root: workspace_root.to_path_buf(),
        resolved_out_dir,
        devtools_ws_url: None,
        devtools_token: None,
        devtools_session_id: None,
        timeout_ms: 240_000,
        poll_ms: 50,
        stats_top,
        stats_json,
        warmup_frames,
        max_test_ids: 200,
        lint_all_test_ids_bounds: false,
        lint_eps_px: 0.5,
        suite_lint: true,
        pack_include_screenshots: false,
        reuse_launch: false,
        launch: None,
        launch_env: Vec::new(),
        launch_high_priority: false,
        launch_write_bundle_json: false,
        keep_open: false,
        checks: crate::diag_suite::SuiteChecks::default(),
    }
}

fn campaign_filter_rest(
    filters: &contracts::commands::campaign::CampaignFilterArgs,
) -> Vec<String> {
    let mut rest = Vec::new();
    filters.append_rest(&mut rest);
    rest
}

fn list_filter_rest(filters: &contracts::commands::list::ListFilterArgs) -> Vec<String> {
    let mut rest = Vec::new();
    filters.append_rest(&mut rest);
    rest
}

fn parse_sessions_command(
    args: contracts::commands::sessions::SessionsCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::sessions::SessionsSubcommandArgs;

    match args.command {
        SessionsSubcommandArgs::Clean(clean) => {
            if matches!(clean.top, Some(0)) {
                return Err("--top must be >= 1".to_string());
            }
            let resolved_out_dir =
                resolve_diag_out_dir(workspace_root, "sessions", None, clean.dir.clone())?;
            let mut rest = vec!["clean".to_string()];
            if let Some(keep) = clean.keep {
                rest.push("--keep".to_string());
                rest.push(keep.to_string());
            }
            if let Some(days) = clean.older_than_days {
                rest.push("--older-than-days".to_string());
                rest.push(days.to_string());
            }
            if clean.apply {
                rest.push("--apply".to_string());
            }

            Ok(MigratedDiagCommand::Sessions(SessionsCmdContext {
                rest,
                resolved_out_dir,
                stats_json: clean.json,
                top_override: clean.top,
            }))
        }
    }
}

fn registry_action_rest(
    action: &str,
    args: &contracts::commands::registry::RegistryActionArgs,
) -> Vec<String> {
    let mut rest = vec![action.to_string()];
    if let Some(path) = args.path.as_ref() {
        rest.push("--path".to_string());
        rest.push(path.display().to_string());
    }
    rest
}

fn parse_registry_command(
    args: contracts::commands::registry::RegistryCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::registry::RegistrySubcommandArgs;

    let (rest, stats_json) = match args.command {
        RegistrySubcommandArgs::Check(check) => (registry_action_rest("check", &check), check.json),
        RegistrySubcommandArgs::Write(write) => (registry_action_rest("write", &write), write.json),
        RegistrySubcommandArgs::Print(print) => (registry_action_rest("print", &print), print.json),
    };

    Ok(MigratedDiagCommand::Registry(RegistryCmdContext {
        rest,
        workspace_root: workspace_root.to_path_buf(),
        stats_json,
    }))
}

fn parse_config_command(
    args: contracts::commands::config::ConfigCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::config::ConfigSubcommandArgs;

    match args.command {
        ConfigSubcommandArgs::Doctor(doctor) => {
            let launch_env = parse_env_assignments(&doctor.env)?;
            let ResolvedDiagCliPaths {
                resolved_out_dir,
                resolved_run_context,
                ..
            } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
                workspace_root,
                sub: "config",
                launch: None,
                session_auto: false,
                session_id: None,
                overrides: DiagPathOverrides {
                    out_dir: doctor.dir.clone(),
                    ..DiagPathOverrides::default()
                },
            })?;

            let mut rest = vec!["doctor".to_string()];
            if let Some(mode) = doctor.mode {
                rest.push("--mode".to_string());
                rest.push(mode.as_str().to_string());
            }
            if let Some(config_path) = doctor.config_path {
                rest.push("--config-path".to_string());
                rest.push(config_path.display().to_string());
            }
            if let Some(show_env) = doctor.show_env {
                rest.push("--show-env".to_string());
                rest.push(show_env.as_str().to_string());
            }
            if doctor.report_json {
                rest.push("--report-json".to_string());
            }
            if doctor.print_launch_policy {
                rest.push("--print-launch-policy".to_string());
            }

            Ok(MigratedDiagCommand::Config(
                crate::commands::config::ConfigCmdContext {
                    rest,
                    workspace_root: workspace_root.to_path_buf(),
                    resolved_out_dir,
                    resolved_ready_path: resolved_run_context.paths.ready_path.clone(),
                    resolved_exit_path: resolved_run_context.paths.exit_path.clone(),
                    fs_transport_cfg: resolved_run_context.fs_transport_cfg.clone(),
                    launch_env,
                },
            ))
        }
    }
}

fn parse_list_command(
    args: contracts::commands::list::ListCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::list::ListSubcommandArgs;

    match args.command {
        ListSubcommandArgs::Scripts(scripts) => {
            let resolved_out_dir = resolve_diag_out_dir(workspace_root, "list", None, None)?;
            let mut rest = vec!["scripts".to_string()];
            rest.extend(list_filter_rest(&scripts.filters));
            Ok(MigratedDiagCommand::List(ListCmdContext {
                rest,
                resolved_out_dir,
                stats_json: scripts.json,
                stats_top_override: scripts.top,
            }))
        }
        ListSubcommandArgs::Suites(suites) => {
            let resolved_out_dir = resolve_diag_out_dir(workspace_root, "list", None, None)?;
            let mut rest = vec!["suites".to_string()];
            rest.extend(list_filter_rest(&suites.filters));
            Ok(MigratedDiagCommand::List(ListCmdContext {
                rest,
                resolved_out_dir,
                stats_json: suites.json,
                stats_top_override: suites.top,
            }))
        }
        ListSubcommandArgs::Sessions(sessions) => {
            let resolved_out_dir =
                resolve_diag_out_dir(workspace_root, "list", None, sessions.dir.clone())?;
            let mut rest = vec!["sessions".to_string()];
            rest.extend(list_filter_rest(&sessions.filters));
            Ok(MigratedDiagCommand::List(ListCmdContext {
                rest,
                resolved_out_dir,
                stats_json: sessions.json,
                stats_top_override: sessions.top,
            }))
        }
    }
}

fn parse_doctor_command(
    args: contracts::commands::doctor::DoctorCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::doctor::DoctorSubcommandArgs;

    match args.command {
        Some(DoctorSubcommandArgs::Scripts(scripts)) => {
            let resolved_out_dir = resolve_diag_out_dir(workspace_root, "doctor", None, None)?;
            let mut rest = vec!["scripts".to_string()];
            if scripts.max_examples != 20 {
                rest.push("--max-examples".to_string());
                rest.push(scripts.max_examples.to_string());
            }
            if scripts.strict {
                rest.push("--strict".to_string());
            }
            Ok(MigratedDiagCommand::Doctor(DoctorCmdContext {
                rest,
                resolved_out_dir,
                warmup_frames: 0,
                stats_json: scripts.json,
            }))
        }
        Some(DoctorSubcommandArgs::Campaigns(campaigns)) => {
            let resolved_out_dir = resolve_diag_out_dir(workspace_root, "doctor", None, None)?;
            let mut rest = vec!["campaigns".to_string()];
            if campaigns.strict {
                rest.push("--strict".to_string());
            }
            Ok(MigratedDiagCommand::Doctor(DoctorCmdContext {
                rest,
                resolved_out_dir,
                warmup_frames: 0,
                stats_json: campaigns.json,
            }))
        }
        None => {
            let resolved_out_dir =
                resolve_diag_out_dir(workspace_root, "doctor", None, args.dir.clone())?;
            let mut rest = Vec::new();
            if let Some(source) = args.source {
                rest.push(source);
            }
            if args.fix {
                rest.push("--fix".to_string());
            }
            if args.fix_dry_run {
                rest.push("--fix-dry-run".to_string());
            }
            if args.fix_schema2 {
                rest.push("--fix-schema2".to_string());
            }
            if args.fix_bundle_json {
                rest.push("--fix-bundle-json".to_string());
            }
            if args.fix_sidecars {
                rest.push("--fix-sidecars".to_string());
            }
            if args.check_required {
                rest.push("--check".to_string());
            }
            if args.check_all {
                rest.push("--check-all".to_string());
            }
            Ok(MigratedDiagCommand::Doctor(DoctorCmdContext {
                rest,
                resolved_out_dir,
                warmup_frames: args.warmup_frames,
                stats_json: args.json,
            }))
        }
    }
}

fn parse_artifact_command(
    args: contracts::commands::artifact::ArtifactCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::artifact::ArtifactSubcommandArgs;

    match args.command {
        ArtifactSubcommandArgs::Lint(lint) => {
            let ResolvedDiagCliPaths {
                resolved_out_dir,
                resolved_run_context,
                ..
            } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
                workspace_root,
                sub: "artifact",
                launch: None,
                session_auto: false,
                session_id: None,
                overrides: DiagPathOverrides::default(),
            })?;

            let mut rest = vec!["lint".to_string()];
            if let Some(source) = lint.source {
                rest.push(source);
            }

            Ok(MigratedDiagCommand::Artifact(ArtifactCmdContext {
                rest,
                workspace_root: workspace_root.to_path_buf(),
                resolved_out_dir,
                resolved_script_result_path: resolved_run_context.paths.script_result_path,
                artifact_lint_out: lint.output.out,
                warmup_frames: lint.warmup.warmup_frames,
                stats_json: lint.output.json,
            }))
        }
    }
}

fn parse_bundle_v2_command(
    args: contracts::commands::bundle_v2::BundleV2CommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "bundle-v2", None, None)?;
    let mut rest = Vec::new();
    if let Some(source) = args.source {
        rest.push(source);
    }
    if args.mode != "last" {
        rest.push("--mode".to_string());
        rest.push(args.mode);
    }
    if args.pretty {
        rest.push("--pretty".to_string());
    }
    if args.force {
        rest.push("--force".to_string());
    }

    Ok(MigratedDiagCommand::BundleV2(BundleV2CmdContext {
        rest,
        workspace_root: workspace_root.to_path_buf(),
        resolved_out_dir,
        bundle_v2_out: args.output.out,
        stats_json: args.output.json,
    }))
}

fn parse_compare_command(
    args: contracts::commands::compare::CompareCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::Compare(
        crate::diag_compare::CompareCmdContext {
            rest: vec![args.source_a, args.source_b],
            workspace_root: workspace_root.to_path_buf(),
            warmup_frames: args.warmup.warmup_frames,
            compare_eps_px: args.compare.compare_eps_px,
            compare_ignore_bounds: args.compare.compare_ignore_bounds,
            compare_ignore_scene_fingerprint: args.compare.compare_ignore_scene_fingerprint,
            compare_footprint: args.footprint,
            stats_json: args.json,
        },
    ))
}

fn parse_dashboard_command(
    args: contracts::commands::dashboard::DashboardCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir =
        resolve_diag_out_dir(workspace_root, "dashboard", None, args.output.dir)?;
    Ok(MigratedDiagCommand::Dashboard(
        crate::diag_dashboard::DashboardCmdContext {
            source: args
                .source
                .map(|path| crate::resolve_path(workspace_root, path)),
            resolved_out_dir,
            top: args.top.max(1),
            stats_json: args.output.json,
        },
    ))
}

fn parse_perf_baseline_from_bundles_command(
    args: contracts::commands::perf_baseline_from_bundles::PerfBaselineFromBundlesCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let mut rest = vec![args.script];
    rest.extend(args.bundle_artifacts);

    Ok(MigratedDiagCommand::PerfBaselineFromBundles(
        crate::diag_perf_baseline::PerfBaselineFromBundlesContext {
            pack_after_run: false,
            rest,
            workspace_root: workspace_root.to_path_buf(),
            sort_override: args.sort,
            perf_baseline_out: Some(args.perf_baseline_out),
            perf_baseline_headroom_pct: args.perf_baseline_headroom_pct,
            warmup_frames: args.warmup.warmup_frames,
            stats_json: args.json,
        },
    ))
}

fn parse_stats_notify_hotspot_file_max(values: &[String]) -> Result<Vec<(String, u64)>, String> {
    let mut pairs = Vec::new();
    let mut chunks = values.chunks_exact(2);
    for chunk in &mut chunks {
        let max = chunk[1]
            .parse::<u64>()
            .map_err(|_| "invalid value for --check-notify-hotspot-file-max (max)".to_string())?;
        pairs.push((chunk[0].clone(), max));
    }
    debug_assert!(chunks.remainder().is_empty());
    Ok(pairs)
}

fn parse_stats_command(
    args: contracts::commands::stats::StatsCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let stats_diff = match args.diff {
        Some(values) => {
            let [left, right] = values.try_into().map_err(|_| {
                "invalid value for --diff (expected exactly two bundle artifact paths)".to_string()
            })?;
            Some((
                crate::resolve_path(workspace_root, left),
                crate::resolve_path(workspace_root, right),
            ))
        }
        None => None,
    };
    let checks = args.checks;

    Ok(MigratedDiagCommand::Stats(
        crate::diag_stats::StatsCmdContext {
            rest: args.source.into_iter().collect(),
            stats_diff,
            workspace_root: workspace_root.to_path_buf(),
            sort_override: args.sort,
            stats_top: args.top,
            stats_json: args.json,
            stats_lite_checks_json: args.stats_lite_checks_json,
            stats_verbose: args.verbose,
            warmup_frames: args.warmup.warmup_frames,
            check_stale_paint_test_id: checks.common.check_stale_paint,
            check_stale_paint_eps: checks.common.check_stale_paint_eps,
            check_stale_scene_test_id: checks.common.check_stale_scene,
            check_stale_scene_eps: checks.common.check_stale_scene_eps,
            check_idle_no_paint_min: checks.common.check_idle_no_paint_min,
            check_asset_load_missing_bundle_assets_max: checks
                .check_asset_load_missing_bundle_assets_max,
            check_asset_load_stale_manifest_max: checks.check_asset_load_stale_manifest_max,
            check_asset_load_unsupported_file_max: checks.check_asset_load_unsupported_file_max,
            check_asset_load_unsupported_url_max: checks.check_asset_load_unsupported_url_max,
            check_asset_load_external_reference_unavailable_max: checks
                .check_asset_load_external_reference_unavailable_max,
            check_asset_load_revision_changes_max: checks.check_asset_load_revision_changes_max,
            check_bundled_font_baseline_source: checks.check_bundled_font_baseline_source,
            check_asset_reload_epoch_min: checks.check_asset_reload_epoch_min,
            check_asset_reload_configured_backend: checks.check_asset_reload_configured_backend,
            check_asset_reload_active_backend: checks.check_asset_reload_active_backend,
            check_asset_reload_fallback_reason: checks.check_asset_reload_fallback_reason,
            check_pixels_changed_test_id: checks.common.check_pixels_changed,
            check_pixels_unchanged_test_id: checks.common.check_pixels_unchanged,
            check_semantics_changed_repainted: checks.check_semantics_changed_repainted,
            dump_semantics_changed_repainted_json: checks.dump_semantics_changed_repainted_json,
            check_wheel_scroll_test_id: checks.check_wheel_scroll,
            check_wheel_scroll_hit_changes_test_id: checks.check_wheel_scroll_hit_changes,
            check_drag_cache_root_paint_only_test_id: checks.check_drag_cache_root_paint_only,
            check_hover_layout_max: if checks.check_hover_layout {
                Some(0)
            } else {
                checks.check_hover_layout_max
            },
            check_gc_sweep_liveness: checks.check_gc_sweep_liveness,
            check_notify_hotspot_file_max: parse_stats_notify_hotspot_file_max(
                &checks.check_notify_hotspot_file_max,
            )?,
            check_view_cache_reuse_stable_min: checks.check_view_cache_reuse_stable_min,
            check_view_cache_reuse_min: checks.check_view_cache_reuse_min,
            check_overlay_synthesis_min: checks.check_overlay_synthesis_min,
            check_viewport_input_min: checks.check_viewport_input_min,
            check_dock_drag_min: checks.check_dock_drag_min,
            check_viewport_capture_min: checks.check_viewport_capture_min,
            check_retained_vlist_reconcile_no_notify_min: checks
                .check_retained_vlist_reconcile_no_notify_min,
            check_retained_vlist_attach_detach_max: checks.check_retained_vlist_attach_detach_max,
            check_retained_vlist_keep_alive_reuse_min: checks
                .check_retained_vlist_keep_alive_reuse_min,
        },
    ))
}

fn parse_summarize_command(
    args: contracts::commands::summarize::SummarizeCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir =
        resolve_diag_out_dir(workspace_root, "summarize", None, args.output.dir)?;
    Ok(MigratedDiagCommand::Summarize(
        crate::diag_summarize::SummarizeCmdContext {
            inputs: args
                .inputs
                .into_iter()
                .map(|path| crate::resolve_path(workspace_root, path))
                .collect(),
            workspace_root: workspace_root.to_path_buf(),
            resolved_out_dir,
            stats_json: args.output.json,
        },
    ))
}

fn parse_ai_packet_command(
    args: contracts::commands::ai_packet::AiPacketCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "ai-packet", None, None)?;
    let mut rest = Vec::new();
    if let Some(source) = args.source {
        if !looks_like_bundle_source(&source, workspace_root) {
            return Err(
                "`diag ai-packet <test-id>` was removed; use `diag ai-packet --test-id <test-id>`."
                    .to_string(),
            );
        }
        rest.push(source);
    }
    if let Some(test_id) = args.test_id {
        rest.push("--test-id".to_string());
        rest.push(test_id);
    }
    if args.sidecars_only {
        rest.push("--sidecars-only".to_string());
    }

    Ok(MigratedDiagCommand::AiPacket(AiPacketCmdContext {
        rest,
        workspace_root: workspace_root.to_path_buf(),
        resolved_out_dir,
        packet_out: args.packet_out,
        include_triage: args.include_triage,
        stats_top: 5,
        sort_override: None,
        warmup_frames: args.warmup.warmup_frames,
    }))
}

fn parse_agent_command(
    args: contracts::commands::agent::AgentCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "agent", None, None)?;
    Ok(MigratedDiagCommand::Agent(AgentCmdContext {
        bundle_source: args.source,
        workspace_root: workspace_root.to_path_buf(),
        resolved_out_dir,
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.output.json,
        out: args.output.out,
    }))
}

fn parse_dock_graph_command(
    args: contracts::commands::dock_graph::DockGraphCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::DockGraph(DockGraphCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        stats_json: args.json,
    }))
}

fn parse_dock_routing_command(
    args: contracts::commands::dock_routing::DockRoutingCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::DockRouting(DockRoutingCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.json,
    }))
}

fn parse_extensions_command(
    args: contracts::commands::extensions::ExtensionsCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "extensions", None, None)?;
    let mut rest = Vec::new();
    if let Some(key) = args.key {
        rest.push("--key".to_string());
        rest.push(key);
    }
    if args.print {
        rest.push("--print".to_string());
    }
    if let Some(source) = args.source {
        rest.push(source);
    }

    Ok(MigratedDiagCommand::Extensions(ExtensionsCmdContext {
        rest,
        workspace_root: workspace_root.to_path_buf(),
        resolved_out_dir,
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.output.json,
        out: args.output.out,
    }))
}

fn parse_frames_index_command(
    args: contracts::commands::frames_index::FramesIndexCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::FramesIndex(FramesIndexCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.json,
    }))
}

fn parse_hotspots_command(
    args: contracts::commands::hotspots::HotspotsCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "hotspots", None, None)?;
    let mut rest = Vec::new();
    if let Some(source) = args.source {
        rest.push(source);
    }
    if args.hotspots_top != 20 {
        rest.push("--hotspots-top".to_string());
        rest.push(args.hotspots_top.to_string());
    }
    if args.max_depth != 7 {
        rest.push("--max-depth".to_string());
        rest.push(args.max_depth.to_string());
    }
    if args.min_bytes != 0 {
        rest.push("--min-bytes".to_string());
        rest.push(args.min_bytes.to_string());
    }
    if args.force {
        rest.push("--force".to_string());
    }
    if args.lite {
        rest.push("--lite".to_string());
    }
    if let Some(metric) = args.metric {
        rest.push("--metric".to_string());
        rest.push(contracts::commands::hotspots::hotspots_metric_as_str(metric).to_string());
    }

    Ok(MigratedDiagCommand::Hotspots(HotspotsCmdContext {
        rest,
        workspace_root: workspace_root.to_path_buf(),
        resolved_out_dir,
        hotspots_out: args.output.out,
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.output.json,
    }))
}

fn parse_index_command(
    args: contracts::commands::index::IndexCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::Index(IndexCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        index_out: args.output.out,
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.output.json,
    }))
}

fn parse_inspect_command(
    args: contracts::commands::inspect::InspectCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let ResolvedDiagCliPaths {
        resolved_inspect_path,
        resolved_inspect_trigger_path,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "inspect",
        launch: None,
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides::default(),
    })?;

    Ok(MigratedDiagCommand::Inspect(InspectCmdContext {
        action: args.action.as_str().to_string(),
        resolved_inspect_path,
        resolved_inspect_trigger_path,
        inspect_consume_clicks: args.consume_clicks,
    }))
}

fn parse_latest_command(
    args: contracts::commands::latest::LatestCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "latest", None, args.dir)?;
    Ok(MigratedDiagCommand::Latest(LatestCmdContext {
        resolved_out_dir,
    }))
}

fn parse_layout_sidecar_command(
    args: contracts::commands::layout_sidecar::LayoutSidecarCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "layout-sidecar", None, None)?;
    let mut rest = Vec::new();
    if args.print {
        rest.push("--print".to_string());
    }
    if let Some(source) = args.source {
        rest.push(source);
    }

    Ok(MigratedDiagCommand::LayoutSidecar(
        LayoutSidecarCmdContext {
            rest,
            workspace_root: workspace_root.to_path_buf(),
            resolved_out_dir,
            stats_json: args.output.json,
            out: args.output.out,
        },
    ))
}

fn parse_layout_perf_summary_command(
    args: contracts::commands::layout_perf_summary::LayoutPerfSummaryCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    if args.top == 0 {
        return Err("--top must be >= 1".to_string());
    }
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "layout-perf-summary", None, None)?;
    let mut rest = Vec::new();
    if let Some(source) = args.source {
        rest.push(source);
    }
    if args.top != crate::layout_perf_summary::DEFAULT_LAYOUT_PERF_SUMMARY_TOP {
        rest.push("--top".to_string());
        rest.push(args.top.to_string());
    }

    Ok(MigratedDiagCommand::LayoutPerfSummary(
        LayoutPerfSummaryCmdContext {
            rest,
            workspace_root: workspace_root.to_path_buf(),
            resolved_out_dir,
            warmup_frames: args.warmup.warmup_frames,
            stats_json: args.output.json,
            out: args.output.out,
        },
    ))
}

fn parse_lint_command(
    args: contracts::commands::lint::LintCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::Lint(LintCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        lint_out: args.output.out,
        lint_all_test_ids_bounds: args.all_test_ids,
        lint_eps_px: args.lint_eps_px,
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.output.json,
    }))
}

fn parse_meta_command(
    args: contracts::commands::meta::MetaCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::Meta(MetaCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        meta_out: args.output.out,
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.output.json,
        meta_report: args.meta_report,
    }))
}

fn parse_memory_summary_command(
    args: contracts::commands::memory_summary::MemorySummaryCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    if args.top == 0 {
        return Err("--top must be >= 1".to_string());
    }
    if matches!(args.top_sessions, Some(0)) {
        return Err("--top-sessions must be >= 1".to_string());
    }
    if args.vmmap_regions_sorted_agg_top == 0 {
        return Err("--vmmap-regions-sorted-agg-top must be >= 1".to_string());
    }
    if args.vmmap_regions_sorted_detail_agg_top == 0 {
        return Err("--vmmap-regions-sorted-detail-agg-top must be >= 1".to_string());
    }
    if args.footprint_categories_agg_top == 0 {
        return Err("--footprint-categories-agg-top must be >= 1".to_string());
    }
    if args.max_samples == 0 {
        return Err("--max-samples must be >= 1".to_string());
    }
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "memory-summary", None, None)?;
    let mut rest = Vec::new();
    if let Some(source) = args.source {
        rest.push(source);
    }
    if let Some(within_session) = args.within_session {
        rest.push("--within-session".to_string());
        rest.push(within_session);
    }
    if let Some(top_sessions) = args.top_sessions {
        rest.push("--top-sessions".to_string());
        rest.push(top_sessions.to_string());
    }
    if args.sort_key != "macos_physical_footprint_peak_bytes" {
        rest.push("--sort-key".to_string());
        rest.push(args.sort_key);
    }
    for fit_linear in args.fit_linear {
        rest.push("--fit-linear".to_string());
        rest.push(fit_linear);
    }
    if args.vmmap_regions_sorted_top {
        rest.push("--vmmap-regions-sorted-top".to_string());
    }
    if args.vmmap_regions_sorted_agg {
        rest.push("--vmmap-regions-sorted-agg".to_string());
    }
    if args.vmmap_regions_sorted_agg_top != 10 {
        rest.push("--vmmap-regions-sorted-agg-top".to_string());
        rest.push(args.vmmap_regions_sorted_agg_top.to_string());
    }
    if args.vmmap_regions_sorted_detail_agg {
        rest.push("--vmmap-regions-sorted-detail-agg".to_string());
    }
    if args.vmmap_regions_sorted_detail_agg_top != 12 {
        rest.push("--vmmap-regions-sorted-detail-agg-top".to_string());
        rest.push(args.vmmap_regions_sorted_detail_agg_top.to_string());
    }
    if args.footprint_categories_agg {
        rest.push("--footprint-categories-agg".to_string());
    }
    if args.footprint_categories_agg_top != 12 {
        rest.push("--footprint-categories-agg-top".to_string());
        rest.push(args.footprint_categories_agg_top.to_string());
    }
    if args.no_recursive {
        rest.push("--no-recursive".to_string());
    }
    if args.max_depth != 3 {
        rest.push("--max-depth".to_string());
        rest.push(args.max_depth.to_string());
    }
    if args.max_samples != 200 {
        rest.push("--max-samples".to_string());
        rest.push(args.max_samples.to_string());
    }

    Ok(MigratedDiagCommand::MemorySummary(
        MemorySummaryCmdContext {
            rest,
            workspace_root: workspace_root.to_path_buf(),
            resolved_out_dir,
            top_rows: args.top,
            stats_json: args.output.json,
            out: args.output.out,
        },
    ))
}

fn parse_matrix_command(
    args: contracts::commands::matrix::MatrixCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let launch_env = parse_env_assignments(&args.env)?;
    let launch = args.normalized_launch_argv();
    if launch.is_empty() {
        return Err("--launch requires at least one command argument".to_string());
    }

    let ResolvedDiagCliPaths {
        resolved_out_dir, ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "matrix",
        launch: Some(&launch),
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides {
            out_dir: args.output.dir.clone(),
            ..DiagPathOverrides::default()
        },
    })?;

    Ok(MigratedDiagCommand::Matrix(
        crate::diag_matrix::MatrixCmdContext {
            rest: vec![args.target.as_str().to_string()],
            launch: Some(launch),
            launch_env,
            launch_high_priority: args.launch_high_priority,
            workspace_root: workspace_root.to_path_buf(),
            resolved_out_dir,
            timeout_ms: args.timing.timeout_ms,
            poll_ms: args.timing.poll_ms,
            warmup_frames: args.timing.warmup_frames,
            compare_eps_px: args.compare.compare_eps_px,
            compare_ignore_bounds: args.compare.compare_ignore_bounds,
            compare_ignore_scene_fingerprint: args.compare.compare_ignore_scene_fingerprint,
            check_view_cache_reuse_min: args.check_view_cache_reuse_min,
            check_view_cache_reuse_stable_min: args.check_view_cache_reuse_stable_min,
            check_overlay_synthesis_min: args.check_overlay_synthesis_min,
            check_viewport_input_min: args.check_viewport_input_min,
            stats_json: args.output.json,
        },
    ))
}

fn parse_pack_command(
    args: contracts::commands::pack::PackCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "pack", None, args.dir.clone())?;
    let mut rest = Vec::new();
    if let Some(source) = args.source {
        rest.push(source);
    }

    Ok(MigratedDiagCommand::Pack(PackCmdContext {
        rest,
        workspace_root: workspace_root.to_path_buf(),
        resolved_out_dir,
        pack_out: args.pack_out,
        ensure_ai_packet: args.ai_packet || args.ai_only,
        pack_ai_only: args.ai_only,
        pack_include_root_artifacts: args.include_root_artifacts || args.include_all,
        pack_include_triage: args.include_triage || args.include_all,
        pack_include_screenshots: args.include_screenshots || args.include_all,
        pack_schema2_only: args.pack_schema2_only,
        stats_top: 5,
        sort_override: None,
        warmup_frames: args.warmup.warmup_frames,
    }))
}

fn parse_path_command(
    args: contracts::commands::path::PathCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let ResolvedDiagCliPaths {
        resolved_run_context,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "path",
        launch: None,
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides {
            out_dir: args.dir,
            trigger_path: args.trigger_path,
            ..DiagPathOverrides::default()
        },
    })?;

    Ok(MigratedDiagCommand::Path(PathCmdContext {
        resolved_trigger_path: resolved_run_context.paths.trigger_path,
    }))
}

fn parse_pick_arm_command(workspace_root: &Path) -> Result<MigratedDiagCommand, String> {
    let ResolvedDiagCliPaths {
        resolved_pick_trigger_path,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "pick-arm",
        launch: None,
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides::default(),
    })?;

    Ok(MigratedDiagCommand::PickArm(PickArmCmdContext {
        resolved_pick_trigger_path,
    }))
}

fn parse_pick_command(
    args: contracts::commands::pick::PickCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let ResolvedDiagCliPaths {
        resolved_pick_trigger_path,
        resolved_pick_result_path,
        resolved_pick_result_trigger_path,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "pick",
        launch: None,
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides::default(),
    })?;

    Ok(MigratedDiagCommand::Pick(PickCmdContext {
        resolved_pick_trigger_path,
        resolved_pick_result_path,
        resolved_pick_result_trigger_path,
        timeout_ms: args.wait.timeout_ms,
        poll_ms: args.wait.poll_ms,
    }))
}

fn parse_pick_script_command(
    args: contracts::commands::pick_script::PickScriptCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let ResolvedDiagCliPaths {
        resolved_pick_trigger_path,
        resolved_pick_result_path,
        resolved_pick_result_trigger_path,
        resolved_pick_script_out,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "pick-script",
        launch: None,
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides {
            pick_script_out: args.pick_script_out,
            ..DiagPathOverrides::default()
        },
    })?;

    Ok(MigratedDiagCommand::PickScript(PickScriptCmdContext {
        resolved_pick_trigger_path,
        resolved_pick_result_path,
        resolved_pick_result_trigger_path,
        resolved_pick_script_out,
        timeout_ms: args.wait.timeout_ms,
        poll_ms: args.wait.poll_ms,
    }))
}

fn parse_pick_apply_command(
    args: contracts::commands::pick_apply::PickApplyCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let ResolvedDiagCliPaths {
        resolved_pick_trigger_path,
        resolved_pick_result_path,
        resolved_pick_result_trigger_path,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "pick-apply",
        launch: None,
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides::default(),
    })?;

    Ok(MigratedDiagCommand::PickApply(PickApplyCmdContext {
        script: args.script,
        workspace_root: workspace_root.to_path_buf(),
        resolved_pick_trigger_path,
        resolved_pick_result_path,
        resolved_pick_result_trigger_path,
        pick_apply_pointer: args.ptr,
        pick_apply_out: args.out,
        timeout_ms: args.wait.timeout_ms,
        poll_ms: args.wait.poll_ms,
    }))
}

fn parse_poke_command(
    args: contracts::commands::poke::PokeCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let ResolvedDiagCliPaths {
        resolved_out_dir,
        resolved_run_context,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "poke",
        launch: None,
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides {
            out_dir: args.dir,
            trigger_path: args.trigger_path,
            ..DiagPathOverrides::default()
        },
    })?;

    let mut rest = Vec::new();
    if let Some(label) = args.label {
        rest.push("--label".to_string());
        rest.push(label);
    }
    if let Some(max_snapshots) = args.max_snapshots {
        rest.push("--max-snapshots".to_string());
        rest.push(max_snapshots.to_string());
    }
    if let Some(request_id) = args.request_id {
        rest.push("--request-id".to_string());
        rest.push(request_id.to_string());
    }
    if args.wait {
        rest.push("--wait".to_string());
    }
    if args.record_run {
        rest.push("--record-run".to_string());
    }
    if let Some(run_id) = args.run_id {
        rest.push("--run-id".to_string());
        rest.push(run_id.to_string());
    }

    Ok(MigratedDiagCommand::Poke(PokeCmdContext {
        rest,
        resolved_out_dir,
        resolved_trigger_path: resolved_run_context.paths.trigger_path,
        timeout_ms: args.wait_args.timeout_ms,
        poll_ms: args.wait_args.poll_ms,
    }))
}

fn parse_query_command(
    args: contracts::commands::query::QueryCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::query::QuerySubcommandArgs;

    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "query", None, None)?;

    let (rest, query_out, warmup_frames, stats_json) = match args.command {
        QuerySubcommandArgs::TestId(test_id) => {
            let resolved = contracts::commands::query::resolve_query_test_id_inputs(
                &test_id.inputs,
                workspace_root,
            )?;
            let mut rest = vec!["test-id".to_string()];
            if let Some(source) = resolved.source {
                rest.push(source);
            }
            rest.push(resolved.pattern);
            if test_id.mode != "contains" {
                rest.push("--mode".to_string());
                rest.push(test_id.mode);
            }
            if test_id.top != 50 {
                rest.push("--top".to_string());
                rest.push(test_id.top.to_string());
            }
            if test_id.case_sensitive {
                rest.push("--case-sensitive".to_string());
            }
            (
                rest,
                test_id.output.out,
                test_id.warmup.warmup_frames,
                test_id.output.json,
            )
        }
        QuerySubcommandArgs::Snapshots(snapshots) => {
            let mut rest = vec!["snapshots".to_string()];
            if let Some(source) = snapshots.source {
                rest.push(source);
            }
            if snapshots.top != 20 {
                rest.push("--top".to_string());
                rest.push(snapshots.top.to_string());
            }
            if let Some(window) = snapshots.window {
                rest.push("--window".to_string());
                rest.push(window.to_string());
            }
            if snapshots.include_warmup {
                rest.push("--include-warmup".to_string());
            }
            if snapshots.include_missing_semantics {
                rest.push("--include-missing-semantics".to_string());
            }
            if let Some(semantics_source) = snapshots.semantics_source {
                rest.push("--semantics-source".to_string());
                rest.push(semantics_source);
            }
            if let Some(test_id) = snapshots.test_id {
                rest.push("--test-id".to_string());
                rest.push(test_id);
            }
            if let Some(step_index) = snapshots.step_index {
                rest.push("--step-index".to_string());
                rest.push(step_index.to_string());
            }
            (
                rest,
                snapshots.output.out,
                snapshots.warmup.warmup_frames,
                snapshots.output.json,
            )
        }
        QuerySubcommandArgs::OverlayPlacementTrace(trace) => {
            let mut rest = vec!["overlay-placement-trace".to_string()];
            if let Some(source) = trace.source {
                rest.push(source);
            }
            if trace.top != 50 {
                rest.push("--top".to_string());
                rest.push(trace.top.to_string());
            }
            if let Some(kind) = trace.kind {
                rest.push("--kind".to_string());
                rest.push(kind);
            }
            if let Some(overlay_root_name) = trace.overlay_root_name {
                rest.push("--overlay-root-name".to_string());
                rest.push(overlay_root_name);
            }
            if let Some(anchor_test_id) = trace.anchor_test_id {
                rest.push("--anchor-test-id".to_string());
                rest.push(anchor_test_id);
            }
            if let Some(content_test_id) = trace.content_test_id {
                rest.push("--content-test-id".to_string());
                rest.push(content_test_id);
            }
            if let Some(preferred_side) = trace.preferred_side {
                rest.push("--preferred-side".to_string());
                rest.push(preferred_side);
            }
            if let Some(chosen_side) = trace.chosen_side {
                rest.push("--chosen-side".to_string());
                rest.push(chosen_side);
            }
            if let Some(flipped) = trace.flipped {
                rest.push("--flipped".to_string());
                rest.push(if flipped { "true" } else { "false" }.to_string());
            }
            if let Some(align) = trace.align {
                rest.push("--align".to_string());
                rest.push(align);
            }
            if let Some(sticky) = trace.sticky {
                rest.push("--sticky".to_string());
                rest.push(sticky);
            }
            (rest, trace.output.out, 0, trace.output.json)
        }
        QuerySubcommandArgs::ScrollExtentsObservation(scroll) => {
            let mut rest = vec!["scroll-extents-observation".to_string()];
            if let Some(source) = scroll.source {
                rest.push(source);
            }
            if scroll.top != 200 {
                rest.push("--top".to_string());
                rest.push(scroll.top.to_string());
            }
            if let Some(window) = scroll.window {
                rest.push("--window".to_string());
                rest.push(window.to_string());
            }
            if scroll.all {
                rest.push("--all".to_string());
            }
            if scroll.deep_scan {
                rest.push("--deep-scan".to_string());
            }
            if scroll.timeline {
                rest.push("--timeline".to_string());
            }
            (
                rest,
                scroll.output.out,
                scroll.warmup.warmup_frames,
                scroll.output.json,
            )
        }
    };

    Ok(MigratedDiagCommand::Query(QueryCmdContext {
        rest,
        workspace_root: workspace_root.to_path_buf(),
        resolved_out_dir,
        query_out,
        warmup_frames,
        stats_json,
    }))
}

fn parse_resolve_command(
    args: contracts::commands::resolve::ResolveCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::resolve::ResolveSubcommandArgs;

    match args.command {
        ResolveSubcommandArgs::Latest(latest) => {
            let resolved_out_dir =
                resolve_diag_out_dir(workspace_root, "resolve", None, latest.dir.clone())?;
            let mut rest = vec!["latest".to_string()];
            if let Some(within_session) = latest.within_session {
                rest.push("--within-session".to_string());
                rest.push(within_session);
            }

            Ok(MigratedDiagCommand::Resolve(ResolveCmdContext {
                rest,
                workspace_root: workspace_root.to_path_buf(),
                resolved_out_dir,
                json: latest.json,
            }))
        }
    }
}

fn parse_screenshots_command(
    args: contracts::commands::screenshots::ScreenshotsCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::Screenshots(ScreenshotsCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        stats_json: args.json,
    }))
}

fn parse_slice_command(
    args: contracts::commands::slice::SliceCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let resolved_out_dir = resolve_diag_out_dir(workspace_root, "slice", None, None)?;
    let mut rest = Vec::new();
    if let Some(source) = args.source {
        rest.push(source);
    }
    rest.push("--test-id".to_string());
    rest.push(args.test_id);
    if let Some(frame_id) = args.frame_id {
        rest.push("--frame-id".to_string());
        rest.push(frame_id.to_string());
    }
    if let Some(snapshot_seq) = args.snapshot_seq {
        rest.push("--snapshot-seq".to_string());
        rest.push(snapshot_seq.to_string());
    }
    if let Some(window) = args.window {
        rest.push("--window".to_string());
        rest.push(window.to_string());
    }
    if let Some(step_index) = args.step_index {
        rest.push("--step-index".to_string());
        rest.push(step_index.to_string());
    }
    if args.max_matches != 20 {
        rest.push("--max-matches".to_string());
        rest.push(args.max_matches.to_string());
    }
    if args.max_ancestors != 64 {
        rest.push("--max-ancestors".to_string());
        rest.push(args.max_ancestors.to_string());
    }

    Ok(MigratedDiagCommand::Slice(SliceCmdContext {
        rest,
        workspace_root: workspace_root.to_path_buf(),
        resolved_out_dir,
        slice_out: args.output.out,
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.output.json,
    }))
}

fn parse_triage_command(
    args: contracts::commands::triage::TriageCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let mut rest = vec![args.source];
    if args.lite {
        rest.push("--lite".to_string());
    }
    if let Some(metric) = args.metric {
        rest.push("--metric".to_string());
        rest.push(contracts::commands::triage::triage_metric_as_str(metric).to_string());
    }

    Ok(MigratedDiagCommand::Triage(TriageCmdContext {
        rest,
        workspace_root: workspace_root.to_path_buf(),
        triage_out: args.output.out,
        stats_top: args.top,
        sort_override: args.sort,
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.output.json,
    }))
}

fn parse_test_ids_command(
    args: contracts::commands::test_ids::TestIdsCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::TestIds(TestIdsCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        test_ids_out: args.output.out,
        warmup_frames: args.warmup.warmup_frames,
        max_test_ids: args.max_test_ids,
        stats_json: args.output.json,
    }))
}

fn parse_test_ids_index_command(
    args: contracts::commands::test_ids_index::TestIdsIndexCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::TestIdsIndex(TestIdsIndexCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.json,
    }))
}

fn parse_trace_command(
    args: contracts::commands::trace::TraceCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::Trace(TraceCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        trace_out: args.trace_out,
    }))
}

fn parse_windows_command(
    args: contracts::commands::windows::WindowsCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    Ok(MigratedDiagCommand::Windows(WindowsCmdContext {
        rest: vec![args.source],
        workspace_root: workspace_root.to_path_buf(),
        warmup_frames: args.warmup.warmup_frames,
        stats_json: args.json,
    }))
}

fn parse_script_command(
    args: contracts::commands::script::ScriptCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::script::ScriptSubcommandArgs;

    const DEFAULT_TIMEOUT_MS: u64 = 240_000;
    const DEFAULT_POLL_MS: u64 = 50;

    match args.command {
        ScriptSubcommandArgs::Normalize(normalize) => {
            let ResolvedDiagCliPaths {
                resolved_out_dir,
                resolved_run_context,
                ..
            } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
                workspace_root,
                sub: "script",
                launch: None,
                session_auto: false,
                session_id: None,
                overrides: DiagPathOverrides::default(),
            })?;

            let mut rest = vec!["normalize".to_string()];
            rest.extend(normalize.inputs);

            Ok(MigratedDiagCommand::Script(ScriptCmdContext {
                rest,
                workspace_root: workspace_root.to_path_buf(),
                resolved_out_dir,
                resolved_run_context,
                script_tool_check: normalize.check,
                script_tool_write: normalize.write,
                script_tool_check_out: None,
                shrink_out: None,
                shrink_any_fail: false,
                shrink_match_reason_code: None,
                shrink_match_reason: None,
                shrink_min_steps: 1,
                shrink_max_iters: 200,
                launch: None,
                launch_env: Vec::new(),
                timeout_ms: DEFAULT_TIMEOUT_MS,
                poll_ms: DEFAULT_POLL_MS,
                stats_json: false,
            }))
        }
        ScriptSubcommandArgs::Upgrade(upgrade) => {
            let ResolvedDiagCliPaths {
                resolved_out_dir,
                resolved_run_context,
                ..
            } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
                workspace_root,
                sub: "script",
                launch: None,
                session_auto: false,
                session_id: None,
                overrides: DiagPathOverrides::default(),
            })?;

            let mut rest = vec!["upgrade".to_string()];
            rest.extend(upgrade.inputs);

            Ok(MigratedDiagCommand::Script(ScriptCmdContext {
                rest,
                workspace_root: workspace_root.to_path_buf(),
                resolved_out_dir,
                resolved_run_context,
                script_tool_check: upgrade.check,
                script_tool_write: upgrade.write,
                script_tool_check_out: None,
                shrink_out: None,
                shrink_any_fail: false,
                shrink_match_reason_code: None,
                shrink_match_reason: None,
                shrink_min_steps: 1,
                shrink_max_iters: 200,
                launch: None,
                launch_env: Vec::new(),
                timeout_ms: DEFAULT_TIMEOUT_MS,
                poll_ms: DEFAULT_POLL_MS,
                stats_json: false,
            }))
        }
        ScriptSubcommandArgs::Validate(validate) => {
            let ResolvedDiagCliPaths {
                resolved_out_dir,
                resolved_run_context,
                ..
            } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
                workspace_root,
                sub: "script",
                launch: None,
                session_auto: false,
                session_id: None,
                overrides: DiagPathOverrides {
                    out_dir: validate.output.dir.clone(),
                    ..DiagPathOverrides::default()
                },
            })?;

            let mut rest = vec!["validate".to_string()];
            rest.extend(validate.inputs);

            Ok(MigratedDiagCommand::Script(ScriptCmdContext {
                rest,
                workspace_root: workspace_root.to_path_buf(),
                resolved_out_dir,
                resolved_run_context,
                script_tool_check: false,
                script_tool_write: false,
                script_tool_check_out: validate.check_out,
                shrink_out: None,
                shrink_any_fail: false,
                shrink_match_reason_code: None,
                shrink_match_reason: None,
                shrink_min_steps: 1,
                shrink_max_iters: 200,
                launch: None,
                launch_env: Vec::new(),
                timeout_ms: DEFAULT_TIMEOUT_MS,
                poll_ms: DEFAULT_POLL_MS,
                stats_json: validate.output.json,
            }))
        }
        ScriptSubcommandArgs::Lint(lint) => {
            let ResolvedDiagCliPaths {
                resolved_out_dir,
                resolved_run_context,
                ..
            } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
                workspace_root,
                sub: "script",
                launch: None,
                session_auto: false,
                session_id: None,
                overrides: DiagPathOverrides {
                    out_dir: lint.output.dir.clone(),
                    ..DiagPathOverrides::default()
                },
            })?;

            let mut rest = vec!["lint".to_string()];
            rest.extend(lint.inputs);

            Ok(MigratedDiagCommand::Script(ScriptCmdContext {
                rest,
                workspace_root: workspace_root.to_path_buf(),
                resolved_out_dir,
                resolved_run_context,
                script_tool_check: false,
                script_tool_write: false,
                script_tool_check_out: lint.check_out,
                shrink_out: None,
                shrink_any_fail: false,
                shrink_match_reason_code: None,
                shrink_match_reason: None,
                shrink_min_steps: 1,
                shrink_max_iters: 200,
                launch: None,
                launch_env: Vec::new(),
                timeout_ms: DEFAULT_TIMEOUT_MS,
                poll_ms: DEFAULT_POLL_MS,
                stats_json: lint.output.json,
            }))
        }
        ScriptSubcommandArgs::Shrink(shrink) => {
            let launch_env = parse_env_assignments(&shrink.launch.env)?;
            let launch = shrink.launch.normalized_launch_argv();
            let ResolvedDiagCliPaths {
                resolved_out_dir,
                resolved_run_context,
                ..
            } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
                workspace_root,
                sub: "script",
                launch: launch.as_deref(),
                session_auto: shrink.session.session_auto,
                session_id: shrink.session.session.clone(),
                overrides: DiagPathOverrides {
                    out_dir: shrink.output.dir.clone(),
                    script_path: shrink.paths.script_path.clone(),
                    script_trigger_path: shrink.paths.script_trigger_path.clone(),
                    script_result_path: shrink.paths.script_result_path.clone(),
                    script_result_trigger_path: shrink.paths.script_result_trigger_path.clone(),
                    ..DiagPathOverrides::default()
                },
            })?;

            Ok(MigratedDiagCommand::Script(ScriptCmdContext {
                rest: vec!["shrink".to_string(), shrink.script],
                workspace_root: workspace_root.to_path_buf(),
                resolved_out_dir,
                resolved_run_context,
                script_tool_check: false,
                script_tool_write: false,
                script_tool_check_out: None,
                shrink_out: shrink.shrink_out,
                shrink_any_fail: shrink.shrink_any_fail,
                shrink_match_reason_code: shrink.shrink_match_reason_code,
                shrink_match_reason: shrink.shrink_match_reason,
                shrink_min_steps: shrink.shrink_min_steps,
                shrink_max_iters: shrink.shrink_max_iters,
                launch,
                launch_env,
                timeout_ms: shrink.timing.timeout_ms,
                poll_ms: shrink.timing.poll_ms,
                stats_json: shrink.output.json,
            }))
        }
        ScriptSubcommandArgs::Direct(raw) => {
            let direct = contracts::commands::script::try_parse_direct_script_args(
                std::iter::once("script".to_string()).chain(raw),
            )
            .map_err(|err| err.to_string())?;

            let ResolvedDiagCliPaths {
                resolved_out_dir,
                resolved_run_context,
                ..
            } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
                workspace_root,
                sub: "script",
                launch: None,
                session_auto: false,
                session_id: None,
                overrides: DiagPathOverrides {
                    out_dir: direct.dir,
                    script_path: direct.script_path,
                    script_trigger_path: direct.script_trigger_path,
                    ..DiagPathOverrides::default()
                },
            })?;

            Ok(MigratedDiagCommand::Script(ScriptCmdContext {
                rest: vec![direct.script],
                workspace_root: workspace_root.to_path_buf(),
                resolved_out_dir,
                resolved_run_context,
                script_tool_check: false,
                script_tool_write: false,
                script_tool_check_out: None,
                shrink_out: None,
                shrink_any_fail: false,
                shrink_match_reason_code: None,
                shrink_match_reason: None,
                shrink_min_steps: 1,
                shrink_max_iters: 200,
                launch: None,
                launch_env: Vec::new(),
                timeout_ms: DEFAULT_TIMEOUT_MS,
                poll_ms: DEFAULT_POLL_MS,
                stats_json: false,
            }))
        }
    }
}

fn parse_run_command(
    args: contracts::commands::run::RunCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let pack = pack_policy(&args.pack);
    let mut launch_env = parse_env_assignments(&args.launch.env)?;
    let checks = apply_contract_checks_to_run_checks(&args.checks);

    if checks.check_pixels_changed_test_id.is_some()
        || checks.check_pixels_unchanged_test_id.is_some()
    {
        push_env_if_missing(&mut launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
    }

    if args.launch.keep_open && args.exit_after_run {
        return Err("--keep-open conflicts with --exit-after-run".to_string());
    }

    let launch = args.launch.normalized_launch_argv();
    if uses_devtools_transport(&args.devtools) && (launch.is_some() || args.reuse_launch) {
        return Err("--launch/--reuse-launch is not supported with --devtools-ws-url".to_string());
    }
    let ResolvedDiagCliPaths {
        resolved_run_context,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "run",
        launch: launch.as_deref(),
        session_auto: args.session.session_auto,
        session_id: args.session.session.clone(),
        overrides: DiagPathOverrides {
            out_dir: args.output.dir.clone(),
            script_path: args.script_path.clone(),
            script_trigger_path: args.script_trigger_path.clone(),
            script_result_path: args.script_result_path.clone(),
            script_result_trigger_path: args.script_result_trigger_path.clone(),
            ..DiagPathOverrides::default()
        },
    })?;

    Ok(MigratedDiagCommand::Run(crate::diag_run::RunCmdContext {
        pack_after_run: pack.pack_after_run,
        ensure_ai_packet: pack.ensure_ai_packet,
        rest: vec![args.script],
        workspace_root: workspace_root.to_path_buf(),
        resolved_run_context,
        pack_out: args.pack.pack_out.clone(),
        pack_include_root_artifacts: pack.pack_include_root_artifacts,
        pack_include_triage: pack.pack_include_triage,
        pack_include_screenshots: pack.pack_include_screenshots,
        pack_schema2_only: pack.pack_schema2_only,
        stats_top: 5,
        sort_override: None,
        warmup_frames: args.timing.warmup_frames,
        timeout_ms: args.timing.timeout_ms,
        poll_ms: args.timing.poll_ms,
        trace_chrome: args.trace_chrome,
        devtools_ws_url: args.devtools.devtools_ws_url.clone(),
        devtools_token: args.devtools.devtools_token.clone(),
        devtools_session_id: args.devtools.devtools_session_id.clone(),
        exit_after_run: args.exit_after_run,
        launch,
        launch_env,
        reuse_launch: args.reuse_launch,
        launch_high_priority: args.launch.launch_high_priority,
        launch_write_bundle_json: args.launch.launch_write_bundle_json,
        keep_open: args.launch.keep_open,
        checks,
    }))
}

fn parse_repeat_command(
    args: contracts::commands::repeat::RepeatCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let launch_env = parse_env_assignments(&args.launch.env)?;
    let check_memory_p90_max = parse_memory_p90_thresholds(&args.check_memory_p90_max)?;
    let launch = args.launch.normalized_launch_argv();
    let ResolvedDiagCliPaths {
        resolved_run_context,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "repeat",
        launch: launch.as_deref(),
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides {
            out_dir: args.output.dir.clone(),
            ..DiagPathOverrides::default()
        },
    })?;

    Ok(MigratedDiagCommand::Repeat(
        crate::diag_repeat::RepeatCmdContext {
            pack_after_run: false,
            rest: vec![args.script],
            workspace_root: workspace_root.to_path_buf(),
            resolved_paths: resolved_run_context.paths,
            pack_include_screenshots: false,
            check_pixels_changed_test_id: None,
            check_pixels_unchanged_test_id: None,
            reuse_launch: false,
            launch,
            launch_env,
            launch_high_priority: args.launch.launch_high_priority,
            launch_write_bundle_json: args.launch.launch_write_bundle_json,
            perf_repeat: args.repeat,
            check_memory_p90_max,
            compare_enabled: !args.no_compare,
            compare_eps_px: args.compare.compare_eps_px,
            compare_ignore_bounds: args.compare.compare_ignore_bounds,
            compare_ignore_scene_fingerprint: args.compare.compare_ignore_scene_fingerprint,
            warmup_frames: args.timing.warmup_frames,
            lint_all_test_ids_bounds: false,
            lint_eps_px: 0.5,
            stats_json: args.output.json,
            timeout_ms: args.timing.timeout_ms,
            poll_ms: args.timing.poll_ms,
        },
    ))
}

fn parse_repro_command(
    args: contracts::commands::repro::ReproCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let mut launch_env = parse_env_assignments(&args.launch.env)?;
    let checks = apply_contract_checks_to_run_checks(&args.checks);

    if checks.check_pixels_changed_test_id.is_some()
        || checks.check_pixels_unchanged_test_id.is_some()
        || args.pack.include_screenshots
    {
        push_env_if_missing(&mut launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
    }

    let launch = args.launch.normalized_launch_argv();
    let ResolvedDiagCliPaths {
        resolved_run_context,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "repro",
        launch: launch.as_deref(),
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides {
            out_dir: args.output.dir.clone(),
            script_path: args.script_path.clone(),
            script_trigger_path: args.script_trigger_path.clone(),
            script_result_path: args.script_result_path.clone(),
            script_result_trigger_path: args.script_result_trigger_path.clone(),
            ..DiagPathOverrides::default()
        },
    })?;

    Ok(MigratedDiagCommand::Repro(
        crate::diag_repro::ReproCmdContext {
            rest: args.targets,
            workspace_root: workspace_root.to_path_buf(),
            resolved_run_context,
            pack_out: args.pack.pack_out.clone(),
            ensure_ai_packet: args.pack.ai_packet || args.pack.ai_only,
            pack_ai_only: args.pack.ai_only,
            pack_include_root_artifacts: args.pack.include_root_artifacts || args.pack.include_all,
            pack_include_triage: args.pack.include_triage || args.pack.include_all,
            pack_include_screenshots: args.pack.include_screenshots || args.pack.include_all,
            pack_schema2_only: args.pack.pack_schema2_only,
            stats_top: 5,
            sort_override: None,
            warmup_frames: args.timing.warmup_frames,
            timeout_ms: args.timing.timeout_ms,
            poll_ms: args.timing.poll_ms,
            trace_chrome: args.trace_chrome,
            launch,
            launch_env,
            launch_high_priority: args.launch.launch_high_priority,
            launch_write_bundle_json: args.launch.launch_write_bundle_json,
            with_tracy: false,
            with_renderdoc: false,
            renderdoc_after_frames: None,
            renderdoc_markers: Vec::new(),
            renderdoc_no_outputs_png: false,
            resource_footprint_thresholds: crate::ResourceFootprintThresholds::default(),
            max_macos_owned_unmapped_memory_dirty_bytes_linear_vs_renderer_gpu_images: None,
            renderer_gpu_budget_thresholds: crate::RendererGpuBudgetThresholds::default(),
            code_editor_memory_thresholds: crate::CodeEditorMemoryThresholds::default(),
            render_text_font_db_thresholds: crate::RenderTextFontDbThresholds::default(),
            wgpu_hub_counts_thresholds: crate::WgpuHubCountsThresholds::default(),
            max_wgpu_metal_current_allocated_size_bytes: None,
            max_wgpu_metal_current_allocated_size_bytes_linear_vs_renderer_gpu_images: None,
            max_render_text_atlas_bytes_live_estimate_total: None,
            check_redraw_hitches_max_total_ms_threshold: None,
            checks,
        },
    ))
}

fn parse_perf_command(
    args: contracts::commands::perf::PerfCommandArgs,
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let mut launch_env = parse_env_assignments(&args.launch.env)?;

    if args.check_pixels_changed.is_some() || args.check_pixels_unchanged.is_some() {
        push_env_if_missing(&mut launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
    }

    let launch = args.launch.normalized_launch_argv();
    let ResolvedDiagCliPaths {
        resolved_out_dir,
        resolved_run_context,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "perf",
        launch: launch.as_deref(),
        session_auto: false,
        session_id: None,
        overrides: DiagPathOverrides {
            out_dir: args.output.dir.clone(),
            ..DiagPathOverrides::default()
        },
    })?;

    Ok(MigratedDiagCommand::Perf(
        crate::diag_perf::PerfCmdContext {
            pack_after_run: false,
            rest: args.targets,
            workspace_root: workspace_root.to_path_buf(),
            resolved_out_dir,
            resolved_ready_path: resolved_run_context.paths.ready_path.clone(),
            resolved_exit_path: resolved_run_context.paths.exit_path.clone(),
            resolved_script_path: resolved_run_context.paths.script_path.clone(),
            resolved_script_trigger_path: resolved_run_context.paths.script_trigger_path.clone(),
            resolved_script_result_path: resolved_run_context.paths.script_result_path.clone(),
            resolved_script_result_trigger_path: resolved_run_context
                .paths
                .script_result_trigger_path
                .clone(),
            check_perf_hints: args.check_perf_hints,
            check_perf_hints_deny: args.check_perf_hints_deny,
            check_perf_hints_min_severity: args.check_perf_hints_min_severity,
            check_pixels_changed_test_id: args.check_pixels_changed,
            check_pixels_unchanged_test_id: args.check_pixels_unchanged,
            devtools_session_id: args.devtools.devtools_session_id,
            devtools_token: args.devtools.devtools_token,
            devtools_ws_url: args.devtools.devtools_ws_url,
            keep_open: args.launch.keep_open,
            launch,
            launch_env,
            launch_high_priority: args.launch.launch_high_priority,
            launch_write_bundle_json: args.launch.launch_write_bundle_json,
            max_frame_p95_layout_us: args.max_frame_p95_layout_us,
            max_frame_p95_solve_us: args.max_frame_p95_solve_us,
            max_frame_p95_total_us: args.max_frame_p95_total_us,
            max_pointer_move_dispatch_us: args.max_pointer_move_dispatch_us,
            max_pointer_move_global_changes: args.max_pointer_move_global_changes,
            max_pointer_move_hit_test_us: args.max_pointer_move_hit_test_us,
            max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: args
                .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            max_top_layout_us: args.max_top_layout_us,
            max_top_solve_us: args.max_top_solve_us,
            max_top_total_us: args.max_top_total_us,
            min_run_paint_cache_hit_test_only_replay_allowed_max: args
                .min_run_paint_cache_hit_test_only_replay_allowed_max,
            perf_baseline_headroom_pct: args.perf_baseline_headroom_pct,
            perf_baseline_out: args.perf_baseline_out,
            perf_baseline_path: args.perf_baseline_path,
            perf_baseline_seed_preset_paths: args.perf_baseline_seed_preset_paths,
            perf_baseline_seed_specs: args.perf_baseline_seed_specs,
            perf_repeat: args.repeat,
            perf_threshold_agg: args
                .perf_threshold_agg
                .unwrap_or(crate::PerfThresholdAggregate::Max),
            poll_ms: args.timing.poll_ms,
            reuse_launch: args.reuse_launch,
            reuse_launch_per_script: args.reuse_launch_per_script,
            sort_override: args.sort,
            stats_json: args.output.json,
            stats_top: args.top,
            suite_prelude_each_run: args.prelude_each_run,
            suite_prelude_scripts: args.prelude_scripts,
            suite_prewarm_scripts: args.prewarm_scripts,
            timeout_ms: args.timing.timeout_ms,
            trace_chrome: args.trace_chrome,
            warmup_frames: args.timing.warmup_frames,
        },
    ))
}

fn parse_campaign_command(
    args: contracts::commands::campaign::CampaignCommandArgs,
    raw_args: &[String],
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    use contracts::commands::campaign::CampaignSubcommandArgs;

    match args.command {
        CampaignSubcommandArgs::List(list) => {
            let resolved_out_dir = resolve_diag_out_dir(workspace_root, "campaign", None, None)?;
            let mut rest = vec!["list".to_string()];
            rest.extend(campaign_filter_rest(&list.filters));
            Ok(MigratedDiagCommand::Campaign(default_campaign_context(
                rest,
                workspace_root,
                resolved_out_dir,
                list.json,
                5,
                0,
            )))
        }
        CampaignSubcommandArgs::Show(show) => {
            let resolved_out_dir = resolve_diag_out_dir(workspace_root, "campaign", None, None)?;
            let rest = vec!["show".to_string(), show.campaign_id];
            Ok(MigratedDiagCommand::Campaign(default_campaign_context(
                rest,
                workspace_root,
                resolved_out_dir,
                show.json,
                5,
                0,
            )))
        }
        CampaignSubcommandArgs::Validate(validate) => {
            let resolved_out_dir = resolve_diag_out_dir(workspace_root, "campaign", None, None)?;
            let mut rest = vec!["validate".to_string()];
            rest.extend(
                validate
                    .manifests
                    .into_iter()
                    .map(|path| path.display().to_string()),
            );
            Ok(MigratedDiagCommand::Campaign(default_campaign_context(
                rest,
                workspace_root,
                resolved_out_dir,
                validate.json,
                5,
                0,
            )))
        }
        CampaignSubcommandArgs::Share(share) => {
            let resolved_out_dir = resolve_diag_out_dir(workspace_root, "campaign", None, None)?;
            let mut rest = vec!["share".to_string(), share.source];
            if share.include_passed {
                rest.push("--include-passed".to_string());
            }
            Ok(MigratedDiagCommand::Campaign(default_campaign_context(
                rest,
                workspace_root,
                resolved_out_dir,
                share.json,
                share.top,
                share.warmup_frames,
            )))
        }
        CampaignSubcommandArgs::Run(run) => {
            let mut launch_env = parse_env_assignments(&run.launch.env)?;
            let checks = apply_contract_checks_to_run_checks(&run.checks);
            if checks.check_pixels_changed_test_id.is_some()
                || checks.check_pixels_unchanged_test_id.is_some()
            {
                push_env_if_missing(&mut launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
            }

            let launch = run.launch.normalized_launch_argv();
            let resolved_out_dir = resolve_diag_out_dir(
                workspace_root,
                "campaign",
                launch.as_deref(),
                run.output.dir.clone(),
            )?;
            let mut rest = vec!["run".to_string()];
            rest.extend(run.campaign_ids);
            rest.extend(campaign_filter_rest(&run.filters));

            Ok(MigratedDiagCommand::Campaign(
                crate::diag_campaign::CampaignCmdContext {
                    pack_after_run: run.pack,
                    rest,
                    suite_script_inputs: ordered_suite_script_inputs(raw_args),
                    suite_prewarm_scripts: run.prewarm_scripts,
                    suite_prelude_scripts: run.prelude_scripts,
                    suite_prelude_each_run: run.prelude_each_run,
                    workspace_root: workspace_root.to_path_buf(),
                    resolved_out_dir,
                    devtools_ws_url: run.devtools.devtools_ws_url,
                    devtools_token: run.devtools.devtools_token,
                    devtools_session_id: run.devtools.devtools_session_id,
                    timeout_ms: run.timing.timeout_ms,
                    poll_ms: run.timing.poll_ms,
                    stats_top: run.top,
                    stats_json: run.output.json,
                    warmup_frames: run.timing.warmup_frames,
                    max_test_ids: run.max_test_ids,
                    lint_all_test_ids_bounds: run.lint_all_test_ids_bounds,
                    lint_eps_px: run.lint_eps_px,
                    suite_lint: !run.no_suite_lint,
                    pack_include_screenshots: run.include_screenshots,
                    reuse_launch: run.reuse_launch,
                    launch,
                    launch_env,
                    launch_high_priority: run.launch.launch_high_priority,
                    launch_write_bundle_json: run.launch.launch_write_bundle_json,
                    keep_open: run.launch.keep_open,
                    checks,
                },
            ))
        }
    }
}

fn parse_suite_command(
    args: contracts::commands::suite::SuiteCommandArgs,
    raw_args: &[String],
    workspace_root: &Path,
) -> Result<MigratedDiagCommand, String> {
    let pack = pack_policy(&args.pack);
    let mut launch_env = parse_env_assignments(&args.launch.env)?;
    let checks = apply_contract_checks_to_run_checks(&args.checks);

    if checks.check_pixels_changed_test_id.is_some()
        || checks.check_pixels_unchanged_test_id.is_some()
    {
        push_env_if_missing(&mut launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
    }

    let launch = args.launch.normalized_launch_argv();
    if args.suite.is_none() && args.script_dirs.is_empty() && args.globs.is_empty() {
        return Err(
            "missing suite/script input (pass a suite name, script path, `--script-dir`, or `--glob`)\n\
hint: try `fretboard diag suite ui-gallery`, `fretboard diag suite --script-dir tools/diag-scripts/ui-gallery/data_table`, or `fretboard diag suite --glob 'tools/diag-scripts/ui-gallery-select-*.json'`\n\
hint: list suites via `fretboard diag list suites`"
                .to_string(),
        );
    }
    if uses_devtools_transport(&args.devtools) && (launch.is_some() || args.reuse_launch) {
        return Err("--launch/--reuse-launch is not supported with --devtools-ws-url".to_string());
    }
    let ResolvedDiagCliPaths {
        resolved_run_context,
        ..
    } = resolve_diag_cli_paths(ResolveDiagCliPathsRequest {
        workspace_root,
        sub: "suite",
        launch: launch.as_deref(),
        session_auto: args.session.session_auto,
        session_id: args.session.session.clone(),
        overrides: DiagPathOverrides {
            out_dir: args.output.dir.clone(),
            ..DiagPathOverrides::default()
        },
    })?;

    Ok(MigratedDiagCommand::Suite(
        crate::diag_suite::SuiteCmdContext {
            pack_after_run: pack.pack_after_run,
            rest: args.suite.into_iter().collect(),
            suite_script_inputs: ordered_suite_script_inputs(raw_args),
            suite_prewarm_scripts: args.prewarm_scripts.clone(),
            suite_prelude_scripts: args.prelude_scripts.clone(),
            suite_prelude_each_run: args.prelude_each_run,
            workspace_root: workspace_root.to_path_buf(),
            resolved_paths: resolved_run_context.paths.clone(),
            devtools_ws_url: args.devtools.devtools_ws_url.clone(),
            devtools_token: args.devtools.devtools_token.clone(),
            devtools_session_id: args.devtools.devtools_session_id.clone(),
            timeout_ms: args.timing.timeout_ms,
            poll_ms: args.timing.poll_ms,
            stats_top: 5,
            stats_json: args.output.json,
            warmup_frames: args.timing.warmup_frames,
            max_test_ids: args.max_test_ids,
            lint_all_test_ids_bounds: args.lint_all_test_ids_bounds,
            lint_eps_px: args.lint_eps_px,
            suite_lint: !args.no_suite_lint,
            pack_include_screenshots: pack.pack_include_screenshots,
            reuse_launch: args.reuse_launch,
            launch,
            launch_env,
            launch_high_priority: args.launch.launch_high_priority,
            launch_write_bundle_json: args.launch.launch_write_bundle_json,
            keep_open: args.launch.keep_open,
            checks,
        },
    ))
}

fn maybe_parse_migrated_command_with_workspace(
    args: &[String],
    workspace_root: &Path,
) -> Option<Result<MigratedDiagCommand, String>> {
    if args.is_empty() {
        return None;
    }
    if let Some(err) = retired_diag_alias_error(args) {
        return Some(Err(err));
    }

    let argv = std::iter::once("fretboard diag".to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();
    let parsed = match contracts::try_parse_contract(argv) {
        Ok(parsed) => parsed,
        Err(err) => return Some(Err(clap_error_to_string(err))),
    };

    Some(match parsed.command {
        contracts::DiagCommandContract::Agent(agent) => parse_agent_command(agent, workspace_root),
        contracts::DiagCommandContract::AiPacket(ai_packet) => {
            parse_ai_packet_command(ai_packet, workspace_root)
        }
        contracts::DiagCommandContract::Artifact(artifact) => {
            parse_artifact_command(artifact, workspace_root)
        }
        contracts::DiagCommandContract::BundleV2(bundle_v2) => {
            parse_bundle_v2_command(bundle_v2, workspace_root)
        }
        contracts::DiagCommandContract::Campaign(campaign) => {
            parse_campaign_command(campaign, args, workspace_root)
        }
        contracts::DiagCommandContract::Compare(compare) => {
            parse_compare_command(compare, workspace_root)
        }
        contracts::DiagCommandContract::Config(config) => {
            parse_config_command(config, workspace_root)
        }
        contracts::DiagCommandContract::Dashboard(dashboard) => {
            parse_dashboard_command(dashboard, workspace_root)
        }
        contracts::DiagCommandContract::DockGraph(dock_graph) => {
            parse_dock_graph_command(dock_graph, workspace_root)
        }
        contracts::DiagCommandContract::DockRouting(dock_routing) => {
            parse_dock_routing_command(dock_routing, workspace_root)
        }
        contracts::DiagCommandContract::Doctor(doctor) => {
            parse_doctor_command(doctor, workspace_root)
        }
        contracts::DiagCommandContract::Extensions(extensions) => {
            parse_extensions_command(extensions, workspace_root)
        }
        contracts::DiagCommandContract::FramesIndex(frames_index) => {
            parse_frames_index_command(frames_index, workspace_root)
        }
        contracts::DiagCommandContract::Hotspots(hotspots) => {
            parse_hotspots_command(hotspots, workspace_root)
        }
        contracts::DiagCommandContract::Index(index) => parse_index_command(index, workspace_root),
        contracts::DiagCommandContract::Inspect(inspect) => {
            parse_inspect_command(inspect, workspace_root)
        }
        contracts::DiagCommandContract::Latest(latest) => {
            parse_latest_command(latest, workspace_root)
        }
        contracts::DiagCommandContract::LayoutSidecar(layout_sidecar) => {
            parse_layout_sidecar_command(layout_sidecar, workspace_root)
        }
        contracts::DiagCommandContract::LayoutPerfSummary(layout_perf_summary) => {
            parse_layout_perf_summary_command(layout_perf_summary, workspace_root)
        }
        contracts::DiagCommandContract::Lint(lint) => parse_lint_command(lint, workspace_root),
        contracts::DiagCommandContract::List(list) => parse_list_command(list, workspace_root),
        contracts::DiagCommandContract::MemorySummary(memory_summary) => {
            parse_memory_summary_command(memory_summary, workspace_root)
        }
        contracts::DiagCommandContract::Meta(meta) => parse_meta_command(meta, workspace_root),
        contracts::DiagCommandContract::Matrix(matrix) => {
            parse_matrix_command(matrix, workspace_root)
        }
        contracts::DiagCommandContract::Pack(pack) => parse_pack_command(pack, workspace_root),
        contracts::DiagCommandContract::Path(path) => parse_path_command(path, workspace_root),
        contracts::DiagCommandContract::Perf(perf) => parse_perf_command(perf, workspace_root),
        contracts::DiagCommandContract::PerfBaselineFromBundles(perf_baseline) => {
            parse_perf_baseline_from_bundles_command(perf_baseline, workspace_root)
        }
        contracts::DiagCommandContract::Poke(poke) => parse_poke_command(poke, workspace_root),
        contracts::DiagCommandContract::Pick(pick) => parse_pick_command(pick, workspace_root),
        contracts::DiagCommandContract::PickApply(pick_apply) => {
            parse_pick_apply_command(pick_apply, workspace_root)
        }
        contracts::DiagCommandContract::PickArm(_) => parse_pick_arm_command(workspace_root),
        contracts::DiagCommandContract::PickScript(pick_script) => {
            parse_pick_script_command(pick_script, workspace_root)
        }
        contracts::DiagCommandContract::Query(query) => parse_query_command(query, workspace_root),
        contracts::DiagCommandContract::Registry(registry) => {
            parse_registry_command(registry, workspace_root)
        }
        contracts::DiagCommandContract::Resolve(resolve) => {
            parse_resolve_command(resolve, workspace_root)
        }
        contracts::DiagCommandContract::Repro(repro) => parse_repro_command(repro, workspace_root),
        contracts::DiagCommandContract::Run(run) => parse_run_command(run, workspace_root),
        contracts::DiagCommandContract::Repeat(repeat) => {
            parse_repeat_command(repeat, workspace_root)
        }
        contracts::DiagCommandContract::Screenshots(screenshots) => {
            parse_screenshots_command(screenshots, workspace_root)
        }
        contracts::DiagCommandContract::Script(script) => {
            parse_script_command(script, workspace_root)
        }
        contracts::DiagCommandContract::Sessions(sessions) => {
            parse_sessions_command(sessions, workspace_root)
        }
        contracts::DiagCommandContract::Slice(slice) => parse_slice_command(slice, workspace_root),
        contracts::DiagCommandContract::Stats(stats) => parse_stats_command(stats, workspace_root),
        contracts::DiagCommandContract::Summarize(summarize) => {
            parse_summarize_command(summarize, workspace_root)
        }
        contracts::DiagCommandContract::Suite(suite) => {
            parse_suite_command(suite, args, workspace_root)
        }
        contracts::DiagCommandContract::TestIds(test_ids) => {
            parse_test_ids_command(test_ids, workspace_root)
        }
        contracts::DiagCommandContract::TestIdsIndex(test_ids_index) => {
            parse_test_ids_index_command(test_ids_index, workspace_root)
        }
        contracts::DiagCommandContract::Trace(trace) => parse_trace_command(trace, workspace_root),
        contracts::DiagCommandContract::Triage(triage) => {
            parse_triage_command(triage, workspace_root)
        }
        contracts::DiagCommandContract::Windows(windows) => {
            parse_windows_command(windows, workspace_root)
        }
    })
}

pub(crate) fn dispatch_diag_command(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        let err = contracts::try_parse_contract(["fretboard diag", "--help"])
            .expect_err("diag --help should render clap help");
        print!("{err}");
        return Ok(());
    }

    if let Some(err) = retired_diag_alias_error(args) {
        return Err(err);
    }

    let argv = std::iter::once("fretboard diag".to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();
    match contracts::try_parse_contract(argv) {
        Ok(_) => {}
        Err(err)
            if err.kind() == ErrorKind::DisplayHelp || err.kind() == ErrorKind::DisplayVersion =>
        {
            print!("{err}");
            return Ok(());
        }
        Err(err) => return Err(clap_error_to_string(err)),
    }

    let workspace_root = workspace_root()?;
    let parsed = maybe_parse_migrated_command_with_workspace(args, &workspace_root)
        .ok_or_else(|| "internal error: diag contract parser produced no command".to_string())?;

    match parsed {
        Ok(MigratedDiagCommand::Agent(ctx)) => crate::commands::agent::cmd_agent(
            ctx.bundle_source.as_deref(),
            ctx.out,
            false,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::AiPacket(ctx)) => crate::commands::ai_packet::cmd_ai_packet(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            ctx.packet_out,
            ctx.include_triage,
            ctx.stats_top,
            ctx.sort_override,
            ctx.warmup_frames,
        ),
        Ok(MigratedDiagCommand::Artifact(ctx)) => crate::commands::artifact::cmd_artifact(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            &ctx.resolved_script_result_path,
            ctx.artifact_lint_out,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::BundleV2(ctx)) => crate::commands::bundle_v2::cmd_bundle_v2(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            ctx.bundle_v2_out,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Campaign(ctx)) => crate::diag_campaign::cmd_campaign(ctx),
        Ok(MigratedDiagCommand::Compare(ctx)) => crate::diag_compare::cmd_compare(ctx),
        Ok(MigratedDiagCommand::Config(ctx)) => crate::commands::config::cmd_config(ctx),
        Ok(MigratedDiagCommand::Dashboard(ctx)) => crate::diag_dashboard::cmd_dashboard(ctx),
        Ok(MigratedDiagCommand::DockGraph(ctx)) => crate::commands::dock_graph::cmd_dock_graph(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::DockRouting(ctx)) => {
            crate::commands::dock_routing::cmd_dock_routing(
                &ctx.rest,
                false,
                &ctx.workspace_root,
                ctx.warmup_frames,
                ctx.stats_json,
            )
        }
        Ok(MigratedDiagCommand::Doctor(ctx)) => crate::commands::doctor::cmd_doctor(
            &ctx.rest,
            false,
            &workspace_root,
            &ctx.resolved_out_dir,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Extensions(ctx)) => crate::commands::extensions::cmd_extensions(
            &ctx.rest,
            &ctx.resolved_out_dir,
            &ctx.workspace_root,
            ctx.warmup_frames,
            ctx.stats_json,
            ctx.out.as_deref(),
        ),
        Ok(MigratedDiagCommand::FramesIndex(ctx)) => crate::commands::artifacts::cmd_frames_index(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Hotspots(ctx)) => crate::commands::hotspots::cmd_hotspots(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            ctx.hotspots_out,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Index(ctx)) => crate::commands::index::cmd_index(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            ctx.index_out,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Inspect(ctx)) => crate::commands::inspect::cmd_inspect(
            &[ctx.action],
            &ctx.resolved_inspect_path,
            &ctx.resolved_inspect_trigger_path,
            ctx.inspect_consume_clicks,
        ),
        Ok(MigratedDiagCommand::Latest(ctx)) => {
            crate::commands::session::cmd_latest(&[], false, &ctx.resolved_out_dir)
        }
        Ok(MigratedDiagCommand::LayoutSidecar(ctx)) => {
            crate::commands::layout_sidecar::cmd_layout_sidecar(
                &ctx.rest,
                &ctx.resolved_out_dir,
                &ctx.workspace_root,
                ctx.stats_json,
                ctx.out.as_deref(),
            )
        }
        Ok(MigratedDiagCommand::LayoutPerfSummary(ctx)) => {
            crate::commands::layout_perf_summary::cmd_layout_perf_summary(
                &ctx.rest,
                &ctx.resolved_out_dir,
                &ctx.workspace_root,
                ctx.warmup_frames,
                ctx.stats_json,
                ctx.out.as_deref(),
            )
        }
        Ok(MigratedDiagCommand::Lint(ctx)) => crate::commands::artifacts::cmd_lint(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            ctx.lint_out,
            ctx.lint_all_test_ids_bounds,
            ctx.lint_eps_px,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::List(ctx)) => crate::diag_list::cmd_list(
            &ctx.rest,
            &workspace_root,
            &ctx.resolved_out_dir,
            ctx.stats_json,
            ctx.stats_top_override,
        ),
        Ok(MigratedDiagCommand::MemorySummary(ctx)) => {
            crate::commands::memory_summary::cmd_memory_summary(
                &ctx.rest,
                &ctx.resolved_out_dir,
                &ctx.workspace_root,
                ctx.stats_json,
                ctx.top_rows,
                ctx.out.as_deref(),
            )
        }
        Ok(MigratedDiagCommand::Meta(ctx)) => crate::commands::artifacts::cmd_meta(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            ctx.meta_out,
            ctx.warmup_frames,
            ctx.stats_json,
            ctx.meta_report,
        ),
        Ok(MigratedDiagCommand::Matrix(ctx)) => crate::diag_matrix::cmd_matrix(ctx),
        Ok(MigratedDiagCommand::Pack(ctx)) => crate::commands::artifacts::cmd_pack(
            &ctx.rest,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            ctx.pack_out,
            ctx.ensure_ai_packet,
            ctx.pack_ai_only,
            ctx.pack_include_root_artifacts,
            ctx.pack_include_triage,
            ctx.pack_include_screenshots,
            ctx.pack_schema2_only,
            ctx.stats_top,
            ctx.sort_override,
            ctx.warmup_frames,
        ),
        Ok(MigratedDiagCommand::Path(ctx)) => {
            crate::commands::session::cmd_path(&[], false, &ctx.resolved_trigger_path)
        }
        Ok(MigratedDiagCommand::Perf(ctx)) => crate::diag_perf::cmd_perf(ctx),
        Ok(MigratedDiagCommand::PerfBaselineFromBundles(ctx)) => {
            crate::diag_perf_baseline::cmd_perf_baseline_from_bundles(ctx)
        }
        Ok(MigratedDiagCommand::Poke(ctx)) => crate::commands::session::cmd_poke(
            &ctx.rest,
            false,
            &ctx.resolved_out_dir,
            &ctx.resolved_trigger_path,
            ctx.timeout_ms,
            ctx.poll_ms,
        ),
        Ok(MigratedDiagCommand::Pick(ctx)) => crate::commands::pick::cmd_pick(
            &[],
            &ctx.resolved_pick_trigger_path,
            &ctx.resolved_pick_result_path,
            &ctx.resolved_pick_result_trigger_path,
            ctx.timeout_ms,
            ctx.poll_ms,
        ),
        Ok(MigratedDiagCommand::PickApply(ctx)) => crate::commands::pick::cmd_pick_apply(
            &[ctx.script],
            &ctx.workspace_root,
            &ctx.resolved_pick_trigger_path,
            &ctx.resolved_pick_result_path,
            &ctx.resolved_pick_result_trigger_path,
            Some(ctx.pick_apply_pointer.as_str()),
            ctx.pick_apply_out,
            ctx.timeout_ms,
            ctx.poll_ms,
        ),
        Ok(MigratedDiagCommand::PickArm(ctx)) => {
            crate::commands::pick::cmd_pick_arm(&[], &ctx.resolved_pick_trigger_path)
        }
        Ok(MigratedDiagCommand::PickScript(ctx)) => crate::commands::pick::cmd_pick_script(
            &[],
            &ctx.resolved_pick_trigger_path,
            &ctx.resolved_pick_result_path,
            &ctx.resolved_pick_result_trigger_path,
            &ctx.resolved_pick_script_out,
            ctx.timeout_ms,
            ctx.poll_ms,
        ),
        Ok(MigratedDiagCommand::Query(ctx)) => crate::commands::query::cmd_query(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            ctx.query_out,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Registry(ctx)) => {
            crate::commands::registry::cmd_registry(&ctx.rest, &ctx.workspace_root, ctx.stats_json)
        }
        Ok(MigratedDiagCommand::Resolve(ctx)) => crate::commands::resolve::cmd_resolve(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            ctx.json,
        ),
        Ok(MigratedDiagCommand::Repro(ctx)) => crate::diag_repro::cmd_repro(ctx),
        Ok(MigratedDiagCommand::Run(ctx)) => crate::diag_run::cmd_run(ctx),
        Ok(MigratedDiagCommand::Repeat(ctx)) => crate::diag_repeat::cmd_repeat(ctx),
        Ok(MigratedDiagCommand::Screenshots(ctx)) => crate::commands::screenshots::cmd_screenshots(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Script(ctx)) => crate::commands::script::cmd_script(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            &ctx.resolved_run_context.paths.script_path,
            &ctx.resolved_run_context.paths.script_trigger_path,
            &ctx.resolved_run_context.paths.script_result_path,
            &ctx.resolved_run_context.paths.script_result_trigger_path,
            &ctx.resolved_run_context.paths.ready_path,
            &ctx.resolved_run_context.paths.exit_path,
            ctx.script_tool_check,
            ctx.script_tool_write,
            ctx.script_tool_check_out,
            ctx.shrink_out,
            ctx.shrink_any_fail,
            ctx.shrink_match_reason_code,
            ctx.shrink_match_reason,
            ctx.shrink_min_steps,
            ctx.shrink_max_iters,
            &ctx.launch,
            &ctx.launch_env,
            ctx.timeout_ms,
            ctx.poll_ms,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Sessions(ctx)) => crate::diag_sessions::cmd_sessions(
            &ctx.rest,
            &ctx.resolved_out_dir,
            ctx.stats_json,
            ctx.top_override,
        ),
        Ok(MigratedDiagCommand::Slice(ctx)) => crate::commands::slice::cmd_slice(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            &ctx.resolved_out_dir,
            ctx.slice_out,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Stats(ctx)) => crate::diag_stats::cmd_stats(ctx),
        Ok(MigratedDiagCommand::Summarize(ctx)) => crate::diag_summarize::cmd_summarize(ctx),
        Ok(MigratedDiagCommand::Suite(ctx)) => crate::diag_suite::cmd_suite(ctx),
        Ok(MigratedDiagCommand::TestIds(ctx)) => crate::commands::artifacts::cmd_test_ids(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            ctx.test_ids_out,
            ctx.warmup_frames,
            ctx.max_test_ids,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::TestIdsIndex(ctx)) => {
            crate::commands::artifacts::cmd_test_ids_index(
                &ctx.rest,
                false,
                &ctx.workspace_root,
                ctx.warmup_frames,
                ctx.stats_json,
            )
        }
        Ok(MigratedDiagCommand::Trace(ctx)) => {
            crate::commands::trace::cmd_trace(&ctx.rest, false, &ctx.workspace_root, ctx.trace_out)
        }
        Ok(MigratedDiagCommand::Triage(ctx)) => crate::commands::artifacts::cmd_triage(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            ctx.triage_out,
            ctx.stats_top,
            ctx.sort_override,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Ok(MigratedDiagCommand::Windows(ctx)) => crate::commands::windows::cmd_windows(
            &ctx.rest,
            false,
            &ctx.workspace_root,
            ctx.warmup_frames,
            ctx.stats_json,
        ),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn workspace_root_for_tests() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate dir should have workspace parent")
            .parent()
            .expect("workspace root should exist")
            .to_path_buf()
    }

    #[test]
    fn dispatch_diag_command_accepts_empty_args_as_root_help() {
        dispatch_diag_command(&[]).expect("empty diag args should render root help");
    }

    #[test]
    fn migrated_run_subset_builds_a_real_run_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "run".to_string(),
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-run".to_string(),
            "--timeout-ms".to_string(),
            "7".to_string(),
            "--poll-ms".to_string(),
            "3".to_string(),
            "--check-pixels-changed".to_string(),
            "ui-gallery-root".to_string(),
            "--launch".to_string(),
            "--".to_string(),
            "cargo".to_string(),
            "run".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("run subset should be intercepted")
            .expect("run subset should parse");

        let MigratedDiagCommand::Run(ctx) = parsed else {
            panic!("expected run context");
        };

        assert_eq!(
            ctx.rest,
            vec!["tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string()]
        );
        assert_eq!(ctx.timeout_ms, 7);
        assert_eq!(ctx.poll_ms, 3);
        assert!(
            ctx.launch_env
                .iter()
                .any(|(key, value)| key == "FRET_DIAG_GPU_SCREENSHOTS" && value == "1")
        );
        assert_eq!(
            ctx.launch,
            Some(vec!["cargo".to_string(), "run".to_string()])
        );
        assert!(
            ctx.resolved_run_context
                .paths
                .out_dir
                .ends_with("target/fret-diag-cutover-run")
        );
    }

    #[test]
    fn migrated_run_rejects_devtools_transport_with_launch_or_reuse_launch() {
        let workspace_root = workspace_root_for_tests();
        let err = match maybe_parse_migrated_command_with_workspace(
            &[
                "run".to_string(),
                "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
                "--devtools-ws-url".to_string(),
                "ws://127.0.0.1:7331/".to_string(),
                "--devtools-token".to_string(),
                "secret".to_string(),
                "--launch".to_string(),
                "--".to_string(),
                "cargo".to_string(),
                "run".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("devtools + launch should be rejected"),
            None => panic!("run should stay on migrated parser"),
        };
        assert!(err.contains("--launch/--reuse-launch is not supported with --devtools-ws-url"));
    }

    #[test]
    fn migrated_campaign_list_builds_a_real_campaign_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "campaign".to_string(),
            "list".to_string(),
            "--lane".to_string(),
            "smoke".to_string(),
            "--tag".to_string(),
            "ui-gallery".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("campaign list should be intercepted")
            .expect("campaign list should parse");

        let MigratedDiagCommand::Campaign(ctx) = parsed else {
            panic!("expected campaign context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "list".to_string(),
                "--lane".to_string(),
                "smoke".to_string(),
                "--tag".to_string(),
                "ui-gallery".to_string(),
            ]
        );
        assert!(ctx.stats_json);
        assert_eq!(ctx.stats_top, 5);
    }

    #[test]
    fn migrated_campaign_run_builds_a_real_campaign_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "campaign".to_string(),
            "run".to_string(),
            "ui-gallery-smoke".to_string(),
            "--lane".to_string(),
            "smoke".to_string(),
            "--script-dir".to_string(),
            "tools/diag-scripts/ui-gallery/data_table".to_string(),
            "--glob".to_string(),
            "tools/diag-scripts/ui-gallery-select-*.json".to_string(),
            "--prewarm-script".to_string(),
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
            "--prelude-script".to_string(),
            "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json".to_string(),
            "--prelude-each-run".to_string(),
            "--pack".to_string(),
            "--include-screenshots".to_string(),
            "--top".to_string(),
            "8".to_string(),
            "--check-pixels-changed".to_string(),
            "ui-gallery-root".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-campaign".to_string(),
            "--launch".to_string(),
            "--".to_string(),
            "cargo".to_string(),
            "run".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("campaign run should be intercepted")
            .expect("campaign run should parse");

        let MigratedDiagCommand::Campaign(ctx) = parsed else {
            panic!("expected campaign context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "run".to_string(),
                "ui-gallery-smoke".to_string(),
                "--lane".to_string(),
                "smoke".to_string(),
            ]
        );
        assert_eq!(
            ctx.suite_script_inputs,
            vec![
                "tools/diag-scripts/ui-gallery/data_table".to_string(),
                "tools/diag-scripts/ui-gallery-select-*.json".to_string(),
            ]
        );
        assert_eq!(ctx.stats_top, 8);
        assert!(ctx.pack_after_run);
        assert!(ctx.pack_include_screenshots);
        assert!(ctx.suite_prelude_each_run);
        assert_eq!(
            ctx.suite_prewarm_scripts,
            vec![PathBuf::from(
                "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json"
            )]
        );
        assert_eq!(
            ctx.suite_prelude_scripts,
            vec![PathBuf::from(
                "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json"
            )]
        );
        assert_eq!(
            ctx.launch,
            Some(vec!["cargo".to_string(), "run".to_string()])
        );
        assert!(
            ctx.launch_env
                .iter()
                .any(|(key, value)| key == "FRET_DIAG_GPU_SCREENSHOTS" && value == "1")
        );
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-campaign")
        );
    }

    #[test]
    fn migrated_list_scripts_builds_a_real_list_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "list".to_string(),
            "scripts".to_string(),
            "--contains".to_string(),
            "ui-gallery".to_string(),
            "--case-sensitive".to_string(),
            "--all".to_string(),
            "--top".to_string(),
            "9".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("list scripts should be intercepted")
            .expect("list scripts should parse");

        let MigratedDiagCommand::List(ctx) = parsed else {
            panic!("expected list context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "scripts".to_string(),
                "--contains".to_string(),
                "ui-gallery".to_string(),
                "--case-sensitive".to_string(),
                "--all".to_string(),
            ]
        );
        assert!(ctx.stats_json);
        assert_eq!(ctx.stats_top_override, Some(9));
    }

    #[test]
    fn migrated_list_sessions_builds_a_real_list_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "list".to_string(),
            "sessions".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-list".to_string(),
            "--contains".to_string(),
            "smoke".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("list sessions should be intercepted")
            .expect("list sessions should parse");

        let MigratedDiagCommand::List(ctx) = parsed else {
            panic!("expected list context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "sessions".to_string(),
                "--contains".to_string(),
                "smoke".to_string(),
            ]
        );
        assert!(!ctx.stats_json);
        assert_eq!(ctx.stats_top_override, None);
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-list")
        );
    }

    #[test]
    fn migrated_sessions_clean_builds_a_real_sessions_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "sessions".to_string(),
            "clean".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-sessions".to_string(),
            "--keep".to_string(),
            "10".to_string(),
            "--older-than-days".to_string(),
            "14".to_string(),
            "--top".to_string(),
            "25".to_string(),
            "--apply".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("sessions clean should be intercepted")
            .expect("sessions clean should parse");

        let MigratedDiagCommand::Sessions(ctx) = parsed else {
            panic!("expected sessions context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "clean".to_string(),
                "--keep".to_string(),
                "10".to_string(),
                "--older-than-days".to_string(),
                "14".to_string(),
                "--apply".to_string(),
            ]
        );
        assert!(ctx.stats_json);
        assert_eq!(ctx.top_override, Some(25));
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-sessions")
        );
    }

    #[test]
    fn migrated_doctor_bundle_builds_a_real_doctor_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "doctor".to_string(),
            "target/fret-diag/sessions/demo".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-doctor".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--fix".to_string(),
            "--fix-schema2".to_string(),
            "--fix-bundle-json".to_string(),
            "--check-all".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("doctor should be intercepted")
            .expect("doctor should parse");

        let MigratedDiagCommand::Doctor(ctx) = parsed else {
            panic!("expected doctor context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "target/fret-diag/sessions/demo".to_string(),
                "--fix".to_string(),
                "--fix-schema2".to_string(),
                "--fix-bundle-json".to_string(),
                "--check-all".to_string(),
            ]
        );
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_json);
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-doctor")
        );
    }

    #[test]
    fn migrated_doctor_scripts_builds_a_real_doctor_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "doctor".to_string(),
            "scripts".to_string(),
            "--max-examples".to_string(),
            "7".to_string(),
            "--strict".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("doctor scripts should be intercepted")
            .expect("doctor scripts should parse");

        let MigratedDiagCommand::Doctor(ctx) = parsed else {
            panic!("expected doctor context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "scripts".to_string(),
                "--max-examples".to_string(),
                "7".to_string(),
                "--strict".to_string(),
            ]
        );
        assert_eq!(ctx.warmup_frames, 0);
        assert!(ctx.stats_json);
    }

    #[test]
    fn migrated_doctor_campaigns_builds_a_real_doctor_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "doctor".to_string(),
            "campaigns".to_string(),
            "--strict".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("doctor campaigns should be intercepted")
            .expect("doctor campaigns should parse");

        let MigratedDiagCommand::Doctor(ctx) = parsed else {
            panic!("expected doctor context");
        };

        assert_eq!(
            ctx.rest,
            vec!["campaigns".to_string(), "--strict".to_string()]
        );
        assert_eq!(ctx.warmup_frames, 0);
        assert!(!ctx.stats_json);
    }

    #[test]
    fn migrated_config_doctor_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "config".to_string(),
            "doctor".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-config".to_string(),
            "--env".to_string(),
            "FRET_DIAG_MAX_SNAPSHOTS=50".to_string(),
            "--mode".to_string(),
            "manual".to_string(),
            "--config-path".to_string(),
            "tools/diag-configs/diag.config.example.json".to_string(),
            "--show-env".to_string(),
            "all".to_string(),
            "--report-json".to_string(),
            "--print-launch-policy".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("config doctor should be intercepted")
            .expect("config doctor should parse");

        let MigratedDiagCommand::Config(ctx) = parsed else {
            panic!("expected config context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "doctor".to_string(),
                "--mode".to_string(),
                "manual".to_string(),
                "--config-path".to_string(),
                "tools/diag-configs/diag.config.example.json".to_string(),
                "--show-env".to_string(),
                "all".to_string(),
                "--report-json".to_string(),
                "--print-launch-policy".to_string(),
            ]
        );
        assert_eq!(
            ctx.launch_env,
            vec![("FRET_DIAG_MAX_SNAPSHOTS".to_string(), "50".to_string())]
        );
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-config")
        );
        assert!(
            ctx.resolved_ready_path
                .ends_with("target/fret-diag-cutover-config/ready.touch")
        );
    }

    #[test]
    fn migrated_registry_write_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "registry".to_string(),
            "write".to_string(),
            "--path".to_string(),
            "target/diag.index.json".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("registry write should be intercepted")
            .expect("registry write should parse");

        let MigratedDiagCommand::Registry(ctx) = parsed else {
            panic!("expected registry context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "write".to_string(),
                "--path".to_string(),
                "target/diag.index.json".to_string(),
            ]
        );
        assert!(ctx.stats_json);
        assert_eq!(ctx.workspace_root, workspace_root);
    }

    #[test]
    fn migrated_path_builds_a_real_trigger_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "path".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-path".to_string(),
            "--trigger-path".to_string(),
            "target/fret-diag-cutover-path/custom.trigger".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("path should be intercepted")
            .expect("path should parse");

        let MigratedDiagCommand::Path(ctx) = parsed else {
            panic!("expected path context");
        };

        assert!(
            ctx.resolved_trigger_path
                .ends_with("target/fret-diag-cutover-path/custom.trigger")
        );
    }

    #[test]
    fn migrated_agent_builds_a_real_output_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "agent".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "7".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/agent.plan.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("agent should be intercepted")
            .expect("agent should parse");

        let MigratedDiagCommand::Agent(ctx) = parsed else {
            panic!("expected agent context");
        };

        assert_eq!(ctx.bundle_source.as_deref(), Some("target/fret-diag/demo"));
        assert_eq!(ctx.workspace_root, workspace_root);
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
        assert_eq!(ctx.warmup_frames, 7);
        assert!(ctx.stats_json);
        assert_eq!(ctx.out, Some(PathBuf::from("target/agent.plan.json")));
    }

    #[test]
    fn migrated_poke_builds_a_real_poke_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "poke".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-poke".to_string(),
            "--trigger-path".to_string(),
            "target/fret-diag-cutover-poke/custom.trigger".to_string(),
            "--label".to_string(),
            "manual-dump".to_string(),
            "--max-snapshots".to_string(),
            "8".to_string(),
            "--request-id".to_string(),
            "42".to_string(),
            "--wait".to_string(),
            "--record-run".to_string(),
            "--run-id".to_string(),
            "99".to_string(),
            "--timeout-ms".to_string(),
            "9".to_string(),
            "--poll-ms".to_string(),
            "3".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("poke should be intercepted")
            .expect("poke should parse");

        let MigratedDiagCommand::Poke(ctx) = parsed else {
            panic!("expected poke context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "--label".to_string(),
                "manual-dump".to_string(),
                "--max-snapshots".to_string(),
                "8".to_string(),
                "--request-id".to_string(),
                "42".to_string(),
                "--wait".to_string(),
                "--record-run".to_string(),
                "--run-id".to_string(),
                "99".to_string(),
            ]
        );
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-poke")
        );
        assert!(
            ctx.resolved_trigger_path
                .ends_with("target/fret-diag-cutover-poke/custom.trigger")
        );
        assert_eq!(ctx.timeout_ms, 9);
        assert_eq!(ctx.poll_ms, 3);
    }

    #[test]
    fn migrated_latest_builds_a_real_output_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "latest".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-latest".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("latest should be intercepted")
            .expect("latest should parse");

        let MigratedDiagCommand::Latest(ctx) = parsed else {
            panic!("expected latest context");
        };

        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-latest")
        );
    }

    #[test]
    fn migrated_artifact_lint_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "artifact".to_string(),
            "lint".to_string(),
            "target/fret-diag/42".to_string(),
            "--warmup-frames".to_string(),
            "3".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/artifact.lint.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("artifact lint should be intercepted")
            .expect("artifact lint should parse");

        let MigratedDiagCommand::Artifact(ctx) = parsed else {
            panic!("expected artifact context");
        };

        assert_eq!(
            ctx.rest,
            vec!["lint".to_string(), "target/fret-diag/42".to_string()]
        );
        assert_eq!(ctx.warmup_frames, 3);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.artifact_lint_out,
            Some(PathBuf::from("target/artifact.lint.json"))
        );
        assert!(
            ctx.resolved_script_result_path
                .ends_with("target/fret-diag/script.result.json")
        );
    }

    #[test]
    fn migrated_lint_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "lint".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "3".to_string(),
            "--all-test-ids".to_string(),
            "--lint-eps-px".to_string(),
            "1.25".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/bundle.lint.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("lint should be intercepted")
            .expect("lint should parse");

        let MigratedDiagCommand::Lint(ctx) = parsed else {
            panic!("expected lint context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.warmup_frames, 3);
        assert!(ctx.lint_all_test_ids_bounds);
        assert_eq!(ctx.lint_eps_px, 1.25);
        assert!(ctx.stats_json);
        assert_eq!(ctx.lint_out, Some(PathBuf::from("target/bundle.lint.json")));
    }

    #[test]
    fn migrated_pack_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "pack".to_string(),
            "target/fret-diag/demo".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-pack".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--pack-out".to_string(),
            "target/demo.zip".to_string(),
            "--ai-packet".to_string(),
            "--include-all".to_string(),
            "--pack-schema2-only".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("pack should be intercepted")
            .expect("pack should parse");

        let MigratedDiagCommand::Pack(ctx) = parsed else {
            panic!("expected pack context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.pack_out, Some(PathBuf::from("target/demo.zip")));
        assert!(ctx.ensure_ai_packet);
        assert!(!ctx.pack_ai_only);
        assert!(ctx.pack_include_root_artifacts);
        assert!(ctx.pack_include_triage);
        assert!(ctx.pack_include_screenshots);
        assert!(ctx.pack_schema2_only);
        assert_eq!(ctx.warmup_frames, 4);
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-pack")
        );
    }

    #[test]
    fn migrated_triage_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "triage".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "5".to_string(),
            "--top".to_string(),
            "8".to_string(),
            "--sort".to_string(),
            "time".to_string(),
            "--lite".to_string(),
            "--metric".to_string(),
            "layout".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/triage.lite.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("triage should be intercepted")
            .expect("triage should parse");

        let MigratedDiagCommand::Triage(ctx) = parsed else {
            panic!("expected triage context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "target/fret-diag/demo".to_string(),
                "--lite".to_string(),
                "--metric".to_string(),
                "layout".to_string(),
            ]
        );
        assert_eq!(ctx.stats_top, 8);
        assert_eq!(ctx.sort_override, Some(crate::BundleStatsSort::Time));
        assert_eq!(ctx.warmup_frames, 5);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.triage_out,
            Some(PathBuf::from("target/triage.lite.json"))
        );
    }

    #[test]
    fn migrated_ai_packet_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "ai-packet".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "6".to_string(),
            "--packet-out".to_string(),
            "target/ai.packet".to_string(),
            "--test-id".to_string(),
            "ui-gallery-root".to_string(),
            "--sidecars-only".to_string(),
            "--include-triage".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("ai-packet should be intercepted")
            .expect("ai-packet should parse");

        let MigratedDiagCommand::AiPacket(ctx) = parsed else {
            panic!("expected ai-packet context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "target/fret-diag/demo".to_string(),
                "--test-id".to_string(),
                "ui-gallery-root".to_string(),
                "--sidecars-only".to_string(),
            ]
        );
        assert_eq!(ctx.packet_out, Some(PathBuf::from("target/ai.packet")));
        assert!(ctx.include_triage);
        assert_eq!(ctx.warmup_frames, 6);
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
    }

    #[test]
    fn migrated_meta_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "meta".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/bundle.meta.json".to_string(),
            "--meta-report".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("meta should be intercepted")
            .expect("meta should parse");

        let MigratedDiagCommand::Meta(ctx) = parsed else {
            panic!("expected meta context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_json);
        assert!(ctx.meta_report);
        assert_eq!(ctx.meta_out, Some(PathBuf::from("target/bundle.meta.json")));
    }

    #[test]
    fn migrated_index_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "index".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "2".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/bundle.index.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("index should be intercepted")
            .expect("index should parse");

        let MigratedDiagCommand::Index(ctx) = parsed else {
            panic!("expected index context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.warmup_frames, 2);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.index_out,
            Some(PathBuf::from("target/bundle.index.json"))
        );
    }

    #[test]
    fn migrated_test_ids_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "test-ids".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "5".to_string(),
            "--max-test-ids".to_string(),
            "17".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/test_ids.index.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("test-ids should be intercepted")
            .expect("test-ids should parse");

        let MigratedDiagCommand::TestIds(ctx) = parsed else {
            panic!("expected test-ids context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.warmup_frames, 5);
        assert_eq!(ctx.max_test_ids, 17);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.test_ids_out,
            Some(PathBuf::from("target/test_ids.index.json"))
        );
    }

    #[test]
    fn migrated_layout_sidecar_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "layout-sidecar".to_string(),
            "target/fret-diag/demo".to_string(),
            "--print".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/layout.taffy.v1.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("layout-sidecar should be intercepted")
            .expect("layout-sidecar should parse");

        let MigratedDiagCommand::LayoutSidecar(ctx) = parsed else {
            panic!("expected layout-sidecar context");
        };

        assert_eq!(
            ctx.rest,
            vec!["--print".to_string(), "target/fret-diag/demo".to_string()]
        );
        assert!(ctx.stats_json);
        assert_eq!(ctx.out, Some(PathBuf::from("target/layout.taffy.v1.json")));
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
    }

    #[test]
    fn migrated_layout_perf_summary_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "layout-perf-summary".to_string(),
            "target/fret-diag/demo".to_string(),
            "--top".to_string(),
            "7".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/layout_perf_summary.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("layout-perf-summary should be intercepted")
            .expect("layout-perf-summary should parse");

        let MigratedDiagCommand::LayoutPerfSummary(ctx) = parsed else {
            panic!("expected layout-perf-summary context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "target/fret-diag/demo".to_string(),
                "--top".to_string(),
                "7".to_string(),
            ]
        );
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.out,
            Some(PathBuf::from("target/layout_perf_summary.json"))
        );
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
    }

    #[test]
    fn migrated_memory_summary_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "memory-summary".to_string(),
            "target/fret-diag/demo".to_string(),
            "--within-session".to_string(),
            "latest".to_string(),
            "--top-sessions".to_string(),
            "3".to_string(),
            "--sort-key".to_string(),
            "renderer_gpu_images_bytes_estimate".to_string(),
            "--fit-linear".to_string(),
            "renderer_gpu_images_bytes_estimate:macos_physical_footprint_peak_bytes".to_string(),
            "--top".to_string(),
            "9".to_string(),
            "--vmmap-regions-sorted-top".to_string(),
            "--footprint-categories-agg".to_string(),
            "--max-depth".to_string(),
            "4".to_string(),
            "--max-samples".to_string(),
            "120".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/memory_summary.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("memory-summary should be intercepted")
            .expect("memory-summary should parse");

        let MigratedDiagCommand::MemorySummary(ctx) = parsed else {
            panic!("expected memory-summary context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "target/fret-diag/demo".to_string(),
                "--within-session".to_string(),
                "latest".to_string(),
                "--top-sessions".to_string(),
                "3".to_string(),
                "--sort-key".to_string(),
                "renderer_gpu_images_bytes_estimate".to_string(),
                "--fit-linear".to_string(),
                "renderer_gpu_images_bytes_estimate:macos_physical_footprint_peak_bytes"
                    .to_string(),
                "--vmmap-regions-sorted-top".to_string(),
                "--footprint-categories-agg".to_string(),
                "--max-depth".to_string(),
                "4".to_string(),
                "--max-samples".to_string(),
                "120".to_string(),
            ]
        );
        assert_eq!(ctx.top_rows, 9);
        assert!(ctx.stats_json);
        assert_eq!(ctx.out, Some(PathBuf::from("target/memory_summary.json")));
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
    }

    #[test]
    fn migrated_inspect_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "inspect".to_string(),
            "toggle".to_string(),
            "--consume-clicks".to_string(),
            "false".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("inspect should be intercepted")
            .expect("inspect should parse");

        let MigratedDiagCommand::Inspect(ctx) = parsed else {
            panic!("expected inspect context");
        };

        assert_eq!(ctx.action, "toggle");
        assert_eq!(ctx.inspect_consume_clicks, Some(false));
        assert!(
            ctx.resolved_inspect_path
                .ends_with("target/fret-diag/inspect.json")
        );
        assert!(
            ctx.resolved_inspect_trigger_path
                .ends_with("target/fret-diag/inspect.touch")
        );
    }

    #[test]
    fn migrated_pick_helpers_build_real_contexts() {
        let workspace_root = workspace_root_for_tests();

        let pick_arm =
            maybe_parse_migrated_command_with_workspace(&["pick-arm".to_string()], &workspace_root)
                .expect("pick-arm should be intercepted")
                .expect("pick-arm should parse");
        let MigratedDiagCommand::PickArm(pick_arm_ctx) = pick_arm else {
            panic!("expected pick-arm context");
        };
        assert!(
            pick_arm_ctx
                .resolved_pick_trigger_path
                .ends_with("target/fret-diag/pick.touch")
        );

        let pick = maybe_parse_migrated_command_with_workspace(
            &[
                "pick".to_string(),
                "--timeout-ms".to_string(),
                "9".to_string(),
                "--poll-ms".to_string(),
                "3".to_string(),
            ],
            &workspace_root,
        )
        .expect("pick should be intercepted")
        .expect("pick should parse");
        let MigratedDiagCommand::Pick(pick_ctx) = pick else {
            panic!("expected pick context");
        };
        assert_eq!(pick_ctx.timeout_ms, 9);
        assert_eq!(pick_ctx.poll_ms, 3);
        assert!(
            pick_ctx
                .resolved_pick_result_path
                .ends_with("target/fret-diag/pick.result.json")
        );

        let pick_script = maybe_parse_migrated_command_with_workspace(
            &[
                "pick-script".to_string(),
                "--timeout-ms".to_string(),
                "9".to_string(),
                "--poll-ms".to_string(),
                "3".to_string(),
                "--pick-script-out".to_string(),
                "target/picked.script.json".to_string(),
            ],
            &workspace_root,
        )
        .expect("pick-script should be intercepted")
        .expect("pick-script should parse");
        let MigratedDiagCommand::PickScript(pick_script_ctx) = pick_script else {
            panic!("expected pick-script context");
        };
        assert_eq!(pick_script_ctx.timeout_ms, 9);
        assert_eq!(pick_script_ctx.poll_ms, 3);
        assert_eq!(
            pick_script_ctx.resolved_pick_script_out,
            workspace_root.join("target/picked.script.json")
        );

        let pick_apply = maybe_parse_migrated_command_with_workspace(
            &[
                "pick-apply".to_string(),
                "./script.json".to_string(),
                "--ptr".to_string(),
                "/steps/0/target".to_string(),
                "--out".to_string(),
                "target/picked.json".to_string(),
                "--timeout-ms".to_string(),
                "9".to_string(),
                "--poll-ms".to_string(),
                "3".to_string(),
            ],
            &workspace_root,
        )
        .expect("pick-apply should be intercepted")
        .expect("pick-apply should parse");
        let MigratedDiagCommand::PickApply(pick_apply_ctx) = pick_apply else {
            panic!("expected pick-apply context");
        };
        assert_eq!(pick_apply_ctx.script, "./script.json");
        assert_eq!(pick_apply_ctx.pick_apply_pointer, "/steps/0/target");
        assert_eq!(
            pick_apply_ctx.pick_apply_out,
            Some(PathBuf::from("target/picked.json"))
        );
        assert_eq!(pick_apply_ctx.timeout_ms, 9);
        assert_eq!(pick_apply_ctx.poll_ms, 3);
    }

    #[test]
    fn migrated_extensions_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "extensions".to_string(),
            "target/fret-diag/demo".to_string(),
            "--key".to_string(),
            "a.v1".to_string(),
            "--print".to_string(),
            "--warmup-frames".to_string(),
            "6".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/extensions.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("extensions should be intercepted")
            .expect("extensions should parse");

        let MigratedDiagCommand::Extensions(ctx) = parsed else {
            panic!("expected extensions context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "--key".to_string(),
                "a.v1".to_string(),
                "--print".to_string(),
                "target/fret-diag/demo".to_string(),
            ]
        );
        assert_eq!(ctx.warmup_frames, 6);
        assert!(ctx.stats_json);
        assert_eq!(ctx.out, Some(PathBuf::from("target/extensions.json")));
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
    }

    #[test]
    fn migrated_windows_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "windows".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("windows should be intercepted")
            .expect("windows should parse");

        let MigratedDiagCommand::Windows(ctx) = parsed else {
            panic!("expected windows context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_json);
    }

    #[test]
    fn migrated_dock_routing_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "dock-routing".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("dock-routing should be intercepted")
            .expect("dock-routing should parse");

        let MigratedDiagCommand::DockRouting(ctx) = parsed else {
            panic!("expected dock-routing context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_json);
    }

    #[test]
    fn migrated_dock_graph_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "dock-graph".to_string(),
            "target/fret-diag/demo".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("dock-graph should be intercepted")
            .expect("dock-graph should parse");

        let MigratedDiagCommand::DockGraph(ctx) = parsed else {
            panic!("expected dock-graph context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert!(ctx.stats_json);
    }

    #[test]
    fn migrated_screenshots_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "screenshots".to_string(),
            "target/fret-diag/demo".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("screenshots should be intercepted")
            .expect("screenshots should parse");

        let MigratedDiagCommand::Screenshots(ctx) = parsed else {
            panic!("expected screenshots context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert!(ctx.stats_json);
    }

    #[test]
    fn migrated_hotspots_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "hotspots".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--hotspots-top".to_string(),
            "9".to_string(),
            "--max-depth".to_string(),
            "5".to_string(),
            "--min-bytes".to_string(),
            "4096".to_string(),
            "--force".to_string(),
            "--lite".to_string(),
            "--metric".to_string(),
            "paint".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/hotspots.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("hotspots should be intercepted")
            .expect("hotspots should parse");

        let MigratedDiagCommand::Hotspots(ctx) = parsed else {
            panic!("expected hotspots context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "target/fret-diag/demo".to_string(),
                "--hotspots-top".to_string(),
                "9".to_string(),
                "--max-depth".to_string(),
                "5".to_string(),
                "--min-bytes".to_string(),
                "4096".to_string(),
                "--force".to_string(),
                "--lite".to_string(),
                "--metric".to_string(),
                "paint".to_string(),
            ]
        );
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.hotspots_out,
            Some(PathBuf::from("target/hotspots.json"))
        );
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
    }

    #[test]
    fn migrated_bundle_v2_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "bundle-v2".to_string(),
            "target/fret-diag/demo".to_string(),
            "--mode".to_string(),
            "changed".to_string(),
            "--pretty".to_string(),
            "--force".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/bundle.schema2.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("bundle-v2 should be intercepted")
            .expect("bundle-v2 should parse");

        let MigratedDiagCommand::BundleV2(ctx) = parsed else {
            panic!("expected bundle-v2 context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "target/fret-diag/demo".to_string(),
                "--mode".to_string(),
                "changed".to_string(),
                "--pretty".to_string(),
                "--force".to_string(),
            ]
        );
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.bundle_v2_out,
            Some(PathBuf::from("target/bundle.schema2.json"))
        );
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
    }

    #[test]
    fn migrated_compare_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "compare".to_string(),
            "target/fret-diag/demo-a".to_string(),
            "target/fret-diag/demo-b".to_string(),
            "--footprint".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--compare-eps-px".to_string(),
            "0.25".to_string(),
            "--compare-ignore-bounds".to_string(),
            "--compare-ignore-scene-fingerprint".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("compare should be intercepted")
            .expect("compare should parse");

        let MigratedDiagCommand::Compare(ctx) = parsed else {
            panic!("expected compare context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "target/fret-diag/demo-a".to_string(),
                "target/fret-diag/demo-b".to_string(),
            ]
        );
        assert_eq!(ctx.warmup_frames, 4);
        assert_eq!(ctx.compare_eps_px, 0.25);
        assert!(ctx.compare_ignore_bounds);
        assert!(ctx.compare_ignore_scene_fingerprint);
        assert!(ctx.compare_footprint);
        assert!(ctx.stats_json);
    }

    #[test]
    fn migrated_dashboard_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "dashboard".to_string(),
            "target/fret-diag/campaigns/ui-gallery-smoke/1774499171270".to_string(),
            "--dir".to_string(),
            "target/fret-diag/campaigns/ui-gallery-smoke/1774499171270".to_string(),
            "--top".to_string(),
            "9".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("dashboard should be intercepted")
            .expect("dashboard should parse");

        let MigratedDiagCommand::Dashboard(ctx) = parsed else {
            panic!("expected dashboard context");
        };

        assert_eq!(ctx.top, 9);
        assert!(ctx.stats_json);
        assert!(ctx.source.as_deref().is_some_and(|path| {
            path.ends_with("target/fret-diag/campaigns/ui-gallery-smoke/1774499171270")
        }));
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag/campaigns/ui-gallery-smoke/1774499171270")
        );
    }

    #[test]
    fn migrated_stats_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "stats".to_string(),
            "target/fret-diag/demo".to_string(),
            "--top".to_string(),
            "9".to_string(),
            "--sort".to_string(),
            "time".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--verbose".to_string(),
            "--json".to_string(),
            "--check-hover-layout".to_string(),
            "--check-notify-hotspot-file-max".to_string(),
            "src/view.rs".to_string(),
            "7".to_string(),
            "--check-view-cache-reuse-min".to_string(),
            "1".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("stats should be intercepted")
            .expect("stats should parse");

        let MigratedDiagCommand::Stats(ctx) = parsed else {
            panic!("expected stats context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.stats_top, 9);
        assert_eq!(ctx.sort_override, Some(crate::BundleStatsSort::Time));
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_verbose);
        assert!(ctx.stats_json);
        assert_eq!(ctx.check_hover_layout_max, Some(0));
        assert_eq!(
            ctx.check_notify_hotspot_file_max,
            vec![("src/view.rs".to_string(), 7)]
        );
        assert_eq!(ctx.check_view_cache_reuse_min, Some(1));
    }

    #[test]
    fn migrated_summarize_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "summarize".to_string(),
            "target/fret-diag/campaigns/ui-gallery-smoke/1774499171270".to_string(),
            "target/fret-diag/regression.summary.json".to_string(),
            "--dir".to_string(),
            "target/fret-diag-clap-smoke/summarize".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("summarize should be intercepted")
            .expect("summarize should parse");

        let MigratedDiagCommand::Summarize(ctx) = parsed else {
            panic!("expected summarize context");
        };

        assert_eq!(ctx.inputs.len(), 2);
        assert!(
            ctx.inputs[0].ends_with("target/fret-diag/campaigns/ui-gallery-smoke/1774499171270")
        );
        assert!(ctx.inputs[1].ends_with("target/fret-diag/regression.summary.json"));
        assert!(ctx.stats_json);
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-clap-smoke/summarize")
        );
    }

    #[test]
    fn migrated_trace_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "trace".to_string(),
            "target/fret-diag/demo".to_string(),
            "--trace-out".to_string(),
            "target/trace.chrome.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("trace should be intercepted")
            .expect("trace should parse");

        let MigratedDiagCommand::Trace(ctx) = parsed else {
            panic!("expected trace context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(
            ctx.trace_out,
            Some(PathBuf::from("target/trace.chrome.json"))
        );
    }

    #[test]
    fn migrated_resolve_latest_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "resolve".to_string(),
            "latest".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-resolve".to_string(),
            "--within-session".to_string(),
            "latest".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("resolve latest should be intercepted")
            .expect("resolve latest should parse");

        let MigratedDiagCommand::Resolve(ctx) = parsed else {
            panic!("expected resolve context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "latest".to_string(),
                "--within-session".to_string(),
                "latest".to_string(),
            ]
        );
        assert!(ctx.json);
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-resolve")
        );
    }

    #[test]
    fn migrated_test_ids_index_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "test-ids-index".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("test-ids-index should be intercepted")
            .expect("test-ids-index should parse");

        let MigratedDiagCommand::TestIdsIndex(ctx) = parsed else {
            panic!("expected test-ids-index context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_json);
    }

    #[test]
    fn migrated_frames_index_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "frames-index".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("frames-index should be intercepted")
            .expect("frames-index should parse");

        let MigratedDiagCommand::FramesIndex(ctx) = parsed else {
            panic!("expected frames-index context");
        };

        assert_eq!(ctx.rest, vec!["target/fret-diag/demo".to_string()]);
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_json);
    }

    #[test]
    fn migrated_query_test_id_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "query".to_string(),
            "test-id".to_string(),
            "ui-gallery".to_string(),
            "--warmup-frames".to_string(),
            "4".to_string(),
            "--mode".to_string(),
            "prefix".to_string(),
            "--top".to_string(),
            "8".to_string(),
            "--case-sensitive".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/query.test-id.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("query test-id should be intercepted")
            .expect("query test-id should parse");

        let MigratedDiagCommand::Query(ctx) = parsed else {
            panic!("expected query context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "test-id".to_string(),
                "ui-gallery".to_string(),
                "--mode".to_string(),
                "prefix".to_string(),
                "--top".to_string(),
                "8".to_string(),
                "--case-sensitive".to_string(),
            ]
        );
        assert_eq!(ctx.warmup_frames, 4);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.query_out,
            Some(PathBuf::from("target/query.test-id.json"))
        );
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
    }

    #[test]
    fn migrated_query_snapshots_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "query".to_string(),
            "snapshots".to_string(),
            "target/fret-diag/demo".to_string(),
            "--warmup-frames".to_string(),
            "3".to_string(),
            "--window".to_string(),
            "2".to_string(),
            "--include-warmup".to_string(),
            "--include-missing-semantics".to_string(),
            "--semantics-source".to_string(),
            "table".to_string(),
            "--test-id".to_string(),
            "ui-gallery-root".to_string(),
            "--step-index".to_string(),
            "11".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/query.snapshots.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("query snapshots should be intercepted")
            .expect("query snapshots should parse");

        let MigratedDiagCommand::Query(ctx) = parsed else {
            panic!("expected query context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "snapshots".to_string(),
                "target/fret-diag/demo".to_string(),
                "--window".to_string(),
                "2".to_string(),
                "--include-warmup".to_string(),
                "--include-missing-semantics".to_string(),
                "--semantics-source".to_string(),
                "table".to_string(),
                "--test-id".to_string(),
                "ui-gallery-root".to_string(),
                "--step-index".to_string(),
                "11".to_string(),
            ]
        );
        assert_eq!(ctx.warmup_frames, 3);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.query_out,
            Some(PathBuf::from("target/query.snapshots.json"))
        );
    }

    #[test]
    fn migrated_slice_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "slice".to_string(),
            "target/fret-diag/demo".to_string(),
            "--test-id".to_string(),
            "ui-gallery-root".to_string(),
            "--warmup-frames".to_string(),
            "6".to_string(),
            "--window".to_string(),
            "3".to_string(),
            "--step-index".to_string(),
            "17".to_string(),
            "--max-matches".to_string(),
            "4".to_string(),
            "--max-ancestors".to_string(),
            "9".to_string(),
            "--json".to_string(),
            "--out".to_string(),
            "target/slice.test-id.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("slice should be intercepted")
            .expect("slice should parse");

        let MigratedDiagCommand::Slice(ctx) = parsed else {
            panic!("expected slice context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "target/fret-diag/demo".to_string(),
                "--test-id".to_string(),
                "ui-gallery-root".to_string(),
                "--window".to_string(),
                "3".to_string(),
                "--step-index".to_string(),
                "17".to_string(),
                "--max-matches".to_string(),
                "4".to_string(),
                "--max-ancestors".to_string(),
                "9".to_string(),
            ]
        );
        assert_eq!(ctx.warmup_frames, 6);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.slice_out,
            Some(PathBuf::from("target/slice.test-id.json"))
        );
        assert!(ctx.resolved_out_dir.ends_with("target/fret-diag"));
    }

    #[test]
    fn migrated_perf_subset_builds_a_real_perf_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "perf".to_string(),
            "ui-gallery".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-perf".to_string(),
            "--repeat".to_string(),
            "7".to_string(),
            "--top".to_string(),
            "9".to_string(),
            "--sort".to_string(),
            "time".to_string(),
            "--prewarm-script".to_string(),
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
            "--prelude-script".to_string(),
            "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json".to_string(),
            "--prelude-each-run".to_string(),
            "--perf-threshold-agg".to_string(),
            "p95".to_string(),
            "--max-frame-p95-total-us".to_string(),
            "18000".to_string(),
            "--check-perf-hints".to_string(),
            "--check-perf-hints-min-severity".to_string(),
            "error".to_string(),
            "--check-perf-hints-deny".to_string(),
            "gpu.over-budget".to_string(),
            "--check-pixels-unchanged".to_string(),
            "ui-gallery-root".to_string(),
            "--launch".to_string(),
            "--".to_string(),
            "cargo".to_string(),
            "run".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("perf subset should be intercepted")
            .expect("perf subset should parse");

        let MigratedDiagCommand::Perf(ctx) = parsed else {
            panic!("expected perf context");
        };

        assert_eq!(ctx.rest, vec!["ui-gallery".to_string()]);
        assert_eq!(ctx.perf_repeat, 7);
        assert_eq!(ctx.stats_top, 9);
        assert_eq!(ctx.sort_override, Some(crate::BundleStatsSort::Time));
        assert!(ctx.suite_prelude_each_run);
        assert_eq!(
            ctx.suite_prewarm_scripts,
            vec![PathBuf::from(
                "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json"
            )]
        );
        assert_eq!(
            ctx.suite_prelude_scripts,
            vec![PathBuf::from(
                "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json"
            )]
        );
        assert_eq!(ctx.perf_threshold_agg, crate::PerfThresholdAggregate::P95);
        assert_eq!(ctx.max_frame_p95_total_us, Some(18_000));
        assert!(ctx.check_perf_hints);
        assert_eq!(ctx.check_perf_hints_min_severity.as_deref(), Some("error"));
        assert_eq!(
            ctx.check_perf_hints_deny,
            vec!["gpu.over-budget".to_string()]
        );
        assert_eq!(
            ctx.check_pixels_unchanged_test_id.as_deref(),
            Some("ui-gallery-root")
        );
        assert_eq!(
            ctx.launch,
            Some(vec!["cargo".to_string(), "run".to_string()])
        );
        assert!(
            ctx.launch_env
                .iter()
                .any(|(key, value)| key == "FRET_DIAG_GPU_SCREENSHOTS" && value == "1")
        );
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-perf")
        );
    }

    #[test]
    fn migrated_perf_baseline_from_bundles_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "perf-baseline-from-bundles".to_string(),
            "tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json".to_string(),
            "target/fret-diag/demo-a".to_string(),
            "target/fret-diag/demo-b".to_string(),
            "--sort".to_string(),
            "time".to_string(),
            "--perf-baseline-out".to_string(),
            "target/perf.baseline.json".to_string(),
            "--perf-baseline-headroom-pct".to_string(),
            "25".to_string(),
            "--warmup-frames".to_string(),
            "5".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("perf-baseline-from-bundles should be intercepted")
            .expect("perf-baseline-from-bundles should parse");

        let MigratedDiagCommand::PerfBaselineFromBundles(ctx) = parsed else {
            panic!("expected perf-baseline-from-bundles context");
        };

        assert!(!ctx.pack_after_run);
        assert_eq!(
            ctx.rest,
            vec![
                "tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json".to_string(),
                "target/fret-diag/demo-a".to_string(),
                "target/fret-diag/demo-b".to_string(),
            ]
        );
        assert_eq!(ctx.sort_override, Some(crate::BundleStatsSort::Time));
        assert_eq!(
            ctx.perf_baseline_out,
            Some(PathBuf::from("target/perf.baseline.json"))
        );
        assert_eq!(ctx.perf_baseline_headroom_pct, 25);
        assert_eq!(ctx.warmup_frames, 5);
        assert!(ctx.stats_json);
    }

    #[test]
    fn migrated_matrix_builds_a_real_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "matrix".to_string(),
            "ui-gallery".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-matrix".to_string(),
            "--timeout-ms".to_string(),
            "9".to_string(),
            "--poll-ms".to_string(),
            "3".to_string(),
            "--warmup-frames".to_string(),
            "5".to_string(),
            "--compare-ignore-bounds".to_string(),
            "--check-view-cache-reuse-min".to_string(),
            "1".to_string(),
            "--check-view-cache-reuse-stable-min".to_string(),
            "2".to_string(),
            "--check-overlay-synthesis-min".to_string(),
            "3".to_string(),
            "--check-viewport-input-min".to_string(),
            "4".to_string(),
            "--env".to_string(),
            "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1".to_string(),
            "--json".to_string(),
            "--launch-high-priority".to_string(),
            "--launch".to_string(),
            "--".to_string(),
            "cargo".to_string(),
            "run".to_string(),
            "-p".to_string(),
            "fret-ui-gallery".to_string(),
            "--release".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("matrix should be intercepted")
            .expect("matrix should parse");

        let MigratedDiagCommand::Matrix(ctx) = parsed else {
            panic!("expected matrix context");
        };

        assert_eq!(ctx.rest, vec!["ui-gallery".to_string()]);
        assert_eq!(
            ctx.launch,
            Some(vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "fret-ui-gallery".to_string(),
                "--release".to_string(),
            ])
        );
        assert_eq!(
            ctx.launch_env,
            vec![(
                "FRET_UI_GALLERY_VIEW_CACHE_SHELL".to_string(),
                "1".to_string(),
            )]
        );
        assert!(ctx.launch_high_priority);
        assert_eq!(ctx.timeout_ms, 9);
        assert_eq!(ctx.poll_ms, 3);
        assert_eq!(ctx.warmup_frames, 5);
        assert!(ctx.compare_ignore_bounds);
        assert_eq!(ctx.check_view_cache_reuse_min, Some(1));
        assert_eq!(ctx.check_view_cache_reuse_stable_min, Some(2));
        assert_eq!(ctx.check_overlay_synthesis_min, Some(3));
        assert_eq!(ctx.check_viewport_input_min, Some(4));
        assert!(ctx.stats_json);
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-matrix")
        );
    }

    #[test]
    fn migrated_suite_subset_builds_a_real_suite_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "suite".to_string(),
            "ui-gallery".to_string(),
            "--script-dir".to_string(),
            "tools/diag-scripts/ui-gallery/data_table".to_string(),
            "--glob".to_string(),
            "tools/diag-scripts/ui-gallery-select-*.json".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-suite".to_string(),
            "--prelude-script".to_string(),
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("suite subset should be intercepted")
            .expect("suite subset should parse");

        let MigratedDiagCommand::Suite(ctx) = parsed else {
            panic!("expected suite context");
        };

        assert_eq!(ctx.rest, vec!["ui-gallery".to_string()]);
        assert_eq!(
            ctx.suite_script_inputs,
            vec![
                "tools/diag-scripts/ui-gallery/data_table".to_string(),
                "tools/diag-scripts/ui-gallery-select-*.json".to_string(),
            ]
        );
        assert_eq!(
            ctx.suite_prelude_scripts,
            vec![PathBuf::from(
                "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json"
            )]
        );
        assert!(
            ctx.resolved_paths
                .out_dir
                .ends_with("target/fret-diag-cutover-suite")
        );
    }

    #[test]
    fn migrated_suite_rejects_missing_suite_and_script_inputs() {
        let workspace_root = workspace_root_for_tests();
        let err = match maybe_parse_migrated_command_with_workspace(
            &[
                "suite".to_string(),
                "--timeout-ms".to_string(),
                "1".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("empty suite input should be rejected"),
            None => panic!("suite should stay on migrated parser"),
        };
        assert!(err.contains("missing suite/script input"));
    }

    #[test]
    fn migrated_suite_rejects_devtools_transport_with_launch_or_reuse_launch() {
        let workspace_root = workspace_root_for_tests();
        let err = match maybe_parse_migrated_command_with_workspace(
            &[
                "suite".to_string(),
                "ui-gallery".to_string(),
                "--devtools-ws-url".to_string(),
                "ws://127.0.0.1:7331/".to_string(),
                "--devtools-token".to_string(),
                "secret".to_string(),
                "--reuse-launch".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("devtools + reuse-launch should be rejected"),
            None => panic!("suite should stay on migrated parser"),
        };
        assert!(err.contains("--launch/--reuse-launch is not supported with --devtools-ws-url"));
    }

    #[test]
    fn migrated_repeat_subset_builds_a_real_repeat_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "repeat".to_string(),
            "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-repeat".to_string(),
            "--repeat".to_string(),
            "7".to_string(),
            "--no-compare".to_string(),
            "--check-memory-p90-max".to_string(),
            "rss:1024".to_string(),
            "--compare-ignore-bounds".to_string(),
            "--launch".to_string(),
            "--".to_string(),
            "cargo".to_string(),
            "run".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("repeat subset should be intercepted")
            .expect("repeat subset should parse");

        let MigratedDiagCommand::Repeat(ctx) = parsed else {
            panic!("expected repeat context");
        };

        assert_eq!(
            ctx.rest,
            vec!["tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json".to_string()]
        );
        assert_eq!(ctx.perf_repeat, 7);
        assert!(!ctx.compare_enabled);
        assert!(ctx.compare_ignore_bounds);
        assert_eq!(ctx.check_memory_p90_max, vec![("rss".to_string(), 1024)]);
        assert_eq!(
            ctx.launch,
            Some(vec!["cargo".to_string(), "run".to_string()])
        );
        assert!(
            ctx.resolved_paths
                .out_dir
                .ends_with("target/fret-diag-cutover-repeat")
        );
    }

    #[test]
    fn migrated_repeat_rejects_zero_repeat_count() {
        let workspace_root = workspace_root_for_tests();
        let err = match maybe_parse_migrated_command_with_workspace(
            &[
                "repeat".to_string(),
                "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json".to_string(),
                "--repeat".to_string(),
                "0".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("repeat=0 should be rejected"),
            None => panic!("repeat should stay on migrated parser"),
        };
        assert!(err.contains("--repeat"));
    }

    #[test]
    fn migrated_repro_subset_builds_a_real_repro_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "repro".to_string(),
            "ui-gallery".to_string(),
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-repro".to_string(),
            "--ai-only".to_string(),
            "--include-screenshots".to_string(),
            "--trace-chrome".to_string(),
            "--launch".to_string(),
            "--".to_string(),
            "cargo".to_string(),
            "run".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("repro subset should be intercepted")
            .expect("repro subset should parse");

        let MigratedDiagCommand::Repro(ctx) = parsed else {
            panic!("expected repro context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "ui-gallery".to_string(),
                "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
            ]
        );
        assert!(ctx.pack_ai_only);
        assert!(ctx.ensure_ai_packet);
        assert!(ctx.pack_include_screenshots);
        assert!(ctx.trace_chrome);
        assert_eq!(
            ctx.launch,
            Some(vec!["cargo".to_string(), "run".to_string()])
        );
        assert!(
            ctx.launch_env
                .iter()
                .any(|(key, value)| key == "FRET_DIAG_GPU_SCREENSHOTS" && value == "1")
        );
        assert!(
            ctx.resolved_run_context
                .paths
                .out_dir
                .ends_with("target/fret-diag-cutover-repro")
        );
    }

    #[test]
    fn migrated_script_direct_builds_a_real_script_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "script".to_string(),
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-script".to_string(),
            "--script-path".to_string(),
            "target/custom-script.json".to_string(),
            "--script-trigger-path".to_string(),
            "target/custom-script.touch".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("script direct should be intercepted")
            .expect("script direct should parse");

        let MigratedDiagCommand::Script(ctx) = parsed else {
            panic!("expected script context");
        };

        assert_eq!(
            ctx.rest,
            vec!["tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string()]
        );
        assert!(!ctx.script_tool_check);
        assert!(!ctx.script_tool_write);
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-script")
        );
        assert!(
            ctx.resolved_run_context
                .paths
                .script_path
                .ends_with("target/custom-script.json")
        );
        assert!(
            ctx.resolved_run_context
                .paths
                .script_trigger_path
                .ends_with("target/custom-script.touch")
        );
    }

    #[test]
    fn migrated_script_validate_builds_a_real_script_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "script".to_string(),
            "validate".to_string(),
            "tools/diag-scripts".to_string(),
            "tools/diag-scripts/ui-gallery-select-*.json".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-script-validate".to_string(),
            "--check-out".to_string(),
            "target/check.script_schema.json".to_string(),
            "--json".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("script validate should be intercepted")
            .expect("script validate should parse");

        let MigratedDiagCommand::Script(ctx) = parsed else {
            panic!("expected script context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "validate".to_string(),
                "tools/diag-scripts".to_string(),
                "tools/diag-scripts/ui-gallery-select-*.json".to_string(),
            ]
        );
        assert_eq!(
            ctx.script_tool_check_out,
            Some(PathBuf::from("target/check.script_schema.json"))
        );
        assert!(ctx.stats_json);
        assert!(
            ctx.resolved_out_dir
                .ends_with("target/fret-diag-cutover-script-validate")
        );
    }

    #[test]
    fn migrated_script_shrink_builds_a_real_script_context() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "script".to_string(),
            "shrink".to_string(),
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
            "--dir".to_string(),
            "target/fret-diag-cutover-script-shrink".to_string(),
            "--timeout-ms".to_string(),
            "9".to_string(),
            "--poll-ms".to_string(),
            "3".to_string(),
            "--session-auto".to_string(),
            "--shrink-out".to_string(),
            "target/shrink.min.json".to_string(),
            "--shrink-any-fail".to_string(),
            "--shrink-match-reason-code".to_string(),
            "timeout".to_string(),
            "--shrink-min-steps".to_string(),
            "2".to_string(),
            "--shrink-max-iters".to_string(),
            "11".to_string(),
            "--json".to_string(),
            "--env".to_string(),
            "FRET_UI_GALLERY_VIEW_CACHE=1".to_string(),
            "--launch".to_string(),
            "--".to_string(),
            "cargo".to_string(),
            "run".to_string(),
            "-p".to_string(),
            "fret-ui-gallery".to_string(),
        ];

        let parsed = maybe_parse_migrated_command_with_workspace(&args, &workspace_root)
            .expect("script shrink should be intercepted")
            .expect("script shrink should parse");

        let MigratedDiagCommand::Script(ctx) = parsed else {
            panic!("expected script context");
        };

        assert_eq!(
            ctx.rest,
            vec![
                "shrink".to_string(),
                "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
            ]
        );
        assert_eq!(ctx.timeout_ms, 9);
        assert_eq!(ctx.poll_ms, 3);
        assert_eq!(
            ctx.shrink_out,
            Some(PathBuf::from("target/shrink.min.json"))
        );
        assert!(ctx.shrink_any_fail);
        assert_eq!(ctx.shrink_match_reason_code.as_deref(), Some("timeout"));
        assert_eq!(ctx.shrink_min_steps, 2);
        assert_eq!(ctx.shrink_max_iters, 11);
        assert!(ctx.stats_json);
        assert_eq!(
            ctx.launch,
            Some(vec![
                "cargo".to_string(),
                "run".to_string(),
                "-p".to_string(),
                "fret-ui-gallery".to_string(),
            ])
        );
        assert_eq!(
            ctx.launch_env,
            vec![("FRET_UI_GALLERY_VIEW_CACHE".to_string(), "1".to_string())]
        );
        assert!(
            ctx.resolved_out_dir
                .components()
                .any(|component| component.as_os_str() == "sessions")
        );
    }

    #[test]
    fn unsupported_script_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "script".to_string(),
            "shrink".to_string(),
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
            "--reuse-launch".to_string(),
        ];

        let err = match maybe_parse_migrated_command_with_workspace(&args, &workspace_root) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated script flags should be rejected"),
            None => panic!("script should stay on migrated parser"),
        };
        assert!(err.contains("--reuse-launch"));
    }

    #[test]
    fn unsupported_perf_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "perf".to_string(),
            "ui-gallery".to_string(),
            "--check-stale-paint".to_string(),
            "ui-gallery-root".to_string(),
        ];

        let err = match maybe_parse_migrated_command_with_workspace(&args, &workspace_root) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated perf flags should be rejected"),
            None => panic!("perf should stay on migrated parser"),
        };
        assert!(err.contains("--check-stale-paint"));
    }

    #[test]
    fn unsupported_campaign_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "campaign".to_string(),
            "run".to_string(),
            "--with".to_string(),
            "tracy".to_string(),
        ];

        let err = match maybe_parse_migrated_command_with_workspace(&args, &workspace_root) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported campaign flags should be rejected"),
            None => panic!("campaign should stay on migrated parser"),
        };
        assert!(err.contains("--with"));
    }

    #[test]
    fn unsupported_list_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "list".to_string(),
            "scripts".to_string(),
            "--dir".to_string(),
            "target/fret-diag".to_string(),
        ];

        let err = match maybe_parse_migrated_command_with_workspace(&args, &workspace_root) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated list flags should be rejected"),
            None => panic!("list should stay on migrated parser"),
        };
        assert!(err.contains("--dir"));
    }

    #[test]
    fn unsupported_doctor_nested_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "doctor".to_string(),
            "campaigns".to_string(),
            "--fix".to_string(),
        ];

        let err = match maybe_parse_migrated_command_with_workspace(&args, &workspace_root) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated doctor flags should be rejected"),
            None => panic!("doctor should stay on migrated parser"),
        };
        assert!(err.contains("--fix"));
    }

    #[test]
    fn unsupported_meta_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "meta".to_string(),
            "target/fret-diag/demo".to_string(),
            "--dir".to_string(),
            "target/fret-diag".to_string(),
        ];

        let err = match maybe_parse_migrated_command_with_workspace(&args, &workspace_root) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated meta flags should be rejected"),
            None => panic!("meta should stay on migrated parser"),
        };
        assert!(err.contains("--dir"));
    }

    #[test]
    fn unsupported_trace_resolve_and_index_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();

        let trace_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "trace".to_string(),
                "target/fret-diag/demo".to_string(),
                "--json".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated trace flags should be rejected"),
            None => panic!("trace should stay on migrated parser"),
        };
        assert!(trace_err.contains("--json"));

        let resolve_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "resolve".to_string(),
                "latest".to_string(),
                "--warmup-frames".to_string(),
                "4".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated resolve flags should be rejected"),
            None => panic!("resolve should stay on migrated parser"),
        };
        assert!(resolve_err.contains("--warmup-frames"));

        let test_ids_index_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "test-ids-index".to_string(),
                "target/fret-diag/demo".to_string(),
                "--out".to_string(),
                "target/test_ids.index.json".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated test-ids-index flags should be rejected"),
            None => panic!("test-ids-index should stay on migrated parser"),
        };
        assert!(test_ids_index_err.contains("--out"));

        let frames_index_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "frames-index".to_string(),
                "target/fret-diag/demo".to_string(),
                "--out".to_string(),
                "target/frames.index.json".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated frames-index flags should be rejected"),
            None => panic!("frames-index should stay on migrated parser"),
        };
        assert!(frames_index_err.contains("--out"));
    }

    #[test]
    fn unsupported_compare_dashboard_stats_and_summarize_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();

        let compare_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "compare".to_string(),
                "target/fret-diag/demo-a".to_string(),
                "target/fret-diag/demo-b".to_string(),
                "--compare-footprint".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported compare flags should be rejected"),
            None => panic!("compare should stay on migrated parser"),
        };
        assert!(compare_err.contains("--compare-footprint"));

        let dashboard_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "dashboard".to_string(),
                "--warmup-frames".to_string(),
                "4".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported dashboard flags should be rejected"),
            None => panic!("dashboard should stay on migrated parser"),
        };
        assert!(dashboard_err.contains("--warmup-frames"));

        let stats_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "stats".to_string(),
                "target/fret-diag/demo".to_string(),
                "--check-prepaint-actions-min".to_string(),
                "1".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported stats flags should be rejected"),
            None => panic!("stats should stay on migrated parser"),
        };
        assert!(stats_err.contains("--check-prepaint-actions-min"));

        let summarize_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "summarize".to_string(),
                "--top".to_string(),
                "7".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported summarize flags should be rejected"),
            None => panic!("summarize should stay on migrated parser"),
        };
        assert!(summarize_err.contains("--top"));
    }

    #[test]
    fn unsupported_legacy_layout_memory_and_pick_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();

        let layout_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "layout-perf-summary".to_string(),
                "target/fret-diag/demo".to_string(),
                "--sort-key".to_string(),
                "layout_time_us".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported layout-perf-summary flags should be rejected"),
            None => panic!("layout-perf-summary should stay on migrated parser"),
        };
        assert!(layout_err.contains("--sort-key"));

        let memory_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "memory-summary".to_string(),
                "--sort_key".to_string(),
                "macos_physical_footprint_peak_bytes".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported memory-summary aliases should be rejected"),
            None => panic!("memory-summary should stay on migrated parser"),
        };
        assert!(memory_err.contains("--sort_key"));

        let pick_script_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "pick-script".to_string(),
                "--warmup-frames".to_string(),
                "4".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported pick-script flags should be rejected"),
            None => panic!("pick-script should stay on migrated parser"),
        };
        assert!(pick_script_err.contains("--warmup-frames"));
    }

    #[test]
    fn unsupported_sessions_matrix_and_perf_baseline_flags_are_rejected() {
        let workspace_root = workspace_root_for_tests();

        let sessions_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "sessions".to_string(),
                "clean".to_string(),
                "--keep".to_string(),
                "1".to_string(),
                "--top".to_string(),
                "0".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("invalid sessions clean flags should be rejected"),
            None => panic!("sessions clean should stay on migrated parser"),
        };
        assert!(sessions_err.contains("--top must be >= 1"));

        let matrix_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "matrix".to_string(),
                "ui-gallery".to_string(),
                "--launch-write-bundle-json".to_string(),
                "--launch".to_string(),
                "--".to_string(),
                "cargo".to_string(),
                "run".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported matrix flags should be rejected"),
            None => panic!("matrix should stay on migrated parser"),
        };
        assert!(matrix_err.contains("--launch-write-bundle-json"));

        let perf_baseline_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "perf-baseline-from-bundles".to_string(),
                "tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json".to_string(),
                "target/fret-diag/demo".to_string(),
                "--pack".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported perf baseline flags should be rejected"),
            None => panic!("perf-baseline-from-bundles should stay on migrated parser"),
        };
        assert!(perf_baseline_err.contains("--pack"));
    }

    #[test]
    fn unsupported_artifact_helper_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();

        let windows_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "windows".to_string(),
                "target/fret-diag/demo".to_string(),
                "--out".to_string(),
                "target/window.map.json".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated windows flags should be rejected"),
            None => panic!("windows should stay on migrated parser"),
        };
        assert!(windows_err.contains("--out"));

        let dock_graph_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "dock-graph".to_string(),
                "target/fret-diag/demo".to_string(),
                "--warmup-frames".to_string(),
                "4".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated dock-graph flags should be rejected"),
            None => panic!("dock-graph should stay on migrated parser"),
        };
        assert!(dock_graph_err.contains("--warmup-frames"));

        let screenshots_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "screenshots".to_string(),
                "target/fret-diag/demo".to_string(),
                "--warmup-frames".to_string(),
                "4".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated screenshots flags should be rejected"),
            None => panic!("screenshots should stay on migrated parser"),
        };
        assert!(screenshots_err.contains("--warmup-frames"));

        let hotspots_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "hotspots".to_string(),
                "target/fret-diag/demo".to_string(),
                "--top".to_string(),
                "9".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated hotspots flags should be rejected"),
            None => panic!("hotspots should stay on migrated parser"),
        };
        assert!(hotspots_err.contains("--top"));

        let bundle_v2_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "bundle-v2".to_string(),
                "target/fret-diag/demo".to_string(),
                "--warmup-frames".to_string(),
                "4".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated bundle-v2 flags should be rejected"),
            None => panic!("bundle-v2 should stay on migrated parser"),
        };
        assert!(bundle_v2_err.contains("--warmup-frames"));
    }

    #[test]
    fn migrated_parser_errors_reference_full_diag_command_path() {
        let workspace_root = workspace_root_for_tests();

        let windows_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "windows".to_string(),
                "target/fret-diag/demo".to_string(),
                "--out".to_string(),
                "target/window.map.json".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported migrated windows flags should be rejected"),
            None => panic!("windows should stay on migrated parser"),
        };
        assert!(windows_err.contains("Usage: fretboard diag windows"));
    }

    #[test]
    fn unsupported_pack_triage_and_ai_packet_legacy_aliases_are_rejected() {
        let workspace_root = workspace_root_for_tests();

        let pack_err = match maybe_parse_migrated_command_with_workspace(
            &["pack".to_string(), "--schema2-only".to_string()],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("retired pack alias should be rejected"),
            None => panic!("pack should stay on migrated parser"),
        };
        assert!(pack_err.contains("--pack-schema2-only"));

        let triage_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "triage".to_string(),
                "target/fret-diag/demo".to_string(),
                "--frames-index".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("retired triage alias should be rejected"),
            None => panic!("triage should stay on migrated parser"),
        };
        assert!(triage_err.contains("diag triage --lite"));

        let triage_from_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "triage".to_string(),
                "target/fret-diag/demo".to_string(),
                "--from-frames-index".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("retired triage alias should be rejected"),
            None => panic!("triage should stay on migrated parser"),
        };
        assert!(triage_from_err.contains("diag triage --lite"));

        let ai_packet_err = match maybe_parse_migrated_command_with_workspace(
            &["ai-packet".to_string(), "ui-gallery-root".to_string()],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("positional ai-packet test-id should be rejected"),
            None => panic!("ai-packet should stay on migrated parser"),
        };
        assert!(ai_packet_err.contains("diag ai-packet --test-id"));
    }

    #[test]
    fn unsupported_query_and_slice_flags_are_rejected_by_migrated_parser() {
        let workspace_root = workspace_root_for_tests();

        let query_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "query".to_string(),
                "overlay-placement-trace".to_string(),
                "--anchor".to_string(),
                "trigger".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported query alias flags should be rejected"),
            None => panic!("query should stay on migrated parser"),
        };
        assert!(query_err.contains("--anchor"));

        let slice_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "slice".to_string(),
                "--test-id".to_string(),
                "ui-gallery-root".to_string(),
                "--frame-id".to_string(),
                "3".to_string(),
                "--snapshot-seq".to_string(),
                "4".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("conflicting slice selectors should be rejected"),
            None => panic!("slice should stay on migrated parser"),
        };
        assert!(slice_err.contains("--snapshot-seq"));
    }

    #[test]
    fn retired_aliases_are_rejected_without_falling_back() {
        let workspace_root = workspace_root_for_tests();

        let artifacts_err = match maybe_parse_migrated_command_with_workspace(
            &["artifacts".to_string(), "lint".to_string()],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("retired alias should be rejected"),
            None => panic!("retired alias should be intercepted"),
        };
        assert!(artifacts_err.contains("diag artifact"));

        let layout_err = match maybe_parse_migrated_command_with_workspace(
            &["layout_sidecar".to_string(), "--help".to_string()],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("retired alias should be rejected"),
            None => panic!("retired alias should be intercepted"),
        };
        assert!(layout_err.contains("layout-sidecar"));

        let layout_perf_summary_err = match maybe_parse_migrated_command_with_workspace(
            &["layout_perf_summary".to_string(), "--help".to_string()],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("retired alias should be rejected"),
            None => panic!("retired alias should be intercepted"),
        };
        assert!(layout_perf_summary_err.contains("layout-perf-summary"));

        let memory_summary_err = match maybe_parse_migrated_command_with_workspace(
            &["memory_summary".to_string(), "--help".to_string()],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("retired alias should be rejected"),
            None => panic!("retired alias should be intercepted"),
        };
        assert!(memory_summary_err.contains("memory-summary"));

        let query_err = match maybe_parse_migrated_command_with_workspace(
            &[
                "query".to_string(),
                "test_ids".to_string(),
                "ui-gallery".to_string(),
            ],
            &workspace_root,
        ) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("retired query alias should be rejected"),
            None => panic!("retired query alias should be intercepted"),
        };
        assert!(query_err.contains("query test-id"));
    }

    #[test]
    fn unsupported_run_or_suite_flags_are_rejected() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "suite".to_string(),
            "ui-gallery".to_string(),
            "--suite-prewarm".to_string(),
            "tools/diag-scripts/ui-gallery-intro-idle-screenshot.json".to_string(),
        ];

        let err = match maybe_parse_migrated_command_with_workspace(&args, &workspace_root) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported suite flags should be rejected"),
            None => panic!("suite should stay on migrated parser"),
        };
        assert!(err.contains("--suite-prewarm"));
    }

    #[test]
    fn unsupported_repeat_flags_are_rejected() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "repeat".to_string(),
            "tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json".to_string(),
            "--pack".to_string(),
        ];

        let err = match maybe_parse_migrated_command_with_workspace(&args, &workspace_root) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported repeat flags should be rejected"),
            None => panic!("repeat should stay on migrated parser"),
        };
        assert!(err.contains("--pack"));
    }

    #[test]
    fn unsupported_repro_flags_are_rejected() {
        let workspace_root = workspace_root_for_tests();
        let args = vec![
            "repro".to_string(),
            "ui-gallery".to_string(),
            "--with".to_string(),
            "tracy".to_string(),
        ];

        let err = match maybe_parse_migrated_command_with_workspace(&args, &workspace_root) {
            Some(Err(err)) => err,
            Some(Ok(_)) => panic!("unsupported repro flags should be rejected"),
            None => panic!("repro should stay on migrated parser"),
        };
        assert!(err.contains("--with"));
    }
}
