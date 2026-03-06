use super::*;

use crate::registry::campaigns::{
    CampaignDefinition, CampaignRegistry, campaign_to_json, lane_to_str, parse_lane,
};
use crate::regression_summary::RegressionLaneV1;

const DIAG_CAMPAIGN_MANIFEST_KIND_V1: &str = "diag_campaign_manifest";
const DIAG_CAMPAIGN_RESULT_KIND_V1: &str = "diag_campaign_result";

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

#[derive(Debug, Clone)]
struct CampaignRunOptions {
    requested_lane: Option<RegressionLaneV1>,
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
struct CampaignSuiteRunResult {
    suite_id: String,
    out_dir: PathBuf,
    regression_summary_path: PathBuf,
    ok: bool,
    error: Option<String>,
}

fn cmd_campaign_list(
    registry: &CampaignRegistry,
    rest: &[String],
    json: bool,
) -> Result<(), String> {
    if let Some(other) = rest.first() {
        return Err(format!(
            "unexpected positional for `diag campaign list`: {other}"
        ));
    }

    if json {
        let payload = serde_json::json!({
            "campaigns": registry
                .list_campaigns()
                .iter()
                .map(campaign_to_json)
                .collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    for campaign in registry.list_campaigns() {
        println!(
            "{} ({}, {} suites) - {}",
            campaign.id.as_str(),
            lane_to_str(campaign.lane),
            campaign.suites.len(),
            campaign.description
        );
    }

    Ok(())
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
    if let Some(profile) = campaign.profile.as_deref() {
        println!("profile: {profile}");
    }
    if !campaign.tags.is_empty() {
        println!("tags: {}", campaign.tags.join(", "));
    }
    println!("suites:");
    for suite in &campaign.suites {
        println!("  - {suite}");
    }

    Ok(())
}

fn cmd_campaign_run(
    registry: &CampaignRegistry,
    rest: &[String],
    ctx: CampaignRunContext,
) -> Result<(), String> {
    let Some(campaign_id) = rest.first() else {
        return Err("missing campaign id for `diag campaign run`".to_string());
    };
    let options = parse_campaign_run_options(&rest[1..])?;
    let campaign = registry.resolve(campaign_id)?;

    if let Some(requested_lane) = options.requested_lane
        && requested_lane != campaign.lane
    {
        return Err(format!(
            "campaign `{}` is lane `{}` but `--lane {}` was requested",
            campaign.id.as_str(),
            lane_to_str(campaign.lane),
            lane_to_str(requested_lane)
        ));
    }

    let created_unix_ms = now_unix_ms();
    let run_id = created_unix_ms.to_string();
    let campaign_root = ctx
        .resolved_out_dir
        .join("campaigns")
        .join(zip_safe_component(&campaign.id))
        .join(&run_id);
    let suite_results_root = campaign_root.join("suite-results");
    std::fs::create_dir_all(&suite_results_root).map_err(|e| {
        format!(
            "failed to create campaign output dir {}: {}",
            suite_results_root.display(),
            e
        )
    })?;

    write_campaign_manifest(&campaign_root, campaign, &run_id, created_unix_ms, &ctx)?;

    let mut suite_results: Vec<CampaignSuiteRunResult> = Vec::new();
    for (index, suite_id) in campaign.suites.iter().enumerate() {
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
        let suite_result = match diag_suite::cmd_suite(diag_suite::SuiteCmdContext {
            pack_after_run: ctx.pack_after_run,
            rest: vec![suite_id.clone()],
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
        }) {
            Ok(()) => CampaignSuiteRunResult {
                suite_id: suite_id.clone(),
                out_dir: suite_dir,
                regression_summary_path,
                ok: true,
                error: None,
            },
            Err(error) => CampaignSuiteRunResult {
                suite_id: suite_id.clone(),
                out_dir: suite_dir,
                regression_summary_path,
                ok: false,
                error: Some(error),
            },
        };
        suite_results.push(suite_result);
    }

    let summarize_result = diag_summarize::cmd_summarize(diag_summarize::SummarizeCmdContext {
        rest: Vec::new(),
        workspace_root: ctx.workspace_root.clone(),
        resolved_out_dir: campaign_root.clone(),
        stats_json: false,
    });

    let finished_unix_ms = now_unix_ms();
    let duration_ms = finished_unix_ms.saturating_sub(created_unix_ms);
    let summary_path =
        campaign_root.join(crate::regression_summary::DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    let index_path =
        campaign_root.join(crate::regression_summary::DIAG_REGRESSION_INDEX_FILENAME_V1);
    write_campaign_result(
        &campaign_root,
        campaign,
        &run_id,
        created_unix_ms,
        finished_unix_ms,
        duration_ms,
        &suite_results,
        summarize_result.as_ref().err(),
        &summary_path,
        &index_path,
        &ctx,
    )?;

    if let Err(error) = summarize_result {
        return Err(format!(
            "campaign `{}` finished suite execution but summarize failed: {}",
            campaign.id.as_str(),
            error
        ));
    }

    let suites_failed = suite_results.iter().filter(|entry| !entry.ok).count();
    if suites_failed > 0 {
        let failing = suite_results
            .iter()
            .filter(|entry| !entry.ok)
            .map(|entry| {
                let error = entry.error.as_deref().unwrap_or("unknown error");
                format!("{}: {}", entry.suite_id, error)
            })
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!(
            "campaign `{}` completed with {} failed suite(s) under {}: {}",
            campaign.id.as_str(),
            suites_failed,
            campaign_root.display(),
            failing
        ));
    }

    if ctx.stats_json {
        let payload = serde_json::json!({
            "campaign_id": campaign.id,
            "run_id": run_id,
            "lane": campaign.lane,
            "out_dir": campaign_root.display().to_string(),
            "summary_path": summary_path.display().to_string(),
            "index_path": index_path.display().to_string(),
            "suites_total": suite_results.len(),
            "suites_failed": suites_failed,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "campaign: ok (id={}, suites={}, suites_failed={}, out_dir={})",
            campaign.id.as_str(),
            suite_results.len(),
            suites_failed,
            campaign_root.display()
        );
    }

    Ok(())
}

fn parse_campaign_run_options(rest: &[String]) -> Result<CampaignRunOptions, String> {
    let mut out = CampaignRunOptions {
        requested_lane: None,
    };
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--lane" => {
                let raw_lane = rest
                    .get(index + 1)
                    .ok_or_else(|| "missing value after --lane".to_string())?;
                out.requested_lane = Some(parse_lane(raw_lane)?);
                index += 2;
            }
            other => {
                return Err(format!("unknown diag campaign run flag: {other}"));
            }
        }
    }
    Ok(out)
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
            "suite_count": campaign.suites.len(),
            "suites": campaign.suites,
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
    suite_results: &[CampaignSuiteRunResult],
    summarize_error: Option<&String>,
    summary_path: &Path,
    index_path: &Path,
    ctx: &CampaignRunContext,
) -> Result<(), String> {
    let result_path = campaign_root.join("campaign.result.json");
    let suites_failed = suite_results.iter().filter(|entry| !entry.ok).count() as u64;
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
            "suites_total": suite_results.len(),
            "suites_passed": suite_results.len() as u64 - suites_failed,
            "suites_failed": suites_failed,
        },
        "aggregate": {
            "summary_path": summary_path.is_file().then(|| summary_path.display().to_string()),
            "index_path": index_path.is_file().then(|| index_path.display().to_string()),
            "summarize_error": summarize_error.cloned(),
        },
        "suite_results": suite_results.iter().map(|entry| serde_json::json!({
            "suite_id": entry.suite_id,
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
