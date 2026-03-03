use super::*;

#[path = "diag_perf/aux_scripts.rs"]
mod aux_scripts;
pub(crate) use aux_scripts::run_suite_aux_script_must_pass;
#[path = "diag_perf/baseline_rows.rs"]
mod baseline_rows;
#[path = "diag_perf/hints.rs"]
mod hints;
#[path = "diag_perf/outputs.rs"]
mod outputs;
#[path = "diag_perf/reporting.rs"]
mod reporting;
#[path = "diag_perf/run_script.rs"]
mod run_script;
#[path = "diag_perf/runs_rows.rs"]
mod runs_rows;
#[path = "diag_perf/stats_rows.rs"]
mod stats_rows;
#[path = "diag_perf/thresholds.rs"]
mod thresholds;

#[derive(Debug, Clone)]
pub(crate) struct PerfCmdContext {
    pub pack_after_run: bool,
    pub rest: Vec<String>,
    pub workspace_root: PathBuf,
    pub resolved_out_dir: PathBuf,
    pub resolved_ready_path: PathBuf,
    pub resolved_exit_path: PathBuf,
    pub resolved_script_path: PathBuf,
    pub resolved_script_trigger_path: PathBuf,
    pub resolved_script_result_path: PathBuf,
    pub resolved_script_result_trigger_path: PathBuf,
    pub check_perf_hints: bool,
    pub check_perf_hints_deny: Vec<String>,
    pub check_perf_hints_min_severity: Option<String>,
    pub check_pixels_changed_test_id: Option<String>,
    pub check_pixels_unchanged_test_id: Option<String>,
    pub devtools_session_id: Option<String>,
    pub devtools_token: Option<String>,
    pub devtools_ws_url: Option<String>,
    pub keep_open: bool,
    pub launch: Option<Vec<String>>,
    pub launch_env: Vec<(String, String)>,
    pub launch_high_priority: bool,
    pub launch_write_bundle_json: bool,
    pub max_frame_p95_layout_us: Option<u64>,
    pub max_frame_p95_solve_us: Option<u64>,
    pub max_frame_p95_total_us: Option<u64>,
    pub max_pointer_move_dispatch_us: Option<u64>,
    pub max_pointer_move_global_changes: Option<u64>,
    pub max_pointer_move_hit_test_us: Option<u64>,
    pub max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Option<u64>,
    pub max_top_layout_us: Option<u64>,
    pub max_top_solve_us: Option<u64>,
    pub max_top_total_us: Option<u64>,
    pub min_run_paint_cache_hit_test_only_replay_allowed_max: Option<u64>,
    pub perf_baseline_headroom_pct: u32,
    pub perf_baseline_out: Option<PathBuf>,
    pub perf_baseline_path: Option<PathBuf>,
    pub perf_baseline_seed_preset_paths: Vec<PathBuf>,
    pub perf_baseline_seed_specs: Vec<String>,
    pub perf_repeat: u64,
    pub perf_threshold_agg: PerfThresholdAggregate,
    pub poll_ms: u64,
    pub reuse_launch: bool,
    pub reuse_launch_per_script: bool,
    pub sort_override: Option<BundleStatsSort>,
    pub stats_json: bool,
    pub stats_top: usize,
    pub suite_prelude_each_run: bool,
    pub suite_prelude_scripts: Vec<PathBuf>,
    pub suite_prewarm_scripts: Vec<PathBuf>,
    pub timeout_ms: u64,
    pub trace_chrome: bool,
    pub warmup_frames: u64,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_perf(ctx: PerfCmdContext) -> Result<(), String> {
    let PerfCmdContext {
        pack_after_run,
        rest,
        workspace_root,
        resolved_out_dir,
        resolved_ready_path,
        resolved_exit_path,
        resolved_script_path,
        resolved_script_trigger_path,
        resolved_script_result_path,
        resolved_script_result_trigger_path,
        check_perf_hints,
        check_perf_hints_deny,
        check_perf_hints_min_severity,
        check_pixels_changed_test_id,
        check_pixels_unchanged_test_id,
        devtools_session_id,
        devtools_token,
        devtools_ws_url,
        keep_open,
        launch,
        launch_env,
        launch_high_priority,
        launch_write_bundle_json,
        max_frame_p95_layout_us,
        max_frame_p95_solve_us,
        max_frame_p95_total_us,
        max_pointer_move_dispatch_us,
        max_pointer_move_global_changes,
        max_pointer_move_hit_test_us,
        max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        max_top_layout_us,
        max_top_solve_us,
        max_top_total_us,
        min_run_paint_cache_hit_test_only_replay_allowed_max,
        perf_baseline_headroom_pct,
        perf_baseline_out,
        perf_baseline_path,
        perf_baseline_seed_preset_paths,
        perf_baseline_seed_specs,
        perf_repeat,
        perf_threshold_agg,
        poll_ms,
        reuse_launch,
        reuse_launch_per_script,
        sort_override,
        stats_json,
        stats_top,
        suite_prelude_each_run,
        suite_prelude_scripts,
        suite_prewarm_scripts,
        timeout_ms,
        trace_chrome,
        warmup_frames,
    } = ctx;

    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let (bundle_doctor_mode, rest) = parse_bundle_doctor_mode_from_rest(&rest)?;
    if rest.is_empty() {
        return Err(
            "missing suite name or script paths (try: fretboard diag perf ui-gallery)\n\
hint: list perf suites via `fretboard diag list suites --contains perf-`"
                .to_string(),
        );
    }

    let mut suite_name: Option<String> = None;
    let scripts: Vec<PathBuf> = if rest.len() == 1 {
        let name = rest[0].as_str();
        if let Some(paths) = perf_seed_policy::scripts_for_perf_suite_name(&workspace_root, name)? {
            suite_name = Some(name.to_string());
            paths
                .iter()
                .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                .collect()
        } else {
            let resolved = resolve_path(&workspace_root, PathBuf::from(name));
            if !resolved.exists() {
                let looks_like_suite_name =
                    !name.contains(['/', '\\', ':']) && !name.ends_with(".json");
                if looks_like_suite_name {
                    return Err(format!(
                        "unknown perf suite or script path: {name:?}\n\
hint: list perf suites via `fretboard diag list suites --contains perf-`\n\
hint: list promoted scripts via `fretboard diag list scripts --contains {name}`"
                    ));
                }
                return Err(format!(
                    "script path does not exist: {}",
                    resolved.display()
                ));
            }
            vec![resolved]
        }
    } else {
        rest.iter()
            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
            .collect()
    };

    let sort = sort_override.unwrap_or(BundleStatsSort::Time);
    let repeat = perf_repeat.max(1) as usize;
    if perf_threshold_agg == PerfThresholdAggregate::P90 && repeat < 10 {
        eprintln!(
            "warning: --perf-threshold-agg p90 with --repeat < 10 is not meaningful (p90 collapses to max); consider --repeat 11+",
        );
    }
    let reuse_process = launch.is_none() || reuse_launch;
    if reuse_launch_per_script {
        if !reuse_launch {
            return Err("--reuse-launch-per-script requires --reuse-launch".to_string());
        }
        if launch.is_none() {
            return Err("--reuse-launch-per-script requires --launch".to_string());
        }
        if keep_open {
            return Err("--reuse-launch-per-script is not supported with --keep-open".to_string());
        }
    }
    let reuse_process_per_script = reuse_process && reuse_launch_per_script && scripts.len() > 1;
    let use_devtools_ws =
        devtools_ws_url.is_some() || devtools_token.is_some() || devtools_session_id.is_some();
    if use_devtools_ws && (launch.is_some() || reuse_launch) {
        return Err("--launch/--reuse-launch is not supported with --devtools-ws-url".to_string());
    }
    let connected_ws: Option<ConnectedToolingTransport> = if use_devtools_ws {
        let ws_url = devtools_ws_url.clone().ok_or_else(|| {
            "missing --devtools-ws-url (required when using DevTools WS transport)".to_string()
        })?;
        let token = devtools_token.clone().ok_or_else(|| {
            "missing --devtools-token (required when using DevTools WS transport)".to_string()
        })?;
        Some(connect_devtools_ws_tooling(
            ws_url.as_str(),
            token.as_str(),
            devtools_session_id.as_deref(),
            timeout_ms,
            poll_ms,
        )?)
    } else {
        None
    };
    let perf_hint_gate_opts = parse_perf_hint_gate_options(
        check_perf_hints,
        &check_perf_hints_deny,
        check_perf_hints_min_severity.as_deref(),
    )?;
    let wants_perf_hints = perf_hint_gate_opts.enabled;
    let cli_thresholds = PerfThresholds {
        max_top_total_us,
        max_top_layout_us,
        max_top_solve_us,
        max_frame_p95_total_us,
        max_frame_p95_layout_us,
        max_frame_p95_solve_us,
        max_pointer_move_dispatch_us,
        max_pointer_move_hit_test_us,
        max_pointer_move_global_changes,
        min_run_paint_cache_hit_test_only_replay_allowed_max,
        max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        max_renderer_encode_scene_us: None,
        max_renderer_upload_us: None,
        max_renderer_record_passes_us: None,
        max_renderer_encoder_finish_us: None,
        max_renderer_prepare_text_us: None,
        max_renderer_prepare_svg_us: None,
    };
    let perf_baseline = perf_baseline_path
        .clone()
        .map(|p| resolve_path(&workspace_root, p))
        .map(|p| read_perf_baseline_file(&workspace_root, &p))
        .transpose()?;
    let perf_baseline_out = perf_baseline_out
        .clone()
        .map(|p| resolve_path(&workspace_root, p));

    let seed_policy: Option<ResolvedPerfBaselineSeedPolicy> = if perf_baseline_out.is_some() {
        Some(perf_seed_policy::resolve_perf_baseline_seed_policy(
            &workspace_root,
            suite_name.as_deref(),
            &scripts,
            &perf_baseline_seed_preset_paths,
            &perf_baseline_seed_specs,
        )?)
    } else {
        None
    };
    let wants_perf_thresholds = cli_thresholds.any() || perf_baseline.is_some();
    let mut child: Option<LaunchedDemo> = None;
    let mut connected_fs: Option<ConnectedToolingTransport> = None;
    let launched_by_fretboard = reuse_launch && launch.is_some();
    let mut perf_launch_env = launch_env.clone();
    std::fs::create_dir_all(&resolved_out_dir).map_err(|e| e.to_string())?;
    let perf_capabilities_check_path = resolved_out_dir.join("check.capabilities.json");
    let _ = ensure_env_var(&mut perf_launch_env, "FRET_DIAG_RENDERER_PERF", "1");
    if let Some(name) = suite_name.as_deref() {
        // Make the common UI gallery perf suites reproducible without requiring callers
        // to remember a pile of `--env` flags. Callers can still override them explicitly
        // via `--env KEY=...`.
        if matches!(
            name,
            "ui-gallery"
                | "ui-gallery-steady"
                | "ui-gallery-complex-steady"
                | "ui-gallery-complex-typical"
                | "ui-resize-probes"
                | "ui-code-editor-resize-probes"
        ) {
            let _ = ensure_env_var(&mut perf_launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
            let _ = ensure_env_var(
                &mut perf_launch_env,
                "FRET_UI_GALLERY_VIEW_CACHE_SHELL",
                "1",
            );
        }
        if matches!(
            name,
            "ui-gallery"
                | "ui-gallery-steady"
                | "ui-gallery-complex-steady"
                | "ui-gallery-complex-typical"
                | "ui-resize-probes"
                | "ui-code-editor-resize-probes"
        ) {
            let _ = ensure_env_var(
                &mut perf_launch_env,
                "FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS",
                "1",
            );
        }
        if matches!(name, "ui-gallery-complex-typical") {
            // Typical-perf triage needs enough snapshots per bundle to make frame
            // percentiles meaningful (otherwise `p95` collapses to `max`).
            let _ = ensure_env_var(&mut perf_launch_env, "FRET_DIAG_MAX_SNAPSHOTS", "180");
            let _ = ensure_env_var(
                &mut perf_launch_env,
                "FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS",
                "180",
            );
        }
    }

    let mut perf_json_rows: Vec<serde_json::Value> = Vec::new();
    let mut perf_threshold_rows: Vec<serde_json::Value> = Vec::new();
    let mut perf_threshold_failures: Vec<serde_json::Value> = Vec::new();
    let mut perf_hint_rows: Vec<serde_json::Value> = Vec::new();
    let mut perf_hint_failures: Vec<serde_json::Value> = Vec::new();
    let mut perf_baseline_rows: Vec<serde_json::Value> = Vec::new();
    let mut overall_worst: Option<(u64, PathBuf, PathBuf)> = None;
    let stats_opts = BundleStatsOptions { warmup_frames };
    let perf_suite_prewarm_scripts: Vec<PathBuf> = suite_prewarm_scripts
        .iter()
        .cloned()
        .map(|p| resolve_path(&workspace_root, p))
        .collect();
    let perf_suite_prelude_scripts: Vec<PathBuf> = suite_prelude_scripts
        .iter()
        .cloned()
        .map(|p| resolve_path(&workspace_root, p))
        .collect();

    let run_suite_aux_script_must_pass = |src: &PathBuf,
                                          child: &mut Option<LaunchedDemo>,
                                          connected_fs: Option<&ConnectedToolingTransport>|
     -> Result<(), String> {
        aux_scripts::run_suite_aux_script_must_pass(
            src,
            launch.is_some() || reuse_launch,
            child,
            use_devtools_ws,
            connected_ws.as_ref(),
            connected_fs,
            &workspace_root,
            &resolved_out_dir,
            &resolved_exit_path,
            true,
            reuse_process,
            &resolved_script_result_path,
            &resolved_script_result_trigger_path,
            &perf_capabilities_check_path,
            timeout_ms,
            poll_ms,
        )
    };

    if let Some(baseline) = perf_baseline.as_ref() {
        for src in &scripts {
            let key = normalize_repo_relative_path(&workspace_root, src);
            if !baseline.thresholds_by_script.contains_key(&key) {
                return Err(format!(
                    "perf baseline missing entry for script: {key} (baseline={})",
                    baseline.path.display()
                ));
            }
        }
    }

    let mut launch_fs_transport_cfg =
        crate::transport::FsDiagTransportConfig::from_out_dir(resolved_out_dir.clone());
    launch_fs_transport_cfg.script_path = resolved_script_path.clone();
    launch_fs_transport_cfg.script_trigger_path = resolved_script_trigger_path.clone();
    launch_fs_transport_cfg.script_result_path = resolved_script_result_path.clone();
    launch_fs_transport_cfg.script_result_trigger_path =
        resolved_script_result_trigger_path.clone();

    let perf_wants_screenshots = check_pixels_changed_test_id.is_some()
        || check_pixels_unchanged_test_id.is_some()
        || scripts
            .iter()
            .any(|src| crate::script_requests_screenshots(src))
        || perf_suite_prewarm_scripts
            .iter()
            .any(|src| crate::script_requests_screenshots(src))
        || perf_suite_prelude_scripts
            .iter()
            .any(|src| crate::script_requests_screenshots(src));

    if launched_by_fretboard && !reuse_process_per_script {
        child = maybe_launch_demo(
            &launch,
            &perf_launch_env,
            &workspace_root,
            &resolved_ready_path,
            &resolved_exit_path,
            &launch_fs_transport_cfg,
            perf_wants_screenshots,
            launch_write_bundle_json,
            timeout_ms,
            poll_ms,
            launch_high_priority,
        )
        .inspect_err(|err| {
            write_tooling_failure_script_result_if_missing(
                &resolved_script_result_path,
                "tooling.launch.failed",
                err,
                "tooling_error",
                Some("maybe_launch_demo".to_string()),
            );
        })?;
        connected_fs = None;
    }

    if reuse_process && !reuse_process_per_script && !perf_suite_prewarm_scripts.is_empty() {
        ensure_perf_fs_transport_connected(
            &mut connected_fs,
            use_devtools_ws,
            &launch_fs_transport_cfg,
            &resolved_ready_path,
            child.is_some(),
            timeout_ms,
            poll_ms,
            &resolved_script_result_path,
        )?;
        for prewarm in &perf_suite_prewarm_scripts {
            run_suite_aux_script_must_pass(prewarm, &mut child, connected_fs.as_ref())?;
        }
    }

    for (script_index, src) in scripts.into_iter().enumerate() {
        if reuse_process_per_script && launched_by_fretboard && script_index > 0 {
            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
            child = None;
            connected_fs = None;
        }
        if reuse_process_per_script && launched_by_fretboard && child.is_none() {
            child = maybe_launch_demo(
                &launch,
                &perf_launch_env,
                &workspace_root,
                &resolved_ready_path,
                &resolved_exit_path,
                &launch_fs_transport_cfg,
                perf_wants_screenshots,
                launch_write_bundle_json,
                timeout_ms,
                poll_ms,
                launch_high_priority,
            )
            .inspect_err(|err| {
                write_tooling_failure_script_result_if_missing(
                    &resolved_script_result_path,
                    "tooling.launch.failed",
                    err,
                    "tooling_error",
                    Some("maybe_launch_demo".to_string()),
                );
            })?;
            connected_fs = None;
            if !perf_suite_prewarm_scripts.is_empty() {
                ensure_perf_fs_transport_connected(
                    &mut connected_fs,
                    use_devtools_ws,
                    &launch_fs_transport_cfg,
                    &resolved_ready_path,
                    child.is_some(),
                    timeout_ms,
                    poll_ms,
                    &resolved_script_result_path,
                )?;
                for prewarm in &perf_suite_prewarm_scripts {
                    run_suite_aux_script_must_pass(prewarm, &mut child, connected_fs.as_ref())?;
                }
            }
        }

        if repeat == 1 {
            if !reuse_process {
                child = maybe_launch_demo(
                    &launch,
                    &perf_launch_env,
                    &workspace_root,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    &launch_fs_transport_cfg,
                    perf_wants_screenshots,
                    launch_write_bundle_json,
                    timeout_ms,
                    poll_ms,
                    launch_high_priority,
                )
                .inspect_err(|err| {
                    write_tooling_failure_script_result_if_missing(
                        &resolved_script_result_path,
                        "tooling.launch.failed",
                        err,
                        "tooling_error",
                        Some("maybe_launch_demo".to_string()),
                    );
                })?;
                connected_fs = None;
            }

            if !reuse_process {
                clear_script_result_files(
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                );
            }

            if !reuse_process && !perf_suite_prewarm_scripts.is_empty() {
                ensure_perf_fs_transport_connected(
                    &mut connected_fs,
                    use_devtools_ws,
                    &launch_fs_transport_cfg,
                    &resolved_ready_path,
                    child.is_some(),
                    timeout_ms,
                    poll_ms,
                    &resolved_script_result_path,
                )?;
                for prewarm in &perf_suite_prewarm_scripts {
                    run_suite_aux_script_must_pass(prewarm, &mut child, connected_fs.as_ref())?;
                }
            }
            if !perf_suite_prelude_scripts.is_empty() {
                ensure_perf_fs_transport_connected(
                    &mut connected_fs,
                    use_devtools_ws,
                    &launch_fs_transport_cfg,
                    &resolved_ready_path,
                    child.is_some(),
                    timeout_ms,
                    poll_ms,
                    &resolved_script_result_path,
                )?;
                for prelude in &perf_suite_prelude_scripts {
                    run_suite_aux_script_must_pass(prelude, &mut child, connected_fs.as_ref())?;
                }
            }

            ensure_perf_fs_transport_connected(
                &mut connected_fs,
                use_devtools_ws,
                &launch_fs_transport_cfg,
                &resolved_ready_path,
                child.is_some(),
                timeout_ms,
                poll_ms,
                &resolved_script_result_path,
            )?;
            let script_key = normalize_repo_relative_path(&workspace_root, &src);
            let bundle_path = run_script::run_perf_script_and_resolve_bundle_artifact_path(
                &src,
                script_key.as_str(),
                launch.is_some() || reuse_launch,
                &mut child,
                use_devtools_ws,
                connected_ws.as_ref(),
                connected_fs.as_ref(),
                &resolved_out_dir,
                &resolved_exit_path,
                &resolved_script_result_path,
                &perf_capabilities_check_path,
                timeout_ms,
                poll_ms,
            )?;

            if let Some(bundle_path) = bundle_path {
                let mut report =
                    bundle_stats_from_path(&bundle_path, stats_top.max(1), sort, stats_opts)?;
                let mut report_warmup_frames = warmup_frames;
                if warmup_frames > 0 && report.top.is_empty() {
                    report = bundle_stats_from_path(
                        &bundle_path,
                        stats_top.max(1),
                        sort,
                        BundleStatsOptions::default(),
                    )?;
                    report_warmup_frames = 0;
                }
                if trace_chrome && let Some(dir) = bundle_path.parent() {
                    let trace_path = dir.join("trace.chrome.json");
                    let _ = crate::trace::write_chrome_trace_from_bundle_path(
                        &bundle_path,
                        &trace_path,
                    );
                }
                if wants_perf_hints {
                    let triage =
                        triage_json_from_stats(&bundle_path, &report, sort, report_warmup_frames);
                    let failures = perf_hint_gate_failures_for_triage_json(
                        script_key.as_str(),
                        &bundle_path,
                        Some(0),
                        &triage,
                        &perf_hint_gate_opts,
                    );
                    let hints = triage
                        .get("hints")
                        .cloned()
                        .unwrap_or(serde_json::json!([]));
                    let unit_costs = triage
                        .get("unit_costs")
                        .cloned()
                        .unwrap_or(serde_json::json!({}));
                    let worst = triage
                        .get("worst")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    let trace_chrome_json_path = triage
                        .get("bundle")
                        .and_then(|b| b.get("trace_chrome_json_path"))
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);

                    perf_hint_failures.extend(failures.clone());
                    hints::push_perf_hint_row(
                        &mut perf_hint_rows,
                        script_key.as_str(),
                        sort,
                        repeat,
                        0usize,
                        &bundle_path,
                        report_warmup_frames,
                        hints,
                        unit_costs,
                        worst,
                        trace_chrome_json_path,
                        failures,
                    );
                }
                let top = report.top.first();
                let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
                let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
                let top_solve = top.map(|r| r.layout_engine_solve_time_us).unwrap_or(0);
                let top_solves = top.map(|r| r.layout_engine_solves).unwrap_or(0);
                let top_prepaint = top.map(|r| r.prepaint_time_us).unwrap_or(0);
                let top_paint = top.map(|r| r.paint_time_us).unwrap_or(0);
                let top_dispatch = top.map(|r| r.dispatch_time_us).unwrap_or(0);
                let top_hit_test = top.map(|r| r.hit_test_time_us).unwrap_or(0);
                let top_tick = top.map(|r| r.tick_id).unwrap_or(0);
                let top_frame = top.map(|r| r.frame_id).unwrap_or(0);
                let pointer_move_frames_present = report.pointer_move_frames_present;
                let pointer_move_frames_considered = report.pointer_move_frames_considered as u64;
                let pointer_move_max_dispatch_time_us = report.pointer_move_max_dispatch_time_us;
                let pointer_move_max_hit_test_time_us = report.pointer_move_max_hit_test_time_us;
                let pointer_move_snapshots_with_global_changes =
                    report.pointer_move_snapshots_with_global_changes as u64;
                let (
                    run_paint_cache_hit_test_only_replay_allowed_max,
                    run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                ) = diag_policy::bundle_paint_cache_hit_test_only_replay_maxes(
                    &bundle_path,
                    report_warmup_frames,
                )?;
                if stats_json {
                    stats_rows::push_perf_json_row(
                        &mut perf_json_rows,
                        script_key.as_str(),
                        sort,
                        &report,
                        &bundle_path,
                    );
                } else {
                    println!(
                        "PERF {} sort={} top.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{} top.tick={} top.frame={} bundle={}",
                        src.display(),
                        sort.as_str(),
                        top_total,
                        top_layout,
                        top_solve,
                        top_prepaint,
                        top_paint,
                        top_dispatch,
                        top_hit_test,
                        top_tick,
                        top_frame,
                        bundle_path.display(),
                    );
                }

                if perf_baseline_out.is_some() {
                    let policy = seed_policy
                        .as_ref()
                        .ok_or_else(|| "internal error: missing seed policy".to_string())?;

                    let p90_total = top_total;
                    let p95_total = top_total;
                    let p90_layout = top_layout;
                    let p95_layout = top_layout;
                    let p90_solve = top_solve;
                    let p95_solve = top_solve;

                    let seed_total = policy.seed_for(&script_key, PerfSeedMetric::TopTotalTimeUs);
                    let seed_layout = policy.seed_for(&script_key, PerfSeedMetric::TopLayoutTimeUs);
                    let seed_solve =
                        policy.seed_for(&script_key, PerfSeedMetric::TopLayoutEngineSolveTimeUs);

                    let seed_total_value = match seed_total {
                        PerfBaselineSeed::Max => top_total,
                        PerfBaselineSeed::P90 => p90_total,
                        PerfBaselineSeed::P95 => p95_total,
                    };
                    let seed_layout_value = match seed_layout {
                        PerfBaselineSeed::Max => top_layout,
                        PerfBaselineSeed::P90 => p90_layout,
                        PerfBaselineSeed::P95 => p95_layout,
                    };
                    let seed_solve_value = match seed_solve {
                        PerfBaselineSeed::Max => top_solve,
                        PerfBaselineSeed::P90 => p90_solve,
                        PerfBaselineSeed::P95 => p95_solve,
                    };

                    let thr_total =
                        apply_perf_baseline_headroom(seed_total_value, perf_baseline_headroom_pct);
                    let thr_layout =
                        apply_perf_baseline_headroom(seed_layout_value, perf_baseline_headroom_pct);
                    let thr_solve =
                        apply_perf_baseline_headroom(seed_solve_value, perf_baseline_headroom_pct);

                    let thr_pointer_move_dispatch = apply_perf_baseline_headroom(
                        pointer_move_max_dispatch_time_us,
                        perf_baseline_headroom_pct,
                    );
                    let thr_pointer_move_hit_test = apply_perf_baseline_headroom(
                        pointer_move_max_hit_test_time_us,
                        perf_baseline_headroom_pct,
                    );
                    let thr_pointer_move_global_changes = apply_perf_baseline_headroom(
                        pointer_move_snapshots_with_global_changes,
                        perf_baseline_headroom_pct,
                    );
                    let thr_min_run_paint_cache_hit_test_only_replay_allowed_max =
                        apply_perf_baseline_floor(
                            run_paint_cache_hit_test_only_replay_allowed_max,
                            perf_baseline_headroom_pct,
                        );
                    let thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max =
                        apply_perf_baseline_headroom(
                            run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            perf_baseline_headroom_pct,
                        );
                    let thr_renderer_encode_scene_us = apply_perf_baseline_headroom(
                        report.max_renderer_encode_scene_us,
                        perf_baseline_headroom_pct,
                    );
                    let thr_renderer_upload_us = apply_perf_baseline_headroom(
                        report.max_renderer_upload_us,
                        perf_baseline_headroom_pct,
                    );
                    let thr_renderer_record_passes_us = apply_perf_baseline_headroom(
                        report.max_renderer_record_passes_us,
                        perf_baseline_headroom_pct,
                    );
                    let thr_renderer_encoder_finish_us = apply_perf_baseline_headroom(
                        report.max_renderer_encoder_finish_us,
                        perf_baseline_headroom_pct,
                    );
                    let thr_renderer_prepare_text_us = apply_perf_baseline_headroom(
                        report.max_renderer_prepare_text_us,
                        perf_baseline_headroom_pct,
                    );
                    let thr_renderer_prepare_svg_us = apply_perf_baseline_headroom(
                        report.max_renderer_prepare_svg_us,
                        perf_baseline_headroom_pct,
                    );

                    baseline_rows::push_perf_baseline_row_single(
                        &mut perf_baseline_rows,
                        script_key.as_str(),
                        baseline_rows::TopTimesUs::new(top_total, top_layout, top_solve),
                        baseline_rows::TopTimesUs::new(p90_total, p90_layout, p90_solve),
                        baseline_rows::TopTimesUs::new(p95_total, p95_layout, p95_solve),
                        baseline_rows::PointerMoveMetrics::new(
                            pointer_move_max_dispatch_time_us,
                            pointer_move_max_hit_test_time_us,
                            pointer_move_snapshots_with_global_changes,
                        ),
                        baseline_rows::PaintCacheReplayMetrics::new(
                            run_paint_cache_hit_test_only_replay_allowed_max,
                            run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        ),
                        baseline_rows::RendererTimesUs::new(
                            report.max_renderer_encode_scene_us,
                            report.max_renderer_upload_us,
                            report.max_renderer_record_passes_us,
                            report.max_renderer_encoder_finish_us,
                            report.max_renderer_prepare_text_us,
                            report.max_renderer_prepare_svg_us,
                        ),
                        seed_total,
                        seed_layout,
                        seed_solve,
                        seed_total_value,
                        seed_layout_value,
                        seed_solve_value,
                        thr_total,
                        thr_layout,
                        thr_solve,
                        baseline_rows::PointerMoveMetrics::new(
                            thr_pointer_move_dispatch,
                            thr_pointer_move_hit_test,
                            thr_pointer_move_global_changes,
                        ),
                        thr_min_run_paint_cache_hit_test_only_replay_allowed_max,
                        thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        baseline_rows::RendererTimesUs::new(
                            thr_renderer_encode_scene_us,
                            thr_renderer_upload_us,
                            thr_renderer_record_passes_us,
                            thr_renderer_encoder_finish_us,
                            thr_renderer_prepare_text_us,
                            thr_renderer_prepare_svg_us,
                        ),
                    );
                }
                if wants_perf_thresholds {
                    let baseline_thresholds = perf_baseline
                        .as_ref()
                        .and_then(|b| b.thresholds_by_script.get(&script_key).copied())
                        .unwrap_or_default();
                    let (thr_total, src_total) = resolve_threshold(
                        cli_thresholds.max_top_total_us,
                        baseline_thresholds.max_top_total_us,
                    );
                    let (thr_layout, src_layout) = resolve_threshold(
                        cli_thresholds.max_top_layout_us,
                        baseline_thresholds.max_top_layout_us,
                    );
                    let (thr_solve, src_solve) = resolve_threshold(
                        cli_thresholds.max_top_solve_us,
                        baseline_thresholds.max_top_solve_us,
                    );
                    let (thr_frame_p95_total, src_frame_p95_total) = resolve_threshold(
                        cli_thresholds.max_frame_p95_total_us,
                        baseline_thresholds.max_frame_p95_total_us,
                    );
                    let (thr_frame_p95_layout, src_frame_p95_layout) = resolve_threshold(
                        cli_thresholds.max_frame_p95_layout_us,
                        baseline_thresholds.max_frame_p95_layout_us,
                    );
                    let (thr_frame_p95_solve, src_frame_p95_solve) = resolve_threshold(
                        cli_thresholds.max_frame_p95_solve_us,
                        baseline_thresholds.max_frame_p95_solve_us,
                    );
                    let (thr_pointer_move_dispatch, src_pointer_move_dispatch) = resolve_threshold(
                        cli_thresholds.max_pointer_move_dispatch_us,
                        baseline_thresholds.max_pointer_move_dispatch_us,
                    );
                    let (thr_pointer_move_hit_test, src_pointer_move_hit_test) = resolve_threshold(
                        cli_thresholds.max_pointer_move_hit_test_us,
                        baseline_thresholds.max_pointer_move_hit_test_us,
                    );
                    let (thr_pointer_move_global_changes, src_pointer_move_global_changes) =
                        resolve_threshold(
                            cli_thresholds.max_pointer_move_global_changes,
                            baseline_thresholds.max_pointer_move_global_changes,
                        );
                    let (
                        thr_paint_cache_hit_test_only_replay_allowed_max,
                        src_paint_cache_hit_test_only_replay_allowed_max,
                    ) = resolve_threshold(
                        cli_thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max,
                        baseline_thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max,
                    );
                    let (
                        thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    ) = resolve_threshold(
                        cli_thresholds
                            .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        baseline_thresholds
                            .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    );
                    thresholds::push_single_run_threshold_row_and_failures(
                        &mut perf_threshold_rows,
                        &mut perf_threshold_failures,
                        thresholds::SingleRunThresholdInputs {
                            script_key: script_key.as_str(),
                            sort,
                            perf_threshold_agg,
                            cli_thresholds,
                            baseline_thresholds,
                            top_total,
                            top_layout,
                            top_solve,
                            top_solves,
                            top_tick,
                            top_frame,
                            frame_p95_total_time_us: report.p95_total_time_us,
                            frame_p95_layout_time_us: report.p95_layout_time_us,
                            frame_p95_layout_engine_solve_time_us: report
                                .p95_layout_engine_solve_time_us,
                            pointer_move_frames_present,
                            pointer_move_frames_considered,
                            pointer_move_max_dispatch_time_us,
                            pointer_move_max_hit_test_time_us,
                            pointer_move_snapshots_with_global_changes,
                            run_paint_cache_hit_test_only_replay_allowed_max,
                            run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            max_renderer_encode_scene_us: report.max_renderer_encode_scene_us,
                            max_renderer_upload_us: report.max_renderer_upload_us,
                            max_renderer_record_passes_us: report.max_renderer_record_passes_us,
                            max_renderer_encoder_finish_us: report.max_renderer_encoder_finish_us,
                            max_renderer_prepare_text_us: report.max_renderer_prepare_text_us,
                            max_renderer_prepare_svg_us: report.max_renderer_prepare_svg_us,
                            bundle_path: bundle_path.as_path(),
                            thr_total,
                            src_total,
                            thr_layout,
                            src_layout,
                            thr_solve,
                            src_solve,
                            thr_frame_p95_total,
                            src_frame_p95_total,
                            thr_frame_p95_layout,
                            src_frame_p95_layout,
                            thr_frame_p95_solve,
                            src_frame_p95_solve,
                            thr_pointer_move_dispatch,
                            src_pointer_move_dispatch,
                            thr_pointer_move_hit_test,
                            src_pointer_move_hit_test,
                            thr_pointer_move_global_changes,
                            src_pointer_move_global_changes,
                            thr_paint_cache_hit_test_only_replay_allowed_max:
                                thr_paint_cache_hit_test_only_replay_allowed_max,
                            src_paint_cache_hit_test_only_replay_allowed_max,
                            thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max:
                                thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        },
                    );
                }

                match &overall_worst {
                    Some((prev_us, _, _)) if *prev_us >= top_total => {}
                    _ => overall_worst = Some((top_total, src.clone(), bundle_path)),
                }
            } else if stats_json {
                reporting::push_perf_json_no_last_bundle_dir(
                    &mut perf_json_rows,
                    script_key.clone(),
                    sort,
                    None,
                );
            } else {
                reporting::print_perf_no_last_bundle_dir(src.as_path(), sort, None);
            }

            if !reuse_process {
                stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                connected_fs = None;
            }
            continue;
        }

        let mut runs_total: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_layout: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_solve: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_prepaint: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_paint: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_dispatch: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_hit_test: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_frame_p95_total: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_frame_p95_layout: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_frame_p95_solve: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_pointer_move_dispatch: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_pointer_move_hit_test: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_pointer_move_global_changes: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_paint_cache_hit_test_only_replay_allowed_max: Vec<u64> =
            Vec::with_capacity(repeat);
        let mut runs_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Vec<u64> =
            Vec::with_capacity(repeat);
        let mut runs_renderer_encode_scene_us: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_renderer_upload_us: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_renderer_record_passes_us: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_renderer_encoder_finish_us: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_renderer_prepare_text_us: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_renderer_prepare_svg_us: Vec<u64> = Vec::with_capacity(repeat);
        let mut runs_json: Vec<serde_json::Value> = Vec::with_capacity(repeat);
        let mut script_worst: Option<(u64, PathBuf, u64)> = None;
        let mut script_worst_layout: Option<(u64, PathBuf, u64)> = None;
        let mut script_worst_solve: Option<(u64, PathBuf, u64)> = None;

        for run_index in 0..repeat {
            if !reuse_process {
                child = maybe_launch_demo(
                    &launch,
                    &perf_launch_env,
                    &workspace_root,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    &launch_fs_transport_cfg,
                    perf_wants_screenshots,
                    launch_write_bundle_json,
                    timeout_ms,
                    poll_ms,
                    launch_high_priority,
                )
                .inspect_err(|err| {
                    write_tooling_failure_script_result_if_missing(
                        &resolved_script_result_path,
                        "tooling.launch.failed",
                        err,
                        "tooling_error",
                        Some("maybe_launch_demo".to_string()),
                    );
                })?;
                connected_fs = None;
            }

            if !reuse_process {
                clear_script_result_files(
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                );
            }

            if !reuse_process && !perf_suite_prewarm_scripts.is_empty() {
                ensure_perf_fs_transport_connected(
                    &mut connected_fs,
                    use_devtools_ws,
                    &launch_fs_transport_cfg,
                    &resolved_ready_path,
                    child.is_some(),
                    timeout_ms,
                    poll_ms,
                    &resolved_script_result_path,
                )?;
                for prewarm in &perf_suite_prewarm_scripts {
                    run_suite_aux_script_must_pass(prewarm, &mut child, connected_fs.as_ref())?;
                }
            }
            if !perf_suite_prelude_scripts.is_empty()
                && (!reuse_process || suite_prelude_each_run || run_index == 0)
            {
                ensure_perf_fs_transport_connected(
                    &mut connected_fs,
                    use_devtools_ws,
                    &launch_fs_transport_cfg,
                    &resolved_ready_path,
                    child.is_some(),
                    timeout_ms,
                    poll_ms,
                    &resolved_script_result_path,
                )?;
                for prelude in &perf_suite_prelude_scripts {
                    run_suite_aux_script_must_pass(prelude, &mut child, connected_fs.as_ref())?;
                }
            }

            ensure_perf_fs_transport_connected(
                &mut connected_fs,
                use_devtools_ws,
                &launch_fs_transport_cfg,
                &resolved_ready_path,
                child.is_some(),
                timeout_ms,
                poll_ms,
                &resolved_script_result_path,
            )?;
            let script_key = normalize_repo_relative_path(&workspace_root, &src);
            let bundle_path = run_script::run_perf_script_and_resolve_bundle_artifact_path(
                &src,
                script_key.as_str(),
                launch.is_some() || reuse_launch,
                &mut child,
                use_devtools_ws,
                connected_ws.as_ref(),
                connected_fs.as_ref(),
                &resolved_out_dir,
                &resolved_exit_path,
                &resolved_script_result_path,
                &perf_capabilities_check_path,
                timeout_ms,
                poll_ms,
            )?;

            let Some(bundle_path) = bundle_path else {
                if stats_json {
                    reporting::push_perf_json_no_last_bundle_dir(
                        &mut perf_json_rows,
                        src.display().to_string(),
                        sort,
                        Some(repeat),
                    );
                } else {
                    reporting::print_perf_no_last_bundle_dir(src.as_path(), sort, Some(repeat));
                }
                if !reuse_process {
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                    connected_fs = None;
                }
                break;
            };

            if bundle_doctor_mode != BundleDoctorMode::Off {
                if let Err(err) = run_bundle_doctor_for_bundle_path(
                    &bundle_path,
                    bundle_doctor_mode,
                    warmup_frames,
                ) {
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                    return Err(err);
                }
            }

            let mut report =
                bundle_stats_from_path(&bundle_path, stats_top.max(1), sort, stats_opts)?;
            let mut report_warmup_frames = warmup_frames;
            if warmup_frames > 0 && report.top.is_empty() {
                report = bundle_stats_from_path(
                    &bundle_path,
                    stats_top.max(1),
                    sort,
                    BundleStatsOptions::default(),
                )?;
                report_warmup_frames = 0;
            }
            if trace_chrome && let Some(dir) = bundle_path.parent() {
                let trace_path = dir.join("trace.chrome.json");
                let _ =
                    crate::trace::write_chrome_trace_from_bundle_path(&bundle_path, &trace_path);
            }

            if wants_perf_hints {
                let triage =
                    triage_json_from_stats(&bundle_path, &report, sort, report_warmup_frames);
                let failures = perf_hint_gate_failures_for_triage_json(
                    &script_key,
                    &bundle_path,
                    Some(run_index as u64),
                    &triage,
                    &perf_hint_gate_opts,
                );
                let hints = triage
                    .get("hints")
                    .cloned()
                    .unwrap_or(serde_json::json!([]));
                let unit_costs = triage
                    .get("unit_costs")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));
                let worst = triage
                    .get("worst")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let trace_chrome_json_path = triage
                    .get("bundle")
                    .and_then(|b| b.get("trace_chrome_json_path"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                perf_hint_failures.extend(failures.clone());
                hints::push_perf_hint_row(
                    &mut perf_hint_rows,
                    script_key.as_str(),
                    sort,
                    repeat,
                    run_index,
                    &bundle_path,
                    report_warmup_frames,
                    hints,
                    unit_costs,
                    worst,
                    trace_chrome_json_path,
                    failures,
                );
            }

            let top = report.top.first();
            let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
            let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
            let top_solve = top.map(|r| r.layout_engine_solve_time_us).unwrap_or(0);
            let top_prepaint = top.map(|r| r.prepaint_time_us).unwrap_or(0);
            let top_paint = top.map(|r| r.paint_time_us).unwrap_or(0);
            let top_dispatch = top.map(|r| r.dispatch_time_us).unwrap_or(0);
            let top_hit_test = top.map(|r| r.hit_test_time_us).unwrap_or(0);

            runs_total.push(top_total);
            runs_layout.push(top_layout);
            runs_solve.push(top_solve);
            runs_prepaint.push(top_prepaint);
            runs_paint.push(top_paint);
            runs_dispatch.push(top_dispatch);
            runs_hit_test.push(top_hit_test);
            runs_frame_p95_total.push(report.p95_total_time_us);
            runs_frame_p95_layout.push(report.p95_layout_time_us);
            runs_frame_p95_solve.push(report.p95_layout_engine_solve_time_us);
            let pointer_move_max_dispatch_time_us = report.pointer_move_max_dispatch_time_us;
            let pointer_move_max_hit_test_time_us = report.pointer_move_max_hit_test_time_us;
            let pointer_move_snapshots_with_global_changes =
                report.pointer_move_snapshots_with_global_changes as u64;
            let (
                run_paint_cache_hit_test_only_replay_allowed_max,
                run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            ) = diag_policy::bundle_paint_cache_hit_test_only_replay_maxes(
                &bundle_path,
                report_warmup_frames,
            )?;
            runs_pointer_move_dispatch.push(pointer_move_max_dispatch_time_us);
            runs_pointer_move_hit_test.push(pointer_move_max_hit_test_time_us);
            runs_pointer_move_global_changes.push(pointer_move_snapshots_with_global_changes);
            runs_paint_cache_hit_test_only_replay_allowed_max
                .push(run_paint_cache_hit_test_only_replay_allowed_max);
            runs_paint_cache_hit_test_only_replay_rejected_key_mismatch_max
                .push(run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max);
            runs_renderer_encode_scene_us.push(report.max_renderer_encode_scene_us);
            runs_renderer_upload_us.push(report.max_renderer_upload_us);
            runs_renderer_record_passes_us.push(report.max_renderer_record_passes_us);
            runs_renderer_encoder_finish_us.push(report.max_renderer_encoder_finish_us);
            runs_renderer_prepare_text_us.push(report.max_renderer_prepare_text_us);
            runs_renderer_prepare_svg_us.push(report.max_renderer_prepare_svg_us);
            runs_rows::push_perf_repeat_run_json_row(
                &mut runs_json,
                run_index,
                &report,
                top_total,
                top_layout,
                top_solve,
                top_prepaint,
                top_paint,
                top_dispatch,
                top_hit_test,
                run_paint_cache_hit_test_only_replay_allowed_max,
                run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                &bundle_path,
            );

            match &script_worst {
                Some((prev_us, _, _)) if *prev_us >= top_total => {}
                _ => script_worst = Some((top_total, bundle_path.clone(), run_index as u64)),
            }
            match &script_worst_layout {
                Some((prev_us, _, _)) if *prev_us >= top_layout => {}
                _ => {
                    script_worst_layout = Some((top_layout, bundle_path.clone(), run_index as u64))
                }
            }
            match &script_worst_solve {
                Some((prev_us, _, _)) if *prev_us >= top_solve => {}
                _ => script_worst_solve = Some((top_solve, bundle_path.clone(), run_index as u64)),
            }

            match &overall_worst {
                Some((prev_us, _, _)) if *prev_us >= top_total => {}
                _ => overall_worst = Some((top_total, src.clone(), bundle_path.clone())),
            }

            if !reuse_process {
                stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                connected_fs = None;
            }
        }

        if runs_total.len() == repeat {
            if stats_json {
                reporting::push_perf_json_repeat_summary_row(
                    &mut perf_json_rows,
                    src.as_path(),
                    sort,
                    repeat,
                    &runs_json,
                    &runs_total,
                    &runs_layout,
                    &runs_solve,
                    &runs_prepaint,
                    &runs_paint,
                    &runs_dispatch,
                    &runs_hit_test,
                    &runs_pointer_move_dispatch,
                    &runs_pointer_move_hit_test,
                    &runs_pointer_move_global_changes,
                    script_worst.as_ref(),
                );
            } else {
                let total = summarize_times_us(&runs_total);
                let layout = summarize_times_us(&runs_layout);
                let solve = summarize_times_us(&runs_solve);
                let prepaint = summarize_times_us(&runs_prepaint);
                let paint = summarize_times_us(&runs_paint);
                let dispatch = summarize_times_us(&runs_dispatch);
                let hit_test = summarize_times_us(&runs_hit_test);
                reporting::print_perf_repeat_summary(
                    src.as_path(),
                    sort,
                    repeat,
                    &total,
                    &layout,
                    &solve,
                    &prepaint,
                    &paint,
                    &dispatch,
                    &hit_test,
                );
            }

            let max_total = *runs_total.iter().max().unwrap_or(&0);
            let max_layout = *runs_layout.iter().max().unwrap_or(&0);
            let max_solve = *runs_solve.iter().max().unwrap_or(&0);

            let mut sorted_total = runs_total.clone();
            sorted_total.sort_unstable();
            let p90_total = percentile_nearest_rank_sorted(&sorted_total, 0.90);
            let p95_total = percentile_nearest_rank_sorted(&sorted_total, 0.95);

            let mut sorted_layout = runs_layout.clone();
            sorted_layout.sort_unstable();
            let p90_layout = percentile_nearest_rank_sorted(&sorted_layout, 0.90);
            let p95_layout = percentile_nearest_rank_sorted(&sorted_layout, 0.95);

            let mut sorted_solve = runs_solve.clone();
            sorted_solve.sort_unstable();
            let p90_solve = percentile_nearest_rank_sorted(&sorted_solve, 0.90);
            let p95_solve = percentile_nearest_rank_sorted(&sorted_solve, 0.95);

            let max_frame_p95_total = *runs_frame_p95_total.iter().max().unwrap_or(&0);
            let max_frame_p95_layout = *runs_frame_p95_layout.iter().max().unwrap_or(&0);
            let max_frame_p95_solve = *runs_frame_p95_solve.iter().max().unwrap_or(&0);

            let mut sorted_frame_p95_total = runs_frame_p95_total.clone();
            sorted_frame_p95_total.sort_unstable();
            let p90_frame_p95_total = percentile_nearest_rank_sorted(&sorted_frame_p95_total, 0.90);
            let p95_frame_p95_total = percentile_nearest_rank_sorted(&sorted_frame_p95_total, 0.95);

            let mut sorted_frame_p95_layout = runs_frame_p95_layout.clone();
            sorted_frame_p95_layout.sort_unstable();
            let p90_frame_p95_layout =
                percentile_nearest_rank_sorted(&sorted_frame_p95_layout, 0.90);
            let p95_frame_p95_layout =
                percentile_nearest_rank_sorted(&sorted_frame_p95_layout, 0.95);

            let mut sorted_frame_p95_solve = runs_frame_p95_solve.clone();
            sorted_frame_p95_solve.sort_unstable();
            let p90_frame_p95_solve = percentile_nearest_rank_sorted(&sorted_frame_p95_solve, 0.90);
            let p95_frame_p95_solve = percentile_nearest_rank_sorted(&sorted_frame_p95_solve, 0.95);
            let max_pointer_move_dispatch = *runs_pointer_move_dispatch.iter().max().unwrap_or(&0);
            let max_pointer_move_hit_test = *runs_pointer_move_hit_test.iter().max().unwrap_or(&0);
            let max_pointer_move_global_changes =
                *runs_pointer_move_global_changes.iter().max().unwrap_or(&0);
            let max_run_paint_cache_hit_test_only_replay_allowed_max =
                *runs_paint_cache_hit_test_only_replay_allowed_max
                    .iter()
                    .max()
                    .unwrap_or(&0);
            let max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max =
                *runs_paint_cache_hit_test_only_replay_rejected_key_mismatch_max
                    .iter()
                    .max()
                    .unwrap_or(&0);
            let max_renderer_encode_scene_us =
                *runs_renderer_encode_scene_us.iter().max().unwrap_or(&0);
            let max_renderer_upload_us = *runs_renderer_upload_us.iter().max().unwrap_or(&0);
            let max_renderer_record_passes_us =
                *runs_renderer_record_passes_us.iter().max().unwrap_or(&0);
            let max_renderer_encoder_finish_us =
                *runs_renderer_encoder_finish_us.iter().max().unwrap_or(&0);
            let max_renderer_prepare_text_us =
                *runs_renderer_prepare_text_us.iter().max().unwrap_or(&0);
            let max_renderer_prepare_svg_us =
                *runs_renderer_prepare_svg_us.iter().max().unwrap_or(&0);

            let mut sorted_renderer_encode_scene_us = runs_renderer_encode_scene_us.clone();
            sorted_renderer_encode_scene_us.sort_unstable();
            let p90_renderer_encode_scene_us =
                percentile_nearest_rank_sorted(&sorted_renderer_encode_scene_us, 0.90);
            let p95_renderer_encode_scene_us =
                percentile_nearest_rank_sorted(&sorted_renderer_encode_scene_us, 0.95);

            let mut sorted_renderer_upload_us = runs_renderer_upload_us.clone();
            sorted_renderer_upload_us.sort_unstable();
            let p90_renderer_upload_us =
                percentile_nearest_rank_sorted(&sorted_renderer_upload_us, 0.90);
            let p95_renderer_upload_us =
                percentile_nearest_rank_sorted(&sorted_renderer_upload_us, 0.95);

            let mut sorted_renderer_record_passes_us = runs_renderer_record_passes_us.clone();
            sorted_renderer_record_passes_us.sort_unstable();
            let p90_renderer_record_passes_us =
                percentile_nearest_rank_sorted(&sorted_renderer_record_passes_us, 0.90);
            let p95_renderer_record_passes_us =
                percentile_nearest_rank_sorted(&sorted_renderer_record_passes_us, 0.95);

            let mut sorted_renderer_encoder_finish_us = runs_renderer_encoder_finish_us.clone();
            sorted_renderer_encoder_finish_us.sort_unstable();
            let p90_renderer_encoder_finish_us =
                percentile_nearest_rank_sorted(&sorted_renderer_encoder_finish_us, 0.90);
            let p95_renderer_encoder_finish_us =
                percentile_nearest_rank_sorted(&sorted_renderer_encoder_finish_us, 0.95);

            let mut sorted_renderer_prepare_text_us = runs_renderer_prepare_text_us.clone();
            sorted_renderer_prepare_text_us.sort_unstable();
            let p90_renderer_prepare_text_us =
                percentile_nearest_rank_sorted(&sorted_renderer_prepare_text_us, 0.90);
            let p95_renderer_prepare_text_us =
                percentile_nearest_rank_sorted(&sorted_renderer_prepare_text_us, 0.95);

            let mut sorted_renderer_prepare_svg_us = runs_renderer_prepare_svg_us.clone();
            sorted_renderer_prepare_svg_us.sort_unstable();
            let p90_renderer_prepare_svg_us =
                percentile_nearest_rank_sorted(&sorted_renderer_prepare_svg_us, 0.90);
            let p95_renderer_prepare_svg_us =
                percentile_nearest_rank_sorted(&sorted_renderer_prepare_svg_us, 0.95);
            let pointer_move_frames_present = runs_json.iter().any(|run| {
                run.get("pointer_move_frames_present")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            });
            let script_key = normalize_repo_relative_path(&workspace_root, &src);

            if perf_baseline_out.is_some() {
                let policy = seed_policy
                    .as_ref()
                    .ok_or_else(|| "internal error: missing seed policy".to_string())?;

                let seed_total = policy.seed_for(&script_key, PerfSeedMetric::TopTotalTimeUs);
                let seed_layout = policy.seed_for(&script_key, PerfSeedMetric::TopLayoutTimeUs);
                let seed_solve =
                    policy.seed_for(&script_key, PerfSeedMetric::TopLayoutEngineSolveTimeUs);
                let seed_frame_p95_total =
                    policy.seed_for(&script_key, PerfSeedMetric::FrameP95TotalTimeUs);
                let seed_frame_p95_layout =
                    policy.seed_for(&script_key, PerfSeedMetric::FrameP95LayoutTimeUs);
                let seed_frame_p95_solve =
                    policy.seed_for(&script_key, PerfSeedMetric::FrameP95LayoutEngineSolveTimeUs);

                let seed_total_value = match seed_total {
                    PerfBaselineSeed::Max => max_total,
                    PerfBaselineSeed::P90 => p90_total,
                    PerfBaselineSeed::P95 => p95_total,
                };
                let seed_layout_value = match seed_layout {
                    PerfBaselineSeed::Max => max_layout,
                    PerfBaselineSeed::P90 => p90_layout,
                    PerfBaselineSeed::P95 => p95_layout,
                };
                let seed_solve_value = match seed_solve {
                    PerfBaselineSeed::Max => max_solve,
                    PerfBaselineSeed::P90 => p90_solve,
                    PerfBaselineSeed::P95 => p95_solve,
                };
                let seed_frame_p95_total_value = match seed_frame_p95_total {
                    PerfBaselineSeed::Max => max_frame_p95_total,
                    PerfBaselineSeed::P90 => p90_frame_p95_total,
                    PerfBaselineSeed::P95 => p95_frame_p95_total,
                };
                let seed_frame_p95_layout_value = match seed_frame_p95_layout {
                    PerfBaselineSeed::Max => max_frame_p95_layout,
                    PerfBaselineSeed::P90 => p90_frame_p95_layout,
                    PerfBaselineSeed::P95 => p95_frame_p95_layout,
                };
                let seed_frame_p95_solve_value = match seed_frame_p95_solve {
                    PerfBaselineSeed::Max => max_frame_p95_solve,
                    PerfBaselineSeed::P90 => p90_frame_p95_solve,
                    PerfBaselineSeed::P95 => p95_frame_p95_solve,
                };

                let thr_total =
                    apply_perf_baseline_headroom(seed_total_value, perf_baseline_headroom_pct);
                let thr_layout =
                    apply_perf_baseline_headroom(seed_layout_value, perf_baseline_headroom_pct);
                let thr_solve =
                    apply_perf_baseline_headroom(seed_solve_value, perf_baseline_headroom_pct);
                let wants_frame_p95_thresholds = suite_name
                    .as_deref()
                    .is_some_and(|name| name.contains("typical"));
                let thr_frame_p95_total = wants_frame_p95_thresholds.then(|| {
                    apply_perf_baseline_headroom(
                        seed_frame_p95_total_value,
                        perf_baseline_headroom_pct,
                    )
                });
                let thr_frame_p95_layout = wants_frame_p95_thresholds.then(|| {
                    apply_perf_baseline_headroom(
                        seed_frame_p95_layout_value,
                        perf_baseline_headroom_pct,
                    )
                });
                let thr_frame_p95_solve = wants_frame_p95_thresholds.then(|| {
                    apply_perf_baseline_headroom(
                        seed_frame_p95_solve_value,
                        perf_baseline_headroom_pct,
                    )
                });
                let thr_pointer_move_dispatch = apply_perf_baseline_headroom(
                    max_pointer_move_dispatch,
                    perf_baseline_headroom_pct,
                );
                let thr_pointer_move_hit_test = apply_perf_baseline_headroom(
                    max_pointer_move_hit_test,
                    perf_baseline_headroom_pct,
                );
                let thr_pointer_move_global_changes = apply_perf_baseline_headroom(
                    max_pointer_move_global_changes,
                    perf_baseline_headroom_pct,
                );
                let thr_min_run_paint_cache_hit_test_only_replay_allowed_max =
                    apply_perf_baseline_floor(
                        max_run_paint_cache_hit_test_only_replay_allowed_max,
                        perf_baseline_headroom_pct,
                    );
                let thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max =
                    apply_perf_baseline_headroom(
                        max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        perf_baseline_headroom_pct,
                    );
                let thr_renderer_encode_scene_us = apply_perf_baseline_headroom(
                    max_renderer_encode_scene_us,
                    perf_baseline_headroom_pct,
                );
                let thr_renderer_upload_us = apply_perf_baseline_headroom(
                    max_renderer_upload_us,
                    perf_baseline_headroom_pct,
                );
                let thr_renderer_record_passes_us = apply_perf_baseline_headroom(
                    max_renderer_record_passes_us,
                    perf_baseline_headroom_pct,
                );
                let thr_renderer_encoder_finish_us = apply_perf_baseline_headroom(
                    max_renderer_encoder_finish_us,
                    perf_baseline_headroom_pct,
                );
                let thr_renderer_prepare_text_us = apply_perf_baseline_headroom(
                    max_renderer_prepare_text_us,
                    perf_baseline_headroom_pct,
                );
                let thr_renderer_prepare_svg_us = apply_perf_baseline_headroom(
                    max_renderer_prepare_svg_us,
                    perf_baseline_headroom_pct,
                );

                baseline_rows::push_perf_baseline_row_repeat(
                    &mut perf_baseline_rows,
                    script_key.as_str(),
                    baseline_rows::TopTimesUs::new(max_total, max_layout, max_solve),
                    baseline_rows::TopTimesUs::new(
                        max_frame_p95_total,
                        max_frame_p95_layout,
                        max_frame_p95_solve,
                    ),
                    baseline_rows::PointerMoveMetrics::new(
                        max_pointer_move_dispatch,
                        max_pointer_move_hit_test,
                        max_pointer_move_global_changes,
                    ),
                    baseline_rows::PaintCacheReplayMetrics::new(
                        max_run_paint_cache_hit_test_only_replay_allowed_max,
                        max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    ),
                    baseline_rows::RendererTimesUs::new(
                        max_renderer_encode_scene_us,
                        max_renderer_upload_us,
                        max_renderer_record_passes_us,
                        max_renderer_encoder_finish_us,
                        max_renderer_prepare_text_us,
                        max_renderer_prepare_svg_us,
                    ),
                    baseline_rows::TopTimesUs::new(p90_total, p90_layout, p90_solve),
                    baseline_rows::TopTimesUs::new(
                        p90_frame_p95_total,
                        p90_frame_p95_layout,
                        p90_frame_p95_solve,
                    ),
                    baseline_rows::RendererTimesUs::new(
                        p90_renderer_encode_scene_us,
                        p90_renderer_upload_us,
                        p90_renderer_record_passes_us,
                        p90_renderer_encoder_finish_us,
                        p90_renderer_prepare_text_us,
                        p90_renderer_prepare_svg_us,
                    ),
                    baseline_rows::TopTimesUs::new(p95_total, p95_layout, p95_solve),
                    baseline_rows::TopTimesUs::new(
                        p95_frame_p95_total,
                        p95_frame_p95_layout,
                        p95_frame_p95_solve,
                    ),
                    baseline_rows::RendererTimesUs::new(
                        p95_renderer_encode_scene_us,
                        p95_renderer_upload_us,
                        p95_renderer_record_passes_us,
                        p95_renderer_encoder_finish_us,
                        p95_renderer_prepare_text_us,
                        p95_renderer_prepare_svg_us,
                    ),
                    seed_total,
                    seed_layout,
                    seed_solve,
                    seed_frame_p95_total,
                    seed_frame_p95_layout,
                    seed_frame_p95_solve,
                    seed_total_value,
                    seed_layout_value,
                    seed_solve_value,
                    seed_frame_p95_total_value,
                    seed_frame_p95_layout_value,
                    seed_frame_p95_solve_value,
                    wants_frame_p95_thresholds,
                    thr_total,
                    thr_layout,
                    thr_solve,
                    thr_frame_p95_total,
                    thr_frame_p95_layout,
                    thr_frame_p95_solve,
                    baseline_rows::PointerMoveMetrics::new(
                        thr_pointer_move_dispatch,
                        thr_pointer_move_hit_test,
                        thr_pointer_move_global_changes,
                    ),
                    thr_min_run_paint_cache_hit_test_only_replay_allowed_max,
                    thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    baseline_rows::RendererTimesUs::new(
                        thr_renderer_encode_scene_us,
                        thr_renderer_upload_us,
                        thr_renderer_record_passes_us,
                        thr_renderer_encoder_finish_us,
                        thr_renderer_prepare_text_us,
                        thr_renderer_prepare_svg_us,
                    ),
                );
            }

            if wants_perf_thresholds {
                let baseline_thresholds = perf_baseline
                    .as_ref()
                    .and_then(|b| b.thresholds_by_script.get(&script_key).copied())
                    .unwrap_or_default();
                let (thr_total, src_total) = resolve_threshold(
                    cli_thresholds.max_top_total_us,
                    baseline_thresholds.max_top_total_us,
                );
                let (thr_layout, src_layout) = resolve_threshold(
                    cli_thresholds.max_top_layout_us,
                    baseline_thresholds.max_top_layout_us,
                );
                let (thr_solve, src_solve) = resolve_threshold(
                    cli_thresholds.max_top_solve_us,
                    baseline_thresholds.max_top_solve_us,
                );
                let (thr_frame_p95_total, src_frame_p95_total) = resolve_threshold(
                    cli_thresholds.max_frame_p95_total_us,
                    baseline_thresholds.max_frame_p95_total_us,
                );
                let (thr_frame_p95_layout, src_frame_p95_layout) = resolve_threshold(
                    cli_thresholds.max_frame_p95_layout_us,
                    baseline_thresholds.max_frame_p95_layout_us,
                );
                let (thr_frame_p95_solve, src_frame_p95_solve) = resolve_threshold(
                    cli_thresholds.max_frame_p95_solve_us,
                    baseline_thresholds.max_frame_p95_solve_us,
                );
                let (thr_pointer_move_dispatch, src_pointer_move_dispatch) = resolve_threshold(
                    cli_thresholds.max_pointer_move_dispatch_us,
                    baseline_thresholds.max_pointer_move_dispatch_us,
                );
                let (thr_pointer_move_hit_test, src_pointer_move_hit_test) = resolve_threshold(
                    cli_thresholds.max_pointer_move_hit_test_us,
                    baseline_thresholds.max_pointer_move_hit_test_us,
                );
                let (thr_pointer_move_global_changes, src_pointer_move_global_changes) =
                    resolve_threshold(
                        cli_thresholds.max_pointer_move_global_changes,
                        baseline_thresholds.max_pointer_move_global_changes,
                    );
                let (
                    thr_paint_cache_hit_test_only_replay_allowed_max,
                    src_paint_cache_hit_test_only_replay_allowed_max,
                ) = resolve_threshold(
                    cli_thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max,
                    baseline_thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max,
                );
                let (
                    thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                ) = resolve_threshold(
                    cli_thresholds
                        .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    baseline_thresholds
                        .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                );

                let observed_total = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_total,
                    PerfThresholdAggregate::P90 => p90_total,
                    PerfThresholdAggregate::P95 => p95_total,
                };
                let observed_layout = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_layout,
                    PerfThresholdAggregate::P90 => p90_layout,
                    PerfThresholdAggregate::P95 => p95_layout,
                };
                let observed_solve = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_solve,
                    PerfThresholdAggregate::P90 => p90_solve,
                    PerfThresholdAggregate::P95 => p95_solve,
                };
                let observed_frame_p95_total = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_frame_p95_total,
                    PerfThresholdAggregate::P90 => p90_frame_p95_total,
                    PerfThresholdAggregate::P95 => p95_frame_p95_total,
                };
                let observed_frame_p95_layout = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_frame_p95_layout,
                    PerfThresholdAggregate::P90 => p90_frame_p95_layout,
                    PerfThresholdAggregate::P95 => p95_frame_p95_layout,
                };
                let observed_frame_p95_solve = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_frame_p95_solve,
                    PerfThresholdAggregate::P90 => p90_frame_p95_solve,
                    PerfThresholdAggregate::P95 => p95_frame_p95_solve,
                };
                let observed_renderer_encode_scene_us = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_renderer_encode_scene_us,
                    PerfThresholdAggregate::P90 => p90_renderer_encode_scene_us,
                    PerfThresholdAggregate::P95 => p95_renderer_encode_scene_us,
                };
                let observed_renderer_upload_us = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_renderer_upload_us,
                    PerfThresholdAggregate::P90 => p90_renderer_upload_us,
                    PerfThresholdAggregate::P95 => p95_renderer_upload_us,
                };
                let observed_renderer_record_passes_us = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_renderer_record_passes_us,
                    PerfThresholdAggregate::P90 => p90_renderer_record_passes_us,
                    PerfThresholdAggregate::P95 => p95_renderer_record_passes_us,
                };
                let observed_renderer_encoder_finish_us = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_renderer_encoder_finish_us,
                    PerfThresholdAggregate::P90 => p90_renderer_encoder_finish_us,
                    PerfThresholdAggregate::P95 => p95_renderer_encoder_finish_us,
                };
                let observed_renderer_prepare_text_us = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_renderer_prepare_text_us,
                    PerfThresholdAggregate::P90 => p90_renderer_prepare_text_us,
                    PerfThresholdAggregate::P95 => p95_renderer_prepare_text_us,
                };
                let observed_renderer_prepare_svg_us = match perf_threshold_agg {
                    PerfThresholdAggregate::Max => max_renderer_prepare_svg_us,
                    PerfThresholdAggregate::P90 => p90_renderer_prepare_svg_us,
                    PerfThresholdAggregate::P95 => p95_renderer_prepare_svg_us,
                };
                thresholds::push_repeat_threshold_row_and_failures(
                    &mut perf_threshold_rows,
                    &mut perf_threshold_failures,
                    thresholds::RepeatThresholdInputs {
                        script_key: script_key.as_str(),
                        sort,
                        repeat,
                        runs_json: &runs_json,
                        perf_threshold_agg,
                        cli_thresholds,
                        baseline_thresholds,
                        observed_total,
                        max_total,
                        p95_total,
                        sorted_total: &sorted_total,
                        p90_total,
                        observed_layout,
                        max_layout,
                        p95_layout,
                        sorted_layout: &sorted_layout,
                        p90_layout,
                        observed_solve,
                        max_solve,
                        p95_solve,
                        sorted_solve: &sorted_solve,
                        p90_solve,
                        observed_frame_p95_total,
                        max_frame_p95_total,
                        p95_frame_p95_total,
                        sorted_frame_p95_total: &sorted_frame_p95_total,
                        p90_frame_p95_total,
                        observed_frame_p95_layout,
                        max_frame_p95_layout,
                        p95_frame_p95_layout,
                        sorted_frame_p95_layout: &sorted_frame_p95_layout,
                        p90_frame_p95_layout,
                        observed_frame_p95_solve,
                        max_frame_p95_solve,
                        p95_frame_p95_solve,
                        sorted_frame_p95_solve: &sorted_frame_p95_solve,
                        p90_frame_p95_solve,
                        pointer_move_frames_present,
                        max_pointer_move_dispatch,
                        max_pointer_move_hit_test,
                        max_pointer_move_global_changes,
                        max_run_paint_cache_hit_test_only_replay_allowed_max,
                        max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        observed_renderer_encode_scene_us,
                        max_renderer_encode_scene_us,
                        p95_renderer_encode_scene_us,
                        observed_renderer_upload_us,
                        max_renderer_upload_us,
                        p95_renderer_upload_us,
                        observed_renderer_record_passes_us,
                        max_renderer_record_passes_us,
                        p95_renderer_record_passes_us,
                        observed_renderer_encoder_finish_us,
                        max_renderer_encoder_finish_us,
                        p95_renderer_encoder_finish_us,
                        observed_renderer_prepare_text_us,
                        max_renderer_prepare_text_us,
                        p95_renderer_prepare_text_us,
                        observed_renderer_prepare_svg_us,
                        max_renderer_prepare_svg_us,
                        p95_renderer_prepare_svg_us,
                        script_worst: &script_worst,
                        script_worst_layout: &script_worst_layout,
                        script_worst_solve: &script_worst_solve,
                        thr_total,
                        src_total,
                        thr_layout,
                        src_layout,
                        thr_solve,
                        src_solve,
                        thr_frame_p95_total,
                        src_frame_p95_total,
                        thr_frame_p95_layout,
                        src_frame_p95_layout,
                        thr_frame_p95_solve,
                        src_frame_p95_solve,
                        thr_pointer_move_dispatch,
                        src_pointer_move_dispatch,
                        thr_pointer_move_hit_test,
                        src_pointer_move_hit_test,
                        thr_pointer_move_global_changes,
                        src_pointer_move_global_changes,
                        thr_paint_cache_hit_test_only_replay_allowed_max:
                            thr_paint_cache_hit_test_only_replay_allowed_max,
                        src_paint_cache_hit_test_only_replay_allowed_max,
                        thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max:
                            thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    },
                );
            }
        }
    }

    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);

    if let Some(test_id) = check_pixels_changed_test_id.as_deref() {
        stats::check_out_dir_for_pixels_changed(&resolved_out_dir, test_id, warmup_frames)?;
    }
    if let Some(test_id) = check_pixels_unchanged_test_id.as_deref() {
        stats::check_out_dir_for_pixels_unchanged(&resolved_out_dir, test_id, warmup_frames)?;
    }

    if let Some(path) = perf_baseline_out.as_ref() {
        let out_path = path;
        let policy = seed_policy
            .as_ref()
            .ok_or_else(|| "internal error: missing seed policy".to_string())?;
        outputs::write_perf_baseline_json(
            out_path,
            warmup_frames,
            sort,
            repeat,
            perf_baseline_headroom_pct,
            policy.threshold_seed_policy_json(),
            &perf_baseline_rows,
            stats_json,
        )?;
    }

    let mut layout_perf_summary: Option<serde_json::Value> = None;
    let mut layout_perf_summary_path: Option<PathBuf> = None;
    if let Some((_us, _src, bundle_path)) = overall_worst.as_ref() {
        let bundle_dir = crate::resolve_bundle_root_dir(bundle_path).unwrap_or_else(|_| {
            bundle_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        });
        let summary =
            crate::layout_perf_summary::layout_perf_summary_v1_from_bundle_path_best_effort(
                bundle_path.as_path(),
                &bundle_dir,
                warmup_frames,
                crate::layout_perf_summary::DEFAULT_LAYOUT_PERF_SUMMARY_TOP,
            );
        let out_path = resolved_out_dir.join("layout.perf.summary.v1.json");
        let _ = crate::util::write_json_value(&out_path, &summary);
        layout_perf_summary = Some(summary);
        layout_perf_summary_path = Some(out_path);
    }

    let mut perf_threshold_failure: Option<(usize, PathBuf)> = None;
    if wants_perf_thresholds {
        let baseline_summary = perf_baseline.as_ref().map(|b| {
            serde_json::json!({
                "path": b.path.display().to_string(),
                "scripts": b.thresholds_by_script.len(),
            })
        });
        let out_path = outputs::write_perf_thresholds_json(
            &resolved_out_dir,
            warmup_frames,
            perf_threshold_agg,
            &perf_suite_prewarm_scripts,
            &perf_suite_prelude_scripts,
            suite_prelude_each_run,
            &cli_thresholds,
            baseline_summary,
            layout_perf_summary.clone(),
            layout_perf_summary_path
                .as_ref()
                .map(|p| p.display().to_string()),
            &perf_threshold_rows,
            &perf_threshold_failures,
        );
        if !perf_threshold_failures.is_empty() {
            perf_threshold_failure = Some((perf_threshold_failures.len(), out_path.clone()));
        }
    }

    let mut perf_hint_failure: Option<(usize, PathBuf)> = None;
    if wants_perf_hints {
        let out_path = outputs::write_perf_hints_json(
            &resolved_out_dir,
            warmup_frames,
            &perf_hint_gate_opts,
            layout_perf_summary.clone(),
            layout_perf_summary_path
                .as_ref()
                .map(|p| p.display().to_string()),
            &perf_hint_rows,
            &perf_hint_failures,
        );
        if !perf_hint_failures.is_empty() {
            perf_hint_failure = Some((perf_hint_failures.len(), out_path.clone()));
        }
    }

    if launched_by_fretboard {
        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
    }

    if stats_json {
        outputs::print_perf_stats_stdout_json(
            sort,
            repeat,
            &perf_json_rows,
            overall_worst.as_ref(),
        );
    } else if let Some((us, src, bundle)) = overall_worst {
        println!(
            "PERF worst overall: {} us={} bundle={}",
            src.display(),
            us,
            bundle.display()
        );
    }

    if let Some((failures, evidence)) = perf_threshold_failure {
        eprintln!(
            "PERF threshold gate failed (failures={}, evidence={})",
            failures,
            evidence.display()
        );
        std::process::exit(1);
    }
    if let Some((failures, evidence)) = perf_hint_failure {
        eprintln!(
            "PERF hints gate failed (failures={}, evidence={})",
            failures,
            evidence.display()
        );
        std::process::exit(1);
    }

    std::process::exit(0);
}

fn ensure_perf_fs_transport_connected(
    connected_fs: &mut Option<ConnectedToolingTransport>,
    use_devtools_ws: bool,
    fs_transport_cfg: &crate::transport::FsDiagTransportConfig,
    ready_path: &Path,
    require_ready: bool,
    timeout_ms: u64,
    poll_ms: u64,
    script_result_path: &Path,
) -> Result<(), String> {
    if use_devtools_ws {
        return Ok(());
    }
    if connected_fs.is_some() {
        return Ok(());
    }

    match connect_filesystem_tooling(
        fs_transport_cfg,
        ready_path,
        require_ready,
        timeout_ms,
        poll_ms,
    ) {
        Ok(v) => {
            *connected_fs = Some(v);
            Ok(())
        }
        Err(err) => {
            write_tooling_failure_script_result(
                script_result_path,
                "tooling.connect.failed",
                &err,
                "tooling_error",
                Some("connect_filesystem_tooling".to_string()),
            );
            Err(err)
        }
    }
}
