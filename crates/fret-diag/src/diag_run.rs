use super::*;

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
hint: pass an explicit path, or ensure `tools/diag-scripts/index.json` exists (run `python tools/check_diag_scripts_registry.py --write`)",
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
hint: regenerate tools/diag-scripts/index.json (python tools/check_diag_scripts_registry.py --write)",
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

#[derive(Debug, Clone)]
pub(crate) struct RunChecks {
    pub check_chart_sampling_window_shifts_min: Option<u64>,
    pub check_dock_drag_min: Option<u64>,
    pub check_drag_cache_root_paint_only_test_id: Option<String>,
    pub check_gc_sweep_liveness: bool,
    pub check_hover_layout_max: Option<u32>,
    pub check_idle_no_paint_min: Option<u64>,
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
    pub check_wheel_scroll_hit_changes_test_id: Option<String>,
    pub check_wheel_scroll_test_id: Option<String>,
    pub check_windowed_rows_offset_changes_eps: f32,
    pub check_windowed_rows_offset_changes_min: Option<u64>,
    pub check_windowed_rows_visible_start_changes_repainted: bool,
    pub dump_semantics_changed_repainted_json: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RunCmdContext {
    pub pack_after_run: bool,
    pub ensure_ai_packet: bool,
    pub rest: Vec<String>,
    pub workspace_root: PathBuf,
    pub resolved_out_dir: PathBuf,
    pub resolved_trigger_path: PathBuf,
    pub resolved_ready_path: PathBuf,
    pub resolved_exit_path: PathBuf,
    pub resolved_script_path: PathBuf,
    pub resolved_script_result_path: PathBuf,
    pub fs_transport_cfg: crate::transport::FsDiagTransportConfig,
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
        resolved_out_dir,
        resolved_trigger_path,
        resolved_ready_path,
        resolved_exit_path,
        resolved_script_path,
        resolved_script_result_path,
        fs_transport_cfg,
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

    let checks_for_post_run = checks.clone();

    let RunChecks {
        check_chart_sampling_window_shifts_min,
        check_dock_drag_min,
        check_drag_cache_root_paint_only_test_id,
        check_gc_sweep_liveness: _,
        check_hover_layout_max,
        check_idle_no_paint_min: _,
        check_layout_fast_path_min,
        check_node_graph_cull_window_shifts_max,
        check_node_graph_cull_window_shifts_min,
        check_notify_hotspot_file_max: _,
        check_triage_hint_absent_codes: _,
        check_overlay_synthesis_min,
        check_pixels_changed_test_id: _,
        check_pixels_unchanged_test_id: _,
        check_prepaint_actions_min,
        check_retained_vlist_attach_detach_max,
        check_retained_vlist_keep_alive_budget,
        check_retained_vlist_keep_alive_reuse_min,
        check_retained_vlist_reconcile_no_notify_min,
        check_semantics_changed_repainted,
        check_stale_paint_eps: _,
        check_stale_paint_test_id: _,
        check_stale_scene_eps: _,
        check_stale_scene_test_id: _,
        check_ui_gallery_code_editor_a11y_composition: _,
        check_ui_gallery_code_editor_a11y_composition_drag: _,
        check_ui_gallery_code_editor_a11y_composition_wrap: _,
        check_ui_gallery_code_editor_a11y_composition_wrap_scroll: _,
        check_ui_gallery_code_editor_a11y_selection: _,
        check_ui_gallery_code_editor_a11y_selection_wrap: _,
        check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection: _,
        check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll: _,
        check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed:
            _,
        check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed:
            _,
        check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit: _,
        check_ui_gallery_code_editor_torture_folds_placeholder_present: _,
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped:
            _,
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations:
            _,
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed:
            _,
        check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap: _,
        check_ui_gallery_code_editor_torture_geom_fallbacks_low: _,
        check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit: _,
        check_ui_gallery_code_editor_torture_inlays_present: _,
        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped: _,
        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations: _,
        check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed:
            _,
        check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap: _,
        check_ui_gallery_code_editor_torture_marker_present: _,
        check_ui_gallery_code_editor_torture_read_only_blocks_edits,
        check_ui_gallery_code_editor_torture_undo_redo: _,
        check_ui_gallery_code_editor_word_boundary: _,
        check_ui_gallery_markdown_editor_source_a11y_composition: _,
        check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap: _,
        check_ui_gallery_markdown_editor_source_disabled_blocks_edits,
        check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds: _,
        check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit: _,
        check_ui_gallery_markdown_editor_source_folds_placeholder_present: _,
        check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap: _,
        check_ui_gallery_markdown_editor_source_folds_toggle_stable: _,
        check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit: _,
        check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable: _,
        check_ui_gallery_markdown_editor_source_inlays_present: _,
        check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap: _,
        check_ui_gallery_markdown_editor_source_inlays_toggle_stable: _,
        check_ui_gallery_markdown_editor_source_line_boundary_triple_click: _,
        check_ui_gallery_markdown_editor_source_read_only_blocks_edits,
        check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable: _,
        check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable: _,
        check_ui_gallery_markdown_editor_source_word_boundary: _,
        check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change,
        check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change,
        check_ui_gallery_text_mixed_script_bundled_fallback_conformance,
        check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps,
        check_ui_gallery_web_ime_bridge_enabled: _,
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
        check_windowed_rows_visible_start_changes_repainted,
        dump_semantics_changed_repainted_json: _,
    } = checks;

    fn push_env_if_missing(env: &mut Vec<(String, String)>, key: &str, value: &str) {
        if env.iter().any(|(k, _v)| k == key) {
            return;
        }
        env.push((key.to_string(), value.to_string()));
    }

    let (bundle_doctor_mode, rest) = parse_bundle_doctor_mode_from_rest(&rest)?;
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing script path or script id (try: fretboard diag run <script.json|script_id>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
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

    let wants_pack_zip = pack_after_run
        || pack_out.is_some()
        || pack_include_root_artifacts
        || pack_include_triage
        || pack_include_screenshots;
    let wants_post_run_bundle = wants_pack_zip || ensure_ai_packet;
    let check_registry = crate::registry::checks::CheckRegistry::builtin();
    let wants_registered_post_run_checks =
        check_registry.wants_post_run_checks(&checks_for_post_run);
    let wants_ad_hoc_post_run_checks = check_ui_gallery_code_editor_torture_read_only_blocks_edits
        || check_ui_gallery_markdown_editor_source_read_only_blocks_edits
        || check_ui_gallery_markdown_editor_source_disabled_blocks_edits
        || check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps
        || check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change
        || check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change
        || check_ui_gallery_text_mixed_script_bundled_fallback_conformance
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
        || check_windowed_rows_visible_start_changes_repainted
        || check_layout_fast_path_min.is_some()
        || check_drag_cache_root_paint_only_test_id.is_some()
        || check_hover_layout_max.is_some()
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
    let wants_post_run_checks = wants_registered_post_run_checks || wants_ad_hoc_post_run_checks;

    let wants_bundle_artifact = wants_post_run_bundle
        || wants_ad_hoc_post_run_checks
        || check_registry.wants_bundle_artifact(&checks_for_post_run);

    let mut pack_defaults = (
        pack_include_root_artifacts,
        pack_include_triage,
        pack_include_screenshots,
    );
    if pack_after_run && !pack_defaults.0 && !pack_defaults.1 && !pack_defaults.2 {
        pack_defaults = (true, true, true);
    }

    let src = resolve_run_script_source(&workspace_root, &src)?;
    let use_devtools_ws =
        devtools_ws_url.is_some() || devtools_token.is_some() || devtools_session_id.is_some();
    if use_devtools_ws {
        if launch.is_some() || reuse_launch {
            return Err(
                "--launch/--reuse-launch is not supported with --devtools-ws-url".to_string(),
            );
        }

        let ws_url = devtools_ws_url.clone().ok_or_else(|| {
            "missing --devtools-ws-url (required when using DevTools WS transport)".to_string()
        })?;
        let token = devtools_token.clone().ok_or_else(|| {
            "missing --devtools-token (required when using DevTools WS transport)".to_string()
        })?;

        std::fs::create_dir_all(&resolved_out_dir).map_err(|e| e.to_string())?;
        let (script_json, upgraded) = crate::script_execution::load_script_json_for_execution(
            &src,
            crate::script_execution::ScriptLoadPolicy {
                tool_launched: false,
                write_failure: write_tooling_failure_script_result_if_missing,
                failure_note: None,
                include_stage_in_note: true,
            },
            &resolved_script_result_path,
        )?;
        if upgraded {
            eprintln!(
                "warning: script schema_version=1 detected; tooling upgraded to schema_version=2 for execution (source={})",
                src.display()
            );
        }

        let _ = write_json_value(&resolved_script_path, &script_json);

        let connected = connect_devtools_ws_tooling(
            ws_url.as_str(),
            token.as_str(),
            devtools_session_id.as_deref(),
            timeout_ms,
            poll_ms,
        )
        .inspect_err(|err| {
            write_tooling_failure_script_result_if_missing(
                &resolved_script_result_path,
                "tooling.connect.failed",
                err,
                "tooling_error",
                Some("connect_devtools_ws_tooling".to_string()),
            );
        })?;

        let (result, bundle_path) = run_script_over_transport(
            &resolved_out_dir,
            &connected,
            script_json,
            wants_bundle_artifact,
            trace_chrome,
            Some("diag-run"),
            None,
            timeout_ms,
            poll_ms,
            &resolved_script_result_path,
            &resolved_out_dir.join("check.capabilities.json"),
        )
        .inspect_err(|err| {
            write_tooling_failure_script_result_if_missing(
                &resolved_script_result_path,
                "tooling.run.failed",
                err,
                "tooling_error",
                Some("run_script_over_transport".to_string()),
            );
        })?;

        if exit_after_run {
            connected
                .devtools
                .app_exit_request(None, Some("diag.run"), None);
        }

        let stage = match result.stage {
            fret_diag_protocol::UiScriptStageV1::Passed => "passed",
            fret_diag_protocol::UiScriptStageV1::Failed => "failed",
            fret_diag_protocol::UiScriptStageV1::Queued => "queued",
            fret_diag_protocol::UiScriptStageV1::Running => "running",
        };

        let mut summary = crate::stats::ScriptResultSummary {
            run_id: result.run_id,
            stage: Some(stage.to_string()),
            step_index: result.step_index.map(|n| n as u64),
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
            && let Some(bundle_path) = bundle_path.as_ref()
        {
            summary.last_bundle_dir = bundle_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());
        }

        if bundle_doctor_mode != BundleDoctorMode::Off
            && let Some(bundle_path) = bundle_path.as_deref()
        {
            run_bundle_doctor_for_bundle_path(bundle_path, bundle_doctor_mode, warmup_frames)?;
        }

        if wants_post_run_checks
            && matches!(result.stage, fret_diag_protocol::UiScriptStageV1::Passed)
        {
            let Some(bundle_path) = bundle_path.as_ref() else {
                return Err(
                    "script passed but no bundle artifact was captured (required for post-run checks)"
                        .to_string(),
                );
            };
            apply_post_run_checks(
                bundle_path,
                &resolved_out_dir,
                &checks_for_post_run,
                warmup_frames,
            )?;
        }

        if ensure_ai_packet {
            if let Some(bundle_path) = bundle_path.as_ref() {
                let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
                let packet_dir = bundle_dir.join("ai.packet");
                match crate::commands::ai_packet::ensure_ai_packet_dir_best_effort(
                    Some(bundle_path),
                    &bundle_dir,
                    &packet_dir,
                    pack_defaults.1,
                    stats_top,
                    sort_override,
                    warmup_frames,
                    None,
                ) {
                    Ok(()) => println!("AI-PACKET {}", packet_dir.display()),
                    Err(err) => eprintln!("AI-PACKET-ERROR {err}"),
                }
            } else {
                eprintln!(
                    "AI-PACKET-ERROR no bundle artifact captured over DevTools WS (ensure bundles are embedded or the runtime bundle dir is accessible)"
                );
            }
        }

        if wants_pack_zip {
            if let Some(bundle_path) = bundle_path.as_ref() {
                let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
                let out = pack_out
                    .clone()
                    .map(|p| resolve_path(&workspace_root, p))
                    .unwrap_or_else(|| default_pack_out_path(&resolved_out_dir, &bundle_dir));

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
                    &out,
                    pack_defaults.0,
                    pack_defaults.1,
                    pack_defaults.2,
                    pack_schema2_only,
                    false,
                    false,
                    &artifacts_root,
                    stats_top,
                    sort_override.unwrap_or(BundleStatsSort::Invalidation),
                    warmup_frames,
                ) {
                    eprintln!("PACK-ERROR {err}");
                } else {
                    println!("PACK {}", out.display());
                }
            } else {
                eprintln!(
                    "PACK-ERROR no bundle artifact captured over DevTools WS (ensure bundles are embedded or the runtime bundle dir is accessible)"
                );
            }
        }

        report_result_and_exit(&summary);
    }
    let script_wants_screenshots = script_requests_screenshots(&src);
    let mut run_launch_env = launch_env.clone();
    // Tool-launched runs default to *not* redacting text to keep authoring/debugging ergonomic.
    //
    // Privacy-sensitive workflows (pack/share/CI) should explicitly opt back into redaction via:
    // `--env FRET_DIAG_REDACT_TEXT=1` (or by inheriting it from the parent environment).
    push_env_if_missing(&mut run_launch_env, "FRET_DIAG_REDACT_TEXT", "0");
    for (key, value) in script_env_defaults(&src) {
        push_env_if_missing(&mut run_launch_env, &key, &value);
    }
    let _ = ensure_env_var(&mut run_launch_env, "FRET_DIAG_RENDERER_PERF", "1");
    if check_view_cache_reuse_min.is_some_and(|v| v > 0)
        || check_view_cache_reuse_stable_min.is_some_and(|v| v > 0)
    {
        // View-cache reuse gates depend on cache-root debug records, which are only produced when
        // the app enables UiTree debug collection. UI gallery disables debug in perf mode unless
        // `FRET_UI_DEBUG_STATS` is set.
        let _ = ensure_env_var(&mut run_launch_env, "FRET_UI_DEBUG_STATS", "1");
    }
    let mut child = maybe_launch_demo(
        &launch,
        &run_launch_env,
        &workspace_root,
        &resolved_ready_path,
        &resolved_exit_path,
        &fs_transport_cfg,
        pack_defaults.2
            || check_registry.wants_screenshots(&checks_for_post_run)
            || script_wants_screenshots,
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
    let _stop_guard = if keep_open {
        None
    } else {
        Some(StopLaunchedDemoOnDrop {
            child: &mut child,
            exit_path: &resolved_exit_path,
            poll_ms,
        })
    };

    let connected = connect_filesystem_tooling(
        &fs_transport_cfg,
        &resolved_ready_path,
        launch.is_some(),
        timeout_ms,
        poll_ms,
    )
    .inspect_err(|err| {
        write_tooling_failure_script_result_if_missing(
            &resolved_script_result_path,
            "tooling.connect.failed",
            err,
            "tooling_error",
            Some("connect_filesystem_tooling".to_string()),
        );
    })?;
    let tool_launched = launch.is_some() || reuse_launch;
    let (script_json, upgraded) = crate::script_execution::load_script_json_for_execution(
        &src,
        crate::script_execution::ScriptLoadPolicy {
            tool_launched,
            write_failure: write_tooling_failure_script_result_if_missing,
            failure_note: None,
            include_stage_in_note: true,
        },
        &resolved_script_result_path,
    )?;
    if upgraded {
        eprintln!(
            "warning: script schema_version=1 detected; tooling upgraded to schema_version=2 for execution (source={})",
            src.display()
        );
    }
    let (script_result, _bundle_path) = run_script_over_transport(
        &resolved_out_dir,
        &connected,
        script_json,
        wants_bundle_artifact,
        trace_chrome,
        Some("diag-run"),
        None,
        timeout_ms,
        poll_ms,
        &resolved_script_result_path,
        &resolved_out_dir.join("check.capabilities.json"),
    )
    .inspect_err(|err| {
        write_tooling_failure_script_result_if_missing(
            &resolved_script_result_path,
            "tooling.run.failed",
            err,
            "tooling_error",
            Some("run_script_over_transport".to_string()),
        );
    })?;

    let stage = match script_result.stage {
        fret_diag_protocol::UiScriptStageV1::Passed => "passed",
        fret_diag_protocol::UiScriptStageV1::Failed => "failed",
        fret_diag_protocol::UiScriptStageV1::Queued => "queued",
        fret_diag_protocol::UiScriptStageV1::Running => "running",
    };

    let mut result = crate::stats::ScriptResultSummary {
        run_id: script_result.run_id,
        stage: Some(stage.to_string()),
        step_index: script_result.step_index.map(|n| n as u64),
        reason_code: script_result.reason_code.clone(),
        reason: script_result.reason.clone(),
        last_bundle_dir: script_result.last_bundle_dir.clone(),
    };

    if result.stage.as_deref() == Some("failed")
        && let Some(dir) =
            wait_for_failure_dump_bundle(&resolved_out_dir, &result, timeout_ms, poll_ms)
        && let Some(name) = dir.file_name().and_then(|s| s.to_str())
    {
        result.last_bundle_dir = Some(name.to_string());
    }
    if exit_after_run {
        let _ = touch(&resolved_exit_path);
    }

    let mut bundle_doctor_ran: bool = false;
    if result.stage.as_deref() == Some("passed") && wants_post_run_checks {
        let bundle_path = wait_for_bundle_artifact_from_script_result(
            &resolved_out_dir,
            &result,
            timeout_ms,
            poll_ms,
        )
        .ok_or_else(|| {
            "script passed but no bundle artifact was found (required for post-run checks)"
                .to_string()
        })?;

        if bundle_doctor_mode != BundleDoctorMode::Off {
            run_bundle_doctor_for_bundle_path(&bundle_path, bundle_doctor_mode, warmup_frames)?;
            bundle_doctor_ran = true;
        }

        apply_post_run_checks(
            &bundle_path,
            &resolved_out_dir,
            &checks_for_post_run,
            warmup_frames,
        )?;
    }

    if wants_post_run_bundle {
        let mut bundle_path = wait_for_bundle_artifact_from_script_result(
            &resolved_out_dir,
            &result,
            timeout_ms,
            poll_ms,
        );
        if bundle_path.is_none() {
            let _ = touch(&resolved_trigger_path);
            bundle_path = wait_for_bundle_artifact_from_script_result(
                &resolved_out_dir,
                &result,
                timeout_ms,
                poll_ms,
            );
        }

        if let Some(bundle_path) = bundle_path {
            if !bundle_doctor_ran && bundle_doctor_mode != BundleDoctorMode::Off {
                run_bundle_doctor_for_bundle_path(&bundle_path, bundle_doctor_mode, warmup_frames)?;
            }
            let bundle_dir = resolve_bundle_root_dir(&bundle_path)?;

            if ensure_ai_packet {
                let packet_dir = bundle_dir.join("ai.packet");
                match crate::commands::ai_packet::ensure_ai_packet_dir_best_effort(
                    Some(&bundle_path),
                    &bundle_dir,
                    &packet_dir,
                    pack_defaults.1,
                    stats_top,
                    sort_override,
                    warmup_frames,
                    None,
                ) {
                    Ok(()) => println!("AI-PACKET {}", packet_dir.display()),
                    Err(err) => eprintln!("AI-PACKET-ERROR {err}"),
                }
            }

            if wants_pack_zip {
                let out = pack_out
                    .clone()
                    .map(|p| resolve_path(&workspace_root, p))
                    .unwrap_or_else(|| default_pack_out_path(&resolved_out_dir, &bundle_dir));

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
                    &out,
                    pack_defaults.0,
                    pack_defaults.1,
                    pack_defaults.2,
                    pack_schema2_only,
                    false,
                    false,
                    &artifacts_root,
                    stats_top,
                    sort_override.unwrap_or(BundleStatsSort::Invalidation),
                    warmup_frames,
                ) {
                    eprintln!("PACK-ERROR {err}");
                } else {
                    println!("PACK {}", out.display());
                }
            }
        } else {
            eprintln!(
                "POST-RUN-ERROR no bundle artifact found (add `capture_bundle` or enable script auto-dumps)"
            );
        }
    }

    drop(_stop_guard);
    report_result_and_exit(&result);
}
