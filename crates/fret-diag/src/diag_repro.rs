use super::*;

pub(crate) struct ReproCmdContext {
    pub rest: Vec<String>,
    pub workspace_root: PathBuf,
    pub resolved_out_dir: PathBuf,
    pub resolved_ready_path: PathBuf,
    pub resolved_exit_path: PathBuf,
    pub resolved_script_path: PathBuf,
    pub resolved_script_trigger_path: PathBuf,
    pub resolved_script_result_path: PathBuf,
    pub resolved_script_result_trigger_path: PathBuf,
    pub fs_transport_cfg: crate::transport::FsDiagTransportConfig,
    pub pack_out: Option<PathBuf>,
    pub pack_include_root_artifacts: bool,
    pub pack_include_triage: bool,
    pub pack_include_screenshots: bool,
    pub stats_top: usize,
    pub sort_override: Option<BundleStatsSort>,
    pub warmup_frames: u64,
    pub timeout_ms: u64,
    pub poll_ms: u64,
    pub trace_chrome: bool,
    pub launch: Option<Vec<String>>,
    pub launch_env: Vec<(String, String)>,
    pub launch_high_priority: bool,
    pub with_tracy: bool,
    pub with_renderdoc: bool,
    pub renderdoc_after_frames: Option<u32>,
    pub renderdoc_markers: Vec<String>,
    pub renderdoc_no_outputs_png: bool,
    pub resource_footprint_thresholds: ResourceFootprintThresholds,
    pub check_redraw_hitches_max_total_ms_threshold: Option<u64>,
    pub checks: diag_run::RunChecks,
}

fn read_tooling_reason_code(path: &Path) -> Option<String> {
    read_json_value(path).and_then(|v| {
        v.get("reason_code")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    })
}

fn resolve_repro_scripts(rest: &[String], workspace_root: &Path) -> (Vec<PathBuf>, Option<String>) {
    if rest.len() == 1 && rest[0] == "ui-gallery" {
        (
            diag_suite_scripts::ui_gallery_suite_scripts()
                .into_iter()
                .map(|p| resolve_path(workspace_root, PathBuf::from(p)))
                .collect(),
            Some("ui-gallery".to_string()),
        )
    } else if rest.len() == 1 && rest[0] == "ui-gallery-code-editor" {
        (
            diag_suite_scripts::ui_gallery_code_editor_suite_scripts()
                .into_iter()
                .map(|p| resolve_path(workspace_root, PathBuf::from(p)))
                .collect(),
            Some("ui-gallery-code-editor".to_string()),
        )
    } else if rest.len() == 1 && rest[0] == "docking-arbitration" {
        (
            diag_suite_scripts::docking_arbitration_suite_scripts()
                .into_iter()
                .map(|p| resolve_path(workspace_root, PathBuf::from(p)))
                .collect(),
            Some("docking-arbitration".to_string()),
        )
    } else {
        (
            rest.iter()
                .map(|p| resolve_path(workspace_root, PathBuf::from(p)))
                .collect(),
            None,
        )
    }
}

struct PreparedReproLaunch {
    launch: Option<Vec<String>>,
    launch_env: Vec<(String, String)>,
    tracy_feature_injected: bool,
    tracy_env_enabled: bool,
    renderdoc_capture_dir: Option<PathBuf>,
    renderdoc_autocapture_after_frames: Option<u32>,
}

fn prepare_repro_launch(
    resolved_out_dir: &Path,
    launch: Option<Vec<String>>,
    launch_env: Vec<(String, String)>,
    check_redraw_hitches_max_total_ms_threshold: Option<u64>,
    with_tracy: bool,
    with_renderdoc: bool,
    renderdoc_after_frames: Option<u32>,
) -> PreparedReproLaunch {
    let mut repro_launch = launch;
    let mut repro_launch_env = launch_env;
    let _ = ensure_env_var(&mut repro_launch_env, "FRET_DIAG_RENDERER_PERF", "1");

    if check_redraw_hitches_max_total_ms_threshold.is_some() {
        let _ = ensure_env_var(&mut repro_launch_env, "FRET_REDRAW_HITCH_LOG", "1");
        let _ = ensure_env_var(
            &mut repro_launch_env,
            "FRET_REDRAW_HITCH_LOG_PATH",
            "redraw_hitches.log",
        );
    }

    let mut tracy_feature_injected: bool = false;
    let mut tracy_env_enabled: bool = false;
    if with_tracy {
        tracy_env_enabled = ensure_env_var(&mut repro_launch_env, "FRET_TRACY", "1");
        if let Some(cmd) = repro_launch.as_mut() {
            tracy_feature_injected = cargo_run_inject_feature(cmd, "fret-bootstrap/tracy");
        }

        let note = "\
# Tracy capture (best-effort)\n\
\n\
This repro was run with `FRET_TRACY=1` (and may have auto-injected `--features fret-bootstrap/tracy` when the launch command was `cargo run`).\n\
\n\
Notes:\n\
- Tracy requires running the target with the `fret-bootstrap/tracy` feature enabled.\n\
- The capture file is not recorded automatically by `fretboard` yet. Use the Tracy UI to connect and save a capture.\n\
\n\
See: `docs/tracy.md`.\n";
        let _ = std::fs::write(resolved_out_dir.join("tracy.note.md"), note);
    }

    let mut renderdoc_capture_dir: Option<PathBuf> = None;
    let mut renderdoc_autocapture_after_frames: Option<u32> = None;
    if with_renderdoc {
        let after = renderdoc_after_frames.unwrap_or(60);
        let capture_dir = resolved_out_dir.join("renderdoc");
        let _ = std::fs::create_dir_all(&capture_dir);

        let _ = ensure_env_var(&mut repro_launch_env, "FRET_RENDERDOC", "1");
        let _ = ensure_env_var(
            &mut repro_launch_env,
            "FRET_RENDERDOC_CAPTURE_DIR",
            capture_dir.to_string_lossy().as_ref(),
        );
        let _ = ensure_env_var(
            &mut repro_launch_env,
            "FRET_RENDERDOC_AUTOCAPTURE_AFTER_FRAMES",
            &after.to_string(),
        );

        renderdoc_capture_dir = Some(capture_dir);
        renderdoc_autocapture_after_frames = Some(after);
    }

    PreparedReproLaunch {
        launch: repro_launch,
        launch_env: repro_launch_env,
        tracy_feature_injected,
        tracy_env_enabled,
        renderdoc_capture_dir,
        renderdoc_autocapture_after_frames,
    }
}

pub(crate) fn cmd_repro(ctx: ReproCmdContext) -> Result<(), String> {
    let ReproCmdContext {
        rest,
        workspace_root,
        resolved_out_dir,
        resolved_ready_path,
        resolved_exit_path,
        resolved_script_path: _resolved_script_path,
        resolved_script_trigger_path: _resolved_script_trigger_path,
        resolved_script_result_path,
        resolved_script_result_trigger_path: _resolved_script_result_trigger_path,
        fs_transport_cfg,
        pack_out,
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
        stats_top,
        sort_override,
        warmup_frames,
        timeout_ms,
        poll_ms,
        trace_chrome,
        launch,
        launch_env,
        launch_high_priority,
        with_tracy,
        with_renderdoc,
        renderdoc_after_frames,
        renderdoc_markers,
        renderdoc_no_outputs_png,
        resource_footprint_thresholds,
        check_redraw_hitches_max_total_ms_threshold,
        checks,
    } = ctx;

    let diag_run::RunChecks {
        check_chart_sampling_window_shifts_min,
        check_dock_drag_min,
        check_drag_cache_root_paint_only_test_id,
        check_gc_sweep_liveness,
        check_hover_layout_max,
        check_idle_no_paint_min,
        check_layout_fast_path_min,
        check_node_graph_cull_window_shifts_max,
        check_node_graph_cull_window_shifts_min,
        check_notify_hotspot_file_max,
        check_overlay_synthesis_min,
        check_pixels_changed_test_id,
        check_prepaint_actions_min,
        check_retained_vlist_attach_detach_max,
        check_retained_vlist_keep_alive_budget,
        check_retained_vlist_keep_alive_reuse_min,
        check_retained_vlist_reconcile_no_notify_min,
        check_semantics_changed_repainted,
        check_stale_paint_eps,
        check_stale_paint_test_id,
        check_stale_scene_eps,
        check_stale_scene_test_id,
        check_ui_gallery_code_editor_a11y_composition,
        check_ui_gallery_code_editor_a11y_composition_drag,
        check_ui_gallery_code_editor_a11y_composition_wrap,
        check_ui_gallery_code_editor_a11y_composition_wrap_scroll,
        check_ui_gallery_code_editor_a11y_selection,
        check_ui_gallery_code_editor_a11y_selection_wrap,
        check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection,
        check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll,
        check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
        check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed,
        check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit,
        check_ui_gallery_code_editor_torture_folds_placeholder_present,
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped,
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations,
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed,
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap,
        check_ui_gallery_code_editor_torture_geom_fallbacks_low,
        check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit,
        check_ui_gallery_code_editor_torture_inlays_present,
        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped,
        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations,
        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed,
        check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap,
        check_ui_gallery_code_editor_torture_marker_present,
        check_ui_gallery_code_editor_torture_read_only_blocks_edits,
        check_ui_gallery_code_editor_torture_undo_redo,
        check_ui_gallery_code_editor_word_boundary,
        check_ui_gallery_markdown_editor_source_a11y_composition,
        check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap,
        check_ui_gallery_markdown_editor_source_disabled_blocks_edits,
        check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds,
        check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit,
        check_ui_gallery_markdown_editor_source_folds_placeholder_present,
        check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap,
        check_ui_gallery_markdown_editor_source_folds_toggle_stable,
        check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit,
        check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable,
        check_ui_gallery_markdown_editor_source_inlays_present,
        check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap,
        check_ui_gallery_markdown_editor_source_inlays_toggle_stable,
        check_ui_gallery_markdown_editor_source_line_boundary_triple_click,
        check_ui_gallery_markdown_editor_source_read_only_blocks_edits,
        check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable,
        check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable,
        check_ui_gallery_markdown_editor_source_word_boundary,
        check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change,
        check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change,
        check_ui_gallery_text_mixed_script_bundled_fallback_conformance,
        check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps,
        check_ui_gallery_web_ime_bridge_enabled,
        check_view_cache_reuse_min,
        check_view_cache_reuse_stable_min,
        check_viewport_capture_min,
        check_viewport_input_min,
        check_vlist_policy_key_stable,
        check_vlist_visible_range_refreshes_max,
        check_vlist_visible_range_refreshes_min,
        check_vlist_window_shifts_escape_max,
        check_vlist_window_shifts_explainable,
        check_vlist_window_shifts_have_prepaint_actions,
        check_vlist_window_shifts_non_retained_max,
        check_vlist_window_shifts_prefetch_max,
        check_wheel_scroll_hit_changes_test_id,
        check_wheel_scroll_test_id,
        check_windowed_rows_offset_changes_eps,
        check_windowed_rows_offset_changes_min,
        check_windowed_rows_visible_start_changes_repainted,
        dump_semantics_changed_repainted_json,
    } = checks;

    if rest.is_empty() {
        return Err(
            "missing script path or suite name (try: fretboard diag repro ui-gallery | fretboard diag repro ./script.json)"
                .to_string(),
        );
    }

    let mut pack_defaults = (
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
    );
    if !pack_defaults.0 && !pack_defaults.1 && !pack_defaults.2 {
        pack_defaults = (true, true, true);
    }

    let (scripts, suite_name) = resolve_repro_scripts(&rest, &workspace_root);

    let summary_path = resolved_out_dir.join("repro.summary.json");

    let mut required_caps: Vec<String> = Vec::new();
    for src in scripts.iter() {
        required_caps.extend(script_required_capabilities(src));
    }
    required_caps.sort();
    required_caps.dedup();

    let mut overall_reason_code: Option<String> = None;

    let prepared_launch = prepare_repro_launch(
        &resolved_out_dir,
        launch,
        launch_env,
        check_redraw_hitches_max_total_ms_threshold,
        with_tracy,
        with_renderdoc,
        renderdoc_after_frames,
    );

    let mut child = match maybe_launch_demo(
        &prepared_launch.launch,
        &prepared_launch.launch_env,
        &workspace_root,
        &resolved_out_dir,
        &resolved_ready_path,
        &resolved_exit_path,
        pack_defaults.2
            || check_pixels_changed_test_id.is_some()
            || scripts.iter().any(|p| script_requests_screenshots(p)),
        timeout_ms,
        poll_ms,
        launch_high_priority,
    ) {
        Ok(v) => v,
        Err(err) => {
            write_tooling_failure_script_result(
                &resolved_script_result_path,
                "tooling.launch.failed",
                &err,
                "tooling_error",
                Some("maybe_launch_demo".to_string()),
            );
            let payload = serde_json::json!({
                "schema_version": 1,
                "generated_unix_ms": now_unix_ms(),
                "out_dir": resolved_out_dir.display().to_string(),
                "suite": suite_name,
                "scripts": scripts.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "capabilities": serde_json::json!({
                    "required": required_caps,
                    "available": [],
                    "check_file": None::<String>,
                }),
                "error_reason_code": "tooling.launch.failed",
                "error": err,
            });
            let _ = write_json_value(&summary_path, &payload);
            return Err("repro setup failed (see repro.summary.json)".to_string());
        }
    };

    let connected = match connect_filesystem_tooling(
        &fs_transport_cfg,
        &resolved_ready_path,
        false,
        timeout_ms,
        poll_ms,
    ) {
        Ok(v) => v,
        Err(err) => {
            write_tooling_failure_script_result(
                &resolved_script_result_path,
                "tooling.connect.failed",
                &err,
                "tooling_error",
                Some("connect_filesystem_tooling".to_string()),
            );
            let payload = serde_json::json!({
                "schema_version": 1,
                "generated_unix_ms": now_unix_ms(),
                "out_dir": resolved_out_dir.display().to_string(),
                "suite": suite_name,
                "scripts": scripts.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "capabilities": serde_json::json!({
                    "required": required_caps,
                    "available": [],
                    "check_file": None::<String>,
                }),
                "error_reason_code": "tooling.connect.failed",
                "error": err,
            });
            let _ = write_json_value(&summary_path, &payload);
            if prepared_launch.launch.is_some() {
                let _ = stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
            }
            return Err("repro setup failed (see repro.summary.json)".to_string());
        }
    };
    let available_caps = connected.available_caps.clone();
    let capabilities_check_path = resolved_out_dir.join("check.capabilities.json");

    let mut repro_process_footprint: Option<serde_json::Value> = None;
    let mut resource_footprint_gate: Option<ResourceFootprintGateResult> = None;
    let mut redraw_hitches_gate: Option<RedrawHitchesGateResult> = None;

    let mut run_rows: Vec<serde_json::Value> = Vec::new();
    let mut selected_bundle_path: Option<PathBuf> = None;
    let mut last_script_result: Option<ScriptResultSummary> = None;
    let mut overall_error: Option<String> = None;
    let mut pack_items: Vec<ReproPackItem> = Vec::new();

    if !required_caps.is_empty()
        && let Err(err) = gate_required_capabilities_with_script_result(
            &capabilities_check_path,
            &resolved_script_result_path,
            &required_caps,
            &available_caps,
            "filesystem",
        )
    {
        overall_reason_code = read_tooling_reason_code(&resolved_script_result_path)
            .or_else(|| Some("capability.missing".to_string()));
        overall_error = Some(err);
    }

    for (idx, src) in scripts.into_iter().enumerate() {
        if overall_error.is_some() {
            break;
        }
        let script_json_bytes = match std::fs::read(&src) {
            Ok(v) => v,
            Err(e) => {
                let err = e.to_string();
                overall_reason_code = Some("tooling.script.read_failed".to_string());
                write_tooling_failure_script_result(
                    &resolved_script_result_path,
                    "tooling.script.read_failed",
                    &err,
                    "tooling_error",
                    Some(src.display().to_string()),
                );
                overall_error = Some(err);
                break;
            }
        };
        let script_json: serde_json::Value = match serde_json::from_slice(&script_json_bytes) {
            Ok(v) => v,
            Err(e) => {
                let err = e.to_string();
                overall_reason_code = Some("tooling.script.parse_failed".to_string());
                write_tooling_failure_script_result(
                    &resolved_script_result_path,
                    "tooling.script.parse_failed",
                    &err,
                    "tooling_error",
                    Some(src.display().to_string()),
                );
                overall_error = Some(err);
                break;
            }
        };

        let (raw_result, _bundle_path) = match run_script_over_transport(
            &resolved_out_dir,
            &connected,
            script_json,
            false,
            trace_chrome,
            None,
            None,
            timeout_ms,
            poll_ms,
            &resolved_script_result_path,
            &capabilities_check_path,
        ) {
            Ok(v) => v,
            Err(err) => {
                overall_reason_code = read_tooling_reason_code(&resolved_script_result_path)
                    .or_else(|| Some("tooling.run.failed".to_string()));
                overall_error = Some(err);
                break;
            }
        };

        let stage = match raw_result.stage {
            fret_diag_protocol::UiScriptStageV1::Passed => "passed",
            fret_diag_protocol::UiScriptStageV1::Failed => "failed",
            fret_diag_protocol::UiScriptStageV1::Queued => "queued",
            fret_diag_protocol::UiScriptStageV1::Running => "running",
        };

        let mut result = ScriptResultSummary {
            run_id: raw_result.run_id,
            stage: Some(stage.to_string()),
            step_index: raw_result.step_index.map(|n| n as u64),
            reason_code: raw_result.reason_code.clone(),
            reason: raw_result.reason.clone(),
            last_bundle_dir: raw_result.last_bundle_dir.clone(),
        };

        if result.stage.as_deref() == Some("failed")
            && let Some(dir) =
                wait_for_failure_dump_bundle(&resolved_out_dir, &result, timeout_ms, poll_ms)
            && let Some(name) = dir.file_name().and_then(|s| s.to_str())
        {
            result.last_bundle_dir = Some(name.to_string());
        }
        last_script_result = Some(result.clone());

        let dump_label = {
            let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("script");
            let mut sanitized: String = stem
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect();
            while sanitized.contains("--") {
                sanitized = sanitized.replace("--", "-");
            }
            sanitized = sanitized.trim_matches('-').to_string();
            if sanitized.is_empty() {
                sanitized = "script".to_string();
            }
            let mut label = format!("repro-{idx:04}-{sanitized}");
            if label.len() > 80 {
                label.truncate(80);
                label = label.trim_matches('-').to_string();
            }
            label
        };

        let mut bundle_path = wait_for_bundle_json_from_script_result(
            &resolved_out_dir,
            &result,
            timeout_ms,
            poll_ms,
        );
        if bundle_path.is_none() {
            match dump_bundle_over_transport(
                &resolved_out_dir,
                &connected,
                Some(dump_label.as_str()),
                None,
                timeout_ms,
                poll_ms,
            ) {
                Ok(p) => {
                    bundle_path = Some(p);
                }
                Err(err) => {
                    let code = if err.contains("timed out waiting") {
                        "timeout.tooling.bundle_dump"
                    } else {
                        "tooling.bundle_dump.failed"
                    };
                    overall_reason_code = Some(code.to_string());
                    mark_existing_script_result_tooling_failure(
                        &resolved_out_dir,
                        &resolved_script_result_path,
                        code,
                        &err,
                        "tooling_bundle_dump_failed",
                        Some(src.display().to_string()),
                    );
                    overall_error = Some(err);
                    break;
                }
            }
        }

        if let Some(bundle_path) = bundle_path.as_ref() {
            pack_items.push(ReproPackItem {
                script_path: src.clone(),
                bundle_json: bundle_path.clone(),
            });
        }

        if result.stage.as_deref() == Some("failed") && bundle_path.is_some() {
            selected_bundle_path = bundle_path.clone();
        }
        if selected_bundle_path.is_none() {
            selected_bundle_path = bundle_path.clone();
        }

        run_rows.push(serde_json::json!({
            "script_path": src.display().to_string(),
            "run_id": result.run_id,
            "stage": result.stage,
            "step_index": result.step_index,
            "reason": result.reason,
            "last_bundle_dir": result.last_bundle_dir,
            "bundle_json": bundle_path.as_ref().map(|p| p.display().to_string()),
        }));

        if result.stage.as_deref() == Some("passed") {
            let wants_post_run_checks_for_script = check_stale_paint_test_id.is_some()
                || check_stale_scene_test_id.is_some()
                || check_idle_no_paint_min.is_some()
                || check_pixels_changed_test_id.is_some()
                || check_ui_gallery_code_editor_torture_marker_present
                || check_ui_gallery_code_editor_torture_undo_redo
                || check_ui_gallery_code_editor_torture_geom_fallbacks_low
                || check_ui_gallery_code_editor_torture_read_only_blocks_edits
                || check_ui_gallery_markdown_editor_source_read_only_blocks_edits
                || check_ui_gallery_markdown_editor_source_disabled_blocks_edits
                || check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable
                || check_ui_gallery_markdown_editor_source_word_boundary
                || check_ui_gallery_web_ime_bridge_enabled
                || check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps
                || check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change
                || check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change
                || check_ui_gallery_text_mixed_script_bundled_fallback_conformance
                || check_ui_gallery_markdown_editor_source_line_boundary_triple_click
                || check_ui_gallery_markdown_editor_source_a11y_composition
                || check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap
                || check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable
                || check_ui_gallery_markdown_editor_source_folds_toggle_stable
                || check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds
                || check_ui_gallery_markdown_editor_source_folds_placeholder_present
                || check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap
                || check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit
                || check_ui_gallery_markdown_editor_source_inlays_toggle_stable
                || check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable
                || check_ui_gallery_markdown_editor_source_inlays_present
                || check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap
                || check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit
                || check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit
                || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
                || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations
                || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed
                || check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed
                || check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed
                || check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll
                || check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection
                || check_ui_gallery_code_editor_torture_folds_placeholder_present
                || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap
                || check_ui_gallery_code_editor_torture_inlays_present
                || check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit
                || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped
                || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations
                || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed
                || check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap
                || check_ui_gallery_code_editor_word_boundary
                || check_ui_gallery_code_editor_a11y_selection
                || check_ui_gallery_code_editor_a11y_composition
                || check_ui_gallery_code_editor_a11y_selection_wrap
                || check_ui_gallery_code_editor_a11y_composition_wrap
                || check_ui_gallery_code_editor_a11y_composition_wrap_scroll
                || check_ui_gallery_code_editor_a11y_composition_drag
                || check_semantics_changed_repainted
                || check_wheel_scroll_test_id.is_some()
                || check_wheel_scroll_hit_changes_test_id.is_some()
                || check_prepaint_actions_min.is_some()
                || check_chart_sampling_window_shifts_min.is_some()
                || check_node_graph_cull_window_shifts_min.is_some()
                || check_node_graph_cull_window_shifts_max.is_some()
                || check_vlist_visible_range_refreshes_min.is_some()
                || check_vlist_visible_range_refreshes_max.is_some()
                || check_vlist_window_shifts_explainable
                || check_vlist_window_shifts_have_prepaint_actions
                || check_vlist_window_shifts_non_retained_max.is_some()
                || check_vlist_window_shifts_prefetch_max.is_some()
                || check_vlist_window_shifts_escape_max.is_some()
                || check_vlist_policy_key_stable
                || check_windowed_rows_offset_changes_min.is_some()
                || check_layout_fast_path_min.is_some()
                || check_drag_cache_root_paint_only_test_id.is_some()
                || check_hover_layout_max.is_some()
                || check_gc_sweep_liveness
                || !check_notify_hotspot_file_max.is_empty()
                || check_view_cache_reuse_min.is_some()
                || check_view_cache_reuse_stable_min.is_some()
                || check_overlay_synthesis_min.is_some()
                || check_viewport_input_min.is_some()
                || check_dock_drag_min.is_some()
                || check_viewport_capture_min.is_some()
                || check_retained_vlist_reconcile_no_notify_min.is_some()
                || check_retained_vlist_attach_detach_max.is_some()
                || check_retained_vlist_keep_alive_reuse_min.is_some()
                || check_retained_vlist_keep_alive_budget.is_some();

            if wants_post_run_checks_for_script {
                let Some(bundle_path) = bundle_path.as_ref() else {
                    overall_reason_code =
                        Some("tooling.bundle_missing_for_post_run_checks".to_string());
                    overall_error = Some(
                        "script passed but no bundle.json was found (required for post-run checks)"
                            .to_string(),
                    );
                    break;
                };

                if let Err(err) = apply_post_run_checks(
                    bundle_path,
                    &resolved_out_dir,
                    check_idle_no_paint_min,
                    check_stale_paint_test_id.as_deref(),
                    check_stale_paint_eps,
                    check_stale_scene_test_id.as_deref(),
                    check_stale_scene_eps,
                    check_pixels_changed_test_id.as_deref(),
                    check_ui_gallery_code_editor_torture_marker_present,
                    check_ui_gallery_code_editor_torture_undo_redo,
                    check_ui_gallery_code_editor_torture_geom_fallbacks_low,
                    check_ui_gallery_code_editor_torture_read_only_blocks_edits,
                    check_ui_gallery_markdown_editor_source_read_only_blocks_edits,
                    check_ui_gallery_markdown_editor_source_disabled_blocks_edits,
                    check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable,
                    check_ui_gallery_markdown_editor_source_word_boundary,
                    check_ui_gallery_web_ime_bridge_enabled,
                    check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps,
                    check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change,
                    check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change,
                    check_ui_gallery_text_mixed_script_bundled_fallback_conformance,
                    check_ui_gallery_markdown_editor_source_line_boundary_triple_click,
                    check_ui_gallery_markdown_editor_source_a11y_composition,
                    check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap,
                    check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable,
                    check_ui_gallery_markdown_editor_source_folds_toggle_stable,
                    check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds,
                    check_ui_gallery_markdown_editor_source_folds_placeholder_present,
                    check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap,
                    check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit,
                    check_ui_gallery_markdown_editor_source_inlays_toggle_stable,
                    check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable,
                    check_ui_gallery_markdown_editor_source_inlays_present,
                    check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap,
                    check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit,
                    check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit,
                    check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped,
                    check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations,
                    check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed,
                    check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed,
                    check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
                    check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll,
                    check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection,
                    check_ui_gallery_code_editor_torture_folds_placeholder_present,
                    check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap,
                    check_ui_gallery_code_editor_torture_inlays_present,
                    check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit,
                    check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped,
                    check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations,
                    check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed,
                    check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap,
                    check_ui_gallery_code_editor_word_boundary,
                    check_ui_gallery_code_editor_a11y_selection,
                    check_ui_gallery_code_editor_a11y_composition,
                    check_ui_gallery_code_editor_a11y_selection_wrap,
                    check_ui_gallery_code_editor_a11y_composition_wrap,
                    check_ui_gallery_code_editor_a11y_composition_wrap_scroll,
                    check_ui_gallery_code_editor_a11y_composition_drag,
                    check_semantics_changed_repainted,
                    dump_semantics_changed_repainted_json,
                    check_wheel_scroll_test_id.as_deref(),
                    check_wheel_scroll_hit_changes_test_id.as_deref(),
                    check_prepaint_actions_min,
                    check_chart_sampling_window_shifts_min,
                    check_node_graph_cull_window_shifts_min,
                    check_node_graph_cull_window_shifts_max,
                    check_vlist_visible_range_refreshes_min,
                    check_vlist_visible_range_refreshes_max,
                    check_vlist_window_shifts_explainable,
                    check_vlist_window_shifts_have_prepaint_actions,
                    check_vlist_window_shifts_non_retained_max,
                    check_vlist_window_shifts_prefetch_max,
                    check_vlist_window_shifts_escape_max,
                    check_vlist_policy_key_stable,
                    check_windowed_rows_offset_changes_min,
                    check_windowed_rows_offset_changes_eps,
                    check_windowed_rows_visible_start_changes_repainted,
                    check_layout_fast_path_min,
                    check_drag_cache_root_paint_only_test_id.as_deref(),
                    check_hover_layout_max,
                    check_gc_sweep_liveness,
                    &check_notify_hotspot_file_max,
                    check_view_cache_reuse_stable_min,
                    check_view_cache_reuse_min,
                    check_overlay_synthesis_min,
                    check_viewport_input_min,
                    check_dock_drag_min,
                    check_viewport_capture_min,
                    check_retained_vlist_reconcile_no_notify_min,
                    check_retained_vlist_attach_detach_max,
                    check_retained_vlist_keep_alive_reuse_min,
                    check_retained_vlist_keep_alive_budget,
                    warmup_frames,
                ) {
                    overall_reason_code =
                        Some("tooling.post_run_checks.failed".to_string());
                    overall_error = Some(err);
                    break;
                }
            }
        } else {
            overall_reason_code = result
                .reason_code
                .clone()
                .or_else(|| Some("script.failed".to_string()));
            overall_error = Some(format!(
                "script failed: {} (run_id={}, step={:?}, reason={:?})",
                src.display(),
                result.run_id,
                result.step_index,
                result.reason
            ));
            break;
        }
    }

    let zip_out = pack_out
        .map(|p| resolve_path(&workspace_root, p))
        .unwrap_or_else(|| resolved_out_dir.join("repro.zip"));

    let mut packed_zip: Option<PathBuf> = None;
    let mut packed_bundle_json: Option<PathBuf> = None;
    if let Some(bundle_path) = selected_bundle_path.as_ref() {
        let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
        packed_bundle_json = Some(bundle_dir.join("bundle.json"));
    }

    let multi_pack = pack_items.len() > 1;
    let packed_bundles = if multi_pack {
        serde_json::Value::Array(
            pack_items
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    serde_json::json!({
                        "zip_prefix": repro_zip_prefix_for_script(item, idx),
                        "script_path": item.script_path.display().to_string(),
                        "bundle_json": item.bundle_json.display().to_string(),
                    })
                })
                .collect(),
        )
    } else {
        serde_json::Value::Null
    };

    let mut renderdoc_capture_payload: Option<serde_json::Value> = None;
    if with_renderdoc {
        let markers: Vec<String> = if renderdoc_markers.is_empty() {
            vec![
                "fret clip mask pass".to_string(),
                "fret downsample-nearest pass".to_string(),
                "fret upscale-nearest pass".to_string(),
            ]
        } else {
            renderdoc_markers.clone()
        };

        if let Some(dir) = prepared_launch.renderdoc_capture_dir.as_ref() {
            let captures = wait_for_files_with_extensions(dir, &["rdc"], 10_000, poll_ms);
            repro_process_footprint = repro_process_footprint.or(stop_launched_demo(
                &mut child,
                &resolved_exit_path,
                poll_ms,
            ));

            let mut capture_rows: Vec<serde_json::Value> = Vec::new();
            for (cap_idx, capture) in captures.iter().enumerate() {
                let stem = capture
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or("capture");
                let safe_stem = format!(
                    "{:02}-{}",
                    cap_idx.saturating_add(1),
                    zip_safe_component(stem)
                );
                let inspect_root = dir.join("inspect").join(&safe_stem);

                let summary_dir = inspect_root.join("summary");
                let summary_attempt = run_fret_renderdoc_dump(
                    &workspace_root,
                    capture,
                    &summary_dir,
                    "summary",
                    "",
                    Some(200_000),
                    true,
                    true,
                    Some(30),
                );

                let mut attempts: Vec<RenderdocDumpAttempt> = Vec::new();
                attempts.push(summary_attempt);

                for (idx, marker) in markers.iter().enumerate() {
                    let safe_marker = zip_safe_component(marker);
                    let out_dir = inspect_root
                        .join(format!("marker_{:02}_{safe_marker}", idx.saturating_add(1)));
                    let attempt = run_fret_renderdoc_dump(
                        &workspace_root,
                        capture,
                        &out_dir,
                        "dump",
                        marker,
                        Some(2_000),
                        true,
                        renderdoc_no_outputs_png,
                        None,
                    );
                    attempts.push(attempt);
                }

                let attempt_rows = attempts
                    .into_iter()
                    .map(|a| {
                        let out_dir = a
                            .out_dir
                            .strip_prefix(&resolved_out_dir)
                            .unwrap_or(&a.out_dir)
                            .display()
                            .to_string();
                        let stdout_file = a.stdout_file.as_ref().map(|p| {
                            p.strip_prefix(&resolved_out_dir)
                                .unwrap_or(p)
                                .display()
                                .to_string()
                        });
                        let stderr_file = a.stderr_file.as_ref().map(|p| {
                            p.strip_prefix(&resolved_out_dir)
                                .unwrap_or(p)
                                .display()
                                .to_string()
                        });
                        let response_json = a.response_json.as_ref().map(|p| {
                            p.strip_prefix(&resolved_out_dir)
                                .unwrap_or(p)
                                .display()
                                .to_string()
                        });

                        serde_json::json!({
                            "marker": a.marker,
                            "out_dir": out_dir,
                            "exit_code": a.exit_code,
                            "response_json": response_json,
                            "stdout_file": stdout_file,
                            "stderr_file": stderr_file,
                            "error": a.error,
                        })
                    })
                    .collect::<Vec<_>>();

                let capture_rel = capture
                    .strip_prefix(&resolved_out_dir)
                    .unwrap_or(capture)
                    .display()
                    .to_string();
                let inspect_rel = inspect_root
                    .strip_prefix(&resolved_out_dir)
                    .unwrap_or(&inspect_root)
                    .display()
                    .to_string();

                capture_rows.push(serde_json::json!({
                    "capture": capture_rel,
                    "inspect_dir": inspect_rel,
                    "dumps": attempt_rows,
                }));
            }

            let payload = serde_json::json!({
                "schema_version": 2,
                "generated_unix_ms": now_unix_ms(),
                "capture_dir": "renderdoc",
                "autocapture_after_frames": prepared_launch.renderdoc_autocapture_after_frames,
                "captures": capture_rows,
            });
            let _ = write_json_value(&resolved_out_dir.join("renderdoc.captures.json"), &payload);
            renderdoc_capture_payload = Some(payload);
        } else {
            repro_process_footprint = repro_process_footprint.or(stop_launched_demo(
                &mut child,
                &resolved_exit_path,
                poll_ms,
            ));
        }
    } else {
        repro_process_footprint = repro_process_footprint.or(stop_launched_demo(
            &mut child,
            &resolved_exit_path,
            poll_ms,
        ));
    }

    let repro_process_footprint_file = resolved_out_dir.join("resource.footprint.json");
    if let Some(payload) = repro_process_footprint.as_ref() {
        let _ = write_json_value(&repro_process_footprint_file, payload);
    }
    if resource_footprint_thresholds.any() {
        resource_footprint_gate = check_resource_footprint_thresholds(
            &resolved_out_dir,
            &repro_process_footprint_file,
            &resource_footprint_thresholds,
        )
        .ok();
    }
    if let Some(max_total_ms) = check_redraw_hitches_max_total_ms_threshold {
        redraw_hitches_gate =
            check_redraw_hitches_max_total_ms(&resolved_out_dir, max_total_ms).ok();
    }

    let captures_json = serde_json::json!({
        "tracy": if with_tracy {
            serde_json::json!({
                "requested": true,
                "env_enabled": prepared_launch.tracy_env_enabled,
                "feature_injected": prepared_launch.tracy_feature_injected,
                "note": "Capture is not recorded automatically yet; use the Tracy UI to save a capture."
            })
        } else {
            serde_json::Value::Null
        },
        "renderdoc": if with_renderdoc {
            renderdoc_capture_payload.clone().unwrap_or_else(|| serde_json::json!({
                "schema_version": 2,
                "generated_unix_ms": now_unix_ms(),
                "capture_dir": "renderdoc",
                "autocapture_after_frames": prepared_launch.renderdoc_autocapture_after_frames,
                "captures": [],
            }))
        } else {
            serde_json::Value::Null
        }
    });

    let summary_json = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "out_dir": resolved_out_dir.display().to_string(),
        "suite": suite_name,
        "capabilities": serde_json::json!({
            "required": required_caps,
            "available": available_caps,
            "check_file": if capabilities_check_path.is_file() { Some("check.capabilities.json") } else { None },
        }),
        "scripts": run_rows,
        "selected_bundle_json": selected_bundle_path.as_ref().map(|p| p.display().to_string()),
        "packed_bundle_json": packed_bundle_json.as_ref().map(|p| p.display().to_string()),
        "packed_bundles": packed_bundles,
        "repro_zip": Some(zip_out.display().to_string()),
        "resources": serde_json::json!({
            "process_footprint_file": if repro_process_footprint_file.is_file() {
                Some("resource.footprint.json")
            } else {
                None
            },
            "process_footprint": repro_process_footprint,
        }),
        "captures": captures_json,
        "last_result": last_script_result.as_ref().map(|r| serde_json::json!({
            "run_id": r.run_id,
            "stage": r.stage,
            "step_index": r.step_index,
            "reason": r.reason,
            "last_bundle_dir": r.last_bundle_dir,
        })),
        "error_reason_code": overall_reason_code,
        "error": overall_error,
    });

    if let Some(parent) = summary_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = write_json_value(&summary_path, &summary_json);
    let _ = write_evidence_index(&resolved_out_dir, &summary_path, Some(&summary_json));

    if overall_error.is_none() {
        let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);
        if multi_pack {
            let bundles: Vec<ReproZipBundle> = pack_items
                .iter()
                .enumerate()
                .map(|(idx, item)| ReproZipBundle {
                    prefix: repro_zip_prefix_for_script(item, idx),
                    bundle_json: item.bundle_json.clone(),
                    source_script: item.script_path.clone(),
                })
                .collect();

            if let Err(err) = pack_repro_zip_multi(
                &zip_out,
                pack_defaults.0,
                pack_defaults.1,
                pack_defaults.2,
                with_renderdoc,
                with_tracy,
                &resolved_out_dir,
                &summary_path,
                &bundles,
                stats_top,
                sort,
                warmup_frames,
            ) {
                overall_error = Some(format!("failed to pack repro zip: {err}"));
            } else {
                packed_zip = Some(zip_out.clone());
            }
        } else if let Some(bundle_path) = selected_bundle_path.as_ref() {
            let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
            let artifacts_root = if bundle_dir.starts_with(&resolved_out_dir) {
                resolved_out_dir.clone()
            } else {
                bundle_dir
                    .parent()
                    .unwrap_or(&resolved_out_dir)
                    .to_path_buf()
            };

            if let Err(err) = pack_bundle_dir_to_zip(
                &bundle_dir,
                &zip_out,
                pack_defaults.0,
                pack_defaults.1,
                pack_defaults.2,
                with_renderdoc,
                with_tracy,
                &artifacts_root,
                stats_top,
                sort,
                warmup_frames,
            ) {
                overall_error = Some(format!("failed to pack repro zip: {err}"));
                overall_reason_code = Some("tooling.pack.failed".to_string());
            } else {
                packed_zip = Some(zip_out.clone());
            }
        } else {
            overall_error = Some(
                "no bundle.json found (add `capture_bundle` or enable script auto-dumps)"
                    .to_string(),
            );
            overall_reason_code = Some("tooling.bundle_missing".to_string());
        }

        if overall_error.is_some() {
            // Keep the summary coherent even when packing fails.
            let _ = write_json_value(
                &summary_path,
                &summary_json
                    .as_object()
                    .cloned()
                    .map(|mut obj| {
                        obj.insert(
                            "error".to_string(),
                            serde_json::Value::String(overall_error.clone().unwrap_or_default()),
                        );
                        if let Some(code) = overall_reason_code.as_ref() {
                            obj.insert(
                                "error_reason_code".to_string(),
                                serde_json::Value::String(code.clone()),
                            );
                        }
                        serde_json::Value::Object(obj)
                    })
                    .unwrap_or(summary_json.clone()),
            );
        }
    }

    if let Some(r) = resource_footprint_gate.as_ref()
        && r.failures > 0
        && overall_error.is_none()
    {
        overall_error = Some(format!(
            "resource footprint threshold gate failed (failures={}, evidence={})",
            r.failures,
            r.evidence_path.display()
        ));
        overall_reason_code = Some("tooling.resource_footprint.failed".to_string());
    }
    if let Some(r) = redraw_hitches_gate.as_ref()
        && r.failures > 0
        && overall_error.is_none()
    {
        overall_error = Some(format!(
            "redraw hitch threshold gate failed (failures={}, evidence={})",
            r.failures,
            r.evidence_path.display()
        ));
        overall_reason_code = Some("tooling.redraw_hitches.failed".to_string());
    }

    let final_summary_json = summary_json
        .as_object()
        .cloned()
        .map(|mut obj| {
            if let Some(err) = overall_error.as_ref() {
                obj.insert("error".to_string(), serde_json::Value::String(err.clone()));
            }
            if let Some(code) = overall_reason_code.as_ref() {
                obj.insert(
                    "error_reason_code".to_string(),
                    serde_json::Value::String(code.clone()),
                );
            }
            serde_json::Value::Object(obj)
        })
        .unwrap_or_else(|| summary_json.clone());
    let _ = write_json_value(&summary_path, &final_summary_json);
    if let Err(err) =
        write_evidence_index(&resolved_out_dir, &summary_path, Some(&final_summary_json))
    {
        eprintln!("WARN failed to write evidence index: {err}");
    }

    if let Some(path) = packed_bundle_json.as_ref() {
        println!("BUNDLE {}", path.display());
    }
    if let Some(path) = packed_zip.as_ref() {
        println!("PACK {}", path.display());
    }
    println!("SUMMARY {}", summary_path.display());

    if let Some(err) = overall_error {
        eprintln!("REPRO-FAIL {err}");
        std::process::exit(1);
    }

    println!("REPRO-OK");
    std::process::exit(0);
}
