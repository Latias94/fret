use super::*;

fn launched_demo_was_killed(footprint: &serde_json::Value) -> bool {
    footprint
        .get("killed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn launch_command_timeout_ms(launch: &Option<Vec<String>>, timeout_ms: u64) -> u64 {
    let min_timeout_ms = launch
        .as_ref()
        .and_then(|cmd| cmd.first())
        .map(|exe| exe.to_ascii_lowercase())
        .map(|exe_lower| {
            if exe_lower == "cargo"
                || exe_lower.ends_with("\\cargo.exe")
                || exe_lower.ends_with("/cargo.exe")
            {
                600_000
            } else {
                30_000
            }
        })
        .unwrap_or(30_000);
    timeout_ms.max(min_timeout_ms)
}

fn wait_for_launched_demo_exit_without_signal(
    child: &mut Option<LaunchedDemo>,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(), String> {
    let Some(demo) = child.as_mut() else {
        return Ok(());
    };

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(1));
    loop {
        match demo.child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {}
            Err(err) => {
                return Err(format!(
                    "failed to query launched demo status while waiting for clean self-exit: {err}"
                ));
            }
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for launched demo to exit on its own (timeout_ms={timeout_ms})"
            ));
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
}

fn write_tool_owned_script_result(
    out_dir: &Path,
    script_result_path: &Path,
    result: &crate::stats::ScriptResultSummary,
    kind: &str,
    note: Option<String>,
) -> Result<(), String> {
    let now = now_unix_ms();
    let stage = match result.stage.as_deref() {
        Some("passed") => UiScriptStageV1::Passed,
        Some("queued") => UiScriptStageV1::Queued,
        Some("running") => UiScriptStageV1::Running,
        _ => UiScriptStageV1::Failed,
    };
    let step_index = result
        .step_index
        .and_then(|value| u32::try_from(value).ok());
    let script_result = UiScriptResultV1 {
        schema_version: 1,
        run_id: result.run_id,
        updated_unix_ms: now,
        window: None,
        stage,
        step_index,
        reason_code: result.reason_code.clone(),
        reason: result.reason.clone(),
        evidence: Some(UiScriptEvidenceV1 {
            event_log: vec![UiScriptEventLogEntryV1 {
                unix_ms: now,
                kind: kind.to_string(),
                step_index,
                note,
                bundle_dir: result.last_bundle_dir.clone(),
                window: None,
                tick_id: None,
                frame_id: None,
                window_snapshot_seq: None,
            }],
            ..UiScriptEvidenceV1::default()
        }),
        last_bundle_dir: result.last_bundle_dir.clone(),
        last_bundle_artifact: None,
    };

    let value = serde_json::to_value(&script_result).map_err(|err| err.to_string())?;
    write_json_value(script_result_path, &value)?;
    RunArtifactStore::new(out_dir, script_result.run_id).write_script_result(&script_result);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct ExternalNoDiagnosticsPostRunContext<'a> {
    pub src: &'a Path,
    pub launch: &'a Option<Vec<String>>,
    pub launch_env: &'a [(String, String)],
    pub workspace_root: &'a Path,
    pub resolved_out_dir: &'a Path,
    pub resolved_exit_path: &'a Path,
    pub resolved_script_path: &'a Path,
    pub resolved_script_result_path: &'a Path,
    pub timeout_ms: u64,
    pub poll_ms: u64,
    pub launch_high_priority: bool,
    pub warmup_frames: u64,
    pub checks_for_post_run: &'a RunChecks,
    pub tooling_event_kind: &'a str,
    pub tooling_event_note: Option<String>,
}

pub(crate) fn run_external_no_diagnostics_post_run(
    ctx: ExternalNoDiagnosticsPostRunContext<'_>,
) -> Result<crate::stats::ScriptResultSummary, String> {
    let ExternalNoDiagnosticsPostRunContext {
        src,
        launch,
        launch_env,
        workspace_root,
        resolved_out_dir,
        resolved_exit_path,
        resolved_script_path,
        resolved_script_result_path,
        timeout_ms,
        poll_ms,
        launch_high_priority,
        warmup_frames,
        checks_for_post_run,
        tooling_event_kind,
        tooling_event_note,
    } = ctx;

    std::fs::create_dir_all(resolved_out_dir).map_err(|e| e.to_string())?;
    let (script_json, upgraded) = crate::script_execution::load_script_json_for_execution(
        src,
        crate::script_execution::ScriptLoadPolicy {
            tool_launched: true,
            write_failure: write_tooling_failure_script_result_if_missing,
            failure_note: Some(
                "external no-diagnostics post-run path still validates the script envelope"
                    .to_string(),
            ),
            include_stage_in_note: true,
        },
        resolved_script_result_path,
    )?;
    if upgraded {
        eprintln!(
            "warning: script schema_version=1 detected; tooling upgraded to schema_version=2 for execution (source={})",
            src.display()
        );
    }
    let _ = write_json_value(resolved_script_path, &script_json);

    let mut run_launch_env = launch_env.to_vec();
    let internal_report_path = resolved_out_dir.join("hello_world_compare.internal_gpu.json");
    let internal_report_path_value = internal_report_path.display().to_string();
    let _ = ensure_env_var(
        &mut run_launch_env,
        "FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH",
        internal_report_path_value.as_str(),
    );

    let mut child = maybe_launch_demo_without_diagnostics(
        launch,
        &run_launch_env,
        workspace_root,
        resolved_out_dir,
        poll_ms,
        launch_high_priority,
    )
    .inspect_err(|err| {
        write_tooling_failure_script_result_if_missing(
            resolved_script_result_path,
            "tooling.launch.failed",
            err,
            "tooling_error",
            Some("maybe_launch_demo_without_diagnostics".to_string()),
        );
    })?;
    let run_id = child
        .as_ref()
        .map(|demo| demo.launched_unix_ms)
        .unwrap_or_else(now_unix_ms);
    let wait_result = wait_for_launched_demo_exit_without_signal(
        &mut child,
        launch_command_timeout_ms(launch, timeout_ms),
        poll_ms,
    );
    let footprint = stop_launched_demo(&mut child, resolved_exit_path, poll_ms);

    let mut result = crate::stats::ScriptResultSummary {
        run_id,
        stage: Some("passed".to_string()),
        step_index: None,
        reason_code: None,
        reason: None,
        last_bundle_dir: None,
    };

    if let Err(err) = wait_result {
        result.stage = Some("failed".to_string());
        result.reason_code = Some("tooling.external_no_diagnostics.timeout".to_string());
        result.reason = Some(err);
    } else if footprint.as_ref().is_some_and(launched_demo_was_killed) {
        result.stage = Some("failed".to_string());
        result.reason_code = Some("tooling.demo_exit.killed".to_string());
        result.reason = Some(
            "tool-launched demo did not exit cleanly (killed=true in resource.footprint.json)"
                .to_string(),
        );
    } else if let Err(err) =
        apply_post_run_checks(None, resolved_out_dir, checks_for_post_run, warmup_frames)
    {
        result.stage = Some("failed".to_string());
        result.reason_code = Some("tooling.post_run_checks.failed".to_string());
        result.reason = Some(err);
    }

    if let Err(err) = write_tool_owned_script_result(
        resolved_out_dir,
        resolved_script_result_path,
        &result,
        tooling_event_kind,
        tooling_event_note,
    ) {
        eprintln!(
            "WARN: failed to write tool-owned script.result.json for external no-diagnostics path: {err}"
        );
    }

    Ok(result)
}

fn resolve_run_script_source(workspace_root: &Path, raw: &str) -> Result<PathBuf, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("missing script path or script id".to_string());
    }

    let candidate = resolve_path(workspace_root, PathBuf::from(raw));
    if candidate.is_file() {
        return Ok(candidate);
    }

    // A common footgun is putting flags before the required script input.
    if raw.starts_with('-') {
        return Err(format!(
            "missing script path or script id (first argument must be a script; got `{raw}`)\n\
hint: try `fretboard diag run <script.json|script_id> ...`"
        ));
    }

    // If the user passed an explicit path (contains a separator) but it doesn't exist, treat it
    // as a path typo rather than an id.
    if raw.contains('/') || raw.contains('\\') {
        return Err(format!(
            "script file not found: {}\n\
hint: if you meant a promoted script id, pass the id without any path separators (see tools/diag-scripts/index.json)",
            candidate.display()
        ));
    }

    let registry_path = crate::script_registry::promoted_registry_default_path(workspace_root);
    if !registry_path.is_file() {
        return Err(format!(
            "script file not found: {}\n\
and promoted scripts registry is missing: {}\n\
hint: pass an explicit path, or ensure `tools/diag-scripts/index.json` exists (run `cargo run -p fretboard -- diag registry write`)",
            candidate.display(),
            registry_path.display()
        ));
    }

    let registry = crate::script_registry::PromotedScriptRegistry::load_from_path(&registry_path)?;
    let query = crate::script_registry::normalize_script_id_query(raw);

    if let Some(entry) = registry.resolve_id(&query) {
        let resolved = resolve_path(workspace_root, PathBuf::from(&entry.path));
        if resolved.is_file() {
            return Ok(resolved);
        }
        return Err(format!(
            "promoted script id resolved to a missing path: {query}\n\
path: {}\n\
hint: regenerate tools/diag-scripts/index.json (cargo run -p fretboard -- diag registry write)",
            resolved.display()
        ));
    }

    let suggestions = registry.suggest_ids(&query, 5);
    if suggestions.is_empty() {
        return Err(format!(
            "unknown script id (and no file exists at workspace root): {query}\n\
hint: use an explicit path like `tools/diag-scripts/.../script.json`"
        ));
    }

    let mut msg = format!(
        "unknown script id (and no file exists at workspace root): {query}\n\
hint: try one of these promoted ids:"
    );
    for s in suggestions {
        msg.push_str(&format!("\n- {s}"));
    }
    Err(msg)
}

fn run_result_stage_name(stage: fret_diag_protocol::UiScriptStageV1) -> &'static str {
    match stage {
        fret_diag_protocol::UiScriptStageV1::Passed => "passed",
        fret_diag_protocol::UiScriptStageV1::Failed => "failed",
        fret_diag_protocol::UiScriptStageV1::Queued => "queued",
        fret_diag_protocol::UiScriptStageV1::Running => "running",
    }
}

fn build_run_script_result_summary(
    result: &UiScriptResultV1,
    bundle_path: Option<&Path>,
) -> crate::stats::ScriptResultSummary {
    let mut summary = crate::stats::ScriptResultSummary {
        run_id: result.run_id,
        stage: Some(run_result_stage_name(result.stage.clone()).to_string()),
        step_index: result.step_index.map(|value| value as u64),
        reason_code: result.reason_code.clone(),
        reason: result.reason.clone(),
        last_bundle_dir: result.last_bundle_dir.clone(),
    };

    if summary
        .last_bundle_dir
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
        && let Some(bundle_path) = bundle_path
    {
        summary.last_bundle_dir = bundle_path
            .parent()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(|name| name.to_string());
    }

    summary
}

struct RunBundleDoctorAndChecksRequest<'a> {
    bundle_path: Option<&'a Path>,
    resolved_out_dir: &'a Path,
    checks_for_post_run: &'a RunChecks,
    wants_post_run_checks: bool,
    bundle_doctor_mode: BundleDoctorMode,
    warmup_frames: u64,
    bundle_doctor_ran: bool,
    missing_bundle_error: &'a str,
}

fn maybe_run_bundle_doctor_and_checks(
    request: RunBundleDoctorAndChecksRequest<'_>,
) -> Result<bool, String> {
    let mut bundle_doctor_ran = request.bundle_doctor_ran;

    if request.wants_post_run_checks {
        let Some(bundle_path) = request.bundle_path else {
            return Err(request.missing_bundle_error.to_string());
        };

        if !bundle_doctor_ran && request.bundle_doctor_mode != BundleDoctorMode::Off {
            run_bundle_doctor_for_bundle_path(
                bundle_path,
                request.bundle_doctor_mode,
                request.warmup_frames,
            )?;
            bundle_doctor_ran = true;
        }

        apply_post_run_checks(
            Some(bundle_path),
            request.resolved_out_dir,
            request.checks_for_post_run,
            request.warmup_frames,
        )?;
        return Ok(bundle_doctor_ran);
    }

    if !bundle_doctor_ran
        && request.bundle_doctor_mode != BundleDoctorMode::Off
        && let Some(bundle_path) = request.bundle_path
    {
        run_bundle_doctor_for_bundle_path(
            bundle_path,
            request.bundle_doctor_mode,
            request.warmup_frames,
        )?;
        bundle_doctor_ran = true;
    }

    Ok(bundle_doctor_ran)
}

struct RunBundleArtifactsRequest<'a> {
    bundle_path: &'a Path,
    resolved_out_dir: &'a Path,
    workspace_root: &'a Path,
    pack_out: Option<&'a PathBuf>,
    pack_defaults: (bool, bool, bool),
    pack_schema2_only: bool,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    warmup_frames: u64,
    ensure_ai_packet: bool,
    wants_pack_zip: bool,
}

fn emit_run_bundle_artifacts(request: RunBundleArtifactsRequest<'_>) -> Result<(), String> {
    let bundle_dir = resolve_bundle_root_dir(request.bundle_path)?;

    if request.ensure_ai_packet {
        let packet_dir = bundle_dir.join("ai.packet");
        match crate::commands::ai_packet::ensure_ai_packet_dir_best_effort(
            Some(request.bundle_path),
            &bundle_dir,
            &packet_dir,
            request.pack_defaults.1,
            request.stats_top,
            request.sort_override,
            request.warmup_frames,
            None,
        ) {
            Ok(()) => println!("AI-PACKET {}", packet_dir.display()),
            Err(err) => eprintln!("AI-PACKET-ERROR {err}"),
        }
    }

    if request.wants_pack_zip {
        let out = request
            .pack_out
            .cloned()
            .map(|path| resolve_path(request.workspace_root, path))
            .unwrap_or_else(|| default_pack_out_path(request.resolved_out_dir, &bundle_dir));

        let artifacts_root = if bundle_dir.starts_with(request.resolved_out_dir) {
            request.resolved_out_dir.to_path_buf()
        } else {
            bundle_dir
                .parent()
                .unwrap_or(request.resolved_out_dir)
                .to_path_buf()
        };

        if let Err(err) = pack_bundle_dir_to_zip(
            &bundle_dir,
            &out,
            request.pack_defaults.0,
            request.pack_defaults.1,
            request.pack_defaults.2,
            request.pack_schema2_only,
            false,
            false,
            &artifacts_root,
            request.stats_top,
            request
                .sort_override
                .unwrap_or(BundleStatsSort::Invalidation),
            request.warmup_frames,
        ) {
            eprintln!("PACK-ERROR {err}");
        } else {
            println!("PACK {}", out.display());
        }
    }

    Ok(())
}

fn maybe_mark_run_demo_exit_killed(
    result: &mut crate::stats::ScriptResultSummary,
    keep_open: bool,
    resolved_out_dir: &Path,
    resolved_script_result_path: &Path,
) {
    if keep_open {
        return;
    }

    let footprint_path = resolved_out_dir.join("resource.footprint.json");
    if let Ok(bytes) = std::fs::read(&footprint_path)
        && let Ok(footprint) = serde_json::from_slice::<serde_json::Value>(&bytes)
        && launched_demo_was_killed(&footprint)
    {
        crate::tooling_failures::mark_existing_script_result_tooling_failure(
            resolved_out_dir,
            resolved_script_result_path,
            "tooling.demo_exit.killed",
            "tool-launched demo did not exit cleanly (killed=true in resource.footprint.json)",
            "tooling_error",
            Some("stop_launched_demo".to_string()),
        );
        result.stage = Some("failed".to_string());
        result.reason_code = Some("tooling.demo_exit.killed".to_string());
        result.reason = Some(
            "tool-launched demo did not exit cleanly (killed=true in resource.footprint.json)"
                .to_string(),
        );
    }
}

fn maybe_fill_run_failure_dump_bundle_dir(
    result: &mut crate::stats::ScriptResultSummary,
    resolved_out_dir: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) {
    if result.stage.as_deref() == Some("failed")
        && let Some(dir) =
            wait_for_failure_dump_bundle(resolved_out_dir, result, timeout_ms, poll_ms)
        && let Some(name) = dir.file_name().and_then(|value| value.to_str())
    {
        result.last_bundle_dir = Some(name.to_string());
    }
}

struct RunBundlePathResolutionRequest<'a> {
    result: &'a crate::stats::ScriptResultSummary,
    resolved_out_dir: &'a Path,
    resolved_trigger_path: Option<&'a Path>,
    timeout_ms: u64,
    poll_ms: u64,
}

fn resolve_run_bundle_artifact_path(
    request: RunBundlePathResolutionRequest<'_>,
) -> Option<PathBuf> {
    let mut bundle_path = wait_for_bundle_artifact_from_script_result(
        request.resolved_out_dir,
        request.result,
        request.timeout_ms,
        request.poll_ms,
    );
    if bundle_path.is_none()
        && let Some(trigger_path) = request.resolved_trigger_path
    {
        let _ = touch(trigger_path);
        bundle_path = wait_for_bundle_artifact_from_script_result(
            request.resolved_out_dir,
            request.result,
            request.timeout_ms,
            request.poll_ms,
        );
    }
    bundle_path
}

struct RunFilesystemPostRunRequest<'a> {
    result: &'a crate::stats::ScriptResultSummary,
    resolved_out_dir: &'a Path,
    resolved_trigger_path: &'a Path,
    checks_for_post_run: &'a RunChecks,
    wants_post_run_checks: bool,
    wants_post_run_bundle: bool,
    bundle_doctor_mode: BundleDoctorMode,
    warmup_frames: u64,
    timeout_ms: u64,
    poll_ms: u64,
    workspace_root: &'a Path,
    pack_out: Option<&'a PathBuf>,
    pack_defaults: (bool, bool, bool),
    pack_schema2_only: bool,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    ensure_ai_packet: bool,
    wants_pack_zip: bool,
}

fn finalize_run_filesystem_post_run(
    request: RunFilesystemPostRunRequest<'_>,
) -> Result<(), String> {
    let mut bundle_doctor_ran = false;

    if request.result.stage.as_deref() == Some("passed") && request.wants_post_run_checks {
        let bundle_path = resolve_run_bundle_artifact_path(RunBundlePathResolutionRequest {
            result: request.result,
            resolved_out_dir: request.resolved_out_dir,
            resolved_trigger_path: None,
            timeout_ms: request.timeout_ms,
            poll_ms: request.poll_ms,
        })
        .ok_or_else(|| {
            "script passed but no bundle artifact was found (required for post-run checks)"
                .to_string()
        })?;

        bundle_doctor_ran = maybe_run_bundle_doctor_and_checks(RunBundleDoctorAndChecksRequest {
            bundle_path: Some(&bundle_path),
            resolved_out_dir: request.resolved_out_dir,
            checks_for_post_run: request.checks_for_post_run,
            wants_post_run_checks: true,
            bundle_doctor_mode: request.bundle_doctor_mode,
            warmup_frames: request.warmup_frames,
            bundle_doctor_ran,
            missing_bundle_error: "script passed but no bundle artifact was found (required for post-run checks)",
        })?;
    }

    if request.wants_post_run_bundle {
        let bundle_path = resolve_run_bundle_artifact_path(RunBundlePathResolutionRequest {
            result: request.result,
            resolved_out_dir: request.resolved_out_dir,
            resolved_trigger_path: Some(request.resolved_trigger_path),
            timeout_ms: request.timeout_ms,
            poll_ms: request.poll_ms,
        });

        if let Some(bundle_path) = bundle_path {
            let _ = maybe_run_bundle_doctor_and_checks(RunBundleDoctorAndChecksRequest {
                bundle_path: Some(&bundle_path),
                resolved_out_dir: request.resolved_out_dir,
                checks_for_post_run: request.checks_for_post_run,
                wants_post_run_checks: false,
                bundle_doctor_mode: request.bundle_doctor_mode,
                warmup_frames: request.warmup_frames,
                bundle_doctor_ran,
                missing_bundle_error: "",
            })?;
            emit_run_bundle_artifacts(RunBundleArtifactsRequest {
                bundle_path: &bundle_path,
                resolved_out_dir: request.resolved_out_dir,
                workspace_root: request.workspace_root,
                pack_out: request.pack_out,
                pack_defaults: request.pack_defaults,
                pack_schema2_only: request.pack_schema2_only,
                stats_top: request.stats_top,
                sort_override: request.sort_override,
                warmup_frames: request.warmup_frames,
                ensure_ai_packet: request.ensure_ai_packet,
                wants_pack_zip: request.wants_pack_zip,
            })?;
        } else {
            eprintln!(
                "POST-RUN-ERROR no bundle artifact found (add `capture_bundle` or enable script auto-dumps)"
            );
        }
    }

    Ok(())
}

struct RunDevtoolsBranchRequest<'a> {
    src: &'a Path,
    devtools_ws_url: Option<&'a str>,
    devtools_token: Option<&'a str>,
    devtools_session_id: Option<&'a str>,
    resolved_out_dir: &'a Path,
    resolved_script_path: &'a Path,
    resolved_script_result_path: &'a Path,
    wants_bundle_artifact: bool,
    trace_chrome: bool,
    timeout_ms: u64,
    poll_ms: u64,
    exit_after_run: bool,
    checks_for_post_run: &'a RunChecks,
    wants_post_run_checks: bool,
    bundle_doctor_mode: BundleDoctorMode,
    warmup_frames: u64,
    workspace_root: &'a Path,
    pack_out: Option<&'a PathBuf>,
    pack_defaults: (bool, bool, bool),
    pack_schema2_only: bool,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    ensure_ai_packet: bool,
    wants_pack_zip: bool,
}

fn run_cmd_run_devtools_branch(request: RunDevtoolsBranchRequest<'_>) -> Result<(), String> {
    let ws_url = request.devtools_ws_url.ok_or_else(|| {
        "missing --devtools-ws-url (required when using DevTools WS transport)".to_string()
    })?;
    let token = request.devtools_token.ok_or_else(|| {
        "missing --devtools-token (required when using DevTools WS transport)".to_string()
    })?;

    std::fs::create_dir_all(request.resolved_out_dir).map_err(|err| err.to_string())?;
    let (script_json, upgraded) = crate::script_execution::load_script_json_for_execution(
        request.src,
        crate::script_execution::ScriptLoadPolicy {
            tool_launched: false,
            write_failure: write_tooling_failure_script_result_if_missing,
            failure_note: None,
            include_stage_in_note: true,
        },
        request.resolved_script_result_path,
    )?;
    if upgraded {
        eprintln!(
            "warning: script schema_version=1 detected; tooling upgraded to schema_version=2 for execution (source={})",
            request.src.display()
        );
    }

    let _ = write_json_value(request.resolved_script_path, &script_json);

    let connected = connect_devtools_ws_tooling(
        ws_url,
        token,
        request.devtools_session_id,
        request.timeout_ms,
        request.poll_ms,
    )
    .inspect_err(|err| {
        write_tooling_failure_script_result_if_missing(
            request.resolved_script_result_path,
            "tooling.connect.failed",
            err,
            "tooling_error",
            Some("connect_devtools_ws_tooling".to_string()),
        );
    })?;

    let (result, bundle_path) = run_script_over_transport(
        request.resolved_out_dir,
        &connected,
        script_json,
        request.wants_bundle_artifact,
        request.trace_chrome,
        Some("diag-run"),
        None,
        request.timeout_ms,
        request.poll_ms,
        request.resolved_script_result_path,
        &request.resolved_out_dir.join("check.capabilities.json"),
    )
    .inspect_err(|err| {
        write_tooling_failure_script_result_if_missing(
            request.resolved_script_result_path,
            "tooling.run.failed",
            err,
            "tooling_error",
            Some("run_script_over_transport".to_string()),
        );
    })?;

    if request.exit_after_run {
        connected
            .devtools
            .app_exit_request(None, Some("diag.run"), None);
    }

    let summary = build_run_script_result_summary(&result, bundle_path.as_deref());

    let _ = maybe_run_bundle_doctor_and_checks(RunBundleDoctorAndChecksRequest {
        bundle_path: bundle_path.as_deref(),
        resolved_out_dir: request.resolved_out_dir,
        checks_for_post_run: request.checks_for_post_run,
        wants_post_run_checks: request.wants_post_run_checks
            && matches!(result.stage, fret_diag_protocol::UiScriptStageV1::Passed),
        bundle_doctor_mode: request.bundle_doctor_mode,
        warmup_frames: request.warmup_frames,
        bundle_doctor_ran: false,
        missing_bundle_error: "script passed but no bundle artifact was captured (required for post-run checks)",
    })?;

    if let Some(bundle_path) = bundle_path.as_deref() {
        emit_run_bundle_artifacts(RunBundleArtifactsRequest {
            bundle_path,
            resolved_out_dir: request.resolved_out_dir,
            workspace_root: request.workspace_root,
            pack_out: request.pack_out,
            pack_defaults: request.pack_defaults,
            pack_schema2_only: request.pack_schema2_only,
            stats_top: request.stats_top,
            sort_override: request.sort_override,
            warmup_frames: request.warmup_frames,
            ensure_ai_packet: request.ensure_ai_packet,
            wants_pack_zip: request.wants_pack_zip,
        })?;
    } else {
        if request.ensure_ai_packet {
            eprintln!(
                "AI-PACKET-ERROR no bundle artifact captured over DevTools WS (ensure bundles are embedded or the runtime bundle dir is accessible)"
            );
        }
        if request.wants_pack_zip {
            eprintln!(
                "PACK-ERROR no bundle artifact captured over DevTools WS (ensure bundles are embedded or the runtime bundle dir is accessible)"
            );
        }
    }

    report_result_and_exit(&summary);
}

fn run_push_env_if_missing(env: &mut Vec<(String, String)>, key: &str, value: &str) {
    if env.iter().any(|(existing_key, _value)| existing_key == key) {
        return;
    }
    env.push((key.to_string(), value.to_string()));
}

fn build_run_launch_env(
    base_launch_env: &[(String, String)],
    src: &Path,
    checks_for_post_run: &RunChecks,
) -> Vec<(String, String)> {
    let mut run_launch_env = base_launch_env.to_vec();
    run_push_env_if_missing(&mut run_launch_env, "FRET_DIAG_REDACT_TEXT", "0");
    for (key, value) in script_env_defaults(src) {
        run_push_env_if_missing(&mut run_launch_env, &key, &value);
    }
    let _ = ensure_env_var(&mut run_launch_env, "FRET_DIAG_RENDERER_PERF", "1");
    if checks_for_post_run
        .check_view_cache_reuse_min
        .is_some_and(|value| value > 0)
        || checks_for_post_run
            .check_view_cache_reuse_stable_min
            .is_some_and(|value| value > 0)
    {
        let _ = ensure_env_var(&mut run_launch_env, "FRET_UI_DEBUG_STATS", "1");
    }
    run_launch_env
}

struct StopLaunchedDemoOnDrop<'a> {
    child: &'a mut Option<LaunchedDemo>,
    exit_path: &'a Path,
    poll_ms: u64,
}

impl Drop for StopLaunchedDemoOnDrop<'_> {
    fn drop(&mut self) {
        let _ = stop_launched_demo(self.child, self.exit_path, self.poll_ms);
    }
}

struct RunFilesystemBranchRequest<'a> {
    src: &'a Path,
    launch: &'a Option<Vec<String>>,
    launch_env: &'a [(String, String)],
    reuse_launch: bool,
    launch_high_priority: bool,
    launch_write_bundle_json: bool,
    keep_open: bool,
    workspace_root: &'a Path,
    resolved_out_dir: &'a Path,
    resolved_ready_path: &'a Path,
    resolved_exit_path: &'a Path,
    resolved_trigger_path: &'a Path,
    resolved_script_result_path: &'a Path,
    fs_transport_cfg: &'a crate::transport::FsDiagTransportConfig,
    checks_for_post_run: &'a RunChecks,
    pack_defaults: (bool, bool, bool),
    wants_bundle_artifact: bool,
    wants_post_run_checks: bool,
    wants_post_run_bundle: bool,
    trace_chrome: bool,
    timeout_ms: u64,
    poll_ms: u64,
    exit_after_run: bool,
    bundle_doctor_mode: BundleDoctorMode,
    warmup_frames: u64,
    pack_out: Option<&'a PathBuf>,
    pack_schema2_only: bool,
    stats_top: usize,
    sort_override: Option<BundleStatsSort>,
    ensure_ai_packet: bool,
    wants_pack_zip: bool,
}

fn run_cmd_run_filesystem_branch(request: RunFilesystemBranchRequest<'_>) -> Result<(), String> {
    let script_wants_screenshots = script_requests_screenshots(request.src);
    let run_launch_env =
        build_run_launch_env(request.launch_env, request.src, request.checks_for_post_run);

    let mut child = maybe_launch_demo(
        request.launch,
        &run_launch_env,
        request.workspace_root,
        request.resolved_ready_path,
        request.resolved_exit_path,
        request.fs_transport_cfg,
        request.pack_defaults.2
            || crate::registry::checks::CheckRegistry::builtin()
                .wants_screenshots(request.checks_for_post_run)
            || script_wants_screenshots,
        request.launch_write_bundle_json,
        request.timeout_ms,
        request.poll_ms,
        request.launch_high_priority,
    )
    .inspect_err(|err| {
        write_tooling_failure_script_result_if_missing(
            request.resolved_script_result_path,
            "tooling.launch.failed",
            err,
            "tooling_error",
            Some("maybe_launch_demo".to_string()),
        );
    })?;
    let stop_guard = if request.keep_open {
        None
    } else {
        Some(StopLaunchedDemoOnDrop {
            child: &mut child,
            exit_path: request.resolved_exit_path,
            poll_ms: request.poll_ms,
        })
    };

    let connected = connect_filesystem_tooling(
        request.fs_transport_cfg,
        request.resolved_ready_path,
        request.launch.is_some(),
        request.timeout_ms,
        request.poll_ms,
    )
    .inspect_err(|err| {
        write_tooling_failure_script_result_if_missing(
            request.resolved_script_result_path,
            "tooling.connect.failed",
            err,
            "tooling_error",
            Some("connect_filesystem_tooling".to_string()),
        );
    })?;
    let tool_launched = request.launch.is_some() || request.reuse_launch;
    let (script_json, upgraded) = crate::script_execution::load_script_json_for_execution(
        request.src,
        crate::script_execution::ScriptLoadPolicy {
            tool_launched,
            write_failure: write_tooling_failure_script_result_if_missing,
            failure_note: None,
            include_stage_in_note: true,
        },
        request.resolved_script_result_path,
    )?;
    if upgraded {
        eprintln!(
            "warning: script schema_version=1 detected; tooling upgraded to schema_version=2 for execution (source={})",
            request.src.display()
        );
    }
    let (script_result, bundle_path) = run_script_over_transport(
        request.resolved_out_dir,
        &connected,
        script_json,
        request.wants_bundle_artifact,
        request.trace_chrome,
        Some("diag-run"),
        None,
        request.timeout_ms,
        request.poll_ms,
        request.resolved_script_result_path,
        &request.resolved_out_dir.join("check.capabilities.json"),
    )
    .inspect_err(|err| {
        write_tooling_failure_script_result_if_missing(
            request.resolved_script_result_path,
            "tooling.run.failed",
            err,
            "tooling_error",
            Some("run_script_over_transport".to_string()),
        );
    })?;

    let mut result = build_run_script_result_summary(&script_result, bundle_path.as_deref());

    maybe_fill_run_failure_dump_bundle_dir(
        &mut result,
        request.resolved_out_dir,
        request.timeout_ms,
        request.poll_ms,
    );
    if request.exit_after_run {
        let _ = touch(request.resolved_exit_path);
    }

    finalize_run_filesystem_post_run(RunFilesystemPostRunRequest {
        result: &result,
        resolved_out_dir: request.resolved_out_dir,
        resolved_trigger_path: request.resolved_trigger_path,
        checks_for_post_run: request.checks_for_post_run,
        wants_post_run_checks: request.wants_post_run_checks,
        wants_post_run_bundle: request.wants_post_run_bundle,
        bundle_doctor_mode: request.bundle_doctor_mode,
        warmup_frames: request.warmup_frames,
        timeout_ms: request.timeout_ms,
        poll_ms: request.poll_ms,
        workspace_root: request.workspace_root,
        pack_out: request.pack_out,
        pack_defaults: request.pack_defaults,
        pack_schema2_only: request.pack_schema2_only,
        stats_top: request.stats_top,
        sort_override: request.sort_override,
        ensure_ai_packet: request.ensure_ai_packet,
        wants_pack_zip: request.wants_pack_zip,
    })?;

    drop(stop_guard);

    maybe_mark_run_demo_exit_killed(
        &mut result,
        request.keep_open,
        request.resolved_out_dir,
        request.resolved_script_result_path,
    );
    report_result_and_exit(&result);
}

struct RunCommandSetupRequest<'a> {
    rest: &'a [String],
    workspace_root: &'a Path,
    checks: RunChecks,
    pack_after_run: bool,
    pack_out: Option<&'a PathBuf>,
    pack_include_root_artifacts: bool,
    pack_include_triage: bool,
    pack_include_screenshots: bool,
    ensure_ai_packet: bool,
    devtools_ws_url: Option<&'a str>,
    devtools_token: Option<&'a str>,
    devtools_session_id: Option<&'a str>,
    launch_requested: bool,
    reuse_launch: bool,
    keep_open: bool,
    trace_chrome: bool,
    launch_write_bundle_json: bool,
}

struct PreparedRunCommandSetup {
    src: PathBuf,
    checks_for_post_run: RunChecks,
    bundle_doctor_mode: BundleDoctorMode,
    wants_pack_zip: bool,
    wants_post_run_bundle: bool,
    wants_post_run_checks: bool,
    wants_bundle_artifact: bool,
    pack_defaults: (bool, bool, bool),
    use_devtools_ws: bool,
    prefers_external_no_diag_post_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreparedRunExecutionRoute {
    DevtoolsWs,
    ExternalNoDiagnostics,
    Filesystem,
}

impl PreparedRunCommandSetup {
    fn execution_route(&self) -> PreparedRunExecutionRoute {
        if self.use_devtools_ws {
            PreparedRunExecutionRoute::DevtoolsWs
        } else if self.prefers_external_no_diag_post_run {
            PreparedRunExecutionRoute::ExternalNoDiagnostics
        } else {
            PreparedRunExecutionRoute::Filesystem
        }
    }
}

fn prepare_cmd_run_setup(
    request: RunCommandSetupRequest<'_>,
) -> Result<PreparedRunCommandSetup, String> {
    let (bundle_doctor_mode, rest) = parse_bundle_doctor_mode_from_rest(request.rest)?;
    let Some(src_raw) = rest.first().cloned() else {
        return Err(
            "missing script path or script id (try: fretboard diag run <script.json|script_id>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = resolve_run_script_source(request.workspace_root, &src_raw)?;
    let mut checks_for_post_run = request.checks;

    let policy_wheel_events_max_per_frame =
        crate::diag_policy::ui_gallery_script_requires_wheel_events_max_per_frame_gate(&src)
            .then_some(1);
    checks_for_post_run.check_wheel_events_max_per_frame = checks_for_post_run
        .check_wheel_events_max_per_frame
        .or(policy_wheel_events_max_per_frame);
    checks_for_post_run.check_hello_world_compare_idle_present_max_delta = checks_for_post_run
        .check_hello_world_compare_idle_present_max_delta
        .or(crate::diag_policy::hello_world_compare_script_idle_present_max_delta(&src));

    let wants_pack_zip = request.pack_after_run
        || request.pack_out.is_some()
        || request.pack_include_root_artifacts
        || request.pack_include_triage
        || request.pack_include_screenshots;
    let wants_post_run_bundle = wants_pack_zip || request.ensure_ai_packet;

    let check_registry = crate::registry::checks::CheckRegistry::builtin();
    let wants_post_run_checks = check_registry.wants_post_run_checks(&checks_for_post_run);
    let wants_bundle_artifact =
        wants_post_run_bundle || check_registry.wants_bundle_artifact(&checks_for_post_run);

    let mut pack_defaults = (
        request.pack_include_root_artifacts,
        request.pack_include_triage,
        request.pack_include_screenshots,
    );
    if request.pack_after_run && !pack_defaults.0 && !pack_defaults.1 && !pack_defaults.2 {
        pack_defaults = (true, true, true);
    }

    let use_devtools_ws = request.devtools_ws_url.is_some()
        || request.devtools_token.is_some()
        || request.devtools_session_id.is_some();
    if use_devtools_ws && (request.launch_requested || request.reuse_launch) {
        return Err("--launch/--reuse-launch is not supported with --devtools-ws-url".to_string());
    }
    let prefers_external_no_diag_post_run =
        crate::diag_policy::hello_world_compare_script_prefers_external_no_diag_post_run(&src)
            && request.launch_requested
            && !request.reuse_launch
            && !request.keep_open
            && !use_devtools_ws
            && !request.trace_chrome
            && !request.launch_write_bundle_json
            && bundle_doctor_mode == BundleDoctorMode::Off
            && !wants_post_run_bundle
            && !check_registry.wants_bundle_artifact(&checks_for_post_run);

    Ok(PreparedRunCommandSetup {
        src,
        checks_for_post_run,
        bundle_doctor_mode,
        wants_pack_zip,
        wants_post_run_bundle,
        wants_post_run_checks,
        wants_bundle_artifact,
        pack_defaults,
        use_devtools_ws,
        prefers_external_no_diag_post_run,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct RunChecks {
    pub check_chart_sampling_window_shifts_min: Option<u64>,
    pub check_dock_drag_min: Option<u64>,
    pub check_drag_cache_root_paint_only_test_id: Option<String>,
    pub check_gc_sweep_liveness: bool,
    pub check_hover_layout_max: Option<u32>,
    pub check_hello_world_compare_idle_present_max_delta: Option<u64>,
    pub check_idle_no_paint_min: Option<u64>,
    pub check_asset_load_missing_bundle_assets_max: Option<u64>,
    pub check_asset_load_unsupported_file_max: Option<u64>,
    pub check_asset_load_unsupported_url_max: Option<u64>,
    pub check_asset_load_external_reference_unavailable_max: Option<u64>,
    pub check_asset_load_revision_changes_max: Option<u64>,
    pub check_bundled_font_baseline_source: Option<String>,
    pub check_layout_fast_path_min: Option<u64>,
    pub check_node_graph_cull_window_shifts_max: Option<u64>,
    pub check_node_graph_cull_window_shifts_min: Option<u64>,
    pub check_notify_hotspot_file_max: Vec<(String, u64)>,
    pub check_triage_hint_absent_codes: Vec<String>,
    pub check_overlay_synthesis_min: Option<u64>,
    pub check_pixels_changed_test_id: Option<String>,
    pub check_pixels_unchanged_test_id: Option<String>,
    pub check_prepaint_actions_min: Option<u64>,
    pub check_retained_vlist_attach_detach_max: Option<u64>,
    pub check_retained_vlist_keep_alive_budget: Option<(u64, u64)>,
    pub check_retained_vlist_keep_alive_reuse_min: Option<u64>,
    pub check_retained_vlist_reconcile_no_notify_min: Option<u64>,
    pub check_semantics_changed_repainted: bool,
    pub check_stale_paint_eps: f32,
    pub check_stale_paint_test_id: Option<String>,
    pub check_stale_scene_eps: f32,
    pub check_stale_scene_test_id: Option<String>,
    pub check_ui_gallery_code_editor_a11y_composition: bool,
    pub check_ui_gallery_code_editor_a11y_composition_drag: bool,
    pub check_ui_gallery_code_editor_a11y_composition_wrap: bool,
    pub check_ui_gallery_code_editor_a11y_composition_wrap_scroll: bool,
    pub check_ui_gallery_code_editor_a11y_selection: bool,
    pub check_ui_gallery_code_editor_a11y_selection_wrap: bool,
    pub check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection: bool,
    pub check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll: bool,
    pub check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed:
        bool,
    pub check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed:
        bool,
    pub check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit: bool,
    pub check_ui_gallery_code_editor_torture_folds_placeholder_present: bool,
    pub check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped:
        bool,
    pub check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations:
        bool,
    pub check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed:
        bool,
    pub check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap: bool,
    pub check_ui_gallery_code_editor_torture_geom_fallbacks_low: bool,
    pub check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit: bool,
    pub check_ui_gallery_code_editor_torture_inlays_present: bool,
    pub check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped: bool,
    pub check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations:
        bool,
    pub check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed:
        bool,
    pub check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap: bool,
    pub check_ui_gallery_code_editor_torture_marker_present: bool,
    pub check_ui_gallery_code_editor_torture_read_only_blocks_edits: bool,
    pub check_ui_gallery_code_editor_torture_undo_redo: bool,
    pub check_ui_gallery_code_editor_word_boundary: bool,
    pub check_ui_gallery_markdown_editor_source_a11y_composition: bool,
    pub check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap: bool,
    pub check_ui_gallery_markdown_editor_source_disabled_blocks_edits: bool,
    pub check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds: bool,
    pub check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit: bool,
    pub check_ui_gallery_markdown_editor_source_folds_placeholder_present: bool,
    pub check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap: bool,
    pub check_ui_gallery_markdown_editor_source_folds_toggle_stable: bool,
    pub check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit: bool,
    pub check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable: bool,
    pub check_ui_gallery_markdown_editor_source_inlays_present: bool,
    pub check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap: bool,
    pub check_ui_gallery_markdown_editor_source_inlays_toggle_stable: bool,
    pub check_ui_gallery_markdown_editor_source_line_boundary_triple_click: bool,
    pub check_ui_gallery_markdown_editor_source_read_only_blocks_edits: bool,
    pub check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable: bool,
    pub check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable: bool,
    pub check_ui_gallery_markdown_editor_source_word_boundary: bool,
    pub check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change: bool,
    pub check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change: bool,
    pub check_ui_gallery_text_mixed_script_bundled_fallback_conformance: bool,
    pub check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps: bool,
    pub check_ui_gallery_web_ime_bridge_enabled: bool,
    pub check_view_cache_reuse_min: Option<u64>,
    pub check_view_cache_reuse_stable_min: Option<u64>,
    pub check_viewport_capture_min: Option<u64>,
    pub check_viewport_input_min: Option<u64>,
    pub check_vlist_policy_key_stable: bool,
    pub check_vlist_visible_range_refreshes_max: Option<u64>,
    pub check_vlist_visible_range_refreshes_min: Option<u64>,
    pub check_vlist_window_shifts_escape_max: Option<u64>,
    pub check_vlist_window_shifts_explainable: bool,
    pub check_vlist_window_shifts_have_prepaint_actions: bool,
    pub check_vlist_window_shifts_non_retained_max: Option<u64>,
    pub check_vlist_window_shifts_prefetch_max: Option<u64>,
    pub check_wheel_events_max_per_frame: Option<u64>,
    pub check_wheel_scroll_hit_changes_test_id: Option<String>,
    pub check_wheel_scroll_test_id: Option<String>,
    pub check_windowed_rows_offset_changes_eps: f32,
    pub check_windowed_rows_offset_changes_min: Option<u64>,
    pub check_windowed_rows_visible_start_changes_repainted: bool,
    pub dump_semantics_changed_repainted_json: bool,
}

impl Default for RunChecks {
    fn default() -> Self {
        Self {
            check_chart_sampling_window_shifts_min: None,
            check_dock_drag_min: None,
            check_drag_cache_root_paint_only_test_id: None,
            check_gc_sweep_liveness: false,
            check_hover_layout_max: None,
            check_hello_world_compare_idle_present_max_delta: None,
            check_idle_no_paint_min: None,
            check_asset_load_missing_bundle_assets_max: None,
            check_asset_load_unsupported_file_max: None,
            check_asset_load_unsupported_url_max: None,
            check_asset_load_external_reference_unavailable_max: None,
            check_asset_load_revision_changes_max: None,
            check_bundled_font_baseline_source: None,
            check_layout_fast_path_min: None,
            check_node_graph_cull_window_shifts_max: None,
            check_node_graph_cull_window_shifts_min: None,
            check_notify_hotspot_file_max: Vec::new(),
            check_triage_hint_absent_codes: Vec::new(),
            check_overlay_synthesis_min: None,
            check_pixels_changed_test_id: None,
            check_pixels_unchanged_test_id: None,
            check_prepaint_actions_min: None,
            check_retained_vlist_attach_detach_max: None,
            check_retained_vlist_keep_alive_budget: None,
            check_retained_vlist_keep_alive_reuse_min: None,
            check_retained_vlist_reconcile_no_notify_min: None,
            check_semantics_changed_repainted: false,
            check_stale_paint_eps: 0.5,
            check_stale_paint_test_id: None,
            check_stale_scene_eps: 0.5,
            check_stale_scene_test_id: None,
            check_ui_gallery_code_editor_a11y_composition: false,
            check_ui_gallery_code_editor_a11y_composition_drag: false,
            check_ui_gallery_code_editor_a11y_composition_wrap: false,
            check_ui_gallery_code_editor_a11y_composition_wrap_scroll: false,
            check_ui_gallery_code_editor_a11y_selection: false,
            check_ui_gallery_code_editor_a11y_selection_wrap: false,
            check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection: false,
            check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll: false,
            check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed:
                false,
            check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed:
                false,
            check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit: false,
            check_ui_gallery_code_editor_torture_folds_placeholder_present: false,
            check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped:
                false,
            check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations:
                false,
            check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed:
                false,
            check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap: false,
            check_ui_gallery_code_editor_torture_geom_fallbacks_low: false,
            check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit: false,
            check_ui_gallery_code_editor_torture_inlays_present: false,
            check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped: false,
            check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations:
                false,
            check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed:
                false,
            check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap: false,
            check_ui_gallery_code_editor_torture_marker_present: false,
            check_ui_gallery_code_editor_torture_read_only_blocks_edits: false,
            check_ui_gallery_code_editor_torture_undo_redo: false,
            check_ui_gallery_code_editor_word_boundary: false,
            check_ui_gallery_markdown_editor_source_a11y_composition: false,
            check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap: false,
            check_ui_gallery_markdown_editor_source_disabled_blocks_edits: false,
            check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds: false,
            check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit: false,
            check_ui_gallery_markdown_editor_source_folds_placeholder_present: false,
            check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap: false,
            check_ui_gallery_markdown_editor_source_folds_toggle_stable: false,
            check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit: false,
            check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable: false,
            check_ui_gallery_markdown_editor_source_inlays_present: false,
            check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap: false,
            check_ui_gallery_markdown_editor_source_inlays_toggle_stable: false,
            check_ui_gallery_markdown_editor_source_line_boundary_triple_click: false,
            check_ui_gallery_markdown_editor_source_read_only_blocks_edits: false,
            check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable: false,
            check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable: false,
            check_ui_gallery_markdown_editor_source_word_boundary: false,
            check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change: false,
            check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change: false,
            check_ui_gallery_text_mixed_script_bundled_fallback_conformance: false,
            check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps: false,
            check_ui_gallery_web_ime_bridge_enabled: false,
            check_view_cache_reuse_min: None,
            check_view_cache_reuse_stable_min: None,
            check_viewport_capture_min: None,
            check_viewport_input_min: None,
            check_vlist_policy_key_stable: false,
            check_vlist_visible_range_refreshes_max: None,
            check_vlist_visible_range_refreshes_min: None,
            check_vlist_window_shifts_escape_max: None,
            check_vlist_window_shifts_explainable: false,
            check_vlist_window_shifts_have_prepaint_actions: false,
            check_vlist_window_shifts_non_retained_max: None,
            check_vlist_window_shifts_prefetch_max: None,
            check_wheel_events_max_per_frame: None,
            check_wheel_scroll_hit_changes_test_id: None,
            check_wheel_scroll_test_id: None,
            check_windowed_rows_offset_changes_eps: 0.5,
            check_windowed_rows_offset_changes_min: None,
            check_windowed_rows_visible_start_changes_repainted: false,
            dump_semantics_changed_repainted_json: false,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RunCmdContext {
    pub pack_after_run: bool,
    pub ensure_ai_packet: bool,
    pub rest: Vec<String>,
    pub workspace_root: PathBuf,
    pub resolved_run_context: ResolvedRunContext,
    pub pack_out: Option<PathBuf>,
    pub pack_include_root_artifacts: bool,
    pub pack_include_triage: bool,
    pub pack_include_screenshots: bool,
    pub pack_schema2_only: bool,
    pub stats_top: usize,
    pub sort_override: Option<BundleStatsSort>,
    pub warmup_frames: u64,
    pub timeout_ms: u64,
    pub poll_ms: u64,
    pub trace_chrome: bool,
    pub devtools_ws_url: Option<String>,
    pub devtools_token: Option<String>,
    pub devtools_session_id: Option<String>,
    pub exit_after_run: bool,
    pub launch: Option<Vec<String>>,
    pub launch_env: Vec<(String, String)>,
    pub reuse_launch: bool,
    pub launch_high_priority: bool,
    pub launch_write_bundle_json: bool,
    pub keep_open: bool,
    pub checks: RunChecks,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_run(ctx: RunCmdContext) -> Result<(), String> {
    let RunCmdContext {
        pack_after_run,
        ensure_ai_packet,
        rest,
        workspace_root,
        resolved_run_context,
        pack_out,
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
        pack_schema2_only,
        stats_top,
        sort_override,
        warmup_frames,
        timeout_ms,
        poll_ms,
        trace_chrome,
        devtools_ws_url,
        devtools_token,
        devtools_session_id,
        exit_after_run,
        launch,
        launch_env,
        reuse_launch,
        launch_high_priority,
        launch_write_bundle_json,
        keep_open,
        checks,
    } = ctx;

    let ResolvedRunContext {
        paths: resolved_paths,
        fs_transport_cfg,
    } = resolved_run_context;

    let resolved_out_dir = resolved_paths.out_dir;
    let resolved_trigger_path = resolved_paths.trigger_path;
    let resolved_ready_path = resolved_paths.ready_path;
    let resolved_exit_path = resolved_paths.exit_path;
    let resolved_script_path = resolved_paths.script_path;
    let resolved_script_result_path = resolved_paths.script_result_path;
    let prepared = prepare_cmd_run_setup(RunCommandSetupRequest {
        rest: &rest,
        workspace_root: &workspace_root,
        checks,
        pack_after_run,
        pack_out: pack_out.as_ref(),
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
        ensure_ai_packet,
        devtools_ws_url: devtools_ws_url.as_deref(),
        devtools_token: devtools_token.as_deref(),
        devtools_session_id: devtools_session_id.as_deref(),
        launch_requested: launch.is_some(),
        reuse_launch,
        keep_open,
        trace_chrome,
        launch_write_bundle_json,
    })?;
    let route = prepared.execution_route();
    let PreparedRunCommandSetup {
        src,
        checks_for_post_run,
        bundle_doctor_mode,
        wants_pack_zip,
        wants_post_run_bundle,
        wants_post_run_checks,
        wants_bundle_artifact,
        pack_defaults,
        use_devtools_ws: _,
        prefers_external_no_diag_post_run: _,
    } = prepared;
    match route {
        PreparedRunExecutionRoute::DevtoolsWs => {
            run_cmd_run_devtools_branch(RunDevtoolsBranchRequest {
                src: &src,
                devtools_ws_url: devtools_ws_url.as_deref(),
                devtools_token: devtools_token.as_deref(),
                devtools_session_id: devtools_session_id.as_deref(),
                resolved_out_dir: &resolved_out_dir,
                resolved_script_path: &resolved_script_path,
                resolved_script_result_path: &resolved_script_result_path,
                wants_bundle_artifact,
                trace_chrome,
                timeout_ms,
                poll_ms,
                exit_after_run,
                checks_for_post_run: &checks_for_post_run,
                wants_post_run_checks,
                bundle_doctor_mode,
                warmup_frames,
                workspace_root: &workspace_root,
                pack_out: pack_out.as_ref(),
                pack_defaults,
                pack_schema2_only,
                stats_top,
                sort_override,
                ensure_ai_packet,
                wants_pack_zip,
            })?;
            return Ok(());
        }
        PreparedRunExecutionRoute::ExternalNoDiagnostics => {
            let mut run_launch_env = launch_env.clone();
            run_push_env_if_missing(&mut run_launch_env, "FRET_DIAG_REDACT_TEXT", "0");
            for (key, value) in script_env_defaults(&src) {
                run_push_env_if_missing(&mut run_launch_env, &key, &value);
            }

            let result =
                run_external_no_diagnostics_post_run(ExternalNoDiagnosticsPostRunContext {
                    src: &src,
                    launch: &launch,
                    launch_env: &run_launch_env,
                    workspace_root: &workspace_root,
                    resolved_out_dir: &resolved_out_dir,
                    resolved_exit_path: &resolved_exit_path,
                    resolved_script_path: &resolved_script_path,
                    resolved_script_result_path: &resolved_script_result_path,
                    timeout_ms,
                    poll_ms,
                    launch_high_priority,
                    warmup_frames,
                    checks_for_post_run: &checks_for_post_run,
                    tooling_event_kind: "tooling_external_no_diagnostics",
                    tooling_event_note: Some(
                        "diag-run external no-diagnostics post-run path".to_string(),
                    ),
                })?;

            report_result_and_exit(&result);
        }
        PreparedRunExecutionRoute::Filesystem => {}
    }

    run_cmd_run_filesystem_branch(RunFilesystemBranchRequest {
        src: &src,
        launch: &launch,
        launch_env: &launch_env,
        reuse_launch,
        launch_high_priority,
        launch_write_bundle_json,
        keep_open,
        workspace_root: &workspace_root,
        resolved_out_dir: &resolved_out_dir,
        resolved_ready_path: &resolved_ready_path,
        resolved_exit_path: &resolved_exit_path,
        resolved_trigger_path: &resolved_trigger_path,
        resolved_script_result_path: &resolved_script_result_path,
        fs_transport_cfg: &fs_transport_cfg,
        checks_for_post_run: &checks_for_post_run,
        pack_defaults,
        wants_bundle_artifact,
        wants_post_run_checks,
        wants_post_run_bundle,
        trace_chrome,
        timeout_ms,
        poll_ms,
        exit_after_run,
        bundle_doctor_mode,
        warmup_frames,
        pack_out: pack_out.as_ref(),
        pack_schema2_only,
        stats_top,
        sort_override,
        ensure_ai_packet,
        wants_pack_zip,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-run-tests-{label}-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp dir");
        root
    }

    #[test]
    fn build_run_script_result_summary_backfills_last_bundle_dir_from_bundle_path() {
        let result = UiScriptResultV1 {
            schema_version: 1,
            run_id: 7,
            updated_unix_ms: 0,
            window: None,
            stage: fret_diag_protocol::UiScriptStageV1::Passed,
            step_index: Some(3),
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        let bundle_path = Path::new("diag-out/sessions/run-7/bundle.json");

        let summary = build_run_script_result_summary(&result, Some(bundle_path));

        assert_eq!(summary.stage.as_deref(), Some("passed"));
        assert_eq!(summary.last_bundle_dir.as_deref(), Some("run-7"));
    }

    #[test]
    fn maybe_run_bundle_doctor_and_checks_requires_bundle_when_checks_are_requested() {
        let err = maybe_run_bundle_doctor_and_checks(RunBundleDoctorAndChecksRequest {
            bundle_path: None,
            resolved_out_dir: Path::new("diag-out"),
            checks_for_post_run: &RunChecks::default(),
            wants_post_run_checks: true,
            bundle_doctor_mode: BundleDoctorMode::Off,
            warmup_frames: 0,
            bundle_doctor_ran: false,
            missing_bundle_error: "bundle missing",
        })
        .expect_err("post-run checks should require a bundle path");

        assert_eq!(err, "bundle missing");
    }

    #[test]
    fn prepare_cmd_run_setup_applies_pack_defaults_and_policy_wheel_gate() {
        let workspace_root = make_temp_dir("prepare-setup-defaults");
        let script_path = workspace_root.join("ui-gallery-wheel-burst-coalescing.json");
        std::fs::write(&script_path, b"{}" as &[u8]).expect("write script");

        let prepared = prepare_cmd_run_setup(RunCommandSetupRequest {
            rest: &[script_path.to_string_lossy().to_string()],
            workspace_root: &workspace_root,
            checks: RunChecks::default(),
            pack_after_run: true,
            pack_out: None,
            pack_include_root_artifacts: false,
            pack_include_triage: false,
            pack_include_screenshots: false,
            ensure_ai_packet: false,
            devtools_ws_url: None,
            devtools_token: None,
            devtools_session_id: None,
            launch_requested: false,
            reuse_launch: false,
            keep_open: false,
            trace_chrome: false,
            launch_write_bundle_json: false,
        })
        .expect("prepare setup");

        assert_eq!(prepared.src, script_path);
        assert_eq!(prepared.pack_defaults, (true, true, true));
        assert!(prepared.wants_pack_zip);
        assert!(prepared.wants_post_run_bundle);
        assert!(prepared.wants_bundle_artifact);
        assert_eq!(
            prepared
                .checks_for_post_run
                .check_wheel_events_max_per_frame,
            Some(1)
        );
        assert!(!prepared.prefers_external_no_diag_post_run);
        assert_eq!(
            prepared.execution_route(),
            PreparedRunExecutionRoute::Filesystem
        );
    }

    #[test]
    fn prepare_cmd_run_setup_rejects_devtools_ws_when_launch_is_requested() {
        let workspace_root = make_temp_dir("prepare-setup-devtools-launch");
        let script_path = workspace_root.join("script.json");
        std::fs::write(&script_path, b"{}" as &[u8]).expect("write script");

        let result = prepare_cmd_run_setup(RunCommandSetupRequest {
            rest: &[script_path.to_string_lossy().to_string()],
            workspace_root: &workspace_root,
            checks: RunChecks::default(),
            pack_after_run: false,
            pack_out: None,
            pack_include_root_artifacts: false,
            pack_include_triage: false,
            pack_include_screenshots: false,
            ensure_ai_packet: false,
            devtools_ws_url: Some("ws://127.0.0.1:9222/devtools/browser/test"),
            devtools_token: None,
            devtools_session_id: None,
            launch_requested: true,
            reuse_launch: false,
            keep_open: false,
            trace_chrome: false,
            launch_write_bundle_json: false,
        });

        let err = match result {
            Ok(_) => panic!("devtools ws plus launch should be rejected"),
            Err(err) => err,
        };

        assert_eq!(
            err,
            "--launch/--reuse-launch is not supported with --devtools-ws-url"
        );
    }

    #[test]
    fn prepare_cmd_run_setup_applies_hello_world_compare_policy_and_external_preference() {
        let workspace_root = make_temp_dir("prepare-setup-hello-world-compare");
        let script_path = workspace_root.join("hello-world-compare-idle-present-gate.json");
        std::fs::write(&script_path, b"{}" as &[u8]).expect("write script");

        let prepared = prepare_cmd_run_setup(RunCommandSetupRequest {
            rest: &[script_path.to_string_lossy().to_string()],
            workspace_root: &workspace_root,
            checks: RunChecks::default(),
            pack_after_run: false,
            pack_out: None,
            pack_include_root_artifacts: false,
            pack_include_triage: false,
            pack_include_screenshots: false,
            ensure_ai_packet: false,
            devtools_ws_url: None,
            devtools_token: None,
            devtools_session_id: None,
            launch_requested: true,
            reuse_launch: false,
            keep_open: false,
            trace_chrome: false,
            launch_write_bundle_json: false,
        })
        .expect("prepare setup");

        assert_eq!(
            prepared
                .checks_for_post_run
                .check_hello_world_compare_idle_present_max_delta,
            Some(1)
        );
        assert!(prepared.prefers_external_no_diag_post_run);
        assert_eq!(
            prepared.execution_route(),
            PreparedRunExecutionRoute::ExternalNoDiagnostics
        );
    }
}
