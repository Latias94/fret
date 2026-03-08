use super::*;

use crate::registry::campaigns::{
    CampaignDefinition, CampaignFilterOptions, CampaignItemDefinition, CampaignItemKind,
    CampaignRegistry, campaign_to_json, item_kind_str, lane_to_str, parse_lane, source_kind_str,
};
use crate::regression_summary::{RegressionItemSummaryV1, RegressionStatusV1, RegressionSummaryV1};

const DIAG_CAMPAIGN_MANIFEST_KIND_V1: &str = "diag_campaign_manifest";
const DIAG_CAMPAIGN_RESULT_KIND_V1: &str = "diag_campaign_result";
const DIAG_CAMPAIGN_BATCH_MANIFEST_KIND_V1: &str = "diag_campaign_batch_manifest";
const DIAG_CAMPAIGN_BATCH_RESULT_KIND_V1: &str = "diag_campaign_batch_result";
const DIAG_CAMPAIGN_SHARE_MANIFEST_KIND_V1: &str = "diag_campaign_share_manifest";

#[derive(Debug, Clone)]
pub(crate) struct CampaignCmdContext {
    pub pack_after_run: bool,
    pub rest: Vec<String>,
    pub suite_script_inputs: Vec<String>,
    pub suite_prewarm_scripts: Vec<PathBuf>,
    pub suite_prelude_scripts: Vec<PathBuf>,
    pub suite_prelude_each_run: bool,
    pub workspace_root: PathBuf,
    pub resolved_out_dir: PathBuf,
    pub devtools_ws_url: Option<String>,
    pub devtools_token: Option<String>,
    pub devtools_session_id: Option<String>,
    pub timeout_ms: u64,
    pub poll_ms: u64,
    pub stats_top: usize,
    pub stats_json: bool,
    pub warmup_frames: u64,
    pub max_test_ids: usize,
    pub lint_all_test_ids_bounds: bool,
    pub lint_eps_px: f32,
    pub suite_lint: bool,
    pub pack_include_screenshots: bool,
    pub reuse_launch: bool,
    pub launch: Option<Vec<String>>,
    pub launch_env: Vec<(String, String)>,
    pub launch_high_priority: bool,
    pub launch_write_bundle_json: bool,
    pub keep_open: bool,
    pub checks: diag_suite::SuiteChecks,
}

#[derive(Debug, Clone, Default)]
struct CampaignRunOptions {
    campaign_ids: Vec<String>,
    filter: CampaignFilterOptions,
}

#[derive(Debug, Clone)]
struct CampaignExecutionReport {
    campaign_id: String,
    out_dir: PathBuf,
    aggregate: CampaignAggregateArtifacts,
    items_total: usize,
    items_failed: usize,
    suites_total: usize,
    scripts_total: usize,
    ok: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct CampaignExecutionOutcome {
    items_failed: usize,
    error: Option<String>,
    share_manifest_path: Option<PathBuf>,
    share_error: Option<String>,
}

#[derive(Debug, Clone)]
struct CampaignExecutionFinalization {
    items_failed: usize,
    aggregate: CampaignAggregateArtifacts,
}

#[derive(Debug, Clone)]
struct CampaignExecutionFinalizePlan {
    items_failed: usize,
    summary_finalize: CampaignSummaryFinalizePlan,
}

#[derive(Debug, Clone)]
struct CampaignSummaryArtifacts {
    finished_unix_ms: u64,
    duration_ms: u64,
    summarize_error: Option<String>,
    share_manifest_path: Option<PathBuf>,
    share_error: Option<String>,
}

#[derive(Debug, Clone)]
struct CampaignSummaryFinalizePlan {
    summarize_inputs: Vec<String>,
    out_dir: PathBuf,
    summary_path: PathBuf,
    created_unix_ms: u64,
    should_generate_share_manifest: bool,
}

#[derive(Debug, Clone)]
struct CampaignSummaryFinalizeOutcome {
    summarize_error: Option<String>,
    share_manifest_path: Option<PathBuf>,
    share_error: Option<String>,
}

#[derive(Debug, Clone)]
struct CampaignJsonWritePlan {
    output_path: PathBuf,
    payload: serde_json::Value,
}

#[derive(Debug, Clone)]
struct CampaignExecutionStartPlan {
    execution: CampaignExecutionPlan,
    manifest_write: CampaignJsonWritePlan,
}

#[derive(Debug, Clone)]
struct CampaignBatchArtifactWritePlan {
    batch: CampaignBatchPlan,
    manifest_write: CampaignJsonWritePlan,
    summary_finalize: CampaignSummaryFinalizePlan,
}

#[derive(Debug, Clone)]
struct CampaignResultPayloadSections {
    run: serde_json::Value,
    counters: serde_json::Value,
    aggregate: serde_json::Value,
    item_results: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct CampaignBatchResultPayloadSections {
    selection: serde_json::Value,
    run: serde_json::Value,
    counters: serde_json::Value,
    aggregate: serde_json::Value,
    runs: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct CampaignBatchArtifacts {
    batch_root: PathBuf,
    aggregate: CampaignAggregateArtifacts,
}

#[derive(Debug, Clone)]
struct CampaignAggregateArtifacts {
    summary_path: PathBuf,
    index_path: PathBuf,
    share_manifest_path: Option<PathBuf>,
    summarize_error: Option<String>,
    share_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CampaignAggregatePathProjection {
    summary_path: Option<String>,
    index_path: Option<String>,
    share_manifest_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct CampaignRunCounters {
    campaigns_total: usize,
    campaigns_failed: usize,
    campaigns_passed: usize,
    items_total: usize,
    items_failed: usize,
    suites_total: usize,
    scripts_total: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct CampaignItemRunCounters {
    items_total: usize,
    items_passed: usize,
    items_failed: usize,
    suites_total: usize,
    suites_failed: usize,
    scripts_total: usize,
    scripts_failed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CampaignReportPathMode {
    RunOutcome,
    ResultArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CampaignSubcommand {
    List,
    Show,
    Share,
    Run,
}

#[derive(Debug, Clone)]
struct CampaignRunOutcome {
    reports: Vec<CampaignExecutionReport>,
    batch: Option<CampaignBatchArtifacts>,
    counters: CampaignRunCounters,
    command_failures: Vec<String>,
}

#[derive(Debug, Clone)]
struct CampaignExecutionPlan {
    created_unix_ms: u64,
    run_id: String,
    campaign_root: PathBuf,
    suite_results_root: PathBuf,
    script_results_root: PathBuf,
    summary_path: PathBuf,
    index_path: PathBuf,
}

#[derive(Debug, Clone)]
struct CampaignBatchPlan {
    created_unix_ms: u64,
    run_id: String,
    batch_root: PathBuf,
    summary_path: PathBuf,
    index_path: PathBuf,
}

#[derive(Debug, Clone)]
struct CampaignShareOptions {
    source: String,
    include_passed: bool,
}

#[derive(Debug, Clone)]
struct CampaignShareSelection {
    summary_path: PathBuf,
    root_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct CampaignRunContext {
    pack_after_run: bool,
    suite_script_inputs: Vec<String>,
    suite_prewarm_scripts: Vec<PathBuf>,
    suite_prelude_scripts: Vec<PathBuf>,
    suite_prelude_each_run: bool,
    workspace_root: PathBuf,
    resolved_out_dir: PathBuf,
    devtools_ws_url: Option<String>,
    devtools_token: Option<String>,
    devtools_session_id: Option<String>,
    timeout_ms: u64,
    poll_ms: u64,
    stats_top: usize,
    stats_json: bool,
    warmup_frames: u64,
    max_test_ids: usize,
    lint_all_test_ids_bounds: bool,
    lint_eps_px: f32,
    suite_lint: bool,
    pack_include_screenshots: bool,
    reuse_launch: bool,
    launch: Option<Vec<String>>,
    launch_env: Vec<(String, String)>,
    launch_high_priority: bool,
    launch_write_bundle_json: bool,
    keep_open: bool,
    checks: diag_suite::SuiteChecks,
}

#[derive(Debug, Clone)]
struct CampaignItemRunResult {
    kind: CampaignItemKind,
    item_id: String,
    out_dir: PathBuf,
    regression_summary_path: PathBuf,
    ok: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct CampaignItemExecutionPlan {
    kind: CampaignItemKind,
    item_id: String,
    out_dir: PathBuf,
    regression_summary_path: PathBuf,
    suite_rest: Vec<String>,
    suite_script_inputs: Vec<String>,
}

impl From<CampaignCmdContext> for CampaignRunContext {
    fn from(ctx: CampaignCmdContext) -> Self {
        let CampaignCmdContext {
            pack_after_run,
            rest: _,
            suite_script_inputs,
            suite_prewarm_scripts,
            suite_prelude_scripts,
            suite_prelude_each_run,
            workspace_root,
            resolved_out_dir,
            devtools_ws_url,
            devtools_token,
            devtools_session_id,
            timeout_ms,
            poll_ms,
            stats_top,
            stats_json,
            warmup_frames,
            max_test_ids,
            lint_all_test_ids_bounds,
            lint_eps_px,
            suite_lint,
            pack_include_screenshots,
            reuse_launch,
            launch,
            launch_env,
            launch_high_priority,
            launch_write_bundle_json,
            keep_open,
            checks,
        } = ctx;
        Self {
            pack_after_run,
            suite_script_inputs,
            suite_prewarm_scripts,
            suite_prelude_scripts,
            suite_prelude_each_run,
            workspace_root,
            resolved_out_dir,
            devtools_ws_url,
            devtools_token,
            devtools_session_id,
            timeout_ms,
            poll_ms,
            stats_top,
            stats_json,
            warmup_frames,
            max_test_ids,
            lint_all_test_ids_bounds,
            lint_eps_px,
            suite_lint,
            pack_include_screenshots,
            reuse_launch,
            launch,
            launch_env,
            launch_high_priority,
            launch_write_bundle_json,
            keep_open,
            checks,
        }
    }
}

fn build_campaign_execution_plan_at(
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
    created_unix_ms: u64,
) -> CampaignExecutionPlan {
    let run_id = created_unix_ms.to_string();
    let campaign_root = ctx
        .resolved_out_dir
        .join("campaigns")
        .join(zip_safe_component(&campaign.id))
        .join(&run_id);
    let summary_path =
        campaign_root.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    let index_path =
        campaign_root.join(crate::regression_summary::DIAG_REGRESSION_INDEX_FILENAME_V1);
    let suite_results_root = campaign_root.join("suite-results");
    let script_results_root = campaign_root.join("script-results");
    CampaignExecutionPlan {
        created_unix_ms,
        run_id,
        campaign_root,
        suite_results_root,
        script_results_root,
        summary_path,
        index_path,
    }
}

fn build_campaign_execution_plan(
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
) -> CampaignExecutionPlan {
    build_campaign_execution_plan_at(campaign, ctx, now_unix_ms())
}

fn ensure_campaign_execution_dirs(plan: &CampaignExecutionPlan) -> Result<(), String> {
    std::fs::create_dir_all(&plan.suite_results_root).map_err(|e| {
        format!(
            "failed to create suite results dir {}: {}",
            plan.suite_results_root.display(),
            e
        )
    })?;
    std::fs::create_dir_all(&plan.script_results_root).map_err(|e| {
        format!(
            "failed to create script results dir {}: {}",
            plan.script_results_root.display(),
            e
        )
    })?;
    Ok(())
}

fn build_campaign_batch_plan_at(
    options: &CampaignRunOptions,
    selected_count: usize,
    ctx: &CampaignRunContext,
    created_unix_ms: u64,
) -> CampaignBatchPlan {
    let run_id = created_unix_ms.to_string();
    let batch_root = ctx
        .resolved_out_dir
        .join("campaign-batches")
        .join(campaign_batch_selection_slug(options, selected_count))
        .join(&run_id);
    let summary_path =
        batch_root.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    let index_path = batch_root.join(crate::regression_summary::DIAG_REGRESSION_INDEX_FILENAME_V1);
    CampaignBatchPlan {
        created_unix_ms,
        run_id,
        batch_root,
        summary_path,
        index_path,
    }
}

fn build_campaign_batch_plan(
    options: &CampaignRunOptions,
    selected_count: usize,
    ctx: &CampaignRunContext,
) -> CampaignBatchPlan {
    build_campaign_batch_plan_at(options, selected_count, ctx, now_unix_ms())
}

fn build_campaign_item_execution_plan(
    index: usize,
    item: &CampaignItemDefinition,
    suite_results_root: &Path,
    script_results_root: &Path,
    ctx: &CampaignRunContext,
) -> Result<CampaignItemExecutionPlan, String> {
    let (label, results_root, kind, value, suite_rest, suite_script_inputs) = match item.kind {
        CampaignItemKind::Suite => (
            "suite",
            suite_results_root,
            CampaignItemKind::Suite,
            item.value.as_str(),
            vec![item.value.clone()],
            ctx.suite_script_inputs.clone(),
        ),
        CampaignItemKind::Script => (
            "script",
            script_results_root,
            CampaignItemKind::Script,
            item.value.as_str(),
            Vec::new(),
            vec![item.value.clone()],
        ),
    };

    let out_dir = results_root.join(format!("{:02}-{}", index + 1, zip_safe_component(value)));
    std::fs::create_dir_all(&out_dir).map_err(|e| {
        format!(
            "failed to create {} output dir {}: {}",
            label,
            out_dir.display(),
            e
        )
    })?;

    let regression_summary_path =
        out_dir.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);

    Ok(CampaignItemExecutionPlan {
        kind,
        item_id: item.value.clone(),
        out_dir,
        regression_summary_path,
        suite_rest,
        suite_script_inputs,
    })
}

fn build_campaign_item_suite_context(
    plan: &CampaignItemExecutionPlan,
    ctx: &CampaignRunContext,
) -> diag_suite::SuiteCmdContext {
    diag_suite::SuiteCmdContext {
        pack_after_run: ctx.pack_after_run,
        rest: plan.suite_rest.clone(),
        suite_script_inputs: plan.suite_script_inputs.clone(),
        suite_prewarm_scripts: ctx.suite_prewarm_scripts.clone(),
        suite_prelude_scripts: ctx.suite_prelude_scripts.clone(),
        suite_prelude_each_run: ctx.suite_prelude_each_run,
        workspace_root: ctx.workspace_root.clone(),
        resolved_paths: ResolvedScriptPaths::for_out_dir(&ctx.workspace_root, &plan.out_dir),
        devtools_ws_url: ctx.devtools_ws_url.clone(),
        devtools_token: ctx.devtools_token.clone(),
        devtools_session_id: ctx.devtools_session_id.clone(),
        timeout_ms: ctx.timeout_ms,
        poll_ms: ctx.poll_ms,
        stats_top: ctx.stats_top,
        stats_json: ctx.stats_json,
        warmup_frames: ctx.warmup_frames,
        max_test_ids: ctx.max_test_ids,
        lint_all_test_ids_bounds: ctx.lint_all_test_ids_bounds,
        lint_eps_px: ctx.lint_eps_px,
        suite_lint: ctx.suite_lint,
        pack_include_screenshots: ctx.pack_include_screenshots,
        reuse_launch: ctx.reuse_launch,
        launch: ctx.launch.clone(),
        launch_env: ctx.launch_env.clone(),
        launch_high_priority: ctx.launch_high_priority,
        launch_write_bundle_json: ctx.launch_write_bundle_json,
        keep_open: ctx.keep_open,
        checks: ctx.checks.clone(),
    }
}

pub(crate) fn cmd_campaign(ctx: CampaignCmdContext) -> Result<(), String> {
    let registry = CampaignRegistry::load_from_workspace_root(&ctx.workspace_root)?;
    let subcommand = resolve_campaign_subcommand(&ctx.rest)?;

    match subcommand {
        CampaignSubcommand::List => cmd_campaign_list(&registry, &ctx.rest[1..], ctx.stats_json),
        CampaignSubcommand::Show => cmd_campaign_show(&registry, &ctx.rest[1..], ctx.stats_json),
        CampaignSubcommand::Share => cmd_campaign_share(
            &ctx.rest[1..],
            &ctx.workspace_root,
            ctx.stats_json,
            ctx.stats_top,
            ctx.warmup_frames,
        ),
        CampaignSubcommand::Run => {
            let run_rest = ctx.rest[1..].to_vec();
            cmd_campaign_run(&registry, &run_rest, ctx.into())
        }
    }
}

fn resolve_campaign_subcommand(rest: &[String]) -> Result<CampaignSubcommand, String> {
    let Some(sub) = rest.first().map(|value| value.as_str()) else {
        return Err(
            "missing campaign subcommand (try: fretboard diag campaign list | show <id> | run <id>)"
                .to_string(),
        );
    };

    match sub {
        "list" => Ok(CampaignSubcommand::List),
        "show" => Ok(CampaignSubcommand::Show),
        "share" => Ok(CampaignSubcommand::Share),
        "run" => Ok(CampaignSubcommand::Run),
        other => Err(format!("unknown diag campaign subcommand: {other}")),
    }
}

fn cmd_campaign_share(
    rest: &[String],
    workspace_root: &Path,
    json: bool,
    stats_top: usize,
    warmup_frames: u64,
) -> Result<(), String> {
    let options = parse_campaign_share_options(rest)?;
    let selection = resolve_campaign_share_selection(workspace_root, &options.source)?;
    let share_manifest_path = write_campaign_share_manifest(
        &selection.root_dir,
        &selection.summary_path,
        workspace_root,
        options.include_passed,
        stats_top,
        warmup_frames,
    )?;
    let payload = read_json_value(&share_manifest_path).ok_or_else(|| {
        format!(
            "failed to read share manifest {}",
            share_manifest_path.display()
        )
    })?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    } else {
        let bundles_total = payload
            .pointer("/counters/bundles_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let bundles_packed = payload
            .pointer("/counters/bundles_packed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let bundles_missing = payload
            .pointer("/counters/bundles_missing")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        println!(
            "campaign share: ok (bundles={}, packed={}, missing={}, manifest={})",
            bundles_total,
            bundles_packed,
            bundles_missing,
            share_manifest_path.display()
        );
    }
    Ok(())
}

fn cmd_campaign_list(
    registry: &CampaignRegistry,
    rest: &[String],
    json: bool,
) -> Result<(), String> {
    let filter = parse_campaign_list_filters(rest)?;
    let campaigns = registry.filtered_campaigns(&filter);

    if json {
        let payload = serde_json::json!({
            "filters": campaign_filter_to_json(&filter),
            "campaigns": campaigns
                .iter()
                .map(|campaign| campaign_to_json(campaign))
                .collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    for campaign in campaigns {
        let mut details = vec![
            lane_to_str(campaign.lane).to_string(),
            format!("suites={}", campaign.suite_count()),
            format!("scripts={}", campaign.script_count()),
            format!("source={}", source_kind_str(&campaign.source)),
        ];
        if let Some(tier) = campaign.tier.as_deref() {
            details.push(format!("tier={tier}"));
        }
        if let Some(owner) = campaign.owner.as_deref() {
            details.push(format!("owner={owner}"));
        }
        println!(
            "{} ({}) - {}",
            campaign.id,
            details.join(", "),
            campaign.description
        );
    }

    Ok(())
}

fn parse_campaign_list_filters(rest: &[String]) -> Result<CampaignFilterOptions, String> {
    let mut filter = CampaignFilterOptions::default();
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--lane" => {
                let value = rest
                    .get(index + 1)
                    .ok_or_else(|| "missing value after --lane".to_string())?;
                filter.lane = Some(parse_lane(value)?);
                index += 2;
            }
            "--tier" => {
                let value = rest
                    .get(index + 1)
                    .ok_or_else(|| "missing value after --tier".to_string())?;
                filter.tier = Some(value.to_string());
                index += 2;
            }
            "--tag" => {
                let value = rest
                    .get(index + 1)
                    .ok_or_else(|| "missing value after --tag".to_string())?;
                filter.tags.push(value.to_string());
                index += 2;
            }
            "--platform" => {
                let value = rest
                    .get(index + 1)
                    .ok_or_else(|| "missing value after --platform".to_string())?;
                filter.platforms.push(value.to_string());
                index += 2;
            }
            other => {
                return Err(format!("unknown diag campaign list flag: {other}"));
            }
        }
    }
    Ok(filter)
}

fn parse_campaign_share_options(rest: &[String]) -> Result<CampaignShareOptions, String> {
    let mut source: Option<String> = None;
    let mut include_passed = false;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--include-passed" => {
                include_passed = true;
                index += 1;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag campaign share flag: {other}"));
            }
            other => {
                if source.is_some() {
                    return Err(format!(
                        "unexpected extra positional for `diag campaign share`: {other}"
                    ));
                }
                source = Some(other.to_string());
                index += 1;
            }
        }
    }
    let source = source
        .ok_or_else(|| "missing campaign or batch run dir for `diag campaign share`".to_string())?;
    Ok(CampaignShareOptions {
        source,
        include_passed,
    })
}

fn resolve_campaign_share_selection(
    workspace_root: &Path,
    raw: &str,
) -> Result<CampaignShareSelection, String> {
    let path = crate::resolve_path(workspace_root, PathBuf::from(raw));
    if path.is_file() {
        let expected = crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1;
        if path.file_name().and_then(|v| v.to_str()) != Some(expected) {
            return Err(format!(
                "expected campaign share input to be a directory or {}: {}",
                expected,
                path.display()
            ));
        }
        let root_dir = path
            .parent()
            .ok_or_else(|| format!("summary path has no parent dir: {}", path.display()))?
            .to_path_buf();
        return Ok(CampaignShareSelection {
            summary_path: path,
            root_dir,
        });
    }
    if path.is_dir() {
        let summary_path =
            path.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);
        if !summary_path.is_file() {
            return Err(format!(
                "campaign share input does not contain {}: {}",
                crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1,
                path.display()
            ));
        }
        return Ok(CampaignShareSelection {
            summary_path,
            root_dir: path,
        });
    }
    Err(format!(
        "campaign share input not found: {}",
        path.display()
    ))
}

fn campaign_filter_to_json(filter: &CampaignFilterOptions) -> serde_json::Value {
    serde_json::json!({
        "lane": filter.lane,
        "tier": filter.tier,
        "tags": filter.tags,
        "platforms": filter.platforms,
    })
}

fn cmd_campaign_show(
    registry: &CampaignRegistry,
    rest: &[String],
    json: bool,
) -> Result<(), String> {
    let Some(campaign_id) = rest.first() else {
        return Err("missing campaign id for `diag campaign show`".to_string());
    };
    if rest.len() > 1 {
        return Err(format!(
            "unexpected extra positional for `diag campaign show`: {}",
            rest[1]
        ));
    }

    let campaign = registry.resolve(campaign_id)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&campaign_to_json(campaign)).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    println!("id: {}", campaign.id);
    println!("description: {}", campaign.description);
    println!("lane: {}", lane_to_str(campaign.lane));
    println!("source: {}", source_kind_str(&campaign.source));
    if let Some(profile) = campaign.profile.as_deref() {
        println!("profile: {profile}");
    }
    if let Some(owner) = campaign.owner.as_deref() {
        println!("owner: {owner}");
    }
    if let Some(tier) = campaign.tier.as_deref() {
        println!("tier: {tier}");
    }
    if let Some(expected_duration_ms) = campaign.expected_duration_ms {
        println!("expected_duration_ms: {expected_duration_ms}");
    }
    if !campaign.platforms.is_empty() {
        println!("platforms: {}", campaign.platforms.join(", "));
    }
    if !campaign.tags.is_empty() {
        println!("tags: {}", campaign.tags.join(", "));
    }
    println!("items ({}):", campaign.items.len());
    for item in &campaign.items {
        println!("  - {}: {}", item_kind_str(item.kind), item.value);
    }
    println!("suites ({}):", campaign.suite_count());
    for suite in campaign.suites() {
        println!("  - {suite}");
    }
    println!("scripts ({}):", campaign.script_count());
    for script in campaign.scripts() {
        println!("  - {script}");
    }

    Ok(())
}

fn cmd_campaign_run(
    registry: &CampaignRegistry,
    rest: &[String],
    ctx: CampaignRunContext,
) -> Result<(), String> {
    let options = parse_campaign_run_options(rest)?;
    let outcome = execute_campaign_run_selection(registry, &options, &ctx)?;
    print_campaign_run_output(&options, &outcome, ctx.stats_json)?;
    if !outcome.command_failures.is_empty() {
        return Err(outcome.command_failures.join("; "));
    }
    Ok(())
}

fn execute_campaign_run_selection(
    registry: &CampaignRegistry,
    options: &CampaignRunOptions,
    ctx: &CampaignRunContext,
) -> Result<CampaignRunOutcome, String> {
    let selected = select_campaigns_for_run(registry, options)?;

    let mut reports = Vec::new();
    for campaign in selected {
        reports.push(execute_campaign(campaign, ctx));
    }

    let batch = if reports.len() > 1 {
        Some(write_campaign_batch_artifacts(&reports, options, ctx)?)
    } else {
        None
    };

    Ok(build_campaign_run_outcome(reports, batch))
}

fn build_campaign_run_outcome(
    reports: Vec<CampaignExecutionReport>,
    batch: Option<CampaignBatchArtifacts>,
) -> CampaignRunOutcome {
    let counters = build_campaign_run_counters(&reports);
    let command_failures =
        collect_campaign_run_failures(&reports, batch.as_ref(), counters.campaigns_failed);
    CampaignRunOutcome {
        reports,
        batch,
        counters,
        command_failures,
    }
}

fn build_campaign_run_counters(reports: &[CampaignExecutionReport]) -> CampaignRunCounters {
    let campaigns_failed = reports.iter().filter(|report| !report.ok).count();
    CampaignRunCounters {
        campaigns_total: reports.len(),
        campaigns_failed,
        campaigns_passed: reports.len().saturating_sub(campaigns_failed),
        items_total: reports.iter().map(|report| report.items_total).sum(),
        items_failed: reports.iter().map(|report| report.items_failed).sum(),
        suites_total: reports.iter().map(|report| report.suites_total).sum(),
        scripts_total: reports.iter().map(|report| report.scripts_total).sum(),
    }
}

fn collect_campaign_run_failures(
    reports: &[CampaignExecutionReport],
    batch: Option<&CampaignBatchArtifacts>,
    campaigns_failed: usize,
) -> Vec<String> {
    let mut command_failures = Vec::new();
    if let Some(failure_summary) = campaign_failed_reports_summary(reports, campaigns_failed) {
        command_failures.push(failure_summary);
    }
    if let Some(batch) = batch {
        command_failures.extend(
            [
                campaign_batch_summarize_failure_text(batch),
                campaign_batch_share_failure_text(batch),
            ]
            .into_iter()
            .flatten(),
        );
    }
    command_failures.extend(
        reports
            .iter()
            .filter_map(campaign_report_share_failure_text),
    );
    command_failures
}

fn campaign_failed_reports_summary(
    reports: &[CampaignExecutionReport],
    campaigns_failed: usize,
) -> Option<String> {
    if campaigns_failed == 0 {
        return None;
    }
    let failures = reports
        .iter()
        .filter(|report| !report.ok)
        .map(|report| {
            format!(
                "{}: {}",
                report.campaign_id,
                report.error.as_deref().unwrap_or("unknown error")
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    Some(format!(
        "campaign run completed with {} failed campaign(s): {}",
        campaigns_failed, failures
    ))
}

fn campaign_batch_summarize_failure_text(batch: &CampaignBatchArtifacts) -> Option<String> {
    batch.aggregate.summarize_error.as_deref().map(|error| {
        format!(
            "campaign batch summarize failed under {}: {}",
            batch.batch_root.display(),
            error
        )
    })
}

fn campaign_batch_share_failure_text(batch: &CampaignBatchArtifacts) -> Option<String> {
    batch.aggregate.share_error.as_deref().map(|error| {
        format!(
            "campaign batch share export failed under {}: {}",
            batch.batch_root.display(),
            error
        )
    })
}

fn campaign_report_share_failure_text(report: &CampaignExecutionReport) -> Option<String> {
    report.aggregate.share_error.as_deref().map(|error| {
        format!(
            "campaign `{}` share export failed under {}: {}",
            report.campaign_id,
            report.out_dir.display(),
            error
        )
    })
}

fn print_campaign_run_output(
    options: &CampaignRunOptions,
    outcome: &CampaignRunOutcome,
    stats_json: bool,
) -> Result<(), String> {
    if stats_json {
        let payload = campaign_run_outcome_to_json(options, outcome);
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    let lines = if outcome.reports.len() == 1 {
        campaign_single_run_output_lines(&outcome.reports[0])
    } else {
        campaign_batch_run_output_lines(outcome)
    };
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

fn campaign_single_run_output_lines(report: &CampaignExecutionReport) -> Vec<String> {
    if report.ok {
        return vec![format!(
            "campaign: ok (id={}, items={}, suites={}, scripts={}, out_dir={})",
            report.campaign_id,
            report.items_total,
            report.suites_total,
            report.scripts_total,
            report.out_dir.display()
        )];
    }
    report
        .aggregate
        .share_manifest_path
        .as_deref()
        .map(|share_manifest_path| {
            vec![format!(
                "campaign: failed evidence exported (id={}, share_manifest={})",
                report.campaign_id,
                share_manifest_path.display()
            )]
        })
        .unwrap_or_default()
}

fn campaign_batch_run_output_lines(outcome: &CampaignRunOutcome) -> Vec<String> {
    let mut lines = vec![format!(
        "campaign batch: {} run(s), {} failed",
        outcome.counters.campaigns_total, outcome.counters.campaigns_failed
    )];
    if let Some(batch) = outcome.batch.as_ref() {
        lines.push(format!("  batch_root: {}", batch.batch_root.display()));
        if let Some(share_manifest_line) = campaign_share_manifest_output_line(
            "  share_manifest",
            batch.aggregate.share_manifest_path.as_deref(),
        ) {
            lines.push(share_manifest_line);
        }
    }
    for report in &outcome.reports {
        lines.extend(campaign_batch_report_output_lines(report));
    }
    lines
}

fn campaign_batch_report_output_lines(report: &CampaignExecutionReport) -> Vec<String> {
    let status = if report.ok { "ok" } else { "failed" };
    let mut lines = vec![format!(
        "  - {} [{}] items={} failed={} -> {}",
        report.campaign_id,
        status,
        report.items_total,
        report.items_failed,
        report.out_dir.display()
    )];
    if let Some(share_manifest_line) = campaign_share_manifest_output_line(
        "    share_manifest",
        report.aggregate.share_manifest_path.as_deref(),
    ) {
        lines.push(share_manifest_line);
    }
    lines
}

fn campaign_share_manifest_output_line(
    label: &str,
    share_manifest_path: Option<&Path>,
) -> Option<String> {
    share_manifest_path
        .map(|share_manifest_path| format!("{label}: {}", share_manifest_path.display()))
}

fn campaign_run_selection_json(options: &CampaignRunOptions) -> serde_json::Value {
    serde_json::json!({
        "campaign_ids": &options.campaign_ids,
        "filters": campaign_filter_to_json(&options.filter),
    })
}

fn campaign_run_outcome_to_json(
    options: &CampaignRunOptions,
    outcome: &CampaignRunOutcome,
) -> serde_json::Value {
    serde_json::json!({
        "selection": campaign_run_selection_json(options),
        "counters": campaign_run_outcome_counters_json(outcome.counters),
        "batch": campaign_run_outcome_batch_json(outcome.batch.as_ref()),
        "runs": campaign_run_reports_json(&outcome.reports),
    })
}

fn campaign_run_outcome_counters_json(counters: CampaignRunCounters) -> serde_json::Value {
    serde_json::json!({
        "campaigns_total": counters.campaigns_total,
        "campaigns_failed": counters.campaigns_failed,
        "campaigns_passed": counters.campaigns_passed,
        "items_total": counters.items_total,
        "items_failed": counters.items_failed,
        "suites_total": counters.suites_total,
        "scripts_total": counters.scripts_total,
    })
}

fn campaign_run_outcome_batch_json(
    batch: Option<&CampaignBatchArtifacts>,
) -> Option<serde_json::Value> {
    batch.map(campaign_batch_to_json)
}

fn campaign_run_reports_json(reports: &[CampaignExecutionReport]) -> Vec<serde_json::Value> {
    reports.iter().map(campaign_run_report_to_json).collect()
}

fn campaign_run_report_to_json(report: &CampaignExecutionReport) -> serde_json::Value {
    campaign_report_json(report, CampaignReportPathMode::RunOutcome)
}

fn select_campaigns_for_run<'a>(
    registry: &'a CampaignRegistry,
    options: &CampaignRunOptions,
) -> Result<Vec<&'a CampaignDefinition>, String> {
    if !options.campaign_ids.is_empty() {
        return select_explicit_campaigns_for_run(registry, options);
    }
    select_filtered_campaigns_for_run(registry, &options.filter)
}

fn select_explicit_campaigns_for_run<'a>(
    registry: &'a CampaignRegistry,
    options: &CampaignRunOptions,
) -> Result<Vec<&'a CampaignDefinition>, String> {
    let mut selected = Vec::new();
    for campaign_id in &options.campaign_ids {
        let campaign = registry.resolve(campaign_id)?;
        if campaign.matches_filter(&options.filter) {
            selected.push(campaign);
        }
    }
    if selected.is_empty() {
        return Err(
            "explicit campaign ids were provided but none matched the requested filters"
                .to_string(),
        );
    }
    Ok(selected)
}

fn select_filtered_campaigns_for_run<'a>(
    registry: &'a CampaignRegistry,
    filter: &CampaignFilterOptions,
) -> Result<Vec<&'a CampaignDefinition>, String> {
    if campaign_filter_is_empty(filter) {
        return Err(
            "missing campaign id or run selector (try: `diag campaign run ui-gallery-smoke` or `diag campaign run --lane smoke --tag ui-gallery`)"
                .to_string(),
        );
    }

    let selected = registry.filtered_campaigns(filter);
    if selected.is_empty() {
        return Err("no campaigns matched the requested run selectors".to_string());
    }
    Ok(selected)
}

fn execute_campaign(
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
) -> CampaignExecutionReport {
    let start_plan = build_campaign_execution_start_plan(campaign, ctx);
    build_campaign_execution_report_from_outcome_result(
        campaign,
        &start_plan.execution,
        execute_campaign_inner(campaign, ctx, &start_plan),
    )
}

fn build_campaign_execution_report_from_outcome_result(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    outcome: Result<CampaignExecutionOutcome, String>,
) -> CampaignExecutionReport {
    build_campaign_execution_report(
        campaign,
        plan,
        normalize_campaign_execution_outcome(outcome, campaign.items.len()),
    )
}

fn normalize_campaign_execution_outcome(
    outcome: Result<CampaignExecutionOutcome, String>,
    item_count: usize,
) -> CampaignExecutionOutcome {
    match outcome {
        Ok(outcome) => outcome,
        Err(error) => CampaignExecutionOutcome {
            items_failed: item_count,
            error: Some(error),
            share_manifest_path: None,
            share_error: None,
        },
    }
}

fn build_campaign_execution_report(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    outcome: CampaignExecutionOutcome,
) -> CampaignExecutionReport {
    let CampaignExecutionOutcome {
        items_failed,
        error,
        share_manifest_path,
        share_error,
    } = outcome;
    CampaignExecutionReport {
        campaign_id: campaign.id.clone(),
        out_dir: plan.campaign_root.clone(),
        aggregate: build_campaign_report_aggregate_artifacts(
            plan,
            share_manifest_path,
            share_error,
        ),
        items_total: campaign.items.len(),
        items_failed,
        suites_total: campaign.suite_count(),
        scripts_total: campaign.script_count(),
        ok: error.is_none(),
        error,
    }
}

fn build_campaign_report_aggregate_artifacts(
    plan: &CampaignExecutionPlan,
    share_manifest_path: Option<PathBuf>,
    share_error: Option<String>,
) -> CampaignAggregateArtifacts {
    CampaignAggregateArtifacts {
        summary_path: plan.summary_path.clone(),
        index_path: plan.index_path.clone(),
        share_manifest_path,
        summarize_error: None,
        share_error,
    }
}

fn execute_campaign_inner(
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
    start_plan: &CampaignExecutionStartPlan,
) -> Result<CampaignExecutionOutcome, String> {
    execute_campaign_start_plan(start_plan)?;

    let item_results = execute_campaign_items(campaign, &start_plan.execution, ctx)?;
    let finalization =
        finalize_campaign_execution(campaign, &start_plan.execution, &item_results, ctx)?;

    Ok(build_campaign_execution_outcome(
        campaign,
        &start_plan.execution,
        &item_results,
        finalization,
    ))
}

#[cfg(test)]
fn build_campaign_execution_start_plan_at(
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
    created_unix_ms: u64,
) -> CampaignExecutionStartPlan {
    let execution = build_campaign_execution_plan_at(campaign, ctx, created_unix_ms);
    CampaignExecutionStartPlan {
        manifest_write: build_campaign_manifest_write_plan(&execution, campaign, ctx),
        execution,
    }
}

fn build_campaign_execution_start_plan(
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
) -> CampaignExecutionStartPlan {
    let execution = build_campaign_execution_plan(campaign, ctx);
    CampaignExecutionStartPlan {
        manifest_write: build_campaign_manifest_write_plan(&execution, campaign, ctx),
        execution,
    }
}

fn execute_campaign_start_plan(start_plan: &CampaignExecutionStartPlan) -> Result<(), String> {
    ensure_campaign_execution_dirs(&start_plan.execution)?;
    write_campaign_json_plan(&start_plan.manifest_write)
}

fn build_campaign_execution_outcome(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    item_results: &[CampaignItemRunResult],
    finalization: CampaignExecutionFinalization,
) -> CampaignExecutionOutcome {
    let CampaignExecutionFinalization {
        items_failed,
        aggregate,
    } = finalization;
    let CampaignAggregateArtifacts {
        summary_path: _,
        index_path: _,
        share_manifest_path,
        summarize_error,
        share_error,
    } = aggregate;
    let error =
        campaign_execution_error(campaign, plan, item_results, items_failed, summarize_error);
    CampaignExecutionOutcome {
        items_failed,
        error,
        share_manifest_path,
        share_error,
    }
}

fn campaign_execution_error(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    item_results: &[CampaignItemRunResult],
    items_failed: usize,
    summarize_error: Option<String>,
) -> Option<String> {
    summarize_error
        .as_deref()
        .map(|error| campaign_summarize_failure_error(&campaign.id, error))
        .or_else(|| {
            (items_failed > 0)
                .then(|| campaign_item_failures_error(campaign, plan, item_results, items_failed))
        })
}

fn campaign_summarize_failure_error(campaign_id: &str, error: &str) -> String {
    format!(
        "campaign `{}` finished item execution but summarize failed: {}",
        campaign_id, error
    )
}

fn execute_campaign_items(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    ctx: &CampaignRunContext,
) -> Result<Vec<CampaignItemRunResult>, String> {
    let item_plans = build_campaign_item_execution_plans(campaign, plan, ctx)?;
    Ok(execute_campaign_item_execution_plans(item_plans, ctx))
}

fn build_campaign_item_execution_plans(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    ctx: &CampaignRunContext,
) -> Result<Vec<CampaignItemExecutionPlan>, String> {
    campaign
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            build_campaign_item_execution_plan(
                index,
                item,
                &plan.suite_results_root,
                &plan.script_results_root,
                ctx,
            )
        })
        .collect()
}

fn execute_campaign_item_execution_plans(
    item_plans: Vec<CampaignItemExecutionPlan>,
    ctx: &CampaignRunContext,
) -> Vec<CampaignItemRunResult> {
    item_plans
        .into_iter()
        .map(|item_plan| run_campaign_item_plan(item_plan, ctx))
        .collect()
}

fn finalize_campaign_execution(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    item_results: &[CampaignItemRunResult],
    ctx: &CampaignRunContext,
) -> Result<CampaignExecutionFinalization, String> {
    let finalize_plan = build_campaign_execution_finalize_plan(item_results, plan);
    execute_campaign_execution_finalize_plan(campaign, plan, item_results, &finalize_plan, ctx)
}

fn build_campaign_execution_finalize_plan(
    item_results: &[CampaignItemRunResult],
    plan: &CampaignExecutionPlan,
) -> CampaignExecutionFinalizePlan {
    CampaignExecutionFinalizePlan {
        items_failed: item_results.iter().filter(|entry| !entry.ok).count(),
        summary_finalize: build_campaign_execution_summary_finalize_plan(item_results, plan),
    }
}

fn execute_campaign_execution_finalize_plan(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    item_results: &[CampaignItemRunResult],
    finalize_plan: &CampaignExecutionFinalizePlan,
    ctx: &CampaignRunContext,
) -> Result<CampaignExecutionFinalization, String> {
    let summary_artifacts =
        finalize_campaign_summary_artifacts(&finalize_plan.summary_finalize, ctx);
    write_campaign_result(plan, campaign, item_results, &summary_artifacts, ctx)?;

    Ok(build_campaign_execution_finalization(
        finalize_plan.items_failed,
        plan,
        summary_artifacts,
    ))
}

fn build_campaign_execution_finalization(
    items_failed: usize,
    plan: &CampaignExecutionPlan,
    summary_artifacts: CampaignSummaryArtifacts,
) -> CampaignExecutionFinalization {
    CampaignExecutionFinalization {
        items_failed,
        aggregate: build_campaign_aggregate_artifacts(
            &plan.summary_path,
            &plan.index_path,
            &summary_artifacts,
        ),
    }
}

fn finalize_campaign_summary_artifacts(
    plan: &CampaignSummaryFinalizePlan,
    ctx: &CampaignRunContext,
) -> CampaignSummaryArtifacts {
    let outcome = execute_campaign_summary_finalize_outcome(plan, ctx);
    build_campaign_summary_artifacts(plan.created_unix_ms, now_unix_ms(), outcome)
}

fn execute_campaign_summary_finalize_outcome(
    plan: &CampaignSummaryFinalizePlan,
    ctx: &CampaignRunContext,
) -> CampaignSummaryFinalizeOutcome {
    let summarize_result = diag_summarize::cmd_summarize(diag_summarize::SummarizeCmdContext {
        rest: plan.summarize_inputs.clone(),
        workspace_root: ctx.workspace_root.clone(),
        resolved_out_dir: plan.out_dir.clone(),
        stats_json: false,
    });
    let (share_manifest_path, share_error) = maybe_write_failure_share_manifest(
        &plan.out_dir,
        &plan.summary_path,
        &ctx.workspace_root,
        plan.should_generate_share_manifest,
        ctx.stats_top,
        ctx.warmup_frames,
    );

    CampaignSummaryFinalizeOutcome {
        summarize_error: summarize_result.err(),
        share_manifest_path,
        share_error,
    }
}

fn build_campaign_summary_artifacts(
    created_unix_ms: u64,
    finished_unix_ms: u64,
    outcome: CampaignSummaryFinalizeOutcome,
) -> CampaignSummaryArtifacts {
    CampaignSummaryArtifacts {
        finished_unix_ms,
        duration_ms: finished_unix_ms.saturating_sub(created_unix_ms),
        summarize_error: outcome.summarize_error,
        share_manifest_path: outcome.share_manifest_path,
        share_error: outcome.share_error,
    }
}

fn campaign_item_failures_error(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    item_results: &[CampaignItemRunResult],
    items_failed: usize,
) -> String {
    let failing = campaign_item_failures_summary(item_results);
    format!(
        "campaign `{}` completed with {} failed item(s) under {}: {}",
        campaign.id,
        items_failed,
        plan.campaign_root.display(),
        failing
    )
}

fn campaign_item_failures_summary(item_results: &[CampaignItemRunResult]) -> String {
    item_results
        .iter()
        .filter(|entry| !entry.ok)
        .map(campaign_item_failure_summary)
        .collect::<Vec<_>>()
        .join("; ")
}

fn campaign_item_failure_summary(entry: &CampaignItemRunResult) -> String {
    let error = entry.error.as_deref().unwrap_or("unknown error");
    format!("{} {}: {}", item_kind_str(entry.kind), entry.item_id, error)
}

fn write_campaign_batch_artifacts(
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    ctx: &CampaignRunContext,
) -> Result<CampaignBatchArtifacts, String> {
    let write_plan = build_campaign_batch_artifact_write_plan(reports, options, ctx);
    execute_campaign_batch_artifact_write_plan(&write_plan, reports, options, ctx)
}

fn finalize_campaign_batch_artifacts_with_finalize_plan(
    plan: &CampaignBatchPlan,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    finalize_plan: &CampaignSummaryFinalizePlan,
    ctx: &CampaignRunContext,
) -> Result<CampaignBatchArtifacts, String> {
    let summary_artifacts = finalize_campaign_summary_artifacts(finalize_plan, ctx);
    write_campaign_batch_result(plan, reports, options, &summary_artifacts, ctx)?;

    Ok(build_campaign_batch_artifacts(plan, summary_artifacts))
}

fn build_campaign_batch_artifact_write_plan(
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    ctx: &CampaignRunContext,
) -> CampaignBatchArtifactWritePlan {
    let batch = build_campaign_batch_plan(options, reports.len(), ctx);
    CampaignBatchArtifactWritePlan {
        manifest_write: build_campaign_batch_manifest_write_plan(&batch, reports, options, ctx),
        summary_finalize: build_campaign_batch_summary_finalize_plan(reports, &batch),
        batch,
    }
}

fn execute_campaign_batch_artifact_write_plan(
    write_plan: &CampaignBatchArtifactWritePlan,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    ctx: &CampaignRunContext,
) -> Result<CampaignBatchArtifacts, String> {
    write_campaign_json_plan(&write_plan.manifest_write)?;
    finalize_campaign_batch_artifacts_with_finalize_plan(
        &write_plan.batch,
        reports,
        options,
        &write_plan.summary_finalize,
        ctx,
    )
}

fn build_campaign_execution_summary_finalize_plan(
    item_results: &[CampaignItemRunResult],
    plan: &CampaignExecutionPlan,
) -> CampaignSummaryFinalizePlan {
    CampaignSummaryFinalizePlan {
        summarize_inputs: Vec::new(),
        out_dir: plan.campaign_root.clone(),
        summary_path: plan.summary_path.clone(),
        created_unix_ms: plan.created_unix_ms,
        should_generate_share_manifest: item_results.iter().any(|entry| !entry.ok),
    }
}

fn build_campaign_batch_summary_finalize_plan(
    reports: &[CampaignExecutionReport],
    plan: &CampaignBatchPlan,
) -> CampaignSummaryFinalizePlan {
    CampaignSummaryFinalizePlan {
        summarize_inputs: campaign_report_out_dirs(reports),
        out_dir: plan.batch_root.clone(),
        summary_path: plan.summary_path.clone(),
        created_unix_ms: plan.created_unix_ms,
        should_generate_share_manifest: reports.iter().any(|report| !report.ok),
    }
}

fn build_campaign_batch_artifacts(
    plan: &CampaignBatchPlan,
    summary_artifacts: CampaignSummaryArtifacts,
) -> CampaignBatchArtifacts {
    CampaignBatchArtifacts {
        batch_root: plan.batch_root.clone(),
        aggregate: build_campaign_aggregate_artifacts(
            &plan.summary_path,
            &plan.index_path,
            &summary_artifacts,
        ),
    }
}

fn campaign_report_out_dirs(reports: &[CampaignExecutionReport]) -> Vec<String> {
    reports
        .iter()
        .map(|report| report.out_dir.display().to_string())
        .collect()
}

fn run_campaign_item_plan(
    item_plan: CampaignItemExecutionPlan,
    ctx: &CampaignRunContext,
) -> CampaignItemRunResult {
    let CampaignItemExecutionPlan {
        kind,
        item_id,
        out_dir,
        regression_summary_path,
        suite_rest: _,
        suite_script_inputs: _,
    } = item_plan.clone();
    let suite_ctx = build_campaign_item_suite_context(&item_plan, ctx);
    let result = diag_suite::cmd_suite(suite_ctx);

    build_campaign_item_run_result(kind, item_id, out_dir, regression_summary_path, result)
}

fn build_campaign_item_run_result(
    kind: CampaignItemKind,
    item_id: String,
    out_dir: PathBuf,
    regression_summary_path: PathBuf,
    result: Result<(), String>,
) -> CampaignItemRunResult {
    match result {
        Ok(()) => CampaignItemRunResult {
            kind,
            item_id,
            out_dir,
            regression_summary_path,
            ok: true,
            error: None,
        },
        Err(error) => CampaignItemRunResult {
            kind,
            item_id,
            out_dir,
            regression_summary_path,
            ok: false,
            error: Some(error),
        },
    }
}

fn parse_campaign_run_options(rest: &[String]) -> Result<CampaignRunOptions, String> {
    let mut out = CampaignRunOptions::default();
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            flag @ ("--lane" | "--tier" | "--tag" | "--platform") => {
                let value = require_campaign_run_flag_value(rest, index, flag)?;
                apply_campaign_run_flag(&mut out, flag, value)?;
                index += 2;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag campaign run flag: {other}"));
            }
            other => {
                out.campaign_ids.push(other.to_string());
                index += 1;
            }
        }
    }
    Ok(out)
}

fn require_campaign_run_flag_value<'a>(
    rest: &'a [String],
    index: usize,
    flag: &str,
) -> Result<&'a str, String> {
    rest.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value after {flag}"))
}

fn apply_campaign_run_flag(
    options: &mut CampaignRunOptions,
    flag: &str,
    value: &str,
) -> Result<(), String> {
    match flag {
        "--lane" => options.filter.lane = Some(parse_lane(value)?),
        "--tier" => options.filter.tier = Some(value.to_string()),
        "--tag" => options.filter.tags.push(value.to_string()),
        "--platform" => options.filter.platforms.push(value.to_string()),
        other => return Err(format!("unknown diag campaign run flag: {other}")),
    }
    Ok(())
}

fn campaign_filter_is_empty(filter: &CampaignFilterOptions) -> bool {
    filter.lane.is_none()
        && filter.tier.is_none()
        && filter.tags.is_empty()
        && filter.platforms.is_empty()
}

fn write_campaign_share_manifest(
    root_dir: &Path,
    summary_path: &Path,
    workspace_root: &Path,
    include_passed: bool,
    stats_top: usize,
    warmup_frames: u64,
) -> Result<PathBuf, String> {
    let bytes = std::fs::read(summary_path).map_err(|e| {
        format!(
            "failed to read regression summary {}: {}",
            summary_path.display(),
            e
        )
    })?;
    let summary = serde_json::from_slice::<RegressionSummaryV1>(&bytes).map_err(|e| {
        format!(
            "invalid regression summary {}: {}",
            summary_path.display(),
            e
        )
    })?;

    let share_dir = root_dir.join("share");
    std::fs::create_dir_all(&share_dir)
        .map_err(|e| format!("failed to create share dir {}: {}", share_dir.display(), e))?;

    let mut counters = CampaignShareManifestCounters::default();
    let mut run_entries = Vec::new();
    let mut combined_entries = Vec::new();

    for item in &summary.items {
        if !include_passed && item.status == RegressionStatusV1::Passed {
            continue;
        }

        let share_item = build_campaign_share_manifest_item(CampaignShareManifestItemRequest {
            item,
            root_dir,
            share_dir: &share_dir,
            bundle_ordinal: counters.bundles_total.saturating_add(1),
            stats_top,
            warmup_frames,
        });
        counters.merge(&share_item.counters);
        run_entries.push(share_item.run_entry);
        combined_entries.push(share_item.combined_entry);
    }

    let share_manifest_path = share_dir.join("share.manifest.json");
    let mut payload = build_campaign_share_manifest_payload(CampaignShareManifestPayloadRequest {
        root_dir,
        summary_path,
        workspace_root,
        share_dir: &share_dir,
        summary: &summary,
        include_passed,
        counters: &counters,
        run_entries,
    });
    write_json_value(&share_manifest_path, &payload)?;

    let combined_zip_outcome = write_campaign_combined_failure_zip(
        root_dir,
        &share_dir,
        &share_manifest_path,
        summary_path,
        &combined_entries,
    );
    apply_campaign_share_manifest_combined_zip(&mut payload, &combined_zip_outcome);
    write_json_value(&share_manifest_path, &payload)?;
    Ok(share_manifest_path)
}

fn write_campaign_combined_failure_zip(
    root_dir: &Path,
    share_dir: &Path,
    share_manifest_path: &Path,
    summary_path: &Path,
    entries: &[CampaignCombinedFailureEntry],
) -> CampaignCombinedZipOutcome {
    if !entries
        .iter()
        .any(CampaignCombinedFailureEntry::has_exported_artifact)
    {
        return CampaignCombinedZipOutcome::default();
    }

    let out_path = share_dir.join("combined-failures.zip");
    match write_campaign_combined_failure_zip_inner(
        root_dir,
        &out_path,
        share_manifest_path,
        summary_path,
        entries,
    ) {
        Ok(()) => CampaignCombinedZipOutcome {
            path: Some(out_path),
            error: None,
        },
        Err(error) => CampaignCombinedZipOutcome {
            path: None,
            error: Some(error),
        },
    }
}

fn write_campaign_combined_failure_zip_inner(
    root_dir: &Path,
    out_path: &Path,
    share_manifest_path: &Path,
    summary_path: &Path,
    entries: &[CampaignCombinedFailureEntry],
) -> Result<(), String> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    add_file_to_zip(
        &mut zip,
        share_manifest_path,
        "_root/share.manifest.json",
        options,
    )?;
    add_file_to_zip(
        &mut zip,
        summary_path,
        "_root/regression.summary.json",
        options,
    )?;
    let index_path = root_dir.join(crate::regression_summary::DIAG_REGRESSION_INDEX_FILENAME_V1);
    if index_path.is_file() {
        add_file_to_zip(
            &mut zip,
            &index_path,
            "_root/regression.index.json",
            options,
        )?;
    }

    for (index, entry) in entries.iter().enumerate() {
        let safe_item_id = zip_safe_component(&entry.item_id);
        if let Some(share_zip) = entry.share_zip.as_deref()
            && share_zip.is_file()
        {
            add_file_to_zip(
                &mut zip,
                share_zip,
                &format!("items/{:02}-{safe_item_id}.ai.zip", index + 1),
                options,
            )?;
        }
        if let Some(triage_path) = entry.triage_path.as_deref()
            && triage_path.is_file()
        {
            add_file_to_zip(
                &mut zip,
                triage_path,
                &format!("items/{:02}-{safe_item_id}.triage.json", index + 1),
                options,
            )?;
        }
        if let Some(screenshots_manifest) = entry.screenshots_manifest.as_deref()
            && screenshots_manifest.is_file()
        {
            add_file_to_zip(
                &mut zip,
                screenshots_manifest,
                &format!(
                    "items/{:02}-{safe_item_id}.screenshots.manifest.json",
                    index + 1
                ),
                options,
            )?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
struct CampaignShareManifestCounters {
    bundles_total: usize,
    bundles_packed: usize,
    bundles_missing: usize,
    triage_generated: usize,
    triage_failed: usize,
}

impl CampaignShareManifestCounters {
    fn merge(&mut self, other: &Self) {
        self.bundles_total = self.bundles_total.saturating_add(other.bundles_total);
        self.bundles_packed = self.bundles_packed.saturating_add(other.bundles_packed);
        self.bundles_missing = self.bundles_missing.saturating_add(other.bundles_missing);
        self.triage_generated = self.triage_generated.saturating_add(other.triage_generated);
        self.triage_failed = self.triage_failed.saturating_add(other.triage_failed);
    }
}

#[derive(Debug, Clone)]
struct CampaignCombinedFailureEntry {
    item_id: String,
    share_zip: Option<PathBuf>,
    triage_path: Option<PathBuf>,
    screenshots_manifest: Option<PathBuf>,
}

impl CampaignCombinedFailureEntry {
    fn has_exported_artifact(&self) -> bool {
        self.share_zip.is_some()
            || self.triage_path.is_some()
            || self.screenshots_manifest.is_some()
    }
}

struct CampaignShareManifestItemRequest<'a> {
    item: &'a RegressionItemSummaryV1,
    root_dir: &'a Path,
    share_dir: &'a Path,
    bundle_ordinal: usize,
    stats_top: usize,
    warmup_frames: u64,
}

struct CampaignShareManifestItem {
    counters: CampaignShareManifestCounters,
    run_entry: serde_json::Value,
    combined_entry: CampaignCombinedFailureEntry,
}

struct CampaignShareManifestPayloadRequest<'a> {
    root_dir: &'a Path,
    summary_path: &'a Path,
    workspace_root: &'a Path,
    share_dir: &'a Path,
    summary: &'a RegressionSummaryV1,
    include_passed: bool,
    counters: &'a CampaignShareManifestCounters,
    run_entries: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
struct CampaignCombinedZipOutcome {
    path: Option<PathBuf>,
    error: Option<String>,
}

fn build_campaign_share_manifest_payload(
    request: CampaignShareManifestPayloadRequest<'_>,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_SHARE_MANIFEST_KIND_V1,
        "source": {
            "root_dir": request.root_dir.display().to_string(),
            "summary_path": request.summary_path.display().to_string(),
            "campaign_name": request.summary.campaign.name,
            "lane": request.summary.campaign.lane,
        },
        "selection": {
            "include_passed": request.include_passed,
        },
        "counters": {
            "items_selected": request.run_entries.len(),
            "bundles_total": request.counters.bundles_total,
            "bundles_packed": request.counters.bundles_packed,
            "bundles_missing": request.counters.bundles_missing,
            "triage_generated": request.counters.triage_generated,
            "triage_failed": request.counters.triage_failed,
        },
        "share": {
            "share_dir": request.share_dir.display().to_string(),
            "workflow_hint": format!(
                "open {} in DevTools or share {} plus the generated *.ai.zip artifacts",
                request.root_dir.display(),
                request.summary_path.display()
            ),
            "workspace_root": request.workspace_root.display().to_string(),
            "combined_zip": serde_json::Value::Null,
            "combined_zip_error": serde_json::Value::Null,
        },
        "items": request.run_entries,
    })
}

fn apply_campaign_share_manifest_combined_zip(
    payload: &mut serde_json::Value,
    outcome: &CampaignCombinedZipOutcome,
) {
    if let Some(share) = payload
        .get_mut("share")
        .and_then(|value| value.as_object_mut())
    {
        share.insert(
            "combined_zip".to_string(),
            outcome
                .path
                .as_ref()
                .map(|path| serde_json::Value::String(path.display().to_string()))
                .unwrap_or(serde_json::Value::Null),
        );
        share.insert(
            "combined_zip_error".to_string(),
            outcome
                .error
                .clone()
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );
    }
}

fn build_campaign_share_manifest_item(
    request: CampaignShareManifestItemRequest<'_>,
) -> CampaignShareManifestItem {
    let bundle_dir = request
        .item
        .evidence
        .as_ref()
        .and_then(|evidence| evidence.bundle_dir.as_deref())
        .map(PathBuf::from);
    let mut counters = CampaignShareManifestCounters::default();
    if bundle_dir.is_none() {
        counters.bundles_missing = 1;
    }

    let (triage_path, triage_error) = if let Some(bundle_dir) = bundle_dir.as_deref() {
        maybe_write_bundle_triage_json(bundle_dir, request.stats_top, request.warmup_frames)
    } else {
        (None, None)
    };
    let screenshots_manifest = bundle_dir.as_deref().and_then(|bundle_dir| {
        crate::commands::screenshots::resolve_screenshots_manifest_path(bundle_dir)
            .map(|(_screenshots_dir, manifest_path)| manifest_path)
    });
    if triage_path.is_some() {
        counters.triage_generated = 1;
    }
    if triage_error.is_some() {
        counters.triage_failed = 1;
    }

    let pack_result = if let Some(bundle_dir) = bundle_dir.as_deref() {
        counters.bundles_total = 1;
        let packet_dir = bundle_dir.join("ai.packet");
        let ai_packet_result = crate::commands::ai_packet::ensure_ai_packet_dir_best_effort(
            None,
            bundle_dir,
            &packet_dir,
            true,
            request.stats_top,
            None,
            request.warmup_frames,
            None,
        );
        let share_zip = request.share_dir.join(format!(
            "{:02}-{}.ai.zip",
            request.bundle_ordinal,
            zip_safe_component(&request.item.item_id)
        ));
        match ai_packet_result.and_then(|_| {
            crate::pack_ai_packet_dir_to_zip(bundle_dir, &share_zip, request.root_dir)
        }) {
            Ok(()) => {
                counters.bundles_packed = 1;
                Ok(share_zip)
            }
            Err(error) => Err(error),
        }
    } else {
        Err("item does not expose evidence.bundle_dir".to_string())
    };

    let pack_path = pack_result.as_ref().ok().cloned();
    let pack_error = pack_result.as_ref().err().cloned();
    let run_entry = serde_json::json!({
        "item_id": request.item.item_id,
        "name": request.item.name,
        "status": request.item.status,
        "reason_code": request.item.reason_code,
        "bundle_dir": bundle_dir.as_ref().map(|path| path.display().to_string()),
        "triage_json": triage_path.as_ref().map(|path| path.display().to_string()),
        "triage_error": triage_error,
        "screenshots_manifest": screenshots_manifest.as_ref().map(|path| path.display().to_string()),
        "source_script": request
            .item
            .source
            .as_ref()
            .and_then(|source| source.script.clone()),
        "share_zip": pack_path.as_ref().map(|path| path.display().to_string()),
        "error": pack_error,
    });

    CampaignShareManifestItem {
        counters,
        run_entry,
        combined_entry: CampaignCombinedFailureEntry {
            item_id: request.item.item_id.clone(),
            share_zip: pack_path,
            triage_path,
            screenshots_manifest,
        },
    }
}

fn add_file_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    src: &Path,
    dest: &str,
    options: zip::write::FileOptions,
) -> Result<(), String> {
    use std::io::Write;

    zip.start_file(dest, options).map_err(|e| e.to_string())?;
    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;
    zip.write_all(&bytes).map_err(|e| e.to_string())?;
    Ok(())
}

fn maybe_write_bundle_triage_json(
    bundle_dir: &Path,
    stats_top: usize,
    warmup_frames: u64,
) -> (Option<PathBuf>, Option<String>) {
    let Some(bundle_path) = resolve_bundle_artifact_path_no_materialize(bundle_dir) else {
        return (None, None);
    };
    let triage_path = crate::default_triage_out_path(&bundle_path);
    if triage_path.is_file() {
        return (Some(triage_path), None);
    }
    let sort = BundleStatsSort::Invalidation;
    let report = match bundle_stats_from_path(
        &bundle_path,
        stats_top,
        sort,
        BundleStatsOptions { warmup_frames },
    ) {
        Ok(report) => report,
        Err(error) => {
            return (
                None,
                Some(format!(
                    "failed to generate triage.json for {}: {}",
                    bundle_dir.display(),
                    error
                )),
            );
        }
    };
    let payload = crate::triage_json_from_stats(&bundle_path, &report, sort, warmup_frames);
    match write_json_value(&triage_path, &payload) {
        Ok(()) => (Some(triage_path), None),
        Err(error) => (
            None,
            Some(format!(
                "failed to write triage.json for {}: {}",
                bundle_dir.display(),
                error
            )),
        ),
    }
}

fn campaign_batch_to_json(batch: &CampaignBatchArtifacts) -> serde_json::Value {
    let mut payload = serde_json::Map::new();
    payload.extend(campaign_batch_root_json(batch));
    payload.extend(campaign_batch_paths_json(batch));
    payload.extend(campaign_batch_status_json(batch));
    serde_json::Value::Object(payload)
}

fn campaign_batch_root_json(
    batch: &CampaignBatchArtifacts,
) -> serde_json::Map<String, serde_json::Value> {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "out_dir".to_string(),
        serde_json::json!(batch.batch_root.display().to_string()),
    );
    payload
}

fn campaign_batch_paths_json(
    batch: &CampaignBatchArtifacts,
) -> serde_json::Map<String, serde_json::Value> {
    let paths = campaign_aggregate_path_projection(
        &batch.aggregate,
        CampaignReportPathMode::ResultArtifact,
    );
    let mut payload = serde_json::Map::new();
    payload.insert(
        "summary_path".to_string(),
        serde_json::json!(paths.summary_path),
    );
    payload.insert(
        "index_path".to_string(),
        serde_json::json!(paths.index_path),
    );
    payload.insert(
        "share_manifest_path".to_string(),
        serde_json::json!(paths.share_manifest_path),
    );
    payload
}

fn campaign_batch_status_json(
    batch: &CampaignBatchArtifacts,
) -> serde_json::Map<String, serde_json::Value> {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "summarize_error".to_string(),
        serde_json::json!(batch.aggregate.summarize_error),
    );
    payload.insert(
        "share_error".to_string(),
        serde_json::json!(batch.aggregate.share_error),
    );
    payload
}

fn maybe_write_failure_share_manifest(
    root_dir: &Path,
    summary_path: &Path,
    workspace_root: &Path,
    should_generate: bool,
    stats_top: usize,
    warmup_frames: u64,
) -> (Option<PathBuf>, Option<String>) {
    if !should_generate || !summary_path.is_file() {
        return (None, None);
    }
    match write_campaign_share_manifest(
        root_dir,
        summary_path,
        workspace_root,
        false,
        stats_top,
        warmup_frames,
    ) {
        Ok(path) => (Some(path), None),
        Err(error) => (None, Some(error)),
    }
}

fn campaign_batch_selection_slug(options: &CampaignRunOptions, selected_count: usize) -> String {
    let mut parts = Vec::new();
    if options.campaign_ids.is_empty() {
        parts.push("filtered".to_string());
    } else if options.campaign_ids.len() == 1 {
        parts.push(format!(
            "ids-{}",
            zip_safe_component(&options.campaign_ids[0])
        ));
    } else {
        parts.push(format!("ids-{}", options.campaign_ids.len()));
    }
    if let Some(lane) = options.filter.lane {
        parts.push(format!("lane-{}", lane_to_str(lane)));
    }
    if let Some(tier) = options.filter.tier.as_deref() {
        parts.push(format!("tier-{}", zip_safe_component(tier)));
    }
    if !options.filter.tags.is_empty() {
        if options.filter.tags.len() == 1 {
            parts.push(format!(
                "tag-{}",
                zip_safe_component(&options.filter.tags[0])
            ));
        } else {
            parts.push(format!("tags-{}", options.filter.tags.len()));
        }
    }
    if !options.filter.platforms.is_empty() {
        if options.filter.platforms.len() == 1 {
            parts.push(format!(
                "platform-{}",
                zip_safe_component(&options.filter.platforms[0])
            ));
        } else {
            parts.push(format!("platforms-{}", options.filter.platforms.len()));
        }
    }
    if selected_count > 1 && options.campaign_ids.is_empty() {
        parts.push(format!("{}-campaigns", selected_count));
    }
    let slug = parts.join("-");
    if slug.is_empty() {
        "selection".to_string()
    } else {
        slug
    }
}

fn item_to_manifest_json(item: &CampaignItemDefinition) -> serde_json::Value {
    serde_json::json!({
        "kind": item_kind_str(item.kind),
        "value": item.value,
    })
}

fn campaign_run_record_json(
    run_id: &str,
    created_unix_ms: u64,
    finished_unix_ms: Option<u64>,
    duration_ms: Option<u64>,
    out_dir: &Path,
    workspace_root: &Path,
) -> serde_json::Value {
    let mut run = serde_json::Map::new();
    run.insert("run_id".to_string(), serde_json::json!(run_id));
    run.insert(
        "created_unix_ms".to_string(),
        serde_json::json!(created_unix_ms),
    );
    if let Some(finished_unix_ms) = finished_unix_ms {
        run.insert(
            "finished_unix_ms".to_string(),
            serde_json::json!(finished_unix_ms),
        );
    }
    if let Some(duration_ms) = duration_ms {
        run.insert("duration_ms".to_string(), serde_json::json!(duration_ms));
    }
    run.insert(
        "tool".to_string(),
        serde_json::json!("fretboard diag campaign"),
    );
    run.insert(
        "workspace_root".to_string(),
        serde_json::json!(workspace_root.display().to_string()),
    );
    run.insert(
        "out_dir".to_string(),
        serde_json::json!(out_dir.display().to_string()),
    );
    serde_json::Value::Object(run)
}

fn campaign_selection_json(
    options: &CampaignRunOptions,
    selected_count: usize,
    selected_campaign_ids: Option<Vec<&str>>,
) -> serde_json::Value {
    let mut selection = serde_json::Map::new();
    selection.insert(
        "selection_slug".to_string(),
        serde_json::json!(campaign_batch_selection_slug(options, selected_count)),
    );
    selection.insert(
        "campaign_ids".to_string(),
        serde_json::json!(&options.campaign_ids),
    );
    selection.insert(
        "filters".to_string(),
        campaign_filter_to_json(&options.filter),
    );
    if let Some(selected_campaign_ids) = selected_campaign_ids {
        selection.insert(
            "selected_campaign_ids".to_string(),
            serde_json::json!(selected_campaign_ids),
        );
    }
    serde_json::Value::Object(selection)
}

fn build_campaign_aggregate_artifacts(
    summary_path: &Path,
    index_path: &Path,
    summary_artifacts: &CampaignSummaryArtifacts,
) -> CampaignAggregateArtifacts {
    CampaignAggregateArtifacts {
        summary_path: summary_path.to_path_buf(),
        index_path: index_path.to_path_buf(),
        share_manifest_path: summary_artifacts.share_manifest_path.clone(),
        summarize_error: summary_artifacts.summarize_error.clone(),
        share_error: summary_artifacts.share_error.clone(),
    }
}

fn campaign_aggregate_json(aggregate: &CampaignAggregateArtifacts) -> serde_json::Value {
    serde_json::json!({
        "summary_path": aggregate.summary_path.is_file().then(|| aggregate.summary_path.display().to_string()),
        "index_path": aggregate.index_path.is_file().then(|| aggregate.index_path.display().to_string()),
        "share_manifest_path": aggregate.share_manifest_path.as_ref().map(|path| path.display().to_string()),
        "summarize_error": aggregate.summarize_error.clone(),
        "share_error": aggregate.share_error.clone(),
    })
}

fn campaign_aggregate_path_projection(
    aggregate: &CampaignAggregateArtifacts,
    path_mode: CampaignReportPathMode,
) -> CampaignAggregatePathProjection {
    let summary_path = match path_mode {
        CampaignReportPathMode::RunOutcome => Some(aggregate.summary_path.display().to_string()),
        CampaignReportPathMode::ResultArtifact => aggregate
            .summary_path
            .is_file()
            .then(|| aggregate.summary_path.display().to_string()),
    };
    let index_path = match path_mode {
        CampaignReportPathMode::RunOutcome => Some(aggregate.index_path.display().to_string()),
        CampaignReportPathMode::ResultArtifact => aggregate
            .index_path
            .is_file()
            .then(|| aggregate.index_path.display().to_string()),
    };
    CampaignAggregatePathProjection {
        summary_path,
        index_path,
        share_manifest_path: aggregate
            .share_manifest_path
            .as_ref()
            .map(|path| path.display().to_string()),
    }
}

fn campaign_item_run_result_to_json(entry: &CampaignItemRunResult) -> serde_json::Value {
    serde_json::json!({
        "kind": item_kind_str(entry.kind),
        "item_id": entry.item_id,
        "ok": entry.ok,
        "error": entry.error,
        "out_dir": entry.out_dir.display().to_string(),
        "regression_summary_path": entry
            .regression_summary_path
            .is_file()
            .then(|| entry.regression_summary_path.display().to_string()),
    })
}

fn campaign_manifest_resolved_json(
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
) -> serde_json::Value {
    serde_json::json!({
        "item_count": campaign.items.len(),
        "items": campaign.items.iter().map(item_to_manifest_json).collect::<Vec<_>>(),
        "suite_count": campaign.suite_count(),
        "script_count": campaign.script_count(),
        "suites": campaign.suites(),
        "scripts": campaign.scripts(),
        "launch": &ctx.launch,
        "launch_env": &ctx.launch_env,
    })
}

fn campaign_batch_manifest_resolved_json(
    reports: &[CampaignExecutionReport],
    ctx: &CampaignRunContext,
) -> serde_json::Value {
    let counters = build_campaign_run_counters(reports);
    serde_json::json!({
        "campaigns_total": counters.campaigns_total,
        "items_total": counters.items_total,
        "suites_total": counters.suites_total,
        "scripts_total": counters.scripts_total,
        "runs": reports.iter().map(campaign_report_to_json).collect::<Vec<_>>(),
        "launch": &ctx.launch,
        "launch_env": &ctx.launch_env,
    })
}

fn build_campaign_item_run_counters(
    item_results: &[CampaignItemRunResult],
) -> CampaignItemRunCounters {
    let items_total = item_results.len();
    let items_failed = item_results.iter().filter(|entry| !entry.ok).count();
    let suites_total = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Suite)
        .count();
    let suites_failed = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Suite && !entry.ok)
        .count();
    let scripts_total = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Script)
        .count();
    let scripts_failed = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Script && !entry.ok)
        .count();
    CampaignItemRunCounters {
        items_total,
        items_passed: items_total.saturating_sub(items_failed),
        items_failed,
        suites_total,
        suites_failed,
        scripts_total,
        scripts_failed,
    }
}

fn campaign_item_run_counters_json(counters: CampaignItemRunCounters) -> serde_json::Value {
    serde_json::json!({
        "items_total": counters.items_total,
        "items_passed": counters.items_passed,
        "items_failed": counters.items_failed,
        "suites_total": counters.suites_total,
        "suites_failed": counters.suites_failed,
        "scripts_total": counters.scripts_total,
        "scripts_failed": counters.scripts_failed,
    })
}

fn campaign_batch_result_counters_json(counters: CampaignRunCounters) -> serde_json::Value {
    serde_json::json!({
        "campaigns_total": counters.campaigns_total,
        "campaigns_passed": counters.campaigns_passed,
        "campaigns_failed": counters.campaigns_failed,
        "items_total": counters.items_total,
        "items_passed": counters.items_total.saturating_sub(counters.items_failed),
        "items_failed": counters.items_failed,
        "suites_total": counters.suites_total,
        "scripts_total": counters.scripts_total,
    })
}

fn campaign_report_json(
    report: &CampaignExecutionReport,
    path_mode: CampaignReportPathMode,
) -> serde_json::Value {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "campaign_id".to_string(),
        serde_json::json!(report.campaign_id),
    );
    payload.extend(campaign_report_status_json(report));
    payload.extend(campaign_report_paths_json(report, path_mode));
    payload.extend(campaign_report_counters_json(report));
    serde_json::Value::Object(payload)
}

fn campaign_report_status_json(
    report: &CampaignExecutionReport,
) -> serde_json::Map<String, serde_json::Value> {
    let mut payload = serde_json::Map::new();
    payload.insert("ok".to_string(), serde_json::json!(report.ok));
    payload.insert("error".to_string(), serde_json::json!(report.error));
    payload.insert(
        "share_error".to_string(),
        serde_json::json!(report.aggregate.share_error),
    );
    payload
}

fn campaign_report_paths_json(
    report: &CampaignExecutionReport,
    path_mode: CampaignReportPathMode,
) -> serde_json::Map<String, serde_json::Value> {
    let paths = campaign_aggregate_path_projection(&report.aggregate, path_mode);
    let mut payload = serde_json::Map::new();
    payload.insert(
        "out_dir".to_string(),
        serde_json::json!(report.out_dir.display().to_string()),
    );
    if path_mode == CampaignReportPathMode::ResultArtifact {
        payload.insert(
            "campaign_result_path".to_string(),
            serde_json::json!(
                report
                    .out_dir
                    .join("campaign.result.json")
                    .display()
                    .to_string()
            ),
        );
    }
    payload.insert(
        "summary_path".to_string(),
        serde_json::json!(paths.summary_path),
    );
    payload.insert(
        "index_path".to_string(),
        serde_json::json!(paths.index_path),
    );
    payload.insert(
        "share_manifest_path".to_string(),
        serde_json::json!(paths.share_manifest_path),
    );
    payload
}

fn campaign_report_counters_json(
    report: &CampaignExecutionReport,
) -> serde_json::Map<String, serde_json::Value> {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "items_total".to_string(),
        serde_json::json!(report.items_total),
    );
    payload.insert(
        "items_failed".to_string(),
        serde_json::json!(report.items_failed),
    );
    payload.insert(
        "suites_total".to_string(),
        serde_json::json!(report.suites_total),
    );
    payload.insert(
        "scripts_total".to_string(),
        serde_json::json!(report.scripts_total),
    );
    payload
}

fn campaign_report_to_json(report: &CampaignExecutionReport) -> serde_json::Value {
    campaign_report_json(report, CampaignReportPathMode::ResultArtifact)
}

fn build_campaign_manifest_write_plan(
    plan: &CampaignExecutionPlan,
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
) -> CampaignJsonWritePlan {
    CampaignJsonWritePlan {
        output_path: plan.campaign_root.join("campaign.manifest.json"),
        payload: campaign_manifest_payload(
            &plan.campaign_root,
            campaign,
            &plan.run_id,
            plan.created_unix_ms,
            ctx,
        ),
    }
}

fn build_campaign_batch_manifest_write_plan(
    plan: &CampaignBatchPlan,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    ctx: &CampaignRunContext,
) -> CampaignJsonWritePlan {
    CampaignJsonWritePlan {
        output_path: plan.batch_root.join("batch.manifest.json"),
        payload: campaign_batch_manifest_payload(
            &plan.batch_root,
            &plan.run_id,
            plan.created_unix_ms,
            reports,
            options,
            ctx,
        ),
    }
}

fn campaign_manifest_payload(
    campaign_root: &Path,
    campaign: &CampaignDefinition,
    run_id: &str,
    created_unix_ms: u64,
    ctx: &CampaignRunContext,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_MANIFEST_KIND_V1,
        "campaign": campaign_to_json(campaign),
        "run": campaign_run_record_json(
            run_id,
            created_unix_ms,
            None,
            None,
            campaign_root,
            &ctx.workspace_root,
        ),
        "resolved": campaign_manifest_resolved_json(campaign, ctx)
    })
}

fn campaign_batch_manifest_payload(
    batch_root: &Path,
    run_id: &str,
    created_unix_ms: u64,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    ctx: &CampaignRunContext,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_BATCH_MANIFEST_KIND_V1,
        "selection": campaign_selection_json(
            options,
            reports.len(),
            Some(
                reports
                    .iter()
                    .map(|report| report.campaign_id.as_str())
                    .collect::<Vec<_>>(),
            ),
        ),
        "run": campaign_run_record_json(
            run_id,
            created_unix_ms,
            None,
            None,
            batch_root,
            &ctx.workspace_root,
        ),
        "resolved": campaign_batch_manifest_resolved_json(reports, ctx)
    })
}

fn write_campaign_result(
    plan: &CampaignExecutionPlan,
    campaign: &CampaignDefinition,
    item_results: &[CampaignItemRunResult],
    summary_artifacts: &CampaignSummaryArtifacts,
    ctx: &CampaignRunContext,
) -> Result<(), String> {
    let write_plan =
        build_campaign_result_write_plan(plan, campaign, item_results, summary_artifacts, ctx);
    write_campaign_json_plan(&write_plan)
}

fn write_campaign_batch_result(
    plan: &CampaignBatchPlan,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    summary_artifacts: &CampaignSummaryArtifacts,
    ctx: &CampaignRunContext,
) -> Result<(), String> {
    let write_plan =
        build_campaign_batch_result_write_plan(plan, reports, options, summary_artifacts, ctx);
    write_campaign_json_plan(&write_plan)
}

fn build_campaign_result_write_plan(
    plan: &CampaignExecutionPlan,
    campaign: &CampaignDefinition,
    item_results: &[CampaignItemRunResult],
    summary_artifacts: &CampaignSummaryArtifacts,
    ctx: &CampaignRunContext,
) -> CampaignJsonWritePlan {
    CampaignJsonWritePlan {
        output_path: plan.campaign_root.join("campaign.result.json"),
        payload: campaign_result_payload(campaign, plan, item_results, summary_artifacts, ctx),
    }
}

fn build_campaign_batch_result_write_plan(
    plan: &CampaignBatchPlan,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    summary_artifacts: &CampaignSummaryArtifacts,
    ctx: &CampaignRunContext,
) -> CampaignJsonWritePlan {
    CampaignJsonWritePlan {
        output_path: plan.batch_root.join("batch.result.json"),
        payload: campaign_batch_result_payload(plan, reports, options, summary_artifacts, ctx),
    }
}

fn write_campaign_json_plan(plan: &CampaignJsonWritePlan) -> Result<(), String> {
    write_json_value(&plan.output_path, &plan.payload)
}

fn campaign_result_payload(
    campaign: &CampaignDefinition,
    plan: &CampaignExecutionPlan,
    item_results: &[CampaignItemRunResult],
    summary_artifacts: &CampaignSummaryArtifacts,
    ctx: &CampaignRunContext,
) -> serde_json::Value {
    let sections =
        build_campaign_result_payload_sections(plan, item_results, summary_artifacts, ctx);
    serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_RESULT_KIND_V1,
        "campaign": campaign_to_json(campaign),
        "run": sections.run,
        "counters": sections.counters,
        "aggregate": sections.aggregate,
        "item_results": sections.item_results,
    })
}

fn campaign_batch_result_payload(
    plan: &CampaignBatchPlan,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    summary_artifacts: &CampaignSummaryArtifacts,
    ctx: &CampaignRunContext,
) -> serde_json::Value {
    let sections = build_campaign_batch_result_payload_sections(
        plan,
        reports,
        options,
        summary_artifacts,
        ctx,
    );
    serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_BATCH_RESULT_KIND_V1,
        "selection": sections.selection,
        "run": sections.run,
        "counters": sections.counters,
        "aggregate": sections.aggregate,
        "runs": sections.runs,
    })
}

fn build_campaign_result_payload_sections(
    plan: &CampaignExecutionPlan,
    item_results: &[CampaignItemRunResult],
    summary_artifacts: &CampaignSummaryArtifacts,
    ctx: &CampaignRunContext,
) -> CampaignResultPayloadSections {
    let counters = build_campaign_item_run_counters(item_results);
    CampaignResultPayloadSections {
        run: campaign_result_run_json(
            &plan.run_id,
            plan.created_unix_ms,
            &plan.campaign_root,
            summary_artifacts,
            &ctx.workspace_root,
        ),
        counters: campaign_item_run_counters_json(counters),
        aggregate: campaign_result_aggregate_json(
            &plan.summary_path,
            &plan.index_path,
            summary_artifacts,
        ),
        item_results: item_results
            .iter()
            .map(campaign_item_run_result_to_json)
            .collect(),
    }
}

fn build_campaign_batch_result_payload_sections(
    plan: &CampaignBatchPlan,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    summary_artifacts: &CampaignSummaryArtifacts,
    ctx: &CampaignRunContext,
) -> CampaignBatchResultPayloadSections {
    let counters = build_campaign_run_counters(reports);
    CampaignBatchResultPayloadSections {
        selection: campaign_selection_json(options, reports.len(), None),
        run: campaign_result_run_json(
            &plan.run_id,
            plan.created_unix_ms,
            &plan.batch_root,
            summary_artifacts,
            &ctx.workspace_root,
        ),
        counters: campaign_batch_result_counters_json(counters),
        aggregate: campaign_result_aggregate_json(
            &plan.summary_path,
            &plan.index_path,
            summary_artifacts,
        ),
        runs: reports.iter().map(campaign_report_to_json).collect(),
    }
}

fn campaign_result_run_json(
    run_id: &str,
    created_unix_ms: u64,
    out_dir: &Path,
    summary_artifacts: &CampaignSummaryArtifacts,
    workspace_root: &Path,
) -> serde_json::Value {
    campaign_run_record_json(
        run_id,
        created_unix_ms,
        Some(summary_artifacts.finished_unix_ms),
        Some(summary_artifacts.duration_ms),
        out_dir,
        workspace_root,
    )
}

fn campaign_result_aggregate_json(
    summary_path: &Path,
    index_path: &Path,
    summary_artifacts: &CampaignSummaryArtifacts,
) -> serde_json::Value {
    let aggregate = build_campaign_aggregate_artifacts(summary_path, index_path, summary_artifacts);
    campaign_aggregate_json(&aggregate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regression_summary::{
        RegressionArtifactsV1, RegressionCampaignSummaryV1, RegressionEvidenceV1,
        RegressionHighlightsV1, RegressionItemKindV1, RegressionItemSummaryV1, RegressionLaneV1,
        RegressionRunSummaryV1, RegressionSummaryV1, RegressionTotalsV1,
    };

    fn sample_campaign_cmd_context(root: &Path, rest: Vec<String>) -> CampaignCmdContext {
        CampaignCmdContext {
            pack_after_run: false,
            rest,
            suite_script_inputs: vec!["shared-input.json".to_string()],
            suite_prewarm_scripts: vec![PathBuf::from("prewarm.json")],
            suite_prelude_scripts: vec![PathBuf::from("prelude.json")],
            suite_prelude_each_run: true,
            workspace_root: root.to_path_buf(),
            resolved_out_dir: root.join("diag-out"),
            devtools_ws_url: Some("ws://localhost:1234".to_string()),
            devtools_token: Some("token".to_string()),
            devtools_session_id: Some("session".to_string()),
            timeout_ms: 1000,
            poll_ms: 5,
            stats_top: 20,
            stats_json: true,
            warmup_frames: 4,
            max_test_ids: 100,
            lint_all_test_ids_bounds: false,
            lint_eps_px: 0.5,
            suite_lint: false,
            pack_include_screenshots: false,
            reuse_launch: false,
            launch: Some(vec!["cargo".to_string(), "run".to_string()]),
            launch_env: vec![("BASE".to_string(), "1".to_string())],
            launch_high_priority: false,
            launch_write_bundle_json: false,
            keep_open: false,
            checks: diag_suite::SuiteChecks::default(),
        }
    }

    #[test]
    fn resolve_campaign_subcommand_rejects_missing_and_unknown_values() {
        let missing = resolve_campaign_subcommand(&[]).unwrap_err();
        let unknown = resolve_campaign_subcommand(&["mystery".to_string()]).unwrap_err();

        assert!(missing.contains("missing campaign subcommand"));
        assert!(unknown.contains("unknown diag campaign subcommand: mystery"));
    }

    #[test]
    fn campaign_run_context_from_cmd_context_preserves_runtime_fields() {
        let root = PathBuf::from("diag-root");
        let cmd_ctx = sample_campaign_cmd_context(
            &root,
            vec!["run".to_string(), "ui-gallery-smoke".to_string()],
        );

        let run_ctx: CampaignRunContext = cmd_ctx.into();

        assert_eq!(run_ctx.workspace_root, root);
        assert_eq!(
            run_ctx.suite_script_inputs,
            vec!["shared-input.json".to_string()]
        );
        assert_eq!(
            run_ctx.devtools_ws_url.as_deref(),
            Some("ws://localhost:1234")
        );
        assert_eq!(
            run_ctx.launch,
            Some(vec!["cargo".to_string(), "run".to_string()])
        );
        assert!(run_ctx.stats_json);
    }

    #[test]
    fn normalize_campaign_execution_outcome_converts_err_to_failed_items() {
        let outcome = normalize_campaign_execution_outcome(Err("boom".to_string()), 3);

        assert_eq!(outcome.items_failed, 3);
        assert_eq!(outcome.error.as_deref(), Some("boom"));
        assert!(outcome.share_manifest_path.is_none());
        assert!(outcome.share_error.is_none());
    }

    #[test]
    fn build_campaign_execution_report_from_outcome_result_normalizes_err_before_building_report() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);

        let report = build_campaign_execution_report_from_outcome_result(
            &campaign,
            &plan,
            Err("boom".to_string()),
        );

        assert_eq!(report.campaign_id, "ui-gallery-smoke");
        assert_eq!(report.out_dir, plan.campaign_root);
        assert_eq!(report.items_total, 1);
        assert_eq!(report.items_failed, 1);
        assert_eq!(report.suites_total, 1);
        assert_eq!(report.scripts_total, 0);
        assert!(!report.ok);
        assert_eq!(report.error.as_deref(), Some("boom"));
        assert!(report.aggregate.share_manifest_path.is_none());
        assert!(report.aggregate.share_error.is_none());
    }

    #[test]
    fn build_campaign_execution_report_uses_plan_and_campaign_counts() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);
        let outcome = CampaignExecutionOutcome {
            items_failed: 1,
            error: Some("failed".to_string()),
            share_manifest_path: Some(PathBuf::from("share/manifest.json")),
            share_error: Some("share failed".to_string()),
        };

        let report = build_campaign_execution_report(&campaign, &plan, outcome);

        assert_eq!(report.campaign_id, "ui-gallery-smoke");
        assert_eq!(report.out_dir, plan.campaign_root);
        assert_eq!(report.aggregate.summary_path, plan.summary_path);
        assert_eq!(report.aggregate.index_path, plan.index_path);
        assert_eq!(report.items_total, 1);
        assert_eq!(report.items_failed, 1);
        assert_eq!(report.suites_total, 1);
        assert_eq!(report.scripts_total, 0);
        assert!(!report.ok);
        assert_eq!(report.error.as_deref(), Some("failed"));
        assert_eq!(
            report.aggregate.share_error.as_deref(),
            Some("share failed")
        );
        assert_eq!(
            report.aggregate.share_manifest_path,
            Some(PathBuf::from("share/manifest.json"))
        );
    }

    #[test]
    fn campaign_run_selection_json_matches_existing_shape() {
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions {
                lane: Some(RegressionLaneV1::Smoke),
                tier: Some("smoke".to_string()),
                tags: vec!["ui-gallery".to_string()],
                platforms: vec!["native".to_string()],
            },
        };

        let selection = campaign_run_selection_json(&options);

        assert_eq!(
            selection
                .get("campaign_ids")
                .and_then(|value| value.as_array())
                .map(|items| items.len()),
            Some(1)
        );
        assert!(selection.get("selection_slug").is_none());
        assert_eq!(
            selection
                .get("filters")
                .and_then(|value| value.get("tier"))
                .and_then(|value| value.as_str()),
            Some("smoke")
        );
    }

    #[test]
    fn apply_campaign_run_flag_sets_expected_filter_fields() {
        let mut options = CampaignRunOptions::default();

        apply_campaign_run_flag(&mut options, "--lane", "smoke").unwrap();
        apply_campaign_run_flag(&mut options, "--tier", "smoke").unwrap();
        apply_campaign_run_flag(&mut options, "--tag", "ui-gallery").unwrap();
        apply_campaign_run_flag(&mut options, "--platform", "native").unwrap();

        assert_eq!(options.filter.lane, Some(RegressionLaneV1::Smoke));
        assert_eq!(options.filter.tier.as_deref(), Some("smoke"));
        assert_eq!(options.filter.tags, vec!["ui-gallery".to_string()]);
        assert_eq!(options.filter.platforms, vec!["native".to_string()]);
    }

    #[test]
    fn select_explicit_campaigns_for_run_requires_match_after_filtering() {
        let registry = CampaignRegistry::builtin();
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions {
                lane: None,
                tier: None,
                tags: vec!["nonexistent-tag".to_string()],
                platforms: Vec::new(),
            },
        };

        let error = select_explicit_campaigns_for_run(&registry, &options).unwrap_err();

        assert!(error.contains("explicit campaign ids were provided but none matched"));
    }

    #[test]
    fn parse_campaign_run_options_collects_ids_and_filters() {
        let options = parse_campaign_run_options(&[
            "ui-gallery-smoke".to_string(),
            "--tag".to_string(),
            "ui-gallery".to_string(),
            "--platform".to_string(),
            "native".to_string(),
        ])
        .unwrap();

        assert_eq!(options.campaign_ids, vec!["ui-gallery-smoke".to_string()]);
        assert_eq!(options.filter.tags, vec!["ui-gallery".to_string()]);
        assert_eq!(options.filter.platforms, vec!["native".to_string()]);
    }

    #[test]
    fn select_campaigns_for_run_supports_filter_only_selection() {
        let registry = CampaignRegistry::builtin();
        let options = CampaignRunOptions {
            campaign_ids: Vec::new(),
            filter: CampaignFilterOptions {
                lane: Some(RegressionLaneV1::Smoke),
                tier: Some("smoke".to_string()),
                tags: vec!["ui-gallery".to_string()],
                platforms: vec!["native".to_string()],
            },
        };

        let selected = select_campaigns_for_run(&registry, &options).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "ui-gallery-smoke");
    }

    #[test]
    fn campaign_batch_selection_slug_tracks_explicit_ids() {
        let slug = campaign_batch_selection_slug(
            &CampaignRunOptions {
                campaign_ids: vec!["ui-gallery-smoke".to_string(), "docking-smoke".to_string()],
                filter: CampaignFilterOptions::default(),
            },
            2,
        );

        assert_eq!(slug, "ids-2");
    }

    #[test]
    fn campaign_batch_selection_slug_tracks_filters() {
        let slug = campaign_batch_selection_slug(
            &CampaignRunOptions {
                campaign_ids: Vec::new(),
                filter: CampaignFilterOptions {
                    lane: Some(RegressionLaneV1::Smoke),
                    tier: Some("smoke".to_string()),
                    tags: vec!["ui-gallery".to_string()],
                    platforms: vec!["native".to_string()],
                },
            },
            2,
        );

        assert_eq!(
            slug,
            "filtered-lane-smoke-tier-smoke-tag-ui-gallery-platform-native-2-campaigns"
        );
    }

    #[test]
    fn parse_campaign_share_options_collects_source_and_flag() {
        let options = parse_campaign_share_options(&[
            "target/fret-diag/campaigns/ui-gallery-smoke/1234".to_string(),
            "--include-passed".to_string(),
        ])
        .unwrap();

        assert_eq!(
            options.source,
            "target/fret-diag/campaigns/ui-gallery-smoke/1234"
        );
        assert!(options.include_passed);
    }

    #[test]
    fn build_campaign_item_plan_and_suite_context_map_suite_and_script_items() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-campaign-invocation-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let ctx = CampaignRunContext {
            pack_after_run: false,
            suite_script_inputs: vec!["shared-input.json".to_string()],
            suite_prewarm_scripts: vec![PathBuf::from("prewarm.json")],
            suite_prelude_scripts: vec![PathBuf::from("prelude.json")],
            suite_prelude_each_run: true,
            workspace_root: root.clone(),
            resolved_out_dir: root.join("campaign-run"),
            devtools_ws_url: None,
            devtools_token: None,
            devtools_session_id: None,
            timeout_ms: 1000,
            poll_ms: 5,
            stats_top: 20,
            stats_json: false,
            warmup_frames: 4,
            max_test_ids: 100,
            lint_all_test_ids_bounds: false,
            lint_eps_px: 0.5,
            suite_lint: false,
            pack_include_screenshots: false,
            reuse_launch: false,
            launch: None,
            launch_env: vec![("BASE".to_string(), "1".to_string())],
            launch_high_priority: false,
            launch_write_bundle_json: false,
            keep_open: false,
            checks: diag_suite::SuiteChecks::default(),
        };

        let suite_plan = build_campaign_item_execution_plan(
            0,
            &CampaignItemDefinition {
                kind: CampaignItemKind::Suite,
                value: "ui-gallery-smoke".to_string(),
            },
            &root.join("suites"),
            &root.join("scripts"),
            &ctx,
        )
        .unwrap();
        let suite_ctx = build_campaign_item_suite_context(&suite_plan, &ctx);
        assert_eq!(suite_plan.kind, CampaignItemKind::Suite);
        assert_eq!(suite_plan.item_id, "ui-gallery-smoke");
        assert_eq!(suite_ctx.rest, vec!["ui-gallery-smoke".to_string()]);
        assert_eq!(
            suite_ctx.suite_script_inputs,
            vec!["shared-input.json".to_string()]
        );

        let script_plan = build_campaign_item_execution_plan(
            1,
            &CampaignItemDefinition {
                kind: CampaignItemKind::Script,
                value: "tools/diag-scripts/demo.json".to_string(),
            },
            &root.join("suites"),
            &root.join("scripts"),
            &ctx,
        )
        .unwrap();
        let script_ctx = build_campaign_item_suite_context(&script_plan, &ctx);
        assert_eq!(script_plan.kind, CampaignItemKind::Script);
        assert!(script_ctx.rest.is_empty());
        assert_eq!(
            script_ctx.suite_script_inputs,
            vec!["tools/diag-scripts/demo.json".to_string()]
        );
        assert_eq!(
            script_ctx.launch_env,
            vec![("BASE".to_string(), "1".to_string())]
        );
    }

    #[test]
    fn build_campaign_item_execution_plan_maps_suite_and_script_items() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-campaign-item-plan-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let ctx = sample_campaign_run_context(&root);
        let suite_root = root.join("suites");
        let script_root = root.join("scripts");

        let suite_plan = build_campaign_item_execution_plan(
            0,
            &CampaignItemDefinition {
                kind: CampaignItemKind::Suite,
                value: "ui-gallery-smoke".to_string(),
            },
            &suite_root,
            &script_root,
            &ctx,
        )
        .unwrap();
        assert_eq!(suite_plan.kind, CampaignItemKind::Suite);
        assert_eq!(suite_plan.item_id, "ui-gallery-smoke");
        assert_eq!(suite_plan.out_dir, suite_root.join("01-ui-gallery-smoke"));
        assert_eq!(suite_plan.suite_rest, vec!["ui-gallery-smoke".to_string()]);
        assert_eq!(
            suite_plan.suite_script_inputs,
            vec!["shared-input.json".to_string()]
        );

        let script_plan = build_campaign_item_execution_plan(
            1,
            &CampaignItemDefinition {
                kind: CampaignItemKind::Script,
                value: "tools/diag-scripts/demo.json".to_string(),
            },
            &suite_root,
            &script_root,
            &ctx,
        )
        .unwrap();
        assert_eq!(script_plan.kind, CampaignItemKind::Script);
        assert_eq!(script_plan.item_id, "tools/diag-scripts/demo.json");
        assert_eq!(
            script_plan.out_dir,
            script_root.join(format!(
                "02-{}",
                zip_safe_component("tools/diag-scripts/demo.json")
            ))
        );
        assert!(script_plan.suite_rest.is_empty());
        assert_eq!(
            script_plan.suite_script_inputs,
            vec!["tools/diag-scripts/demo.json".to_string()]
        );
    }

    #[test]
    fn build_campaign_item_execution_plans_preserves_order_and_result_roots() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-campaign-item-plans-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let ctx = sample_campaign_run_context(&root);
        let campaign = CampaignDefinition {
            id: "mixed-campaign".to_string(),
            description: "sample".to_string(),
            lane: crate::regression_summary::RegressionLaneV1::Smoke,
            profile: Some("bounded".to_string()),
            items: vec![
                CampaignItemDefinition {
                    kind: CampaignItemKind::Suite,
                    value: "ui-gallery-smoke".to_string(),
                },
                CampaignItemDefinition {
                    kind: CampaignItemKind::Script,
                    value: "tools/diag-scripts/demo.json".to_string(),
                },
            ],
            owner: None,
            platforms: vec!["native".to_string()],
            tier: Some("smoke".to_string()),
            expected_duration_ms: None,
            tags: vec!["ui-gallery".to_string()],
            source: crate::registry::campaigns::CampaignDefinitionSource::Builtin,
        };
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);

        let item_plans = build_campaign_item_execution_plans(&campaign, &plan, &ctx).unwrap();

        assert_eq!(item_plans.len(), 2);
        assert_eq!(item_plans[0].kind, CampaignItemKind::Suite);
        assert_eq!(item_plans[0].item_id, "ui-gallery-smoke");
        assert_eq!(
            item_plans[0].out_dir,
            plan.suite_results_root.join("01-ui-gallery-smoke")
        );
        assert_eq!(
            item_plans[0].suite_rest,
            vec!["ui-gallery-smoke".to_string()]
        );

        assert_eq!(item_plans[1].kind, CampaignItemKind::Script);
        assert_eq!(item_plans[1].item_id, "tools/diag-scripts/demo.json");
        assert_eq!(
            item_plans[1].out_dir,
            plan.script_results_root.join(format!(
                "02-{}",
                zip_safe_component("tools/diag-scripts/demo.json")
            ))
        );
        assert!(item_plans[1].suite_rest.is_empty());
        assert_eq!(
            item_plans[1].suite_script_inputs,
            vec!["tools/diag-scripts/demo.json".to_string()]
        );
    }

    #[test]
    fn build_campaign_item_suite_context_preserves_runtime_flags_checks_and_paths() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-campaign-item-suite-ctx-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let mut ctx = sample_campaign_run_context(&root);
        ctx.pack_after_run = true;
        ctx.reuse_launch = true;
        ctx.launch = Some(vec!["demo-bin".to_string()]);
        ctx.checks.check_semantics_changed_repainted = true;
        ctx.checks.check_hover_layout_max = Some(3);

        let plan = CampaignItemExecutionPlan {
            kind: CampaignItemKind::Suite,
            item_id: "ui-gallery-smoke".to_string(),
            out_dir: root.join("suites").join("01-ui-gallery-smoke"),
            regression_summary_path: root
                .join("suites")
                .join("01-ui-gallery-smoke")
                .join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1),
            suite_rest: vec!["ui-gallery-smoke".to_string()],
            suite_script_inputs: vec!["shared-input.json".to_string()],
        };

        let suite_ctx = build_campaign_item_suite_context(&plan, &ctx);

        assert!(suite_ctx.pack_after_run);
        assert!(suite_ctx.reuse_launch);
        assert_eq!(suite_ctx.launch, Some(vec!["demo-bin".to_string()]));
        assert!(suite_ctx.checks.check_semantics_changed_repainted);
        assert_eq!(suite_ctx.checks.check_hover_layout_max, Some(3));
        assert_eq!(suite_ctx.rest, plan.suite_rest);
        assert_eq!(suite_ctx.suite_script_inputs, plan.suite_script_inputs);
        assert_eq!(suite_ctx.resolved_paths.out_dir, plan.out_dir);
    }

    #[test]
    fn build_campaign_item_run_result_preserves_projection_for_ok_and_err_results() {
        let success = build_campaign_item_run_result(
            CampaignItemKind::Suite,
            "ui-gallery-smoke".to_string(),
            PathBuf::from("runs/ui-gallery-smoke"),
            PathBuf::from("runs/ui-gallery-smoke/regression.summary.json"),
            Ok(()),
        );
        let failure = build_campaign_item_run_result(
            CampaignItemKind::Script,
            "tools/diag-scripts/demo.json".to_string(),
            PathBuf::from("runs/demo"),
            PathBuf::from("runs/demo/regression.summary.json"),
            Err("script failed".to_string()),
        );

        assert!(success.ok);
        assert!(success.error.is_none());
        assert_eq!(success.kind, CampaignItemKind::Suite);
        assert_eq!(success.item_id, "ui-gallery-smoke");
        assert_eq!(success.out_dir, PathBuf::from("runs/ui-gallery-smoke"));

        assert!(!failure.ok);
        assert_eq!(failure.error.as_deref(), Some("script failed"));
        assert_eq!(failure.kind, CampaignItemKind::Script);
        assert_eq!(failure.item_id, "tools/diag-scripts/demo.json");
        assert_eq!(failure.out_dir, PathBuf::from("runs/demo"));
        assert_eq!(
            failure.regression_summary_path,
            PathBuf::from("runs/demo/regression.summary.json")
        );
    }

    fn sample_campaign_run_context(root: &Path) -> CampaignRunContext {
        CampaignRunContext {
            pack_after_run: false,
            suite_script_inputs: vec!["shared-input.json".to_string()],
            suite_prewarm_scripts: vec![PathBuf::from("prewarm.json")],
            suite_prelude_scripts: vec![PathBuf::from("prelude.json")],
            suite_prelude_each_run: true,
            workspace_root: root.to_path_buf(),
            resolved_out_dir: root.join("diag-out"),
            devtools_ws_url: None,
            devtools_token: None,
            devtools_session_id: None,
            timeout_ms: 1000,
            poll_ms: 5,
            stats_top: 20,
            stats_json: false,
            warmup_frames: 4,
            max_test_ids: 100,
            lint_all_test_ids_bounds: false,
            lint_eps_px: 0.5,
            suite_lint: false,
            pack_include_screenshots: false,
            reuse_launch: false,
            launch: None,
            launch_env: vec![("BASE".to_string(), "1".to_string())],
            launch_high_priority: false,
            launch_write_bundle_json: false,
            keep_open: false,
            checks: diag_suite::SuiteChecks::default(),
        }
    }

    fn sample_campaign_definition() -> CampaignDefinition {
        CampaignDefinition {
            id: "ui-gallery-smoke".to_string(),
            description: "sample".to_string(),
            lane: crate::regression_summary::RegressionLaneV1::Smoke,
            profile: Some("bounded".to_string()),
            items: vec![CampaignItemDefinition {
                kind: CampaignItemKind::Suite,
                value: "ui-gallery-lite-smoke".to_string(),
            }],
            owner: None,
            platforms: vec!["native".to_string()],
            tier: Some("smoke".to_string()),
            expected_duration_ms: None,
            tags: vec!["ui-gallery".to_string()],
            source: crate::registry::campaigns::CampaignDefinitionSource::Builtin,
        }
    }

    fn sample_campaign_execution_report(
        campaign_id: &str,
        ok: bool,
        items_total: usize,
        items_failed: usize,
    ) -> CampaignExecutionReport {
        CampaignExecutionReport {
            campaign_id: campaign_id.to_string(),
            out_dir: PathBuf::from(format!("runs/{campaign_id}")),
            aggregate: CampaignAggregateArtifacts {
                summary_path: PathBuf::from(format!("runs/{campaign_id}/regression.summary.json")),
                index_path: PathBuf::from(format!("runs/{campaign_id}/regression.index.json")),
                share_manifest_path: None,
                summarize_error: None,
                share_error: None,
            },
            items_total,
            items_failed,
            suites_total: items_total,
            scripts_total: usize::from(items_total > 0),
            ok,
            error: (!ok).then(|| format!("{campaign_id} failed")),
        }
    }

    fn sample_campaign_item_run_result(
        kind: CampaignItemKind,
        item_id: &str,
        ok: bool,
    ) -> CampaignItemRunResult {
        CampaignItemRunResult {
            kind,
            item_id: item_id.to_string(),
            out_dir: PathBuf::from(format!("runs/{item_id}")),
            regression_summary_path: PathBuf::from(format!(
                "runs/{item_id}/regression.summary.json"
            )),
            ok,
            error: (!ok).then(|| format!("{item_id} failed")),
        }
    }

    #[test]
    fn campaign_single_run_output_lines_cover_success_and_failed_evidence() {
        let success_report = sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0);
        let mut failed_report = sample_campaign_execution_report("docking-smoke", false, 5, 2);
        failed_report.aggregate.share_manifest_path =
            Some(PathBuf::from("runs/docking-smoke/share.manifest.json"));

        let success_lines = campaign_single_run_output_lines(&success_report);
        let failure_lines = campaign_single_run_output_lines(&failed_report);

        assert_eq!(success_lines.len(), 1);
        assert!(success_lines[0].contains("campaign: ok (id=ui-gallery-smoke"));
        assert_eq!(failure_lines.len(), 1);
        assert!(failure_lines[0].contains("campaign: failed evidence exported (id=docking-smoke"));
        assert!(failure_lines[0].contains("share.manifest.json"));
    }

    #[test]
    fn campaign_batch_run_output_lines_include_batch_and_report_share_manifest_paths() {
        let mut report_with_share =
            sample_campaign_execution_report("ui-gallery-smoke", false, 3, 1);
        report_with_share.aggregate.share_manifest_path =
            Some(PathBuf::from("runs/ui-gallery-smoke/share.manifest.json"));
        let report_without_share = sample_campaign_execution_report("docking-smoke", true, 5, 0);
        let batch = CampaignBatchArtifacts {
            batch_root: PathBuf::from("batch/root"),
            aggregate: CampaignAggregateArtifacts {
                summary_path: PathBuf::from("batch/root/regression.summary.json"),
                index_path: PathBuf::from("batch/root/regression.index.json"),
                share_manifest_path: Some(PathBuf::from("batch/root/share.manifest.json")),
                summarize_error: None,
                share_error: None,
            },
        };
        let reports = vec![report_with_share, report_without_share];
        let outcome = CampaignRunOutcome {
            counters: build_campaign_run_counters(&reports),
            reports,
            batch: Some(batch),
            command_failures: Vec::new(),
        };

        let lines = campaign_batch_run_output_lines(&outcome);

        assert_eq!(
            lines.first().map(String::as_str),
            Some("campaign batch: 2 run(s), 1 failed")
        );
        assert!(lines.iter().any(|line| line == "  batch_root: batch/root"));
        assert!(
            lines
                .iter()
                .any(|line| line == "  share_manifest: batch/root/share.manifest.json")
        );
        assert!(lines.iter().any(|line| {
            line.contains("  - ui-gallery-smoke [failed] items=3 failed=1 -> runs/ui-gallery-smoke")
        }));
        assert!(lines.iter().any(|line| line == "    share_manifest: runs/ui-gallery-smoke/share.manifest.json"));
        assert!(lines.iter().any(|line| {
            line.contains("  - docking-smoke [ok] items=5 failed=0 -> runs/docking-smoke")
        }));
    }

    #[test]
    fn campaign_report_json_uses_requested_path_mode() {
        let report = sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0);

        let run_json = campaign_report_json(&report, CampaignReportPathMode::RunOutcome);
        let result_json = campaign_report_json(&report, CampaignReportPathMode::ResultArtifact);

        assert!(run_json.get("campaign_result_path").is_none());
        let expected_summary_path = PathBuf::from("runs/ui-gallery-smoke/regression.summary.json")
            .display()
            .to_string();
        assert_eq!(
            run_json
                .get("summary_path")
                .and_then(|value| value.as_str()),
            Some(expected_summary_path.as_str())
        );
        assert!(result_json.get("campaign_result_path").is_some());
        assert!(
            result_json
                .get("summary_path")
                .is_some_and(|value| value.is_null())
        );
    }

    #[test]
    fn campaign_report_status_and_counters_json_capture_report_fields() {
        let mut report = sample_campaign_execution_report("ui-gallery-smoke", false, 3, 1);
        report.aggregate.share_error = Some("share failed".to_string());

        let status = campaign_report_status_json(&report);
        let counters = campaign_report_counters_json(&report);

        assert_eq!(
            status.get("ok").and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            status.get("error").and_then(|value| value.as_str()),
            Some("ui-gallery-smoke failed")
        );
        assert_eq!(
            status.get("share_error").and_then(|value| value.as_str()),
            Some("share failed")
        );
        assert_eq!(
            counters.get("items_total").and_then(|value| value.as_u64()),
            Some(3)
        );
        assert_eq!(
            counters
                .get("items_failed")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            counters
                .get("suites_total")
                .and_then(|value| value.as_u64()),
            Some(3)
        );
        assert_eq!(
            counters
                .get("scripts_total")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
    }

    #[test]
    fn campaign_report_paths_json_includes_result_path_only_for_result_artifact_mode() {
        let mut report = sample_campaign_execution_report("ui-gallery-smoke", false, 3, 1);
        report.aggregate.share_manifest_path =
            Some(PathBuf::from("runs/ui-gallery-smoke/share.manifest.json"));

        let run_paths = campaign_report_paths_json(&report, CampaignReportPathMode::RunOutcome);
        let result_paths =
            campaign_report_paths_json(&report, CampaignReportPathMode::ResultArtifact);

        assert_eq!(
            run_paths.get("out_dir").and_then(|value| value.as_str()),
            Some("runs/ui-gallery-smoke")
        );
        assert!(run_paths.get("campaign_result_path").is_none());
        assert_eq!(
            run_paths
                .get("summary_path")
                .and_then(|value| value.as_str()),
            Some("runs/ui-gallery-smoke/regression.summary.json")
        );
        assert_eq!(
            run_paths
                .get("share_manifest_path")
                .and_then(|value| value.as_str()),
            Some("runs/ui-gallery-smoke/share.manifest.json")
        );
        let expected_result_path = PathBuf::from("runs/ui-gallery-smoke")
            .join("campaign.result.json")
            .display()
            .to_string();
        assert_eq!(
            result_paths
                .get("campaign_result_path")
                .and_then(|value| value.as_str()),
            Some(expected_result_path.as_str())
        );
        assert!(
            result_paths
                .get("summary_path")
                .is_some_and(|value| value.is_null())
        );
    }

    #[test]
    fn campaign_aggregate_path_projection_hides_missing_result_paths_only() {
        let aggregate = CampaignAggregateArtifacts {
            summary_path: PathBuf::from("runs/ui-gallery-smoke/regression.summary.json"),
            index_path: PathBuf::from("runs/ui-gallery-smoke/regression.index.json"),
            share_manifest_path: Some(PathBuf::from("runs/ui-gallery-smoke/share.manifest.json")),
            summarize_error: None,
            share_error: Some("share failed".to_string()),
        };

        let run_paths =
            campaign_aggregate_path_projection(&aggregate, CampaignReportPathMode::RunOutcome);
        let result_paths =
            campaign_aggregate_path_projection(&aggregate, CampaignReportPathMode::ResultArtifact);

        assert_eq!(
            run_paths.summary_path.as_deref(),
            Some("runs/ui-gallery-smoke/regression.summary.json")
        );
        assert_eq!(
            run_paths.index_path.as_deref(),
            Some("runs/ui-gallery-smoke/regression.index.json")
        );
        assert_eq!(
            run_paths.share_manifest_path.as_deref(),
            Some("runs/ui-gallery-smoke/share.manifest.json")
        );
        assert!(result_paths.summary_path.is_none());
        assert!(result_paths.index_path.is_none());
        assert_eq!(
            result_paths.share_manifest_path.as_deref(),
            Some("runs/ui-gallery-smoke/share.manifest.json")
        );
    }

    #[test]
    fn campaign_batch_to_json_uses_result_artifact_path_projection() {
        let batch = CampaignBatchArtifacts {
            batch_root: PathBuf::from("batch/root"),
            aggregate: CampaignAggregateArtifacts {
                summary_path: PathBuf::from("batch/root/regression.summary.json"),
                index_path: PathBuf::from("batch/root/regression.index.json"),
                share_manifest_path: Some(PathBuf::from("batch/root/share.manifest.json")),
                summarize_error: Some("batch summarize failed".to_string()),
                share_error: Some("batch share failed".to_string()),
            },
        };

        let payload = campaign_batch_to_json(&batch);

        assert_eq!(
            payload.get("out_dir").and_then(|value| value.as_str()),
            Some("batch/root")
        );
        assert!(
            payload
                .get("summary_path")
                .is_some_and(|value| value.is_null())
        );
        assert!(
            payload
                .get("index_path")
                .is_some_and(|value| value.is_null())
        );
        assert_eq!(
            payload
                .get("share_manifest_path")
                .and_then(|value| value.as_str()),
            Some("batch/root/share.manifest.json")
        );
        assert_eq!(
            payload
                .get("summarize_error")
                .and_then(|value| value.as_str()),
            Some("batch summarize failed")
        );
        assert_eq!(
            payload.get("share_error").and_then(|value| value.as_str()),
            Some("batch share failed")
        );
    }

    #[test]
    fn campaign_batch_root_and_status_json_capture_batch_fields() {
        let batch = CampaignBatchArtifacts {
            batch_root: PathBuf::from("batch/root"),
            aggregate: CampaignAggregateArtifacts {
                summary_path: PathBuf::from("batch/root/regression.summary.json"),
                index_path: PathBuf::from("batch/root/regression.index.json"),
                share_manifest_path: Some(PathBuf::from("batch/root/share.manifest.json")),
                summarize_error: Some("batch summarize failed".to_string()),
                share_error: Some("batch share failed".to_string()),
            },
        };

        let root = campaign_batch_root_json(&batch);
        let status = campaign_batch_status_json(&batch);

        assert_eq!(
            root.get("out_dir").and_then(|value| value.as_str()),
            Some("batch/root")
        );
        assert_eq!(
            status
                .get("summarize_error")
                .and_then(|value| value.as_str()),
            Some("batch summarize failed")
        );
        assert_eq!(
            status.get("share_error").and_then(|value| value.as_str()),
            Some("batch share failed")
        );
    }

    #[test]
    fn campaign_batch_paths_json_uses_result_artifact_projection() {
        let batch = CampaignBatchArtifacts {
            batch_root: PathBuf::from("batch/root"),
            aggregate: CampaignAggregateArtifacts {
                summary_path: PathBuf::from("batch/root/regression.summary.json"),
                index_path: PathBuf::from("batch/root/regression.index.json"),
                share_manifest_path: Some(PathBuf::from("batch/root/share.manifest.json")),
                summarize_error: None,
                share_error: None,
            },
        };

        let paths = campaign_batch_paths_json(&batch);

        assert!(
            paths
                .get("summary_path")
                .is_some_and(|value| value.is_null())
        );
        assert!(paths.get("index_path").is_some_and(|value| value.is_null()));
        assert_eq!(
            paths
                .get("share_manifest_path")
                .and_then(|value| value.as_str()),
            Some("batch/root/share.manifest.json")
        );
    }

    #[test]
    fn campaign_run_outcome_to_json_includes_selection_counters_batch_and_runs() {
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };
        let mut report = sample_campaign_execution_report("ui-gallery-smoke", false, 3, 1);
        report.aggregate.share_manifest_path =
            Some(PathBuf::from("runs/ui-gallery-smoke/share.manifest.json"));
        let reports = vec![report];
        let batch = CampaignBatchArtifacts {
            batch_root: PathBuf::from("batch/root"),
            aggregate: CampaignAggregateArtifacts {
                summary_path: PathBuf::from("batch/root/regression.summary.json"),
                index_path: PathBuf::from("batch/root/regression.index.json"),
                share_manifest_path: Some(PathBuf::from("batch/root/share.manifest.json")),
                summarize_error: None,
                share_error: Some("batch share failed".to_string()),
            },
        };
        let outcome = CampaignRunOutcome {
            counters: build_campaign_run_counters(&reports),
            reports,
            batch: Some(batch),
            command_failures: Vec::new(),
        };

        let payload = campaign_run_outcome_to_json(&options, &outcome);

        assert_eq!(
            payload
                .get("selection")
                .and_then(|value| value.get("campaign_ids"))
                .and_then(|value| value.as_array())
                .map(|items| items.len()),
            Some(1)
        );
        assert_eq!(
            payload
                .get("counters")
                .and_then(|value| value.get("campaigns_failed"))
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            payload
                .get("batch")
                .and_then(|value| value.get("share_manifest_path"))
                .and_then(|value| value.as_str()),
            Some("batch/root/share.manifest.json")
        );
        assert_eq!(
            payload
                .get("runs")
                .and_then(|value| value.as_array())
                .map(|items| items.len()),
            Some(1)
        );
        assert_eq!(
            payload
                .get("runs")
                .and_then(|value| value.as_array())
                .and_then(|items| items.first())
                .and_then(|value| value.get("summary_path"))
                .and_then(|value| value.as_str()),
            Some("runs/ui-gallery-smoke/regression.summary.json")
        );
    }

    #[test]
    fn campaign_result_payload_uses_plan_and_summary_artifacts() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);
        let item_results = vec![
            sample_campaign_item_run_result(CampaignItemKind::Suite, "suite-a", true),
            sample_campaign_item_run_result(CampaignItemKind::Script, "script-a", false),
        ];
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 55,
            duration_ms: 13,
            summarize_error: Some("summary failed".to_string()),
            share_manifest_path: Some(PathBuf::from("share/manifest.json")),
            share_error: Some("share failed".to_string()),
        };

        let payload =
            campaign_result_payload(&campaign, &plan, &item_results, &summary_artifacts, &ctx);

        assert_eq!(
            payload.get("kind").and_then(|value| value.as_str()),
            Some(DIAG_CAMPAIGN_RESULT_KIND_V1)
        );
        assert_eq!(
            payload
                .get("run")
                .and_then(|value| value.get("run_id"))
                .and_then(|value| value.as_str()),
            Some("42")
        );
        assert_eq!(
            payload
                .get("run")
                .and_then(|value| value.get("duration_ms"))
                .and_then(|value| value.as_u64()),
            Some(13)
        );
        assert_eq!(
            payload
                .get("counters")
                .and_then(|value| value.get("items_failed"))
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            payload
                .get("aggregate")
                .and_then(|value| value.get("share_error"))
                .and_then(|value| value.as_str()),
            Some("share failed")
        );
        assert_eq!(
            payload
                .get("item_results")
                .and_then(|value| value.as_array())
                .map(|items| items.len()),
            Some(2)
        );
    }

    #[test]
    fn build_campaign_result_write_plan_uses_result_payload_and_output_path() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);
        let item_results = vec![sample_campaign_item_run_result(
            CampaignItemKind::Suite,
            "suite-a",
            true,
        )];
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 55,
            duration_ms: 13,
            summarize_error: None,
            share_manifest_path: None,
            share_error: None,
        };

        let write_plan = build_campaign_result_write_plan(
            &plan,
            &campaign,
            &item_results,
            &summary_artifacts,
            &ctx,
        );

        assert_eq!(
            write_plan.output_path,
            plan.campaign_root.join("campaign.result.json")
        );
        assert_eq!(
            write_plan
                .payload
                .get("kind")
                .and_then(|value| value.as_str()),
            Some(DIAG_CAMPAIGN_RESULT_KIND_V1)
        );
        assert_eq!(
            write_plan
                .payload
                .get("run")
                .and_then(|value| value.get("run_id"))
                .and_then(|value| value.as_str()),
            Some("42")
        );
    }

    #[test]
    fn build_campaign_result_payload_sections_use_run_counters_aggregate_and_item_results() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);
        let item_results = vec![
            sample_campaign_item_run_result(CampaignItemKind::Suite, "suite-a", true),
            sample_campaign_item_run_result(CampaignItemKind::Script, "script-a", false),
        ];
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 55,
            duration_ms: 13,
            summarize_error: Some("summary failed".to_string()),
            share_manifest_path: Some(PathBuf::from("share/manifest.json")),
            share_error: Some("share failed".to_string()),
        };

        let sections =
            build_campaign_result_payload_sections(&plan, &item_results, &summary_artifacts, &ctx);

        assert_eq!(
            sections.run.get("run_id").and_then(|value| value.as_str()),
            Some("42")
        );
        assert_eq!(
            sections
                .counters
                .get("items_failed")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            sections
                .aggregate
                .get("share_error")
                .and_then(|value| value.as_str()),
            Some("share failed")
        );
        assert_eq!(sections.item_results.len(), 2);
    }

    #[test]
    fn campaign_batch_result_payload_uses_plan_and_summary_artifacts() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];
        let plan = build_campaign_batch_plan_at(&options, reports.len(), &ctx, 77);
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 88,
            duration_ms: 21,
            summarize_error: None,
            share_manifest_path: Some(PathBuf::from("batch/share.manifest.json")),
            share_error: Some("batch share failed".to_string()),
        };

        let payload =
            campaign_batch_result_payload(&plan, &reports, &options, &summary_artifacts, &ctx);

        assert_eq!(
            payload.get("kind").and_then(|value| value.as_str()),
            Some(DIAG_CAMPAIGN_BATCH_RESULT_KIND_V1)
        );
        assert_eq!(
            payload
                .get("run")
                .and_then(|value| value.get("run_id"))
                .and_then(|value| value.as_str()),
            Some("77")
        );
        assert_eq!(
            payload
                .get("selection")
                .and_then(|value| value.get("selection_slug"))
                .and_then(|value| value.as_str()),
            Some("ids-ui-gallery-smoke")
        );
        assert_eq!(
            payload
                .get("counters")
                .and_then(|value| value.get("campaigns_failed"))
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            payload
                .get("aggregate")
                .and_then(|value| value.get("share_manifest_path"))
                .and_then(|value| value.as_str()),
            Some("batch/share.manifest.json")
        );
        assert_eq!(
            payload
                .get("runs")
                .and_then(|value| value.as_array())
                .map(|items| items.len()),
            Some(2)
        );
    }

    #[test]
    fn build_campaign_batch_result_payload_sections_use_selection_run_counters_aggregate_and_runs()
    {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];
        let plan = build_campaign_batch_plan_at(&options, reports.len(), &ctx, 77);
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 88,
            duration_ms: 21,
            summarize_error: None,
            share_manifest_path: Some(PathBuf::from("batch/share.manifest.json")),
            share_error: Some("batch share failed".to_string()),
        };

        let sections = build_campaign_batch_result_payload_sections(
            &plan,
            &reports,
            &options,
            &summary_artifacts,
            &ctx,
        );

        assert_eq!(
            sections
                .selection
                .get("selection_slug")
                .and_then(|value| value.as_str()),
            Some("ids-ui-gallery-smoke")
        );
        assert_eq!(
            sections.run.get("run_id").and_then(|value| value.as_str()),
            Some("77")
        );
        assert_eq!(
            sections
                .counters
                .get("campaigns_failed")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            sections
                .aggregate
                .get("share_manifest_path")
                .and_then(|value| value.as_str()),
            Some("batch/share.manifest.json")
        );
        assert_eq!(sections.runs.len(), 2);
    }

    #[test]
    fn build_campaign_batch_result_write_plan_uses_result_payload_and_output_path() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];
        let plan = build_campaign_batch_plan_at(&options, reports.len(), &ctx, 77);
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 88,
            duration_ms: 21,
            summarize_error: None,
            share_manifest_path: Some(PathBuf::from("batch/share.manifest.json")),
            share_error: None,
        };

        let write_plan = build_campaign_batch_result_write_plan(
            &plan,
            &reports,
            &options,
            &summary_artifacts,
            &ctx,
        );

        assert_eq!(
            write_plan.output_path,
            plan.batch_root.join("batch.result.json")
        );
        assert_eq!(
            write_plan
                .payload
                .get("kind")
                .and_then(|value| value.as_str()),
            Some(DIAG_CAMPAIGN_BATCH_RESULT_KIND_V1)
        );
        assert_eq!(
            write_plan
                .payload
                .get("run")
                .and_then(|value| value.get("run_id"))
                .and_then(|value| value.as_str()),
            Some("77")
        );
    }

    #[test]
    fn build_campaign_summary_artifacts_uses_saturating_duration_and_preserves_outcome() {
        let artifacts = build_campaign_summary_artifacts(
            100,
            88,
            CampaignSummaryFinalizeOutcome {
                summarize_error: Some("summary failed".to_string()),
                share_manifest_path: Some(PathBuf::from("share/manifest.json")),
                share_error: Some("share failed".to_string()),
            },
        );

        assert_eq!(artifacts.finished_unix_ms, 88);
        assert_eq!(artifacts.duration_ms, 0);
        assert_eq!(artifacts.summarize_error.as_deref(), Some("summary failed"));
        assert_eq!(
            artifacts.share_manifest_path.as_deref(),
            Some(Path::new("share/manifest.json"))
        );
        assert_eq!(artifacts.share_error.as_deref(), Some("share failed"));
    }

    #[test]
    fn build_campaign_execution_finalization_preserves_summary_outputs() {
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 55,
            duration_ms: 13,
            summarize_error: Some("summary failed".to_string()),
            share_manifest_path: Some(PathBuf::from("share/manifest.json")),
            share_error: Some("share failed".to_string()),
        };

        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);
        let finalization = build_campaign_execution_finalization(2, &plan, summary_artifacts);

        assert_eq!(finalization.items_failed, 2);
        assert_eq!(
            finalization.aggregate.share_manifest_path,
            Some(PathBuf::from("share/manifest.json"))
        );
        assert_eq!(
            finalization.aggregate.summarize_error.as_deref(),
            Some("summary failed")
        );
        assert_eq!(
            finalization.aggregate.share_error.as_deref(),
            Some("share failed")
        );
        assert_eq!(finalization.aggregate.summary_path, plan.summary_path);
        assert_eq!(finalization.aggregate.index_path, plan.index_path);
    }

    #[test]
    fn build_campaign_execution_finalize_plan_uses_failure_count_and_summary_setup() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);
        let item_results = vec![
            sample_campaign_item_run_result(CampaignItemKind::Suite, "suite-a", true),
            sample_campaign_item_run_result(CampaignItemKind::Script, "script-a", false),
        ];

        let finalize_plan = build_campaign_execution_finalize_plan(&item_results, &plan);

        assert_eq!(finalize_plan.items_failed, 1);
        assert!(finalize_plan.summary_finalize.summarize_inputs.is_empty());
        assert_eq!(finalize_plan.summary_finalize.out_dir, plan.campaign_root);
        assert_eq!(
            finalize_plan.summary_finalize.summary_path,
            plan.summary_path
        );
        assert_eq!(finalize_plan.summary_finalize.created_unix_ms, 42);
        assert!(
            finalize_plan
                .summary_finalize
                .should_generate_share_manifest
        );
    }

    #[test]
    fn build_campaign_execution_summary_finalize_plan_uses_plan_root_and_failure_state() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);
        let item_results = vec![
            sample_campaign_item_run_result(CampaignItemKind::Suite, "suite-a", true),
            sample_campaign_item_run_result(CampaignItemKind::Script, "script-a", false),
        ];

        let finalize_plan = build_campaign_execution_summary_finalize_plan(&item_results, &plan);

        assert!(finalize_plan.summarize_inputs.is_empty());
        assert_eq!(finalize_plan.out_dir, plan.campaign_root);
        assert_eq!(finalize_plan.summary_path, plan.summary_path);
        assert_eq!(finalize_plan.created_unix_ms, 42);
        assert!(finalize_plan.should_generate_share_manifest);
    }

    #[test]
    fn build_campaign_batch_artifacts_preserves_plan_paths_and_summary_outputs() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };
        let plan = build_campaign_batch_plan_at(&options, 1, &ctx, 77);
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 88,
            duration_ms: 21,
            summarize_error: Some("batch summarize failed".to_string()),
            share_manifest_path: Some(PathBuf::from("batch/share.manifest.json")),
            share_error: Some("batch share failed".to_string()),
        };

        let batch = build_campaign_batch_artifacts(&plan, summary_artifacts);

        assert_eq!(batch.batch_root, plan.batch_root);
        assert_eq!(batch.aggregate.summary_path, plan.summary_path);
        assert_eq!(batch.aggregate.index_path, plan.index_path);
        assert_eq!(
            batch.aggregate.share_manifest_path,
            Some(PathBuf::from("batch/share.manifest.json"))
        );
        assert_eq!(
            batch.aggregate.summarize_error.as_deref(),
            Some("batch summarize failed")
        );
        assert_eq!(
            batch.aggregate.share_error.as_deref(),
            Some("batch share failed")
        );
    }

    #[test]
    fn build_campaign_batch_manifest_write_plan_uses_batch_manifest_payload_and_output_path() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };
        let plan = build_campaign_batch_plan_at(&options, 2, &ctx, 77);
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];

        let write_plan = build_campaign_batch_manifest_write_plan(&plan, &reports, &options, &ctx);

        assert_eq!(
            write_plan.output_path,
            plan.batch_root.join("batch.manifest.json")
        );
        assert_eq!(
            write_plan.payload,
            campaign_batch_manifest_payload(
                &plan.batch_root,
                &plan.run_id,
                plan.created_unix_ms,
                &reports,
                &options,
                &ctx,
            )
        );
    }

    #[test]
    fn build_campaign_batch_artifact_write_plan_reuses_manifest_and_finalize_seams() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];

        let write_plan = build_campaign_batch_artifact_write_plan(&reports, &options, &ctx);

        assert_eq!(
            write_plan.manifest_write.output_path,
            write_plan.batch.batch_root.join("batch.manifest.json")
        );
        assert_eq!(
            write_plan.summary_finalize.out_dir,
            write_plan.batch.batch_root
        );
        assert_eq!(
            write_plan.summary_finalize.summary_path,
            write_plan.batch.summary_path
        );
        assert_eq!(
            write_plan.summary_finalize.created_unix_ms,
            write_plan.batch.created_unix_ms
        );
        assert!(write_plan.summary_finalize.should_generate_share_manifest);
        assert_eq!(
            write_plan.manifest_write.payload,
            campaign_batch_manifest_payload(
                &write_plan.batch.batch_root,
                &write_plan.batch.run_id,
                write_plan.batch.created_unix_ms,
                &reports,
                &options,
                &ctx,
            )
        );
    }

    #[test]
    fn build_campaign_batch_summary_finalize_plan_uses_report_dirs_and_failure_state() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };
        let plan = build_campaign_batch_plan_at(&options, 2, &ctx, 77);
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];

        let finalize_plan = build_campaign_batch_summary_finalize_plan(&reports, &plan);

        assert_eq!(
            finalize_plan.summarize_inputs,
            vec![
                "runs/ui-gallery-smoke".to_string(),
                "runs/docking-smoke".to_string()
            ]
        );
        assert_eq!(finalize_plan.out_dir, plan.batch_root);
        assert_eq!(finalize_plan.summary_path, plan.summary_path);
        assert_eq!(finalize_plan.created_unix_ms, 77);
        assert!(finalize_plan.should_generate_share_manifest);
    }

    #[test]
    fn build_campaign_aggregate_artifacts_preserves_paths_and_errors() {
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 88,
            duration_ms: 21,
            summarize_error: Some("summary failed".to_string()),
            share_manifest_path: Some(PathBuf::from("batch/share.manifest.json")),
            share_error: Some("share failed".to_string()),
        };

        let aggregate = build_campaign_aggregate_artifacts(
            Path::new("batch/root/regression.summary.json"),
            Path::new("batch/root/regression.index.json"),
            &summary_artifacts,
        );

        assert_eq!(
            aggregate.summary_path,
            PathBuf::from("batch/root/regression.summary.json")
        );
        assert_eq!(
            aggregate.index_path,
            PathBuf::from("batch/root/regression.index.json")
        );
        assert_eq!(
            aggregate.share_manifest_path,
            Some(PathBuf::from("batch/share.manifest.json"))
        );
        assert_eq!(aggregate.summarize_error.as_deref(), Some("summary failed"));
        assert_eq!(aggregate.share_error.as_deref(), Some("share failed"));
    }

    #[test]
    fn build_campaign_report_aggregate_artifacts_uses_plan_paths_and_share_outputs() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);

        let aggregate = build_campaign_report_aggregate_artifacts(
            &plan,
            Some(PathBuf::from("share/manifest.json")),
            Some("share failed".to_string()),
        );

        assert_eq!(aggregate.summary_path, plan.summary_path);
        assert_eq!(aggregate.index_path, plan.index_path);
        assert_eq!(
            aggregate.share_manifest_path,
            Some(PathBuf::from("share/manifest.json"))
        );
        assert!(aggregate.summarize_error.is_none());
        assert_eq!(aggregate.share_error.as_deref(), Some("share failed"));
    }

    #[test]
    fn campaign_failed_reports_summary_and_share_helpers_capture_expected_text() {
        let mut report = sample_campaign_execution_report("ui-gallery-smoke", false, 3, 1);
        report.aggregate.share_error = Some("report share failed".to_string());
        let batch = CampaignBatchArtifacts {
            batch_root: PathBuf::from("batch/root"),
            aggregate: CampaignAggregateArtifacts {
                summary_path: PathBuf::from("batch/root/regression.summary.json"),
                index_path: PathBuf::from("batch/root/regression.index.json"),
                share_manifest_path: None,
                summarize_error: Some("batch summarize failed".to_string()),
                share_error: Some("batch share failed".to_string()),
            },
        };

        let summary = campaign_failed_reports_summary(&[report.clone()], 1).unwrap();
        let batch_summarize = campaign_batch_summarize_failure_text(&batch).unwrap();
        let batch_share = campaign_batch_share_failure_text(&batch).unwrap();
        let report_share = campaign_report_share_failure_text(&report).unwrap();

        assert!(summary.contains("campaign run completed with 1 failed campaign(s)"));
        assert!(summary.contains("ui-gallery-smoke: ui-gallery-smoke failed"));
        assert!(batch_summarize.contains("campaign batch summarize failed under batch/root"));
        assert!(batch_share.contains("campaign batch share export failed under batch/root"));
        assert!(report_share.contains("campaign `ui-gallery-smoke` share export failed"));
    }

    #[test]
    fn build_campaign_item_run_counters_tracks_suite_and_script_failures() {
        let item_results = vec![
            sample_campaign_item_run_result(CampaignItemKind::Suite, "suite-a", true),
            sample_campaign_item_run_result(CampaignItemKind::Suite, "suite-b", false),
            sample_campaign_item_run_result(CampaignItemKind::Script, "script-a", false),
        ];

        let counters = build_campaign_item_run_counters(&item_results);

        assert_eq!(
            counters,
            CampaignItemRunCounters {
                items_total: 3,
                items_passed: 1,
                items_failed: 2,
                suites_total: 2,
                suites_failed: 1,
                scripts_total: 1,
                scripts_failed: 1,
            }
        );
    }

    #[test]
    fn campaign_batch_manifest_resolved_json_reuses_aggregate_totals() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];

        let resolved = campaign_batch_manifest_resolved_json(&reports, &ctx);

        assert_eq!(
            resolved.get("campaigns_total").and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            resolved.get("items_total").and_then(|v| v.as_u64()),
            Some(8)
        );
        assert_eq!(
            resolved.get("suites_total").and_then(|v| v.as_u64()),
            Some(8)
        );
        assert_eq!(
            resolved.get("scripts_total").and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            resolved
                .get("runs")
                .and_then(|v| v.as_array())
                .map(|items| items.len()),
            Some(2)
        );
    }

    #[test]
    fn campaign_manifest_payload_uses_run_and_resolved_sections() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();

        let payload = campaign_manifest_payload(&root, &campaign, "42", 7, &ctx);

        assert_eq!(
            payload.get("kind").and_then(|value| value.as_str()),
            Some(DIAG_CAMPAIGN_MANIFEST_KIND_V1)
        );
        assert_eq!(
            payload
                .get("run")
                .and_then(|value| value.get("run_id"))
                .and_then(|value| value.as_str()),
            Some("42")
        );
        assert_eq!(
            payload
                .get("run")
                .and_then(|value| value.get("created_unix_ms"))
                .and_then(|value| value.as_u64()),
            Some(7)
        );
        assert_eq!(
            payload
                .get("resolved")
                .and_then(|value| value.get("item_count"))
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            payload
                .get("resolved")
                .and_then(|value| value.get("launch_env"))
                .and_then(|value| value.as_array())
                .map(|items| items.len()),
            Some(1)
        );
        assert_eq!(
            payload
                .get("resolved")
                .and_then(|value| value.get("items"))
                .and_then(|value| value.as_array())
                .map(|items| items.len()),
            Some(1)
        );
    }

    #[test]
    fn campaign_batch_manifest_payload_includes_selection_run_and_resolved_sections() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];

        let payload = campaign_batch_manifest_payload(&root, "77", 11, &reports, &options, &ctx);

        assert_eq!(
            payload.get("kind").and_then(|value| value.as_str()),
            Some(DIAG_CAMPAIGN_BATCH_MANIFEST_KIND_V1)
        );
        assert_eq!(
            payload
                .get("selection")
                .and_then(|value| value.get("selection_slug"))
                .and_then(|value| value.as_str()),
            Some("ids-ui-gallery-smoke")
        );
        assert_eq!(
            payload
                .get("selection")
                .and_then(|value| value.get("selected_campaign_ids"))
                .and_then(|value| value.as_array())
                .map(|items| items.len()),
            Some(2)
        );
        assert_eq!(
            payload
                .get("run")
                .and_then(|value| value.get("run_id"))
                .and_then(|value| value.as_str()),
            Some("77")
        );
        assert_eq!(
            payload
                .get("resolved")
                .and_then(|value| value.get("campaigns_total"))
                .and_then(|value| value.as_u64()),
            Some(2)
        );
        assert_eq!(
            payload
                .get("resolved")
                .and_then(|value| value.get("runs"))
                .and_then(|value| value.as_array())
                .map(|items| items.len()),
            Some(2)
        );
    }

    #[test]
    fn campaign_run_record_json_omits_finished_fields_when_absent() {
        let run = campaign_run_record_json(
            "42",
            7,
            None,
            None,
            Path::new("runs/demo"),
            Path::new("workspace/root"),
        );

        assert_eq!(run.get("run_id").and_then(|v| v.as_str()), Some("42"));
        assert_eq!(run.get("created_unix_ms").and_then(|v| v.as_u64()), Some(7));
        assert!(run.get("finished_unix_ms").is_none());
        assert!(run.get("duration_ms").is_none());
        let expected_out_dir = PathBuf::from("runs/demo").display().to_string();
        assert_eq!(
            run.get("out_dir").and_then(|v| v.as_str()),
            Some(expected_out_dir.as_str())
        );
    }

    #[test]
    fn campaign_result_run_json_uses_summary_artifact_timing_and_paths() {
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 55,
            duration_ms: 13,
            summarize_error: None,
            share_manifest_path: None,
            share_error: None,
        };

        let run = campaign_result_run_json(
            "42",
            7,
            Path::new("runs/demo"),
            &summary_artifacts,
            Path::new("workspace/root"),
        );

        assert_eq!(run.get("run_id").and_then(|v| v.as_str()), Some("42"));
        assert_eq!(
            run.get("finished_unix_ms").and_then(|v| v.as_u64()),
            Some(55)
        );
        assert_eq!(run.get("duration_ms").and_then(|v| v.as_u64()), Some(13));
        let expected_out_dir = PathBuf::from("runs/demo").display().to_string();
        assert_eq!(
            run.get("out_dir").and_then(|v| v.as_str()),
            Some(expected_out_dir.as_str())
        );
    }

    #[test]
    fn campaign_result_aggregate_json_uses_summary_artifact_outputs() {
        let summary_artifacts = CampaignSummaryArtifacts {
            finished_unix_ms: 55,
            duration_ms: 13,
            summarize_error: Some("summary failed".to_string()),
            share_manifest_path: Some(PathBuf::from("batch/share.manifest.json")),
            share_error: Some("share failed".to_string()),
        };

        let aggregate = campaign_result_aggregate_json(
            Path::new("batch/root/regression.summary.json"),
            Path::new("batch/root/regression.index.json"),
            &summary_artifacts,
        );

        assert!(
            aggregate
                .get("summary_path")
                .is_some_and(|value| value.is_null())
        );
        assert!(
            aggregate
                .get("index_path")
                .is_some_and(|value| value.is_null())
        );
        assert_eq!(
            aggregate
                .get("share_manifest_path")
                .and_then(|value| value.as_str()),
            Some("batch/share.manifest.json")
        );
        assert_eq!(
            aggregate
                .get("summarize_error")
                .and_then(|value| value.as_str()),
            Some("summary failed")
        );
        assert_eq!(
            aggregate
                .get("share_error")
                .and_then(|value| value.as_str()),
            Some("share failed")
        );
    }

    #[test]
    fn campaign_selection_json_includes_selected_campaign_ids_only_when_present() {
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };

        let without_selected = campaign_selection_json(&options, 1, None);
        let with_selected =
            campaign_selection_json(&options, 1, Some(vec!["ui-gallery-smoke", "docking-smoke"]));

        assert!(without_selected.get("selected_campaign_ids").is_none());
        assert_eq!(
            with_selected
                .get("selected_campaign_ids")
                .and_then(|v| v.as_array())
                .map(|items| items.len()),
            Some(2)
        );
        assert_eq!(
            with_selected.get("selection_slug").and_then(|v| v.as_str()),
            Some("ids-ui-gallery-smoke")
        );
    }

    #[test]
    fn build_campaign_run_outcome_collects_counters_and_failures() {
        let mut failed_report = sample_campaign_execution_report("ui-gallery-smoke", false, 3, 1);
        failed_report.error = Some("suite failed".to_string());
        failed_report.aggregate.share_error = Some("share failed".to_string());
        let reports = vec![failed_report];

        let outcome = build_campaign_run_outcome(reports, None);

        assert_eq!(outcome.counters.campaigns_total, 1);
        assert_eq!(outcome.counters.campaigns_failed, 1);
        assert_eq!(outcome.command_failures.len(), 2);
        assert!(outcome.command_failures[0].contains("failed campaign"));
        assert!(outcome.command_failures[1].contains("share export failed"));
    }

    #[test]
    fn build_campaign_run_counters_accumulates_report_totals() {
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];

        let counters = build_campaign_run_counters(&reports);

        assert_eq!(
            counters,
            CampaignRunCounters {
                campaigns_total: 2,
                campaigns_failed: 1,
                campaigns_passed: 1,
                items_total: 8,
                items_failed: 2,
                suites_total: 8,
                scripts_total: 2,
            }
        );
    }

    #[test]
    fn collect_campaign_run_failures_tracks_run_batch_and_share_errors() {
        let mut failed_report = sample_campaign_execution_report("ui-gallery-smoke", false, 3, 1);
        failed_report.aggregate.share_error = Some("report share failed".to_string());
        let reports = vec![failed_report];
        let batch = CampaignBatchArtifacts {
            batch_root: PathBuf::from("batch/root"),
            aggregate: CampaignAggregateArtifacts {
                summary_path: PathBuf::from("batch/root/regression.summary.json"),
                index_path: PathBuf::from("batch/root/regression.index.json"),
                share_manifest_path: None,
                summarize_error: Some("batch summarize failed".to_string()),
                share_error: Some("batch share failed".to_string()),
            },
        };

        let failures = collect_campaign_run_failures(&reports, Some(&batch), 1);

        assert_eq!(failures.len(), 4);
        assert!(failures.iter().any(|failure| {
            failure.contains("campaign run completed with 1 failed campaign(s)")
                && failure.contains("ui-gallery-smoke: ui-gallery-smoke failed")
        }));
        assert!(failures.iter().any(|failure| {
            failure.contains("campaign batch summarize failed under batch/root")
                && failure.contains("batch summarize failed")
        }));
        assert!(failures.iter().any(|failure| {
            failure.contains("campaign batch share export failed under batch/root")
                && failure.contains("batch share failed")
        }));
        assert!(failures.iter().any(|failure| {
            failure.contains("campaign `ui-gallery-smoke` share export failed")
                && failure.contains("report share failed")
        }));
    }

    #[test]
    fn campaign_report_out_dirs_preserves_report_order() {
        let reports = vec![
            sample_campaign_execution_report("ui-gallery-smoke", true, 3, 0),
            sample_campaign_execution_report("docking-smoke", false, 5, 2),
        ];

        let out_dirs = campaign_report_out_dirs(&reports);

        assert_eq!(
            out_dirs,
            vec![
                PathBuf::from("runs/ui-gallery-smoke").display().to_string(),
                PathBuf::from("runs/docking-smoke").display().to_string(),
            ]
        );
    }

    #[test]
    fn build_campaign_execution_outcome_prefers_summarize_failure_over_item_failures() {
        let root = PathBuf::from("diag-out-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);
        let item_results = vec![sample_campaign_item_run_result(
            CampaignItemKind::Suite,
            "ui-gallery-lite-smoke",
            false,
        )];
        let finalization = CampaignExecutionFinalization {
            items_failed: 1,
            aggregate: CampaignAggregateArtifacts {
                summary_path: plan.summary_path.clone(),
                index_path: plan.index_path.clone(),
                summarize_error: Some("summary boom".to_string()),
                share_manifest_path: Some(PathBuf::from("share/manifest.json")),
                share_error: Some("share boom".to_string()),
            },
        };

        let outcome =
            build_campaign_execution_outcome(&campaign, &plan, &item_results, finalization);

        assert_eq!(outcome.items_failed, 1);
        assert_eq!(
            outcome.error.as_deref(),
            Some(
                "campaign `ui-gallery-smoke` finished item execution but summarize failed: summary boom"
            )
        );
        assert_eq!(
            outcome.share_manifest_path,
            Some(PathBuf::from("share/manifest.json"))
        );
        assert_eq!(outcome.share_error.as_deref(), Some("share boom"));
    }

    #[test]
    fn campaign_item_failure_summary_uses_unknown_error_fallback() {
        let mut item = sample_campaign_item_run_result(
            CampaignItemKind::Script,
            "tools/diag-scripts/demo.json",
            false,
        );
        item.error = None;

        let summary = campaign_item_failure_summary(&item);

        assert_eq!(
            summary,
            "script tools/diag-scripts/demo.json: unknown error"
        );
    }

    #[test]
    fn campaign_item_failures_error_lists_failing_items() {
        let root = PathBuf::from("diag-out-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);
        let item_results = vec![
            sample_campaign_item_run_result(
                CampaignItemKind::Suite,
                "ui-gallery-lite-smoke",
                false,
            ),
            sample_campaign_item_run_result(
                CampaignItemKind::Script,
                "tools/diag-scripts/demo.json",
                false,
            ),
        ];

        let error = campaign_item_failures_error(&campaign, &plan, &item_results, 2);

        assert!(error.contains("campaign `ui-gallery-smoke` completed with 2 failed item(s)"));
        assert!(error.contains("suite ui-gallery-lite-smoke: ui-gallery-lite-smoke failed"));
        assert!(
            error.contains(
                "script tools/diag-scripts/demo.json: tools/diag-scripts/demo.json failed"
            )
        );
        assert!(error.contains(&plan.campaign_root.display().to_string()));
    }

    #[test]
    fn build_campaign_manifest_write_plan_uses_manifest_payload_and_output_path() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();
        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);

        let write_plan = build_campaign_manifest_write_plan(&plan, &campaign, &ctx);

        assert_eq!(
            write_plan.output_path,
            plan.campaign_root.join("campaign.manifest.json")
        );
        assert_eq!(
            write_plan.payload,
            campaign_manifest_payload(
                &plan.campaign_root,
                &campaign,
                &plan.run_id,
                plan.created_unix_ms,
                &ctx,
            )
        );
    }

    #[test]
    fn build_campaign_execution_start_plan_reuses_execution_and_manifest_setup() {
        let root = PathBuf::from("diag-root");
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();

        let start_plan = build_campaign_execution_start_plan_at(&campaign, &ctx, 42);

        assert_eq!(start_plan.execution.run_id, "42");
        assert_eq!(
            start_plan.execution.campaign_root,
            root.join("diag-out")
                .join("campaigns")
                .join("ui-gallery-smoke")
                .join("42")
        );
        assert_eq!(
            start_plan.manifest_write.output_path,
            start_plan
                .execution
                .campaign_root
                .join("campaign.manifest.json")
        );
        assert_eq!(
            start_plan.manifest_write.payload,
            campaign_manifest_payload(
                &start_plan.execution.campaign_root,
                &campaign,
                &start_plan.execution.run_id,
                start_plan.execution.created_unix_ms,
                &ctx,
            )
        );
    }

    #[test]
    fn build_campaign_execution_plan_uses_campaign_root_and_run_id() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-campaign-plan-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let ctx = sample_campaign_run_context(&root);
        let campaign = sample_campaign_definition();

        let plan = build_campaign_execution_plan_at(&campaign, &ctx, 42);

        assert_eq!(plan.run_id, "42");
        assert_eq!(
            plan.campaign_root,
            root.join("diag-out")
                .join("campaigns")
                .join("ui-gallery-smoke")
                .join("42")
        );
        assert_eq!(
            plan.suite_results_root,
            plan.campaign_root.join("suite-results")
        );
        assert_eq!(
            plan.script_results_root,
            plan.campaign_root.join("script-results")
        );
        assert_eq!(
            plan.summary_path,
            plan.campaign_root
                .join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1)
        );
    }

    #[test]
    fn build_campaign_batch_plan_uses_selection_slug_and_run_id() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-campaign-batch-plan-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let ctx = sample_campaign_run_context(&root);
        let options = CampaignRunOptions {
            campaign_ids: vec!["ui-gallery-smoke".to_string()],
            filter: CampaignFilterOptions::default(),
        };

        let plan = build_campaign_batch_plan_at(&options, 1, &ctx, 77);

        assert_eq!(plan.run_id, "77");
        assert_eq!(
            plan.batch_root,
            root.join("diag-out")
                .join("campaign-batches")
                .join("ids-ui-gallery-smoke")
                .join("77")
        );
        assert_eq!(
            plan.index_path,
            plan.batch_root
                .join(crate::regression_summary::DIAG_REGRESSION_INDEX_FILENAME_V1)
        );
    }

    #[test]
    fn campaign_share_writes_manifest_and_ai_zip_for_failed_items() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-campaign-share-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let bundle_dir = root.join("bundle-a");
        let packet_dir = bundle_dir.join("ai.packet");
        std::fs::create_dir_all(&packet_dir).unwrap();
        std::fs::write(packet_dir.join("summary.json"), b"{}" as &[u8]).unwrap();
        let screenshots_dir = root.join("screenshots").join("bundle-a");
        std::fs::create_dir_all(&screenshots_dir).unwrap();
        std::fs::write(screenshots_dir.join("manifest.json"), b"{}" as &[u8]).unwrap();

        let mut totals = RegressionTotalsV1::default();
        totals.record_status(RegressionStatusV1::FailedDeterministic);
        let mut summary = RegressionSummaryV1::new(
            RegressionCampaignSummaryV1 {
                name: "ui-gallery-smoke".to_string(),
                lane: RegressionLaneV1::Smoke,
                profile: None,
                schema_version: None,
                requested_by: None,
                filters: None,
            },
            RegressionRunSummaryV1 {
                run_id: "1234".to_string(),
                tool: "fretboard diag campaign".to_string(),
                created_unix_ms: crate::util::now_unix_ms(),
                duration_ms: None,
                workspace_root: Some(root.display().to_string()),
                out_dir: Some(root.display().to_string()),
                started_unix_ms: None,
                finished_unix_ms: None,
                tool_version: None,
                git_commit: None,
                git_branch: None,
                host: None,
            },
            totals,
        );
        summary.items.push(RegressionItemSummaryV1 {
            item_id: "accordion-basic".to_string(),
            kind: RegressionItemKindV1::Script,
            name: "accordion-basic".to_string(),
            status: RegressionStatusV1::FailedDeterministic,
            reason_code: Some("diag.test.failure".to_string()),
            source_reason_code: None,
            lane: RegressionLaneV1::Smoke,
            owner: None,
            feature_tags: Vec::new(),
            timing: None,
            attempts: None,
            evidence: Some(RegressionEvidenceV1 {
                bundle_artifact: None,
                bundle_dir: Some(bundle_dir.display().to_string()),
                triage_json: None,
                script_result_json: None,
                ai_packet_dir: None,
                pack_path: None,
                screenshots_manifest: None,
                perf_summary_json: None,
                compare_json: None,
                extra: None,
            }),
            source: None,
            notes: None,
        });
        summary.highlights = RegressionHighlightsV1::from_items(&summary.items);
        summary.artifacts = Some(RegressionArtifactsV1 {
            summary_dir: Some(root.display().to_string()),
            packed_report: None,
            index_json: None,
            html_report: None,
        });

        let summary_path =
            root.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);
        write_json_value(
            &summary_path,
            &serde_json::to_value(&summary).expect("summary value"),
        )
        .unwrap();

        let manifest_path =
            write_campaign_share_manifest(&root, &summary_path, &root, false, 5, 0).unwrap();
        let manifest = read_json_value(&manifest_path).expect("share manifest");

        assert_eq!(
            manifest
                .pointer("/counters/bundles_packed")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        let share_zip = manifest
            .pointer("/items/0/share_zip")
            .and_then(|v| v.as_str())
            .expect("share zip path");
        assert!(PathBuf::from(share_zip).is_file());
        let combined_zip = manifest
            .pointer("/share/combined_zip")
            .and_then(|v| v.as_str())
            .expect("combined zip path");
        assert!(PathBuf::from(combined_zip).is_file());
        let screenshots_manifest = manifest
            .pointer("/items/0/screenshots_manifest")
            .and_then(|v| v.as_str())
            .expect("screenshots manifest path");
        assert!(PathBuf::from(screenshots_manifest).is_file());
    }
}
