use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn run_suite_aux_script_must_pass(
    src: &PathBuf,
    child: &mut Option<LaunchedDemo>,
    use_devtools_ws: bool,
    connected_ws: Option<&ConnectedToolingTransport>,
    workspace_root: &Path,
    resolved_out_dir: &Path,
    resolved_exit_path: &Path,
    reuse_process: bool,
    resolved_script_path: &Path,
    resolved_script_trigger_path: &Path,
    resolved_script_result_path: &Path,
    resolved_script_result_trigger_path: &Path,
    perf_capabilities_check_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(), String> {
    if use_devtools_ws {
        let connected = connected_ws
            .ok_or_else(|| "missing DevTools WS transport (this is a tooling bug)".to_string())?;
        let script_key = normalize_repo_relative_path(workspace_root, src);
        let script_value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(src).map_err(|e| {
                let err = e.to_string();
                write_tooling_failure_script_result(
                    resolved_script_result_path,
                    "tooling.script.read_failed",
                    &err,
                    "tooling_error",
                    Some(script_key.clone()),
                );
                err
            })?)
            .map_err(|e| {
                let err = e.to_string();
                write_tooling_failure_script_result(
                    resolved_script_result_path,
                    "tooling.script.parse_failed",
                    &err,
                    "tooling_error",
                    Some(script_key.clone()),
                );
                err
            })?;
        let script_json =
            crate::script_tooling::resolve_script_json_redirects_from_value(src, script_value)
                .inspect_err(|err| {
                    write_tooling_failure_script_result(
                        resolved_script_result_path,
                        "tooling.script.redirect_failed",
                        err,
                        "tooling_error",
                        Some(script_key.clone()),
                    );
                })?
                .value;
        let (result, _bundle_path) = run_script_over_transport(
            resolved_out_dir,
            connected,
            script_json,
            false,
            false,
            None,
            None,
            timeout_ms,
            poll_ms,
            resolved_script_result_path,
            perf_capabilities_check_path,
        )
        .inspect_err(|err| {
            write_tooling_failure_script_result_if_missing(
                resolved_script_result_path,
                "tooling.run.failed",
                err,
                "tooling_error",
                Some(script_key.clone()),
            );
        })?;

        match result.stage {
            fret_diag_protocol::UiScriptStageV1::Passed => return Ok(()),
            fret_diag_protocol::UiScriptStageV1::Failed => {
                eprintln!(
                    "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                    src.display(),
                    result.run_id,
                    result.step_index.unwrap_or(0),
                    result.reason.as_deref().unwrap_or("unknown"),
                    result.last_bundle_dir.as_deref().unwrap_or("")
                );
                stop_launched_demo(child, resolved_exit_path, poll_ms);
                std::process::exit(1);
            }
            _ => {
                eprintln!(
                    "unexpected script stage for {}: {:?}",
                    src.display(),
                    result
                );
                stop_launched_demo(child, resolved_exit_path, poll_ms);
                std::process::exit(1);
            }
        }
    }

    if !reuse_process {
        clear_script_result_files(
            resolved_script_result_path,
            resolved_script_result_trigger_path,
        );
    }

    let mut result = run_script_and_wait(
        src,
        resolved_script_path,
        resolved_script_trigger_path,
        resolved_script_result_path,
        resolved_script_result_trigger_path,
        timeout_ms,
        poll_ms,
    );
    if let Ok(summary) = &result
        && summary.stage.as_deref() == Some("failed")
        && let Some(dir) =
            wait_for_failure_dump_bundle(resolved_out_dir, summary, timeout_ms, poll_ms)
        && let Some(name) = dir.file_name().and_then(|s| s.to_str())
        && let Ok(summary) = result.as_mut()
    {
        summary.last_bundle_dir = Some(name.to_string());
    }

    match result {
        Ok(result) => match result.stage.as_deref() {
            Some("passed") => Ok(()),
            Some("failed") => {
                eprintln!(
                    "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                    src.display(),
                    result.run_id,
                    result.step_index.unwrap_or(0),
                    result.reason.as_deref().unwrap_or("unknown"),
                    result.last_bundle_dir.as_deref().unwrap_or("")
                );
                stop_launched_demo(child, resolved_exit_path, poll_ms);
                std::process::exit(1);
            }
            _ => {
                eprintln!(
                    "unexpected script stage for {}: {:?}",
                    src.display(),
                    result
                );
                stop_launched_demo(child, resolved_exit_path, poll_ms);
                std::process::exit(1);
            }
        },
        Err(err) => {
            stop_launched_demo(child, resolved_exit_path, poll_ms);
            Err(err)
        }
    }
}
