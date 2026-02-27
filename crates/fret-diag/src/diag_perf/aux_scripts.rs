use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_suite_aux_script_must_pass(
    src: &PathBuf,
    tool_launched: bool,
    child: &mut Option<LaunchedDemo>,
    use_devtools_ws: bool,
    connected_ws: Option<&ConnectedToolingTransport>,
    connected_fs: Option<&ConnectedToolingTransport>,
    workspace_root: &Path,
    resolved_out_dir: &Path,
    resolved_exit_path: &Path,
    stop_demo_on_fail: bool,
    reuse_process: bool,
    resolved_script_result_path: &Path,
    resolved_script_result_trigger_path: &Path,
    perf_capabilities_check_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(), String> {
    let connected = if use_devtools_ws {
        connected_ws
            .ok_or_else(|| "missing DevTools WS transport (this is a tooling bug)".to_string())?
    } else {
        connected_fs.ok_or_else(|| {
            "missing filesystem transport (this is a tooling bug; expected connect_filesystem_tooling)".to_string()
        })?
    };

    if !reuse_process && !use_devtools_ws {
        // Avoid stale run_id baselines when perf launches a fresh process per run.
        clear_script_result_files(
            resolved_script_result_path,
            resolved_script_result_trigger_path,
        );
    }

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
    let schema_version = crate::compat::script::script_schema_version_from_value(&script_json);
    if schema_version == 1 && tool_launched {
        let msg = format!(
            "script schema_version=1 is disabled for tool-launched runs (--launch/--reuse-launch); upgrade to schema_version=2 (tip: fretboard diag script upgrade --write {})",
            src.display()
        );
        write_tooling_failure_script_result(
            resolved_script_result_path,
            "script.schema_v1_disabled",
            &msg,
            "tooling_error",
            Some(script_key.clone()),
        );
        return Err(msg);
    }
    let (mut script_json, upgraded) =
        crate::compat::script::upgrade_script_json_value_to_v2_if_needed(script_json).inspect_err(
            |err| {
                write_tooling_failure_script_result(
                    resolved_script_result_path,
                    "tooling.script.upgrade_failed",
                    err,
                    "tooling_error",
                    Some(script_key.clone()),
                );
            },
        )?;
    crate::script_tooling::canonicalize_json_value(&mut script_json);
    if upgraded {
        eprintln!(
            "warning: script schema_version=1 detected; tooling upgraded to schema_version=2 for execution (source={})",
            src.display()
        );
    }

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
        fret_diag_protocol::UiScriptStageV1::Passed => Ok(()),
        fret_diag_protocol::UiScriptStageV1::Failed => {
            let msg = format!(
                "aux script failed: {} (run_id={}) step={} reason={} last_bundle_dir={}",
                src.display(),
                result.run_id,
                result.step_index.unwrap_or(0),
                result.reason.as_deref().unwrap_or("unknown"),
                result.last_bundle_dir.as_deref().unwrap_or("")
            );
            eprintln!("FAIL {msg}");
            if stop_demo_on_fail {
                stop_launched_demo(child, resolved_exit_path, poll_ms);
            }
            Err(msg)
        }
        _ => {
            let msg = format!(
                "unexpected aux script stage for {}: {:?}",
                src.display(),
                result
            );
            eprintln!("{msg}");
            if stop_demo_on_fail {
                stop_launched_demo(child, resolved_exit_path, poll_ms);
            }
            Err(msg)
        }
    }
}
