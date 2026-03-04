use super::*;

pub(crate) type SuiteChecks = diag_run::RunChecks;

use crate::registry::suites::SuiteResolver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuiltinSuite {
    UiGallery,
    UiGalleryCodeEditor,
    UiGalleryLayout,
    DockingArbitration,
    DockingMotionPilot,
}

fn push_env_if_missing(env: &mut Vec<(String, String)>, key: &str, value: &str) {
    if env.iter().any(|(k, _v)| k == key) {
        return;
    }
    env.push((key.to_string(), value.to_string()));
}

fn resolve_builtin_suite_scripts(
    workspace_root: &Path,
    suite_name: &str,
    launch_env: &mut Vec<(String, String)>,
) -> Result<Option<(Vec<PathBuf>, Option<BuiltinSuite>)>, String> {
    let scripts_from_suite_dir = |suite: &str| -> Result<Vec<PathBuf>, String> {
        SuiteResolver::scripts_from_suite_dir(workspace_root, suite)
    };

    let resolved = match suite_name {
        "ui-gallery" => {
            // The UI Gallery suite includes scripts that run the `--check-pixels-changed`
            // post-run gate. Enable screenshots so those checks can resolve semantics
            // bounds against captured PNGs.
            push_env_if_missing(launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
            let inputs = diag_suite_scripts::ui_gallery_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-overlay-steady" => {
            // The overlay-steady suite intentionally exercises role-and-name predicates and
            // inspector help search queries, which are incompatible with redaction.
            push_env_if_missing(launch_env, "FRET_DIAG_REDACT_TEXT", "0");
            let inputs = diag_suite_scripts::ui_gallery_overlay_steady_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-motion-pilot" => {
            // The motion pilot suite relies on stable semantics surfaces; keep diagnostics
            // redaction disabled so any role-and-name selectors remain usable in scripts.
            push_env_if_missing(launch_env, "FRET_DIAG_REDACT_TEXT", "0");
            // Some motion feel gates use the `--check-pixels-changed` post-run check, which
            // requires GPU screenshots.
            push_env_if_missing(launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
            let scripts = scripts_from_suite_dir("ui-gallery-motion-pilot")?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-code-editor" => {
            // The code-editor-focused UI Gallery suite also includes the pixels-changed
            // gate (soft-wrap editing baseline), so screenshots must be enabled.
            push_env_if_missing(launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
            let inputs = diag_suite_scripts::ui_gallery_code_editor_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::UiGalleryCodeEditor))
        }
        "ui-gallery-date-picker" => {
            // Keep date picker scripts deterministic (date + page seed) so keyboard navigation and
            // disabled-day skipping are repeatable.
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_START_PAGE", "date_picker");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_DIAG_CALENDAR_ROVING", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_FIXED_TODAY", "2024-02-01");
            let inputs = diag_suite_scripts::ui_gallery_date_picker_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-text-ime" => {
            let inputs = diag_suite_scripts::ui_gallery_text_ime_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-text-wrap" => {
            // Text wrap/baseline gates rely on screenshots and should run with deterministic
            // bundled fonts on desktop.
            push_env_if_missing(launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_BOOTSTRAP_FONTS", "1");
            let inputs = diag_suite_scripts::ui_gallery_text_wrap_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-select" => {
            // Keep this suite redaction-friendly: scripts should prefer `test_id` selectors so we can
            // safely share bundles when redaction is enabled.
            let inputs = diag_suite_scripts::ui_gallery_select_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-combobox" => {
            let inputs = diag_suite_scripts::ui_gallery_combobox_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-shadcn-conformance" => {
            // Conformance scripts rely on stable role-and-name semantics selectors and use
            // screenshot evidence for overlap regressions.
            push_env_if_missing(launch_env, "FRET_DIAG_REDACT_TEXT", "0");
            push_env_if_missing(launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
            // Ensure bundled fonts are loaded on desktop so font metrics are deterministic.
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_BOOTSTRAP_FONTS", "1");
            let mut scripts = expand_script_inputs(
                workspace_root,
                &diag_suite_scripts::ui_gallery_shadcn_conformance_suite_scripts(),
            )?;
            scripts.extend(expand_script_inputs(
                workspace_root,
                &diag_suite_scripts::ui_gallery_select_suite_scripts(),
            )?);
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-layout" => {
            let inputs = diag_suite_scripts::ui_gallery_layout_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::UiGalleryLayout))
        }
        "ui-gallery-lite-smoke" => {
            // Keep this suite fast and deterministic: always start on the Intro page.
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_START_PAGE", "intro");
            let scripts = scripts_from_suite_dir("ui-gallery-lite-smoke")?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-virt-retained"
        | "ui-gallery-virt-retained-measured"
        | "ui-gallery-tree-retained"
        | "ui-gallery-tree-retained-measured"
        | "ui-gallery-data-table-retained"
        | "ui-gallery-data-table-retained-measured"
        | "ui-gallery-table-retained"
        | "ui-gallery-table-retained-measured"
        | "ui-gallery-retained-measured"
        | "ui-gallery-ui-kit-list-retained"
        | "ui-gallery-inspector-torture"
        | "ui-gallery-inspector-torture-keep-alive"
        | "ui-gallery-file-tree-torture"
        | "ui-gallery-file-tree-torture-interactive"
        | "ui-gallery-cache005" => {
            let scripts = scripts_from_suite_dir(suite_name)?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "workspace-shell-demo" => {
            let scripts = scripts_from_suite_dir("workspace-shell-demo")?;
            (scripts, None)
        }
        "workspace-shell-demo-file-tree-keep-alive" => {
            let scripts = scripts_from_suite_dir("workspace-shell-demo-file-tree-keep-alive")?;
            (scripts, None)
        }
        "ui-gallery-data-table-retained-keep-alive" => {
            let scripts = scripts_from_suite_dir("ui-gallery-data-table-retained-keep-alive")?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-ai-transcript-retained" => {
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
            push_env_if_missing(
                launch_env,
                "FRET_UI_GALLERY_AI_TRANSCRIPT_VARIABLE_HEIGHT",
                "1",
            );
            let scripts = scripts_from_suite_dir("ui-gallery-ai-transcript-retained")?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-canvas-cull"
        | "ui-gallery-node-graph-cull"
        | "ui-gallery-node-graph-cull-window-shifts"
        | "ui-gallery-node-graph-cull-window-no-shifts-small-pan"
        | "ui-gallery-chart-torture" => {
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
            // This harness uses `capture_screenshot` to enable the `--check-pixels-changed` gate.
            push_env_if_missing(launch_env, "FRET_DIAG_GPU_SCREENSHOTS", "1");
            let scripts = scripts_from_suite_dir(suite_name)?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-vlist-window-boundary" => {
            // The window-boundary harness is specifically intended to exercise the
            // view-cache + shell reuse seam under a stable (known-heights) VirtualList
            // baseline. Make these env defaults implicit so the suite is reproducible
            // without requiring the caller to remember a pile of `--env` flags.
            //
            // Callers can still override them explicitly via `--env KEY=...`.
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS", "1");
            // Default to the non-retained VirtualList path so this harness gates the
            // highest-risk, most common implementation track (ADR 0175 Track B). The
            // retained-host track (ADR 0177) has dedicated suites/scripts.
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VLIST_RETAINED", "0");
            let scripts = scripts_from_suite_dir("ui-gallery-vlist-window-boundary")?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-vlist-no-window-shifts-small-scroll" => {
            // Guard rail harness: under view-cache + shell, small scroll deltas should
            // not force a non-retained VirtualList window shift (which currently implies
            // a cache-root rerender to rebuild visible items).
            //
            // Callers can still override env explicitly via `--env KEY=...`.
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VLIST_MINIMAL", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VLIST_RETAINED", "0");
            let scripts = scripts_from_suite_dir("ui-gallery-vlist-no-window-shifts-small-scroll")?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "ui-gallery-vlist-window-boundary-retained" => {
            // Retained-host counterpart of the window-boundary harness. This suite is used
            // to validate the ADR 0177 track (retained reconcile) with the same script and
            // baseline env, while keeping the non-retained suite as the default.
            //
            // Callers can still override them explicitly via `--env KEY=...`.
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VIEW_CACHE_SHELL", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS", "1");
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VLIST_RETAINED", "1");
            // Enable keep-alive in the retained-host harness so boundary scroll back can
            // reuse detached row subtrees (reduces attach cost and stabilizes worst tick).
            push_env_if_missing(launch_env, "FRET_UI_GALLERY_VLIST_KEEP_ALIVE", "128");
            let scripts = scripts_from_suite_dir("ui-gallery-vlist-window-boundary-retained")?;
            (scripts, Some(BuiltinSuite::UiGallery))
        }
        "components-gallery-file-tree" => {
            // components_gallery's "file tree torture" surface is behind env gates; the
            // scripted harness assumes it is enabled and large enough to cross overscan
            // boundaries deterministically.
            push_env_if_missing(launch_env, "FRET_COMPONENTS_GALLERY_FILE_TREE_TORTURE", "1");
            push_env_if_missing(
                launch_env,
                "FRET_COMPONENTS_GALLERY_FILE_TREE_TORTURE_N",
                "50000",
            );
            // Enable view-cache reuse by default for suite regressions. (components_gallery
            // reads `FRET_EXAMPLES_VIEW_CACHE`.)
            push_env_if_missing(launch_env, "FRET_EXAMPLES_VIEW_CACHE", "1");
            // Keep-alive is only observed by the `*bounce*` script, but setting it here
            // keeps the suite defaults consistent.
            push_env_if_missing(
                launch_env,
                "FRET_COMPONENTS_GALLERY_FILE_TREE_KEEP_ALIVE",
                "256",
            );
            let scripts = scripts_from_suite_dir("components-gallery-file-tree")?;
            (scripts, None)
        }
        "components-gallery-table" => {
            // components_gallery's "table torture" surface is behind an env gate; the
            // scripted harness assumes it is enabled.
            push_env_if_missing(launch_env, "FRET_COMPONENTS_GALLERY_TABLE_TORTURE", "1");
            push_env_if_missing(
                launch_env,
                "FRET_COMPONENTS_GALLERY_TABLE_TORTURE_N",
                "50000",
            );
            push_env_if_missing(launch_env, "FRET_EXAMPLES_VIEW_CACHE", "1");
            let scripts = scripts_from_suite_dir("components-gallery-table")?;
            (scripts, None)
        }
        "components-gallery-table-keep-alive" => {
            push_env_if_missing(launch_env, "FRET_COMPONENTS_GALLERY_TABLE_TORTURE", "1");
            push_env_if_missing(
                launch_env,
                "FRET_COMPONENTS_GALLERY_TABLE_TORTURE_N",
                "50000",
            );
            push_env_if_missing(launch_env, "FRET_EXAMPLES_VIEW_CACHE", "1");
            push_env_if_missing(
                launch_env,
                "FRET_COMPONENTS_GALLERY_TABLE_KEEP_ALIVE",
                "256",
            );
            let scripts = scripts_from_suite_dir("components-gallery-table-keep-alive")?;
            (scripts, None)
        }
        "docking-motion-pilot" => {
            let scripts = scripts_from_suite_dir("docking-motion-pilot")?;
            (scripts, Some(BuiltinSuite::DockingMotionPilot))
        }
        "docking-arbitration" => {
            let inputs = diag_suite_scripts::docking_arbitration_suite_scripts();
            let scripts = expand_script_inputs(workspace_root, &inputs)?;
            (scripts, Some(BuiltinSuite::DockingArbitration))
        }
        _ => return Ok(None),
    };

    Ok(Some(resolved))
}

#[derive(Debug, Clone)]
pub(crate) struct SuiteCmdContext {
    pub pack_after_run: bool,
    pub rest: Vec<String>,
    pub suite_script_inputs: Vec<String>,
    pub suite_prewarm_scripts: Vec<PathBuf>,
    pub suite_prelude_scripts: Vec<PathBuf>,
    pub suite_prelude_each_run: bool,
    pub workspace_root: PathBuf,
    pub resolved_out_dir: PathBuf,
    pub resolved_ready_path: PathBuf,
    pub resolved_script_result_path: PathBuf,
    pub devtools_ws_url: Option<String>,
    pub devtools_token: Option<String>,
    pub devtools_session_id: Option<String>,
    pub timeout_ms: u64,
    pub poll_ms: u64,
    pub stats_top: usize,
    pub stats_json: bool,
    pub warmup_frames: u64,
    pub max_test_ids: usize,
    pub lint_all_test_ids_bounds: bool,
    pub lint_eps_px: f32,
    pub suite_lint: bool,
    pub pack_include_screenshots: bool,
    pub reuse_launch: bool,
    pub launch: Option<Vec<String>>,
    pub launch_env: Vec<(String, String)>,
    pub launch_high_priority: bool,
    pub launch_write_bundle_json: bool,
    pub keep_open: bool,
    pub checks: SuiteChecks,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_suite(ctx: SuiteCmdContext) -> Result<(), String> {
    let SuiteCmdContext {
        pack_after_run,
        rest,
        suite_script_inputs,
        suite_prewarm_scripts,
        suite_prelude_scripts,
        suite_prelude_each_run,
        workspace_root,
        resolved_out_dir,
        resolved_ready_path,
        resolved_script_result_path,
        devtools_ws_url,
        devtools_token,
        devtools_session_id,
        timeout_ms,
        poll_ms,
        stats_top: _stats_top,
        stats_json,
        mut warmup_frames,
        max_test_ids: _max_test_ids,
        lint_all_test_ids_bounds,
        lint_eps_px,
        suite_lint,
        pack_include_screenshots,
        reuse_launch,
        launch,
        mut launch_env,
        launch_high_priority,
        launch_write_bundle_json,
        keep_open,
        checks,
    } = ctx;

    let checks_for_post_run_template = checks.clone();

    let SuiteChecks {
        check_chart_sampling_window_shifts_min,
        check_dock_drag_min,
        check_drag_cache_root_paint_only_test_id,
        check_gc_sweep_liveness: _,
        check_hover_layout_max,
        check_idle_no_paint_min,
        check_layout_fast_path_min,
        check_node_graph_cull_window_shifts_max,
        check_node_graph_cull_window_shifts_min,
        check_notify_hotspot_file_max,
        check_triage_hint_absent_codes: _,
        check_overlay_synthesis_min,
        check_pixels_changed_test_id,
        check_pixels_unchanged_test_id,
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
        check_windowed_rows_visible_start_changes_repainted,
        dump_semantics_changed_repainted_json: _,
    } = checks;

    let wants_registered_post_run_checks = crate::registry::checks::CheckRegistry::builtin()
        .wants_post_run_checks(&checks_for_post_run_template);

    // Tool-launched suites default to *not* redacting text to keep authoring/debugging ergonomic.
    //
    // Privacy-sensitive workflows (pack/share/CI) should explicitly opt back into redaction via:
    // `--env FRET_DIAG_REDACT_TEXT=1` (or by inheriting it from the parent environment).
    push_env_if_missing(&mut launch_env, "FRET_DIAG_REDACT_TEXT", "0");
    // Match `diag run` launch defaults: keep the app actively rendering so fixed-delta script
    // timeouts and keepalive timers remain reliable under OS occlusion/throttling.
    push_env_if_missing(&mut launch_env, "FRET_DIAG_RENDERER_PERF", "1");

    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let (bundle_doctor_mode, rest) = parse_bundle_doctor_mode_from_rest(&rest)?;
    if rest.is_empty() && suite_script_inputs.is_empty() {
        return Err(
            "missing suite name or script paths (try: fretboard diag suite ui-gallery | fretboard diag suite docking-arbitration)\n\
hint: list suites via `fretboard diag list suites`"
                .to_string(),
        );
    }

    let suite_args: Vec<String> = rest.clone();
    let strict_termination = suite_args.len() == 1
        && suite_args[0].starts_with("diag-hardening-smoke");

    let is_suite = |name: &str| suite_args.len() == 1 && suite_args[0] == name;
    let is_ui_gallery_ai_transcript_retained_suite = is_suite("ui-gallery-ai-transcript-retained");
    let is_ui_gallery_canvas_cull_suite = is_suite("ui-gallery-canvas-cull");
    let is_ui_gallery_node_graph_cull_suite = is_suite("ui-gallery-node-graph-cull");
    let is_ui_gallery_node_graph_cull_window_shifts_suite =
        is_suite("ui-gallery-node-graph-cull-window-shifts");
    let is_ui_gallery_node_graph_cull_window_no_shifts_small_pan_suite =
        is_suite("ui-gallery-node-graph-cull-window-no-shifts-small-pan");
    let is_ui_gallery_chart_torture_suite = is_suite("ui-gallery-chart-torture");
    let is_ui_gallery_vlist_window_boundary_suite = is_suite("ui-gallery-vlist-window-boundary");
    let is_ui_gallery_vlist_window_boundary_retained_suite =
        is_suite("ui-gallery-vlist-window-boundary-retained");
    let is_ui_gallery_vlist_no_window_shifts_small_scroll_suite =
        is_suite("ui-gallery-vlist-no-window-shifts-small-scroll");
    let is_components_gallery_file_tree_suite = is_suite("components-gallery-file-tree");
    let is_components_gallery_table_suite = is_suite("components-gallery-table");
    let is_components_gallery_table_keep_alive_suite =
        is_suite("components-gallery-table-keep-alive");

    let suite_resolver = SuiteResolver::try_load_from_workspace_root(&workspace_root)?;

    let mut used_fallback_paths = false;
    let (mut scripts, builtin_suite): (Vec<PathBuf>, Option<BuiltinSuite>) =
        if suite_args.is_empty() {
            (Vec::new(), None)
        } else if suite_args.len() == 1 {
            let suite_name = suite_args[0].as_str();
            if let Some(resolved) =
                resolve_builtin_suite_scripts(&workspace_root, suite_name, &mut launch_env)?
            {
                resolved
            } else if let Some(scripts) =
                suite_resolver.resolve_suite_scripts(&workspace_root, suite_name)?
            {
                (scripts, None)
            } else {
                used_fallback_paths = true;
                (
                    suite_args
                        .into_iter()
                        .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                        .collect(),
                    None,
                )
            }
        } else {
            used_fallback_paths = true;
            (
                suite_args
                    .into_iter()
                    .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
                    .collect(),
                None,
            )
        };

    if !suite_script_inputs.is_empty() {
        scripts.extend(expand_script_inputs(&workspace_root, &suite_script_inputs)?);
        scripts.sort();
        scripts.dedup();
    }

    if scripts.is_empty() {
        return Err("suite produced no scripts".to_string());
    }
    if strict_termination {
        let issues = crate::script_tooling::preflight_strict_termination_issues(&scripts)?;
        if !issues.is_empty() {
            let out = resolved_out_dir.join("check.script_termination.json");
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "diag_script_termination_preflight",
                "status": "failed",
                "issue_count": issues.len(),
                "issues": issues,
            });
            let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
            let _ = std::fs::create_dir_all(&resolved_out_dir);
            let _ = std::fs::write(&out, pretty.as_bytes());

            return Err(format!(
                "suite script termination preflight failed (issue_count={}) (see: {})\n\
hint: smoke/gate suites require deterministic termination; avoid trailing wait_frames/wait_ms and avoid wait_frames/wait_ms after the final capture_bundle",
                payload["issue_count"].as_u64().unwrap_or(0),
                out.display()
            ));
        }
    }

    if used_fallback_paths
        && suite_script_inputs.is_empty()
        && rest.len() == 1
        && scripts.len() == 1
        && !scripts[0].exists()
    {
        let name = rest[0].as_str();
        let looks_like_suite_name = !name.contains(['/', '\\', ':']) && !name.ends_with(".json");
        if looks_like_suite_name {
            return Err(format!(
                "unknown suite or script path: {name:?}\n\
hint: list suites via `fretboard diag list suites --contains {name}`\n\
hint: list promoted scripts via `fretboard diag list scripts --contains {name}`"
            ));
        }
        return Err(format!(
            "script path does not exist: {}",
            scripts[0].display()
        ));
    }

    let suite_wants_screenshots = pack_include_screenshots
        || crate::registry::checks::CheckRegistry::builtin()
            .wants_screenshots(&checks_for_post_run_template)
        || (check_pixels_changed_test_id.is_none()
            && check_pixels_unchanged_test_id.is_none()
            && scripts
                .iter()
                .any(|src| diag_policy::ui_gallery_script_pixels_changed_test_id(src).is_some()))
        || scripts.iter().any(|src| script_requests_screenshots(src));
    // Suite defaults: most suites only need a small warmup to skip startup churn, but
    // "no shift" gates should avoid the initial VirtualList window stabilization phase.
    if warmup_frames == 0 && is_ui_gallery_vlist_no_window_shifts_small_scroll_suite {
        warmup_frames = 32;
    }

    if warmup_frames == 0
        && (is_ui_gallery_vlist_window_boundary_suite
            || is_ui_gallery_vlist_window_boundary_retained_suite
            || is_ui_gallery_canvas_cull_suite
            || is_ui_gallery_node_graph_cull_suite
            || is_ui_gallery_node_graph_cull_window_shifts_suite
            || is_ui_gallery_node_graph_cull_window_no_shifts_small_pan_suite
            || is_ui_gallery_chart_torture_suite
            || is_ui_gallery_ai_transcript_retained_suite)
    {
        warmup_frames = 5;
    }

    let use_devtools_ws =
        devtools_ws_url.is_some() || devtools_token.is_some() || devtools_session_id.is_some();
    let reuse_process = use_devtools_ws || launch.is_none() || reuse_launch;
    let tool_launched = launch.is_some() || reuse_launch;

    if reuse_process {
        // If the suite reuses a single process, we must pick a single launch env for the whole run.
        // We treat `meta.env_defaults` as "suite-affecting" in this mode (and reject conflicts).
        let mut suite_script_env_defaults: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        let mut suite_env_conflicts: Vec<String> = Vec::new();
        for src in scripts.iter() {
            for (key, value) in script_env_defaults(src) {
                if let Some(prev) = suite_script_env_defaults.insert(key.clone(), value.clone())
                    && prev != value
                {
                    suite_env_conflicts.push(format!(
                        "{} wants {}={}, but another script requested {}={}",
                        src.display(),
                        key,
                        value,
                        key,
                        prev
                    ));
                }
            }
        }
        if !suite_env_conflicts.is_empty() {
            suite_env_conflicts.sort();
            return Err(format!(
                "conflicting script meta.env_defaults in suite:\n- {}",
                suite_env_conflicts.join("\n- ")
            ));
        }
        for (key, value) in suite_script_env_defaults {
            push_env_if_missing(&mut launch_env, &key, &value);
        }
    }

    let suite_launch_env = launch_env.clone();

    let resolved_exit_path = {
        let raw = std::env::var_os("FRET_DIAG_EXIT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| resolved_out_dir.join("exit.touch"));
        resolve_path(&workspace_root, raw)
    };

    let resolved_script_result_trigger_path = {
        let raw = std::env::var_os("FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| resolved_out_dir.join("script.result.touch"));
        resolve_path(&workspace_root, raw)
    };

    let mut fs_transport_cfg =
        crate::transport::FsDiagTransportConfig::from_out_dir(resolved_out_dir.clone());
    fs_transport_cfg.script_result_path = resolved_script_result_path.clone();
    fs_transport_cfg.script_result_trigger_path = resolved_script_result_trigger_path.clone();

    let trace_chrome = false;

    let suite_summary_path = resolved_out_dir.join("suite.summary.json");
    let suite_summary_suite = (rest.len() == 1).then(|| rest[0].clone());
    let suite_summary_generated_unix_ms = now_unix_ms();
    let mut suite_stage_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut suite_reason_code_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut suite_rows: Vec<serde_json::Value> = Vec::new();
    let mut suite_evidence_agg = suite_summary::SuiteEvidenceAggregate::default();

    let capabilities_check_path = resolved_out_dir.join("check.capabilities.json");
    let resolved_suite_prewarm_scripts: Vec<PathBuf> = suite_prewarm_scripts
        .into_iter()
        .map(|p| resolve_path(&workspace_root, p))
        .collect();
    let resolved_suite_prelude_scripts: Vec<PathBuf> = suite_prelude_scripts
        .into_iter()
        .map(|p| resolve_path(&workspace_root, p))
        .collect();

    let connected_ws: Option<ConnectedToolingTransport> = if use_devtools_ws {
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

        match connect_devtools_ws_tooling(
            ws_url.as_str(),
            token.as_str(),
            devtools_session_id.as_deref(),
            timeout_ms,
            poll_ms,
        ) {
            Ok(v) => Some(v),
            Err(err) => {
                write_tooling_failure_script_result(
                    &resolved_script_result_path,
                    "tooling.connect.failed",
                    &err,
                    "tooling_error",
                    Some("connect_devtools_ws_tooling".to_string()),
                );
                suite_rows.push(serde_json::json!({
                    "error_code": "tooling.connect.failed",
                    "reason_code": "tooling.connect.failed",
                    "error": err,
                }));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": suite_summary_generated_unix_ms,
                    "kind": "suite_summary",
                    "status": "error",
                    "error_reason_code": "tooling.connect.failed",
                    "suite": suite_summary_suite,
                    "out_dir": resolved_out_dir.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "reuse_launch": reuse_launch,
                    "wants_screenshots": suite_wants_screenshots,
                    "stage_counts": suite_stage_counts,
                    "reason_code_counts": suite_reason_code_counts,
                    "evidence_aggregate": suite_evidence_agg.as_json(),
                    "rows": suite_rows,
                });
                let _ = write_json_value(&suite_summary_path, &payload);
                return Err("suite setup failed (see suite.summary.json)".to_string());
            }
        }
    } else {
        None
    };

    let mut child = if use_devtools_ws {
        None
    } else if reuse_process {
        match maybe_launch_demo(
            &launch,
            &suite_launch_env,
            &workspace_root,
            &resolved_ready_path,
            &resolved_exit_path,
            &fs_transport_cfg,
            suite_wants_screenshots,
            launch_write_bundle_json,
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
                suite_rows.push(serde_json::json!({
                    "error_code": "tooling.launch.failed",
                    "reason_code": "tooling.launch.failed",
                    "error": err,
                }));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": suite_summary_generated_unix_ms,
                    "kind": "suite_summary",
                    "status": "error",
                    "error_reason_code": "tooling.launch.failed",
                    "suite": suite_summary_suite,
                    "out_dir": resolved_out_dir.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "reuse_launch": reuse_launch,
                    "wants_screenshots": suite_wants_screenshots,
                    "stage_counts": suite_stage_counts,
                    "reason_code_counts": suite_reason_code_counts,
                    "evidence_aggregate": suite_evidence_agg.as_json(),
                    "rows": suite_rows,
                });
                let _ = write_json_value(&suite_summary_path, &payload);
                return Err("suite setup failed (see suite.summary.json)".to_string());
            }
        }
    } else {
        None
    };

    let connected_fs: Option<ConnectedToolingTransport> = if !use_devtools_ws && reuse_process {
        match connect_filesystem_tooling(
            &fs_transport_cfg,
            &resolved_ready_path,
            child.is_some(),
            timeout_ms,
            poll_ms,
        ) {
            Ok(v) => Some(v),
            Err(err) => {
                write_tooling_failure_script_result(
                    &resolved_script_result_path,
                    "tooling.connect.failed",
                    &err,
                    "tooling_error",
                    Some("connect_filesystem_tooling".to_string()),
                );
                suite_rows.push(serde_json::json!({
                    "error_code": "tooling.connect.failed",
                    "reason_code": "tooling.connect.failed",
                    "error": err,
                }));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": suite_summary_generated_unix_ms,
                    "kind": "suite_summary",
                    "status": "error",
                    "error_reason_code": "tooling.connect.failed",
                    "suite": suite_summary_suite,
                    "out_dir": resolved_out_dir.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "reuse_launch": reuse_launch,
                    "wants_screenshots": suite_wants_screenshots,
                    "stage_counts": suite_stage_counts,
                    "reason_code_counts": suite_reason_code_counts,
                    "evidence_aggregate": suite_evidence_agg.as_json(),
                    "rows": suite_rows,
                });
                if !keep_open {
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                }
                let _ = write_json_value(&suite_summary_path, &payload);
                return Err("suite setup failed (see suite.summary.json)".to_string());
            }
        }
    } else {
        None
    };

    let script_count = scripts.len();
    for (idx, src) in scripts.into_iter().enumerate() {
        let script_key = normalize_repo_relative_path(&workspace_root, &src);
        if !reuse_process {
            let mut per_script_launch_env = suite_launch_env.clone();
            for (key, value) in script_env_defaults(&src) {
                push_env_if_missing(&mut per_script_launch_env, &key, &value);
            }
            child = match maybe_launch_demo(
                &launch,
                &per_script_launch_env,
                &workspace_root,
                &resolved_ready_path,
                &resolved_exit_path,
                &fs_transport_cfg,
                suite_wants_screenshots,
                launch_write_bundle_json,
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
                        Some(script_key.clone()),
                    );
                    suite_rows.push(serde_json::json!({
                        "script": script_key,
                        "error_code": "tooling.launch.failed",
                        "reason_code": "tooling.launch.failed",
                        "error": err,
                    }));
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "generated_unix_ms": suite_summary_generated_unix_ms,
                        "kind": "suite_summary",
                        "status": "error",
                        "error_reason_code": "tooling.launch.failed",
                        "suite": suite_summary_suite,
                        "out_dir": resolved_out_dir.display().to_string(),
                        "warmup_frames": warmup_frames,
                        "reuse_launch": reuse_launch,
                        "wants_screenshots": suite_wants_screenshots,
                        "stage_counts": suite_stage_counts,
                        "reason_code_counts": suite_reason_code_counts,
                        "evidence_aggregate": suite_evidence_agg.as_json(),
                        "rows": suite_rows,
                    });
                    if !keep_open {
                        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                    }
                    let _ = write_json_value(&suite_summary_path, &payload);
                    return Err("suite run failed (see suite.summary.json)".to_string());
                }
            };
        }
        let result: Result<crate::stats::ScriptResultSummary, String> = (|| {
            let child_running = child.is_some();
            let connected_fs_iter: ConnectedToolingTransport;
            let connected: &ConnectedToolingTransport = if use_devtools_ws {
                connected_ws.as_ref().ok_or_else(|| {
                    "missing DevTools WS transport (this is a tooling bug)".to_string()
                })?
            } else if reuse_process {
                connected_fs.as_ref().ok_or_else(|| {
                    "missing filesystem transport (this is a tooling bug)".to_string()
                })?
            } else {
                connected_fs_iter = connect_filesystem_tooling(
                    &fs_transport_cfg,
                    &resolved_ready_path,
                    child_running,
                    timeout_ms,
                    poll_ms,
                )
                .inspect_err(|err| {
                    write_tooling_failure_script_result(
                        &resolved_script_result_path,
                        "tooling.connect.failed",
                        err,
                        "tooling_error",
                        Some(script_key.clone()),
                    );
                })?;
                &connected_fs_iter
            };

            let connected_fs_for_aux = if use_devtools_ws {
                None
            } else {
                Some(connected)
            };
            if !resolved_suite_prewarm_scripts.is_empty() && (!reuse_process || idx == 0) {
                for prewarm in &resolved_suite_prewarm_scripts {
                    crate::diag_perf::run_suite_aux_script_must_pass(
                        prewarm,
                        tool_launched,
                        &mut child,
                        use_devtools_ws,
                        connected_ws.as_ref(),
                        connected_fs_for_aux,
                        &workspace_root,
                        &resolved_out_dir,
                        &resolved_exit_path,
                        !keep_open,
                        reuse_process,
                        &resolved_script_result_path,
                        &resolved_script_result_trigger_path,
                        &capabilities_check_path,
                        timeout_ms,
                        poll_ms,
                    )?;
                }
            }
            if !resolved_suite_prelude_scripts.is_empty()
                && (!reuse_process || suite_prelude_each_run || idx == 0)
            {
                for prelude in &resolved_suite_prelude_scripts {
                    crate::diag_perf::run_suite_aux_script_must_pass(
                        prelude,
                        tool_launched,
                        &mut child,
                        use_devtools_ws,
                        connected_ws.as_ref(),
                        connected_fs_for_aux,
                        &workspace_root,
                        &resolved_out_dir,
                        &resolved_exit_path,
                        !keep_open,
                        reuse_process,
                        &resolved_script_result_path,
                        &resolved_script_result_trigger_path,
                        &capabilities_check_path,
                        timeout_ms,
                        poll_ms,
                    )?;
                }
            }

            let (script_json, upgraded) = crate::script_execution::load_script_json_for_execution(
                &src,
                crate::script_execution::ScriptLoadPolicy {
                    tool_launched,
                    write_failure: write_tooling_failure_script_result,
                    failure_note: Some(script_key.clone()),
                    include_stage_in_note: false,
                },
                &resolved_script_result_path,
            )?;
            if upgraded {
                eprintln!(
                    "warning: script schema_version=1 detected; tooling upgraded to schema_version=2 for execution (source={})",
                    src.display()
                );
            }

            // Always dump a bounded bundle for suite runs so lint and post-run checks can
            // operate on a local artifact (parity across transports).
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
                let mut label = format!("suite-{idx:04}-{sanitized}");
                if label.len() > 80 {
                    label.truncate(80);
                    label = label.trim_matches('-').to_string();
                }
                label
            };

            let (script_result, _bundle_path) = run_script_over_transport(
                &resolved_out_dir,
                connected,
                script_json,
                true,
                trace_chrome,
                Some(dump_label.as_str()),
                None,
                timeout_ms,
                poll_ms,
                &resolved_script_result_path,
                &capabilities_check_path,
            )
            .inspect_err(|err| {
                write_tooling_failure_script_result_if_missing(
                    &resolved_script_result_path,
                    "tooling.run.failed",
                    err,
                    "tooling_error",
                    Some(script_key.clone()),
                );
            })?;

            let stage = match script_result.stage {
                fret_diag_protocol::UiScriptStageV1::Passed => "passed",
                fret_diag_protocol::UiScriptStageV1::Failed => "failed",
                fret_diag_protocol::UiScriptStageV1::Queued => "queued",
                fret_diag_protocol::UiScriptStageV1::Running => "running",
            };

            Ok(crate::stats::ScriptResultSummary {
                run_id: script_result.run_id,
                stage: Some(stage.to_string()),
                step_index: script_result.step_index.map(|n| n as u64),
                reason_code: script_result.reason_code.clone(),
                reason: script_result.reason.clone(),
                last_bundle_dir: script_result.last_bundle_dir.clone(),
            })
        })();

        let result = match result {
            Ok(v) => v,
            Err(e) => {
                let tooling_reason_code =
                    read_json_value(&resolved_script_result_path).and_then(|v| {
                        v.get("reason_code")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    });
                let error_reason_code = tooling_reason_code
                    .clone()
                    .unwrap_or_else(|| "tooling.suite.error".to_string());
                suite_rows.push(serde_json::json!({
                    "script": script_key.clone(),
                    "error_code": "tooling.suite.error",
                    "reason_code": tooling_reason_code,
                    "error": e,
                }));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": suite_summary_generated_unix_ms,
                    "kind": "suite_summary",
                    "status": "error",
                    "error_reason_code": error_reason_code,
                    "suite": suite_summary_suite,
                    "out_dir": resolved_out_dir.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "reuse_launch": reuse_launch,
                    "wants_screenshots": suite_wants_screenshots,
                    "stage_counts": suite_stage_counts,
                    "reason_code_counts": suite_reason_code_counts,
                    "evidence_aggregate": suite_evidence_agg.as_json(),
                    "rows": suite_rows,
                });
                if !keep_open {
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                }
                let _ = write_json_value(&suite_summary_path, &payload);
                return Err("suite run failed (see suite.summary.json)".to_string());
            }
        };

        if let Some(stage) = result.stage.as_deref() {
            *suite_stage_counts.entry(stage.to_string()).or_default() += 1;
        }
        if let Some(code) = result.reason_code.as_deref()
            && !code.trim().is_empty()
        {
            *suite_reason_code_counts
                .entry(code.to_string())
                .or_default() += 1;
        }

        let script_key = normalize_repo_relative_path(&workspace_root, &src);
        let mut lint_summary: Option<serde_json::Value> = None;
        let evidence_highlights = suite_summary::evidence_highlights_from_script_result_path(
            &resolved_script_result_path,
            &mut suite_evidence_agg,
        );
        match result.stage.as_deref() {
            Some("passed") => {
                println!("PASS {} (run_id={})", src.display(), result.run_id)
            }
            Some("failed") => {
                eprintln!(
                    "FAIL {} (run_id={}) step={} reason={} last_bundle_dir={}",
                    src.display(),
                    result.run_id,
                    result.step_index.unwrap_or(0),
                    result.reason.as_deref().unwrap_or("unknown"),
                    result.last_bundle_dir.as_deref().unwrap_or("")
                );
                suite_rows.push(serde_json::json!({
                    "script": script_key,
                    "run_id": result.run_id,
                    "stage": result.stage,
                    "step_index": result.step_index,
                    "reason_code": result.reason_code,
                    "reason": result.reason,
                    "last_bundle_dir": result.last_bundle_dir,
                    "lint": lint_summary,
                    "evidence_highlights": evidence_highlights,
                }));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": suite_summary_generated_unix_ms,
                    "kind": "suite_summary",
                    "status": "failed",
                    "suite": suite_summary_suite,
                    "out_dir": resolved_out_dir.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "reuse_launch": reuse_launch,
                    "wants_screenshots": suite_wants_screenshots,
                    "stage_counts": suite_stage_counts,
                    "reason_code_counts": suite_reason_code_counts,
                    "evidence_aggregate": suite_evidence_agg.as_json(),
                    "rows": suite_rows,
                });
                if !keep_open {
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                }
                let _ = write_json_value(&suite_summary_path, &payload);
                std::process::exit(1);
            }
            _ => {
                eprintln!(
                    "unexpected script stage for {}: {:?}",
                    src.display(),
                    result
                );
                suite_rows.push(serde_json::json!({
                    "script": script_key,
                    "run_id": result.run_id,
                    "stage": result.stage,
                    "step_index": result.step_index,
                    "reason_code": result.reason_code,
                    "reason": result.reason,
                    "last_bundle_dir": result.last_bundle_dir,
                    "lint": lint_summary,
                    "evidence_highlights": evidence_highlights,
                }));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_unix_ms": suite_summary_generated_unix_ms,
                    "kind": "suite_summary",
                    "status": "failed",
                    "suite": suite_summary_suite,
                    "out_dir": resolved_out_dir.display().to_string(),
                    "warmup_frames": warmup_frames,
                    "reuse_launch": reuse_launch,
                    "wants_screenshots": suite_wants_screenshots,
                    "stage_counts": suite_stage_counts,
                    "reason_code_counts": suite_reason_code_counts,
                    "evidence_aggregate": suite_evidence_agg.as_json(),
                    "rows": suite_rows,
                });
                if !keep_open {
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                }
                let _ = write_json_value(&suite_summary_path, &payload);
                std::process::exit(1);
            }
        }

        if suite_lint {
            let last_bundle_dir = result
                .last_bundle_dir
                .as_deref()
                .and_then(|s| (!s.trim().is_empty()).then_some(s.trim()));
            if let Some(last_bundle_dir) = last_bundle_dir {
                let bundle_dir = PathBuf::from(last_bundle_dir);
                let bundle_dir = if bundle_dir.is_absolute() {
                    bundle_dir
                } else {
                    resolved_out_dir.join(bundle_dir)
                };
                let bundle_path = wait_for_bundle_artifact_in_dir(&bundle_dir, timeout_ms, poll_ms)
                    .ok_or_else(|| {
                        format!(
                            "suite lint is enabled but no bundle artifact was found in time: {}",
                            bundle_dir.display()
                        )
                    })?;

                if bundle_doctor_mode != BundleDoctorMode::Off {
                    run_bundle_doctor_for_bundle_path(
                        &bundle_path,
                        bundle_doctor_mode,
                        warmup_frames,
                    )?;
                }

                let report = lint_bundle_from_path(
                    &bundle_path,
                    warmup_frames,
                    LintOptions {
                        all_test_ids_bounds: lint_all_test_ids_bounds,
                        eps_px: lint_eps_px,
                    },
                )?;

                let out = default_lint_out_path(&bundle_path);
                lint_summary = Some(serde_json::json!({
                    "out": out.display().to_string(),
                    "error_issues": report
                        .payload
                        .get("error_issues")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(report.error_issues),
                    "warning_issues": report
                        .payload
                        .get("warning_issues")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    "counts_by_code": report.payload.get("counts_by_code").cloned(),
                }));
                if let Some(parent) = out.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let pretty = serde_json::to_string_pretty(&report.payload)
                    .unwrap_or_else(|_| "{}".to_string());
                std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

                if report.error_issues > 0 {
                    eprintln!(
                        "LINT-FAIL {} (run_id={}) errors={} out={}",
                        src.display(),
                        result.run_id,
                        report.error_issues,
                        out.display()
                    );
                    suite_rows.push(serde_json::json!({
                        "script": script_key,
                        "run_id": result.run_id,
                        "stage": result.stage,
                        "step_index": result.step_index,
                        "reason_code": result.reason_code,
                        "reason": result.reason,
                        "last_bundle_dir": result.last_bundle_dir,
                        "lint": lint_summary,
                        "evidence_highlights": evidence_highlights,
                    }));
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "generated_unix_ms": suite_summary_generated_unix_ms,
                        "kind": "suite_summary",
                        "status": "failed",
                        "suite": suite_summary_suite,
                        "out_dir": resolved_out_dir.display().to_string(),
                        "warmup_frames": warmup_frames,
                        "reuse_launch": reuse_launch,
                        "wants_screenshots": suite_wants_screenshots,
                        "stage_counts": suite_stage_counts,
                        "reason_code_counts": suite_reason_code_counts,
                        "evidence_aggregate": suite_evidence_agg.as_json(),
                        "rows": suite_rows,
                        "failure_kind": "lint_failed",
                    });
                    stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
                    let _ = write_json_value(&suite_summary_path, &payload);
                    std::process::exit(1);
                }
            }
        }

        let retained_vlist_gate_for_script =
            check_retained_vlist_reconcile_no_notify_min.filter(|_| {
                diag_policy::ui_gallery_script_requires_retained_vlist_reconcile_gate(&src)
            });
        let retained_vlist_attach_detach_max_for_script = check_retained_vlist_attach_detach_max
            .filter(|_| {
                diag_policy::ui_gallery_script_requires_retained_vlist_reconcile_gate(&src)
            });
        let retained_vlist_keep_alive_reuse_min_for_script =
            check_retained_vlist_keep_alive_reuse_min.filter(|_| {
                diag_policy::ui_gallery_script_requires_retained_vlist_keep_alive_reuse_gate(&src)
            });
        let retained_vlist_keep_alive_budget_for_script = check_retained_vlist_keep_alive_budget
            .filter(|_| {
                diag_policy::ui_gallery_script_requires_retained_vlist_keep_alive_reuse_gate(&src)
            });
        let vlist_window_shifts_non_retained_max_for_script =
            check_vlist_window_shifts_non_retained_max.filter(|_| {
                diag_policy::ui_gallery_script_requires_retained_vlist_reconcile_gate(&src)
            });
        let wants_post_run_checks_for_script = wants_registered_post_run_checks
            || check_stale_paint_test_id.is_some()
            || check_stale_scene_test_id.is_some()
            || check_idle_no_paint_min.is_some()
            || check_ui_gallery_web_ime_bridge_enabled
            || check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps
            || check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change
            || check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change
            || check_ui_gallery_text_mixed_script_bundled_fallback_conformance
            || check_ui_gallery_code_editor_torture_marker_present
            || check_ui_gallery_code_editor_torture_undo_redo
            || check_ui_gallery_code_editor_torture_geom_fallbacks_low
            || check_ui_gallery_code_editor_torture_read_only_blocks_edits
            || check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit
            || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
            || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations
            || check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed
            || check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed
            || check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed
            || check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll
            || check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection
            || check_ui_gallery_code_editor_torture_folds_placeholder_present
            || check_ui_gallery_code_editor_torture_inlays_present
            || check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit
            || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped
            || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations
            || check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed
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
            || check_drag_cache_root_paint_only_test_id.is_some()
            || check_vlist_policy_key_stable
            || check_windowed_rows_offset_changes_min.is_some()
            || check_windowed_rows_visible_start_changes_repainted
            || check_layout_fast_path_min.is_some()
            || check_hover_layout_max.is_some()
            || check_view_cache_reuse_min.is_some()
            || check_view_cache_reuse_stable_min.is_some()
            || check_overlay_synthesis_min.is_some()
            || check_viewport_input_min.is_some()
            || check_dock_drag_min.is_some()
            || check_viewport_capture_min.is_some()
            || retained_vlist_gate_for_script.is_some()
            || retained_vlist_attach_detach_max_for_script.is_some()
            || retained_vlist_keep_alive_reuse_min_for_script.is_some()
            || retained_vlist_keep_alive_budget_for_script.is_some()
            || vlist_window_shifts_non_retained_max_for_script.is_some()
            || diag_policy::ui_gallery_script_requires_text_rescan_system_fonts_font_stack_key_bumps_gate(
                &src,
            )
            || diag_policy::ui_gallery_script_requires_windowed_rows_offset_changes_gate(&src)
            || diag_policy::ui_gallery_script_requires_windowed_rows_visible_start_repaint_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_read_only_blocks_edits_gate(
                &src,
            )
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_disabled_blocks_edits_gate(
                &src,
            )
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_soft_wrap_toggle_stable_gate(
                &src,
            )
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_word_boundary_gate(&src)
            || diag_policy::ui_gallery_script_requires_web_ime_bridge_enabled_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_line_boundary_triple_click_gate(
                &src,
            )
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_a11y_composition_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_a11y_composition_soft_wrap_gate(
                &src,
            )
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_folds_toggle_stable_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_gate(
                &src,
            )
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_under_soft_wrap_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_toggle_stable_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_caret_navigation_stable_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_present_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_present_under_soft_wrap_gate(&src)
            || diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_absent_under_inline_preedit_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_composed_preedit_stable_after_wheel_scroll_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_composed_preedit_cancels_on_drag_selection_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_gate(&src)
            || diag_policy::ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_gate(&src)
            || diag_policy::ui_gallery_script_wheel_scroll_hit_changes_test_id(&src).is_some()
            || diag_policy::ui_gallery_script_requires_retained_vlist_reconcile_gate(&src);

        let is_gc_liveness_script = src.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
            n == "ui-gallery-overlay-torture.json" || n == "ui-gallery-sidebar-scroll-refresh.json"
        });

        let wants_post_run_checks_for_script = wants_post_run_checks_for_script
            || builtin_suite == Some(BuiltinSuite::DockingArbitration)
            || builtin_suite == Some(BuiltinSuite::UiGalleryCodeEditor)
            || is_ui_gallery_canvas_cull_suite
            || is_ui_gallery_chart_torture_suite
            || is_ui_gallery_vlist_window_boundary_suite
            || is_ui_gallery_vlist_window_boundary_retained_suite
            || is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
            || is_components_gallery_file_tree_suite
            || is_components_gallery_table_suite
            || is_components_gallery_table_keep_alive_suite
            || (builtin_suite == Some(BuiltinSuite::UiGallery) && is_gc_liveness_script);

        if result.stage.as_deref() == Some("passed") && wants_post_run_checks_for_script {
            let bundle_path = wait_for_bundle_artifact_from_script_result(
                &resolved_out_dir,
                &result,
                timeout_ms,
                poll_ms,
            )
            .ok_or_else(|| {
                format!(
                    "script passed but no bundle artifact was found (required for post-run checks): {}",
                    src.display()
                )
            })?;

            if bundle_doctor_mode != BundleDoctorMode::Off {
                run_bundle_doctor_for_bundle_path(&bundle_path, bundle_doctor_mode, warmup_frames)?;
            }

            let (suite_viewport_input_min, suite_dock_drag_min, suite_viewport_capture_min) =
                if builtin_suite == Some(BuiltinSuite::DockingArbitration) {
                    diag_policy::docking_arbitration_script_default_gates(&src)
                } else {
                    (None, None, None)
                };
            let vlist_window_boundary_suite = is_ui_gallery_vlist_window_boundary_suite
                || is_ui_gallery_vlist_window_boundary_retained_suite;
            let vlist_window_boundary_retained_suite =
                is_ui_gallery_vlist_window_boundary_retained_suite;
            let components_gallery_suite = is_components_gallery_file_tree_suite
                || is_components_gallery_table_suite
                || is_components_gallery_table_keep_alive_suite;
            let pan_zoom_suite =
                is_ui_gallery_canvas_cull_suite || is_ui_gallery_chart_torture_suite;
            let ai_transcript_suite = is_ui_gallery_ai_transcript_retained_suite;
            let suite_wheel_scroll_test_id =
                is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
                    .then_some("ui-gallery-virtual-list-row-0-label")
                    .filter(|_| check_wheel_scroll_test_id.is_none());
            let suite_components_gallery_stale_paint_test_id =
                is_components_gallery_file_tree_suite
                    .then_some("components-gallery-file-tree-root")
                    .or_else(|| {
                        (is_components_gallery_table_suite
                            || is_components_gallery_table_keep_alive_suite)
                            .then_some("components-gallery-table-root")
                    })
                    .filter(|_| check_stale_paint_test_id.is_none());
            let suite_ai_transcript_stale_paint_test_id = ai_transcript_suite
                .then_some("ui-gallery-ai-transcript-row-0")
                .filter(|_| check_stale_paint_test_id.is_none());
            let suite_wheel_scroll_hit_changes_test_id =
                diag_policy::ui_gallery_script_wheel_scroll_hit_changes_test_id(&src)
                    .or_else(|| {
                        is_components_gallery_file_tree_suite
                            .then_some("components-gallery-file-tree-root")
                            .or_else(|| {
                                (is_components_gallery_table_suite
                                    || is_components_gallery_table_keep_alive_suite)
                                    .then_some("components-gallery-table-root")
                            })
                    })
                    .filter(|_| check_wheel_scroll_hit_changes_test_id.is_none());
            let suite_components_gallery_view_cache_reuse_min = components_gallery_suite
                .then_some(1u64)
                .filter(|_| check_view_cache_reuse_min.is_none());
            let suite_layout_fast_path_min = components_gallery_suite
                .then_some(1u64)
                .filter(|_| check_layout_fast_path_min.is_none());
            let suite_stale_paint_test_id = vlist_window_boundary_suite
                .then_some("ui-gallery-virtual-list-root")
                .or(suite_ai_transcript_stale_paint_test_id)
                .filter(|_| check_stale_paint_test_id.is_none());
            let suite_view_cache_reuse_min = (vlist_window_boundary_suite || pan_zoom_suite)
                .then_some(1u64)
                .or_else(|| ai_transcript_suite.then_some(10u64))
                .filter(|_| check_view_cache_reuse_min.is_none());
            let suite_view_cache_reuse_stable_min = ai_transcript_suite
                .then_some(10u64)
                .filter(|_| check_view_cache_reuse_stable_min.is_none());
            let suite_default_pixels_changed_test_id = is_ui_gallery_canvas_cull_suite
                .then_some("ui-gallery-canvas-cull-root")
                .or_else(|| {
                    is_ui_gallery_chart_torture_suite.then_some("ui-gallery-chart-torture-root")
                })
                .filter(|_| {
                    check_pixels_changed_test_id.is_none()
                        && check_pixels_unchanged_test_id.is_none()
                });
            let suite_vlist_visible_range_refreshes_min = vlist_window_boundary_suite
                .then_some(1u64)
                .filter(|_| check_vlist_visible_range_refreshes_min.is_none());
            let suite_vlist_visible_range_refreshes_max = vlist_window_boundary_suite
                // Default budget:
                // - Non-retained path: keep this relatively tight so we catch churn
                //   regressions early while still allowing prefetch shifts.
                // - Retained-host path: allow a looser cap since reconcile can legitimately
                //   refresh more often (and we have additional retained-only gates).
                .then_some(if vlist_window_boundary_retained_suite {
                    50u64
                } else {
                    20u64
                })
                .filter(|_| check_vlist_visible_range_refreshes_max.is_none());
            let suite_vlist_window_shifts_explainable =
                vlist_window_boundary_suite && !check_vlist_window_shifts_explainable;
            let suite_prepaint_actions_min = vlist_window_boundary_suite
                .then_some(1u64)
                .filter(|_| check_prepaint_actions_min.is_none());
            let suite_hover_layout_max = ai_transcript_suite
                .then_some(0u32)
                .filter(|_| check_hover_layout_max.is_none());
            let suite_chart_sampling_window_shifts_min = is_ui_gallery_chart_torture_suite
                .then_some(1u64)
                .filter(|_| check_chart_sampling_window_shifts_min.is_none());
            let suite_node_graph_cull_window_shifts_min =
                is_ui_gallery_node_graph_cull_window_shifts_suite
                    .then_some(1u64)
                    .or_else(|| is_ui_gallery_node_graph_cull_suite.then_some(0u64))
                    .filter(|_| check_node_graph_cull_window_shifts_min.is_none());
            let suite_node_graph_cull_window_shifts_max =
                is_ui_gallery_node_graph_cull_window_no_shifts_small_pan_suite
                    .then_some(0u64)
                    .filter(|_| check_node_graph_cull_window_shifts_max.is_none());
            let suite_vlist_window_shifts_have_prepaint_actions =
                vlist_window_boundary_suite && !check_vlist_window_shifts_have_prepaint_actions;
            let suite_vlist_window_shifts_prefetch_max = vlist_window_boundary_suite
                .then_some(if vlist_window_boundary_retained_suite {
                    100u64
                } else {
                    12u64
                })
                .filter(|_| check_vlist_window_shifts_prefetch_max.is_none());
            let suite_vlist_window_shifts_escape_max = vlist_window_boundary_suite
                .then_some(if vlist_window_boundary_retained_suite {
                    6u64
                } else {
                    4u64
                })
                .filter(|_| check_vlist_window_shifts_escape_max.is_none());
            let script_requires_retained_vlist_reconcile_gate =
                diag_policy::ui_gallery_script_requires_retained_vlist_reconcile_gate(&src)
                    || vlist_window_boundary_retained_suite;
            let suite_vlist_window_shifts_non_retained_max =
                script_requires_retained_vlist_reconcile_gate
                    .then_some(0u64)
                    .filter(|_| check_vlist_window_shifts_non_retained_max.is_none());
            let suite_vlist_small_scroll_window_shifts_non_retained_max =
                is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
                    .then_some(0u64)
                    .filter(|_| check_vlist_window_shifts_non_retained_max.is_none());
            let suite_vlist_small_scroll_window_shifts_prefetch_max =
                is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
                    .then_some(0u64)
                    .filter(|_| check_vlist_window_shifts_prefetch_max.is_none());
            let suite_vlist_small_scroll_window_shifts_escape_max =
                is_ui_gallery_vlist_no_window_shifts_small_scroll_suite
                    .then_some(0u64)
                    .filter(|_| check_vlist_window_shifts_escape_max.is_none());
            let suite_vlist_policy_key_stable = components_gallery_suite
                && script_requires_retained_vlist_reconcile_gate
                && !check_vlist_policy_key_stable;
            let suite_windowed_rows_offset_changes_min =
                diag_policy::ui_gallery_script_requires_windowed_rows_offset_changes_gate(&src)
                    .then_some(1u64)
                    .filter(|_| check_windowed_rows_offset_changes_min.is_none());
            let suite_windowed_rows_visible_start_changes_repainted =
                diag_policy::ui_gallery_script_requires_windowed_rows_visible_start_repaint_gate(
                    &src,
                ) && !check_windowed_rows_visible_start_changes_repainted;
            let suite_pixels_changed_test_id =
                diag_policy::ui_gallery_script_pixels_changed_test_id(&src).filter(|_| {
                    check_pixels_changed_test_id.is_none()
                        && check_pixels_unchanged_test_id.is_none()
                });
            let suite_ui_gallery_code_editor_torture_marker_present =
                diag_policy::ui_gallery_script_requires_code_editor_torture_marker_present_gate(
                    &src,
                ) && !check_ui_gallery_code_editor_torture_marker_present;
            let suite_ui_gallery_code_editor_torture_undo_redo =
                diag_policy::ui_gallery_script_requires_code_editor_torture_undo_redo_gate(&src)
                    && !check_ui_gallery_code_editor_torture_undo_redo;
            let suite_ui_gallery_code_editor_torture_geom_fallbacks_low =
                diag_policy::ui_gallery_script_requires_code_editor_torture_geom_fallbacks_low_gate(
                    &src,
                ) && !check_ui_gallery_code_editor_torture_geom_fallbacks_low;
            let suite_ui_gallery_code_editor_torture_read_only_blocks_edits =
                diag_policy::ui_gallery_script_requires_code_editor_torture_read_only_blocks_edits_gate(&src)
                    && !check_ui_gallery_code_editor_torture_read_only_blocks_edits;
            let suite_ui_gallery_markdown_editor_source_read_only_blocks_edits =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_read_only_blocks_edits_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_read_only_blocks_edits;
            let suite_ui_gallery_markdown_editor_source_disabled_blocks_edits =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_disabled_blocks_edits_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_disabled_blocks_edits;
            let suite_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_soft_wrap_toggle_stable_gate(
                    &src,
                ) && !check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable;
            let suite_ui_gallery_markdown_editor_source_word_boundary =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_word_boundary_gate(
                    &src,
                ) && !check_ui_gallery_markdown_editor_source_word_boundary;
            let suite_ui_gallery_web_ime_bridge_enabled =
                diag_policy::ui_gallery_script_requires_web_ime_bridge_enabled_gate(&src)
                    && !check_ui_gallery_web_ime_bridge_enabled;
            let suite_ui_gallery_markdown_editor_source_line_boundary_triple_click =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_line_boundary_triple_click_gate(
                    &src,
                ) && !check_ui_gallery_markdown_editor_source_line_boundary_triple_click;
            let suite_ui_gallery_markdown_editor_source_a11y_composition =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_a11y_composition_gate(
                    &src,
                ) && !check_ui_gallery_markdown_editor_source_a11y_composition;
            let suite_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_a11y_composition_soft_wrap_gate(
                    &src,
                ) && !check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap;
            let suite_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable;
            let suite_ui_gallery_markdown_editor_source_folds_toggle_stable =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_folds_toggle_stable_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_folds_toggle_stable;
            let suite_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_folds_clamp_selection_out_of_folds_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds;
            let suite_ui_gallery_markdown_editor_source_folds_placeholder_present =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_gate(
                    &src,
                ) && !check_ui_gallery_markdown_editor_source_folds_placeholder_present;
            let suite_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_under_soft_wrap_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap;
            let suite_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit;
            let suite_ui_gallery_markdown_editor_source_inlays_toggle_stable =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_toggle_stable_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_inlays_toggle_stable;
            let suite_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_caret_navigation_stable_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable;
            let suite_ui_gallery_markdown_editor_source_inlays_present =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_present_gate(
                    &src,
                ) && !check_ui_gallery_markdown_editor_source_inlays_present;
            let suite_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_present_under_soft_wrap_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap;
            let suite_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit =
                diag_policy::ui_gallery_script_requires_markdown_editor_source_inlays_absent_under_inline_preedit_gate(&src)
                    && !check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit;
            let suite_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit =
                diag_policy::ui_gallery_script_requires_code_editor_torture_folds_placeholder_absent_under_inline_preedit_gate(&src)
                    && !check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit;
            let suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped =
                diag_policy::ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_gate(&src)
                    && !check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped;
            let suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations =
                diag_policy::ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_gate(&src)
                    && !check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations;
            let suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed =
                diag_policy::ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_gate(&src)
                    && !check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed;
            let suite_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed =
                diag_policy::ui_gallery_script_requires_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_gate(&src)
                    && !check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed;
            let suite_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed =
                diag_policy::ui_gallery_script_requires_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_gate(&src)
                    && !check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed;
            let suite_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll =
                diag_policy::ui_gallery_script_requires_code_editor_torture_composed_preedit_stable_after_wheel_scroll_gate(&src)
                    && !check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll;
            let suite_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection =
                diag_policy::ui_gallery_script_requires_code_editor_torture_composed_preedit_cancels_on_drag_selection_gate(&src)
                    && !check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection;
            let suite_ui_gallery_code_editor_torture_folds_placeholder_present =
                diag_policy::ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_gate(&src)
                    && !check_ui_gallery_code_editor_torture_folds_placeholder_present;
            let suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap =
                diag_policy::ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_soft_wrap_gate(&src)
                    && !check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap;
            let suite_ui_gallery_code_editor_torture_inlays_present =
                diag_policy::ui_gallery_script_requires_code_editor_torture_inlays_present_gate(
                    &src,
                ) && !check_ui_gallery_code_editor_torture_inlays_present;
            let suite_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit =
                diag_policy::ui_gallery_script_requires_code_editor_torture_inlays_absent_under_inline_preedit_gate(&src)
                    && !check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit;
            let suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped =
                diag_policy::ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_gate(&src)
                    && !check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped;
            let suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations =
                diag_policy::ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_gate(&src)
                    && !check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations;
            let suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed =
                diag_policy::ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_gate(&src)
                    && !check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed;
            let suite_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap =
                diag_policy::ui_gallery_script_requires_code_editor_torture_inlays_present_under_soft_wrap_gate(
                    &src,
                ) && !check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap;
            let suite_ui_gallery_code_editor_word_boundary =
                diag_policy::ui_gallery_script_requires_code_editor_word_boundary_gate(&src)
                    && !check_ui_gallery_code_editor_word_boundary;
            let suite_ui_gallery_code_editor_a11y_selection =
                diag_policy::ui_gallery_script_requires_code_editor_a11y_selection_gate(&src)
                    && !check_ui_gallery_code_editor_a11y_selection;
            let suite_ui_gallery_code_editor_a11y_composition =
                diag_policy::ui_gallery_script_requires_code_editor_a11y_composition_gate(&src)
                    && !check_ui_gallery_code_editor_a11y_composition;
            let suite_ui_gallery_code_editor_a11y_selection_wrap =
                diag_policy::ui_gallery_script_requires_code_editor_a11y_selection_wrap_gate(&src)
                    && !check_ui_gallery_code_editor_a11y_selection_wrap;
            let suite_ui_gallery_code_editor_a11y_composition_wrap =
                diag_policy::ui_gallery_script_requires_code_editor_a11y_composition_wrap_gate(
                    &src,
                ) && !check_ui_gallery_code_editor_a11y_composition_wrap;
            let suite_ui_gallery_code_editor_a11y_composition_wrap_scroll =
                diag_policy::ui_gallery_script_requires_code_editor_a11y_composition_wrap_scroll_gate(&src)
                    && !check_ui_gallery_code_editor_a11y_composition_wrap_scroll;
            let suite_ui_gallery_code_editor_a11y_composition_drag =
                diag_policy::ui_gallery_script_requires_code_editor_a11y_composition_drag_gate(
                    &src,
                ) && !check_ui_gallery_code_editor_a11y_composition_drag;
            let suite_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps =
                diag_policy::ui_gallery_script_requires_text_rescan_system_fonts_font_stack_key_bumps_gate(&src)
                    && !check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps;
            let suite_ui_gallery_text_fallback_policy_key_bumps_on_settings_change =
                diag_policy::ui_gallery_script_requires_text_fallback_policy_key_bumps_on_settings_change_gate(
                    &src,
                ) && !check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change;
            let suite_ui_gallery_text_fallback_policy_key_bumps_on_locale_change =
                diag_policy::ui_gallery_script_requires_text_fallback_policy_key_bumps_on_locale_change_gate(
                    &src,
                ) && !check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change;
            let suite_ui_gallery_text_mixed_script_bundled_fallback_conformance =
                diag_policy::ui_gallery_script_requires_text_mixed_script_bundled_fallback_conformance_gate(
                    &src,
                ) && !check_ui_gallery_text_mixed_script_bundled_fallback_conformance;
            let script_requires_retained_vlist_keep_alive_reuse_gate =
                diag_policy::ui_gallery_script_requires_retained_vlist_keep_alive_reuse_gate(&src);
            let retained_vlist_suite = components_gallery_suite
                || ai_transcript_suite
                || vlist_window_boundary_retained_suite;
            let suite_retained_vlist_reconcile_no_notify_min = (retained_vlist_suite
                && script_requires_retained_vlist_reconcile_gate)
                .then_some(1u64)
                .filter(|_| check_retained_vlist_reconcile_no_notify_min.is_none());
            let suite_retained_vlist_attach_detach_max = (retained_vlist_suite
                && script_requires_retained_vlist_reconcile_gate)
                .then_some(if vlist_window_boundary_retained_suite {
                    64u64
                } else {
                    256u64
                })
                .filter(|_| check_retained_vlist_attach_detach_max.is_none());
            let suite_retained_vlist_keep_alive_reuse_min = ((components_gallery_suite
                && script_requires_retained_vlist_keep_alive_reuse_gate)
                || vlist_window_boundary_retained_suite)
                .then_some(if vlist_window_boundary_retained_suite {
                    5u64
                } else {
                    1u64
                })
                .filter(|_| check_retained_vlist_keep_alive_reuse_min.is_none());
            let suite_retained_vlist_keep_alive_budget = ((components_gallery_suite
                && script_requires_retained_vlist_keep_alive_reuse_gate)
                || vlist_window_boundary_retained_suite)
                .then_some((1u64, 0u64))
                .filter(|_| check_retained_vlist_keep_alive_budget.is_none());
            let suite_gc_sweep_liveness =
                builtin_suite == Some(BuiltinSuite::UiGallery) && is_gc_liveness_script;

            let mut notify_hotspot_file_max_for_script = check_notify_hotspot_file_max.clone();
            if notify_hotspot_file_max_for_script.is_empty()
                && builtin_suite == Some(BuiltinSuite::UiGallery)
                && src
                    .file_name()
                    .and_then(|v| v.to_str())
                    .is_some_and(|v| v == "ui-gallery-virtual-list-torture.json")
            {
                notify_hotspot_file_max_for_script.push((
                    "crates/fret-ui/src/declarative/host_widget/event/pressable.rs".to_string(),
                    0,
                ));
            }
            let mut checks_for_post_run = checks_for_post_run_template.clone();

            if checks_for_post_run.check_stale_paint_test_id.is_none() {
                checks_for_post_run.check_stale_paint_test_id = suite_stale_paint_test_id
                    .or(suite_components_gallery_stale_paint_test_id)
                    .map(|s| s.to_string());
            }
            if checks_for_post_run.check_pixels_changed_test_id.is_none() {
                checks_for_post_run.check_pixels_changed_test_id = suite_pixels_changed_test_id
                    .or(suite_default_pixels_changed_test_id)
                    .map(|s| s.to_string());
            }

            checks_for_post_run.check_ui_gallery_code_editor_torture_marker_present |=
                suite_ui_gallery_code_editor_torture_marker_present;
            checks_for_post_run.check_ui_gallery_code_editor_torture_undo_redo |=
                suite_ui_gallery_code_editor_torture_undo_redo;
            checks_for_post_run.check_ui_gallery_code_editor_torture_geom_fallbacks_low |=
                suite_ui_gallery_code_editor_torture_geom_fallbacks_low;
            checks_for_post_run.check_ui_gallery_code_editor_torture_read_only_blocks_edits |=
                suite_ui_gallery_code_editor_torture_read_only_blocks_edits;

            checks_for_post_run.check_ui_gallery_markdown_editor_source_read_only_blocks_edits |=
                suite_ui_gallery_markdown_editor_source_read_only_blocks_edits;
            checks_for_post_run.check_ui_gallery_markdown_editor_source_disabled_blocks_edits |=
                suite_ui_gallery_markdown_editor_source_disabled_blocks_edits;
            checks_for_post_run.check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable |=
                suite_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable;
            checks_for_post_run.check_ui_gallery_markdown_editor_source_word_boundary |=
                suite_ui_gallery_markdown_editor_source_word_boundary;
            checks_for_post_run.check_ui_gallery_web_ime_bridge_enabled |=
                suite_ui_gallery_web_ime_bridge_enabled;

            checks_for_post_run.check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps |=
                suite_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps;
            checks_for_post_run
                .check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change |=
                suite_ui_gallery_text_fallback_policy_key_bumps_on_settings_change;
            checks_for_post_run.check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change |=
                suite_ui_gallery_text_fallback_policy_key_bumps_on_locale_change;
            checks_for_post_run.check_ui_gallery_text_mixed_script_bundled_fallback_conformance |=
                suite_ui_gallery_text_mixed_script_bundled_fallback_conformance;

            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_line_boundary_triple_click |=
                suite_ui_gallery_markdown_editor_source_line_boundary_triple_click;
            checks_for_post_run.check_ui_gallery_markdown_editor_source_a11y_composition |=
                suite_ui_gallery_markdown_editor_source_a11y_composition;
            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap |=
                suite_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap;
            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable |=
                suite_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable;
            checks_for_post_run.check_ui_gallery_markdown_editor_source_folds_toggle_stable |=
                suite_ui_gallery_markdown_editor_source_folds_toggle_stable;
            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds |=
                suite_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds;
            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_folds_placeholder_present |=
                suite_ui_gallery_markdown_editor_source_folds_placeholder_present;
            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap |=
                suite_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap;
            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit |=
                suite_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit;
            checks_for_post_run.check_ui_gallery_markdown_editor_source_inlays_toggle_stable |=
                suite_ui_gallery_markdown_editor_source_inlays_toggle_stable;
            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable |=
                suite_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable;
            checks_for_post_run.check_ui_gallery_markdown_editor_source_inlays_present |=
                suite_ui_gallery_markdown_editor_source_inlays_present;
            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap |=
                suite_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap;
            checks_for_post_run
                .check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit |=
                suite_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit;

            checks_for_post_run
                .check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit |=
                suite_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped |=
                suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations |=
                suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed |=
                suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed |=
                suite_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed |=
                suite_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll |=
                suite_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection |=
                suite_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection;
            checks_for_post_run.check_ui_gallery_code_editor_torture_folds_placeholder_present |=
                suite_ui_gallery_code_editor_torture_folds_placeholder_present;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap |=
                suite_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap;
            checks_for_post_run.check_ui_gallery_code_editor_torture_inlays_present |=
                suite_ui_gallery_code_editor_torture_inlays_present;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit |=
                suite_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped |=
                suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations |=
                suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed |=
                suite_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed;
            checks_for_post_run
                .check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap |=
                suite_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap;

            checks_for_post_run.check_ui_gallery_code_editor_word_boundary |=
                suite_ui_gallery_code_editor_word_boundary;
            checks_for_post_run.check_ui_gallery_code_editor_a11y_selection |=
                suite_ui_gallery_code_editor_a11y_selection;
            checks_for_post_run.check_ui_gallery_code_editor_a11y_composition |=
                suite_ui_gallery_code_editor_a11y_composition;
            checks_for_post_run.check_ui_gallery_code_editor_a11y_selection_wrap |=
                suite_ui_gallery_code_editor_a11y_selection_wrap;
            checks_for_post_run.check_ui_gallery_code_editor_a11y_composition_wrap |=
                suite_ui_gallery_code_editor_a11y_composition_wrap;
            checks_for_post_run.check_ui_gallery_code_editor_a11y_composition_wrap_scroll |=
                suite_ui_gallery_code_editor_a11y_composition_wrap_scroll;
            checks_for_post_run.check_ui_gallery_code_editor_a11y_composition_drag |=
                suite_ui_gallery_code_editor_a11y_composition_drag;

            if checks_for_post_run.check_wheel_scroll_test_id.is_none() {
                checks_for_post_run.check_wheel_scroll_test_id =
                    suite_wheel_scroll_test_id.map(|s| s.to_string());
            }
            if checks_for_post_run
                .check_wheel_scroll_hit_changes_test_id
                .is_none()
            {
                checks_for_post_run.check_wheel_scroll_hit_changes_test_id =
                    suite_wheel_scroll_hit_changes_test_id.map(|s| s.to_string());
            }

            checks_for_post_run.check_prepaint_actions_min = checks_for_post_run
                .check_prepaint_actions_min
                .or(suite_prepaint_actions_min);
            checks_for_post_run.check_chart_sampling_window_shifts_min = checks_for_post_run
                .check_chart_sampling_window_shifts_min
                .or(suite_chart_sampling_window_shifts_min);
            checks_for_post_run.check_node_graph_cull_window_shifts_min = checks_for_post_run
                .check_node_graph_cull_window_shifts_min
                .or(suite_node_graph_cull_window_shifts_min);
            checks_for_post_run.check_node_graph_cull_window_shifts_max = checks_for_post_run
                .check_node_graph_cull_window_shifts_max
                .or(suite_node_graph_cull_window_shifts_max);
            checks_for_post_run.check_vlist_visible_range_refreshes_min = checks_for_post_run
                .check_vlist_visible_range_refreshes_min
                .or(suite_vlist_visible_range_refreshes_min);
            checks_for_post_run.check_vlist_visible_range_refreshes_max = checks_for_post_run
                .check_vlist_visible_range_refreshes_max
                .or(suite_vlist_visible_range_refreshes_max);

            checks_for_post_run.check_vlist_window_shifts_explainable |=
                suite_vlist_window_shifts_explainable;
            checks_for_post_run.check_vlist_window_shifts_have_prepaint_actions |=
                suite_vlist_window_shifts_have_prepaint_actions;
            checks_for_post_run.check_vlist_window_shifts_non_retained_max =
                vlist_window_shifts_non_retained_max_for_script
                    .or(suite_vlist_window_shifts_non_retained_max)
                    .or(suite_vlist_small_scroll_window_shifts_non_retained_max);
            checks_for_post_run.check_vlist_window_shifts_prefetch_max = checks_for_post_run
                .check_vlist_window_shifts_prefetch_max
                .or(suite_vlist_window_shifts_prefetch_max)
                .or(suite_vlist_small_scroll_window_shifts_prefetch_max);
            checks_for_post_run.check_vlist_window_shifts_escape_max = checks_for_post_run
                .check_vlist_window_shifts_escape_max
                .or(suite_vlist_window_shifts_escape_max)
                .or(suite_vlist_small_scroll_window_shifts_escape_max);
            checks_for_post_run.check_vlist_policy_key_stable |= suite_vlist_policy_key_stable;

            checks_for_post_run.check_windowed_rows_offset_changes_min = checks_for_post_run
                .check_windowed_rows_offset_changes_min
                .or(suite_windowed_rows_offset_changes_min);
            checks_for_post_run.check_windowed_rows_visible_start_changes_repainted |=
                suite_windowed_rows_visible_start_changes_repainted;
            checks_for_post_run.check_layout_fast_path_min = checks_for_post_run
                .check_layout_fast_path_min
                .or(suite_layout_fast_path_min);
            checks_for_post_run.check_hover_layout_max = checks_for_post_run
                .check_hover_layout_max
                .or(suite_hover_layout_max);
            checks_for_post_run.check_gc_sweep_liveness |= suite_gc_sweep_liveness;

            checks_for_post_run.check_notify_hotspot_file_max = notify_hotspot_file_max_for_script;
            checks_for_post_run.check_view_cache_reuse_stable_min = checks_for_post_run
                .check_view_cache_reuse_stable_min
                .or(suite_view_cache_reuse_stable_min);
            checks_for_post_run.check_view_cache_reuse_min = checks_for_post_run
                .check_view_cache_reuse_min
                .or(suite_view_cache_reuse_min)
                .or(suite_components_gallery_view_cache_reuse_min);

            checks_for_post_run.check_viewport_input_min = checks_for_post_run
                .check_viewport_input_min
                .or(suite_viewport_input_min);
            checks_for_post_run.check_dock_drag_min = checks_for_post_run
                .check_dock_drag_min
                .or(suite_dock_drag_min);
            checks_for_post_run.check_viewport_capture_min = checks_for_post_run
                .check_viewport_capture_min
                .or(suite_viewport_capture_min);

            checks_for_post_run.check_retained_vlist_reconcile_no_notify_min =
                retained_vlist_gate_for_script.or(suite_retained_vlist_reconcile_no_notify_min);
            checks_for_post_run.check_retained_vlist_attach_detach_max =
                retained_vlist_attach_detach_max_for_script
                    .or(suite_retained_vlist_attach_detach_max);
            checks_for_post_run.check_retained_vlist_keep_alive_reuse_min =
                retained_vlist_keep_alive_reuse_min_for_script
                    .or(suite_retained_vlist_keep_alive_reuse_min);
            checks_for_post_run.check_retained_vlist_keep_alive_budget =
                retained_vlist_keep_alive_budget_for_script
                    .or(suite_retained_vlist_keep_alive_budget);

            apply_post_run_checks(
                &bundle_path,
                &resolved_out_dir,
                &checks_for_post_run,
                warmup_frames,
            )?;
        }

        suite_rows.push(serde_json::json!({
            "script": script_key,
            "run_id": result.run_id,
            "stage": result.stage,
            "step_index": result.step_index,
            "reason_code": result.reason_code,
            "reason": result.reason,
            "last_bundle_dir": result.last_bundle_dir,
            "lint": lint_summary,
            "evidence_highlights": evidence_highlights,
        }));

        if !reuse_process {
            let is_last = idx.saturating_add(1) >= script_count;
            if !(keep_open && is_last) {
                stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
            }
        }
    }

    if !keep_open {
        stop_launched_demo(&mut child, &resolved_exit_path, poll_ms);
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": suite_summary_generated_unix_ms,
        "kind": "suite_summary",
        "status": "passed",
        "suite": suite_summary_suite,
        "out_dir": resolved_out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "reuse_launch": reuse_launch,
        "wants_screenshots": suite_wants_screenshots,
        "stage_counts": suite_stage_counts,
        "reason_code_counts": suite_reason_code_counts,
        "evidence_aggregate": suite_evidence_agg.as_json(),
        "rows": suite_rows,
    });
    let _ = write_json_value(&suite_summary_path, &payload);
    if !stats_json {
        println!("SUITE-SUMMARY {}", suite_summary_path.display());
    }
    std::process::exit(0);
}
