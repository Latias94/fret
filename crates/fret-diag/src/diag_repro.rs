use super::*;

mod launch;
mod pack;
mod renderdoc;
mod scripts;
mod summary;
mod util;

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

    let checks_for_post_run = checks.clone();

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
        check_stale_paint_eps: _,
        check_stale_paint_test_id,
        check_stale_scene_eps: _,
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
        check_windowed_rows_offset_changes_eps: _,
        check_windowed_rows_offset_changes_min,
        check_windowed_rows_visible_start_changes_repainted: _,
        dump_semantics_changed_repainted_json: _,
    } = checks;

    if rest.is_empty() {
        return Err(
            "missing script path or suite name (try: fretboard diag repro ui-gallery | fretboard diag repro ./script.json)"
                .to_string(),
        );
    }

    let pack_defaults = util::pack_defaults_with_fallback(
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
    );

    let (scripts, suite_name) = scripts::resolve_repro_scripts(&rest, &workspace_root);

    let summary_path = resolved_out_dir.join("repro.summary.json");

    let required_caps = scripts::compute_required_caps(&scripts);

    let mut overall_reason_code: Option<String> = None;

    let prepared_launch = launch::prepare_repro_launch(
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
        overall_reason_code = util::read_tooling_reason_code(&resolved_script_result_path)
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
                overall_reason_code = util::read_tooling_reason_code(&resolved_script_result_path)
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
                        "script passed but no bundle artifact was found (required for post-run checks)"
                            .to_string(),
                    );
                    break;
                };

                if let Err(err) = apply_post_run_checks(
                    bundle_path,
                    &resolved_out_dir,
                    &checks_for_post_run,
                    warmup_frames,
                ) {
                    overall_reason_code = Some("tooling.post_run_checks.failed".to_string());
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

    let (new_footprint, renderdoc_capture_payload) =
        renderdoc::stop_demo_and_collect_renderdoc_captures(
            &workspace_root,
            &resolved_out_dir,
            &resolved_exit_path,
            poll_ms,
            &mut child,
            with_renderdoc,
            prepared_launch.renderdoc_capture_dir.as_ref(),
            prepared_launch.renderdoc_autocapture_after_frames,
            &renderdoc_markers,
            renderdoc_no_outputs_png,
        );
    repro_process_footprint = repro_process_footprint.or(new_footprint);

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

    let captures_json = summary::build_captures_json(
        with_tracy,
        with_renderdoc,
        &prepared_launch,
        renderdoc_capture_payload.as_ref(),
    );

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

    summary::write_summary_and_evidence_best_effort(
        &resolved_out_dir,
        &summary_path,
        &summary_json,
    );

    if overall_error.is_none() {
        let sort = sort_override.unwrap_or(BundleStatsSort::Invalidation);
        let outcome = pack::pack_repro_zip(
            multi_pack,
            &pack_items,
            selected_bundle_path.as_ref(),
            &resolved_out_dir,
            &summary_path,
            &zip_out,
            pack_defaults,
            with_renderdoc,
            with_tracy,
            stats_top,
            sort,
            warmup_frames,
        )?;
        packed_zip = outcome.packed_zip;
        overall_error = outcome.overall_error;
        if outcome.overall_reason_code.is_some() {
            overall_reason_code = outcome.overall_reason_code;
        }

        if overall_error.is_some() {
            // Keep the summary coherent even when packing fails.
            let packing_failed_summary_json = summary::summary_json_with_error(
                &summary_json,
                overall_error.as_deref(),
                overall_reason_code.as_deref(),
            );
            let _ = write_json_value(&summary_path, &packing_failed_summary_json);
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

    let final_summary_json = summary::summary_json_with_error(
        &summary_json,
        overall_error.as_deref(),
        overall_reason_code.as_deref(),
    );
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
