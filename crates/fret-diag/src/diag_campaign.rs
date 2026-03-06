use super::*;

use crate::registry::campaigns::{
    CampaignDefinition, CampaignRegistry, campaign_to_json, lane_to_str, parse_lane,
    source_kind_str,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CampaignItemKind {
    Suite,
    Script,
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
        let mut details = vec![
            lane_to_str(campaign.lane).to_string(),
            format!("suites={}", campaign.suites.len()),
            format!("scripts={}", campaign.scripts.len()),
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
    println!("suites ({}):", campaign.suites.len());
    for suite in &campaign.suites {
        println!("  - {suite}");
    }
    println!("scripts ({}):", campaign.scripts.len());
    for script in &campaign.scripts {
        println!("  - {script}");
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
            campaign.id,
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

    write_campaign_manifest(&campaign_root, campaign, &run_id, created_unix_ms, &ctx)?;

    let mut item_results: Vec<CampaignItemRunResult> = Vec::new();
    for (index, suite_id) in campaign.suites.iter().enumerate() {
        item_results.push(run_campaign_suite_item(
            index,
            suite_id,
            &suite_results_root,
            &ctx,
        )?);
    }
    for (index, script_path) in campaign.scripts.iter().enumerate() {
        item_results.push(run_campaign_script_item(
            index,
            script_path,
            &script_results_root,
            &ctx,
        )?);
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
        &item_results,
        summarize_result.as_ref().err(),
        &summary_path,
        &index_path,
        &ctx,
    )?;

    if let Err(error) = summarize_result {
        return Err(format!(
            "campaign `{}` finished item execution but summarize failed: {}",
            campaign.id, error
        ));
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
        return Err(format!(
            "campaign `{}` completed with {} failed item(s) under {}: {}",
            campaign.id,
            items_failed,
            campaign_root.display(),
            failing
        ));
    }

    let suites_total = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Suite)
        .count();
    let scripts_total = item_results
        .iter()
        .filter(|entry| entry.kind == CampaignItemKind::Script)
        .count();

    if ctx.stats_json {
        let payload = serde_json::json!({
            "campaign_id": campaign.id,
            "run_id": run_id,
            "lane": campaign.lane,
            "out_dir": campaign_root.display().to_string(),
            "summary_path": summary_path.display().to_string(),
            "index_path": index_path.display().to_string(),
            "items_total": item_results.len(),
            "items_failed": items_failed,
            "suites_total": suites_total,
            "scripts_total": scripts_total,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "campaign: ok (id={}, items={}, suites={}, scripts={}, out_dir={})",
            campaign.id,
            item_results.len(),
            suites_total,
            scripts_total,
            campaign_root.display()
        );
    }

    Ok(())
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

fn item_kind_str(kind: CampaignItemKind) -> &'static str {
    match kind {
        CampaignItemKind::Suite => "suite",
        CampaignItemKind::Script => "script",
    }
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
            "script_count": campaign.scripts.len(),
            "suites": campaign.suites,
            "scripts": campaign.scripts,
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
