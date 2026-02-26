use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn run_perf_script_and_resolve_bundle_artifact_path(
    src: &PathBuf,
    script_key: &str,
    child: &mut Option<LaunchedDemo>,
    use_devtools_ws: bool,
    connected_ws: Option<&ConnectedToolingTransport>,
    connected_fs: Option<&ConnectedToolingTransport>,
    resolved_out_dir: &Path,
    resolved_exit_path: &Path,
    resolved_script_result_path: &Path,
    perf_capabilities_check_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<Option<PathBuf>, String> {
    let connected = if use_devtools_ws {
        connected_ws
            .ok_or_else(|| "missing DevTools WS transport (this is a tooling bug)".to_string())?
    } else {
        connected_fs.ok_or_else(|| {
            "missing filesystem transport (this is a tooling bug; expected connect_filesystem_tooling)".to_string()
        })?
    };

    let script_value: serde_json::Value =
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
    let script_json =
        crate::script_tooling::resolve_script_json_redirects_from_value(src, script_value)
            .inspect_err(|err| {
                write_tooling_failure_script_result(
                    resolved_script_result_path,
                    "tooling.script.redirect_failed",
                    err,
                    "tooling_error",
                    Some(script_key.to_string()),
                );
            })?
            .value;
    let (mut script_json, upgraded) =
        crate::script_tooling::upgrade_script_json_value_to_v2_if_needed(script_json).inspect_err(
            |err| {
                write_tooling_failure_script_result(
                    resolved_script_result_path,
                    "tooling.script.upgrade_failed",
                    err,
                    "tooling_error",
                    Some(script_key.to_string()),
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
            eprintln!(
                "unexpected script stage for {}: {:?}",
                src.display(),
                result
            );
            stop_launched_demo(child, resolved_exit_path, poll_ms);
            std::process::exit(1);
        }
    }

    Ok(bundle_path.map(|p| {
        let run_dir = run_id_artifact_dir(resolved_out_dir, result.run_id);
        let stable = crate::resolve_bundle_artifact_path(&run_dir);
        if stable.is_file() { stable } else { p }
    }))
}
