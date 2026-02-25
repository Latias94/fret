use super::*;

#[path = "diag_perf/aux_scripts.rs"]
mod aux_scripts;
#[path = "diag_perf/run_script.rs"]
mod run_script;
#[path = "diag_perf/reporting.rs"]
mod reporting;

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
    pub devtools_session_id: Option<String>,
    pub devtools_token: Option<String>,
    pub devtools_ws_url: Option<String>,
    pub keep_open: bool,
    pub launch: Option<Vec<String>>,
    pub launch_env: Vec<(String, String)>,
    pub launch_high_priority: bool,
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
        devtools_session_id,
        devtools_token,
        devtools_ws_url,
        keep_open,
        launch,
        launch_env,
        launch_high_priority,
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
            "missing suite name or script paths (try: fretboard diag perf ui-gallery)".to_string(),
        );
    }

    let mut suite_name: Option<String> = None;
    let scripts: Vec<PathBuf> = if rest.len() == 1 {
        let name = rest[0].as_str();
        if let Some(paths) = perf_seed_policy::scripts_for_perf_suite_name(name) {
            suite_name = Some(name.to_string());
            paths
                .iter()
                .map(|p| resolve_path(&workspace_root, PathBuf::from(*p)))
                .collect()
        } else if name == "ui-gallery" {
            suite_name = Some(name.to_string());
            [
                "tools/diag-scripts/ui-gallery-overlay-torture.json",
                "tools/diag-scripts/ui-gallery-dropdown-open-select.json",
                "tools/diag-scripts/ui-gallery-context-menu-right-click.json",
                "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json",
                "tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json",
                "tools/diag-scripts/ui-gallery-virtual-list-torture.json",
                "tools/diag-scripts/ui-gallery-material3-tabs-switch-perf.json",
                "tools/diag-scripts/ui-gallery-view-cache-toggle-perf.json",
                "tools/diag-scripts/ui-gallery-window-resize-stress.json",
            ]
            .into_iter()
            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
            .collect()
        } else if name == "ui-gallery-steady" {
            suite_name = Some(name.to_string());
            [
                "tools/diag-scripts/ui-gallery-overlay-torture-steady.json",
                "tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json",
                "tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json",
                "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json",
                "tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json",
                "tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json",
                "tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json",
                "tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json",
                "tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json",
                "tools/diag-scripts/ui-gallery-window-resize-stress-steady.json",
            ]
            .into_iter()
            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
            .collect()
        } else if name == "ui-resize-probes" {
            suite_name = Some(name.to_string());
            [
                "tools/diag-scripts/ui-gallery-window-resize-stress-steady.json",
                "tools/diag-scripts/ui-gallery-window-resize-drag-jitter-steady.json",
            ]
            .into_iter()
            .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
            .collect()
        } else if name == "ui-code-editor-resize-probes" {
            suite_name = Some(name.to_string());
            ["tools/diag-scripts/ui-gallery-code-editor-window-resize-drag-jitter-steady.json"]
                .into_iter()
                .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                .collect()
        } else if name == "extras-marquee-steady" {
            suite_name = Some(name.to_string());
            ["tools/diag-scripts/extras-marquee-steady.json"]
                .into_iter()
                .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                .collect()
        } else {
            vec![resolve_path(&workspace_root, PathBuf::from(name))]
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

    let run_suite_aux_script_must_pass =
        |src: &PathBuf, child: &mut Option<LaunchedDemo>| -> Result<(), String> {
            aux_scripts::run_suite_aux_script_must_pass(
                src,
                child,
                use_devtools_ws,
                connected_ws.as_ref(),
                &workspace_root,
                &resolved_out_dir,
                &resolved_exit_path,
                reuse_process,
                &resolved_script_path,
                &resolved_script_trigger_path,
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

    if launched_by_fretboard && !reuse_process_per_script {
        child = maybe_launch_demo(
            &launch,
            &perf_launch_env,
            &workspace_root,
            &resolved_out_dir,
            &resolved_ready_path,
            &resolved_exit_path,
            false,
            timeout_ms,
            poll_ms,
            launch_high_priority,
        )?;
    }

    if reuse_process && !reuse_process_per_script && !perf_suite_prewarm_scripts.is_empty() {
        for prewarm in &perf_suite_prewarm_scripts {
            run_suite_aux_script_must_pass(prewarm, &mut child)?;
        }
    }

    for (script_index, src) in scripts.into_iter().enumerate() {
        if reuse_process_per_script && launched_by_fretboard && script_index > 0 {
            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
            child = None;
        }
        if reuse_process_per_script && launched_by_fretboard && child.is_none() {
            child = maybe_launch_demo(
                &launch,
                &perf_launch_env,
                &workspace_root,
                &resolved_out_dir,
                &resolved_ready_path,
                &resolved_exit_path,
                false,
                timeout_ms,
                poll_ms,
                launch_high_priority,
            )?;
            if !perf_suite_prewarm_scripts.is_empty() {
                for prewarm in &perf_suite_prewarm_scripts {
                    run_suite_aux_script_must_pass(prewarm, &mut child)?;
                }
            }
        }

        if repeat == 1 {
            if !reuse_process {
                child = maybe_launch_demo(
                    &launch,
                    &perf_launch_env,
                    &workspace_root,
                    &resolved_out_dir,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    false,
                    timeout_ms,
                    poll_ms,
                    launch_high_priority,
                )?;
            }

            if !reuse_process {
                clear_script_result_files(
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                );
            }

            if !reuse_process && !perf_suite_prewarm_scripts.is_empty() {
                for prewarm in &perf_suite_prewarm_scripts {
                    run_suite_aux_script_must_pass(prewarm, &mut child)?;
                }
            }
            if !perf_suite_prelude_scripts.is_empty() {
                for prelude in &perf_suite_prelude_scripts {
                    run_suite_aux_script_must_pass(prelude, &mut child)?;
                }
            }

            let script_key = normalize_repo_relative_path(&workspace_root, &src);
            let bundle_path = run_script::run_perf_script_and_resolve_bundle_artifact_path(
                &src,
                script_key.as_str(),
                &mut child,
                use_devtools_ws,
                connected_ws.as_ref(),
                &resolved_out_dir,
                &resolved_exit_path,
                &resolved_script_path,
                &resolved_script_trigger_path,
                &resolved_script_result_path,
                &resolved_script_result_trigger_path,
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
                    perf_hint_rows.push(serde_json::json!({
                        "script": script_key.clone(),
                        "sort": sort.as_str(),
                        "repeat": repeat,
                        "run_index": 0,
                        "bundle": bundle_path.display().to_string(),
                        "warmup_frames": report_warmup_frames,
                        "hints": hints,
                        "unit_costs": unit_costs,
                        "worst": worst,
                        "trace_chrome_json_path": trace_chrome_json_path,
                        "failures": failures,
                    }));
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
                let top_dispatch_events = top.map(|r| r.dispatch_events).unwrap_or(0);
                let top_hit_test_queries = top.map(|r| r.hit_test_queries).unwrap_or(0);
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
                let top_hit_test_bounds_tree_queries =
                    top.map(|r| r.hit_test_bounds_tree_queries).unwrap_or(0);
                let top_hit_test_bounds_tree_disabled =
                    top.map(|r| r.hit_test_bounds_tree_disabled).unwrap_or(0);
                let top_hit_test_bounds_tree_misses =
                    top.map(|r| r.hit_test_bounds_tree_misses).unwrap_or(0);
                let top_hit_test_bounds_tree_hits =
                    top.map(|r| r.hit_test_bounds_tree_hits).unwrap_or(0);
                let top_hit_test_bounds_tree_candidate_rejected = top
                    .map(|r| r.hit_test_bounds_tree_candidate_rejected)
                    .unwrap_or(0);
                let top_frame_arena_capacity_estimate_bytes = top
                    .map(|r| r.frame_arena_capacity_estimate_bytes)
                    .unwrap_or(0);
                let top_frame_arena_grow_events =
                    top.map(|r| r.frame_arena_grow_events).unwrap_or(0);
                let top_element_children_vec_pool_reuses =
                    top.map(|r| r.element_children_vec_pool_reuses).unwrap_or(0);
                let top_element_children_vec_pool_misses =
                    top.map(|r| r.element_children_vec_pool_misses).unwrap_or(0);
                let top_frame = top.map(|r| r.frame_id).unwrap_or(0);
                let top_tick = top.map(|r| r.tick_id).unwrap_or(0);
                let top_view_cache_contained_relayouts =
                    top.map(|r| r.view_cache_contained_relayouts).unwrap_or(0);
                let top_view_cache_roots_total = top.map(|r| r.view_cache_roots_total).unwrap_or(0);
                let top_view_cache_roots_reused =
                    top.map(|r| r.view_cache_roots_reused).unwrap_or(0);
                let top_view_cache_roots_first_mount =
                    top.map(|r| r.view_cache_roots_first_mount).unwrap_or(0);
                let top_view_cache_roots_node_recreated =
                    top.map(|r| r.view_cache_roots_node_recreated).unwrap_or(0);
                let top_view_cache_roots_cache_key_mismatch = top
                    .map(|r| r.view_cache_roots_cache_key_mismatch)
                    .unwrap_or(0);
                let top_view_cache_roots_not_marked_reuse_root = top
                    .map(|r| r.view_cache_roots_not_marked_reuse_root)
                    .unwrap_or(0);
                let top_view_cache_roots_needs_rerender =
                    top.map(|r| r.view_cache_roots_needs_rerender).unwrap_or(0);
                let top_view_cache_roots_layout_invalidated = top
                    .map(|r| r.view_cache_roots_layout_invalidated)
                    .unwrap_or(0);
                let top_view_cache_roots_manual =
                    top.map(|r| r.view_cache_roots_manual).unwrap_or(0);
                let top_cache_roots_contained_relayout =
                    top.map(|r| r.cache_roots_contained_relayout).unwrap_or(0);
                let top_set_children_barrier_writes =
                    top.map(|r| r.set_children_barrier_writes).unwrap_or(0);
                let top_barrier_relayouts_scheduled =
                    top.map(|r| r.barrier_relayouts_scheduled).unwrap_or(0);
                let top_barrier_relayouts_performed =
                    top.map(|r| r.barrier_relayouts_performed).unwrap_or(0);
                let top_virtual_list_visible_range_checks = top
                    .map(|r| r.virtual_list_visible_range_checks)
                    .unwrap_or(0);
                let top_virtual_list_visible_range_refreshes = top
                    .map(|r| r.virtual_list_visible_range_refreshes)
                    .unwrap_or(0);
                let top_renderer_tick_id = top.map(|r| r.renderer_tick_id).unwrap_or(0);
                let top_renderer_frame_id = top.map(|r| r.renderer_frame_id).unwrap_or(0);
                let top_renderer_encode_scene_us =
                    top.map(|r| r.renderer_encode_scene_us).unwrap_or(0);
                let top_renderer_prepare_text_us =
                    top.map(|r| r.renderer_prepare_text_us).unwrap_or(0);
                let top_renderer_prepare_svg_us =
                    top.map(|r| r.renderer_prepare_svg_us).unwrap_or(0);
                let top_renderer_draw_calls = top.map(|r| r.renderer_draw_calls).unwrap_or(0);
                let top_renderer_pipeline_switches =
                    top.map(|r| r.renderer_pipeline_switches).unwrap_or(0);
                let top_renderer_bind_group_switches =
                    top.map(|r| r.renderer_bind_group_switches).unwrap_or(0);
                let top_renderer_scissor_sets = top.map(|r| r.renderer_scissor_sets).unwrap_or(0);
                let top_renderer_scene_encoding_cache_misses = top
                    .map(|r| r.renderer_scene_encoding_cache_misses)
                    .unwrap_or(0);
                let top_renderer_material_quad_ops =
                    top.map(|r| r.renderer_material_quad_ops).unwrap_or(0);
                let top_renderer_material_sampled_quad_ops = top
                    .map(|r| r.renderer_material_sampled_quad_ops)
                    .unwrap_or(0);
                let top_renderer_material_distinct =
                    top.map(|r| r.renderer_material_distinct).unwrap_or(0);
                let top_renderer_material_unknown_ids =
                    top.map(|r| r.renderer_material_unknown_ids).unwrap_or(0);
                let top_renderer_material_degraded_due_to_budget = top
                    .map(|r| r.renderer_material_degraded_due_to_budget)
                    .unwrap_or(0);
                let top_renderer_text_atlas_upload_bytes =
                    top.map(|r| r.renderer_text_atlas_upload_bytes).unwrap_or(0);
                let top_renderer_text_atlas_evicted_pages = top
                    .map(|r| r.renderer_text_atlas_evicted_pages)
                    .unwrap_or(0);
                let top_renderer_svg_upload_bytes =
                    top.map(|r| r.renderer_svg_upload_bytes).unwrap_or(0);
                let top_renderer_image_upload_bytes =
                    top.map(|r| r.renderer_image_upload_bytes).unwrap_or(0);
                let top_renderer_svg_raster_cache_misses =
                    top.map(|r| r.renderer_svg_raster_cache_misses).unwrap_or(0);
                let top_renderer_svg_raster_budget_evictions = top
                    .map(|r| r.renderer_svg_raster_budget_evictions)
                    .unwrap_or(0);
                let top_renderer_svg_raster_budget_bytes =
                    top.map(|r| r.renderer_svg_raster_budget_bytes).unwrap_or(0);
                let top_renderer_svg_rasters_live =
                    top.map(|r| r.renderer_svg_rasters_live).unwrap_or(0);
                let top_renderer_svg_standalone_bytes_live = top
                    .map(|r| r.renderer_svg_standalone_bytes_live)
                    .unwrap_or(0);
                let top_renderer_svg_mask_atlas_pages_live = top
                    .map(|r| r.renderer_svg_mask_atlas_pages_live)
                    .unwrap_or(0);
                let top_renderer_svg_mask_atlas_bytes_live = top
                    .map(|r| r.renderer_svg_mask_atlas_bytes_live)
                    .unwrap_or(0);
                let top_renderer_svg_mask_atlas_used_px =
                    top.map(|r| r.renderer_svg_mask_atlas_used_px).unwrap_or(0);
                let top_renderer_svg_mask_atlas_capacity_px = top
                    .map(|r| r.renderer_svg_mask_atlas_capacity_px)
                    .unwrap_or(0);
                let top_renderer_svg_raster_cache_hits =
                    top.map(|r| r.renderer_svg_raster_cache_hits).unwrap_or(0);
                let top_renderer_svg_mask_atlas_page_evictions = top
                    .map(|r| r.renderer_svg_mask_atlas_page_evictions)
                    .unwrap_or(0);
                let top_renderer_svg_mask_atlas_entries_evicted = top
                    .map(|r| r.renderer_svg_mask_atlas_entries_evicted)
                    .unwrap_or(0);
                let top_renderer_intermediate_budget_bytes = top
                    .map(|r| r.renderer_intermediate_budget_bytes)
                    .unwrap_or(0);
                let top_renderer_intermediate_in_use_bytes = top
                    .map(|r| r.renderer_intermediate_in_use_bytes)
                    .unwrap_or(0);
                let top_renderer_intermediate_peak_in_use_bytes = top
                    .map(|r| r.renderer_intermediate_peak_in_use_bytes)
                    .unwrap_or(0);
                let top_renderer_intermediate_release_targets = top
                    .map(|r| r.renderer_intermediate_release_targets)
                    .unwrap_or(0);
                let top_renderer_intermediate_pool_allocations = top
                    .map(|r| r.renderer_intermediate_pool_allocations)
                    .unwrap_or(0);
                let top_renderer_intermediate_pool_reuses = top
                    .map(|r| r.renderer_intermediate_pool_reuses)
                    .unwrap_or(0);
                let top_renderer_intermediate_pool_releases = top
                    .map(|r| r.renderer_intermediate_pool_releases)
                    .unwrap_or(0);
                let top_renderer_intermediate_pool_evictions = top
                    .map(|r| r.renderer_intermediate_pool_evictions)
                    .unwrap_or(0);
                let top_renderer_intermediate_pool_free_bytes = top
                    .map(|r| r.renderer_intermediate_pool_free_bytes)
                    .unwrap_or(0);
                let top_renderer_intermediate_pool_free_textures = top
                    .map(|r| r.renderer_intermediate_pool_free_textures)
                    .unwrap_or(0);

                if stats_json {
                    perf_json_rows.push(serde_json::json!({
                        "script": script_key.clone(),
                        "sort": sort.as_str(),
                        "top_total_time_us": top_total,
                        "top_layout_time_us": top_layout,
                        "top_layout_engine_solve_time_us": top_solve,
                        "top_layout_engine_solves": top_solves,
                        "top_prepaint_time_us": top_prepaint,
                        "top_paint_time_us": top_paint,
                        "top_dispatch_time_us": top_dispatch,
                        "top_hit_test_time_us": top_hit_test,
                        "top_dispatch_events": top_dispatch_events,
                        "top_hit_test_queries": top_hit_test_queries,
                        "pointer_move_frames_present": pointer_move_frames_present,
                        "pointer_move_frames_considered": pointer_move_frames_considered,
                        "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                        "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                        "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                        "top_hit_test_bounds_tree_queries": top_hit_test_bounds_tree_queries,
                        "top_hit_test_bounds_tree_disabled": top_hit_test_bounds_tree_disabled,
                        "top_hit_test_bounds_tree_misses": top_hit_test_bounds_tree_misses,
                        "top_hit_test_bounds_tree_hits": top_hit_test_bounds_tree_hits,
                        "top_hit_test_bounds_tree_candidate_rejected": top_hit_test_bounds_tree_candidate_rejected,
                        "top_frame_arena_capacity_estimate_bytes": top_frame_arena_capacity_estimate_bytes,
                        "top_frame_arena_grow_events": top_frame_arena_grow_events,
                        "top_element_children_vec_pool_reuses": top_element_children_vec_pool_reuses,
                        "top_element_children_vec_pool_misses": top_element_children_vec_pool_misses,
                        "top_tick_id": top_tick,
                        "top_frame_id": top_frame,
                        "top_view_cache_contained_relayouts": top_view_cache_contained_relayouts,
                        "top_view_cache_roots_total": top_view_cache_roots_total,
                        "top_view_cache_roots_reused": top_view_cache_roots_reused,
                        "top_view_cache_roots_first_mount": top_view_cache_roots_first_mount,
                        "top_view_cache_roots_node_recreated": top_view_cache_roots_node_recreated,
                        "top_view_cache_roots_cache_key_mismatch": top_view_cache_roots_cache_key_mismatch,
                        "top_view_cache_roots_not_marked_reuse_root": top_view_cache_roots_not_marked_reuse_root,
                        "top_view_cache_roots_needs_rerender": top_view_cache_roots_needs_rerender,
                        "top_view_cache_roots_layout_invalidated": top_view_cache_roots_layout_invalidated,
                        "top_view_cache_roots_manual": top_view_cache_roots_manual,
                        "top_cache_roots_contained_relayout": top_cache_roots_contained_relayout,
                        "top_set_children_barrier_writes": top_set_children_barrier_writes,
                        "top_barrier_relayouts_scheduled": top_barrier_relayouts_scheduled,
                        "top_barrier_relayouts_performed": top_barrier_relayouts_performed,
                        "top_virtual_list_visible_range_checks": top_virtual_list_visible_range_checks,
                        "top_virtual_list_visible_range_refreshes": top_virtual_list_visible_range_refreshes,
                        "top_renderer_tick_id": top_renderer_tick_id,
                        "top_renderer_frame_id": top_renderer_frame_id,
                        "top_renderer_encode_scene_us": top_renderer_encode_scene_us,
                        "top_renderer_prepare_text_us": top_renderer_prepare_text_us,
                        "top_renderer_prepare_svg_us": top_renderer_prepare_svg_us,
                        "top_renderer_draw_calls": top_renderer_draw_calls,
                        "top_renderer_pipeline_switches": top_renderer_pipeline_switches,
                        "top_renderer_bind_group_switches": top_renderer_bind_group_switches,
                        "top_renderer_scissor_sets": top_renderer_scissor_sets,
                        "top_renderer_scene_encoding_cache_misses": top_renderer_scene_encoding_cache_misses,
                        "top_renderer_material_quad_ops": top_renderer_material_quad_ops,
                        "top_renderer_material_sampled_quad_ops": top_renderer_material_sampled_quad_ops,
                        "top_renderer_material_distinct": top_renderer_material_distinct,
                        "top_renderer_material_unknown_ids": top_renderer_material_unknown_ids,
                        "top_renderer_material_degraded_due_to_budget": top_renderer_material_degraded_due_to_budget,
                        "top_renderer_text_atlas_upload_bytes": top_renderer_text_atlas_upload_bytes,
                        "top_renderer_text_atlas_evicted_pages": top_renderer_text_atlas_evicted_pages,
                        "top_renderer_svg_upload_bytes": top_renderer_svg_upload_bytes,
                        "top_renderer_image_upload_bytes": top_renderer_image_upload_bytes,
                        "top_renderer_svg_raster_cache_misses": top_renderer_svg_raster_cache_misses,
                        "top_renderer_svg_raster_budget_evictions": top_renderer_svg_raster_budget_evictions,
                        "top_renderer_svg_raster_budget_bytes": top_renderer_svg_raster_budget_bytes,
                        "top_renderer_svg_rasters_live": top_renderer_svg_rasters_live,
                        "top_renderer_svg_standalone_bytes_live": top_renderer_svg_standalone_bytes_live,
                        "top_renderer_svg_mask_atlas_pages_live": top_renderer_svg_mask_atlas_pages_live,
                        "top_renderer_svg_mask_atlas_bytes_live": top_renderer_svg_mask_atlas_bytes_live,
                        "top_renderer_svg_mask_atlas_used_px": top_renderer_svg_mask_atlas_used_px,
    	                                "top_renderer_svg_mask_atlas_capacity_px": top_renderer_svg_mask_atlas_capacity_px,
    	                                "top_renderer_svg_raster_cache_hits": top_renderer_svg_raster_cache_hits,
    	                                "top_renderer_svg_mask_atlas_page_evictions": top_renderer_svg_mask_atlas_page_evictions,
    	                                "top_renderer_svg_mask_atlas_entries_evicted": top_renderer_svg_mask_atlas_entries_evicted,
    	                                "top_renderer_intermediate_budget_bytes": top_renderer_intermediate_budget_bytes,
    	                                "top_renderer_intermediate_in_use_bytes": top_renderer_intermediate_in_use_bytes,
    	                                "top_renderer_intermediate_peak_in_use_bytes": top_renderer_intermediate_peak_in_use_bytes,
    	                                "top_renderer_intermediate_release_targets": top_renderer_intermediate_release_targets,
    	                                "top_renderer_intermediate_pool_allocations": top_renderer_intermediate_pool_allocations,
    	                                "top_renderer_intermediate_pool_reuses": top_renderer_intermediate_pool_reuses,
    	                                "top_renderer_intermediate_pool_releases": top_renderer_intermediate_pool_releases,
    	                                "top_renderer_intermediate_pool_evictions": top_renderer_intermediate_pool_evictions,
    	                                "top_renderer_intermediate_pool_free_bytes": top_renderer_intermediate_pool_free_bytes,
    	                                "top_renderer_intermediate_pool_free_textures": top_renderer_intermediate_pool_free_textures,
    	                                "bundle": bundle_path.display().to_string(),
    	                            }));
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

                    perf_baseline_rows.push(serde_json::json!({
                        "script": script_key.clone(),
                        "measured_max": {
                            "top_total_time_us": top_total,
                            "top_layout_time_us": top_layout,
                            "top_layout_engine_solve_time_us": top_solve,
                            "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                            "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                            "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                            "run_paint_cache_hit_test_only_replay_allowed_max": run_paint_cache_hit_test_only_replay_allowed_max,
                            "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            "renderer_encode_scene_us": report.max_renderer_encode_scene_us,
                            "renderer_upload_us": report.max_renderer_upload_us,
                            "renderer_record_passes_us": report.max_renderer_record_passes_us,
                            "renderer_encoder_finish_us": report.max_renderer_encoder_finish_us,
                            "renderer_prepare_text_us": report.max_renderer_prepare_text_us,
                            "renderer_prepare_svg_us": report.max_renderer_prepare_svg_us,
                        },
                        "measured_p90": {
                            "top_total_time_us": p90_total,
                            "top_layout_time_us": p90_layout,
                            "top_layout_engine_solve_time_us": p90_solve,
                        },
                        "measured_p95": {
                            "top_total_time_us": p95_total,
                            "top_layout_time_us": p95_layout,
                            "top_layout_engine_solve_time_us": p95_solve,
                        },
                        "threshold_seed": {
                            "top_total_time_us": seed_total_value,
                            "top_layout_time_us": seed_layout_value,
                            "top_layout_engine_solve_time_us": seed_solve_value,
                        },
                        "threshold_seed_source": {
                            "top_total_time_us": seed_total.as_str(),
                            "top_layout_time_us": seed_layout.as_str(),
                            "top_layout_engine_solve_time_us": seed_solve.as_str(),
                        },
                        "thresholds": {
                            "max_top_total_us": thr_total,
                            "max_top_layout_us": thr_layout,
                            "max_top_solve_us": thr_solve,
                            "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
                            "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
                            "max_pointer_move_global_changes": thr_pointer_move_global_changes,
                            "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_min_run_paint_cache_hit_test_only_replay_allowed_max,
                            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                            "max_renderer_encode_scene_us": thr_renderer_encode_scene_us,
                            "max_renderer_upload_us": thr_renderer_upload_us,
                            "max_renderer_record_passes_us": thr_renderer_record_passes_us,
                            "max_renderer_encoder_finish_us": thr_renderer_encoder_finish_us,
                            "max_renderer_prepare_text_us": thr_renderer_prepare_text_us,
                            "max_renderer_prepare_svg_us": thr_renderer_prepare_svg_us,
                        },
                    }));
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
                    let run = serde_json::json!({
                        "run_index": 0,
                        "top_total_time_us": top_total,
                        "top_layout_time_us": top_layout,
                        "top_layout_engine_solve_time_us": top_solve,
                        "top_layout_engine_solves": top_solves,
                        "pointer_move_frames_present": pointer_move_frames_present,
                        "pointer_move_frames_considered": pointer_move_frames_considered,
                        "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                        "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                        "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                        "run_paint_cache_hit_test_only_replay_allowed_max": run_paint_cache_hit_test_only_replay_allowed_max,
                        "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        "top_tick_id": top_tick,
                        "top_frame_id": top_frame,
                        "bundle": bundle_path.display().to_string(),
                    });
                    let row = serde_json::json!({
                        "script": script_key.clone(),
                        "sort": sort.as_str(),
                        "repeat": 1,
                        "runs": [run],
                        "observed_aggregate": perf_threshold_agg.as_str(),
                        "observed": {
                            "top_total_time_us": top_total,
                            "top_layout_time_us": top_layout,
                            "top_layout_engine_solve_time_us": top_solve,
                        },
                        "worst_run": {
                            "top_total_time_us": top_total,
                            "bundle": bundle_path.display().to_string(),
                            "trace_chrome": bundle_path
                                .parent()
                                .map(|dir| dir.join("trace.chrome.json"))
                                .filter(|p| p.is_file())
                                .map(|p| p.display().to_string()),
                        },
                        "max": {
                            "top_total_time_us": top_total,
                            "top_layout_time_us": top_layout,
                            "top_layout_engine_solve_time_us": top_solve,
                            "frame_p95_total_time_us": report.p95_total_time_us,
                            "frame_p95_layout_time_us": report.p95_layout_time_us,
                            "frame_p95_layout_engine_solve_time_us": report.p95_layout_engine_solve_time_us,
                            "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                            "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                            "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                            "run_paint_cache_hit_test_only_replay_allowed_max": run_paint_cache_hit_test_only_replay_allowed_max,
                            "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        },
                        "p50": {
                            "top_total_time_us": top_total,
                            "top_layout_time_us": top_layout,
                            "top_layout_engine_solve_time_us": top_solve,
                            "frame_p95_total_time_us": report.p95_total_time_us,
                            "frame_p95_layout_time_us": report.p95_layout_time_us,
                            "frame_p95_layout_engine_solve_time_us": report.p95_layout_engine_solve_time_us,
                        },
                        "p95": {
                            "top_total_time_us": top_total,
                            "top_layout_time_us": top_layout,
                            "top_layout_engine_solve_time_us": top_solve,
                            "frame_p95_total_time_us": report.p95_total_time_us,
                            "frame_p95_layout_time_us": report.p95_layout_time_us,
                            "frame_p95_layout_engine_solve_time_us": report.p95_layout_engine_solve_time_us,
                        },
                        "thresholds": {
                            "max_top_total_us": thr_total,
                            "max_top_layout_us": thr_layout,
                            "max_top_solve_us": thr_solve,
                            "max_frame_p95_total_us": thr_frame_p95_total,
                            "max_frame_p95_layout_us": thr_frame_p95_layout,
                            "max_frame_p95_solve_us": thr_frame_p95_solve,
                            "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
                            "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
                            "max_pointer_move_global_changes": thr_pointer_move_global_changes,
                            "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_paint_cache_hit_test_only_replay_allowed_max,
                            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        },
                        "threshold_sources": {
                            "max_top_total_us": src_total,
                            "max_top_layout_us": src_layout,
                            "max_top_solve_us": src_solve,
                            "max_frame_p95_total_us": src_frame_p95_total,
                            "max_frame_p95_layout_us": src_frame_p95_layout,
                            "max_frame_p95_solve_us": src_frame_p95_solve,
                            "max_pointer_move_dispatch_us": src_pointer_move_dispatch,
                            "max_pointer_move_hit_test_us": src_pointer_move_hit_test,
                            "max_pointer_move_global_changes": src_pointer_move_global_changes,
                            "min_run_paint_cache_hit_test_only_replay_allowed_max": src_paint_cache_hit_test_only_replay_allowed_max,
                            "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        },
                    });
                    perf_threshold_rows.push(row);
                    perf_threshold_failures.extend(scan_perf_threshold_failures(
                        script_key.as_str(),
                        sort,
                        perf_threshold_agg,
                        cli_thresholds,
                        baseline_thresholds,
                        top_total,
                        top_total,
                        top_total,
                        top_layout,
                        top_layout,
                        top_layout,
                        top_solve,
                        top_solve,
                        top_solve,
                        report.p95_total_time_us,
                        report.p95_total_time_us,
                        report.p95_total_time_us,
                        report.p95_layout_time_us,
                        report.p95_layout_time_us,
                        report.p95_layout_time_us,
                        report.p95_layout_engine_solve_time_us,
                        report.p95_layout_engine_solve_time_us,
                        report.p95_layout_engine_solve_time_us,
                        pointer_move_frames_present,
                        pointer_move_max_dispatch_time_us,
                        pointer_move_max_hit_test_time_us,
                        pointer_move_snapshots_with_global_changes,
                        run_paint_cache_hit_test_only_replay_allowed_max,
                        run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        report.max_renderer_encode_scene_us,
                        report.max_renderer_encode_scene_us,
                        report.max_renderer_encode_scene_us,
                        report.max_renderer_upload_us,
                        report.max_renderer_upload_us,
                        report.max_renderer_upload_us,
                        report.max_renderer_record_passes_us,
                        report.max_renderer_record_passes_us,
                        report.max_renderer_record_passes_us,
                        report.max_renderer_encoder_finish_us,
                        report.max_renderer_encoder_finish_us,
                        report.max_renderer_encoder_finish_us,
                        report.max_renderer_prepare_text_us,
                        report.max_renderer_prepare_text_us,
                        report.max_renderer_prepare_text_us,
                        report.max_renderer_prepare_svg_us,
                        report.max_renderer_prepare_svg_us,
                        report.max_renderer_prepare_svg_us,
                        Some(bundle_path.as_path()),
                        Some(0),
                        None,
                        None,
                        None,
                        None,
                    ));
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
                    &resolved_out_dir,
                    &resolved_ready_path,
                    &resolved_exit_path,
                    false,
                    timeout_ms,
                    poll_ms,
                    launch_high_priority,
                )?;
            }

            if !reuse_process {
                clear_script_result_files(
                    &resolved_script_result_path,
                    &resolved_script_result_trigger_path,
                );
            }

            if !reuse_process && !perf_suite_prewarm_scripts.is_empty() {
                for prewarm in &perf_suite_prewarm_scripts {
                    run_suite_aux_script_must_pass(prewarm, &mut child)?;
                }
            }
            if !perf_suite_prelude_scripts.is_empty()
                && (!reuse_process || suite_prelude_each_run || run_index == 0)
            {
                for prelude in &perf_suite_prelude_scripts {
                    run_suite_aux_script_must_pass(prelude, &mut child)?;
                }
            }

            let script_key = normalize_repo_relative_path(&workspace_root, &src);
            let bundle_path = run_script::run_perf_script_and_resolve_bundle_artifact_path(
                &src,
                script_key.as_str(),
                &mut child,
                use_devtools_ws,
                connected_ws.as_ref(),
                &resolved_out_dir,
                &resolved_exit_path,
                &resolved_script_path,
                &resolved_script_trigger_path,
                &resolved_script_result_path,
                &resolved_script_result_trigger_path,
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
                perf_hint_rows.push(serde_json::json!({
                    "script": script_key.clone(),
                    "sort": sort.as_str(),
                    "repeat": repeat,
                    "run_index": run_index,
                    "bundle": bundle_path.display().to_string(),
                    "warmup_frames": report_warmup_frames,
                    "hints": hints,
                    "unit_costs": unit_costs,
                    "worst": worst,
                    "trace_chrome_json_path": trace_chrome_json_path,
                    "failures": failures,
                }));
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
            let top_dispatch_events = top.map(|r| r.dispatch_events).unwrap_or(0);
            let top_hit_test_queries = top.map(|r| r.hit_test_queries).unwrap_or(0);
            let top_hit_test_bounds_tree_queries =
                top.map(|r| r.hit_test_bounds_tree_queries).unwrap_or(0);
            let top_hit_test_bounds_tree_disabled =
                top.map(|r| r.hit_test_bounds_tree_disabled).unwrap_or(0);
            let top_hit_test_bounds_tree_misses =
                top.map(|r| r.hit_test_bounds_tree_misses).unwrap_or(0);
            let top_hit_test_bounds_tree_hits =
                top.map(|r| r.hit_test_bounds_tree_hits).unwrap_or(0);
            let top_hit_test_bounds_tree_candidate_rejected = top
                .map(|r| r.hit_test_bounds_tree_candidate_rejected)
                .unwrap_or(0);
            let top_frame_arena_capacity_estimate_bytes = top
                .map(|r| r.frame_arena_capacity_estimate_bytes)
                .unwrap_or(0);
            let top_frame_arena_grow_events = top.map(|r| r.frame_arena_grow_events).unwrap_or(0);
            let top_element_children_vec_pool_reuses =
                top.map(|r| r.element_children_vec_pool_reuses).unwrap_or(0);
            let top_element_children_vec_pool_misses =
                top.map(|r| r.element_children_vec_pool_misses).unwrap_or(0);
            let top_frame = top.map(|r| r.frame_id).unwrap_or(0);
            let top_tick = top.map(|r| r.tick_id).unwrap_or(0);
            let top_view_cache_contained_relayouts =
                top.map(|r| r.view_cache_contained_relayouts).unwrap_or(0);
            let top_view_cache_roots_total = top.map(|r| r.view_cache_roots_total).unwrap_or(0);
            let top_view_cache_roots_reused = top.map(|r| r.view_cache_roots_reused).unwrap_or(0);
            let top_view_cache_roots_first_mount =
                top.map(|r| r.view_cache_roots_first_mount).unwrap_or(0);
            let top_view_cache_roots_node_recreated =
                top.map(|r| r.view_cache_roots_node_recreated).unwrap_or(0);
            let top_view_cache_roots_cache_key_mismatch = top
                .map(|r| r.view_cache_roots_cache_key_mismatch)
                .unwrap_or(0);
            let top_view_cache_roots_not_marked_reuse_root = top
                .map(|r| r.view_cache_roots_not_marked_reuse_root)
                .unwrap_or(0);
            let top_view_cache_roots_needs_rerender =
                top.map(|r| r.view_cache_roots_needs_rerender).unwrap_or(0);
            let top_view_cache_roots_layout_invalidated = top
                .map(|r| r.view_cache_roots_layout_invalidated)
                .unwrap_or(0);
            let top_view_cache_roots_manual = top.map(|r| r.view_cache_roots_manual).unwrap_or(0);
            let top_cache_roots_contained_relayout =
                top.map(|r| r.cache_roots_contained_relayout).unwrap_or(0);
            let top_set_children_barrier_writes =
                top.map(|r| r.set_children_barrier_writes).unwrap_or(0);
            let top_barrier_relayouts_scheduled =
                top.map(|r| r.barrier_relayouts_scheduled).unwrap_or(0);
            let top_barrier_relayouts_performed =
                top.map(|r| r.barrier_relayouts_performed).unwrap_or(0);
            let top_virtual_list_visible_range_checks = top
                .map(|r| r.virtual_list_visible_range_checks)
                .unwrap_or(0);
            let top_virtual_list_visible_range_refreshes = top
                .map(|r| r.virtual_list_visible_range_refreshes)
                .unwrap_or(0);
            let top_renderer_tick_id = top.map(|r| r.renderer_tick_id).unwrap_or(0);
            let top_renderer_frame_id = top.map(|r| r.renderer_frame_id).unwrap_or(0);
            let top_renderer_encode_scene_us = top.map(|r| r.renderer_encode_scene_us).unwrap_or(0);
            let top_renderer_prepare_text_us = top.map(|r| r.renderer_prepare_text_us).unwrap_or(0);
            let top_renderer_prepare_svg_us = top.map(|r| r.renderer_prepare_svg_us).unwrap_or(0);
            let top_renderer_draw_calls = top.map(|r| r.renderer_draw_calls).unwrap_or(0);
            let top_renderer_pipeline_switches =
                top.map(|r| r.renderer_pipeline_switches).unwrap_or(0);
            let top_renderer_bind_group_switches =
                top.map(|r| r.renderer_bind_group_switches).unwrap_or(0);
            let top_renderer_scissor_sets = top.map(|r| r.renderer_scissor_sets).unwrap_or(0);
            let top_renderer_scene_encoding_cache_misses = top
                .map(|r| r.renderer_scene_encoding_cache_misses)
                .unwrap_or(0);
            let top_renderer_material_quad_ops =
                top.map(|r| r.renderer_material_quad_ops).unwrap_or(0);
            let top_renderer_material_sampled_quad_ops = top
                .map(|r| r.renderer_material_sampled_quad_ops)
                .unwrap_or(0);
            let top_renderer_material_distinct =
                top.map(|r| r.renderer_material_distinct).unwrap_or(0);
            let top_renderer_material_unknown_ids =
                top.map(|r| r.renderer_material_unknown_ids).unwrap_or(0);
            let top_renderer_material_degraded_due_to_budget = top
                .map(|r| r.renderer_material_degraded_due_to_budget)
                .unwrap_or(0);
            let top_renderer_text_atlas_upload_bytes =
                top.map(|r| r.renderer_text_atlas_upload_bytes).unwrap_or(0);
            let top_renderer_text_atlas_evicted_pages = top
                .map(|r| r.renderer_text_atlas_evicted_pages)
                .unwrap_or(0);
            let top_renderer_svg_upload_bytes =
                top.map(|r| r.renderer_svg_upload_bytes).unwrap_or(0);
            let top_renderer_image_upload_bytes =
                top.map(|r| r.renderer_image_upload_bytes).unwrap_or(0);
            let top_renderer_svg_raster_cache_misses =
                top.map(|r| r.renderer_svg_raster_cache_misses).unwrap_or(0);
            let top_renderer_svg_raster_budget_evictions = top
                .map(|r| r.renderer_svg_raster_budget_evictions)
                .unwrap_or(0);
            let top_renderer_svg_raster_budget_bytes =
                top.map(|r| r.renderer_svg_raster_budget_bytes).unwrap_or(0);
            let top_renderer_svg_rasters_live =
                top.map(|r| r.renderer_svg_rasters_live).unwrap_or(0);
            let top_renderer_svg_standalone_bytes_live = top
                .map(|r| r.renderer_svg_standalone_bytes_live)
                .unwrap_or(0);
            let top_renderer_svg_mask_atlas_pages_live = top
                .map(|r| r.renderer_svg_mask_atlas_pages_live)
                .unwrap_or(0);
            let top_renderer_svg_mask_atlas_bytes_live = top
                .map(|r| r.renderer_svg_mask_atlas_bytes_live)
                .unwrap_or(0);
            let top_renderer_svg_mask_atlas_used_px =
                top.map(|r| r.renderer_svg_mask_atlas_used_px).unwrap_or(0);
            let top_renderer_svg_mask_atlas_capacity_px = top
                .map(|r| r.renderer_svg_mask_atlas_capacity_px)
                .unwrap_or(0);
            let top_renderer_svg_raster_cache_hits =
                top.map(|r| r.renderer_svg_raster_cache_hits).unwrap_or(0);
            let top_renderer_svg_mask_atlas_page_evictions = top
                .map(|r| r.renderer_svg_mask_atlas_page_evictions)
                .unwrap_or(0);
            let top_renderer_svg_mask_atlas_entries_evicted = top
                .map(|r| r.renderer_svg_mask_atlas_entries_evicted)
                .unwrap_or(0);
            let top_renderer_intermediate_budget_bytes = top
                .map(|r| r.renderer_intermediate_budget_bytes)
                .unwrap_or(0);
            let top_renderer_intermediate_in_use_bytes = top
                .map(|r| r.renderer_intermediate_in_use_bytes)
                .unwrap_or(0);
            let top_renderer_intermediate_peak_in_use_bytes = top
                .map(|r| r.renderer_intermediate_peak_in_use_bytes)
                .unwrap_or(0);
            let top_renderer_intermediate_release_targets = top
                .map(|r| r.renderer_intermediate_release_targets)
                .unwrap_or(0);
            let top_renderer_intermediate_pool_allocations = top
                .map(|r| r.renderer_intermediate_pool_allocations)
                .unwrap_or(0);
            let top_renderer_intermediate_pool_reuses = top
                .map(|r| r.renderer_intermediate_pool_reuses)
                .unwrap_or(0);
            let top_renderer_intermediate_pool_releases = top
                .map(|r| r.renderer_intermediate_pool_releases)
                .unwrap_or(0);
            let top_renderer_intermediate_pool_evictions = top
                .map(|r| r.renderer_intermediate_pool_evictions)
                .unwrap_or(0);
            let top_renderer_intermediate_pool_free_bytes = top
                .map(|r| r.renderer_intermediate_pool_free_bytes)
                .unwrap_or(0);
            let top_renderer_intermediate_pool_free_textures = top
                .map(|r| r.renderer_intermediate_pool_free_textures)
                .unwrap_or(0);

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
            runs_json.push(serde_json::json!({
                "run_index": run_index,
                "frames_considered": report.snapshots_considered,
                "frames_warmup_skipped": report.snapshots_skipped_warmup,
                "top_total_time_us": top_total,
                "top_layout_time_us": top_layout,
                "top_layout_engine_solve_time_us": top_solve,
                "top_layout_engine_solves": top_solves,
                "top_prepaint_time_us": top_prepaint,
                "top_paint_time_us": top_paint,
                "top_dispatch_time_us": top_dispatch,
                "top_hit_test_time_us": top_hit_test,
                "frame_p50_total_time_us": report.p50_total_time_us,
                "frame_p95_total_time_us": report.p95_total_time_us,
                "frame_max_total_time_us": report.max_total_time_us,
                "frame_p50_ui_thread_cpu_time_us": report.p50_ui_thread_cpu_time_us,
                "frame_p95_ui_thread_cpu_time_us": report.p95_ui_thread_cpu_time_us,
                "frame_max_ui_thread_cpu_time_us": report.max_ui_thread_cpu_time_us,
                "frame_p50_layout_time_us": report.p50_layout_time_us,
                "frame_p95_layout_time_us": report.p95_layout_time_us,
                "frame_p50_layout_engine_solve_time_us": report.p50_layout_engine_solve_time_us,
                "frame_p95_layout_engine_solve_time_us": report.p95_layout_engine_solve_time_us,
                "frame_max_layout_engine_solve_time_us": report.max_layout_engine_solve_time_us,
                "frame_p50_prepaint_time_us": report.p50_prepaint_time_us,
                "frame_p95_prepaint_time_us": report.p95_prepaint_time_us,
                "frame_max_prepaint_time_us": report.max_prepaint_time_us,
                "frame_p50_paint_time_us": report.p50_paint_time_us,
                "frame_p95_paint_time_us": report.p95_paint_time_us,
                "frame_max_paint_time_us": report.max_paint_time_us,
                "frame_p50_dispatch_time_us": report.p50_dispatch_time_us,
                "frame_p95_dispatch_time_us": report.p95_dispatch_time_us,
                "frame_p50_hit_test_time_us": report.p50_hit_test_time_us,
                "frame_p95_hit_test_time_us": report.p95_hit_test_time_us,
                "top_dispatch_events": top_dispatch_events,
                "top_hit_test_queries": top_hit_test_queries,
                "pointer_move_frames_present": pointer_move_frames_present,
                "pointer_move_frames_considered": pointer_move_frames_considered,
                "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
                "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
                "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
                "run_paint_cache_hit_test_only_replay_allowed_max": run_paint_cache_hit_test_only_replay_allowed_max,
                "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                "top_hit_test_bounds_tree_queries": top_hit_test_bounds_tree_queries,
                "top_hit_test_bounds_tree_disabled": top_hit_test_bounds_tree_disabled,
                "top_hit_test_bounds_tree_misses": top_hit_test_bounds_tree_misses,
                "top_hit_test_bounds_tree_hits": top_hit_test_bounds_tree_hits,
                "top_hit_test_bounds_tree_candidate_rejected": top_hit_test_bounds_tree_candidate_rejected,
                "top_frame_arena_capacity_estimate_bytes": top_frame_arena_capacity_estimate_bytes,
                "top_frame_arena_grow_events": top_frame_arena_grow_events,
                "top_element_children_vec_pool_reuses": top_element_children_vec_pool_reuses,
                "top_element_children_vec_pool_misses": top_element_children_vec_pool_misses,
                "top_tick_id": top_tick,
                "top_frame_id": top_frame,
                "top_view_cache_contained_relayouts": top_view_cache_contained_relayouts,
                "top_view_cache_roots_total": top_view_cache_roots_total,
                "top_view_cache_roots_reused": top_view_cache_roots_reused,
                "top_view_cache_roots_first_mount": top_view_cache_roots_first_mount,
                "top_view_cache_roots_node_recreated": top_view_cache_roots_node_recreated,
                "top_view_cache_roots_cache_key_mismatch": top_view_cache_roots_cache_key_mismatch,
                "top_view_cache_roots_not_marked_reuse_root": top_view_cache_roots_not_marked_reuse_root,
                "top_view_cache_roots_needs_rerender": top_view_cache_roots_needs_rerender,
                "top_view_cache_roots_layout_invalidated": top_view_cache_roots_layout_invalidated,
                "top_view_cache_roots_manual": top_view_cache_roots_manual,
                "top_cache_roots_contained_relayout": top_cache_roots_contained_relayout,
                "top_set_children_barrier_writes": top_set_children_barrier_writes,
                "top_barrier_relayouts_scheduled": top_barrier_relayouts_scheduled,
                "top_barrier_relayouts_performed": top_barrier_relayouts_performed,
                "top_virtual_list_visible_range_checks": top_virtual_list_visible_range_checks,
                "top_virtual_list_visible_range_refreshes": top_virtual_list_visible_range_refreshes,
                "top_renderer_tick_id": top_renderer_tick_id,
                "top_renderer_frame_id": top_renderer_frame_id,
                "top_renderer_encode_scene_us": top_renderer_encode_scene_us,
                "top_renderer_prepare_text_us": top_renderer_prepare_text_us,
                "top_renderer_prepare_svg_us": top_renderer_prepare_svg_us,
                "top_renderer_draw_calls": top_renderer_draw_calls,
                "top_renderer_pipeline_switches": top_renderer_pipeline_switches,
                "top_renderer_bind_group_switches": top_renderer_bind_group_switches,
                "top_renderer_scissor_sets": top_renderer_scissor_sets,
                "top_renderer_scene_encoding_cache_misses": top_renderer_scene_encoding_cache_misses,
                "top_renderer_material_quad_ops": top_renderer_material_quad_ops,
                "top_renderer_material_sampled_quad_ops": top_renderer_material_sampled_quad_ops,
                "top_renderer_material_distinct": top_renderer_material_distinct,
                "top_renderer_material_unknown_ids": top_renderer_material_unknown_ids,
                "top_renderer_material_degraded_due_to_budget": top_renderer_material_degraded_due_to_budget,
                "top_renderer_text_atlas_upload_bytes": top_renderer_text_atlas_upload_bytes,
                "top_renderer_text_atlas_evicted_pages": top_renderer_text_atlas_evicted_pages,
                "top_renderer_svg_upload_bytes": top_renderer_svg_upload_bytes,
                "top_renderer_image_upload_bytes": top_renderer_image_upload_bytes,
                "top_renderer_svg_raster_cache_misses": top_renderer_svg_raster_cache_misses,
                "top_renderer_svg_raster_budget_evictions": top_renderer_svg_raster_budget_evictions,
                "top_renderer_svg_raster_budget_bytes": top_renderer_svg_raster_budget_bytes,
                "top_renderer_svg_rasters_live": top_renderer_svg_rasters_live,
                "top_renderer_svg_standalone_bytes_live": top_renderer_svg_standalone_bytes_live,
                "top_renderer_svg_mask_atlas_pages_live": top_renderer_svg_mask_atlas_pages_live,
                "top_renderer_svg_mask_atlas_bytes_live": top_renderer_svg_mask_atlas_bytes_live,
                "top_renderer_svg_mask_atlas_used_px": top_renderer_svg_mask_atlas_used_px,
    	                        "top_renderer_svg_mask_atlas_capacity_px": top_renderer_svg_mask_atlas_capacity_px,
    	                        "top_renderer_svg_raster_cache_hits": top_renderer_svg_raster_cache_hits,
    	                        "top_renderer_svg_mask_atlas_page_evictions": top_renderer_svg_mask_atlas_page_evictions,
    	                        "top_renderer_svg_mask_atlas_entries_evicted": top_renderer_svg_mask_atlas_entries_evicted,
    	                        "top_renderer_intermediate_budget_bytes": top_renderer_intermediate_budget_bytes,
    	                        "top_renderer_intermediate_in_use_bytes": top_renderer_intermediate_in_use_bytes,
    	                        "top_renderer_intermediate_peak_in_use_bytes": top_renderer_intermediate_peak_in_use_bytes,
    	                        "top_renderer_intermediate_release_targets": top_renderer_intermediate_release_targets,
    	                        "top_renderer_intermediate_pool_allocations": top_renderer_intermediate_pool_allocations,
    	                        "top_renderer_intermediate_pool_reuses": top_renderer_intermediate_pool_reuses,
    	                        "top_renderer_intermediate_pool_releases": top_renderer_intermediate_pool_releases,
    	                        "top_renderer_intermediate_pool_evictions": top_renderer_intermediate_pool_evictions,
    	                        "top_renderer_intermediate_pool_free_bytes": top_renderer_intermediate_pool_free_bytes,
    	                        "top_renderer_intermediate_pool_free_textures": top_renderer_intermediate_pool_free_textures,
    	                        "bundle": bundle_path.display().to_string(),
    	                    }));

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
            }
        }

        if runs_total.len() == repeat {
            if stats_json {
                let mut top_frame_arena_capacity_estimate_bytes: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_frame_arena_grow_events: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_element_children_vec_pool_reuses: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_element_children_vec_pool_misses: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_view_cache_contained_relayouts: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_view_cache_roots_total: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_view_cache_roots_reused: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_view_cache_roots_cache_key_mismatch: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_view_cache_roots_needs_rerender: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_view_cache_roots_layout_invalidated: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_cache_roots_contained_relayout: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_set_children_barrier_writes: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_barrier_relayouts_scheduled: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_barrier_relayouts_performed: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_virtual_list_visible_range_checks: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_virtual_list_visible_range_refreshes: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_encode_scene_us: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_prepare_text_us: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_draw_calls: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_pipeline_switches: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_bind_group_switches: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_scene_encoding_cache_misses: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_material_quad_ops: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_material_sampled_quad_ops: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_material_distinct: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_material_unknown_ids: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_material_degraded_due_to_budget: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_text_atlas_upload_bytes: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_text_atlas_evicted_pages: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_svg_upload_bytes: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_image_upload_bytes: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_svg_raster_cache_misses: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_svg_raster_budget_evictions: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_svg_rasters_live: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_svg_mask_atlas_pages_live: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_svg_mask_atlas_used_px: Vec<u64> = Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_budget_bytes: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_in_use_bytes: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_peak_in_use_bytes: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_release_targets: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_pool_allocations: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_pool_reuses: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_pool_releases: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_pool_evictions: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_pool_free_bytes: Vec<u64> =
                    Vec::with_capacity(repeat);
                let mut top_renderer_intermediate_pool_free_textures: Vec<u64> =
                    Vec::with_capacity(repeat);
                for run in &runs_json {
                    top_frame_arena_capacity_estimate_bytes.push(
                        run.get("top_frame_arena_capacity_estimate_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_frame_arena_grow_events.push(
                        run.get("top_frame_arena_grow_events")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_element_children_vec_pool_reuses.push(
                        run.get("top_element_children_vec_pool_reuses")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_element_children_vec_pool_misses.push(
                        run.get("top_element_children_vec_pool_misses")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_view_cache_contained_relayouts.push(
                        run.get("top_view_cache_contained_relayouts")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_view_cache_roots_total.push(
                        run.get("top_view_cache_roots_total")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_view_cache_roots_reused.push(
                        run.get("top_view_cache_roots_reused")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_view_cache_roots_cache_key_mismatch.push(
                        run.get("top_view_cache_roots_cache_key_mismatch")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_view_cache_roots_needs_rerender.push(
                        run.get("top_view_cache_roots_needs_rerender")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_view_cache_roots_layout_invalidated.push(
                        run.get("top_view_cache_roots_layout_invalidated")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_cache_roots_contained_relayout.push(
                        run.get("top_cache_roots_contained_relayout")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_set_children_barrier_writes.push(
                        run.get("top_set_children_barrier_writes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_barrier_relayouts_scheduled.push(
                        run.get("top_barrier_relayouts_scheduled")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_barrier_relayouts_performed.push(
                        run.get("top_barrier_relayouts_performed")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_virtual_list_visible_range_checks.push(
                        run.get("top_virtual_list_visible_range_checks")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_virtual_list_visible_range_refreshes.push(
                        run.get("top_virtual_list_visible_range_refreshes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_encode_scene_us.push(
                        run.get("top_renderer_encode_scene_us")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_prepare_text_us.push(
                        run.get("top_renderer_prepare_text_us")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_draw_calls.push(
                        run.get("top_renderer_draw_calls")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_pipeline_switches.push(
                        run.get("top_renderer_pipeline_switches")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_bind_group_switches.push(
                        run.get("top_renderer_bind_group_switches")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_scene_encoding_cache_misses.push(
                        run.get("top_renderer_scene_encoding_cache_misses")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_material_quad_ops.push(
                        run.get("top_renderer_material_quad_ops")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_material_sampled_quad_ops.push(
                        run.get("top_renderer_material_sampled_quad_ops")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_material_distinct.push(
                        run.get("top_renderer_material_distinct")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_material_unknown_ids.push(
                        run.get("top_renderer_material_unknown_ids")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_material_degraded_due_to_budget.push(
                        run.get("top_renderer_material_degraded_due_to_budget")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_text_atlas_upload_bytes.push(
                        run.get("top_renderer_text_atlas_upload_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_text_atlas_evicted_pages.push(
                        run.get("top_renderer_text_atlas_evicted_pages")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_svg_upload_bytes.push(
                        run.get("top_renderer_svg_upload_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_image_upload_bytes.push(
                        run.get("top_renderer_image_upload_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_svg_raster_cache_misses.push(
                        run.get("top_renderer_svg_raster_cache_misses")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_svg_raster_budget_evictions.push(
                        run.get("top_renderer_svg_raster_budget_evictions")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_svg_rasters_live.push(
                        run.get("top_renderer_svg_rasters_live")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_svg_mask_atlas_pages_live.push(
                        run.get("top_renderer_svg_mask_atlas_pages_live")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_svg_mask_atlas_used_px.push(
                        run.get("top_renderer_svg_mask_atlas_used_px")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_budget_bytes.push(
                        run.get("top_renderer_intermediate_budget_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_in_use_bytes.push(
                        run.get("top_renderer_intermediate_in_use_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_peak_in_use_bytes.push(
                        run.get("top_renderer_intermediate_peak_in_use_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_release_targets.push(
                        run.get("top_renderer_intermediate_release_targets")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_pool_allocations.push(
                        run.get("top_renderer_intermediate_pool_allocations")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_pool_reuses.push(
                        run.get("top_renderer_intermediate_pool_reuses")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_pool_releases.push(
                        run.get("top_renderer_intermediate_pool_releases")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_pool_evictions.push(
                        run.get("top_renderer_intermediate_pool_evictions")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_pool_free_bytes.push(
                        run.get("top_renderer_intermediate_pool_free_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                    top_renderer_intermediate_pool_free_textures.push(
                        run.get("top_renderer_intermediate_pool_free_textures")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    );
                }
                perf_json_rows.push(serde_json::json!({
    	                        "script": src.display().to_string(),
    	                        "sort": sort.as_str(),
    		                        "repeat": repeat,
    		                            "runs": runs_json,
    		                            "stats": {
    		                                "total_time_us": summarize_times_us(&runs_total),
    		                                "layout_time_us": summarize_times_us(&runs_layout),
    		                                "layout_engine_solve_time_us": summarize_times_us(&runs_solve),
    		                                "prepaint_time_us": summarize_times_us(&runs_prepaint),
    		                                "paint_time_us": summarize_times_us(&runs_paint),
    		                                "dispatch_time_us": summarize_times_us(&runs_dispatch),
    		                                "hit_test_time_us": summarize_times_us(&runs_hit_test),
    		                                "pointer_move_max_dispatch_time_us": summarize_times_us(&runs_pointer_move_dispatch),
    		                                "pointer_move_max_hit_test_time_us": summarize_times_us(&runs_pointer_move_hit_test),
    		                                "pointer_move_snapshots_with_global_changes": summarize_times_us(&runs_pointer_move_global_changes),
    		                                "top_frame_arena_capacity_estimate_bytes": summarize_times_us(&top_frame_arena_capacity_estimate_bytes),
    		                                "top_frame_arena_grow_events": summarize_times_us(&top_frame_arena_grow_events),
    		                                "top_element_children_vec_pool_reuses": summarize_times_us(&top_element_children_vec_pool_reuses),
    		                                "top_element_children_vec_pool_misses": summarize_times_us(&top_element_children_vec_pool_misses),
    	                                "top_view_cache_contained_relayouts": summarize_times_us(&top_view_cache_contained_relayouts),
    	                                "top_view_cache_roots_total": summarize_times_us(&top_view_cache_roots_total),
    	                                "top_view_cache_roots_reused": summarize_times_us(&top_view_cache_roots_reused),
    	                                "top_view_cache_roots_cache_key_mismatch": summarize_times_us(&top_view_cache_roots_cache_key_mismatch),
    	                                "top_view_cache_roots_needs_rerender": summarize_times_us(&top_view_cache_roots_needs_rerender),
    	                                "top_view_cache_roots_layout_invalidated": summarize_times_us(&top_view_cache_roots_layout_invalidated),
    	                                "top_cache_roots_contained_relayout": summarize_times_us(&top_cache_roots_contained_relayout),
    	                                "top_set_children_barrier_writes": summarize_times_us(&top_set_children_barrier_writes),
    	                                "top_barrier_relayouts_scheduled": summarize_times_us(&top_barrier_relayouts_scheduled),
    	                                "top_barrier_relayouts_performed": summarize_times_us(&top_barrier_relayouts_performed),
    	                                "top_virtual_list_visible_range_checks": summarize_times_us(&top_virtual_list_visible_range_checks),
    	                                "top_virtual_list_visible_range_refreshes": summarize_times_us(&top_virtual_list_visible_range_refreshes),
    	                                "top_renderer_encode_scene_us": summarize_times_us(&top_renderer_encode_scene_us),
    	                                "top_renderer_prepare_text_us": summarize_times_us(&top_renderer_prepare_text_us),
    	                                "top_renderer_draw_calls": summarize_times_us(&top_renderer_draw_calls),
    	                                "top_renderer_pipeline_switches": summarize_times_us(&top_renderer_pipeline_switches),
    	                                "top_renderer_bind_group_switches": summarize_times_us(&top_renderer_bind_group_switches),
    	                                "top_renderer_scene_encoding_cache_misses": summarize_times_us(&top_renderer_scene_encoding_cache_misses),
    	                                "top_renderer_material_quad_ops": summarize_times_us(&top_renderer_material_quad_ops),
    	                                "top_renderer_material_sampled_quad_ops": summarize_times_us(&top_renderer_material_sampled_quad_ops),
    	                                "top_renderer_material_distinct": summarize_times_us(&top_renderer_material_distinct),
    	                                "top_renderer_material_unknown_ids": summarize_times_us(&top_renderer_material_unknown_ids),
    	                                "top_renderer_material_degraded_due_to_budget": summarize_times_us(&top_renderer_material_degraded_due_to_budget),
    	                                "top_renderer_text_atlas_upload_bytes": summarize_times_us(&top_renderer_text_atlas_upload_bytes),
    	                                "top_renderer_text_atlas_evicted_pages": summarize_times_us(&top_renderer_text_atlas_evicted_pages),
    	                                "top_renderer_svg_upload_bytes": summarize_times_us(&top_renderer_svg_upload_bytes),
    	                                "top_renderer_image_upload_bytes": summarize_times_us(&top_renderer_image_upload_bytes),
    	                                "top_renderer_svg_raster_cache_misses": summarize_times_us(&top_renderer_svg_raster_cache_misses),
    	                                "top_renderer_svg_raster_budget_evictions": summarize_times_us(&top_renderer_svg_raster_budget_evictions),
    	                                "top_renderer_svg_rasters_live": summarize_times_us(&top_renderer_svg_rasters_live),
    	                                "top_renderer_svg_mask_atlas_pages_live": summarize_times_us(&top_renderer_svg_mask_atlas_pages_live),
    	                                "top_renderer_svg_mask_atlas_used_px": summarize_times_us(&top_renderer_svg_mask_atlas_used_px),
    	                                "top_renderer_intermediate_budget_bytes": summarize_times_us(&top_renderer_intermediate_budget_bytes),
    	                                "top_renderer_intermediate_in_use_bytes": summarize_times_us(&top_renderer_intermediate_in_use_bytes),
    	                                "top_renderer_intermediate_peak_in_use_bytes": summarize_times_us(&top_renderer_intermediate_peak_in_use_bytes),
    	                                "top_renderer_intermediate_release_targets": summarize_times_us(&top_renderer_intermediate_release_targets),
    	                                "top_renderer_intermediate_pool_allocations": summarize_times_us(&top_renderer_intermediate_pool_allocations),
    	                                "top_renderer_intermediate_pool_reuses": summarize_times_us(&top_renderer_intermediate_pool_reuses),
    	                                "top_renderer_intermediate_pool_releases": summarize_times_us(&top_renderer_intermediate_pool_releases),
    	                                "top_renderer_intermediate_pool_evictions": summarize_times_us(&top_renderer_intermediate_pool_evictions),
    	                                "top_renderer_intermediate_pool_free_bytes": summarize_times_us(&top_renderer_intermediate_pool_free_bytes),
    	                                "top_renderer_intermediate_pool_free_textures": summarize_times_us(&top_renderer_intermediate_pool_free_textures),
    	                            },
    	                            "worst_run": script_worst.as_ref().map(|(us, bundle, run_index)| serde_json::json!({
    	                                "top_total_time_us": us,
    	                                "bundle": bundle.display().to_string(),
    	                                "run_index": run_index,
    	                            })),
    	                        }));
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

                perf_baseline_rows.push(serde_json::json!({
                    "script": script_key.clone(),
                    "measured_max": {
                        "top_total_time_us": max_total,
                        "top_layout_time_us": max_layout,
                        "top_layout_engine_solve_time_us": max_solve,
                        "frame_p95_total_time_us": max_frame_p95_total,
                        "frame_p95_layout_time_us": max_frame_p95_layout,
                        "frame_p95_layout_engine_solve_time_us": max_frame_p95_solve,
                        "pointer_move_max_dispatch_time_us": max_pointer_move_dispatch,
                        "pointer_move_max_hit_test_time_us": max_pointer_move_hit_test,
                        "pointer_move_snapshots_with_global_changes": max_pointer_move_global_changes,
                        "run_paint_cache_hit_test_only_replay_allowed_max": max_run_paint_cache_hit_test_only_replay_allowed_max,
                        "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        "renderer_encode_scene_us": max_renderer_encode_scene_us,
                        "renderer_upload_us": max_renderer_upload_us,
                        "renderer_record_passes_us": max_renderer_record_passes_us,
                        "renderer_encoder_finish_us": max_renderer_encoder_finish_us,
                        "renderer_prepare_text_us": max_renderer_prepare_text_us,
                        "renderer_prepare_svg_us": max_renderer_prepare_svg_us,
                    },
                    "measured_p90": {
                        "top_total_time_us": p90_total,
                        "top_layout_time_us": p90_layout,
                        "top_layout_engine_solve_time_us": p90_solve,
                        "frame_p95_total_time_us": p90_frame_p95_total,
                        "frame_p95_layout_time_us": p90_frame_p95_layout,
                        "frame_p95_layout_engine_solve_time_us": p90_frame_p95_solve,
                        "renderer_encode_scene_us": p90_renderer_encode_scene_us,
                        "renderer_upload_us": p90_renderer_upload_us,
                        "renderer_record_passes_us": p90_renderer_record_passes_us,
                        "renderer_encoder_finish_us": p90_renderer_encoder_finish_us,
                        "renderer_prepare_text_us": p90_renderer_prepare_text_us,
                        "renderer_prepare_svg_us": p90_renderer_prepare_svg_us,
                    },
                    "measured_p95": {
                        "top_total_time_us": p95_total,
                        "top_layout_time_us": p95_layout,
                        "top_layout_engine_solve_time_us": p95_solve,
                        "frame_p95_total_time_us": p95_frame_p95_total,
                        "frame_p95_layout_time_us": p95_frame_p95_layout,
                        "frame_p95_layout_engine_solve_time_us": p95_frame_p95_solve,
                        "renderer_encode_scene_us": p95_renderer_encode_scene_us,
                        "renderer_upload_us": p95_renderer_upload_us,
                        "renderer_record_passes_us": p95_renderer_record_passes_us,
                        "renderer_encoder_finish_us": p95_renderer_encoder_finish_us,
                        "renderer_prepare_text_us": p95_renderer_prepare_text_us,
                        "renderer_prepare_svg_us": p95_renderer_prepare_svg_us,
                    },
                    "threshold_seed": {
                        "top_total_time_us": seed_total_value,
                        "top_layout_time_us": seed_layout_value,
                        "top_layout_engine_solve_time_us": seed_solve_value,
                        "frame_p95_total_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_total_value),
                        "frame_p95_layout_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_layout_value),
                        "frame_p95_layout_engine_solve_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_solve_value),
                    },
                    "threshold_seed_source": {
                        "top_total_time_us": seed_total.as_str(),
                        "top_layout_time_us": seed_layout.as_str(),
                        "top_layout_engine_solve_time_us": seed_solve.as_str(),
                        "frame_p95_total_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_total.as_str()),
                        "frame_p95_layout_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_layout.as_str()),
                        "frame_p95_layout_engine_solve_time_us": wants_frame_p95_thresholds.then_some(seed_frame_p95_solve.as_str()),
                    },
                    "thresholds": {
                        "max_top_total_us": (!wants_frame_p95_thresholds).then_some(thr_total),
                        "max_top_layout_us": (!wants_frame_p95_thresholds).then_some(thr_layout),
                        "max_top_solve_us": (!wants_frame_p95_thresholds).then_some(thr_solve),
                        "max_frame_p95_total_us": thr_frame_p95_total,
                        "max_frame_p95_layout_us": thr_frame_p95_layout,
                        "max_frame_p95_solve_us": thr_frame_p95_solve,
                        "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
                        "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
                        "max_pointer_move_global_changes": thr_pointer_move_global_changes,
                        "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_min_run_paint_cache_hit_test_only_replay_allowed_max,
                        "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                        "max_renderer_encode_scene_us": thr_renderer_encode_scene_us,
                        "max_renderer_upload_us": thr_renderer_upload_us,
                        "max_renderer_record_passes_us": thr_renderer_record_passes_us,
                        "max_renderer_encoder_finish_us": thr_renderer_encoder_finish_us,
                        "max_renderer_prepare_text_us": thr_renderer_prepare_text_us,
                        "max_renderer_prepare_svg_us": thr_renderer_prepare_svg_us,
                    },
                }));
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
                let row = serde_json::json!({
                    "script": script_key.clone(),
                    "sort": sort.as_str(),
                    "repeat": repeat,
                    "runs": runs_json,
                    "observed_aggregate": perf_threshold_agg.as_str(),
                    "observed": {
                        "top_total_time_us": observed_total,
                        "top_layout_time_us": observed_layout,
                        "top_layout_engine_solve_time_us": observed_solve,
                        "frame_p95_total_time_us": observed_frame_p95_total,
                        "frame_p95_layout_time_us": observed_frame_p95_layout,
                        "frame_p95_layout_engine_solve_time_us": observed_frame_p95_solve,
                    },
                    "worst_run": script_worst.as_ref().map(|(us, bundle, run_index)| serde_json::json!({
                        "top_total_time_us": us,
                        "bundle": bundle.display().to_string(),
                        "run_index": run_index,
                        "trace_chrome": bundle
                            .parent()
                            .map(|dir| dir.join("trace.chrome.json"))
                            .filter(|p| p.is_file())
                            .map(|p| p.display().to_string()),
                    })),
                    "max": {
                        "top_total_time_us": max_total,
                        "top_layout_time_us": max_layout,
                        "top_layout_engine_solve_time_us": max_solve,
                        "frame_p95_total_time_us": max_frame_p95_total,
                        "frame_p95_layout_time_us": max_frame_p95_layout,
                        "frame_p95_layout_engine_solve_time_us": max_frame_p95_solve,
                        "pointer_move_max_dispatch_time_us": max_pointer_move_dispatch,
                        "pointer_move_max_hit_test_time_us": max_pointer_move_hit_test,
                        "pointer_move_snapshots_with_global_changes": max_pointer_move_global_changes,
                        "run_paint_cache_hit_test_only_replay_allowed_max": max_run_paint_cache_hit_test_only_replay_allowed_max,
                        "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    },
                    "p50": {
                        "top_total_time_us": percentile_nearest_rank_sorted(&sorted_total, 0.50),
                        "top_layout_time_us": percentile_nearest_rank_sorted(&sorted_layout, 0.50),
                        "top_layout_engine_solve_time_us": percentile_nearest_rank_sorted(&sorted_solve, 0.50),
                        "frame_p95_total_time_us": percentile_nearest_rank_sorted(&sorted_frame_p95_total, 0.50),
                        "frame_p95_layout_time_us": percentile_nearest_rank_sorted(&sorted_frame_p95_layout, 0.50),
                        "frame_p95_layout_engine_solve_time_us": percentile_nearest_rank_sorted(&sorted_frame_p95_solve, 0.50),
                    },
                    "p90": {
                        "top_total_time_us": p90_total,
                        "top_layout_time_us": p90_layout,
                        "top_layout_engine_solve_time_us": p90_solve,
                        "frame_p95_total_time_us": p90_frame_p95_total,
                        "frame_p95_layout_time_us": p90_frame_p95_layout,
                        "frame_p95_layout_engine_solve_time_us": p90_frame_p95_solve,
                    },
                    "p95": {
                        "top_total_time_us": p95_total,
                        "top_layout_time_us": p95_layout,
                        "top_layout_engine_solve_time_us": p95_solve,
                        "frame_p95_total_time_us": p95_frame_p95_total,
                        "frame_p95_layout_time_us": p95_frame_p95_layout,
                        "frame_p95_layout_engine_solve_time_us": p95_frame_p95_solve,
                    },
                    "thresholds": {
                        "max_top_total_us": thr_total,
                        "max_top_layout_us": thr_layout,
                        "max_top_solve_us": thr_solve,
                        "max_frame_p95_total_us": thr_frame_p95_total,
                        "max_frame_p95_layout_us": thr_frame_p95_layout,
                        "max_frame_p95_solve_us": thr_frame_p95_solve,
                        "max_pointer_move_dispatch_us": thr_pointer_move_dispatch,
                        "max_pointer_move_hit_test_us": thr_pointer_move_hit_test,
                        "max_pointer_move_global_changes": thr_pointer_move_global_changes,
                        "min_run_paint_cache_hit_test_only_replay_allowed_max": thr_paint_cache_hit_test_only_replay_allowed_max,
                        "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": thr_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    },
                    "threshold_sources": {
                        "max_top_total_us": src_total,
                        "max_top_layout_us": src_layout,
                        "max_top_solve_us": src_solve,
                        "max_frame_p95_total_us": src_frame_p95_total,
                        "max_frame_p95_layout_us": src_frame_p95_layout,
                        "max_frame_p95_solve_us": src_frame_p95_solve,
                        "max_pointer_move_dispatch_us": src_pointer_move_dispatch,
                        "max_pointer_move_hit_test_us": src_pointer_move_hit_test,
                        "max_pointer_move_global_changes": src_pointer_move_global_changes,
                        "min_run_paint_cache_hit_test_only_replay_allowed_max": src_paint_cache_hit_test_only_replay_allowed_max,
                        "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": src_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
                    },
                });
                perf_threshold_rows.push(row);
                perf_threshold_failures.extend(scan_perf_threshold_failures(
                    script_key.as_str(),
                    sort,
                    perf_threshold_agg,
                    cli_thresholds,
                    baseline_thresholds,
                    observed_total,
                    max_total,
                    p95_total,
                    observed_layout,
                    max_layout,
                    p95_layout,
                    observed_solve,
                    max_solve,
                    p95_solve,
                    observed_frame_p95_total,
                    max_frame_p95_total,
                    p95_frame_p95_total,
                    observed_frame_p95_layout,
                    max_frame_p95_layout,
                    p95_frame_p95_layout,
                    observed_frame_p95_solve,
                    max_frame_p95_solve,
                    p95_frame_p95_solve,
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
                    script_worst
                        .as_ref()
                        .map(|(_us, bundle, _run)| bundle.as_path()),
                    script_worst.as_ref().map(|(_us, _bundle, run)| *run),
                    script_worst_layout
                        .as_ref()
                        .map(|(_us, bundle, _run)| bundle.as_path()),
                    script_worst_layout.as_ref().map(|(_us, _bundle, run)| *run),
                    script_worst_solve
                        .as_ref()
                        .map(|(_us, bundle, _run)| bundle.as_path()),
                    script_worst_solve.as_ref().map(|(_us, _bundle, run)| *run),
                ));
            }
        }
    }

    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);

    if let Some(test_id) = check_pixels_changed_test_id.as_deref() {
        stats::check_out_dir_for_pixels_changed(&resolved_out_dir, test_id, warmup_frames)?;
    }

    if let Some(path) = perf_baseline_out.as_ref() {
        let out_path = path;
        let policy = seed_policy
            .as_ref()
            .ok_or_else(|| "internal error: missing seed policy".to_string())?;
        let payload = serde_json::json!({
            "schema_version": 1,
            "generated_unix_ms": now_unix_ms(),
            "kind": "perf_baseline",
            "out_path": out_path.display().to_string(),
            "warmup_frames": warmup_frames,
            "sort": sort.as_str(),
            "repeat": repeat,
            "headroom_pct": perf_baseline_headroom_pct,
            "threshold_seed_policy": policy.threshold_seed_policy_json(),
            "rows": perf_baseline_rows,
        });
        write_json_value(out_path, &payload)?;
        if !stats_json {
            println!("wrote perf baseline: {}", out_path.display());
        }
    }

    let mut perf_threshold_failure: Option<(usize, PathBuf)> = None;
    if wants_perf_thresholds {
        let out_path = resolved_out_dir.join("check.perf_thresholds.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "generated_unix_ms": now_unix_ms(),
            "kind": "perf_thresholds",
            "out_dir": resolved_out_dir.display().to_string(),
            "warmup_frames": warmup_frames,
            "observed_aggregate": perf_threshold_agg.as_str(),
            "suite_hooks": {
                "prewarm": perf_suite_prewarm_scripts.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "prelude": perf_suite_prelude_scripts.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "prelude_each_run": suite_prelude_each_run,
            },
            "thresholds": {
                "max_top_total_us": cli_thresholds.max_top_total_us,
                "max_top_layout_us": cli_thresholds.max_top_layout_us,
                "max_top_solve_us": cli_thresholds.max_top_solve_us,
                "max_pointer_move_dispatch_us": cli_thresholds.max_pointer_move_dispatch_us,
                "max_pointer_move_hit_test_us": cli_thresholds.max_pointer_move_hit_test_us,
                "max_pointer_move_global_changes": cli_thresholds.max_pointer_move_global_changes,
                "min_run_paint_cache_hit_test_only_replay_allowed_max": cli_thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max,
                "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": cli_thresholds.max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
            },
            "baseline": perf_baseline.as_ref().map(|b| serde_json::json!({
                "path": b.path.display().to_string(),
                "scripts": b.thresholds_by_script.len(),
            })),
            "rows": perf_threshold_rows,
            "failures": perf_threshold_failures,
        });
        let _ = write_json_value(&out_path, &payload);
        if !perf_threshold_failures.is_empty() {
            perf_threshold_failure = Some((perf_threshold_failures.len(), out_path.clone()));
        }
    }

    let mut perf_hint_failure: Option<(usize, PathBuf)> = None;
    if wants_perf_hints {
        let out_path = resolved_out_dir.join("check.perf_hints.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "generated_unix_ms": now_unix_ms(),
            "kind": "perf_hints",
            "out_dir": resolved_out_dir.display().to_string(),
            "warmup_frames": warmup_frames,
            "min_severity": perf_hint_gate_opts.min_severity.as_str(),
            "deny": perf_hint_gate_opts.deny_codes.iter().cloned().collect::<Vec<_>>(),
            "rows": perf_hint_rows,
            "failures": perf_hint_failures,
        });
        let _ = write_json_value(&out_path, &payload);
        if !perf_hint_failures.is_empty() {
            perf_hint_failure = Some((perf_hint_failures.len(), out_path.clone()));
        }
    }

    if launched_by_fretboard {
        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
    }

    if stats_json {
        let worst = overall_worst.as_ref().map(|(us, src, bundle)| {
            serde_json::json!({
                "script": src.display().to_string(),
                "top_total_time_us": us,
                "bundle": bundle.display().to_string(),
            })
        });
        let payload = serde_json::json!({
            "schema_version": 1,
            "sort": sort.as_str(),
            "repeat": repeat,
            "rows": perf_json_rows,
            "worst_overall": worst,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
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
