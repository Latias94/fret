use super::*;

use crate::registry::campaigns::{
    CampaignDefinition, CampaignFilterOptions, CampaignItemDefinition, CampaignItemKind,
    CampaignRegistry, campaign_to_json, item_kind_str, lane_to_str, parse_lane, source_kind_str,
};

const DIAG_CAMPAIGN_MANIFEST_KIND_V1: &str = "diag_campaign_manifest";
const DIAG_CAMPAIGN_RESULT_KIND_V1: &str = "diag_campaign_result";
const DIAG_CAMPAIGN_BATCH_MANIFEST_KIND_V1: &str = "diag_campaign_batch_manifest";
const DIAG_CAMPAIGN_BATCH_RESULT_KIND_V1: &str = "diag_campaign_batch_result";

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
}

#[derive(Debug, Clone)]
struct CampaignBatchArtifacts {
    batch_root: PathBuf,
    summary_path: PathBuf,
    index_path: PathBuf,
    summarize_error: Option<String>,
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
                "out_dir": report.out_dir.display().to_string(),
                "summary_path": report.summary_path.display().to_string(),
                "index_path": report.index_path.display().to_string(),
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
        }
    } else {
        println!(
            "campaign batch: {} run(s), {} failed",
            reports.len(),
            failed_runs
        );
        if let Some(batch) = &batch {
            println!("  batch_root: {}", batch.batch_root.display());
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

    let (items_failed, report_error) =
        match execute_campaign_inner(campaign, ctx, &campaign_root, created_unix_ms, &run_id) {
            Ok(outcome) => (outcome.items_failed, outcome.error),
            Err(error) => (campaign.items.len(), Some(error)),
        };

    CampaignExecutionReport {
        campaign_id: campaign.id.clone(),
        out_dir: campaign_root,
        summary_path,
        index_path,
        items_total: campaign.items.len(),
        items_failed,
        suites_total: campaign.suite_count(),
        scripts_total: campaign.script_count(),
        ok: report_error.is_none(),
        error: report_error,
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
        ctx,
    )?;

    if let Err(error) = summarize_result {
        return Ok(CampaignExecutionOutcome {
            items_failed: item_results.iter().filter(|entry| !entry.ok).count(),
            error: Some(format!(
                "campaign `{}` finished item execution but summarize failed: {}",
                campaign.id, error
            )),
        });
    }

    let items_failed = item_results.iter().filter(|entry| !entry.ok).count();
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
        });
    }

    Ok(CampaignExecutionOutcome {
        items_failed: 0,
        error: None,
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
        ctx,
    )?;

    Ok(CampaignBatchArtifacts {
        batch_root,
        summary_path,
        index_path,
        summarize_error,
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

fn campaign_batch_to_json(batch: &CampaignBatchArtifacts) -> serde_json::Value {
    serde_json::json!({
        "out_dir": batch.batch_root.display().to_string(),
        "summary_path": batch.summary_path.is_file().then(|| batch.summary_path.display().to_string()),
        "index_path": batch.index_path.is_file().then(|| batch.index_path.display().to_string()),
        "summarize_error": batch.summarize_error,
    })
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
        "out_dir": report.out_dir.display().to_string(),
        "campaign_result_path": report.out_dir.join("campaign.result.json").display().to_string(),
        "summary_path": report.summary_path.is_file().then(|| report.summary_path.display().to_string()),
        "index_path": report.index_path.is_file().then(|| report.index_path.display().to_string()),
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
            "summarize_error": summarize_error.cloned(),
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
            "summarize_error": summarize_error.cloned(),
        },
        "runs": reports.iter().map(campaign_report_to_json).collect::<Vec<_>>(),
    });
    write_json_value(&result_path, &payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regression_summary::RegressionLaneV1;

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
}
