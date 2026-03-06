use super::*;

use crate::registry::campaigns::{
    CampaignDefinition, CampaignFilterOptions, CampaignItemDefinition, CampaignItemKind,
    CampaignRegistry, campaign_to_json, item_kind_str, lane_to_str, parse_lane, source_kind_str,
};
use crate::regression_summary::{RegressionStatusV1, RegressionSummaryV1};

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
    summary_path: PathBuf,
    index_path: PathBuf,
    share_manifest_path: Option<PathBuf>,
    items_total: usize,
    items_failed: usize,
    suites_total: usize,
    scripts_total: usize,
    ok: bool,
    error: Option<String>,
    share_error: Option<String>,
}

#[derive(Debug, Clone)]
struct CampaignExecutionOutcome {
    items_failed: usize,
    error: Option<String>,
    share_manifest_path: Option<PathBuf>,
    share_error: Option<String>,
}

#[derive(Debug, Clone)]
struct CampaignBatchArtifacts {
    batch_root: PathBuf,
    summary_path: PathBuf,
    index_path: PathBuf,
    share_manifest_path: Option<PathBuf>,
    summarize_error: Option<String>,
    share_error: Option<String>,
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

pub(crate) fn cmd_campaign(ctx: CampaignCmdContext) -> Result<(), String> {
    let CampaignCmdContext {
        pack_after_run,
        rest,
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

    let registry = CampaignRegistry::load_from_workspace_root(&workspace_root)?;

    let Some(sub) = rest.first().map(|value| value.as_str()) else {
        return Err(
            "missing campaign subcommand (try: fretboard diag campaign list | show <id> | run <id>)"
                .to_string(),
        );
    };

    match sub {
        "list" => cmd_campaign_list(&registry, &rest[1..], stats_json),
        "show" => cmd_campaign_show(&registry, &rest[1..], stats_json),
        "share" => cmd_campaign_share(
            &rest[1..],
            &workspace_root,
            stats_json,
            stats_top,
            warmup_frames,
        ),
        "run" => cmd_campaign_run(
            &registry,
            &rest[1..],
            CampaignRunContext {
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
            },
        ),
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
    let selected = select_campaigns_for_run(registry, &options)?;

    let mut reports = Vec::new();
    for campaign in selected {
        reports.push(execute_campaign(campaign, &ctx));
    }

    let batch = if reports.len() > 1 {
        Some(write_campaign_batch_artifacts(&reports, &options, &ctx)?)
    } else {
        None
    };

    let failed_runs = reports.iter().filter(|report| !report.ok).count();
    if ctx.stats_json {
        let payload = serde_json::json!({
            "selection": {
                "campaign_ids": &options.campaign_ids,
                "filters": campaign_filter_to_json(&options.filter),
            },
            "counters": {
                "campaigns_total": reports.len(),
                "campaigns_failed": failed_runs,
                "campaigns_passed": reports.len().saturating_sub(failed_runs),
                "items_total": reports.iter().map(|report| report.items_total).sum::<usize>(),
                "items_failed": reports.iter().map(|report| report.items_failed).sum::<usize>(),
                "suites_total": reports.iter().map(|report| report.suites_total).sum::<usize>(),
                "scripts_total": reports.iter().map(|report| report.scripts_total).sum::<usize>(),
            },
            "batch": batch.as_ref().map(campaign_batch_to_json),
            "runs": reports.iter().map(|report| serde_json::json!({
                "campaign_id": report.campaign_id,
                "ok": report.ok,
                "error": report.error,
                "share_error": report.share_error,
                "out_dir": report.out_dir.display().to_string(),
                "summary_path": report.summary_path.display().to_string(),
                "index_path": report.index_path.display().to_string(),
                "share_manifest_path": report.share_manifest_path.as_ref().map(|path| path.display().to_string()),
                "items_total": report.items_total,
                "items_failed": report.items_failed,
                "suites_total": report.suites_total,
                "scripts_total": report.scripts_total,
            })).collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    } else if reports.len() == 1 {
        let report = &reports[0];
        if report.ok {
            println!(
                "campaign: ok (id={}, items={}, suites={}, scripts={}, out_dir={})",
                report.campaign_id,
                report.items_total,
                report.suites_total,
                report.scripts_total,
                report.out_dir.display()
            );
        } else if let Some(share_manifest_path) = report.share_manifest_path.as_deref() {
            println!(
                "campaign: failed evidence exported (id={}, share_manifest={})",
                report.campaign_id,
                share_manifest_path.display()
            );
        }
    } else {
        println!(
            "campaign batch: {} run(s), {} failed",
            reports.len(),
            failed_runs
        );
        if let Some(batch) = &batch {
            println!("  batch_root: {}", batch.batch_root.display());
            if let Some(share_manifest_path) = batch.share_manifest_path.as_deref() {
                println!("  share_manifest: {}", share_manifest_path.display());
            }
        }
        for report in &reports {
            let status = if report.ok { "ok" } else { "failed" };
            println!(
                "  - {} [{}] items={} failed={} -> {}",
                report.campaign_id,
                status,
                report.items_total,
                report.items_failed,
                report.out_dir.display()
            );
            if let Some(share_manifest_path) = report.share_manifest_path.as_deref() {
                println!("    share_manifest: {}", share_manifest_path.display());
            }
        }
    }

    let mut command_failures = Vec::new();
    if failed_runs > 0 {
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
        command_failures.push(format!(
            "campaign run completed with {} failed campaign(s): {}",
            failed_runs, failures
        ));
    }
    if let Some(batch) = &batch
        && let Some(error) = batch.summarize_error.as_deref()
    {
        command_failures.push(format!(
            "campaign batch summarize failed under {}: {}",
            batch.batch_root.display(),
            error
        ));
    }
    if let Some(batch) = &batch
        && let Some(error) = batch.share_error.as_deref()
    {
        command_failures.push(format!(
            "campaign batch share export failed under {}: {}",
            batch.batch_root.display(),
            error
        ));
    }
    let report_share_failures = reports
        .iter()
        .filter_map(|report| {
            report.share_error.as_deref().map(|error| {
                format!(
                    "campaign `{}` share export failed under {}: {}",
                    report.campaign_id,
                    report.out_dir.display(),
                    error
                )
            })
        })
        .collect::<Vec<_>>();
    command_failures.extend(report_share_failures);
    if !command_failures.is_empty() {
        return Err(command_failures.join("; "));
    }

    Ok(())
}

fn select_campaigns_for_run<'a>(
    registry: &'a CampaignRegistry,
    options: &CampaignRunOptions,
) -> Result<Vec<&'a CampaignDefinition>, String> {
    if !options.campaign_ids.is_empty() {
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
        return Ok(selected);
    }

    if campaign_filter_is_empty(&options.filter) {
        return Err(
            "missing campaign id or run selector (try: `diag campaign run ui-gallery-smoke` or `diag campaign run --lane smoke --tag ui-gallery`)"
                .to_string(),
        );
    }

    let selected = registry.filtered_campaigns(&options.filter);
    if selected.is_empty() {
        return Err("no campaigns matched the requested run selectors".to_string());
    }
    Ok(selected)
}

fn execute_campaign(
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
) -> CampaignExecutionReport {
    let created_unix_ms = now_unix_ms();
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

    let (items_failed, report_error, share_manifest_path, share_error) =
        match execute_campaign_inner(campaign, ctx, &campaign_root, created_unix_ms, &run_id) {
            Ok(outcome) => (
                outcome.items_failed,
                outcome.error,
                outcome.share_manifest_path,
                outcome.share_error,
            ),
            Err(error) => (campaign.items.len(), Some(error), None, None),
        };

    CampaignExecutionReport {
        campaign_id: campaign.id.clone(),
        out_dir: campaign_root,
        summary_path,
        index_path,
        share_manifest_path,
        items_total: campaign.items.len(),
        items_failed,
        suites_total: campaign.suite_count(),
        scripts_total: campaign.script_count(),
        ok: report_error.is_none(),
        error: report_error,
        share_error,
    }
}

fn execute_campaign_inner(
    campaign: &CampaignDefinition,
    ctx: &CampaignRunContext,
    campaign_root: &Path,
    created_unix_ms: u64,
    run_id: &str,
) -> Result<CampaignExecutionOutcome, String> {
    let suite_results_root = campaign_root.join("suite-results");
    let script_results_root = campaign_root.join("script-results");
    std::fs::create_dir_all(&suite_results_root).map_err(|e| {
        format!(
            "failed to create suite results dir {}: {}",
            suite_results_root.display(),
            e
        )
    })?;
    std::fs::create_dir_all(&script_results_root).map_err(|e| {
        format!(
            "failed to create script results dir {}: {}",
            script_results_root.display(),
            e
        )
    })?;

    write_campaign_manifest(campaign_root, campaign, run_id, created_unix_ms, ctx)?;

    let mut item_results: Vec<CampaignItemRunResult> = Vec::new();
    for (index, item) in campaign.items.iter().enumerate() {
        item_results.push(run_campaign_item(
            index,
            item,
            &suite_results_root,
            &script_results_root,
            ctx,
        )?);
    }

    let summarize_result = diag_summarize::cmd_summarize(diag_summarize::SummarizeCmdContext {
        rest: Vec::new(),
        workspace_root: ctx.workspace_root.clone(),
        resolved_out_dir: campaign_root.to_path_buf(),
        stats_json: false,
    });

    let finished_unix_ms = now_unix_ms();
    let duration_ms = finished_unix_ms.saturating_sub(created_unix_ms);
    let summary_path =
        campaign_root.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    let index_path =
        campaign_root.join(crate::regression_summary::DIAG_REGRESSION_INDEX_FILENAME_V1);
    let items_failed = item_results.iter().filter(|entry| !entry.ok).count();
    let (share_manifest_path, share_error) = maybe_write_failure_share_manifest(
        campaign_root,
        &summary_path,
        &ctx.workspace_root,
        items_failed > 0,
        ctx.stats_top,
        ctx.warmup_frames,
    );
    write_campaign_result(
        campaign_root,
        campaign,
        run_id,
        created_unix_ms,
        finished_unix_ms,
        duration_ms,
        &item_results,
        summarize_result.as_ref().err(),
        &summary_path,
        &index_path,
        share_manifest_path.as_deref(),
        share_error.as_ref(),
        ctx,
    )?;

    if let Err(error) = summarize_result {
        return Ok(CampaignExecutionOutcome {
            items_failed,
            error: Some(format!(
                "campaign `{}` finished item execution but summarize failed: {}",
                campaign.id, error
            )),
            share_manifest_path,
            share_error,
        });
    }

    if items_failed > 0 {
        let failing = item_results
            .iter()
            .filter(|entry| !entry.ok)
            .map(|entry| {
                let error = entry.error.as_deref().unwrap_or("unknown error");
                format!("{} {}: {}", item_kind_str(entry.kind), entry.item_id, error)
            })
            .collect::<Vec<_>>()
            .join("; ");
        return Ok(CampaignExecutionOutcome {
            items_failed,
            error: Some(format!(
                "campaign `{}` completed with {} failed item(s) under {}: {}",
                campaign.id,
                items_failed,
                campaign_root.display(),
                failing
            )),
            share_manifest_path,
            share_error,
        });
    }

    Ok(CampaignExecutionOutcome {
        items_failed: 0,
        error: None,
        share_manifest_path,
        share_error,
    })
}

fn write_campaign_batch_artifacts(
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    ctx: &CampaignRunContext,
) -> Result<CampaignBatchArtifacts, String> {
    let created_unix_ms = now_unix_ms();
    let run_id = created_unix_ms.to_string();
    let selection_slug = campaign_batch_selection_slug(options, reports.len());
    let batch_root = ctx
        .resolved_out_dir
        .join("campaign-batches")
        .join(selection_slug)
        .join(&run_id);
    let summary_path =
        batch_root.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    let index_path = batch_root.join(crate::regression_summary::DIAG_REGRESSION_INDEX_FILENAME_V1);

    write_campaign_batch_manifest(&batch_root, &run_id, created_unix_ms, reports, options, ctx)?;

    let summarize_result = diag_summarize::cmd_summarize(diag_summarize::SummarizeCmdContext {
        rest: reports
            .iter()
            .map(|report| report.out_dir.display().to_string())
            .collect(),
        workspace_root: ctx.workspace_root.clone(),
        resolved_out_dir: batch_root.clone(),
        stats_json: false,
    });
    let summarize_error = summarize_result.err();
    let failed_runs = reports.iter().filter(|report| !report.ok).count();
    let (share_manifest_path, share_error) = maybe_write_failure_share_manifest(
        &batch_root,
        &summary_path,
        &ctx.workspace_root,
        failed_runs > 0,
        ctx.stats_top,
        ctx.warmup_frames,
    );

    let finished_unix_ms = now_unix_ms();
    let duration_ms = finished_unix_ms.saturating_sub(created_unix_ms);
    write_campaign_batch_result(
        &batch_root,
        &run_id,
        created_unix_ms,
        finished_unix_ms,
        duration_ms,
        reports,
        options,
        summarize_error.as_ref(),
        &summary_path,
        &index_path,
        share_manifest_path.as_deref(),
        share_error.as_ref(),
        ctx,
    )?;

    Ok(CampaignBatchArtifacts {
        batch_root,
        summary_path,
        index_path,
        share_manifest_path,
        summarize_error,
        share_error,
    })
}

fn run_campaign_item(
    index: usize,
    item: &CampaignItemDefinition,
    suite_results_root: &Path,
    script_results_root: &Path,
    ctx: &CampaignRunContext,
) -> Result<CampaignItemRunResult, String> {
    match item.kind {
        CampaignItemKind::Suite => {
            run_campaign_suite_item(index, &item.value, suite_results_root, ctx)
        }
        CampaignItemKind::Script => {
            run_campaign_script_item(index, &item.value, script_results_root, ctx)
        }
    }
}

fn run_campaign_suite_item(
    index: usize,
    suite_id: &str,
    suite_results_root: &Path,
    ctx: &CampaignRunContext,
) -> Result<CampaignItemRunResult, String> {
    let suite_dir =
        suite_results_root.join(format!("{:02}-{}", index + 1, zip_safe_component(suite_id)));
    std::fs::create_dir_all(&suite_dir).map_err(|e| {
        format!(
            "failed to create suite output dir {}: {}",
            suite_dir.display(),
            e
        )
    })?;

    let regression_summary_path =
        suite_dir.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    let result = diag_suite::cmd_suite(diag_suite::SuiteCmdContext {
        pack_after_run: ctx.pack_after_run,
        rest: vec![suite_id.to_string()],
        suite_script_inputs: ctx.suite_script_inputs.clone(),
        suite_prewarm_scripts: ctx.suite_prewarm_scripts.clone(),
        suite_prelude_scripts: ctx.suite_prelude_scripts.clone(),
        suite_prelude_each_run: ctx.suite_prelude_each_run,
        workspace_root: ctx.workspace_root.clone(),
        resolved_out_dir: suite_dir.clone(),
        resolved_ready_path: suite_dir.join("ready.touch"),
        resolved_script_result_path: suite_dir.join("script.result.json"),
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
    });

    Ok(CampaignItemRunResult {
        kind: CampaignItemKind::Suite,
        item_id: suite_id.to_string(),
        out_dir: suite_dir,
        regression_summary_path,
        ok: result.is_ok(),
        error: result.err(),
    })
}

fn run_campaign_script_item(
    index: usize,
    script_path: &str,
    script_results_root: &Path,
    ctx: &CampaignRunContext,
) -> Result<CampaignItemRunResult, String> {
    let script_dir = script_results_root.join(format!(
        "{:02}-{}",
        index + 1,
        zip_safe_component(script_path)
    ));
    std::fs::create_dir_all(&script_dir).map_err(|e| {
        format!(
            "failed to create script output dir {}: {}",
            script_dir.display(),
            e
        )
    })?;

    let regression_summary_path =
        script_dir.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    let result = diag_suite::cmd_suite(diag_suite::SuiteCmdContext {
        pack_after_run: ctx.pack_after_run,
        rest: Vec::new(),
        suite_script_inputs: vec![script_path.to_string()],
        suite_prewarm_scripts: ctx.suite_prewarm_scripts.clone(),
        suite_prelude_scripts: ctx.suite_prelude_scripts.clone(),
        suite_prelude_each_run: ctx.suite_prelude_each_run,
        workspace_root: ctx.workspace_root.clone(),
        resolved_out_dir: script_dir.clone(),
        resolved_ready_path: script_dir.join("ready.touch"),
        resolved_script_result_path: script_dir.join("script.result.json"),
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
    });

    Ok(CampaignItemRunResult {
        kind: CampaignItemKind::Script,
        item_id: script_path.to_string(),
        out_dir: script_dir,
        regression_summary_path,
        ok: result.is_ok(),
        error: result.err(),
    })
}

fn parse_campaign_run_options(rest: &[String]) -> Result<CampaignRunOptions, String> {
    let mut out = CampaignRunOptions::default();
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--lane" => {
                let value = rest
                    .get(index + 1)
                    .ok_or_else(|| "missing value after --lane".to_string())?;
                out.filter.lane = Some(parse_lane(value)?);
                index += 2;
            }
            "--tier" => {
                let value = rest
                    .get(index + 1)
                    .ok_or_else(|| "missing value after --tier".to_string())?;
                out.filter.tier = Some(value.to_string());
                index += 2;
            }
            "--tag" => {
                let value = rest
                    .get(index + 1)
                    .ok_or_else(|| "missing value after --tag".to_string())?;
                out.filter.tags.push(value.to_string());
                index += 2;
            }
            "--platform" => {
                let value = rest
                    .get(index + 1)
                    .ok_or_else(|| "missing value after --platform".to_string())?;
                out.filter.platforms.push(value.to_string());
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

    let mut bundles_total = 0usize;
    let mut bundles_packed = 0usize;
    let mut bundles_missing = 0usize;
    let mut triage_generated = 0usize;
    let mut triage_failed = 0usize;
    let mut run_entries = Vec::new();
    let mut combined_entries: Vec<(String, Option<PathBuf>, Option<PathBuf>, Option<PathBuf>)> =
        Vec::new();

    for item in &summary.items {
        if !include_passed && item.status == RegressionStatusV1::Passed {
            continue;
        }

        let bundle_dir = item
            .evidence
            .as_ref()
            .and_then(|evidence| evidence.bundle_dir.as_deref())
            .map(PathBuf::from);
        if bundle_dir.is_none() {
            bundles_missing = bundles_missing.saturating_add(1);
        }
        let (triage_path, triage_error) = if let Some(bundle_dir) = bundle_dir.as_deref() {
            maybe_write_bundle_triage_json(bundle_dir, stats_top, warmup_frames)
        } else {
            (None, None)
        };
        let screenshots_manifest = bundle_dir.as_deref().and_then(|bundle_dir| {
            crate::commands::screenshots::resolve_screenshots_manifest_path(bundle_dir)
                .map(|(_screenshots_dir, manifest_path)| manifest_path)
        });
        if triage_path.is_some() {
            triage_generated = triage_generated.saturating_add(1);
        }
        if triage_error.is_some() {
            triage_failed = triage_failed.saturating_add(1);
        }
        let pack_result = if let Some(bundle_dir) = bundle_dir.as_deref() {
            bundles_total = bundles_total.saturating_add(1);
            let packet_dir = bundle_dir.join("ai.packet");
            let ai_packet_result = crate::commands::ai_packet::ensure_ai_packet_dir_best_effort(
                None,
                bundle_dir,
                &packet_dir,
                true,
                stats_top,
                None,
                warmup_frames,
                None,
            );
            let share_zip = share_dir.join(format!(
                "{:02}-{}.ai.zip",
                bundles_total,
                zip_safe_component(&item.item_id)
            ));
            match ai_packet_result
                .and_then(|_| crate::pack_ai_packet_dir_to_zip(bundle_dir, &share_zip, root_dir))
            {
                Ok(()) => {
                    bundles_packed = bundles_packed.saturating_add(1);
                    Ok(share_zip)
                }
                Err(error) => Err(error),
            }
        } else {
            Err("item does not expose evidence.bundle_dir".to_string())
        };

        let pack_path = pack_result.as_ref().ok().cloned();
        let pack_error = pack_result.as_ref().err().cloned();
        run_entries.push(serde_json::json!({
            "item_id": item.item_id,
            "name": item.name,
            "status": item.status,
            "reason_code": item.reason_code,
            "bundle_dir": bundle_dir.as_ref().map(|path| path.display().to_string()),
            "triage_json": triage_path.as_ref().map(|path| path.display().to_string()),
            "triage_error": triage_error,
            "screenshots_manifest": screenshots_manifest.as_ref().map(|path| path.display().to_string()),
            "source_script": item
                .source
                .as_ref()
                .and_then(|source| source.script.clone()),
            "share_zip": pack_path.as_ref().map(|path| path.display().to_string()),
            "error": pack_error,
        }));
        combined_entries.push((
            item.item_id.clone(),
            pack_path,
            triage_path,
            screenshots_manifest,
        ));
    }

    let share_manifest_path = share_dir.join("share.manifest.json");
    let mut payload = serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_SHARE_MANIFEST_KIND_V1,
        "source": {
            "root_dir": root_dir.display().to_string(),
            "summary_path": summary_path.display().to_string(),
            "campaign_name": summary.campaign.name,
            "lane": summary.campaign.lane,
        },
        "selection": {
            "include_passed": include_passed,
        },
        "counters": {
            "items_selected": run_entries.len(),
            "bundles_total": bundles_total,
            "bundles_packed": bundles_packed,
            "bundles_missing": bundles_missing,
            "triage_generated": triage_generated,
            "triage_failed": triage_failed,
        },
        "share": {
            "share_dir": share_dir.display().to_string(),
            "workflow_hint": format!(
                "open {} in DevTools or share {} plus the generated *.ai.zip artifacts",
                root_dir.display(),
                summary_path.display()
            ),
            "workspace_root": workspace_root.display().to_string(),
            "combined_zip": serde_json::Value::Null,
            "combined_zip_error": serde_json::Value::Null,
        },
        "items": run_entries,
    });
    write_json_value(&share_manifest_path, &payload)?;

    let (combined_zip, combined_zip_error) = write_campaign_combined_failure_zip(
        root_dir,
        &share_dir,
        &share_manifest_path,
        summary_path,
        &combined_entries,
    );
    if let Some(share) = payload
        .get_mut("share")
        .and_then(|value| value.as_object_mut())
    {
        share.insert(
            "combined_zip".to_string(),
            combined_zip
                .as_ref()
                .map(|path| serde_json::Value::String(path.display().to_string()))
                .unwrap_or(serde_json::Value::Null),
        );
        share.insert(
            "combined_zip_error".to_string(),
            combined_zip_error
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );
    }
    write_json_value(&share_manifest_path, &payload)?;
    Ok(share_manifest_path)
}

fn write_campaign_combined_failure_zip(
    root_dir: &Path,
    share_dir: &Path,
    share_manifest_path: &Path,
    summary_path: &Path,
    entries: &[(String, Option<PathBuf>, Option<PathBuf>, Option<PathBuf>)],
) -> (Option<PathBuf>, Option<String>) {
    if !entries
        .iter()
        .any(|(_item_id, share_zip, triage_path, screenshots_manifest)| {
            share_zip.is_some() || triage_path.is_some() || screenshots_manifest.is_some()
        })
    {
        return (None, None);
    }

    let out_path = share_dir.join("combined-failures.zip");
    match write_campaign_combined_failure_zip_inner(
        root_dir,
        &out_path,
        share_manifest_path,
        summary_path,
        entries,
    ) {
        Ok(()) => (Some(out_path), None),
        Err(error) => (None, Some(error)),
    }
}

fn write_campaign_combined_failure_zip_inner(
    root_dir: &Path,
    out_path: &Path,
    share_manifest_path: &Path,
    summary_path: &Path,
    entries: &[(String, Option<PathBuf>, Option<PathBuf>, Option<PathBuf>)],
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

    for (index, (item_id, share_zip, triage_path, screenshots_manifest)) in
        entries.iter().enumerate()
    {
        let safe_item_id = zip_safe_component(item_id);
        if let Some(share_zip) = share_zip.as_deref()
            && share_zip.is_file()
        {
            add_file_to_zip(
                &mut zip,
                share_zip,
                &format!("items/{:02}-{safe_item_id}.ai.zip", index + 1),
                options,
            )?;
        }
        if let Some(triage_path) = triage_path.as_deref()
            && triage_path.is_file()
        {
            add_file_to_zip(
                &mut zip,
                triage_path,
                &format!("items/{:02}-{safe_item_id}.triage.json", index + 1),
                options,
            )?;
        }
        if let Some(screenshots_manifest) = screenshots_manifest.as_deref()
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
    serde_json::json!({
        "out_dir": batch.batch_root.display().to_string(),
        "summary_path": batch.summary_path.is_file().then(|| batch.summary_path.display().to_string()),
        "index_path": batch.index_path.is_file().then(|| batch.index_path.display().to_string()),
        "share_manifest_path": batch.share_manifest_path.as_ref().map(|path| path.display().to_string()),
        "summarize_error": batch.summarize_error,
        "share_error": batch.share_error,
    })
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

fn campaign_report_to_json(report: &CampaignExecutionReport) -> serde_json::Value {
    serde_json::json!({
        "campaign_id": report.campaign_id,
        "ok": report.ok,
        "error": report.error,
        "share_error": report.share_error,
        "out_dir": report.out_dir.display().to_string(),
        "campaign_result_path": report.out_dir.join("campaign.result.json").display().to_string(),
        "summary_path": report.summary_path.is_file().then(|| report.summary_path.display().to_string()),
        "index_path": report.index_path.is_file().then(|| report.index_path.display().to_string()),
        "share_manifest_path": report.share_manifest_path.as_ref().map(|path| path.display().to_string()),
        "items_total": report.items_total,
        "items_failed": report.items_failed,
        "suites_total": report.suites_total,
        "scripts_total": report.scripts_total,
    })
}

fn write_campaign_manifest(
    campaign_root: &Path,
    campaign: &CampaignDefinition,
    run_id: &str,
    created_unix_ms: u64,
    ctx: &CampaignRunContext,
) -> Result<(), String> {
    let manifest_path = campaign_root.join("campaign.manifest.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_MANIFEST_KIND_V1,
        "campaign": campaign_to_json(campaign),
        "run": {
            "run_id": run_id,
            "created_unix_ms": created_unix_ms,
            "tool": "fretboard diag campaign",
            "workspace_root": ctx.workspace_root.display().to_string(),
            "out_dir": campaign_root.display().to_string(),
        },
        "resolved": {
            "item_count": campaign.items.len(),
            "items": campaign.items.iter().map(item_to_manifest_json).collect::<Vec<_>>(),
            "suite_count": campaign.suite_count(),
            "script_count": campaign.script_count(),
            "suites": campaign.suites(),
            "scripts": campaign.scripts(),
            "launch": ctx.launch,
            "launch_env": ctx.launch_env,
        }
    });
    write_json_value(&manifest_path, &payload)
}

fn write_campaign_batch_manifest(
    batch_root: &Path,
    run_id: &str,
    created_unix_ms: u64,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    ctx: &CampaignRunContext,
) -> Result<(), String> {
    let manifest_path = batch_root.join("batch.manifest.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_BATCH_MANIFEST_KIND_V1,
        "selection": {
            "selection_slug": campaign_batch_selection_slug(options, reports.len()),
            "campaign_ids": &options.campaign_ids,
            "filters": campaign_filter_to_json(&options.filter),
            "selected_campaign_ids": reports.iter().map(|report| report.campaign_id.as_str()).collect::<Vec<_>>(),
        },
        "run": {
            "run_id": run_id,
            "created_unix_ms": created_unix_ms,
            "tool": "fretboard diag campaign",
            "workspace_root": ctx.workspace_root.display().to_string(),
            "out_dir": batch_root.display().to_string(),
        },
        "resolved": {
            "campaigns_total": reports.len(),
            "items_total": reports.iter().map(|report| report.items_total).sum::<usize>(),
            "suites_total": reports.iter().map(|report| report.suites_total).sum::<usize>(),
            "scripts_total": reports.iter().map(|report| report.scripts_total).sum::<usize>(),
            "runs": reports.iter().map(campaign_report_to_json).collect::<Vec<_>>(),
            "launch": ctx.launch,
            "launch_env": ctx.launch_env,
        }
    });
    write_json_value(&manifest_path, &payload)
}

#[allow(clippy::too_many_arguments)]
fn write_campaign_result(
    campaign_root: &Path,
    campaign: &CampaignDefinition,
    run_id: &str,
    created_unix_ms: u64,
    finished_unix_ms: u64,
    duration_ms: u64,
    item_results: &[CampaignItemRunResult],
    summarize_error: Option<&String>,
    summary_path: &Path,
    index_path: &Path,
    share_manifest_path: Option<&Path>,
    share_error: Option<&String>,
    ctx: &CampaignRunContext,
) -> Result<(), String> {
    let result_path = campaign_root.join("campaign.result.json");
    let suites_total = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Suite)
        .count() as u64;
    let scripts_total = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Script)
        .count() as u64;
    let suites_failed = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Suite && !entry.ok)
        .count() as u64;
    let scripts_failed = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Script && !entry.ok)
        .count() as u64;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_RESULT_KIND_V1,
        "campaign": campaign_to_json(campaign),
        "run": {
            "run_id": run_id,
            "created_unix_ms": created_unix_ms,
            "finished_unix_ms": finished_unix_ms,
            "duration_ms": duration_ms,
            "tool": "fretboard diag campaign",
            "workspace_root": ctx.workspace_root.display().to_string(),
            "out_dir": campaign_root.display().to_string(),
        },
        "counters": {
            "items_total": item_results.len(),
            "items_passed": item_results.iter().filter(|entry| entry.ok).count(),
            "items_failed": item_results.iter().filter(|entry| !entry.ok).count(),
            "suites_total": suites_total,
            "suites_failed": suites_failed,
            "scripts_total": scripts_total,
            "scripts_failed": scripts_failed,
        },
        "aggregate": {
            "summary_path": summary_path.is_file().then(|| summary_path.display().to_string()),
            "index_path": index_path.is_file().then(|| index_path.display().to_string()),
            "share_manifest_path": share_manifest_path.map(|path| path.display().to_string()),
            "summarize_error": summarize_error.cloned(),
            "share_error": share_error.cloned(),
        },
        "item_results": item_results.iter().map(|entry| serde_json::json!({
            "kind": item_kind_str(entry.kind),
            "item_id": entry.item_id,
            "ok": entry.ok,
            "error": entry.error,
            "out_dir": entry.out_dir.display().to_string(),
            "regression_summary_path": entry
                .regression_summary_path
                .is_file()
                .then(|| entry.regression_summary_path.display().to_string()),
        })).collect::<Vec<_>>(),
    });
    write_json_value(&result_path, &payload)
}

#[allow(clippy::too_many_arguments)]
fn write_campaign_batch_result(
    batch_root: &Path,
    run_id: &str,
    created_unix_ms: u64,
    finished_unix_ms: u64,
    duration_ms: u64,
    reports: &[CampaignExecutionReport],
    options: &CampaignRunOptions,
    summarize_error: Option<&String>,
    summary_path: &Path,
    index_path: &Path,
    share_manifest_path: Option<&Path>,
    share_error: Option<&String>,
    ctx: &CampaignRunContext,
) -> Result<(), String> {
    let result_path = batch_root.join("batch.result.json");
    let campaigns_failed = reports.iter().filter(|report| !report.ok).count();
    let items_total = reports
        .iter()
        .map(|report| report.items_total)
        .sum::<usize>();
    let items_failed = reports
        .iter()
        .map(|report| report.items_failed)
        .sum::<usize>();
    let suites_total = reports
        .iter()
        .map(|report| report.suites_total)
        .sum::<usize>();
    let scripts_total = reports
        .iter()
        .map(|report| report.scripts_total)
        .sum::<usize>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": DIAG_CAMPAIGN_BATCH_RESULT_KIND_V1,
        "selection": {
            "selection_slug": campaign_batch_selection_slug(options, reports.len()),
            "campaign_ids": &options.campaign_ids,
            "filters": campaign_filter_to_json(&options.filter),
        },
        "run": {
            "run_id": run_id,
            "created_unix_ms": created_unix_ms,
            "finished_unix_ms": finished_unix_ms,
            "duration_ms": duration_ms,
            "tool": "fretboard diag campaign",
            "workspace_root": ctx.workspace_root.display().to_string(),
            "out_dir": batch_root.display().to_string(),
        },
        "counters": {
            "campaigns_total": reports.len(),
            "campaigns_passed": reports.len().saturating_sub(campaigns_failed),
            "campaigns_failed": campaigns_failed,
            "items_total": items_total,
            "items_passed": items_total.saturating_sub(items_failed),
            "items_failed": items_failed,
            "suites_total": suites_total,
            "scripts_total": scripts_total,
        },
        "aggregate": {
            "summary_path": summary_path.is_file().then(|| summary_path.display().to_string()),
            "index_path": index_path.is_file().then(|| index_path.display().to_string()),
            "share_manifest_path": share_manifest_path.map(|path| path.display().to_string()),
            "summarize_error": summarize_error.cloned(),
            "share_error": share_error.cloned(),
        },
        "runs": reports.iter().map(campaign_report_to_json).collect::<Vec<_>>(),
    });
    write_json_value(&result_path, &payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regression_summary::{
        RegressionArtifactsV1, RegressionCampaignSummaryV1, RegressionEvidenceV1,
        RegressionHighlightsV1, RegressionItemKindV1, RegressionItemSummaryV1, RegressionLaneV1,
        RegressionRunSummaryV1, RegressionSummaryV1, RegressionTotalsV1,
    };

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
