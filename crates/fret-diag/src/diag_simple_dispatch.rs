use std::path::{Path, PathBuf};

use crate::commands;

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_simple(
    sub: &str,
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    resolved_out_dir: &Path,
    resolved_trigger_path: &Path,
    trace_out: Option<PathBuf>,
    pack_out: &Option<PathBuf>,
    ensure_ai_packet: bool,
    pack_ai_only: bool,
    pack_include_root_artifacts: bool,
    pack_include_triage: bool,
    pack_include_screenshots: bool,
    pack_schema2_only: bool,
    triage_out: &Option<PathBuf>,
    lint_out: &Option<PathBuf>,
    meta_out: &Option<PathBuf>,
    meta_report: bool,
    index_out: &Option<PathBuf>,
    test_ids_out: &Option<PathBuf>,
    hotspots_out: &Option<PathBuf>,
    bundle_v2_out: &Option<PathBuf>,
    query_out: &Option<PathBuf>,
    slice_out: &Option<PathBuf>,
    ai_packet_out: &Option<PathBuf>,
    stats_top: usize,
    sort_override: Option<crate::stats::BundleStatsSort>,
    warmup_frames: u64,
    stats_json: bool,
    max_test_ids: usize,
    lint_all_test_ids_bounds: bool,
    lint_eps_px: f32,
    resolved_script_path: &Path,
    resolved_script_trigger_path: &Path,
    resolved_script_result_path: &Path,
    resolved_script_result_trigger_path: &Path,
    resolved_ready_path: &Path,
    resolved_exit_path: &Path,
    script_tool_check: bool,
    script_tool_write: bool,
    script_tool_check_out: &Option<PathBuf>,
    shrink_out: &Option<PathBuf>,
    shrink_any_fail: bool,
    shrink_match_reason_code: &Option<String>,
    shrink_match_reason: &Option<String>,
    shrink_min_steps: u64,
    shrink_max_iters: u64,
    launch: &Option<Vec<String>>,
    launch_env: &[(String, String)],
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<Result<(), String>> {
    let res = match sub {
        "path" => commands::session::cmd_path(rest, pack_after_run, resolved_trigger_path),
        "poke" => commands::session::cmd_poke(
            rest,
            pack_after_run,
            resolved_out_dir,
            resolved_trigger_path,
            timeout_ms,
            poll_ms,
        ),
        "latest" => commands::session::cmd_latest(rest, pack_after_run, resolved_out_dir),
        "trace" => {
            if pack_after_run {
                return Some(Err("--pack is only supported with `diag run`".to_string()));
            }
            (|| -> Result<(), String> {
                let Some(src) = rest.first().cloned() else {
                    return Err(
                        "missing bundle artifact path (try: fretboard diag trace <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                            .to_string(),
                    );
                };
                if rest.len() != 1 {
                    return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
                }

                let mut src = crate::resolve_path(workspace_root, PathBuf::from(src));
                if src.is_dir()
                    && crate::resolve_bundle_artifact_path_no_materialize(&src).is_none()
                    && (commands::resolve::looks_like_diag_session_root(&src)
                        || src.join(crate::session::SESSIONS_DIRNAME).is_dir())
                    && let Ok((bundle_dir, _session_id, _source)) =
                        commands::resolve::resolve_latest_bundle_dir_from_base_or_session_out_dir(
                            &src, None,
                        )
                {
                    src = bundle_dir;
                }
                let bundle_path = crate::resolve_bundle_artifact_path(&src);
                let bundle_dir = crate::resolve_bundle_root_dir(&bundle_path)?;
                let out = trace_out
                    .map(|p| crate::resolve_path(workspace_root, p))
                    .unwrap_or_else(|| bundle_dir.join("trace.chrome.json"));
                crate::trace::write_chrome_trace_from_bundle_path(&bundle_path, &out)?;
                println!("{}", out.display());
                Ok(())
            })()
        }
        "pack" => commands::artifacts::cmd_pack(
            rest,
            workspace_root,
            resolved_out_dir,
            pack_out.clone(),
            ensure_ai_packet,
            pack_ai_only,
            pack_include_root_artifacts,
            pack_include_triage,
            pack_include_screenshots,
            pack_schema2_only,
            stats_top,
            sort_override,
            warmup_frames,
        ),
        "triage" => commands::artifacts::cmd_triage(
            rest,
            pack_after_run,
            workspace_root,
            triage_out.clone(),
            stats_top,
            sort_override,
            warmup_frames,
            stats_json,
        ),
        "lint" => commands::artifacts::cmd_lint(
            rest,
            pack_after_run,
            workspace_root,
            lint_out.clone(),
            lint_all_test_ids_bounds,
            lint_eps_px,
            warmup_frames,
            stats_json,
        ),
        "meta" => commands::artifacts::cmd_meta(
            rest,
            pack_after_run,
            workspace_root,
            meta_out.clone(),
            warmup_frames,
            stats_json,
            meta_report,
        ),
        "windows" => commands::windows::cmd_windows(
            rest,
            pack_after_run,
            workspace_root,
            warmup_frames,
            stats_json,
        ),
        "dock-routing" => commands::dock_routing::cmd_dock_routing(
            rest,
            pack_after_run,
            workspace_root,
            warmup_frames,
            stats_json,
        ),
        "screenshots" => {
            commands::screenshots::cmd_screenshots(rest, pack_after_run, workspace_root, stats_json)
        }
        "resolve" => commands::resolve::cmd_resolve(
            rest,
            pack_after_run,
            workspace_root,
            resolved_out_dir,
            stats_json,
        ),
        "index" => commands::index::cmd_index(
            rest,
            pack_after_run,
            workspace_root,
            index_out.clone(),
            warmup_frames,
            stats_json,
        ),
        "test-ids" => commands::artifacts::cmd_test_ids(
            rest,
            pack_after_run,
            workspace_root,
            test_ids_out.clone(),
            warmup_frames,
            max_test_ids,
            stats_json,
        ),
        "test-ids-index" => commands::artifacts::cmd_test_ids_index(
            rest,
            pack_after_run,
            workspace_root,
            warmup_frames,
            stats_json,
        ),
        "frames-index" => commands::artifacts::cmd_frames_index(
            rest,
            pack_after_run,
            workspace_root,
            warmup_frames,
            stats_json,
        ),
        "doctor" => commands::doctor::cmd_doctor(
            rest,
            pack_after_run,
            workspace_root,
            resolved_out_dir,
            warmup_frames,
            stats_json,
        ),
        "agent" => commands::agent::cmd_agent(
            rest,
            pack_after_run,
            workspace_root,
            resolved_out_dir,
            warmup_frames,
            stats_json,
        ),
        "hotspots" => commands::hotspots::cmd_hotspots(
            rest,
            pack_after_run,
            workspace_root,
            resolved_out_dir,
            hotspots_out.clone(),
            warmup_frames,
            stats_json,
        ),
        "bundle-v2" => commands::bundle_v2::cmd_bundle_v2(
            rest,
            pack_after_run,
            workspace_root,
            resolved_out_dir,
            bundle_v2_out.clone(),
            stats_json,
        ),
        "ai-packet" => commands::ai_packet::cmd_ai_packet(
            rest,
            pack_after_run,
            workspace_root,
            resolved_out_dir,
            ai_packet_out.clone(),
            pack_include_triage,
            stats_top,
            sort_override,
            warmup_frames,
        ),
        "query" => commands::query::cmd_query(
            rest,
            pack_after_run,
            workspace_root,
            resolved_out_dir,
            query_out.clone(),
            warmup_frames,
            stats_json,
        ),
        "slice" => commands::slice::cmd_slice(
            rest,
            pack_after_run,
            workspace_root,
            resolved_out_dir,
            slice_out.clone(),
            warmup_frames,
            stats_json,
        ),
        "script" => commands::script::cmd_script(
            rest,
            pack_after_run,
            workspace_root,
            resolved_out_dir,
            resolved_script_path,
            resolved_script_trigger_path,
            resolved_script_result_path,
            resolved_script_result_trigger_path,
            resolved_ready_path,
            resolved_exit_path,
            script_tool_check,
            script_tool_write,
            script_tool_check_out.clone(),
            shrink_out.clone(),
            shrink_any_fail,
            shrink_match_reason_code.clone(),
            shrink_match_reason.clone(),
            shrink_min_steps,
            shrink_max_iters,
            launch,
            launch_env,
            timeout_ms,
            poll_ms,
            stats_json,
        ),
        _ => return None,
    };

    Some(res)
}
