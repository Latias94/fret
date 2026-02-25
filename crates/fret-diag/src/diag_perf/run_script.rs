use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn run_perf_script_and_resolve_bundle_artifact_path(
    src: &PathBuf,
    script_key: &str,
    child: &mut Option<LaunchedDemo>,
    use_devtools_ws: bool,
    connected_ws: Option<&ConnectedToolingTransport>,
    resolved_out_dir: &Path,
    resolved_exit_path: &Path,
    resolved_script_path: &Path,
    resolved_script_trigger_path: &Path,
    resolved_script_result_path: &Path,
    resolved_script_result_trigger_path: &Path,
    perf_capabilities_check_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<Option<PathBuf>, String> {
    if use_devtools_ws {
        let connected = connected_ws.ok_or_else(|| {
            "missing DevTools WS transport (this is a tooling bug)".to_string()
        })?;
        let script_json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(src).map_err(|e| {
                let err = e.to_string();
                write_tooling_failure_script_result(
                    resolved_script_result_path,
                    "tooling.script.read_failed",
                    &err,
                    "tooling_error",
                    Some(script_key.to_string()),
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
                    Some(script_key.to_string()),
                );
                err
            })?;
        let (result, bundle_path) = run_script_over_transport(
            resolved_out_dir,
            connected,
            script_json,
            true,
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
                Some(script_key.to_string()),
            );
        })?;

        match result.stage {
            fret_diag_protocol::UiScriptStageV1::Passed => {}
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
                eprintln!("unexpected script stage for {}: {:?}", src.display(), result);
                stop_launched_demo(child, resolved_exit_path, poll_ms);
                std::process::exit(1);
            }
        }

        return Ok(bundle_path.map(|p| {
            let run_dir = run_id_artifact_dir(resolved_out_dir, result.run_id);
            let stable = crate::resolve_bundle_artifact_path(&run_dir);
            if stable.is_file() { stable } else { p }
        }));
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
    let last_bundle_dir_name = match result.as_ref() {
        Ok(summary) if summary.stage.as_deref() == Some("failed") => wait_for_failure_dump_bundle(
            resolved_out_dir,
            summary,
            timeout_ms,
            poll_ms,
        )
        .and_then(|dir| dir.file_name().and_then(|s| s.to_str()).map(ToString::to_string)),
        _ => None,
    };
    if let Some(name) = last_bundle_dir_name
        && let Ok(summary) = result.as_mut()
    {
        summary.last_bundle_dir = Some(name);
    }
    let result = match result {
        Ok(v) => v,
        Err(e) => {
            stop_launched_demo(child, resolved_exit_path, poll_ms);
            return Err(e);
        }
    };

    match result.stage.as_deref() {
        Some("passed") => {}
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
            eprintln!("unexpected script stage for {}: {:?}", src.display(), result);
            stop_launched_demo(child, resolved_exit_path, poll_ms);
            std::process::exit(1);
        }
    }

    let bundle_dir = result
        .last_bundle_dir
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from);

    Ok(match bundle_dir {
        Some(bundle_dir) => Some(resolve_bundle_artifact_path(&resolved_out_dir.join(bundle_dir))),
        None => crate::latest::latest_bundle_dir_path_opt(resolved_out_dir)
            .map(|path| resolve_bundle_artifact_path(path.as_path())),
    })
}
