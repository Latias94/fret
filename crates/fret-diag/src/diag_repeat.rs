use super::*;

#[derive(Debug, Clone)]
pub(crate) struct RepeatCmdContext {
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
    pub pack_include_screenshots: bool,
    pub check_pixels_changed_test_id: Option<String>,
    pub check_pixels_unchanged_test_id: Option<String>,
    pub reuse_launch: bool,
    pub launch: Option<Vec<String>>,
    pub launch_env: Vec<(String, String)>,
    pub launch_high_priority: bool,
    pub launch_write_bundle_json: bool,
    pub perf_repeat: u64,
    pub check_memory_p90_max: Vec<(String, u64)>,
    pub compare_enabled: bool,
    pub compare_eps_px: f32,
    pub compare_ignore_bounds: bool,
    pub compare_ignore_scene_fingerprint: bool,
    pub warmup_frames: u64,
    pub lint_all_test_ids_bounds: bool,
    pub lint_eps_px: f32,
    pub stats_json: bool,
    pub timeout_ms: u64,
    pub poll_ms: u64,
}

fn push_env_if_missing(env: &mut Vec<(String, String)>, key: &str, value: &str) {
    if env.iter().any(|(existing, _)| existing == key) {
        return;
    }
    env.push((key.to_string(), value.to_string()));
}

fn merged_repeat_launch_env(
    launch_env: &[(String, String)],
    script_defaults: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let mut merged = launch_env.to_vec();
    for (key, value) in script_defaults {
        push_env_if_missing(&mut merged, &key, &value);
    }
    merged
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_repeat(ctx: RepeatCmdContext) -> Result<(), String> {
    let RepeatCmdContext {
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
        pack_include_screenshots,
        check_pixels_changed_test_id,
        check_pixels_unchanged_test_id,
        reuse_launch,
        launch,
        launch_env,
        launch_high_priority,
        launch_write_bundle_json,
        perf_repeat,
        check_memory_p90_max,
        compare_enabled,
        compare_eps_px,
        compare_ignore_bounds,
        compare_ignore_scene_fingerprint,
        warmup_frames,
        lint_all_test_ids_bounds,
        lint_eps_px,
        stats_json,
        timeout_ms,
        poll_ms,
    } = ctx;

    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing script path (try: fretboard diag repeat ./script.json --repeat 7)".to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let repeat = perf_repeat.max(1) as usize;

    let src = resolve_path(&workspace_root, PathBuf::from(src));
    let wants_screenshots = script_requests_screenshots(&src)
        || pack_include_screenshots
        || check_pixels_changed_test_id.is_some()
        || check_pixels_unchanged_test_id.is_some();

    let repeat_launch_env = merged_repeat_launch_env(&launch_env, script_env_defaults(&src));
    let reuse_process = launch.is_none() || reuse_launch;

    let mut launch_fs_transport_cfg =
        crate::transport::FsDiagTransportConfig::from_out_dir(resolved_out_dir.clone());
    launch_fs_transport_cfg.script_path = resolved_script_path.clone();
    launch_fs_transport_cfg.script_trigger_path = resolved_script_trigger_path.clone();
    launch_fs_transport_cfg.script_result_path = resolved_script_result_path.clone();
    launch_fs_transport_cfg.script_result_trigger_path =
        resolved_script_result_trigger_path.clone();

    let mut child = if reuse_process {
        maybe_launch_demo(
            &launch,
            &repeat_launch_env,
            &workspace_root,
            &resolved_ready_path,
            &resolved_exit_path,
            &launch_fs_transport_cfg,
            wants_screenshots,
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
        })?
    } else {
        None
    };

    let mut runs: Vec<serde_json::Value> = Vec::with_capacity(repeat);

    let mut baseline_run: Option<usize> = None;
    let mut baseline_bundle: Option<PathBuf> = None;

    let mut tooling_error_reason_code: Option<String> = None;

    let mut failed_runs: u64 = 0;
    let mut differing_runs: u64 = 0;
    let mut first_failed_run: Option<usize> = None;
    let mut first_differing_run: Option<usize> = None;
    let mut worst_perf: Option<(usize, u64, u64)> = None; // (index, total_us, frame_id)
    let mut stage_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut reason_code_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut lint_error_runs: Vec<usize> = Vec::new();
    let mut lint_counts_by_code: std::collections::BTreeMap<String, (u64, u64)> =
        std::collections::BTreeMap::new();
    let mut evidence_present_runs: Vec<usize> = Vec::new();
    let mut focus_mismatch_runs: Vec<usize> = Vec::new();
    let mut blocking_reason_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut overlay_chosen_side_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut ime_event_kind_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();

    fn read_script_result_typed(path: &Path) -> Option<UiScriptResultV1> {
        let bytes = std::fs::read(path).ok()?;
        serde_json::from_slice::<UiScriptResultV1>(&bytes).ok()
    }

    fn read_tooling_reason_code(path: &Path) -> Option<String> {
        read_json_value(path).and_then(|v| {
            v.get("reason_code")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
    }

    fn copy_if_exists(src: &Path, dst: &Path) {
        if !src.is_file() {
            return;
        }
        let _ = std::fs::copy(src, dst);
    }

    fn write_repeat_evidence_index_best_effort(
        run_out_dir: &Path,
        out_dir: &Path,
        bundle_artifact: Option<&PathBuf>,
    ) {
        copy_if_exists(
            &out_dir.join("resource.footprint.json"),
            &run_out_dir.join("resource.footprint.json"),
        );
        copy_if_exists(
            &out_dir.join("resource.macos_footprint.steady.json"),
            &run_out_dir.join("resource.macos_footprint.steady.json"),
        );
        copy_if_exists(
            &out_dir.join("resource.vmmap_summary.txt"),
            &run_out_dir.join("resource.vmmap_summary.txt"),
        );
        copy_if_exists(
            &out_dir.join("resource.vmmap_summary.steady.txt"),
            &run_out_dir.join("resource.vmmap_summary.steady.txt"),
        );
        copy_if_exists(
            &out_dir.join("resource.vmmap_regions_sorted.steady.txt"),
            &run_out_dir.join("resource.vmmap_regions_sorted.steady.txt"),
        );

        let selected_bundle_json = bundle_artifact
            .and_then(|p| {
                if p.is_file() {
                    Some(p.clone())
                } else {
                    let candidate = p.join("bundle.json");
                    candidate.is_file().then_some(candidate)
                }
            })
            .or_else(|| {
                let candidate = run_out_dir.join("bundle.json");
                candidate.is_file().then_some(candidate)
            })
            .map(|p| p.display().to_string());

        let summary_json = serde_json::json!({
            "schema_version": 1,
            "kind": "repro_summary_repeat_v1",
            "selected_bundle_json": selected_bundle_json,
        });
        let summary_path = run_out_dir.join("repro.summary.json");
        let _ = write_json_value(&summary_path, &summary_json);
        let _ = crate::evidence_index::write_evidence_index(
            run_out_dir,
            &summary_path,
            Some(&summary_json),
        );
    }

    fn repeat_tooling_reason_code_from_error(err: &str) -> &'static str {
        if err.contains("timeout waiting for script result") {
            "timeout.tooling.script_result"
        } else {
            "tooling.repeat.failed"
        }
    }

    fn push_count(map: &mut std::collections::BTreeMap<String, u64>, key: &str) {
        if key.trim().is_empty() {
            return;
        }
        *map.entry(key.to_string()).or_default() += 1;
    }

    fn overlay_side_as_str(side: fret_diag_protocol::UiOverlaySideV1) -> &'static str {
        match side {
            fret_diag_protocol::UiOverlaySideV1::Top => "top",
            fret_diag_protocol::UiOverlaySideV1::Bottom => "bottom",
            fret_diag_protocol::UiOverlaySideV1::Left => "left",
            fret_diag_protocol::UiOverlaySideV1::Right => "right",
        }
    }

    fn summarize_script_result_evidence(
        result: &UiScriptResultV1,
        blocking_reason_counts: &mut std::collections::BTreeMap<String, u64>,
        overlay_chosen_side_counts: &mut std::collections::BTreeMap<String, u64>,
        ime_event_kind_counts: &mut std::collections::BTreeMap<String, u64>,
    ) -> (Option<serde_json::Value>, bool, bool) {
        let Some(e) = result.evidence.as_ref() else {
            return (None, false, false);
        };

        let mut evidence_present = false;
        let mut focus_mismatch = false;

        let mut trace_counts = std::collections::BTreeMap::<&str, u64>::new();
        trace_counts.insert(
            "selector_resolution_trace",
            e.selector_resolution_trace.len() as u64,
        );
        trace_counts.insert("hit_test_trace", e.hit_test_trace.len() as u64);
        trace_counts.insert("click_stable_trace", e.click_stable_trace.len() as u64);
        trace_counts.insert("bounds_stable_trace", e.bounds_stable_trace.len() as u64);
        trace_counts.insert("focus_trace", e.focus_trace.len() as u64);
        trace_counts.insert(
            "shortcut_routing_trace",
            e.shortcut_routing_trace.len() as u64,
        );
        trace_counts.insert(
            "overlay_placement_trace",
            e.overlay_placement_trace.len() as u64,
        );
        trace_counts.insert("web_ime_trace", e.web_ime_trace.len() as u64);
        trace_counts.insert("ime_event_trace", e.ime_event_trace.len() as u64);

        if trace_counts.values().any(|&n| n > 0) {
            evidence_present = true;
        }

        let mut hit_test_blocking = std::collections::BTreeMap::<String, u64>::new();
        for entry in &e.hit_test_trace {
            if let Some(reason) = entry.blocking_reason.as_deref() {
                push_count(&mut hit_test_blocking, reason);
                push_count(blocking_reason_counts, reason);
            }
        }

        let mut focus = serde_json::json!({
            "mismatch_count": 0u64,
            "text_input_snapshots": 0u64,
            "composing_true": 0u64,
        });
        let mut mismatch_count: u64 = 0;
        let mut text_input_snapshots: u64 = 0;
        let mut composing_true: u64 = 0;
        for entry in &e.focus_trace {
            if entry.matches_expected == Some(false) {
                mismatch_count += 1;
            }
            if let Some(snap) = entry.text_input_snapshot.as_ref() {
                text_input_snapshots += 1;
                if snap.is_composing {
                    composing_true += 1;
                }
            }
        }
        if mismatch_count > 0 {
            focus_mismatch = true;
        }
        focus["mismatch_count"] = serde_json::Value::Number(mismatch_count.into());
        focus["text_input_snapshots"] = serde_json::Value::Number(text_input_snapshots.into());
        focus["composing_true"] = serde_json::Value::Number(composing_true.into());

        let mut shortcut_outcomes = std::collections::BTreeMap::<String, u64>::new();
        for entry in &e.shortcut_routing_trace {
            push_count(&mut shortcut_outcomes, &entry.outcome);
        }

        let mut overlay_kinds = std::collections::BTreeMap::<&str, u64>::new();
        let mut overlay_chosen_sides = std::collections::BTreeMap::<String, u64>::new();
        for entry in &e.overlay_placement_trace {
            match entry {
                fret_diag_protocol::UiOverlayPlacementTraceEntryV1::AnchoredPanel {
                    chosen_side,
                    ..
                } => {
                    *overlay_kinds.entry("anchored_panel").or_default() += 1;
                    let side = overlay_side_as_str(*chosen_side);
                    push_count(&mut overlay_chosen_sides, side);
                    push_count(overlay_chosen_side_counts, side);
                }
                fret_diag_protocol::UiOverlayPlacementTraceEntryV1::PlacedRect { .. } => {
                    *overlay_kinds.entry("placed_rect").or_default() += 1;
                }
            }
        }

        let mut web_ime = serde_json::json!({
            "enabled_true": 0u64,
            "enabled_false": 0u64,
            "composing_true": 0u64,
        });
        let mut web_ime_enabled_true: u64 = 0;
        let mut web_ime_enabled_false: u64 = 0;
        let mut web_ime_composing_true: u64 = 0;
        for entry in &e.web_ime_trace {
            if entry.enabled {
                web_ime_enabled_true += 1;
            } else {
                web_ime_enabled_false += 1;
            }
            if entry.composing {
                web_ime_composing_true += 1;
            }
        }
        web_ime["enabled_true"] = serde_json::Value::Number(web_ime_enabled_true.into());
        web_ime["enabled_false"] = serde_json::Value::Number(web_ime_enabled_false.into());
        web_ime["composing_true"] = serde_json::Value::Number(web_ime_composing_true.into());

        let mut ime_kinds = std::collections::BTreeMap::<String, u64>::new();
        for entry in &e.ime_event_trace {
            push_count(&mut ime_kinds, &entry.kind);
            push_count(ime_event_kind_counts, &entry.kind);
        }

        let summary = serde_json::json!({
            "trace_counts": trace_counts,
            "hit_test": {
                "blocking_reason_counts": hit_test_blocking,
            },
            "focus": focus,
            "shortcuts": {
                "outcome_counts": shortcut_outcomes,
            },
            "overlay": {
                "kind_counts": overlay_kinds,
                "chosen_side_counts": overlay_chosen_sides,
            },
            "web_ime": web_ime,
            "ime_events": {
                "kind_counts": ime_kinds,
            },
        });

        (Some(summary), evidence_present, focus_mismatch)
    }

    for run_index in 0..repeat {
        if !reuse_process {
            child = maybe_launch_demo(
                &launch,
                &repeat_launch_env,
                &workspace_root,
                &resolved_ready_path,
                &resolved_exit_path,
                &launch_fs_transport_cfg,
                wants_screenshots,
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
        }

        clear_script_result_files(
            &resolved_script_result_path,
            &resolved_script_result_trigger_path,
        );

        let mut summary = run_script_and_wait(
            &src,
            &resolved_script_path,
            &resolved_script_trigger_path,
            &resolved_script_result_path,
            &resolved_script_result_trigger_path,
            timeout_ms,
            poll_ms,
        );

        if let Ok(s) = &summary
            && s.stage.as_deref() == Some("failed")
            && let Some(dir) =
                wait_for_failure_dump_bundle(&resolved_out_dir, s, timeout_ms, poll_ms)
            && let Some(name) = dir.file_name().and_then(|s| s.to_str())
            && let Ok(s) = summary.as_mut()
        {
            s.last_bundle_dir = Some(name.to_string());
        }

        if !reuse_process {
            stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
        }

        let entry = match summary {
            Ok(mut s) => {
                let bundle_artifact = crate::paths::wait_for_bundle_artifact_from_script_result(
                    &resolved_out_dir,
                    &s,
                    timeout_ms,
                    poll_ms,
                );
                if s.last_bundle_dir.is_none()
                    && let Some(name) = bundle_artifact.as_ref().and_then(|artifact| {
                        if artifact.is_dir() {
                            artifact.file_name().and_then(|v| v.to_str())
                        } else {
                            artifact
                                .parent()
                                .and_then(|parent| parent.file_name())
                                .and_then(|v| v.to_str())
                        }
                    })
                {
                    s.last_bundle_dir = Some(name.to_string());
                }
                let stage = s.stage.as_deref().unwrap_or("unknown").to_string();

                if stage == "failed" {
                    failed_runs += 1;
                    if first_failed_run.is_none() {
                        first_failed_run = Some(run_index);
                    }
                }
                *stage_counts.entry(stage.clone()).or_default() += 1;
                if let Some(code) = s.reason_code.as_deref().filter(|v| !v.is_empty()) {
                    *reason_code_counts.entry(code.to_string()).or_default() += 1;
                }

                let mut evidence: Option<serde_json::Value> = None;
                if let Some(full) = read_script_result_typed(&resolved_script_result_path) {
                    let (summary, present, focus_mismatch) = summarize_script_result_evidence(
                        &full,
                        &mut blocking_reason_counts,
                        &mut overlay_chosen_side_counts,
                        &mut ime_event_kind_counts,
                    );
                    evidence = summary;
                    if present {
                        evidence_present_runs.push(run_index);
                    }
                    if focus_mismatch {
                        focus_mismatch_runs.push(run_index);
                    }
                }

                let mut perf: Option<serde_json::Value> = None;
                if let Some(bundle_artifact) = bundle_artifact.as_ref()
                    && let Ok(report) = bundle_stats_from_path(
                        bundle_artifact,
                        1,
                        BundleStatsSort::Time,
                        BundleStatsOptions { warmup_frames },
                    )
                    && let Some(top) = report.top.first()
                {
                    let total_us = top.total_time_us;
                    match &worst_perf {
                        Some((_idx, best_total, _frame)) if *best_total >= total_us => {}
                        _ => {
                            worst_perf = Some((run_index, total_us, top.frame_id));
                        }
                    }
                    perf = Some(serde_json::json!({
                        "top_total_time_us": top.total_time_us,
                        "top_layout_time_us": top.layout_time_us,
                        "top_layout_engine_solve_time_us": top.layout_engine_solve_time_us,
                        "frame_id": top.frame_id,
                    }));
                }

                if let Some(bundle_artifact) = bundle_artifact.as_ref() {
                    let run_out_dir = if bundle_artifact.is_dir() {
                        bundle_artifact.clone()
                    } else {
                        bundle_artifact
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_else(|| resolved_out_dir.clone())
                    };
                    write_repeat_evidence_index_best_effort(
                        &run_out_dir,
                        &resolved_out_dir,
                        Some(bundle_artifact),
                    );
                }

                let mut lint: Option<serde_json::Value> = None;
                if let Some(bundle_artifact) = bundle_artifact.as_ref()
                    && let Ok(report) = lint_bundle_from_path(
                        bundle_artifact,
                        warmup_frames,
                        LintOptions {
                            all_test_ids_bounds: lint_all_test_ids_bounds,
                            eps_px: lint_eps_px,
                        },
                    )
                {
                    let warning_issues = report
                        .payload
                        .get("warning_issues")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    if report.error_issues > 0 {
                        lint_error_runs.push(run_index);
                    }

                    let mut counts_by_code =
                        std::collections::BTreeMap::<String, serde_json::Value>::new();
                    if let Some(map) = report
                        .payload
                        .get("counts_by_code")
                        .and_then(|v| v.as_object())
                    {
                        for (code, entry) in map {
                            let errors = entry.get("errors").and_then(|v| v.as_u64()).unwrap_or(0);
                            let warnings =
                                entry.get("warnings").and_then(|v| v.as_u64()).unwrap_or(0);
                            if errors == 0 && warnings == 0 {
                                continue;
                            }
                            counts_by_code.insert(
                                code.to_string(),
                                serde_json::json!({
                                    "errors": errors,
                                    "warnings": warnings,
                                }),
                            );
                            let entry = lint_counts_by_code
                                .entry(code.to_string())
                                .or_insert((0, 0));
                            entry.0 = entry.0.saturating_add(errors);
                            entry.1 = entry.1.saturating_add(warnings);
                        }
                    }
                    lint = Some(serde_json::json!({
                        "error_issues": report.error_issues,
                        "warning_issues": warning_issues,
                        "counts_by_code": counts_by_code,
                    }));
                }

                let mut compare_to_baseline: Option<serde_json::Value> = None;
                if compare_enabled && stage == "passed" {
                    if baseline_bundle.is_none() {
                        if let Some(bundle_artifact) = bundle_artifact.clone() {
                            baseline_run = Some(run_index);
                            baseline_bundle = Some(bundle_artifact);
                        }
                    } else if let (Some(base), Some(cur)) =
                        (baseline_bundle.as_ref(), bundle_artifact.as_ref())
                    {
                        let report = compare_bundles(
                            base,
                            cur,
                            CompareOptions {
                                warmup_frames,
                                eps_px: compare_eps_px,
                                ignore_bounds: compare_ignore_bounds,
                                ignore_scene_fingerprint: compare_ignore_scene_fingerprint,
                            },
                        )?;

                        let mut kinds: std::collections::BTreeMap<&str, u64> =
                            std::collections::BTreeMap::new();
                        let mut semantics_diffs: u64 = 0;
                        let mut layout_diffs: u64 = 0;
                        let mut scene_fp_mismatch: u64 = 0;
                        for d in &report.diffs {
                            *kinds.entry(d.kind).or_default() += 1;
                            if d.kind == "scene_fingerprint_mismatch" {
                                scene_fp_mismatch += 1;
                                continue;
                            }
                            if d.kind == "node_field_mismatch" && d.field == Some("bounds") {
                                layout_diffs += 1;
                                continue;
                            }
                            semantics_diffs += 1;
                        }

                        if !report.ok {
                            differing_runs += 1;
                            if first_differing_run.is_none() {
                                first_differing_run = Some(run_index);
                            }
                        }

                        compare_to_baseline = Some(serde_json::json!({
                            "ok": report.ok,
                            "diffs": report.diffs.len(),
                            "semantics_diffs": semantics_diffs,
                            "layout_diffs": layout_diffs,
                            "scene_fingerprint_mismatch": scene_fp_mismatch,
                            "diff_kinds": kinds,
                        }));
                    }
                }

                serde_json::json!({
                    "index": run_index,
                    "stage": stage,
                    "run_id": s.run_id,
                    "reason_code": s.reason_code,
                    "reason": s.reason,
                    "last_bundle_dir": s.last_bundle_dir,
                    "bundle_json": bundle_artifact.as_ref().map(|p| p.display().to_string()),
                    "bundle_artifact": bundle_artifact.as_ref().map(|p| p.display().to_string()),
                    "perf": perf,
                    "lint": lint,
                    "evidence": evidence,
                    "compare_to_baseline": compare_to_baseline,
                })
            }
            Err(err) => {
                failed_runs += 1;
                if first_failed_run.is_none() {
                    first_failed_run = Some(run_index);
                }
                *stage_counts.entry("error".to_string()).or_default() += 1;
                tooling_error_reason_code = read_tooling_reason_code(&resolved_script_result_path)
                    .or_else(|| Some(repeat_tooling_reason_code_from_error(&err).to_string()));
                serde_json::json!({
                    "index": run_index,
                    "stage": "error",
                    "reason_code": tooling_error_reason_code,
                    "error": err,
                })
            }
        };

        runs.push(entry);
    }

    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);

    fn quantile_sorted(values: &[u64], q: f64) -> u64 {
        if values.is_empty() {
            return 0;
        }
        let q = q.clamp(0.0, 1.0);
        let idx = ((values.len() - 1) as f64 * q).round() as usize;
        *values.get(idx).unwrap_or(values.last().unwrap_or(&0))
    }

    fn read_u64_resource(resources: &serde_json::Value, key: &str) -> Option<u64> {
        let process = resources
            .get("process_footprint")
            .unwrap_or(&serde_json::Value::Null);
        let bundle = resources
            .get("bundle_last_frame_stats")
            .unwrap_or(&serde_json::Value::Null);
        process
            .get(key)
            .and_then(|v| v.as_u64())
            .or_else(|| bundle.get(key).and_then(|v| v.as_u64()))
    }

    let mut memory_checks: Option<serde_json::Value> = None;
    let mut memory_check_failures: Vec<serde_json::Value> = Vec::new();

    if !check_memory_p90_max.is_empty() {
        let mut sample_dirs: Vec<PathBuf> = Vec::new();
        let mut q: Vec<PathBuf> = vec![resolved_out_dir.clone()];
        while let Some(dir) = q.pop() {
            let evidence = dir.join("evidence.index.json");
            if evidence.is_file() {
                sample_dirs.push(dir);
                continue;
            }
            let entries = match std::fs::read_dir(&dir) {
                Ok(v) => v,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    q.push(path);
                }
            }
        }

        let mut checks_out: Vec<serde_json::Value> = Vec::new();
        if sample_dirs.is_empty() {
            memory_check_failures.push(serde_json::json!({
                "kind": "no_samples",
                "reason": "no evidence.index.json found under repeat out_dir; ensure bundles are written and evidence is materialized per run",
            }));
        }
        for (key, threshold) in &check_memory_p90_max {
            let mut values: Vec<u64> = Vec::new();
            let mut missing: u64 = 0;

            for dir in &sample_dirs {
                let evidence_index_path = dir.join("evidence.index.json");
                let Some(v) = read_json_value(&evidence_index_path) else {
                    missing = missing.saturating_add(1);
                    continue;
                };
                let Some(resources) = v.get("resources") else {
                    missing = missing.saturating_add(1);
                    continue;
                };

                match read_u64_resource(resources, key) {
                    Some(n) => values.push(n),
                    None => missing = missing.saturating_add(1),
                }
            }

            values.sort_unstable();
            let p90 = quantile_sorted(&values, 0.90);
            let max = *values.last().unwrap_or(&0);

            let mut failures: Vec<serde_json::Value> = Vec::new();
            if missing > 0 {
                failures.push(serde_json::json!({
                    "kind": "missing_field",
                    "missing_samples": missing,
                }));
            }
            if values.is_empty() {
                failures.push(serde_json::json!({
                    "kind": "no_values",
                    "reason": "field missing in all samples",
                }));
            }
            if !values.is_empty() && p90 > *threshold {
                failures.push(serde_json::json!({
                    "kind": "p90_exceeded",
                    "threshold": threshold,
                    "observed_p90": p90,
                }));
            }

            if !failures.is_empty() {
                memory_check_failures.push(serde_json::json!({
                    "key": key,
                    "threshold": threshold,
                    "observed_p90": p90,
                    "observed_max": max,
                    "samples_present": values.len(),
                    "samples_missing": missing,
                    "failures": failures,
                }));
            }

            checks_out.push(serde_json::json!({
                "key": key,
                "threshold": threshold,
                "observed_p90": p90,
                "observed_max": max,
                "samples_present": values.len(),
                "samples_missing": missing,
            }));
        }

        let out_path = resolved_out_dir.join("check.repeat_memory_p90_max.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "repeat_memory_p90_max",
            "out_dir": resolved_out_dir.display().to_string(),
            "samples_total": sample_dirs.len(),
            "checks": checks_out,
            "failures": memory_check_failures,
        });
        let _ = write_json_value(&out_path, &payload);
        memory_checks = Some(serde_json::json!({
            "evidence": out_path.display().to_string(),
            "failures_len": memory_check_failures.len(),
        }));
    }

    let status = if failed_runs == 0 && differing_runs == 0 {
        "passed"
    } else {
        "failed"
    };
    let status = if status == "passed" && memory_check_failures.is_empty() {
        "passed"
    } else {
        "failed"
    };

    let lint_counts_by_code_json: std::collections::BTreeMap<String, serde_json::Value> =
        lint_counts_by_code
            .into_iter()
            .map(|(code, (errors, warnings))| {
                (
                    code,
                    serde_json::json!({
                        "errors": errors,
                        "warnings": warnings,
                    }),
                )
            })
            .collect();
    let highlights = serde_json::json!({
        "stage_counts": stage_counts,
        "reason_code_counts": reason_code_counts,
        "first_failed_run": first_failed_run,
        "first_differing_run": first_differing_run,
        "worst_perf": worst_perf.map(|(idx, total_us, frame_id)| serde_json::json!({
            "run": idx,
            "top_total_time_us": total_us,
            "frame_id": frame_id,
        })),
        "lint_error_runs": lint_error_runs,
        "lint_counts_by_code": lint_counts_by_code_json,
        "evidence_present_runs": evidence_present_runs,
        "focus_mismatch_runs": focus_mismatch_runs,
        "blocking_reason_counts": blocking_reason_counts,
        "overlay_chosen_side_counts": overlay_chosen_side_counts,
        "ime_event_kind_counts": ime_event_kind_counts,
    });
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "script": src.display().to_string(),
        "repeat": repeat,
        "baseline_run": baseline_run,
        "highlights": highlights,
        "memory_checks": memory_checks,
        "error_reason_code": tooling_error_reason_code,
        "options": {
            "warmup_frames": warmup_frames,
            "compare_eps_px": compare_eps_px,
            "compare_ignore_bounds": compare_ignore_bounds,
            "compare_enabled": compare_enabled,
            "compare_ignore_scene_fingerprint": compare_ignore_scene_fingerprint,
        },
        "failed_runs": failed_runs,
        "differing_runs": differing_runs,
        "runs": runs,
    });

    let out_path = resolved_out_dir.join("repeat.summary.json");
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&out_path, pretty.as_bytes()).map_err(|e| e.to_string())?;

    if stats_json {
        println!("{pretty}");
    } else {
        println!("{}", out_path.display());
    }

    if status != "passed" {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merged_repeat_launch_env_preserves_explicit_values() {
        let merged = merged_repeat_launch_env(
            &[(
                "FRET_UI_GALLERY_START_PAGE".to_string(),
                "overlay".to_string(),
            )],
            vec![
                ("FRET_UI_GALLERY_START_PAGE".to_string(), "card".to_string()),
                ("FRET_UI_GALLERY_NAV_QUERY".to_string(), "card".to_string()),
            ],
        );

        assert_eq!(
            merged,
            vec![
                (
                    "FRET_UI_GALLERY_START_PAGE".to_string(),
                    "overlay".to_string()
                ),
                ("FRET_UI_GALLERY_NAV_QUERY".to_string(), "card".to_string()),
            ]
        );
    }
}
