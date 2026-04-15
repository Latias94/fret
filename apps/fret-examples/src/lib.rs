#![allow(dead_code)]
#![allow(unused)]

#[cfg(not(target_arch = "wasm32"))]
pub mod alloc_profile;

pub(crate) mod effect_authoring;
pub(crate) mod hotpatch;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn run_native_with_compat_driver<D: fret_launch::WinitAppDriver + 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: fret_app::App,
    driver: D,
) -> anyhow::Result<()> {
    fret::advanced::interop::run_native_with_compat_driver(config, app, driver)
        .map_err(anyhow::Error::from)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn run_native_with_fn_driver_with_hooks<D: 'static, S: 'static>(
    config: fret_launch::WinitRunnerConfig,
    app: fret_app::App,
    driver_state: D,
    create_window_state: fn(&mut D, &mut fret_app::App, fret_core::AppWindowId) -> S,
    handle_event: for<'d, 'cx, 'e> fn(
        &'d mut D,
        fret_launch::WinitEventContext<'cx, S>,
        &'e fret_core::Event,
    ),
    render: for<'d, 'cx> fn(&'d mut D, fret_launch::WinitRenderContext<'cx, S>),
    configure_hooks: impl FnOnce(&mut fret_launch::FnDriverHooks<D, S>),
) -> anyhow::Result<()> {
    fret::advanced::run_native_with_fn_driver_with_hooks(
        config,
        app,
        driver_state,
        create_window_state,
        handle_event,
        render,
        configure_hooks,
    )
    .map_err(anyhow::Error::from)
}

pub(crate) fn parse_editor_theme_preset_key(
    key: &str,
) -> Option<fret_ui_editor::theme::EditorThemePresetV1> {
    match key.trim().to_ascii_lowercase().as_str() {
        "" => None,
        "default" => Some(fret_ui_editor::theme::EditorThemePresetV1::Default),
        "imgui_like_dense" => Some(fret_ui_editor::theme::EditorThemePresetV1::ImguiLikeDense),
        _ => None,
    }
}

pub(crate) fn editor_theme_preset_from_env(
    name: &str,
) -> Option<fret_ui_editor::theme::EditorThemePresetV1> {
    let raw = std::env::var_os(name)?;
    parse_editor_theme_preset_key(&raw.to_string_lossy())
}

/// Shared lower-level examples helper for editor surfaces hosted on a shadcn base theme.
///
/// Use this on manual/non-`FretApp` surfaces and on app code that keeps editor theming on the
/// owning `fret-ui-editor` crate. The ordering stays explicit when `WindowMetricsService` changes
/// can trigger a host-theme reset: sync the host theme first, then replay the installed editor
/// preset.
pub(crate) fn sync_shadcn_host_theme_then_reapply_editor_preset_on_window_metrics_change(
    app: &mut fret_app::App,
    window: fret_core::AppWindowId,
    changed: &[std::any::TypeId],
    base_color: fret::shadcn::themes::ShadcnBaseColor,
    default_scheme_when_unknown: fret::shadcn::themes::ShadcnColorScheme,
) -> Option<fret_ui_editor::theme::EditorThemePresetV1> {
    fret_ui_editor::theme::sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change(
        app,
        changed,
        |app| {
            let _ = fret::shadcn::raw::advanced::sync_theme_from_environment(
                app,
                window,
                base_color,
                default_scheme_when_unknown,
            );
        },
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub mod alpha_mode_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod api_workbench_lite_demo;
pub mod area_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod assets_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod async_playground_demo;
pub mod bars_demo;
pub mod candlestick_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod canvas_datagrid_stress_demo;
pub mod category_line_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod chart_declarative_demo;
pub mod chart_demo;
pub mod chart_multi_axis_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod chart_stress_demo;
pub mod cjk_conformance_demo;
pub mod components_gallery;
#[cfg(not(target_arch = "wasm32"))]
pub mod container_queries_docking_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod custom_effect_v1_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod custom_effect_v2_demo;
#[cfg(target_arch = "wasm32")]
pub mod custom_effect_v2_glass_chrome_web_demo;
#[cfg(target_arch = "wasm32")]
pub mod custom_effect_v2_identity_web_demo;
#[cfg(target_arch = "wasm32")]
pub mod custom_effect_v2_lut_web_demo;
#[cfg(target_arch = "wasm32")]
pub mod custom_effect_v2_web_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod custom_effect_v3_demo;
#[cfg(target_arch = "wasm32")]
pub mod custom_effect_v3_web_demo;
pub mod custom_effect_v3_wgsl;
#[cfg(not(target_arch = "wasm32"))]
pub mod datatable_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod date_picker_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod docking_arbitration_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod docking_demo;
pub mod drag_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod drop_shadow_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod echarts_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod echarts_multi_grid_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod editor_notes_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod editor_notes_device_shell_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod effects_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod embedded_viewport_demo;
pub mod emoji_conformance_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod empty_idle_demo;
pub mod error_bars_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod external_texture_imports_demo;
#[cfg(target_arch = "wasm32")]
pub mod external_texture_imports_web_demo;
#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
pub mod external_video_imports_avf_demo;
#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub mod external_video_imports_mf_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod extras_marquee_perf_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod first_frame_smoke_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod form_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod genui_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod gizmo3d_demo;
pub mod grouped_bars_demo;
pub mod heatmap_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod hello_counter_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod hello_world_compare_demo;
pub mod histogram2d_demo;
pub mod histogram_demo;
pub mod horizontal_bars_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod image_heavy_memory_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod image_upload_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod ime_smoke_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod imui_editor_proof_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod imui_floating_windows_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod imui_hello_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod imui_interaction_showcase_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos-legacy"))]
pub mod imui_node_graph_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod imui_response_signals_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod imui_shadcn_adapter_demo;
pub mod inf_lines_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod launcher_utility_window_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod launcher_utility_window_materials_demo;
pub mod linked_cursor_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod liquid_glass_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod markdown_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos"))]
pub mod node_graph_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos-legacy"))]
pub mod node_graph_domain_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos-legacy"))]
pub mod node_graph_legacy_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos-legacy"))]
pub(crate) mod node_graph_tuning_overlay;
#[cfg(not(target_arch = "wasm32"))]
pub mod plot3d_demo;
pub mod plot_demo;
pub mod plot_image_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod plot_stress_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod postprocess_theme_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod query_async_tokio_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod query_demo;
pub mod shaded_demo;
pub mod simple_todo_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod sonner_demo;
pub mod stacked_bars_demo;
pub mod stairs_demo;
pub mod stems_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod streaming_i420_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod streaming_image_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod streaming_nv12_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod table_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod table_stress_demo;
pub mod tags_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod text_heavy_memory_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod todo_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod virtual_list_stress_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod window_hit_test_probe_demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod workspace_shell_demo;

#[cfg(test)]
mod authoring_surface_policy_tests {
    use std::path::{Path, PathBuf};

    const ASSETS_DEMO: &str = include_str!("assets_demo.rs");
    const ASYNC_PLAYGROUND_DEMO: &str = include_str!("async_playground_demo.rs");
    const API_WORKBENCH_LITE_DEMO: &str = include_str!("api_workbench_lite_demo.rs");
    const CANVAS_DATAGRID_STRESS_DEMO: &str = include_str!("canvas_datagrid_stress_demo.rs");
    const CJK_CONFORMANCE_DEMO: &str = include_str!("cjk_conformance_demo.rs");
    const CHART_DECLARATIVE_DEMO: &str = include_str!("chart_declarative_demo.rs");
    const COMPONENTS_GALLERY_DEMO: &str = include_str!("components_gallery.rs");
    const CUSTOM_EFFECT_V1_DEMO: &str = include_str!("custom_effect_v1_demo.rs");
    const CUSTOM_EFFECT_V2_DEMO: &str = include_str!("custom_effect_v2_demo.rs");
    const CUSTOM_EFFECT_V2_GLASS_CHROME_WEB_DEMO: &str =
        include_str!("custom_effect_v2_glass_chrome_web_demo.rs");
    const CUSTOM_EFFECT_V2_IDENTITY_WEB_DEMO: &str =
        include_str!("custom_effect_v2_identity_web_demo.rs");
    const CUSTOM_EFFECT_V2_LUT_WEB_DEMO: &str = include_str!("custom_effect_v2_lut_web_demo.rs");
    const CUSTOM_EFFECT_V2_WEB_DEMO: &str = include_str!("custom_effect_v2_web_demo.rs");
    const CONTAINER_QUERIES_DOCKING_DEMO: &str = include_str!("container_queries_docking_demo.rs");
    const CUSTOM_EFFECT_V3_DEMO: &str = include_str!("custom_effect_v3_demo.rs");
    const DATATABLE_DEMO: &str = include_str!("datatable_demo.rs");
    const DATE_PICKER_DEMO: &str = include_str!("date_picker_demo.rs");
    const DOCKING_ARBITRATION_DEMO: &str = include_str!("docking_arbitration_demo.rs");
    const DOCKING_DEMO: &str = include_str!("docking_demo.rs");
    const DROP_SHADOW_DEMO: &str = include_str!("drop_shadow_demo.rs");
    const ECHARTS_DEMO: &str = include_str!("echarts_demo.rs");
    const EMBEDDED_VIEWPORT_DEMO: &str = include_str!("embedded_viewport_demo.rs");
    const EDITOR_NOTES_DEMO: &str = include_str!("editor_notes_demo.rs");
    const EMPTY_IDLE_DEMO: &str = include_str!("empty_idle_demo.rs");
    const EMOJI_CONFORMANCE_DEMO: &str = include_str!("emoji_conformance_demo.rs");
    const EXAMPLES_DOCS_README: &str = include_str!("../../../docs/examples/README.md");
    const EXTERNAL_TEXTURE_IMPORTS_DEMO: &str = include_str!("external_texture_imports_demo.rs");
    const EXTERNAL_TEXTURE_IMPORTS_WEB_DEMO: &str =
        include_str!("external_texture_imports_web_demo.rs");
    const EXTERNAL_VIDEO_IMPORTS_AVF_DEMO: &str =
        include_str!("external_video_imports_avf_demo.rs");
    const EXTERNAL_VIDEO_IMPORTS_MF_DEMO: &str = include_str!("external_video_imports_mf_demo.rs");
    const EXTRAS_MARQUEE_PERF_DEMO: &str = include_str!("extras_marquee_perf_demo.rs");
    const FORM_DEMO: &str = include_str!("form_demo.rs");
    const GENUI_DEMO: &str = include_str!("genui_demo.rs");
    const HELLO_COUNTER_DEMO: &str = include_str!("hello_counter_demo.rs");
    const HELLO_WORLD_COMPARE_DEMO: &str = include_str!("hello_world_compare_demo.rs");
    const IMAGE_HEAVY_MEMORY_DEMO: &str = include_str!("image_heavy_memory_demo.rs");
    const IMUI_EDITOR_PROOF_DEMO: &str = include_str!("imui_editor_proof_demo.rs");
    const IMUI_FLOATING_WINDOWS_DEMO: &str = include_str!("imui_floating_windows_demo.rs");
    const IMUI_HELLO_DEMO: &str = include_str!("imui_hello_demo.rs");
    const IMUI_INTERACTION_SHOWCASE_DEMO: &str = include_str!("imui_interaction_showcase_demo.rs");
    const IMUI_NODE_GRAPH_DEMO: &str = include_str!("imui_node_graph_demo.rs");
    const IMUI_PROOF_BUDGET_RULE_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md"
    );
    const IMUI_ROOT_HOSTING_RULE_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P0_ROOT_HOSTING_RULE_2026-04-12.md"
    );
    const IMUI_STABLE_IDENTITY_RULE_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P0_STABLE_IDENTITY_RULE_2026-04-12.md"
    );
    const IMUI_RESPONSE_STATUS_LIFECYCLE_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md");
    const IMUI_RESPONSE_STATUS_LIFECYCLE_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json");
    const IMUI_MENU_TAB_TRIGGER_RESPONSE_SURFACE_FINAL_STATUS: &str = include_str!(
        "../../../docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md"
    );
    const IMUI_MENU_TAB_TRIGGER_RESPONSE_SURFACE_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-menu-tab-trigger-response-surface-v1/WORKSTREAM.json"
    );
    const IMUI_MENU_TAB_TRIGGER_RESPONSE_CANONICALIZATION_DESIGN: &str = include_str!(
        "../../../docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/DESIGN.md"
    );
    const IMUI_MENU_TAB_TRIGGER_RESPONSE_CANONICALIZATION_FINAL_STATUS: &str = include_str!(
        "../../../docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md"
    );
    const IMUI_MENU_TAB_TRIGGER_RESPONSE_CANONICALIZATION_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/WORKSTREAM.json"
    );
    const IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO: &str =
        include_str!("../../../docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md");
    const IMUI_RESPONSE_STATUS_LIFECYCLE_TODO: &str =
        include_str!("../../../docs/workstreams/imui-response-status-lifecycle-v1/TODO.md");
    const IMUI_WORKBENCH_PROOF_MATRIX_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md"
    );
    const IMUI_P1_SHELL_DIAG_SMOKE_DECISION_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md"
    );
    const IMUI_WORKBENCH_ASSEMBLY_DECISION_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-workbench-shell-closure-v1/M1_DEFAULT_WORKBENCH_ASSEMBLY_DECISION_2026-04-13.md"
    );
    const IMUI_P2_FIRST_OPEN_DIAGNOSTICS_PATH_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md"
    );
    const IMUI_P2_DIAGNOSTICS_OWNER_SPLIT_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md"
    );
    const IMUI_P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md"
    );
    const IMUI_P2_DISCOVERABILITY_ENTRY_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P2_DISCOVERABILITY_ENTRY_2026-04-12.md"
    );
    const IMUI_P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md"
    );
    const IMUI_P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json");
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_BASELINE_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M1_MIXED_DPI_ACCEPTANCE_POSTURE_2026-04-13.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_CAPTURE_PLAN_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md"
    );
    const DIAGNOSTICS_FIRST_OPEN_DOC: &str =
        include_str!("../../../docs/diagnostics-first-open.md");
    const DIAGNOSTICS_START_HERE_DOC: &str =
        include_str!("../../../docs/workstreams/diag-fearless-refactor-v2/START_HERE.md");
    const DIAGNOSTICS_GUI_DOGFOOD_DOC: &str = include_str!(
        "../../../docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_DOC: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md"
    );
    const MACOS_DOCKING_MULTIWINDOW_IMGUI_PARITY_DOC: &str = include_str!(
        "../../../docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md"
    );
    const IMUI_P3_MULTIWINDOW_PARITY_CAMPAIGN: &str =
        include_str!("../../../tools/diag-campaigns/imui-p3-multiwindow-parity.json");
    const IMUI_P2_DEVTOOLS_SMOKE_GATE_SCRIPT: &str =
        include_str!("../../../tools/diag_gate_imui_p2_devtools_first_open.py");
    const IMUI_P2_DEVTOOLS_SMOKE_CAMPAIGN: &str =
        include_str!("../../../tools/diag-campaigns/devtools-first-open-smoke.json");
    const IMUI_RESPONSE_SIGNALS_DEMO: &str = include_str!("imui_response_signals_demo.rs");
    const IMUI_SHADCN_ADAPTER_DEMO: &str = include_str!("imui_shadcn_adapter_demo.rs");
    const IME_SMOKE_DEMO: &str = include_str!("ime_smoke_demo.rs");
    const LAUNCHER_UTILITY_WINDOW_DEMO: &str = include_str!("launcher_utility_window_demo.rs");
    const LAUNCHER_UTILITY_WINDOW_MATERIALS_DEMO: &str =
        include_str!("launcher_utility_window_materials_demo.rs");
    const LIQUID_GLASS_DEMO: &str = include_str!("liquid_glass_demo.rs");
    const MARKDOWN_DEMO: &str = include_str!("markdown_demo.rs");
    const NODE_GRAPH_DEMO: &str = include_str!("node_graph_demo.rs");
    const POSTPROCESS_THEME_DEMO: &str = include_str!("postprocess_theme_demo.rs");
    const QUERY_ASYNC_TOKIO_DEMO: &str = include_str!("query_async_tokio_demo.rs");
    const QUERY_DEMO: &str = include_str!("query_demo.rs");
    const SIMPLE_TODO_DEMO: &str = include_str!("simple_todo_demo.rs");
    const SONNER_DEMO: &str = include_str!("sonner_demo.rs");
    const TABLE_DEMO: &str = include_str!("table_demo.rs");
    const TABLE_STRESS_DEMO: &str = include_str!("table_stress_demo.rs");
    const TEXT_HEAVY_MEMORY_DEMO: &str = include_str!("text_heavy_memory_demo.rs");
    const TODO_DEMO: &str = include_str!("todo_demo.rs");
    const WINDOW_HIT_TEST_PROBE_DEMO: &str = include_str!("window_hit_test_probe_demo.rs");
    const WORKSPACE_SHELL_DEMO: &str = include_str!("workspace_shell_demo.rs");
    const WORKSPACE_HARDENING_SHELL_DIAG_SUITE: &str = include_str!(
        "../../../tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json"
    );

    fn collect_rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_rust_sources(&path, out);
                continue;
            }

            if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }

    fn examples_rust_sources() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        collect_rust_sources(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut paths,
        );
        paths.sort();
        paths
    }

    fn assert_uses_advanced_surface(src: &str) {
        assert!(src.contains("advanced::prelude::*"));
        assert!(src.contains("KernelApp"));
        assert!(!src.contains("fret_bootstrap::ui_app("));
        assert!(!src.contains("fret_bootstrap::ui_app_with_hooks("));
        assert!(!src.contains("use fret::prelude::*;"));
        assert!(!src.contains("use fret::prelude::{"));
        assert!(!src.contains(".init_app("));
        assert!(!src.contains("ViewCx<'_, '_, App>"));
        assert!(!src.contains("ElementContext<'_, App>"));
        assert!(!src.contains("UiTree<App>"));
        assert!(!src.contains("RetainedSubtreeProps::new::<App>"));
        assert!(!src.contains("UiChildIntoElement<App>"));
        assert!(
            src.contains("AppUi<'_, '_>")
                || src.contains("ViewCx<'_, '_, KernelApp>")
                || src.contains("ElementContext<'_, KernelApp>")
                || src.contains("UiTree<KernelApp>")
                || src.contains("KernelApp::new()")
        );
    }

    fn assert_explicit_advanced_reference_classification(name: &str, src: &str, reasons: &[&str]) {
        assert!(
            src.contains("Advanced/reference demo:"),
            "{name} should advertise its advanced/reference classification"
        );
        assert!(
            src.contains("Why advanced:"),
            "{name} should explain why it stays on the advanced/reference lane"
        );
        assert!(
            src.contains("Not a first-contact teaching surface:"),
            "{name} should say it is not part of the first-contact teaching surface"
        );
        assert!(
            src.contains("reference/product-validation"),
            "{name} should describe itself as a reference/product-validation surface"
        );
        for reason in reasons {
            assert!(
                src.contains(reason),
                "{name} is missing the advanced/reference rationale marker: {reason}"
            );
        }
    }

    fn assert_uses_default_app_surface_with_page(src: &str, page_fn: &str, call_site: &str) {
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(!src.contains("advanced::prelude::*"));
        assert!(!src.contains("KernelApp"));
        assert!(!src.contains("AppWindowId"));
        assert!(
            src.contains("fn init(_app: &mut App, _window: WindowId) -> Self")
                || src.contains("fn init(app: &mut App, _window: WindowId) -> Self")
        );
        let page_sig =
            format!("fn {page_fn}(theme: ThemeSnapshot, content: impl UiChild) -> impl UiChild");
        let legacy_page_sig = format!("fn {page_fn}(cx: &mut UiCx<'_>,");
        assert!(src.contains(call_site));
        assert!(src.contains(&page_sig));
        assert!(!src.contains(&legacy_page_sig));
        assert!(!src.contains("let card = card.into_element(cx);"));
        assert!(!src.contains(&format!("{page_fn}(theme, card).into_element(cx).into()")));
    }

    fn assert_uses_default_app_surface(src: &str) {
        assert_uses_default_app_surface_with_page(
            src,
            "todo_page",
            "ui::single(cx, todo_page(theme, card))",
        );
    }

    fn assert_default_app_surface_prefers_local_state_first(src: &str) {
        assert!(src.contains("cx.state().local"));
        assert!(!src.contains("app.models_mut().insert("));
        assert!(!src.contains("Model<"));
        assert!(!src.contains("cx.use_local_with("));
        assert!(!src.contains("cx.actions().models::<"));
        assert!(!src.contains("cx.on_action_notify_models::<"));
    }

    fn assert_avoids_legacy_conversion_names(src: &str) {
        assert!(!src.contains("UiIntoElement"));
        assert!(!src.contains("UiHostBoundIntoElement"));
        assert!(!src.contains("UiChildIntoElement"));
        assert!(!src.contains("UiBuilderHostBoundIntoElementExt"));
    }

    fn assert_view_runtime_example_uses_app_ui_aliases(src: &str) {
        assert!(
            src.contains("fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui")
                || src.contains(
                    "fn render(&mut self, cx: &mut fret::AppUi<'_, '_, App>) -> fret::Ui",
                )
        );
        assert!(
            !src.contains("fn render(&mut self, cx: &mut ViewCx<'_, '_, KernelApp>) -> Elements")
        );
        assert!(!src.contains(
            "fn render(&mut self, cx: &mut fret::view::ViewCx<'_, '_, App>) -> Elements",
        ));
        assert!(!src.contains("ViewCx<'_, '_, KernelApp>"));
        assert!(!src.contains("ViewCx<'_, '_, App>"));
    }

    fn assert_prefers_view_builder_then_run(src: &str) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(normalized.contains(".view::<"));
        assert!(normalized.contains(".run()"));
        assert!(!normalized.contains(".run_view::<"));
    }

    fn assert_setup_surface_keeps_inline_closures_off_setup(src: &str) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(!normalized.contains(".setup(|"));
        assert!(!normalized.contains(".setup(move|"));
    }

    fn assert_current_imui_teaching_surface(
        name: &str,
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                normalized.contains(&marker),
                "{name} should keep teaching the current imui facade surface: {marker}"
            );
        }
        for marker in [
            "select_model_ex(",
            "window_ex(",
            "window_open_ex(",
            "floating_area_show_ex(",
            "begin_disabled(",
            "button_adapter(",
            "checkbox_model_adapter(",
            "fret_ui_kit::imui::adapters",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "{name} reintroduced a deleted or non-teaching imui surface: {marker}"
            );
        }
        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "{name} reintroduced a forbidden imui teaching marker: {marker}"
            );
        }
    }

    fn assert_prefers_grouped_data_surface(src: &str) {
        assert!(
            src.contains("cx.data().selector_layout(")
                || src.contains("cx.data().selector(")
                || src.contains("cx.data().query(")
                || src.contains("cx.data().query_async(")
                || src.contains("cx.data().query_async_local(")
        );
        assert!(!src.contains("fret_query::ui::QueryElementContextExt"));
        assert!(!src.contains("fret_selector::ui::SelectorElementContextExt"));
        assert!(!src.contains("cx.use_selector("));
        assert!(!src.contains("cx.use_query("));
        assert!(!src.contains("cx.use_query_async("));
        assert!(!src.contains("cx.use_query_async_local("));
    }

    fn assert_prefers_fret_query_facade(src: &str) {
        assert!(src.contains("use fret::query::{"));
        assert!(!src.contains("use fret_query::{"));
    }

    fn assert_shadcn_surface_is_curated(src: &str) {
        assert!(!src.contains("use fret_ui_shadcn as shadcn;"));
        assert!(!src.contains("use fret_ui_shadcn::{self as shadcn"));
        assert!(!src.contains("shadcn::shadcn_themes::"));
        assert!(!src.contains("shadcn::typography::"));
    }

    fn assert_advanced_entry_prefers_view_elements_alias(src: &str, state: &str) {
        let expected = format!(
            "fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut {state}) -> ViewElements"
        );
        assert!(src.contains(&expected));
        let legacy = format!(
            "fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut {state}) -> Elements"
        );
        assert!(!src.contains(&legacy));
    }

    fn assert_advanced_helpers_prefer_uicx(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(normalized.contains("UiCx<'_>"));
        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "legacy marker still present: {marker}"
            );
        }
    }

    fn assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(normalized.contains("UiTree<App>"));
        assert!(!normalized.contains("KernelApp"));
        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "legacy marker still present: {marker}"
            );
        }
    }

    fn assert_low_level_interop_examples_keep_direct_leaf_roots(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "unexpected marker present: {marker}"
            );
        }
    }

    fn assert_selected_view_runtime_examples_prefer_grouped_helpers(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "legacy marker still present: {marker}"
            );
        }
    }

    fn assert_advanced_generic_helpers_prefer_into_ui_element(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "legacy marker still present: {marker}"
            );
        }
    }

    fn assert_default_app_generic_helpers_prefer_into_ui_element(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(normalized.contains("ElementContext<'_,App>"));
        assert!(!normalized.contains("KernelApp"));
        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "legacy marker still present: {marker}"
            );
        }
    }

    fn source_slice<'a>(src: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
        let start = src
            .find(start_marker)
            .unwrap_or_else(|| panic!("missing start marker: {start_marker}"));
        let end = src[start..]
            .find(end_marker)
            .map(|offset| start + offset)
            .unwrap_or_else(|| panic!("missing end marker: {end_marker}"));
        &src[start..end]
    }

    #[test]
    fn migrated_examples_use_the_explicit_advanced_surface() {
        for src in [
            ASSETS_DEMO,
            ASYNC_PLAYGROUND_DEMO,
            CHART_DECLARATIVE_DEMO,
            CUSTOM_EFFECT_V1_DEMO,
            CUSTOM_EFFECT_V2_DEMO,
            CUSTOM_EFFECT_V3_DEMO,
            DROP_SHADOW_DEMO,
            ECHARTS_DEMO,
            EMBEDDED_VIEWPORT_DEMO,
            EMPTY_IDLE_DEMO,
            EXTRAS_MARQUEE_PERF_DEMO,
            GENUI_DEMO,
            HELLO_WORLD_COMPARE_DEMO,
            IMAGE_HEAVY_MEMORY_DEMO,
            IMUI_EDITOR_PROOF_DEMO,
            IMUI_FLOATING_WINDOWS_DEMO,
            IMUI_HELLO_DEMO,
            IMUI_INTERACTION_SHOWCASE_DEMO,
            IMUI_NODE_GRAPH_DEMO,
            IMUI_RESPONSE_SIGNALS_DEMO,
            IMUI_SHADCN_ADAPTER_DEMO,
            LAUNCHER_UTILITY_WINDOW_DEMO,
            LAUNCHER_UTILITY_WINDOW_MATERIALS_DEMO,
            LIQUID_GLASS_DEMO,
            MARKDOWN_DEMO,
            NODE_GRAPH_DEMO,
            POSTPROCESS_THEME_DEMO,
            TEXT_HEAVY_MEMORY_DEMO,
            WINDOW_HIT_TEST_PROBE_DEMO,
        ] {
            assert_uses_advanced_surface(src);
        }
    }

    #[test]
    fn advanced_reference_demos_are_explicitly_classified() {
        assert_explicit_advanced_reference_classification(
            "custom_effect_v1_demo",
            CUSTOM_EFFECT_V1_DEMO,
            &["effect/runtime ownership", "renderer/effect ABI"],
        );
        assert_explicit_advanced_reference_classification(
            "custom_effect_v2_demo",
            CUSTOM_EFFECT_V2_DEMO,
            &["effect/runtime ownership", "renderer/effect ABI"],
        );
        assert_explicit_advanced_reference_classification(
            "custom_effect_v3_demo",
            CUSTOM_EFFECT_V3_DEMO,
            &[
                "effect/runtime ownership",
                "renderer/effect ABI and diagnostics pipeline",
            ],
        );
        assert_explicit_advanced_reference_classification(
            "postprocess_theme_demo",
            POSTPROCESS_THEME_DEMO,
            &[
                "renderer/theme bridge ownership",
                "high-ceiling post-process story",
            ],
        );
        assert_explicit_advanced_reference_classification(
            "liquid_glass_demo",
            LIQUID_GLASS_DEMO,
            &[
                "renderer capability and effect/control graph ownership",
                "glass/warp behavior ceilings",
            ],
        );
        assert_explicit_advanced_reference_classification(
            "genui_demo",
            GENUI_DEMO,
            &[
                "explicit model ownership",
                "generator/editor integration",
                "catalog, runtime, and validation flows",
            ],
        );
        assert_explicit_advanced_reference_classification(
            "imui_floating_windows_demo",
            IMUI_FLOATING_WINDOWS_DEMO,
            &[
                "immediate-mode overlap/floating proof",
                "IMUI interaction contracts and diagnostics affordances",
            ],
        );
        assert_explicit_advanced_reference_classification(
            "imui_interaction_showcase_demo",
            IMUI_INTERACTION_SHOWCASE_DEMO,
            &[
                "product shell polish",
                "immediate-mode interaction affordances",
                "shadcn shell chrome",
            ],
        );
    }

    #[test]
    fn examples_docs_explicitly_name_the_advanced_reference_roster() {
        let normalized = EXAMPLES_DOCS_README.split_whitespace().collect::<String>();
        for marker in [
            "Explicit advanced/reference roster:",
            "`custom_effect_v1_demo`, `custom_effect_v2_demo`, and `custom_effect_v3_demo`",
            "renderer/effect reference surfaces",
            "`postprocess_theme_demo` and `liquid_glass_demo` are renderer/product-validation surfaces",
            "`genui_demo` is a generator/editor integration reference surface",
            "`imui_floating_windows_demo` is an IMUI overlap/floating proof surface",
            "`imui_response_signals_demo` is an IMUI proof/contract surface",
            "`imui_interaction_showcase_demo` and `imui_shadcn_adapter_demo` are IMUI product-validation surfaces",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                normalized.contains(&marker),
                "examples docs lost advanced/reference roster marker: {marker}"
            );
        }
    }

    #[test]
    fn todo_demo_prefers_default_app_surface() {
        let normalized = TODO_DEMO.split_whitespace().collect::<String>();

        assert!(TODO_DEMO.contains("use fret::app::prelude::*;"));
        assert!(!TODO_DEMO.contains("advanced::prelude::*"));
        assert!(!TODO_DEMO.contains("KernelApp"));
        assert!(!TODO_DEMO.contains("AppWindowId"));
        assert!(TODO_DEMO.contains("fn init(_app: &mut App, _window: WindowId) -> Self"));
        assert!(TODO_DEMO.contains("ui::single(cx, todo_page(theme, responsive, card))"));
        assert!(TODO_DEMO.contains("fn todo_page("));
        assert!(TODO_DEMO.contains("responsive: TodoResponsiveLayout,"));
        assert!(!TODO_DEMO.contains("let card = card.into_element(cx);"));
        assert!(!TODO_DEMO.contains("todo_page(theme, card).into_element(cx).into()"));
        assert!(!TODO_DEMO.contains("fret_cookbook::scaffold::"));
        assert!(!TODO_DEMO.contains("centered_page_muted("));
        assert!(!TODO_DEMO.contains("centered_page_background("));
        assert_avoids_legacy_conversion_names(TODO_DEMO);
        assert!(TODO_DEMO.contains("struct TodoLocals {"));
        assert!(TODO_DEMO.contains("fn new(cx: &mut AppUi<'_, '_>) -> Self {"));
        assert!(TODO_DEMO.contains("struct TodoDemoView;"));
        assert!(TODO_DEMO.contains("fn init(_app: &mut App, _window: WindowId) -> Self"));
        assert!(TODO_DEMO.contains("let locals = TodoLocals::new(cx);"));
        assert!(TODO_DEMO.contains("locals.bind_actions(cx);"));
        assert!(TODO_DEMO.contains("draft: cx.state().local::<String>(),"));
        assert!(normalized.contains(
            "filter:cx.state().local_init(||Some(Arc::<str>::from(TodoFilter::All.value()))),"
        ));
        assert!(TODO_DEMO.contains("next_id: cx.state().local_init(|| 4u64),"));
        assert!(TODO_DEMO.contains("todos: cx.state().local_init(|| {"));
        assert!(TODO_DEMO.contains("fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {"));
        assert!(normalized.contains(
            "letfilter_value=TodoFilter::from_value(locals.filter.layout_value(cx).as_deref());"
        ));
        assert!(TODO_DEMO.contains(".setup(fret_icons_lucide::app::install)"));
        assert!(TODO_DEMO.contains(".window_min_size(TODO_WINDOW_MIN_SIZE)"));
        assert!(TODO_DEMO.contains(".window_position_logical(TODO_WINDOW_POSITION_LOGICAL)"));
        assert!(TODO_DEMO.contains(".window_resize_increments(TODO_WINDOW_RESIZE_INCREMENTS)"));
        assert!(TODO_DEMO.contains("ui::for_each_keyed_with_cx("));
        assert!(TODO_DEMO.contains("fn todo_row<'a, Cx>("));
        assert!(TODO_DEMO.contains("Cx: fret::app::ElementContextAccess<'a, App>,"));
        assert!(!TODO_DEMO.contains("let cx = cx.elements();"));
        assert!(
            normalized.contains(
                "shadcn::Progress::from_value(progress_pct).a11y_label(\"Todocompletionprogress\").ui().rounded(Radius::Full).w_full().build()"
            )
        );
        assert!(normalized.contains(
            ".viewport_test_id(TEST_ID_ROWS).ui().w_full().h_full().flex_1().min_h_0().build()"
        ));
        assert!(!TODO_DEMO.contains("rows_max_height"));
        assert!(!normalized.contains(".a11y_label(\"Todocompletionprogress\").refine_style("));
        assert!(!normalized.contains(".viewport_test_id(TEST_ID_ROWS).refine_layout("));
        assert!(
            normalized.contains(
                ".corner_radii_override(Corners::all(Px(14.0))).ui().shadow_sm().build()"
            )
        );
        assert!(
            normalized
                .contains(".test_id(TEST_ID_DRAFT).ui().shadow_sm().flex_1().min_w_0().build()")
        );
        assert!(normalized.contains(
            ".a11y_label(format!(\"Show{}tasks\",filter.label().to_lowercase())).test_id(test_id).refine_style(ChromeRefinement::default().rounded(Radius::Full)).refine_layout(fret_ui_kit::LayoutRefinement::default().h_px(Px(28.0)).min_h(Px(28.0)),)"
        ));
        assert!(!TODO_DEMO.contains("footer_pill_chrome()"));
        assert!(!TODO_DEMO.contains("footer_pill_layout()"));
        assert!(TODO_DEMO.contains("ui::hover_region(move |cx, hovered| {"));
        assert!(TODO_DEMO.contains("ui::rich_text(rich)"));
        assert!(!TODO_DEMO.contains("HoverRegionProps"));
        assert!(!TODO_DEMO.contains("StyledTextProps"));
        assert!(TODO_DEMO.contains("ui::v_flex(move |cx| ui::single(cx, content))"));
        assert!(!TODO_DEMO.contains("ui::v_flex(move |cx| ui::children![cx; content])"));
        assert!(!TODO_DEMO.contains("cx: &mut fret_ui::ElementContext<'_, App>,"));
        assert!(!TODO_DEMO.contains("TodoLocals::new(app)"));
        assert!(!TODO_DEMO.contains("LocalState::from_model(app.models_mut().insert("));
    }

    #[test]
    fn simple_todo_demo_prefers_default_app_surface() {
        assert_uses_default_app_surface(SIMPLE_TODO_DEMO);
        assert_avoids_legacy_conversion_names(SIMPLE_TODO_DEMO);
        assert!(SIMPLE_TODO_DEMO.contains("struct TodoLocals {"));
        assert!(SIMPLE_TODO_DEMO.contains("fn new(cx: &mut AppUi<'_, '_>) -> Self {"));
        assert!(SIMPLE_TODO_DEMO.contains("struct SimpleTodoView;"));
        assert!(SIMPLE_TODO_DEMO.contains("fn init(_app: &mut App, _window: WindowId) -> Self"));
        assert!(SIMPLE_TODO_DEMO.contains("let locals = TodoLocals::new(cx);"));
        assert!(SIMPLE_TODO_DEMO.contains("locals.bind_actions(cx);"));
        assert!(SIMPLE_TODO_DEMO.contains("ui::single(cx, todo_page(theme, card))"));
        assert!(
            SIMPLE_TODO_DEMO.contains(
                "fn todo_page(theme: ThemeSnapshot, content: impl UiChild) -> impl UiChild"
            )
        );
        assert!(!SIMPLE_TODO_DEMO.contains("fret_cookbook::scaffold::"));
        assert!(!SIMPLE_TODO_DEMO.contains("centered_page_muted("));
        assert!(!SIMPLE_TODO_DEMO.contains("centered_page_background("));
        assert!(SIMPLE_TODO_DEMO.contains("draft: cx.state().local::<String>(),"));
        assert!(SIMPLE_TODO_DEMO.contains("next_id: cx.state().local_init(|| 3u64),"));
        assert!(SIMPLE_TODO_DEMO.contains("todos: cx.state().local_init(|| {"));
        assert!(SIMPLE_TODO_DEMO.contains(".local(&self.todos)"));
        assert!(SIMPLE_TODO_DEMO.contains(".payload_update_if::<act::Toggle>(|rows, id| {"));
        assert!(SIMPLE_TODO_DEMO.contains(".payload_update_if::<act::Remove>(|rows, id| {"));
        assert!(SIMPLE_TODO_DEMO.contains("ui_app_driver::UiAppDriver::new("));
        assert!(
            SIMPLE_TODO_DEMO.contains("fret::advanced::view::view_init_window::<SimpleTodoView>,")
        );
        assert!(SIMPLE_TODO_DEMO.contains("fret::advanced::view::view_view::<SimpleTodoView>,"));
        assert!(!SIMPLE_TODO_DEMO.contains("declarative::RenderRootContext"));
        assert!(!SIMPLE_TODO_DEMO.contains("CommandId"));
        assert!(!SIMPLE_TODO_DEMO.contains("UiTree<App>"));
        assert!(!SIMPLE_TODO_DEMO.contains("Model<"));
        assert!(!SIMPLE_TODO_DEMO.contains("TodoLocals::new(app)"));
        assert!(!SIMPLE_TODO_DEMO.contains("LocalState::from_model(app.models_mut().insert("));
    }

    #[test]
    fn query_demos_prefer_default_app_surface() {
        for src in [QUERY_DEMO, QUERY_ASYNC_TOKIO_DEMO] {
            assert_uses_default_app_surface_with_page(
                src,
                "query_page",
                "ui::single(cx, query_page(theme, card))",
            );
            assert_avoids_legacy_conversion_names(src);
        }
    }

    #[test]
    fn api_workbench_lite_demo_uses_query_for_sqlite_reads_and_mutation_for_explicit_submit() {
        assert!(API_WORKBENCH_LITE_DEMO.contains("use fret::app::prelude::*;"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("advanced::prelude::*"));
        assert!(
            API_WORKBENCH_LITE_DEMO.contains("fn init(_app: &mut App, window: WindowId) -> Self")
        );
        assert!(API_WORKBENCH_LITE_DEMO.contains("Cx: fret::app::RenderContextAccess<'a, App>,"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("fn shell_frame(\n    cx: &mut AppUi<'_, '_>,"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("fn request_panel(cx: &mut AppUi<'_, '_>,"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("fn response_panel(cx: &mut AppUi<'_, '_>,"));
        assert!(API_WORKBENCH_LITE_DEMO.contains(".with_in(cx, |cx| {"));
        assert!(API_WORKBENCH_LITE_DEMO.contains(".into_element_in(cx)"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains(".with(cx.elements(), |cx| {"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains(".into_element(cx.elements())"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("cx.elements()"));
        assert_avoids_legacy_conversion_names(API_WORKBENCH_LITE_DEMO);
        for marker in [
            "cx.data().query_async(",
            "cx.data().mutation_async(",
            "cx.data().update_after_mutation_completion(",
            "move |models, state| apply_response_snapshot(models, &locals, state)",
            "QueryKey::<Vec<PersistedHistoryEntry>>::new(HISTORY_QUERY_NS, &())",
            "persist_history_snapshot(",
            "load_saved_history(",
            "sqlx::query(",
            "cx.data().invalidate_query_namespace_after_mutation_success(",
            "MutationConcurrencyPolicy::AllowParallelLatestWins",
            "response_mutation.retry_last(",
            "history_save_mutation.retry_last(",
        ] {
            assert!(
                API_WORKBENCH_LITE_DEMO.contains(marker),
                "api_workbench_lite_demo should keep the SQLite history proof explicit: {marker}"
            );
        }
        assert!(!API_WORKBENCH_LITE_DEMO.contains("maybe_invalidate_saved_history_query("));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("locals.history"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("next_history_id"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains(".take_mutation_completion("));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("last_applied_seq"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("next_seq"));
    }

    #[test]
    fn hello_counter_demo_prefers_root_helper_surface() {
        assert!(HELLO_COUNTER_DEMO.contains("ui::single(cx, hello_counter_page(theme, card))"));
        assert!(HELLO_COUNTER_DEMO.contains(
            "fn hello_counter_page(theme: ThemeSnapshot, card: impl UiChild) -> impl UiChild"
        ));
        assert!(HELLO_COUNTER_DEMO.contains("let theme = cx.theme_snapshot();"));
        assert!(!HELLO_COUNTER_DEMO.contains("fn hello_counter_page(cx: &mut UiCx<'_>,"));
        assert!(!HELLO_COUNTER_DEMO.contains(".test_id(TEST_ID_ROOT).into_element(cx).into()"));
        assert!(!HELLO_COUNTER_DEMO.contains("Theme::global(&*cx.app).snapshot()"));
    }

    #[test]
    fn default_app_examples_prefer_app_theme_snapshot_helper() {
        for src in [HELLO_COUNTER_DEMO, QUERY_DEMO, QUERY_ASYNC_TOKIO_DEMO] {
            assert!(src.contains("let theme = cx.theme_snapshot();"));
            assert!(!src.contains("Theme::global(&*cx.app).snapshot()"));
        }
    }

    #[test]
    fn selected_advanced_runtime_examples_prefer_context_theme_snapshot_helpers() {
        for src in [
            EMBEDDED_VIEWPORT_DEMO,
            CUSTOM_EFFECT_V1_DEMO,
            CUSTOM_EFFECT_V2_DEMO,
            GENUI_DEMO,
            MARKDOWN_DEMO,
        ] {
            assert!(src.contains("cx.theme_snapshot()"));
            assert!(!src.contains("Theme::global(&*cx.app).snapshot()"));
        }
    }

    #[test]
    fn selected_element_context_examples_prefer_context_theme_reads() {
        for src in [CANVAS_DATAGRID_STRESS_DEMO, IMUI_INTERACTION_SHOWCASE_DEMO] {
            assert!(src.contains("cx.theme().snapshot()"));
            assert!(!src.contains("Theme::global(&*cx.app).snapshot()"));
        }
    }

    #[test]
    fn canonical_default_app_examples_stay_local_state_first() {
        for src in [
            HELLO_COUNTER_DEMO,
            QUERY_DEMO,
            QUERY_ASYNC_TOKIO_DEMO,
            SIMPLE_TODO_DEMO,
            TODO_DEMO,
        ] {
            assert_default_app_surface_prefers_local_state_first(src);
        }
    }

    #[test]
    fn low_level_interop_examples_keep_direct_leaf_root_contracts() {
        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_TEXTURE_IMPORTS_DEMO,
            &[
                "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalTextureImportsView) -> fret::Ui",
                "let theme = cx.theme().snapshot();",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-texture-imports-root\"),",
            ],
            &[
                "fn external_texture_imports_root(",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_TEXTURE_IMPORTS_WEB_DEMO,
            &[
                "let theme = cx.theme().snapshot();",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-texture-imports-web-root\"),",
                "make_panel(cx, fret_core::ViewportFit::Contain, \"ext-tex-web-contain\")",
            ],
            &[
                "fn external_texture_imports_web_root(",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_VIDEO_IMPORTS_AVF_DEMO,
            &[
                "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsAvfView) -> fret::Ui",
                "let theme = cx.theme().snapshot();",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-video-imports-avf-root\"),",
            ],
            &[
                "fn external_video_imports_avf_root(",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_VIDEO_IMPORTS_MF_DEMO,
            &[
                "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsMfView) -> fret::Ui",
                "let theme = cx.theme().snapshot();",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-video-imports-mf-root\"),",
            ],
            &[
                "fn external_video_imports_mf_root(",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            CHART_DECLARATIVE_DEMO,
            &[
                "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
                "chart_canvas_panel(cx.elements(), props).into()",
            ],
            &["fn chart_declarative_root("],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            NODE_GRAPH_DEMO,
            &[
                "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
                "node_graph_surface(cx.elements(), props).into()",
            ],
            &["fn node_graph_root("],
        );
    }

    #[test]
    fn manual_ui_tree_examples_keep_root_wrappers_on_local_typed_helpers() {
        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            CJK_CONFORMANCE_DEMO,
            &[
                "fn cjk_conformance_page<'a, Cx, C>(",
                "Cx: fret_ui::ElementContextAccess<'a, App>,",
                "theme: fret_ui::ThemeSnapshot,",
                "card: C,",
                ") -> impl fret_ui_kit::IntoUiElement<App> + use<Cx, C>",
                "C: fret_ui_kit::IntoUiElement<App>,",
                ".into_element_in(cx)",
                "ui::children![cx; cjk_conformance_page(cx, theme, card)]",
                "ui::v_flex(move |cx| ui::single(cx, card))",
            ],
            &[
                "cx: &mut fret_ui::ElementContext<'_, App>,",
                "let page = ui::container(|cx| {",
                "ui::v_flex(move |_cx| [card])",
            ],
        );

        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            EMOJI_CONFORMANCE_DEMO,
            &[
                "fn emoji_conformance_page<'a, Cx, C>(",
                "Cx: fret_ui::ElementContextAccess<'a, App>,",
                "theme: fret_ui::ThemeSnapshot,",
                "card: C,",
                ") -> impl fret_ui_kit::IntoUiElement<App> + use<Cx, C>",
                "C: fret_ui_kit::IntoUiElement<App>,",
                ".into_element_in(cx)",
                "ui::children![cx; emoji_conformance_page(cx, theme, card)]",
                "ui::v_flex(move |cx| ui::single(cx, card))",
            ],
            &[
                "cx: &mut fret_ui::ElementContext<'_, App>,",
                "let page = ui::container(|cx| {",
                "ui::v_flex(move |_cx| [card])",
            ],
        );
    }

    #[test]
    fn manual_form_demo_uses_app_ui_render_root_bridge() {
        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            FORM_DEMO,
            &[
                "app_ui_root: AppUiRenderRootState,",
                "form_state: LocalState<FormState>,",
                "LocalState::new_in(app.models_mut(), String::new())",
                "LocalState::new_in(app.models_mut(), None::<Arc<str>>)",
                "LocalState::new_in(app.models_mut(), form_state)",
                "let root = render_root_with_app_ui(",
                "let (submit_count, valid, dirty) = form_state.layout(cx).read_ref(",
                "let status_text = status.layout_value(cx);",
            ],
            &[
                "form_state: Model<FormState>,",
                "LocalState::from_model(app.models_mut().insert(",
                ".render_root(\"form-demo\", move |cx| {",
                "cx.observe_model(&form_state, Invalidation::Layout);",
                "cx.app.models().read(&form_state, |st| {",
                "cx.app.models().read(&status, |v| Arc::clone(v))",
                "status.layout(cx).value_or_else(|| Arc::from(\"Idle\"));",
            ],
        );
    }

    #[test]
    fn init_phase_local_state_examples_prefer_new_in_over_from_model() {
        for (src, required_markers) in [
            (
                FORM_DEMO,
                &[
                    "LocalState::new_in(app.models_mut(), String::new())",
                    "LocalState::new_in(app.models_mut(), None::<Arc<str>>)",
                    "LocalState::new_in(app.models_mut(), form_state)",
                ][..],
            ),
            (
                ASYNC_PLAYGROUND_DEMO,
                &[
                    "LocalState::new_in(app.models_mut(), initial.map(Arc::from))",
                    "LocalState::new_in(app.models_mut(), \"2\".to_string())",
                    "LocalState::new_in(app.models_mut(), false)",
                ][..],
            ),
            (
                TABLE_DEMO,
                &[
                    "LocalState::new_in(app.models_mut(), false)",
                    "LocalState::new_in(app.models_mut(), true)",
                    "Some(Arc::<str>::from(\"reorder\"))",
                ][..],
            ),
            (
                GENUI_DEMO,
                &[
                    "LocalState::new_in(app.models_mut(), true)",
                    "LocalState::new_in(app.models_mut(), SPEC_JSON.to_string())",
                    "LocalState::new_in(app.models_mut(), String::new())",
                ][..],
            ),
        ] {
            for marker in required_markers {
                assert!(
                    src.contains(marker),
                    "expected init-time LocalState marker missing: {marker}"
                );
            }
            assert!(!src.contains("LocalState::from_model(app.models_mut().insert("));
        }
    }

    #[test]
    fn manual_date_picker_demo_uses_app_ui_render_root_bridge() {
        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            DATE_PICKER_DEMO,
            &[
                "app_ui_root: AppUiRenderRootState,",
                "locals: Option<DatePickerDemoLocals>,",
                "struct DatePickerDemoLocals {",
                "fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {",
                "open: cx.state().local_init(|| false),",
                "month: cx",
                "if locals.is_none() {",
                "let root = render_root_with_app_ui(",
                "let open_value = open.layout_value(cx);",
                "let selected_value = selected.layout_value(cx);",
                "let month_label: Arc<str> = month.layout(cx).read_ref(",
            ],
            &[
                "open: Model<bool>,",
                "LocalState::from_model(app.models_mut().insert(",
                ".render_root(\"date-picker-demo\", move |cx| {",
                "cx.observe_model(&open, Invalidation::Layout);",
                "cx.app.models().get_copied(&open)",
                "cx.app.models().read(&month, |m| format!(\"{:?} {}\", m.month, m.year))",
                "open.layout(cx).copied_or(false)",
                "selected.layout(cx).value_or_default()",
            ],
        );
    }

    #[test]
    fn manual_sonner_demo_uses_app_ui_render_root_bridge() {
        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            SONNER_DEMO,
            &[
                "app_ui_root: AppUiRenderRootState,",
                "locals: Option<SonnerDemoLocals>,",
                "struct SonnerDemoLocals {",
                "fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {",
                "last_action: cx.state().local_init(|| Arc::<str>::from(\"<none>\")),",
                "if locals.is_none() {",
                "let root = render_root_with_app_ui(",
                "let last_action_value = last_action.layout_value(cx);",
            ],
            &[
                "last_action: Model<Arc<str>>,",
                "LocalState::from_model(app.models_mut().insert(",
                ".render_root(\"sonner-demo\", |cx| {",
                "cx.observe_model(&last_action, Invalidation::Layout);",
                "cx.app.models().get_cloned(&last_action)",
                "last_action.layout(cx).value_or_else(",
            ],
        );
    }

    #[test]
    fn manual_ime_smoke_demo_uses_app_ui_render_root_bridge() {
        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            IME_SMOKE_DEMO,
            &[
                "use fret::app::RenderContextAccess as _;",
                "app_ui_root: AppUiRenderRootState,",
                "locals: Option<ImeSmokeLocals>,",
                "struct ImeSmokeLocals {",
                "fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {",
                "input_single: cx.state().local::<String>(),",
                "last_ime: cx.state().local_init(|| Arc::<str>::from(\"IME: <none>\")),",
                "if locals.is_none() {",
                "let root = render_root_with_app_ui(",
                "let theme = cx.theme_snapshot();",
                "let last = last_ime.paint_value(cx);",
                "shadcn::Input::new(&input_single)",
                "shadcn::Textarea::new(&input_multi)",
            ],
            &[
                "input_single: Model<String>,",
                "last_ime: Model<Arc<str>>,",
                "LocalState::from_model(app.models_mut().insert(",
                ".render_root(\"ime-smoke\",",
                "cx.observe_model(&last_ime, Invalidation::Paint);",
                "cx.app.models().read(&last_ime, |v| v.clone())",
                "last_ime.paint(cx).value_or_else(",
                "input_single.clone_model()",
                "input_multi.clone_model()",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );
    }

    #[test]
    fn manual_emoji_conformance_demo_uses_app_ui_render_root_bridge() {
        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            EMOJI_CONFORMANCE_DEMO,
            &[
                "app_ui_root: AppUiRenderRootState,",
                "locals: Option<EmojiConformanceLocals>,",
                "struct EmojiConformanceLocals {",
                "fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {",
                "emoji_font_override: cx.state().local_init(|| None::<Arc<str>>),",
                "if locals.is_none() {",
                "let root = render_root_with_app_ui(",
                "let selected_emoji_font = emoji_font_override.layout_value(cx);",
            ],
            &[
                "emoji_font_override: Model<Option<Arc<str>>>,",
                "LocalState::from_model(app.models_mut().insert(",
                ".render_root(\"emoji-conformance\", |cx| {",
                "cx.observe_model(&emoji_font_override, Invalidation::Layout);",
                "cx.app.models().read(&emoji_font_override, |v| v.clone())",
                "emoji_font_override.layout(cx).value_or_default()",
            ],
        );
    }

    #[test]
    fn select_examples_prefer_local_state_bridges_over_clone_model() {
        assert!(
            ASYNC_PLAYGROUND_DEMO.contains(
                "shadcn::Select::new(&config.cancel_mode.value, &config.cancel_mode.open)"
            )
        );
        assert!(!ASYNC_PLAYGROUND_DEMO.contains("config.cancel_mode.open.clone_model()"));

        assert!(FORM_DEMO.contains("shadcn::Select::new(&role, &role_open)"));
        assert!(
            !FORM_DEMO.contains("shadcn::Select::new(role.clone_model(), role_open.clone_model())")
        );

        assert!(
            EMOJI_CONFORMANCE_DEMO
                .contains("shadcn::Select::new(&emoji_font_override, &emoji_font_override_open)")
        );
        assert!(!EMOJI_CONFORMANCE_DEMO.contains("emoji_font_override_open.clone_model()"));
    }

    #[test]
    fn date_picker_examples_prefer_local_state_bridges_over_clone_model() {
        assert!(DATE_PICKER_DEMO.contains("shadcn::Switch::new(&week_start_monday)"));
        assert!(DATE_PICKER_DEMO.contains("shadcn::Switch::new(&show_outside_days)"));
        assert!(DATE_PICKER_DEMO.contains("shadcn::Switch::new(&disable_outside_days)"));
        assert!(DATE_PICKER_DEMO.contains("shadcn::Switch::new(&disable_weekends)"));
        assert!(DATE_PICKER_DEMO.contains("shadcn::Switch::new(&disabled)"));
        assert!(DATE_PICKER_DEMO.contains("shadcn::DatePicker::new(&open, &month, &selected)"));
        assert!(DATE_PICKER_DEMO.contains("shadcn::Calendar::new(&month, &selected)"));
        assert!(FORM_DEMO.contains("shadcn::DatePicker::new("));
        assert!(
            FORM_DEMO.contains("&start_date_open,")
                && FORM_DEMO.contains("&start_date_month,")
                && FORM_DEMO.contains("&start_date,")
        );
        assert!(!DATE_PICKER_DEMO.contains("week_start_monday.clone_model()"));
        assert!(!DATE_PICKER_DEMO.contains("show_outside_days.clone_model()"));
        assert!(!DATE_PICKER_DEMO.contains("disable_outside_days.clone_model()"));
        assert!(!DATE_PICKER_DEMO.contains("disable_weekends.clone_model()"));
        assert!(!DATE_PICKER_DEMO.contains("disabled.clone_model()"));
        assert!(!DATE_PICKER_DEMO.contains("open.clone_model()"));
        assert!(!DATE_PICKER_DEMO.contains("month.clone_model()"));
        assert!(!DATE_PICKER_DEMO.contains("selected.clone_model()"));
        assert!(!FORM_DEMO.contains("DatePicker::new_controllable("));
        assert!(!FORM_DEMO.contains("start_date.clone_model()"));
    }

    #[test]
    fn bool_control_examples_prefer_local_state_bridges_over_clone_model() {
        assert!(DROP_SHADOW_DEMO.contains("shadcn::Switch::new(&enabled_state)"));
        assert!(DROP_SHADOW_DEMO.contains("shadcn::Switch::new(&stress_state)"));
        assert!(!DROP_SHADOW_DEMO.contains("enabled_state.clone_model()"));
        assert!(!DROP_SHADOW_DEMO.contains("stress_state.clone_model()"));

        assert!(MARKDOWN_DEMO.contains("shadcn::Switch::new(&wrap_code_state)"));
        assert!(MARKDOWN_DEMO.contains("shadcn::Switch::new(&cap_code_height_state)"));
        assert!(!MARKDOWN_DEMO.contains("wrap_code_state.clone_model()"));
        assert!(!MARKDOWN_DEMO.contains("cap_code_height_state.clone_model()"));
    }

    #[test]
    fn form_examples_prefer_local_state_form_bridges_over_clone_model() {
        assert!(
            FORM_DEMO.contains("registry.register_field(\"name\", &name, String::new(), |v| {")
        );
        assert!(
            FORM_DEMO.contains("registry.register_field(\"email\", &email, String::new(), |v| {")
        );
        assert!(FORM_DEMO.contains("registry.register_field(\"role\", &role, None, |v| {"));
        assert!(
            FORM_DEMO.contains("registry.register_field(\"start_date\", &start_date, None, |v| {")
        );
        assert!(FORM_DEMO.contains("shadcn::FormField::new("));
        assert!(FORM_DEMO.contains("&form_state,"));
        assert!(FORM_DEMO.contains("shadcn::Input::new(&name)"));
        assert!(FORM_DEMO.contains("shadcn::Input::new(&email)"));
        assert!(!FORM_DEMO.contains("form_state.clone_model()"));
        assert!(!FORM_DEMO.contains("name.clone_model()"));
        assert!(!FORM_DEMO.contains("email.clone_model()"));
        assert!(!FORM_DEMO.contains("registry.register_field(\"name\", name.clone_model(),"));
        assert!(!FORM_DEMO.contains("registry.register_field(\"email\", email.clone_model(),"));
        assert!(!FORM_DEMO.contains("registry.register_field(\"role\", role.clone_model(),"));
        assert!(
            !FORM_DEMO
                .contains("registry.register_field(\"start_date\", start_date.clone_model(),")
        );
    }

    #[test]
    fn manual_components_gallery_uses_app_ui_render_root_bridge() {
        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            COMPONENTS_GALLERY_DEMO,
            &[
                "app_ui_root: AppUiRenderRootState,",
                "fn components_gallery_table_cell(",
                "cx: &mut dyn fret_ui::ElementContextAccess<'_, App>,",
                "let cx = cx.elements();",
                "let cell_at = Arc::new(components_gallery_table_cell);",
                "let root = render_root_with_app_ui(",
                "let selected = tree_state.layout(cx).read_ref(|s| s.selected).ok().flatten();",
                "let checkbox_value = checkbox.layout(cx).copied_or(false);",
                "let selected_emoji_font = emoji_font_override.layout(cx).value_or_default();",
                "let last_action_value = last_action.layout(cx).value_or_else(",
            ],
            &[
                "move |cx: &mut ElementContext<'_, App>, col: &ColumnDef<u64>, row: &u64| {",
                ".render_root(\"components-gallery\", |cx| {",
                "cx.observe_model(&tree_state, Invalidation::Layout);",
                "cx.app.models().get_copied(&checkbox).unwrap_or(false);",
                "cx.app.models().get_cloned(&last_action);",
                "cx.app.models().read(&emoji_font_override, |v| v.clone())",
            ],
        );
    }

    #[test]
    fn imui_editor_proof_non_raw_helpers_prefer_typed_return_signatures() {
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("fn render_editor_name_assist_surface("));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("fn render_authoring_parity_surface("));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("fn render_authoring_parity_shared_state("));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("fn render_authoring_parity_declarative_group("));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("fn render_authoring_parity_imui_group("));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("fn render_authoring_parity_imui_host<H, F>("));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains(") -> impl IntoUiElement<KernelApp> + use<> {"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains(") -> impl IntoUiElement<H> + use<H, F>"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("fn proof_compact_readout<H: UiHost>("));
        assert_eq!(
            IMUI_EDITOR_PROOF_DEMO
                .matches(") -> fret_ui::element::AnyElement {")
                .count(),
            1,
            "only the proof-local compact readout leaf helper should keep an AnyElement return"
        );
    }

    #[test]
    fn imui_editor_proof_authoring_immediate_column_uses_official_editor_adapters() {
        let imui_group = source_slice(
            IMUI_EDITOR_PROOF_DEMO,
            "fn render_authoring_parity_imui_group(",
            "fn build_authoring_parity_gradient_editor(",
        );
        let normalized = imui_group.split_whitespace().collect::<String>();

        for marker in [
            "render_authoring_parity_imui_host(cx, move |ui| {",
            "editor_imui::property_group(",
            "editor_imui::property_grid(",
            "editor_imui::text_field(",
            "editor_imui::drag_value(",
            "editor_imui::numeric_input(",
            "editor_imui::slider(",
            "editor_imui::field_status_badge(",
            "editor_imui::checkbox(",
            "editor_imui::enum_select(",
            "let gradient_editor = build_authoring_parity_gradient_editor(",
            "editor_imui::gradient_editor(ui, gradient_editor);",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                normalized.contains(&marker),
                "authoring parity immediate column should route editor controls through official adapters: {marker}"
            );
        }

        for marker in [
            "FieldStatusBadge::new(FieldStatus::Dirty).into_element(cx)",
            "GradientEditor::new(",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "authoring parity immediate column should not bypass the adapter layer: {marker}"
            );
        }
    }

    #[test]
    fn first_party_imui_examples_keep_current_facade_teaching_surface() {
        assert_current_imui_teaching_surface(
            "imui_hello_demo",
            IMUI_HELLO_DEMO,
            &[
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "ui.checkbox_model(\"Enabled\", enabled_state.model())",
            ],
            &[],
        );

        assert_current_imui_teaching_surface(
            "imui_floating_windows_demo",
            IMUI_FLOATING_WINDOWS_DEMO,
            &[
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "ui.window_with_options(",
                "ui.combo_model_with_options(",
            ],
            &[],
        );

        assert_current_imui_teaching_surface(
            "imui_response_signals_demo",
            IMUI_RESPONSE_SIGNALS_DEMO,
            &[
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "click.secondary_clicked()",
                "drag.drag_started()",
                "trigger.context_menu_requested()",
                "menu_lifecycle.activated()",
                "combo_resp.trigger.activated()",
                "combo_model_resp.deactivated_after_edit()",
            ],
            &[],
        );

        assert_current_imui_teaching_surface(
            "imui_interaction_showcase_demo",
            IMUI_INTERACTION_SHOWCASE_DEMO,
            &[
                "Showcase surface for immediate-mode interaction affordances.",
                "Current proof/contract surface stays in `imui_response_signals_demo`.",
                "use fret_ui_shadcn::facade as shadcn;",
                "fret_imui::imui(cx, move |ui| {",
                "ui.begin_menu_with_options(",
                "ui.tab_bar_with_options(",
                "ui.begin_popup_context_menu(",
            ],
            &[],
        );

        assert_current_imui_teaching_surface(
            "imui_shadcn_adapter_demo",
            IMUI_SHADCN_ADAPTER_DEMO,
            &[
                "UiWriterImUiFacadeExt as _",
                "ui.combo_model_with_options(",
                "ui.separator_text(\"Inspector snapshot\")",
                "ui.table_with_options(",
                "ui.virtual_list_with_options(",
            ],
            &[],
        );

        assert_current_imui_teaching_surface(
            "imui_node_graph_demo",
            IMUI_NODE_GRAPH_DEMO,
            &[
                "compatibility-oriented and should not be treated as the default downstream",
                "Prefer the declarative node-graph surfaces for normal downstream guidance.",
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "NodeGraphSurfaceCompatRetainedProps::new(",
                "node_graph_surface_compat_retained(",
            ],
            &[],
        );

        assert_current_imui_teaching_surface(
            "imui_editor_proof_demo",
            IMUI_EDITOR_PROOF_DEMO,
            &[
                "use fret_ui_editor::imui as editor_imui;",
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "editor_imui::property_grid(",
                "editor_imui::numeric_input(",
                "editor_imui::gradient_editor(",
            ],
            &[],
        );
    }

    #[test]
    fn immediate_mode_examples_docs_name_the_golden_pair_and_reference_roster() {
        for marker in [
            "Immediate-mode sidecar (when you intentionally want the IMUI lane):",
            "imui_action_basics",
            "imui_editor_proof_demo",
            "imui_hello_demo",
            "imui_interaction_showcase_demo",
            "imui_response_signals_demo",
            "imui_shadcn_adapter_demo",
            "imui_floating_windows_demo",
            "imui_node_graph_demo",
            "Golden pair:",
            "Reference/smoke:",
            "Reference/contract proof:",
            "Reference/product-validation:",
            "Compatibility-only:",
        ] {
            assert!(
                EXAMPLES_DOCS_README.contains(marker),
                "docs/examples/README.md should classify the immediate-mode teaching surfaces: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_examples_docs_name_the_mounting_rule_for_imui_vs_imui_raw() {
        for marker in [
            "Mounting rule for the immediate-mode lane:",
            "`fret_imui::imui(...)` is now the safe default",
            "`fret_imui::imui_raw(cx, ...)`.",
            "`imui_raw(...)` is the advanced seam",
            "`imui_action_basics` demonstrates the explicit layout-host + raw shape; `imui_hello_demo`",
        ] {
            assert!(
                EXAMPLES_DOCS_README.contains(marker),
                "docs/examples/README.md should keep the immediate-mode mounting rule explicit: {marker}"
            );
        }

        for marker in [
            "If your IMUI content already lives under an explicit layout host, prefer",
            "If you are mounting IMUI directly at the view root or under a non-layout parent, prefer",
            "`imui_raw(...)` is the advanced explicit-layout seam, not evidence that generic helper growth",
        ] {
            assert!(
                IMUI_ROOT_HOSTING_RULE_NOTE.contains(marker),
                "the immediate-mode workstream should keep the root-host teaching rule explicit: {marker}"
            );
        }
    }

    #[test]
    fn imui_editor_proof_demo_defaults_to_imgui_like_dense_preset_for_editor_grade_launches() {
        for marker in [
            "const ENV_EDITOR_PRESET: &str = \"FRET_IMUI_EDITOR_PRESET\";",
            "editor_theme_preset_from_env(ENV_EDITOR_PRESET)",
            "EditorThemePresetV1::ImguiLikeDense",
        ] {
            assert!(
                IMUI_EDITOR_PROOF_DEMO.contains(marker),
                "imui_editor_proof_demo should default to the dense editor preset while still honoring env override: {marker}"
            );
        }
    }

    #[test]
    fn imui_editor_proof_demo_keeps_a_demo_owned_fixed_editor_theme() {
        for marker in [
            ".defaults(Defaults {",
            "shadcn: false,",
            "install_imui_editor_proof_theme(app);",
            "shadcn::themes::apply_shadcn_new_york(",
        ] {
            assert!(
                IMUI_EDITOR_PROOF_DEMO.contains(marker),
                "imui_editor_proof_demo should keep a demo-owned fixed editor host theme: {marker}"
            );
        }

        assert!(
            !IMUI_EDITOR_PROOF_DEMO.contains("shadcn::app::install_with_theme("),
            "imui_editor_proof_demo should not re-enter the default shadcn install lifecycle"
        );
    }

    #[test]
    fn immediate_mode_examples_docs_name_the_stable_identity_rule() {
        for marker in [
            "Stable identity rule for the immediate-mode lane:",
            "`ui.for_each_unkeyed(...)` is acceptable.",
            "`ui.for_each_keyed(...)` or `ui.id(key, ...)`.",
            "Rebuild rows each frame; do not treat element values as cloneable reusable UI.",
            "`imui_editor_proof_demo` is the heavier proof where explicit stable identity is",
        ] {
            assert!(
                EXAMPLES_DOCS_README.contains(marker),
                "docs/examples/README.md should keep the immediate-mode stable-identity rule explicit: {marker}"
            );
        }

        for marker in [
            "For static lists whose order never changes, `ui.for_each_unkeyed(...)` is acceptable.",
            "For dynamic collections that can insert, remove, reorder, or preserve per-row state, prefer",
            "Rebuild UI rows each frame; do not treat elements as cloneable reusable values.",
            "already uses explicit `ui.id(...)` where stable panel identity matters",
        ] {
            assert!(
                IMUI_STABLE_IDENTITY_RULE_NOTE.contains(marker),
                "the immediate-mode workstream should keep the stable-identity teaching rule explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p0_response_status_lifecycle_follow_on() {
        for marker in [
            "`fret-authoring::Response` must stay unchanged.",
            "Richer lifecycle status stays in `fret-ui-kit::imui::ResponseExt`.",
            "The initial quartet is:",
            "`activated`",
            "`deactivated`",
            "`edited`",
            "`deactivated_after_edit`",
            "Do not widen `crates/fret-ui` or invent a global key-owner model in this lane.",
            "`apps/fret-examples/src/imui_response_signals_demo.rs`",
        ] {
            assert!(
                IMUI_RESPONSE_STATUS_LIFECYCLE_DESIGN.contains(marker),
                "the immediate-mode workstream should keep the P0 response-status lifecycle design explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-response-status-lifecycle-v1\"",
            "\"follow_on_of\": \"imui-editor-grade-product-closure-v1\"",
            "immediate_mode_workstream_freezes_the_p0_response_status_lifecycle_follow_on",
        ] {
            assert!(
                IMUI_RESPONSE_STATUS_LIFECYCLE_WORKSTREAM.contains(marker),
                "the response-status lifecycle lane state should keep the follow-on marker: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-response-status-lifecycle-v1/` now proves this rule",
            "`docs/workstreams/imui-response-status-lifecycle-v1/` now owns the narrow",
            "`ResponseExt` lifecycle vocabulary slice",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the narrow P0 follow-on marker explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p0_menu_tab_trigger_response_surface_follow_on() {
        for marker in [
            "Status: closed closeout note",
            "`begin_menu_response[_with_options]` now returns `DisclosureResponse`.",
            "`begin_submenu_response[_with_options]` now returns `DisclosureResponse`.",
            "`tab_bar_response[_with_options]` now returns `TabBarResponse`.",
            "widen `fret-authoring::Response` or `crates/fret-ui`",
            "`ecosystem/fret-ui-kit::imui`",
            "`docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/` now owns the",
        ] {
            assert!(
                IMUI_MENU_TAB_TRIGGER_RESPONSE_SURFACE_FINAL_STATUS.contains(marker),
                "the menu/tab trigger response-surface closeout should stay explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-menu-tab-trigger-response-surface-v1\"",
            "\"follow_on_of\": \"imui-editor-grade-product-closure-v1\"",
            "\"status\": \"closed\"",
            "immediate_mode_workstream_freezes_the_p0_menu_tab_trigger_response_surface_follow_on",
        ] {
            assert!(
                IMUI_MENU_TAB_TRIGGER_RESPONSE_SURFACE_WORKSTREAM.contains(marker),
                "the menu/tab trigger response-surface lane state should keep the follow-on marker: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-menu-tab-trigger-response-surface-v1/` now owns the",
            "helper-owned menu/submenu/tab trigger response-surface decision",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the helper-owned trigger response follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-menu-tab-trigger-response-surface-v1/` now owns the",
            "helper-owned menu/submenu/tab trigger response-surface decision",
        ] {
            assert!(
                IMUI_RESPONSE_STATUS_LIFECYCLE_TODO.contains(marker),
                "the lifecycle lane should keep the helper-owned trigger response deferral explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p0_menu_tab_trigger_response_canonicalization_follow_on()
     {
        for marker in [
            "`begin_menu[_with_options] -> DisclosureResponse`",
            "`begin_submenu[_with_options] -> DisclosureResponse`",
            "`tab_bar[_with_options] -> TabBarResponse`",
            "Remove the duplicate `*_response*` compatibility entry points",
            "helper-owned menu/submenu/tab triggers expose one canonical outward response surface",
        ] {
            assert!(
                IMUI_MENU_TAB_TRIGGER_RESPONSE_CANONICALIZATION_DESIGN.contains(marker),
                "the canonicalization lane design should stay explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout note",
            "`begin_menu[_with_options]` now returns `DisclosureResponse`.",
            "`begin_submenu[_with_options]` now returns `DisclosureResponse`.",
            "`tab_bar[_with_options]` now returns `TabBarResponse`.",
            "The duplicate `begin_menu_response[_with_options]`",
            "The shipped helper surface stays inside `ecosystem/fret-ui-kit::imui`",
            "`fret-authoring::Response` or `crates/fret-ui`.",
        ] {
            assert!(
                IMUI_MENU_TAB_TRIGGER_RESPONSE_CANONICALIZATION_FINAL_STATUS.contains(marker),
                "the canonicalization closeout should stay explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-menu-tab-trigger-response-canonicalization-v1\"",
            "\"follow_on_of\": \"imui-menu-tab-trigger-response-surface-v1\"",
            "\"status\": \"closed\"",
            "menu_and_submenu_helpers_report_toggle_and_trigger_edges",
            "tab_bar_helper_reports_selected_change_and_trigger_edges",
            "imui_response_signals_demo_keeps_canonical_menu_tab_trigger_response_proof",
        ] {
            assert!(
                IMUI_MENU_TAB_TRIGGER_RESPONSE_CANONICALIZATION_WORKSTREAM.contains(marker),
                "the canonicalization lane state should stay explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/` now owns the",
            "cleanup closeout that removes the duplicate alias layer after the response surface landed",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the canonicalization follow-on explicit: {marker}"
            );
        }
    }

    #[test]
    fn imui_response_signals_demo_keeps_menu_and_combo_lifecycle_proof() {
        assert_current_imui_teaching_surface(
            "imui_response_signals_demo_lifecycle_expansion",
            IMUI_RESPONSE_SIGNALS_DEMO,
            &[
                "let menu_lifecycle = ui.menu_item_with_options(",
                "menu_lifecycle.activated()",
                "menu_lifecycle.deactivated()",
                "let combo_resp = ui.combo_with_options(",
                "combo_resp.trigger.activated()",
                "combo_resp.trigger.deactivated()",
                "let combo_model_resp = ui.combo_model_with_options(",
                "combo_model_resp.edited()",
                "combo_model_resp.deactivated_after_edit()",
            ],
            &[],
        );
    }

    #[test]
    fn imui_response_signals_demo_keeps_canonical_menu_tab_trigger_response_proof() {
        assert_current_imui_teaching_surface(
            "imui_response_signals_demo_canonical_trigger_response_surface",
            IMUI_RESPONSE_SIGNALS_DEMO,
            &[
                "let file_menu = ui.begin_menu_with_options(",
                "file_menu.opened()",
                "file_menu.closed()",
                "let recent_menu = ui.begin_submenu_with_options(",
                "recent_menu.toggled()",
                "let tab_response = ui.tab_bar_with_options(",
                "tab_response.selected_changed()",
                "if let Some(scene_tab) = tab_response.trigger(\"scene\") {",
                "scene_tab.clicked()",
                "scene_tab.activated()",
                "scene_tab.deactivated()",
            ],
            &[],
        );
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_workbench_shell_proof_matrix() {
        for marker in [
            "`apps/fret-examples/src/workspace_shell_demo.rs`",
            "`apps/fret-examples/src/editor_notes_demo.rs`",
            "`apps/fret-examples/src/imui_editor_proof_demo.rs`",
            "`docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`",
            "`docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`",
            "`workspace_shell_demo` is the primary P1 coherent workbench-shell proof",
            "`editor_notes_demo` is the minimal secondary proof for shell-mounted rails",
            "shell-level missing pieces should stay out of the generic `imui` backlog",
        ] {
            assert!(
                IMUI_WORKBENCH_PROOF_MATRIX_NOTE.contains(marker),
                "the workstream should keep the P1 workbench proof matrix explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_shell_diag_smoke_minimum() {
        for marker in [
            "`diag-hardening-smoke-workspace` should remain the promoted P1 shell smoke suite.",
            "Do not create a second parallel P1 shell suite yet.",
            "tab close / reorder / split preview",
            "dirty-close prompt",
            "content-focus restore via Escape",
            "left-rail / file-tree liveness",
            "`workspace-shell-demo-tab-close-dirty-shows-prompt-and-discard-closes-smoke.json`",
            "`workspace-shell-demo-tabstrip-escape-restores-content-focus-smoke.json`",
            "`workspace-shell-demo-file-tree-bounce-keep-alive.json`",
        ] {
            assert!(
                IMUI_P1_SHELL_DIAG_SMOKE_DECISION_NOTE.contains(marker),
                "the immediate-mode workstream should keep the P1 shell diagnostics floor explicit: {marker}"
            );
        }

        for marker in [
            "workspace-shell-demo-tab-close-button-closes-tab-smoke.json",
            "workspace-shell-demo-tab-reorder-first-to-end-smoke.json",
            "workspace-shell-demo-tab-drag-to-split-right-preview-invariants-smoke.json",
            "workspace-shell-demo-tab-close-dirty-shows-prompt-and-discard-closes-smoke.json",
            "workspace-shell-demo-tabstrip-escape-restores-content-focus-smoke.json",
            "workspace-shell-demo-file-tree-bounce-keep-alive.json",
        ] {
            assert!(
                WORKSPACE_HARDENING_SHELL_DIAG_SUITE.contains(marker),
                "the promoted workspace shell smoke suite should keep the frozen minimum roster entry: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_default_workbench_assembly_decision() {
        for marker in [
            "`apps/fret-examples/src/workspace_shell_demo.rs`",
            "`apps/fret-examples/src/editor_notes_demo.rs`",
            "`apps/fret-ui-gallery/src/driver/render_flow.rs`",
            "`docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`",
            "Keep the default workbench answer as explicit example-local assembly over the frozen starter set.",
            "Do not introduce `WorkspaceWorkbenchShell`, `EditorWorkbenchShell`, or a similar promoted helper yet.",
            "Keep `workspace_shell_demo` as the broader P1 product proof.",
            "Keep UI Gallery workspace shell as a shell-chrome reference/exemplar, not as a competing P1",
        ] {
            assert!(
                IMUI_WORKBENCH_ASSEMBLY_DECISION_NOTE.contains(marker),
                "the immediate-mode workstream should keep the P1 default workbench assembly decision explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p2_first_open_diagnostics_path() {
        for marker in [
            "The P2 first-open path is CLI-first for evidence production.",
            "DevTools GUI and MCP are thin consumers of the same selector, script, bundle, and regression",
            "\"Compare\" belongs to the shared artifacts layer:",
            "`cargo run -p fretboard-dev -- diag inspect on`",
            "`cargo run -p fretboard-dev -- diag pick-apply <script> --ptr <json-pointer>`",
            "`cargo run -p fretboard-dev -- diag run <script> --dir <session-dir> --session-auto --launch -- <target cmd>`",
            "`cargo run -p fretboard-dev -- diag latest`",
            "`cargo run -p fretboard-dev -- diag compare <a> <b> --json`",
            "`cargo run -p fret-devtools`",
            "`cargo run -p fret-devtools-mcp`",
            "`fret_diag_regression_summarize`",
            "`fret_diag_regression_dashboard` read those same artifacts as thin",
            "Portable artifacts remain the handoff unit;",
        ] {
            assert!(
                IMUI_P2_FIRST_OPEN_DIAGNOSTICS_PATH_NOTE.contains(marker),
                "the immediate-mode workstream should keep the P2 first-open diagnostics path explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p2_diagnostics_owner_split() {
        for marker in [
            "`ecosystem/fret-bootstrap` owns the in-app diagnostics runtime/export seam.",
            "`crates/fret-diag` owns orchestration, artifact tooling, compare/summarize/dashboard projections,",
            "`apps/fret-devtools` owns editor-grade diagnostics UX over the shared contracts.",
            "`apps/fret-devtools-mcp` owns the headless MCP adapter and resource/tool projection over the same",
            "Do not move orchestration policy into `ecosystem/fret-bootstrap`.",
            "Do not let `apps/fret-devtools` or `apps/fret-devtools-mcp` invent a second run model or",
            "When GUI or MCP needs a new capability that CLI would also need, land it in `crates/fret-diag`",
            "When the target app needs new inspect/pick/script/runtime evidence, land it in",
        ] {
            assert!(
                IMUI_P2_DIAGNOSTICS_OWNER_SPLIT_NOTE.contains(marker),
                "the immediate-mode workstream should keep the P2 diagnostics owner split explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p2_bounded_devtools_smoke_package() {
        for marker in [
            "`python3 tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke`",
            "`tools/diag-campaigns/devtools-first-open-smoke.json`",
            "`tools/diag-scripts/tooling/todo/todo-baseline.json`",
            "verify that `diag resolve latest` and `diag latest` resolve through",
            "`script.result.json:last_bundle_dir`",
            "run direct `diag compare` over `todo-after-add` vs `todo-after-toggle-done`",
            "`campaign.manifest.json`",
            "`diag summarize <campaign_root> --dir <campaign_root> --json`",
            "`regression.summary.json`",
            "`regression.index.json`",
            "run `diag dashboard <campaign_root> --json`",
        ] {
            assert!(
                IMUI_P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_NOTE.contains(marker),
                "the immediate-mode workstream should keep the P2 bounded devtools smoke package explicit: {marker}"
            );
        }

        for marker in [
            "CAMPAIGN_ID = \"devtools-first-open-smoke\"",
            "SCRIPT_PATH = \"tools/diag-scripts/tooling/todo/todo-baseline.json\"",
            "\"diag\",",
            "\"run\",",
            "\"resolve\",",
            "\"latest\",",
            "\"compare\",",
            "\"campaign\",",
            "\"dashboard\",",
            "\"regression.summary.json\"",
            "\"regression.index.json\"",
        ] {
            assert!(
                IMUI_P2_DEVTOOLS_SMOKE_GATE_SCRIPT.contains(marker),
                "the bounded P2 devtools smoke gate should keep the shared first-open loop step: {marker}"
            );
        }

        for marker in [
            "\"id\": \"devtools-first-open-smoke\"",
            "\"kind\": \"script\"",
            "\"value\": \"tools/diag-scripts/tooling/todo/todo-baseline.json\"",
            "\"lane\": \"smoke\"",
            "\"profile\": \"bounded\"",
            "\"devtools\"",
            "\"first-open\"",
        ] {
            assert!(
                IMUI_P2_DEVTOOLS_SMOKE_CAMPAIGN.contains(marker),
                "the bounded P2 devtools smoke campaign should keep the frozen manifest marker: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p2_discoverability_entry() {
        for marker in [
            "`docs/diagnostics-first-open.md`",
            "the canonical first-open diagnostics entry",
            "inspect -> selector -> script -> launched run -> bounded evidence -> compare/summarize/dashboard",
            "`docs/debugging-ui-with-inspector-and-scripts.md`",
            "`docs/ui-diagnostics-and-scripted-tests.md`",
            "`docs/workstreams/diag-fearless-refactor-v2/START_HERE.md`",
            "`docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`",
        ] {
            assert!(
                IMUI_P2_DISCOVERABILITY_ENTRY_NOTE.contains(marker),
                "the immediate-mode workstream should keep the P2 discoverability entry explicit: {marker}"
            );
        }

        for marker in [
            "This is the **default first-open diagnostics entry** for Fret.",
            "1. Inspect and pick one stable selector.",
            "2. Patch or choose one JSON script.",
            "3. Run the script into one explicit diagnostics artifacts root.",
            "4. Read bounded evidence first.",
            "5. Compare either one bundle pair or one aggregate root.",
            "`docs/workstreams/diag-fearless-refactor-v2/START_HERE.md` only after this page.",
        ] {
            assert!(
                DIAGNOSTICS_FIRST_OPEN_DOC.contains(marker),
                "the canonical diagnostics first-open doc should keep the frozen entry marker: {marker}"
            );
        }

        for marker in [
            "the canonical first-open diagnostics workflow now lives in",
            "`docs/diagnostics-first-open.md`",
            "maintainer/workstream navigation note",
        ] {
            assert!(
                DIAGNOSTICS_START_HERE_DOC.contains(marker),
                "the diagnostics start-here note should keep its branch/reference role explicit: {marker}"
            );
        }

        for marker in [
            "The canonical first-open diagnostics workflow now lives in",
            "`docs/diagnostics-first-open.md`",
            "this file is the DevTools GUI branch over the shared diagnostics",
        ] {
            assert!(
                DIAGNOSTICS_GUI_DOGFOOD_DOC.contains(marker),
                "the DevTools GUI dogfood note should keep its branch role explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_multiwindow_runner_gap_checklist() {
        for marker in [
            "`crates/fret-launch`",
            "`ecosystem/fret-docking`",
            "`crates/fret-ui` remains the mechanism layer",
            "hovered-window, peek-behind, transparent payload, and mixed-DPI follow-drag",
            "`cargo run -p fret-demo --bin workspace_shell_demo`",
            "P3 is a runner/backend closure problem by default, not an `imui` API backlog.",
            "The next real P3 slice should promote one bounded parity gate or diag suite",
        ] {
            assert!(
                IMUI_P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_NOTE.contains(marker),
                "the immediate-mode workstream should keep the P3 runner gap checklist explicit: {marker}"
            );
        }

        for marker in [
            "Hovered-window selection stays runner-owned",
            "Peek-behind while moving a tear-off window stays runner-owned",
            "Transparent payload overlap behavior stays runner-owned",
            "Mixed-DPI follow-drag correctness stays runner-owned",
        ] {
            assert!(
                IMUI_P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_NOTE.contains(marker),
                "the P3 checklist note should keep the frozen parity budget category: {marker}"
            );
        }

        for marker in [
            "hovered-window",
            "peek-behind",
            "transparent payload",
            "per-monitor DPI transitions while a drag is active",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_DOC.contains(marker),
                "the docking parity workstream should keep the runner-owned evidence marker: {marker}"
            );
        }

        for marker in [
            "Cross-window drag hover is stable",
            "No “stuck follow”",
            "Mouse-up outside any window still commits the drop",
        ] {
            assert!(
                MACOS_DOCKING_MULTIWINDOW_IMGUI_PARITY_DOC.contains(marker),
                "the macOS parity note should keep the hand-feel evidence marker: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_bounded_multiwindow_parity_package() {
        for marker in [
            "`tools/diag-campaigns/imui-p3-multiwindow-parity.json`",
            "`cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`",
            "`cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`",
            "`diag-hardening-smoke-docking` should stay small and general-purpose",
            "do not widen `crates/fret-ui` because one of these four checks is painful",
            "the fourth script is the current bounded mixed-DPI / large-coordinate stress entry",
            "Hovered-window selection under overlap",
            "Peek-behind under the moving tear-off window",
            "Transparent payload overlap behavior",
            "Mixed-DPI follow-drag expectation",
            "docking-arbitration-demo-multiwindow-overlap-zorder-switch.json",
            "docking-arbitration-demo-multiwindow-under-moving-window-basic.json",
            "docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json",
            "docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json",
        ] {
            assert!(
                IMUI_P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_NOTE.contains(marker),
                "the immediate-mode workstream should keep the P3 bounded parity package explicit: {marker}"
            );
        }

        for marker in [
            "\"id\": \"imui-p3-multiwindow-parity\"",
            "\"kind\": \"script\"",
            "docking-arbitration-demo-multiwindow-overlap-zorder-switch.json",
            "docking-arbitration-demo-multiwindow-under-moving-window-basic.json",
            "docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json",
            "docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json",
            "\"profile\": \"bounded\"",
            "\"runner-owned\"",
        ] {
            assert!(
                IMUI_P3_MULTIWINDOW_PARITY_CAMPAIGN.contains(marker),
                "the P3 campaign manifest should keep the bounded package marker: {marker}"
            );
        }

        for marker in [
            "Cross-window hover is stable",
            "hover selection can still target the window behind it",
            "temporary AlwaysOnTop while following",
            "No “stuck follow”",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_DOC.contains(marker)
                    || MACOS_DOCKING_MULTIWINDOW_IMGUI_PARITY_DOC.contains(marker),
                "the P3 package should keep the runner-owned evidence marker reachable: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_docking_parity_lane_resume_surface() {
        for marker in [
            "\"slug\": \"docking-multiwindow-imgui-parity\"",
            "\"status\": \"active\"",
            "M0_BASELINE_AUDIT_2026-04-13.md",
            "docking-multiwindow-imgui-parity-todo.md",
            "cargo run -p fret-demo --bin docking_arbitration_demo",
            "diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json",
            "\"default_action\": \"continue\"",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep an explicit first-open workstream state marker: {marker}"
            );
        }

        for marker in [
            "This lane is the current active execution lane for the remaining P3 multi-window hand-feel problem",
            "`DW-P0-dpi-006` is the smallest real open blocker in this lane",
            "do not reopen generic `imui` helper growth or widen `crates/fret-ui`",
            "`tools/diag-campaigns/imui-p3-multiwindow-parity.json` as the bounded P3 regression entry",
            "capture one real mixed-DPI acceptance pair",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_BASELINE_NOTE.contains(marker),
                "the docking parity baseline note should keep the explicit resume marker: {marker}"
            );
        }

        for marker in [
            "Status: Active execution lane",
            "authoritative first-open index",
            "WORKSTREAM.json",
            "M0_BASELINE_AUDIT_2026-04-13.md",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_DOC.contains(marker),
                "the docking parity narrative note should keep the first-open state visible: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_mixed_dpi_acceptance_posture() {
        for marker in [
            "\"role\": \"status\"",
            "M1_MIXED_DPI_ACCEPTANCE_POSTURE_2026-04-13.md",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the mixed-DPI acceptance posture reachable: {marker}"
            );
        }

        for marker in [
            "Keep the existing bounded P3 campaign generic",
            "`mixed_dpi_signal_observed` is evidence, not a host capability contract",
            "Do not add a new `requires mixed-dpi` campaign or script schema key yet",
            "real-host acceptance pair",
            "\"pre-crossing\" bundle",
            "\"post-crossing\" bundle",
            "`DW-P0-dpi-006` stays open until both the real-host acceptance pair",
            "the automation decision",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_NOTE.contains(marker),
                "the docking parity lane should keep the mixed-DPI acceptance posture explicit: {marker}"
            );
        }

        for marker in [
            "observed_scale_factors_x1000",
            "mixed_dpi_signal_observed",
            "Keep the bounded P3 campaign generic",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the mixed-DPI TODO posture should remain explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_mixed_dpi_capture_plan() {
        for marker in [
            "\"role\": \"next\"",
            "M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md",
            "mixed-dpi-real-host",
            "diag_pick_docking_mixed_dpi_acceptance_pair.py",
            "latest.acceptance-note.md",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the mixed-DPI capture plan reachable: {marker}"
            );
        }

        for marker in [
            "Windows native runner",
            "preferred setup: `100% + 150%`",
            "docking-arbitration-demo-multiwindow-drag-back-outer-pos-sweep.debug.json",
            "target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host",
            "multiwindow-drag-back-outer-sweep-after-tearoff",
            "multiwindow-drag-back-outer-sweep-after-outer-move-pos-x",
            "multiwindow-drag-back-outer-sweep-after-outer-move-neg-x",
            "latest.acceptance-summary.json",
            "latest.acceptance-note.md",
            "tools/diag_pick_docking_mixed_dpi_acceptance_pair.py",
            "--windows-version",
            "--canonical-command",
            "`mixed_dpi_signal_observed: true`",
            "`scale_factors_seen` with at least two distinct values",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_CAPTURE_PLAN_NOTE.contains(marker),
                "the docking parity lane should keep the mixed-DPI capture plan explicit: {marker}"
            );
        }

        for marker in [
            "Real-host capture runbook is now explicit",
            "M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md",
            "pre-crossing",
            "post-crossing",
            "diag_pick_docking_mixed_dpi_acceptance_pair.py",
            "`TODO`",
            "manual checklist",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the mixed-DPI TODO runbook should remain explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_two_surface_proof_budget_before_helper_widening() {
        for marker in [
            "Any future `fret-ui-kit::imui` public helper widening must name at least two real first-party proof",
            "For P0, the current minimum proof budget is the frozen immediate-mode golden pair:",
            "`apps/fret-cookbook/examples/imui_action_basics.rs`",
            "`apps/fret-examples/src/imui_editor_proof_demo.rs`",
            "Reference, advanced, or compatibility-only surfaces do not count by themselves.",
        ] {
            assert!(
                IMUI_PROOF_BUDGET_RULE_NOTE.contains(marker),
                "the immediate-mode workstream should keep the proof budget rule explicit: {marker}"
            );
        }
    }

    #[test]
    fn imui_hello_demo_is_explicitly_demoted_to_smoke_reference() {
        for marker in [
            "Reference/smoke demo: tiny IMUI hello surface.",
            "no longer the best",
            "first-contact teaching surface for the immediate-mode lane.",
            "Prefer `apps/fret-cookbook/examples/imui_action_basics.rs`",
            "`apps/fret-examples/src/imui_editor_proof_demo.rs`",
        ] {
            assert!(
                IMUI_HELLO_DEMO.contains(marker),
                "imui_hello_demo should stay explicitly demoted to smoke/reference: {marker}"
            );
        }
    }

    #[test]
    fn compatibility_only_node_graph_imui_demo_is_the_only_first_party_retained_compatibility_example()
     {
        for marker in [
            "Retained-bridge IMUI demo for `fret-node`.",
            "compatibility-oriented and should not be treated as the default downstream",
            "Prefer the declarative node-graph surfaces for normal downstream guidance.",
            "NodeGraphSurfaceCompatRetainedProps::new(",
            "node_graph_surface_compat_retained(",
        ] {
            assert!(
                IMUI_NODE_GRAPH_DEMO.contains(marker),
                "imui_node_graph_demo should stay explicitly labeled as the retained compatibility proof: {marker}"
            );
        }

        for (name, src) in [
            ("imui_hello_demo", IMUI_HELLO_DEMO),
            ("imui_floating_windows_demo", IMUI_FLOATING_WINDOWS_DEMO),
            (
                "imui_interaction_showcase_demo",
                IMUI_INTERACTION_SHOWCASE_DEMO,
            ),
            ("imui_response_signals_demo", IMUI_RESPONSE_SIGNALS_DEMO),
            ("imui_shadcn_adapter_demo", IMUI_SHADCN_ADAPTER_DEMO),
            ("imui_editor_proof_demo", IMUI_EDITOR_PROOF_DEMO),
        ] {
            for marker in [
                "retained_bridge::",
                "RetainedSubtreeProps",
                "UiTreeRetainedExt as _",
                "retained_subtree_with(",
                "fret_node::imui::",
            ] {
                assert!(
                    !src.contains(marker),
                    "{name} should not reintroduce retained-bridge authoring on the first-party imui teaching surface: {marker}"
                );
            }
        }
    }

    #[test]
    fn view_runtime_examples_prefer_app_ui_and_ui_aliases() {
        for src in [
            ASSETS_DEMO,
            ASYNC_PLAYGROUND_DEMO,
            CHART_DECLARATIVE_DEMO,
            CUSTOM_EFFECT_V1_DEMO,
            CUSTOM_EFFECT_V2_DEMO,
            CUSTOM_EFFECT_V3_DEMO,
            DROP_SHADOW_DEMO,
            EMBEDDED_VIEWPORT_DEMO,
            EXTERNAL_TEXTURE_IMPORTS_DEMO,
            EXTERNAL_VIDEO_IMPORTS_AVF_DEMO,
            EXTERNAL_VIDEO_IMPORTS_MF_DEMO,
            GENUI_DEMO,
            HELLO_COUNTER_DEMO,
            HELLO_WORLD_COMPARE_DEMO,
            IMAGE_HEAVY_MEMORY_DEMO,
            IMUI_EDITOR_PROOF_DEMO,
            IMUI_FLOATING_WINDOWS_DEMO,
            IMUI_HELLO_DEMO,
            IMUI_INTERACTION_SHOWCASE_DEMO,
            IMUI_NODE_GRAPH_DEMO,
            IMUI_RESPONSE_SIGNALS_DEMO,
            IMUI_SHADCN_ADAPTER_DEMO,
            LIQUID_GLASS_DEMO,
            MARKDOWN_DEMO,
            NODE_GRAPH_DEMO,
            POSTPROCESS_THEME_DEMO,
            QUERY_ASYNC_TOKIO_DEMO,
            QUERY_DEMO,
            TODO_DEMO,
        ] {
            assert_view_runtime_example_uses_app_ui_aliases(src);
        }
    }

    #[test]
    fn view_entry_examples_prefer_builder_then_run() {
        for src in [
            ASYNC_PLAYGROUND_DEMO,
            CHART_DECLARATIVE_DEMO,
            DROP_SHADOW_DEMO,
            GENUI_DEMO,
            HELLO_COUNTER_DEMO,
            IMUI_FLOATING_WINDOWS_DEMO,
            IMUI_HELLO_DEMO,
            IMUI_INTERACTION_SHOWCASE_DEMO,
            IMUI_NODE_GRAPH_DEMO,
            IMUI_RESPONSE_SIGNALS_DEMO,
            IMUI_SHADCN_ADAPTER_DEMO,
            MARKDOWN_DEMO,
            NODE_GRAPH_DEMO,
            QUERY_ASYNC_TOKIO_DEMO,
            QUERY_DEMO,
            TODO_DEMO,
        ] {
            assert_prefers_view_builder_then_run(src);
        }
    }

    #[test]
    fn app_facing_state_examples_prefer_grouped_data_surface() {
        for src in [QUERY_ASYNC_TOKIO_DEMO, QUERY_DEMO] {
            assert_prefers_grouped_data_surface(src);
        }
    }

    #[test]
    fn helper_heavy_examples_prefer_grouped_data_surface() {
        for src in [ASYNC_PLAYGROUND_DEMO, MARKDOWN_DEMO] {
            assert_prefers_grouped_data_surface(src);
        }
    }

    #[test]
    fn app_facing_query_examples_prefer_fret_query_facade() {
        for src in [
            ASYNC_PLAYGROUND_DEMO,
            MARKDOWN_DEMO,
            QUERY_ASYNC_TOKIO_DEMO,
            QUERY_DEMO,
        ] {
            assert_prefers_fret_query_facade(src);
        }
    }

    #[test]
    fn examples_source_tree_prefers_curated_shadcn_facade_imports() {
        for path in examples_rust_sources() {
            if path.ends_with("src/lib.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert!(!source.contains("use fret_ui_shadcn as shadcn;"));
            assert!(!source.contains("use fret_ui_shadcn::{self as shadcn"));

            for line in source.lines() {
                if !line.contains("fret_ui_shadcn::") {
                    continue;
                }

                let trimmed = line.trim();
                let allowed = matches!(
                    trimmed,
                    "use fret_ui_shadcn::facade as shadcn;"
                        | "use fret_ui_shadcn::{facade as shadcn, prelude::*};"
                );
                assert!(
                    allowed,
                    "{} reintroduced a non-curated fret_ui_shadcn import: {}",
                    path.display(),
                    trimmed
                );
            }
        }
    }

    #[test]
    fn examples_source_tree_limits_raw_shadcn_escape_hatches() {
        for path in examples_rust_sources() {
            if path.ends_with("src/lib.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            for (line_idx, line) in source.lines().enumerate() {
                let trimmed = line.trim();
                if !(trimmed.contains("shadcn::raw::") || trimmed.contains("fret::shadcn::raw::")) {
                    continue;
                }

                let allowed = trimmed.contains("shadcn::raw::typography::")
                    || trimmed.contains("shadcn::raw::extras::")
                    || trimmed.contains("fret::shadcn::raw::prelude::")
                    || trimmed.contains("shadcn::raw::advanced::sync_theme_from_environment(")
                    || trimmed
                        .contains("fret::shadcn::raw::advanced::sync_theme_from_environment(")
                    || trimmed.contains("shadcn::raw::advanced::install_with_ui_services(")
                    || trimmed.contains("fret::shadcn::raw::advanced::install_with_ui_services(");
                assert!(
                    allowed,
                    "{}:{} used an undocumented shadcn raw escape hatch: {}",
                    path.display(),
                    line_idx + 1,
                    trimmed
                );
            }
        }
    }

    #[test]
    fn examples_source_tree_avoids_raw_action_notify_helpers() {
        let mut raw_action_notify_files = Vec::new();

        for path in examples_rust_sources() {
            if path.ends_with("src/lib.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            let uses_raw_action_notify_trait =
                source.contains("use fret::advanced::AppUiRawActionNotifyExt as _;");
            let uses_raw_action_notify = source.contains("cx.on_action_notify::<");
            let uses_raw_payload_action_notify = source.contains("cx.on_payload_action_notify::<");

            if uses_raw_action_notify_trait
                || uses_raw_action_notify
                || uses_raw_payload_action_notify
            {
                raw_action_notify_files
                    .push(path.file_name().unwrap().to_string_lossy().into_owned());
            }
        }

        assert_eq!(raw_action_notify_files, Vec::<String>::new());
    }

    #[test]
    fn examples_source_tree_keeps_setup_on_named_installers() {
        for path in examples_rust_sources() {
            if path.ends_with("src/lib.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert_setup_surface_keeps_inline_closures_off_setup(&source);
        }
    }

    #[test]
    fn examples_source_tree_limits_setup_with_to_explicit_one_off_case() {
        for path in examples_rust_sources() {
            if path.ends_with("src/lib.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            let normalized = source.split_whitespace().collect::<String>();
            if !normalized.contains(".setup_with(") {
                continue;
            }

            assert_eq!(
                path.file_name().and_then(|name| name.to_str()),
                Some("imui_editor_proof_demo.rs"),
                "{} unexpectedly used setup_with(...)",
                path.display()
            );
            assert!(normalized.contains(".setup_with(move|"));
        }
    }

    #[test]
    fn advanced_entry_examples_prefer_view_elements_aliases() {
        for (src, state) in [
            (CUSTOM_EFFECT_V1_DEMO, "CustomEffectV1State"),
            (CUSTOM_EFFECT_V2_DEMO, "CustomEffectV2State"),
            (CUSTOM_EFFECT_V3_DEMO, "State"),
            (GENUI_DEMO, "GenUiState"),
            (LIQUID_GLASS_DEMO, "LiquidGlassState"),
        ] {
            assert_advanced_entry_prefers_view_elements_alias(src, state);
        }
    }

    #[test]
    fn app_facing_docking_examples_use_owning_fret_docking_crate() {
        for src in [CONTAINER_QUERIES_DOCKING_DEMO, DOCKING_DEMO] {
            assert!(src.contains("use fret_docking::{"));
            assert!(!src.contains("use fret::docking::{"));
        }
    }

    #[test]
    fn advanced_docking_harnesses_keep_raw_fret_docking_imports() {
        for src in [DOCKING_ARBITRATION_DEMO, IMUI_EDITOR_PROOF_DEMO] {
            assert!(src.contains("use fret_docking::{"));
        }
    }

    #[test]
    fn advanced_helper_contexts_prefer_uicx_aliases() {
        assert_advanced_helpers_prefer_uicx(
            ASSETS_DEMO,
            &[
                "fn render_view(cx: &mut UiCx<'_>) -> Ui",
                "fn assets_page<C>(cx: &mut UiCx<'_>, theme: &Theme, card: C) -> Ui",
                "C: IntoUiElement<KernelApp>",
                "fn render_image_panel(",
                "stats: fret_ui_assets::image_asset_cache::ImageAssetStats,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn render_svg_panel(",
                "svg: Option<fret_core::SvgId>,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
            ],
            &[
                "fn render_view(cx: &mut ElementContext<'_, KernelApp>) -> ViewElements",
                "let page = ui::container(|cx| {",
                "fn render_image_panel(cx: &mut UiCx<'_>, theme: &Theme, frame: u64, image: Option<fret_core::ImageId>, status: image_asset_state::ImageLoadingStatus, error: Option<Arc<str>>, stats: fret_ui_assets::image_asset_cache::ImageAssetStats) -> AnyElement",
                "fn render_svg_panel(cx: &mut UiCx<'_>, theme: &Theme, svg: Option<fret_core::SvgId>) -> AnyElement",
                "fn render_image_panel(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_svg_panel(cx: &mut ElementContext<'_, KernelApp>,",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            EMBEDDED_VIEWPORT_DEMO,
            &[
                "fn embedded_viewport_page<C>(",
                "cx: &mut UiCx<'_>,",
                "theme: ThemeSnapshot,",
                "viewport_card: C,",
                "diag: bool,",
                ") -> Ui",
                "C: IntoUiElement<KernelApp>,",
                "embedded_viewport_page(cx.elements(), theme, viewport_card, diag_enabled())",
                "ui::v_flex(move |cx| ui::single(cx, viewport_card))",
            ],
            &["ui::v_flex(move |cx| ui::children![cx; viewport_card])"],
        );

        assert_advanced_helpers_prefer_uicx(
            GENUI_DEMO,
            &[
                "fn genui_page<L, R>(cx: &mut UiCx<'_>, theme: ThemeSnapshot, left: L, right: R) -> Ui",
                "L: IntoUiElement<KernelApp>,",
                "R: IntoUiElement<KernelApp>,",
                "genui_page(cx, theme, left, right)",
            ],
            &["let page = ui::container(move |cx| {"],
        );

        assert_advanced_helpers_prefer_uicx(
            HELLO_WORLD_COMPARE_DEMO,
            &[
                "let swatch = |_cx: &mut UiCx<'_>, fill_rgb: u32, border_rgb: u32|",
                "fn hello_world_compare_root(",
                "cx: &mut UiCx<'_>,",
                "panel_bg: Color,",
                "children: Vec<AnyElement>)",
                ") -> Ui",
                "hello_world_compare_root(cx.elements(), panel_bg, children)",
            ],
            &[
                "let swatch = |cx: &mut ElementContext<'_, KernelApp>,",
                "let swatch = |cx: &mut UiCx<'_>, fill_rgb: u32, border_rgb: u32| -> AnyElement",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            IMAGE_HEAVY_MEMORY_DEMO,
            &["fn render_view(cx: &mut UiCx<'_>) -> Ui"],
            &["fn render_view(cx: &mut ElementContext<'_, KernelApp>) -> Ui"],
        );

        assert_advanced_helpers_prefer_uicx(
            IMUI_EDITOR_PROOF_DEMO,
            &[
                "fn render_view(cx: &mut UiCx<'_>) -> ViewElements",
                "fn render_authoring_parity_surface(cx: &mut UiCx<'_>,",
                "fn render_authoring_parity_shared_state(cx: &mut UiCx<'_>,",
                "fn render_authoring_parity_declarative_group(cx: &mut UiCx<'_>,",
                "fn render_authoring_parity_imui_group(cx: &mut UiCx<'_>,",
            ],
            &[
                "fn render_view(cx: &mut ElementContext<'_, KernelApp>) -> ViewElements",
                "fn render_authoring_parity_surface(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_authoring_parity_shared_state(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_authoring_parity_declarative_group(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_authoring_parity_imui_group(cx: &mut ElementContext<'_, KernelApp>,",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            MARKDOWN_DEMO,
            &["let spinner_box = |cx: &mut UiCx<'_>|"],
            &["let spinner_box = |cx: &mut fret_ui::ElementContext<'_, KernelApp>|"],
        );

        assert_advanced_helpers_prefer_uicx(
            CUSTOM_EFFECT_V1_DEMO,
            &[
                "fn watch_first_f32(cx: &mut UiCx<'_>,",
                "fn stage(cx: &mut UiCx<'_>, enabled: bool, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn lens_row(cx: &mut UiCx<'_>, enabled: bool, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn inspector(cx: &mut UiCx<'_>, st: &mut CustomEffectV1State, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> impl IntoUiElement<KernelApp> + use<>",
                "let label_row = |cx: &mut UiCx<'_>, label: &str, value: String|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut UiCx<'_>, enabled: bool, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> AnyElement",
                "fn lens_row(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_row(cx: &mut UiCx<'_>, enabled: bool, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut UiCx<'_>, st: &mut CustomEffectV1State, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> AnyElement",
                "let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            CUSTOM_EFFECT_V2_DEMO,
            &[
                "fn watch_first_f32(cx: &mut UiCx<'_>,",
                "fn stage(cx: &mut UiCx<'_>, enabled: bool, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn lens_row(cx: &mut UiCx<'_>, enabled: bool, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn inspector(cx: &mut UiCx<'_>, st: &mut CustomEffectV2State, sampling_value: &str, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32,) -> impl IntoUiElement<KernelApp> + use<>",
                "let label_row = |cx: &mut UiCx<'_>, label: &str, value: String|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut UiCx<'_>, enabled: bool, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool,) -> AnyElement",
                "fn lens_row(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_row(cx: &mut UiCx<'_>, enabled: bool, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool,) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut UiCx<'_>, st: &mut CustomEffectV2State, sampling_value: &str, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32,) -> AnyElement",
                "let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            CUSTOM_EFFECT_V3_DEMO,
            &[
                "fn stage(cx: &mut UiCx<'_>, st: &mut State, enabled: bool, show_user0_probe: bool, use_non_filterable_user0: bool, lens_effect: EffectId, user0_probe_effect: Option<EffectId>, show_user1_probe: bool, use_non_filterable_user1: bool, user1_probe_effect: Option<EffectId>, user01_probe_effect: Option<EffectId>, user0_image: Option<ImageId>, user1_image: Option<ImageId>,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn stage_controls(cx: &mut UiCx<'_>, st: &mut State, enabled: bool, show_user0_probe: bool, show_user1_probe: bool, use_non_filterable_user0: bool, use_non_filterable_user1: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn animated_backdrop(cx: &mut UiCx<'_>) -> impl IntoUiElement<KernelApp> + use<>",
                "fn lens_row(cx: &mut UiCx<'_>, enabled: bool, show_user0_probe: bool, lens_effect: EffectId, user0_probe_effect: Option<EffectId>, show_user1_probe: bool, user1_probe_effect: Option<EffectId>, user01_probe_effect: Option<EffectId>, user0_image: Option<ImageId>, user1_image: Option<ImageId>,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn lens_shell(cx: &mut UiCx<'_>, title: &'static str, radius: Px, lens_w: Px, lens_h: Px, with_effect: Option<EffectChain>,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn custom_effect_user01_probe_lens(cx: &mut UiCx<'_>,",
            ],
            &[
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut UiCx<'_>, st: &mut State, enabled: bool, show_user0_probe: bool, use_non_filterable_user0: bool, lens_effect: EffectId, user0_probe_effect: Option<EffectId>, show_user1_probe: bool, use_non_filterable_user1: bool, user1_probe_effect: Option<EffectId>, user01_probe_effect: Option<EffectId>, user0_image: Option<ImageId>, user1_image: Option<ImageId>,) -> AnyElement",
                "fn stage_controls(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage_controls(cx: &mut UiCx<'_>, st: &mut State, enabled: bool, show_user0_probe: bool, show_user1_probe: bool, use_non_filterable_user0: bool, use_non_filterable_user1: bool,) -> AnyElement",
                "fn animated_backdrop(cx: &mut ElementContext<'_, KernelApp>) -> AnyElement",
                "fn animated_backdrop(cx: &mut UiCx<'_>) -> AnyElement",
                "fn lens_row(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_row(cx: &mut UiCx<'_>, enabled: bool, show_user0_probe: bool, lens_effect: EffectId, user0_probe_effect: Option<EffectId>, show_user1_probe: bool, user1_probe_effect: Option<EffectId>, user01_probe_effect: Option<EffectId>, user0_image: Option<ImageId>, user1_image: Option<ImageId>,) -> AnyElement",
                "fn lens_shell(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_shell(cx: &mut UiCx<'_>, title: &'static str, radius: Px, lens_w: Px, lens_h: Px, with_effect: Option<EffectChain>,) -> AnyElement",
                "fn custom_effect_user01_probe_lens(cx: &mut ElementContext<'_, KernelApp>,",
            ],
        );

        assert_advanced_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V3_DEMO,
            &[
                "fn plain_lens(",
                "lens_h: Px,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn custom_effect_lens(",
                "effect: EffectId,",
                "fn custom_effect_user0_probe_lens(",
                "user0_image: ImageId,",
                "fn custom_effect_user1_probe_lens(",
                "user1_image: ImageId,",
                "fn custom_effect_user01_probe_lens(",
                "user0_image: ImageId,",
                "user1_image: ImageId,",
            ],
            &[
                "fn plain_lens(cx: &mut UiCx<'_>, title: &'static str, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
                "fn custom_effect_lens(cx: &mut UiCx<'_>, title: &'static str, effect: EffectId, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
                "fn custom_effect_user0_probe_lens(cx: &mut UiCx<'_>, title: &'static str, effect: EffectId, user0_image: ImageId, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
                "fn custom_effect_user1_probe_lens(cx: &mut UiCx<'_>, title: &'static str, effect: EffectId, user1_image: ImageId, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
                "fn custom_effect_user01_probe_lens(cx: &mut UiCx<'_>, title: &'static str, effect: EffectId, user0_image: ImageId, user1_image: ImageId, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            LIQUID_GLASS_DEMO,
            &[
                "fn watch_first_f32(cx: &mut UiCx<'_>,",
                "let mk_card = |cx: &mut UiCx<'_>,",
                "|cx: &mut UiCx<'_>, label: &str, value: String|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "let mk_card = |cx: &mut ElementContext<'_, KernelApp>,",
                "|cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            POSTPROCESS_THEME_DEMO,
            &[
                "fn watch_first_f32(cx: &mut UiCx<'_>,",
                "fn inspector(cx: &mut UiCx<'_>, st: &mut ThemePostprocessState, theme: &str, chromatic_offset_px: f32, scanline_strength: f32, scanline_spacing_px: f32, vignette_strength: f32, grain_strength: f32, grain_scale: f32, retro_pixel_scale: f32, retro_dither: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "let label_row = |cx: &mut UiCx<'_>, label: &str, value: String|",
                "fn stage(cx: &mut UiCx<'_>, enabled: bool, compare: bool, theme: &str, effect: EffectId, chromatic_offset_px: f32, scanline_strength: f32, scanline_spacing_px: f32, vignette_strength: f32, grain_strength: f32, grain_scale: f32, retro_pixel_scale: f32, retro_dither: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn stage_body(",
                "postprocess_applied: bool,",
                "label: &str,",
                "fn stage_cards(cx: &mut UiCx<'_>) -> impl IntoUiElement<KernelApp> + use<>",
                "let card = |cx: &mut UiCx<'_>, title: &str, subtitle: &str|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut UiCx<'_>, st: &mut ThemePostprocessState, theme: &str, chromatic_offset_px: f32, scanline_strength: f32, scanline_spacing_px: f32, vignette_strength: f32, grain_strength: f32, grain_scale: f32, retro_pixel_scale: f32, retro_dither: bool,) -> AnyElement",
                "let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut UiCx<'_>, enabled: bool, compare: bool, theme: &str, effect: EffectId, chromatic_offset_px: f32, scanline_strength: f32, scanline_spacing_px: f32, vignette_strength: f32, grain_strength: f32, grain_scale: f32, retro_pixel_scale: f32, retro_dither: bool,) -> AnyElement",
                "fn stage_body(cx: &mut UiCx<'_>, postprocess_applied: bool, label: &str) -> AnyElement",
                "fn stage_cards(cx: &mut UiCx<'_>) -> AnyElement",
                "fn stage_body(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage_cards(cx: &mut ElementContext<'_, KernelApp>) -> AnyElement",
                "let card = |cx: &mut ElementContext<'_, KernelApp>, title: &str, subtitle: &str|",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            ASYNC_PLAYGROUND_DEMO,
            &[
                "fn header_bar(cx: &mut UiCx<'_>,",
                "dark: bool,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn body(cx: &mut UiCx<'_>,",
                "selected: QueryId,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn catalog_panel(cx: &mut UiCx<'_>,",
                "selected: QueryId,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn main_panel(cx: &mut UiCx<'_>,",
                "global_slow: bool,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn inspector_panel(cx: &mut UiCx<'_>,",
                "fn policy_editor(cx: &mut UiCx<'_>,",
                "fn query_panel_for_mode(cx: &mut UiCx<'_>,",
                "mode: FetchMode,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn query_inputs_row(cx: &mut UiCx<'_>,",
                "fn query_result_view(cx: &mut UiCx<'_>,",
                "fn active_mode(cx: &mut UiCx<'_>, locals: &AsyncPlaygroundLocals) -> FetchMode",
                "fn status_badge(",
                "diag: Option<&QueryDiag>",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn snapshot_entry_for_key(cx: &mut UiCx<'_>,",
            ],
            &[
                "fn header_bar(cx: &mut ElementContext<'_, KernelApp>,",
                "fn header_bar(cx: &mut UiCx<'_>, st: &mut AsyncPlaygroundState, theme: ThemeSnapshot, global_slow: bool, dark: bool,) -> AnyElement",
                "fn body(cx: &mut ElementContext<'_, KernelApp>,",
                "fn body(cx: &mut UiCx<'_>, st: &mut AsyncPlaygroundState, theme: ThemeSnapshot, global_slow: bool, selected: QueryId,) -> AnyElement",
                "fn catalog_panel(cx: &mut ElementContext<'_, KernelApp>,",
                "fn catalog_panel(cx: &mut UiCx<'_>, st: &mut AsyncPlaygroundState, theme: ThemeSnapshot, selected: QueryId,) -> AnyElement",
                "fn main_panel(cx: &mut ElementContext<'_, KernelApp>,",
                "fn main_panel(cx: &mut UiCx<'_>, st: &mut AsyncPlaygroundState, theme: ThemeSnapshot, global_slow: bool, selected: QueryId,) -> AnyElement",
                "fn inspector_panel(cx: &mut UiCx<'_>, st: &mut AsyncPlaygroundState, theme: ThemeSnapshot, selected: QueryId,) -> AnyElement",
                "fn policy_editor(cx: &mut UiCx<'_>, st: &mut AsyncPlaygroundState, theme: ThemeSnapshot, id: QueryId,) -> AnyElement",
                "fn query_panel_for_mode(cx: &mut ElementContext<'_, KernelApp>,",
                "fn query_panel_for_mode(cx: &mut UiCx<'_>, st: &mut AsyncPlaygroundState, theme: ThemeSnapshot, global_slow: bool, selected: QueryId, mode: FetchMode,) -> AnyElement",
                "fn query_inputs_row(cx: &mut UiCx<'_>, st: &mut AsyncPlaygroundState, theme: ThemeSnapshot, id: QueryId,) -> AnyElement",
                "fn query_result_view(cx: &mut UiCx<'_>, theme: ThemeSnapshot, id: QueryId, key: QueryKey<Arc<str>>, state: &QueryState<Arc<str>>, snap: Option<&QuerySnapshotEntry>, policy: &QueryPolicy,) -> AnyElement",
                "fn active_mode(cx: &mut ElementContext<'_, KernelApp>, locals: &AsyncPlaygroundLocals) -> FetchMode",
                "fn status_badge(cx: &mut UiCx<'_>, diag: Option<&QueryDiag>) -> AnyElement",
                "fn status_badge(cx: &mut ElementContext<'_, KernelApp>, diag: Option<&QueryDiag>) -> AnyElement",
                "fn snapshot_entry_for_key(cx: &mut ElementContext<'_, KernelApp>,",
            ],
        );

        assert_advanced_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V1_DEMO,
            &[
                "fn lens_shell<B>(",
                "body: B,",
                ") -> impl IntoUiElement<KernelApp> + use<B>",
                "B: IntoUiElement<KernelApp>,",
                "body.into_element(cx)",
                "fn plain_lens<L>(",
                "label: L,",
                "radius: Px",
                ") -> impl IntoUiElement<KernelApp> + use<L>",
                "fn custom_effect_lens<L>(",
                "grain_scale: f32,",
                ") -> impl IntoUiElement<KernelApp> + use<L>",
            ],
            &[
                "fn lens_shell(cx: &mut UiCx<'_>, label: Arc<str>, radius: Px, body: AnyElement) -> AnyElement",
                "fn plain_lens(cx: &mut UiCx<'_>, label: impl Into<Arc<str>>, radius: Px) -> AnyElement",
                "fn custom_effect_lens(cx: &mut UiCx<'_>, label: impl Into<Arc<str>>, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32) -> AnyElement",
            ],
        );

        assert_advanced_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V2_DEMO,
            &[
                "fn lens_shell<B>(",
                "body: B,",
                ") -> impl IntoUiElement<KernelApp> + use<B>",
                "B: IntoUiElement<KernelApp>,",
                "body.into_element(cx)",
                "fn plain_lens<L>(",
                "label: L,",
                "radius: Px",
                ") -> impl IntoUiElement<KernelApp> + use<L>",
                "fn custom_effect_lens<L>(",
                "debug_input: bool,",
                ") -> impl IntoUiElement<KernelApp> + use<L>",
            ],
            &[
                "fn lens_shell(cx: &mut UiCx<'_>, label: Arc<str>, radius: Px, body: AnyElement) -> AnyElement",
                "fn plain_lens(cx: &mut UiCx<'_>, label: impl Into<Arc<str>>, radius: Px) -> AnyElement",
                "fn custom_effect_lens(cx: &mut UiCx<'_>, label: impl Into<Arc<str>>, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool) -> AnyElement",
            ],
        );

        assert_advanced_generic_helpers_prefer_into_ui_element(
            DROP_SHADOW_DEMO,
            &[
                "fn card<H: UiHost>(",
                "title: Arc<str>,",
                "subtitle: Arc<str>,",
                "enabled: bool",
                ") -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn card<H: UiHost>(cx: &mut ElementContext<'_, H>, title: Arc<str>, subtitle: Arc<str>, enabled: bool) -> AnyElement",
            ],
        );

        assert_advanced_generic_helpers_prefer_into_ui_element(
            MARKDOWN_DEMO,
            &[
                "fn render_image_placeholder<H: fret_ui::UiHost>(",
                "theme: fret_ui::ThemeSnapshot,",
                "on_link_activate: Option<markdown::OnLinkActivate>,",
                "link: markdown::LinkInfo,",
                ") -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn render_image_placeholder<H: fret_ui::UiHost>(cx: &mut fret_ui::ElementContext<'_, H>, theme: fret_ui::ThemeSnapshot, on_link_activate: Option<markdown::OnLinkActivate>, link: markdown::LinkInfo) -> AnyElement",
            ],
        );

        assert_advanced_generic_helpers_prefer_into_ui_element(
            LIQUID_GLASS_DEMO,
            &[
                "fn lens_panel<H: UiHost>(",
                "label: Arc<str>,",
                "radius: Px,",
                "mode: EffectMode,",
                "chain: EffectChain,",
                ") -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn lens_panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: Arc<str>, radius: Px, mode: EffectMode, chain: EffectChain) -> AnyElement",
            ],
        );

        assert_default_app_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V2_IDENTITY_WEB_DEMO,
            &[
                "fn stage_tile(",
                ") -> impl IntoUiElement<App> + use<>",
                "fn lens(cx: &mut ElementContext<'_, App>, view_settings: &CustomEffectV2IdentityWebViewSettings,) -> impl IntoUiElement<App> + use<>",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls, view_settings: &CustomEffectV2IdentityWebViewSettings,) -> impl IntoUiElement<App> + use<>",
                "items.push(Self::stage_tile(",
                ".into_element(cx),",
                "let view_settings = Self::view_settings(cx, &controls);",
                "let inspector = Self::inspector(cx, &controls, &inspector_settings).into_element(cx);",
                "Self::lens(cx, &stage_settings).into_element(cx)",
            ],
            &[
                "fn stage_tile(cx: &mut ElementContext<'_, App>, color: fret_core::Color, left: Px, top: Px, w: Px, h: Px, corner_radius_px: Px,) -> AnyElement",
                "fn lens(cx: &mut ElementContext<'_, App>, view_settings: &CustomEffectV2IdentityWebViewSettings) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls, view_settings: &CustomEffectV2IdentityWebViewSettings) -> AnyElement",
            ],
        );

        assert_default_app_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V2_WEB_DEMO,
            &[
                "fn stage_tile(",
                ") -> impl IntoUiElement<App> + use<>",
                "fn lens(cx: &mut ElementContext<'_, App>, view_settings: &CustomEffectV2WebViewSettings,) -> impl IntoUiElement<App> + use<>",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls, view_settings: &CustomEffectV2WebViewSettings,) -> impl IntoUiElement<App> + use<>",
                "items.push(Self::stage_tile(",
                ".into_element(cx),",
                "let view_settings = Self::view_settings(cx, &controls);",
                "let inspector = Self::inspector(cx, &controls, &inspector_settings).into_element(cx);",
                "Self::lens(cx, &stage_settings).into_element(cx)",
            ],
            &[
                "fn stage_tile(cx: &mut ElementContext<'_, App>, color: fret_core::Color, left: Px, top: Px, w: Px, h: Px, corner_radius_px: Px,) -> AnyElement",
                "fn lens(cx: &mut ElementContext<'_, App>, view_settings: &CustomEffectV2WebViewSettings) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls, view_settings: &CustomEffectV2WebViewSettings) -> AnyElement",
            ],
        );

        assert_default_app_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V2_LUT_WEB_DEMO,
            &[
                "fn stage_tile(",
                ") -> impl IntoUiElement<App> + use<>",
                "fn lens(cx: &mut ElementContext<'_, App>, view_settings: &CustomEffectV2LutWebViewSettings,) -> impl IntoUiElement<App> + use<>",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls, view_settings: &CustomEffectV2LutWebViewSettings,) -> impl IntoUiElement<App> + use<>",
                "items.push(Self::stage_tile(",
                ".into_element(cx),",
                "let view_settings = Self::view_settings(cx, &controls);",
                "let inspector = Self::inspector(cx, &controls, &inspector_settings).into_element(cx);",
                "Self::lens(cx, &stage_settings).into_element(cx)",
            ],
            &[
                "fn stage_tile(cx: &mut ElementContext<'_, App>, color: fret_core::Color, left: Px, top: Px, w: Px, h: Px, corner_radius_px: Px,) -> AnyElement",
                "fn lens(cx: &mut ElementContext<'_, App>, view_settings: &CustomEffectV2LutWebViewSettings) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls, view_settings: &CustomEffectV2LutWebViewSettings) -> AnyElement",
            ],
        );

        assert_default_app_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V2_GLASS_CHROME_WEB_DEMO,
            &[
                "fn label_row(cx: &mut ElementContext<'_, App>, label: &str, value: String,) -> impl IntoUiElement<App> + use<>",
                "fn stage_tile(",
                ") -> impl IntoUiElement<App> + use<>",
                "fn lens(cx: &mut ElementContext<'_, App>, view_settings: &CustomEffectV2GlassChromeWebViewSettings,) -> impl IntoUiElement<App> + use<>",
                "fn controls_panel(cx: &mut ElementContext<'_, App>, controls: &DemoControls, view_settings: &CustomEffectV2GlassChromeWebViewSettings,) -> impl IntoUiElement<App> + use<>",
                "Self::label_row(cx, \"Uv span\", format!(\"{uv_span:.2}\")).into_element(cx)",
                "items.push(Self::stage_tile(",
                ".into_element(cx),",
                "let view_settings = Self::view_settings(cx, &controls);",
                "let inspector = Self::controls_panel(cx, &controls, &inspector_settings).into_element(cx);",
                "items.push(Self::lens(cx, &stage_settings).into_element(cx));",
            ],
            &[
                "fn label_row(cx: &mut ElementContext<'_, App>, label: &str, value: String) -> AnyElement",
                "fn stage_tile(cx: &mut ElementContext<'_, App>, color: fret_core::Color, left: Px, top: Px, w: Px, h: Px, corner_radius_px: Px,) -> AnyElement",
                "fn lens(cx: &mut ElementContext<'_, App>, view_settings: &CustomEffectV2GlassChromeWebViewSettings) -> AnyElement",
                "fn controls_panel(cx: &mut ElementContext<'_, App>, controls: &DemoControls, view_settings: &CustomEffectV2GlassChromeWebViewSettings) -> AnyElement",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            ASYNC_PLAYGROUND_DEMO,
            &[
                "fn catalog_item(",
                "id: QueryId,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "out.push(catalog_item(cx, st, theme.clone(), selected, id).into_element(cx));",
            ],
            &[
                "fn catalog_item(cx: &mut UiCx<'_>, st: &mut AsyncPlaygroundState, theme: ThemeSnapshot, selected: QueryId, id: QueryId,) -> AnyElement",
                "out.push(catalog_item(cx, st, theme.clone(), selected, id));",
            ],
        );
    }

    #[test]
    fn selected_view_runtime_examples_prefer_grouped_state_actions_and_effects() {
        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            HELLO_COUNTER_DEMO,
            &[
                "let count_state = cx.state().local_init(|| 0i64);",
                "let step_state = cx.state().local_init(|| \"1\".to_string());",
                "let count = count_state.layout_value(cx);",
                "selector_layout(&step_state,",
                "parse_step(step_text.as_str())",
                ".locals_with((&count_state, &step_state))",
                ".on::<act::Inc>(|tx, (count_state, step_state)| {",
                ".on::<act::Dec>(|tx, (count_state, step_state)| {",
                "cx.actions().local(&count_state).set::<act::Reset>(0);",
            ],
            &[
                "cx.use_local_with(|| 0i64)",
                "cx.on_action_notify_models::<act::Inc>",
                "cx.on_action_notify_local_set::<act::Reset, i64>",
                "let count = count_state.layout(cx).value_or(0);",
                "let step_text = step_state.layout(cx).value_or_else(String::new);",
                "tx.value_or_else(&step_state, || \"1\".to_string())",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            QUERY_DEMO,
            &[
                "let fail_mode_state = cx.state().local_init(|| false);",
                "let fail_mode = fail_mode_state.layout_value(cx);",
                "let query_state = query_handle.read_layout(cx);",
                "let status_label = query_state.status.as_str();",
                "let info_line = if query_state.is_refreshing() {",
                "let error_color = if query_state.has_error() {",
                "if cx.effects().take_transient(TRANSIENT_INVALIDATE_KEY)",
                "cx.data().invalidate_query(demo_key());",
                "cx.data().invalidate_query_namespace(key.namespace());",
                "cx.actions().local(&fail_mode_state)",
                ".toggle_bool::<act::ToggleFailMode>();",
                "cx.actions().transient::<act::Invalidate>(TRANSIENT_INVALIDATE_KEY);",
            ],
            &[
                "with_query_client(",
                "cx.use_local_with(|| false)",
                "query_handle.layout(cx).value_or_default()",
                "let status_label = match query_state.status {",
                "QueryStatus::Loading if query_state.data.is_some() =>",
                "fail_mode_state.layout(cx).value_or_default()",
                "cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_KEY)",
                "cx.on_action_notify_toggle_local_bool::<act::ToggleFailMode>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            QUERY_ASYNC_TOKIO_DEMO,
            &[
                "let fail_mode_state = cx.state().local_init(|| false);",
                "let fail_mode = fail_mode_state.layout_value(cx);",
                "let query_state = query_handle.read_layout(cx);",
                "let status_label = query_state.status.as_str();",
                "let info_line = if query_state.is_refreshing() {",
                "let error_color = if query_state.has_error() {",
                "if cx.effects().take_transient(TRANSIENT_INVALIDATE_KEY)",
                "cx.data().invalidate_query(demo_key());",
                "cx.data().invalidate_query_namespace(key.namespace());",
                "cx.actions().local(&fail_mode_state)",
                ".toggle_bool::<act::ToggleFailMode>();",
                "cx.actions().transient::<act::Invalidate>(TRANSIENT_INVALIDATE_KEY);",
            ],
            &[
                "with_query_client(",
                "cx.use_local_with(|| false)",
                "query_handle.layout(cx).value_or_default()",
                "let status_label = match query_state.status {",
                "QueryStatus::Loading if query_state.data.is_some() =>",
                "fail_mode_state.layout(cx).value_or_default()",
                "cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_KEY)",
                "cx.on_action_notify_toggle_local_bool::<act::ToggleFailMode>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            TODO_DEMO,
            &[
                "struct TodoLocals {",
                "fn new(cx: &mut AppUi<'_, '_>) -> Self {",
                "struct TodoDemoView;",
                "let locals = TodoLocals::new(cx);",
                "locals.bind_actions(cx);",
                "let todos = locals.todos.layout_value(cx);",
                "let draft_value = locals.draft.layout_value(cx);",
                "draft: cx.state().local::<String>(),",
                "filter: cx.state().local_init(|| Some(Arc::<str>::from(TodoFilter::All.value()))),",
                "next_id: cx.state().local_init(|| 4u64),",
                "todos: cx.state().local_init(|| {",
                "let filter_value = TodoFilter::from_value(locals.filter.layout_value(cx).as_deref());",
                ".locals_with((&self.draft, &self.next_id, &self.todos))",
                ".on::<act::Add>(|tx, (draft, next_id, todos)| {",
                "let text = tx.value(&draft).trim().to_string();",
                "let id = tx.value(&next_id);",
                ".locals_with(&self.todos)",
                ".on::<act::ClearDone>(|tx, todos| {",
                "cx.actions().local(&self.todos)",
                ".payload_update_if::<act::Toggle>(|rows, id| {",
                ".payload_update_if::<act::Remove>(|rows, id| {",
                "fn todo_row<'a, Cx>(",
                "Cx: fret::app::ElementContextAccess<'a, App>,",
            ],
            &[
                "bind_todo_actions(",
                "cx.use_local::<String>()",
                "cx.on_action_notify_models::<act::Add>",
                "cx.on_payload_action_notify_local_update_if::<act::Toggle, Vec<TodoRow>>",
                "cx: &mut fret_ui::ElementContext<'_, App>,",
                "todos_state.layout(cx).value_or_default()",
                "draft_state.layout(cx).value_or_default()",
                "tx.value_or_else(&draft_state, String::new)",
                "tx.value_or(&next_id_state, 1)",
                "TodoLocals::new(app)",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            EMBEDDED_VIEWPORT_DEMO,
            &[
                "let size_preset_state = cx.state().local_init(|| Some(Arc::<str>::from(SIZE_PRESET_960)));",
                "let preset = size_preset_state.layout_value(cx);",
                "shadcn::ToggleGroup::single(&size_preset_state)",
                ".deselectable(false)",
            ],
            &[
                "cx.use_local_with(|| 1usize)",
                "cx.on_action_notify_local_set::<act::PickSize640, usize>",
                "let preset = size_preset_state.layout(cx).value_or_default();",
                "cx.actions().local(&size_preset_state).set::<act::PickSize640>(0);",
                ".disabled(preset == 0)",
            ],
        );
    }

    #[test]
    fn embedded_viewport_demo_models_size_presets_as_required_toggle_group() {
        let normalized = EMBEDDED_VIEWPORT_DEMO
            .split_whitespace()
            .collect::<String>();

        assert!(
            normalized.contains(
                &"shadcn::ToggleGroup::single(&size_preset_state)"
                    .split_whitespace()
                    .collect::<String>(),
            )
        );
        assert!(
            normalized.contains(
                &".deselectable(false)"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            normalized.contains(
                &"cx.state().local_init(|| Some(Arc::<str>::from(SIZE_PRESET_960)))"
                    .split_whitespace()
                    .collect::<String>(),
            )
        );
        assert!(
            !normalized.contains(
                &".disabled(preset == 0)"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(!normalized.contains(&"PickSize640".split_whitespace().collect::<String>()));
    }

    #[test]
    fn selected_advanced_examples_prefer_grouped_state_actions_and_effects() {
        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_HELLO_DEMO,
            &[
                "let count_state = cx.state().local_init(|| 0u32);",
                "let enabled_state = cx.state().local_init(|| false);",
                "let count = count_state.layout_value(cx);",
                "let enabled = enabled_state.paint_value(cx);",
            ],
            &[
                "cx.use_local_with(|| 0u32)",
                "cx.use_local_with(|| false)",
                "count_state.layout(cx).value_or_default()",
                "enabled_state.paint(cx).value_or_default()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            DROP_SHADOW_DEMO,
            &[
                "let enabled_state = cx.state().local_init(|| false);",
                "let stress_state = cx.state().local_init(|| false);",
                "let enabled = enabled_state.layout_value(cx);",
                "let stress = stress_state.layout_value(cx);",
                "shadcn::Switch::new(&enabled_state)",
                "shadcn::Switch::new(&stress_state)",
            ],
            &[
                "enabled: app.models_mut().insert(false)",
                "stress: app.models_mut().insert(false)",
                "self.st.enabled.layout(cx).value_or_default()",
                "self.st.stress.layout(cx).value_or_default()",
                "enabled_state.clone_model()",
                "stress_state.clone_model()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_FLOATING_WINDOWS_DEMO,
            &[
                "let open_a_state = cx.state().local_init(|| true);",
                "let select_mode_state = cx.state().local_init(|| None::<Arc<str>>);",
                "let a_overlap_clicked_state = cx.state().local_init(|| false);",
                "\"Window A\",",
                ".with_open(open_a_state.model())",
                "let clicked = a_overlap_clicked_state.paint_value_in(cx);",
                "\"Mode\",",
                "select_mode_state.model(),",
            ],
            &[
                "open_a: app.models_mut().insert(true)",
                "select_mode: app.models_mut().insert(None::<Arc<str>>)",
                "a_overlap_clicked: app.models_mut().insert(false)",
                "let clicked = a_overlap_clicked_state.paint_in(cx).value_or(false);",
                "&self.open_a",
                "read_model(",
                "&self.a_overlap_clicked",
                "&self.select_mode",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_RESPONSE_SIGNALS_DEMO,
            &[
                "let left_clicks = cx.state().local_init(|| 0u32);",
                "let drag_offset = cx.state().local_init(Point::default);",
                "let left_clicks_value = left_clicks.layout_value(cx);",
                "let drag_offset_value = drag_offset.layout_value(cx);",
                "let last_anchor_value = last_context_menu_anchor.layout_value(cx);",
            ],
            &[
                "cx.use_local_with(|| 0u32)",
                "cx.use_local_with(Point::default)",
                "left_clicks.layout(cx).value_or_default()",
                "drag_offset.layout(cx).value_or_default()",
                "last_context_menu_anchor.layout(cx).value_or_default()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_INTERACTION_SHOWCASE_DEMO,
            &[
                "let pulse_count = cx.state().local_init(|| 0u32);",
                "let autosave_enabled = cx.state().local_init(|| true);",
                "let selected_tab = cx.state().local_init(|| Some(Arc::<str>::from(\"overview\")));",
                "let pulse_count_value = pulse_count.layout_value(cx);",
                "let autosave_enabled_value = autosave_enabled.layout_value(cx);",
                "let selected_tab_value = selected_tab.layout_value(cx);",
                "let timeline_value = timeline.layout_value(cx);",
                "ui.switch_model(\"Autosave snapshots\", autosave_enabled.model())",
            ],
            &[
                "cx.use_local_with(|| 0u32)",
                "cx.use_local_with(|| true)",
                "pulse_count.layout(cx).value_or_default()",
                "autosave_enabled.layout(cx).value_or_default()",
                "selected_tab.layout(cx).value_or_default()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_SHADCN_ADAPTER_DEMO,
            &[
                "let count_state = cx.state().local_init(|| 0u32);",
                "let enabled_state = cx.state().local_init(|| false);",
                "let value_state = cx.state().local_init(|| 32.0f32);",
                "let mode_state = cx.state().local_init(|| None::<Arc<str>>);",
                "let draft_state = cx.state().local_init(String::new);",
                "let count = count_state.layout_value(cx);",
                "let enabled = enabled_state.paint_value(cx);",
                "let value = value_state.paint_value(cx);",
                "let mode = mode_state.paint_value(cx);",
                "let draft = draft_state.paint_value(cx);",
                "let _ = ui.switch_model(\"Enabled (switch)\", enabled_state.model());",
            ],
            &[
                "count: app.models_mut().insert(0)",
                "enabled: app.models_mut().insert(false)",
                "value: app.models_mut().insert(32.0)",
                "mode: app.models_mut().insert(None::<Arc<str>>)",
                "draft: app.models_mut().insert(String::new())",
                "let count = self.count.layout(cx).value_or_default();",
                "let enabled = self.enabled.paint(cx).value_or_default();",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            ASYNC_PLAYGROUND_DEMO,
            &[
                "use fret::app::RenderContextAccess as _;",
                "value: LocalState::new_in(app.models_mut(), initial.map(Arc::from)),",
                "open: LocalState::new_in(app.models_mut(), false),",
                "stale_time_s: LocalState::new_in(app.models_mut(), \"2\".to_string()),",
                "cache_time_s: LocalState::new_in(app.models_mut(), \"30\".to_string()),",
                "keep_prev: LocalState::new_in(app.models_mut(), true),",
                "fail_mode: LocalState::new_in(app.models_mut(), false),",
                "selected: cx.state().local_init(|| QueryId::Tip),",
                "dark: cx.state().local_init(|| false),",
                "global_slow: cx.state().local_init(|| false),",
                "tabs: cx.state().local_init(|| Some(Arc::<str>::from(\"async\"))),",
                "search_input: cx.state().local_init(|| \"react\".to_string()),",
                "stock_symbol: cx.state().local_init(|| \"FRET\".to_string()),",
                "let selected = locals.selected.layout_value(cx);",
                "let dark = locals.dark.layout_value(cx);",
                "let global_slow = locals.global_slow.layout_value(cx);",
                "let theme = cx.theme_snapshot();",
                "let namespace_input = locals.namespace_input.layout_value(cx);",
                ".locals_with((&locals.selected, &locals.namespace_input))",
                "cx.actions().local(&locals.dark)",
                ".toggle_bool::<act::ToggleTheme>();",
                "shadcn::Switch::new(&locals.global_slow)",
                "shadcn::Tabs::new(&locals.tabs)",
                "shadcn::Switch::new(&config.keep_prev)",
                "shadcn::Switch::new(&config.fail_mode)",
                "shadcn::Select::new(&config.cancel_mode.value, &config.cancel_mode.open)",
                "shadcn::Input::new(&locals.namespace_input)",
                "shadcn::Input::new(&locals.search_input)",
                "shadcn::Input::new(&locals.stock_symbol)",
                "if cx.effects().take_transient(TRANSIENT_INVALIDATE_SELECTED)",
                "cx.actions().transient::<act::InvalidateSelected>(TRANSIENT_INVALIDATE_SELECTED);",
            ],
            &[
                "selected: app.models_mut().insert(QueryId::Tip)",
                "dark: app.models_mut().insert(false)",
                "global_slow: app.models_mut().insert(false)",
                "tabs: app.models_mut().insert(Some(Arc::<str>::from(\"async\")))",
                "namespace_input: app.models_mut().insert(\"tip\".to_string())",
                "search_input: app.models_mut().insert(\"react\".to_string())",
                "stock_symbol: app.models_mut().insert(\"FRET\".to_string())",
                "LocalState::from_model(app.models_mut().insert(",
                "cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_SELECTED)",
                "cx.on_action_notify_models::<act::SelectTip>",
                "cx.actions().models::<act::SelectTip>({",
                "cx.actions().models::<act::ToggleTheme>({",
                "cx.on_action_notify_transient::<act::InvalidateSelected>",
                "shadcn::Switch::new(locals.global_slow.clone_model())",
                "shadcn::Tabs::new(locals.tabs.clone_model())",
                "shadcn::Switch::new(config.keep_prev.clone_model())",
                "shadcn::Switch::new(config.fail_mode.clone_model())",
                "Theme::global(&*cx.app).snapshot()",
                "config.cancel_mode.open.clone_model()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            EDITOR_NOTES_DEMO,
            &[
                "let theme = cx.theme_snapshot();",
                "fn selection_button<'a, Cx>(",
                "Cx: fret::app::ElementContextAccess<'a, App>,",
                ".into_element_in(cx)",
                "fn render_inspector_panel<'a, Cx>(",
                ".into_element_in(cx,",
                "let (name_value, committed_notes, notes_outcome) = cx.data().selector_model_paint(",
                "(&asset.name_model, &asset.notes_model, &asset.notes_outcome_model,)",
            ],
            &[
                "fn selection_button(cx: &mut AppUi<'_, '_>,",
                "fn render_inspector_panel(cx: &mut AppUi<'_, '_>,",
                ".watch_model(&asset.notes_model)",
                ".watch_model(&asset.notes_outcome_model)",
                "cx.elements()",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V1_DEMO,
            &[
                "enabled: cx.state().local_init(|| true),",
                "blur_radius_px: cx.state().local_init(|| vec![14.0]),",
                "blur_downsample: cx.state().local_init(|| vec![2.0]),",
                "refraction_height_px: cx.state().local_init(|| vec![20.0]),",
                "refraction_amount_px: cx.state().local_init(|| vec![12.0]),",
                "depth_effect: cx.state().local_init(|| vec![0.35]),",
                "chromatic_aberration: cx.state().local_init(|| vec![0.75]),",
                "corner_radius_px: cx.state().local_init(|| vec![20.0]),",
                "grain_strength: cx.state().local_init(|| vec![0.06]),",
                "grain_scale: cx.state().local_init(|| vec![1.0]),",
                "cx.actions().local(&st.enabled).set::<act::Reset>(true);",
                ".local(&st.blur_radius_px)",
                ".set::<act::Reset>(vec![14.0]);",
                ".local(&st.grain_scale)",
                ".set::<act::Reset>(vec![1.0]);",
            ],
            &[
                "enabled: app.models_mut().insert(true)",
                "blur_radius_px: app.models_mut().insert(vec![14.0])",
                "blur_downsample: app.models_mut().insert(vec![2.0])",
                "refraction_height_px: app.models_mut().insert(vec![20.0])",
                "refraction_amount_px: app.models_mut().insert(vec![12.0])",
                "depth_effect: app.models_mut().insert(vec![0.35])",
                "chromatic_aberration: app.models_mut().insert(vec![0.75])",
                "corner_radius_px: app.models_mut().insert(vec![20.0])",
                "grain_strength: app.models_mut().insert(vec![0.06])",
                "grain_scale: app.models_mut().insert(vec![1.0])",
                "cx.actions().models::<act::Reset>({",
                "cx.on_action_notify_models::<act::Reset>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V2_DEMO,
            &[
                "enabled: cx.state().local_init(|| true),",
                "use_non_filterable_input: cx.state().local_init(|| false),",
                "sampling: cx.state().local_init(|| Some(Arc::<str>::from(\"linear\"))),",
                "sampling_open: cx.state().local_init(|| false),",
                "uv_span: cx.state().local_init(|| vec![0.25]),",
                "input_strength: cx.state().local_init(|| vec![0.35]),",
                "rim_strength: cx.state().local_init(|| vec![0.65]),",
                "blur_radius_px: cx.state().local_init(|| vec![10.0]),",
                "debug_input: cx.state().local_init(|| false),",
                "cx.actions().local(&st.enabled).set::<act::Reset>(true);",
                ".local(&st.use_non_filterable_input)",
                ".set::<act::Reset>(false);",
                ".local(&st.sampling)",
                ".set::<act::Reset>(Some(Arc::<str>::from(\"linear\")));",
                ".local(&st.blur_radius_px)",
                ".set::<act::Reset>(vec![10.0]);",
                "cx.actions().local(&st.debug_input).set::<act::Reset>(false);",
            ],
            &[
                "enabled: app.models_mut().insert(true)",
                "use_non_filterable_input: app.models_mut().insert(false)",
                "sampling: app.models_mut().insert(Some(Arc::from(\"linear\")))",
                "sampling_open: app.models_mut().insert(false)",
                "uv_span: app.models_mut().insert(vec![0.25])",
                "input_strength: app.models_mut().insert(vec![0.35])",
                "rim_strength: app.models_mut().insert(vec![0.65])",
                "blur_radius_px: app.models_mut().insert(vec![10.0])",
                "debug_input: app.models_mut().insert(false)",
                "cx.actions().models::<act::Reset>({",
                "cx.on_action_notify_models::<act::Reset>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V3_DEMO,
            &[
                "enabled: cx.state().local_init(|| true),",
                "show_user0_probe: cx.state().local_init(|| false),",
                "show_user1_probe: cx.state().local_init(|| false),",
                "use_non_filterable_user0: cx.state().local_init(|| false),",
                "use_non_filterable_user1: cx.state().local_init(|| false),",
                "cx.actions().local(&st.enabled).set::<act::Reset>(true);",
                ".local(&st.show_user0_probe)",
                ".set::<act::Reset>(false);",
                ".local(&st.show_user1_probe)",
                ".local(&st.use_non_filterable_user0)",
                ".local(&st.use_non_filterable_user1)",
            ],
            &[
                "enabled: app.models_mut().insert(true)",
                "show_user0_probe: app.models_mut().insert(false)",
                "show_user1_probe: app.models_mut().insert(false)",
                "use_non_filterable_user0: app.models_mut().insert(false)",
                "use_non_filterable_user1: app.models_mut().insert(false)",
                "cx.actions().models::<act::Reset>({",
                "cx.on_action_notify_models::<act::Reset>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            POSTPROCESS_THEME_DEMO,
            &[
                "enabled: cx.state().local_init(|| true),",
                "compare: cx.state().local_init(|| true),",
                "theme: cx",
                ".local_init(|| Option::<Arc<str>>::Some(Arc::from(\"cyberpunk\"))),",
                "theme_open: cx.state().local_init(|| false),",
                "chromatic_offset_px: cx.state().local_init(|| vec![4.0]),",
                "scanline_strength: cx.state().local_init(|| vec![0.32]),",
                "scanline_spacing_px: cx.state().local_init(|| vec![3.0]),",
                "vignette_strength: cx.state().local_init(|| vec![0.6]),",
                "grain_strength: cx.state().local_init(|| vec![0.12]),",
                "grain_scale: cx.state().local_init(|| vec![1.5]),",
                "retro_pixel_scale: cx.state().local_init(|| vec![10.0]),",
                "retro_dither: cx.state().local_init(|| true),",
                "cx.actions().local(&st.enabled).set::<act::Reset>(true);",
                ".local(&st.theme)",
                ".set::<act::Reset>(Some(Arc::<str>::from(\"cyberpunk\")));",
                ".local(&st.chromatic_offset_px)",
                ".set::<act::Reset>(vec![4.0]);",
                "cx.actions().local(&st.retro_dither).set::<act::Reset>(true);",
            ],
            &[
                "enabled: app.models_mut().insert(true)",
                "compare: app.models_mut().insert(true)",
                ".insert(Option::<Arc<str>>::Some(Arc::from(\"cyberpunk\")))",
                "theme_open: app.models_mut().insert(false)",
                "chromatic_offset_px: app.models_mut().insert(vec![4.0])",
                "scanline_strength: app.models_mut().insert(vec![0.32])",
                "scanline_spacing_px: app.models_mut().insert(vec![3.0])",
                "vignette_strength: app.models_mut().insert(vec![0.6])",
                "grain_strength: app.models_mut().insert(vec![0.12])",
                "grain_scale: app.models_mut().insert(vec![1.5])",
                "retro_pixel_scale: app.models_mut().insert(vec![10.0])",
                "retro_dither: app.models_mut().insert(true)",
                "cx.actions().models::<act::Reset>({",
                "cx.on_action_notify_models::<act::Reset>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            LIQUID_GLASS_DEMO,
            &[
                "show_fake: cx.state().local_init(|| true),",
                "custom_v3_pair: cx.state().local_init(|| false),",
                "warp_strength_px: cx.state().local_init(|| vec![10.0]),",
                "cx.actions().local(&self.show_fake).set::<act::Reset>(true);",
                ".local(&self.custom_v3_bevel_secondary)",
                ".set::<act::Reset>(vec![1.0]);",
                ".local(&self.show_inspector)",
                ".toggle_bool::<act::ToggleInspector>();",
            ],
            &[
                "show_fake: app.models_mut().insert(true)",
                "custom_v3_pair: app.models_mut().insert(false)",
                "warp_strength_px: app.models_mut().insert(vec![10.0])",
                "cx.on_action_notify_models::<act::Reset>",
                "cx.on_action_notify_models::<act::ToggleInspector>",
                "cx.actions().models::<act::Reset>({",
                "cx.actions().models::<act::ToggleInspector>({",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            GENUI_DEMO,
            &[
                "cx.actions().transient::<act::ClearActions>(TRANSIENT_GENUI_CLEAR_ACTIONS);",
                "if cx.effects().take_transient(TRANSIENT_GENUI_CLEAR_ACTIONS)",
            ],
            &[
                "cx.on_action_notify_transient::<act::ClearActions>",
                "cx.take_transient_on_action_root(TRANSIENT_GENUI_CLEAR_ACTIONS)",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            MARKDOWN_DEMO,
            &[
                "if cx.effects().take_transient(TRANSIENT_REFRESH_REMOTE_IMAGES)",
                "cx.actions().transient::<act::RefreshRemoteImages>(TRANSIENT_REFRESH_REMOTE_IMAGES);",
                ".local(&expanded_code_blocks_state)",
                ".payload_update_if::<act::ToggleCodeBlockExpand>(|set, id| {",
                "let expanded_count = expanded_code_blocks_state.layout_read_ref(cx, |set| set.len());",
                "let pending = pending_anchor.layout_value(cx);",
                "let wrap_enabled = wrap_code_state.layout_value(cx);",
                "let cap_enabled = cap_code_height_state.layout_value(cx);",
                "components.on_link_activate = Some(Self::on_link_activate(pending_anchor_state.clone()));",
                "shadcn::Switch::new(&wrap_code_state)",
                "shadcn::Switch::new(&cap_code_height_state)",
            ],
            &[
                "cx.take_transient_on_action_root(TRANSIENT_REFRESH_REMOTE_IMAGES)",
                "cx.on_action_notify_transient::<act::RefreshRemoteImages>",
                "cx.on_payload_action_notify::<act::ToggleCodeBlockExpand>({",
                "let expanded_count = cx.data().selector(",
                "self.st.pending_anchor.layout(cx).value_or_default()",
                "self.st.wrap_code.layout(cx).value_or_default()",
                "self.st.cap_code_height.layout(cx).value_or_default()",
                "wrap_code_state.clone_model()",
                "cap_code_height_state.clone_model()",
            ],
        );
    }

    #[test]
    fn selected_element_context_examples_prefer_handle_first_tracked_model_reads() {
        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            TABLE_DEMO,
            &[
                "use fret::advanced::prelude::LocalState;",
                "view_options_open: LocalState<bool>,",
                "enable_grouping: LocalState<bool>,",
                "grouped_column_mode: LocalState<Option<Arc<str>>>,",
                "view_options_open: LocalState::new_in(app.models_mut(), false),",
                "grouped_column_mode: LocalState::new_in(",
                "let enable_grouping = enable_grouping.layout_value_in(cx);",
                "let grouped_column_mode = grouped_column_mode.layout_value_in(cx);",
                "let view_options_open = view_options_open.clone();",
                "shadcn::DropdownMenu::from_open(open).build(",
                "shadcn::DropdownMenuCheckboxItem::new(",
                "&enable_grouping,",
                "shadcn::DropdownMenuRadioGroup::new(",
                "&grouped_column_mode,",
                "shadcn::ContextMenu::from_open(open).into_element(",
            ],
            &[
                "use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;",
                "LocalState::from_model(app.models_mut().insert(",
                "cx.watch_model(&enable_grouping_model)",
                "cx.watch_model(&grouped_column_mode_model)",
                "view_options_open: Model<bool>,",
                "enable_grouping: Model<bool>,",
                "grouped_column_mode: Model<Option<Arc<str>>>,",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            ASYNC_PLAYGROUND_DEMO,
            &[
                "let selected = locals.selected.layout_value(cx);",
                "let dark = locals.dark.layout_value(cx);",
                "fn tracked_query_inputs(cx: &mut UiCx<'_>, locals: &AsyncPlaygroundLocals) -> QueryKeyInputs {",
                "let query_inputs = tracked_query_inputs(cx, &locals);",
                "locals.tabs.layout_read_ref_in(cx, |tab| match tab.as_deref() {",
                "let policy_settings: QueryPolicySettings = cx.data().selector_layout(",
                "config.fail_mode.layout_value_in(cx)",
            ],
            &[
                "let selected = cx.watch_model(&self.st.selected).layout().value_or_default();",
                "let dark = cx.watch_model(&self.st.dark).layout().value_or_default();",
                "let search = cx.watch_model(&st.search_input).layout().value_or_default();",
                "let symbol = cx.watch_model(&st.stock_symbol).layout().value_or_default();",
                "let tab = cx.watch_model(&st.tabs).layout().value_or_default();",
                "let stale_s = cx.watch_model(&config.stale_time_s).layout().value_or_default();",
                "cx.watch_model(&config.fail_mode)",
                "let search = st.search_input.layout_in(cx).value_or_default();",
                "let symbol = st.stock_symbol.layout_in(cx).value_or_default();",
                "let stale_s = config.stale_time_s.layout_in(cx).value_or_default();",
                "let cache_s = config.cache_time_s.layout_in(cx).value_or_default();",
                "let keep_prev = config.keep_prev.layout_in(cx).value_or_default();",
                "cx.data().selector_layout(&locals.tabs, |tab| match tab.as_deref() {",
                "let policy_settings: QueryPolicySettings = cx.data().selector(",
                "cx.data().selector_layout(&config.fail_mode, |fail_mode| fail_mode)",
                "config.fail_mode.layout_in(cx).value_or_default()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            GENUI_DEMO,
            &[
                "auto_apply_standard_actions: LocalState<bool>,",
                "auto_fix_on_apply: LocalState<bool>,",
                "editor_text: LocalState<String>,",
                "stream_text: LocalState<String>,",
                "stream_patch_only: LocalState<bool>,",
                "auto_apply_standard_actions: LocalState::new_in(app.models_mut(), true),",
                "editor_text: LocalState::new_in(app.models_mut(), SPEC_JSON.to_string()),",
                "stream_patch_only: LocalState::new_in(app.models_mut(), false),",
                "let auto_apply_enabled = st.auto_apply_standard_actions.layout_value_in(cx);",
                "let _auto_fix_enabled = st.auto_fix_on_apply.layout_value_in(cx);",
                "let auto_apply_model = st.auto_apply_standard_actions.clone_model();",
                "let auto_fix_model = st.auto_fix_on_apply.clone_model();",
                "st.genui_state",
                ".layout_in(cx)",
                ".read_ref(|v| {",
                "st.action_queue",
                "st.validation_state",
                "let stream_patch_only = st.stream_patch_only.layout_value_in(cx);",
                "let stream_patch_only_model = st.stream_patch_only.clone_model();",
                "shadcn::Textarea::new(&editor_model)",
                "shadcn::Textarea::new(&stream_model)",
            ],
            &[
                "auto_apply_standard_actions: Model<bool>,",
                "auto_fix_on_apply: Model<bool>,",
                "editor_text: Model<String>,",
                "stream_text: Model<String>,",
                "stream_patch_only: Model<bool>,",
                "LocalState::from_model(app.models_mut().insert(",
                "cx.watch_model(&st.auto_apply_standard_actions)",
                "cx.watch_model(&st.auto_fix_on_apply)",
                "cx.watch_model(&st.genui_state)",
                "cx.watch_model(&st.action_queue)",
                "cx.watch_model(&st.validation_state)",
                "cx.watch_model(&st.stream_patch_only)",
                "let auto_apply_model = st.auto_apply_standard_actions.clone();",
                "let auto_fix_model = st.auto_fix_on_apply.clone();",
                "let stream_patch_only_model = st.stream_patch_only.clone();",
                "shadcn::Textarea::new(editor_model.clone())",
                "shadcn::Textarea::new(stream_model.clone())",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V2_DEMO,
            &[
                "model.layout_read_ref_in(cx, |v| v.first().copied().unwrap_or(default))",
                "let view_settings: CustomEffectV2ViewSettings = cx.data().selector_layout(",
                "&st.enabled,",
                "&st.use_non_filterable_input,",
                "&st.sampling,",
                "&st.debug_input,",
                "let enabled_model = st.enabled.clone_model();",
                "let sampling_model = st.sampling.clone_model();",
                "let sampling_open_model = st.sampling_open.clone_model();",
                "let uv_span_state = st.uv_span.clone();",
                "let input_strength_state = st.input_strength.clone();",
                "let rim_strength_state = st.rim_strength.clone();",
                "let blur_radius_state = st.blur_radius_px.clone();",
            ],
            &[
                "let view_settings: CustomEffectV2ViewSettings = cx.data().selector(",
                "cx.watch_model(model)",
                "let enabled = cx.watch_model(&st.enabled).layout().value_or(true);",
                "let use_non_filterable_input = cx.watch_model(&st.use_non_filterable_input)",
                "let debug_input = cx.watch_model(&st.debug_input).layout().value_or(false);",
                "let enabled = st.enabled.layout_in(cx).value_or(true);",
                "let use_non_filterable_input = st.use_non_filterable_input.layout_in(cx).value_or(false);",
                "let sampling_value = st",
                "let debug_input = st.debug_input.layout_in(cx).value_or(false);",
                "let enabled_model = st.enabled.clone();",
                "let sampling_model = st.sampling.clone();",
                "let sampling_open_model = st.sampling_open.clone();",
                "let uv_span_model = st.uv_span.clone_model();",
                "let input_strength_model = st.input_strength.clone_model();",
                "let rim_strength_model = st.rim_strength.clone_model();",
                "let blur_radius_model = st.blur_radius_px.clone_model();",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V2_WEB_DEMO,
            &[
                "let theme = cx.theme().snapshot();",
                "fn view_settings(",
                "-> CustomEffectV2WebViewSettings {",
                "fn reset_in(&self, models: &mut fret_runtime::ModelStore) {",
                "cx.data().selector_model_paint(",
                "&controls.enabled,",
                "&controls.tile_corner_radius_px,",
                "reset_controls.reset_in(host.models_mut());",
                "state.controls.reset_in(app.models_mut());",
                "let view_settings = Self::view_settings(cx, &controls);",
            ],
            &[
                "cx.data().selector(",
                "cx.observe_model(&enabled_deps, Invalidation::Paint);",
                "cx.app.models().revision(&enabled_deps).unwrap_or(0)",
                "cx.app.models().get_cloned(&enabled).unwrap_or(true)",
                "cx.watch_model(model)",
                "model.paint_in(cx).read_ref(|v| v.first().copied().unwrap_or(default))",
                "model.paint_in(cx).read_ref(|v| v.as_ref().map(|s| s.to_string()))",
                "CustomEffectV2WebDriver::reset_controls(app, &state.controls);",
                "let _ = models.update(&reset_controls.enabled, |v| *v = true);",
                "let enabled = controls.enabled.paint_in(cx).value_or(true);",
                "let debug_input = controls.debug_input.paint_in(cx).value_or(false);",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V2_GLASS_CHROME_WEB_DEMO,
            &[
                "let theme = cx.theme().snapshot();",
                "fn view_settings(",
                "-> CustomEffectV2GlassChromeWebViewSettings {",
                "fn reset_in(&self, models: &mut fret_runtime::ModelStore) {",
                "cx.data().selector_model_paint(",
                "&controls.enabled,",
                "&controls.debug_input,",
                "reset_controls.reset_in(host.models_mut());",
                "state.controls.reset_in(app.models_mut());",
                "let view_settings = Self::view_settings(cx, &controls);",
            ],
            &[
                "cx.data().selector(",
                "cx.observe_model(&enabled_deps, Invalidation::Paint);",
                "cx.app.models().revision(&enabled_deps).unwrap_or(0)",
                "cx.app.models().get_cloned(&enabled).unwrap_or(true)",
                "cx.watch_model(model)",
                "model.paint_in(cx).read_ref(|v| v.first().copied().unwrap_or(default))",
                "model.paint_in(cx).read_ref(|v| v.as_ref().map(|s| s.to_string()))",
                "CustomEffectV2GlassChromeWebDriver::reset_controls(app, &state.controls);",
                "let _ = models.update(&reset_controls.enabled, |v| *v = true);",
                "let enabled = controls.enabled.paint_in(cx).value_or(true);",
                "let debug_input = controls.debug_input.paint_in(cx).value_or(false);",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V2_IDENTITY_WEB_DEMO,
            &[
                "let theme = cx.theme().snapshot();",
                "fn view_settings(",
                "-> CustomEffectV2IdentityWebViewSettings {",
                "fn reset_in(&self, models: &mut fret_runtime::ModelStore) {",
                "cx.data().selector_model_paint(",
                "&controls.enabled,",
                "&controls.debug_input,",
                "reset_controls.reset_in(host.models_mut());",
                "state.controls.reset_in(app.models_mut());",
                "let view_settings = Self::view_settings(cx, &controls);",
            ],
            &[
                "cx.data().selector(",
                "cx.observe_model(&enabled_deps, Invalidation::Paint);",
                "cx.app.models().revision(&enabled_deps).unwrap_or(0)",
                "cx.app.models().get_cloned(&enabled).unwrap_or(true)",
                "cx.watch_model(model)",
                "model.paint_in(cx).read_ref(|v| v.first().copied().unwrap_or(default))",
                "model.paint_in(cx).read_ref(|v| v.as_ref().map(|s| s.to_string()))",
                "CustomEffectV2IdentityWebDriver::reset_controls(app, &state.controls);",
                "let _ = models.update(&reset_controls.enabled, |v| *v = true);",
                "let enabled = controls.enabled.paint_in(cx).value_or(true);",
                "let debug_input = controls.debug_input.paint_in(cx).value_or(false);",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V2_LUT_WEB_DEMO,
            &[
                "let theme = cx.theme().snapshot();",
                "fn view_settings(",
                "-> CustomEffectV2LutWebViewSettings {",
                "fn reset_in(&self, models: &mut fret_runtime::ModelStore) {",
                "cx.data().selector_model_paint(",
                "&controls.enabled,",
                "&controls.tile_corner_radius_px,",
                "reset_controls.reset_in(host.models_mut());",
                "state.controls.reset_in(app.models_mut());",
                "let view_settings = Self::view_settings(cx, &controls);",
            ],
            &[
                "cx.data().selector(",
                "cx.observe_model(&enabled_deps, Invalidation::Paint);",
                "cx.app.models().revision(&enabled_deps).unwrap_or(0)",
                "cx.app.models().get_cloned(&enabled).unwrap_or(true)",
                "cx.watch_model(model)",
                "model.paint_in(cx).read_ref(|v| v.first().copied().unwrap_or(default))",
                "model.paint_in(cx).read_ref(|v| v.as_ref().map(|s| s.to_string()))",
                "CustomEffectV2LutWebDriver::reset_controls(app, &state.controls);",
                "let _ = models.update(&reset_controls.enabled, |v| *v = true);",
                "let enabled = controls.enabled.paint_in(cx).value_or(true);",
                "let debug_input = controls.debug_input.paint_in(cx).value_or(false);",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_EDITOR_PROOF_DEMO,
            &[
                "struct EditorTextAssistReadout {",
                "struct EditorTextFieldReadout {",
                "struct AuthoringParitySharedStateReadout {",
                "fn editor_text_assist_readout(",
                "fn editor_text_field_readout(",
                "fn editor_string_model_readout(",
                "cx.data().selector_model_paint(",
                "(query_model, dismissed_query_model, active_item_id_model),",
                "(committed_model, outcome_model),",
                "editor_string_model_readout(",
                "let name_assist_items = editor_demo_name_assist_items(cx);",
                "let name_assist_readout = editor_text_assist_readout(",
                "let inline_rename_readout = editor_text_field_readout(",
                "let password_readout = editor_text_field_readout(",
                "let notes_readout = editor_text_field_readout(",
                ".selector_model_paint(&editor_gradient_stops_model,",
                ".selector_model_paint(&m.target, |target| target)",
                "let shared = cx.data().selector_model_paint(",
                "(&name_model, &drag_value_model, &numeric_input_model, &slider_model, &enabled_model, &shading_model, &gradient_angle_model, &gradient_stops_model,)",
            ],
            &[
                "let query = cx.watch_model(&editor_name_assist_model)",
                "let dismissed_query = cx.watch_model(&editor_name_assist_dismissed_query_model)",
                "let active_item_id = cx.watch_model(&editor_name_assist_active_item_model)",
                "cx.watch_model(&editor_buffered_name_model)",
                "cx.watch_model(&editor_name_assist_accepted_model)",
                "cx.watch_model(&editor_inline_rename_model)",
                "cx.watch_model(&editor_inline_rename_outcome_model)",
                "cx.watch_model(&editor_password_model)",
                "cx.watch_model(&editor_password_outcome_model)",
                "cx.watch_model(&editor_notes_model)",
                "cx.watch_model(&editor_notes_outcome_model)",
                "watch_model(&editor_gradient_stops_model)",
                "watch_model(&m.target)",
                ".get_model_cloned(&editor_drag_value_outcome_model,",
                ".get_model_cloned(&editor_position_outcome_model,",
                ".get_model_cloned(&editor_transform_outcome_model,",
                ".get_model_cloned(&name_model, fret_ui::Invalidation::Paint)",
                ".get_model_copied(&drag_value_model, fret_ui::Invalidation::Paint)",
                ".get_model_copied(&numeric_input_model, fret_ui::Invalidation::Paint)",
                ".get_model_copied(&slider_model, fret_ui::Invalidation::Paint)",
                ".get_model_copied(&enabled_model, fret_ui::Invalidation::Paint)",
                ".get_model_cloned(&shading_model, fret_ui::Invalidation::Paint)",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V3_DEMO,
            &[
                "let view_settings: CustomEffectV3ViewSettings = cx.data().selector_layout(",
                "&st.enabled,",
                "&st.show_user0_probe,",
                "&st.use_non_filterable_user1,",
                "let enabled_model = st.enabled.clone_model();",
                "let show_user0_probe_model = st.show_user0_probe.clone_model();",
            ],
            &[
                "let enabled = cx.watch_model(&st.enabled).layout().value_or(true);",
                "let show_user0_probe = cx.watch_model(&st.show_user0_probe)",
                "let use_non_filterable_user1 = cx.watch_model(&st.use_non_filterable_user1)",
                "let view_settings: CustomEffectV3ViewSettings = cx.data().selector(",
                "let enabled = st.enabled.layout_in(cx).value_or(true);",
                "let show_user0_probe = st.show_user0_probe.layout_in(cx).value_or(false);",
                "let show_user1_probe = st.show_user1_probe.layout_in(cx).value_or(false);",
                "let use_non_filterable_user0 = st.use_non_filterable_user0.layout_in(cx).value_or(false);",
                "let use_non_filterable_user1 = st.use_non_filterable_user1.layout_in(cx).value_or(false);",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V1_DEMO,
            &[
                "model.layout_read_ref_in(cx, |v| v.first().copied().unwrap_or(default))",
                "let enabled = st.enabled.layout_value_in(cx);",
                "let enabled_model = st.enabled.clone_model();",
                "let blur_radius_state = st.blur_radius_px.clone();",
            ],
            &[
                "cx.watch_model(model)",
                "let enabled = cx.watch_model(&st.enabled).layout().value_or(true);",
                "let enabled_model = st.enabled.clone();",
                "let blur_radius_model = st.blur_radius_px.clone_model();",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            POSTPROCESS_THEME_DEMO,
            &[
                "let view_settings: ThemePostprocessViewSettings = cx.data().selector_layout(",
                "&st.enabled,",
                "&st.compare,",
                "&st.theme,",
                "&st.retro_dither",
                "let enabled_model = st.enabled.clone_model();",
                "let theme_model = st.theme.clone_model();",
                "let theme_open_model = st.theme_open.clone_model();",
                "let chromatic_state = st.chromatic_offset_px.clone();",
            ],
            &[
                "let view_settings: ThemePostprocessViewSettings = cx.data().selector(",
                "let enabled = cx.watch_model(&self.st.enabled).layout().value_or(true);",
                "let compare = cx.watch_model(&self.st.compare).layout().value_or(true);",
                "let theme = cx.watch_model(&self.st.theme).layout().value_or(Option::<Arc<str>>::None);",
                "let retro_dither = cx.watch_model(&self.st.retro_dither).layout().value_or(true);",
                "let enabled = self.st.enabled.layout_in(cx).value_or(true);",
                "let compare = self.st.compare.layout_in(cx).value_or(true);",
                "let theme = self.st.theme.layout_in(cx).value_or(Option::<Arc<str>>::None);",
                "let retro_dither = self.st.retro_dither.layout_in(cx).value_or(true);",
                "let enabled_model = st.enabled.clone();",
                "let theme_model = st.theme.clone();",
                "let theme_open_model = st.theme_open.clone();",
                "let chromatic_model = st.chromatic_offset_px.clone_model();",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            LIQUID_GLASS_DEMO,
            &[
                "model.layout_read_ref_in(cx, |v| v.first().copied().unwrap_or(default))",
                "let visibility_settings: LiquidGlassVisibilitySettings = cx.data().selector_layout(",
                "&st.show_fake,",
                "&st.custom_v3_pair,",
                "let mode_settings: LiquidGlassModeSettings = cx.data().selector_layout(",
                "&st.custom_v3_source_group,",
                "let show_fake_switch_model = st.show_fake.clone_model();",
                "let lens_radius_state = st.lens_radius_px.clone();",
            ],
            &[
                "cx.watch_model(model)",
                "let visibility_settings: LiquidGlassVisibilitySettings = cx.data().selector(",
                "let mode_settings: LiquidGlassModeSettings = cx.data().selector(",
                "cx.observe_model(&show_fake_model, Invalidation::Layout);",
                "let lens_radius_model = st.lens_radius_px.clone_model();",
                "let show_fake = st.show_fake.layout_in(cx).value_or(true);",
                "let show_warp = st.show_warp.layout_in(cx).value_or(true);",
                "let show_warp_v2 = st.show_warp_v2.layout_in(cx).value_or(false);",
                "let show_custom_v2 = st.show_custom_v2.layout_in(cx).value_or(false);",
                "let show_custom_v3 = st.show_custom_v3.layout_in(cx).value_or(false);",
                "let custom_v3_pair = st.custom_v3_pair.layout_in(cx).value_or(false);",
                "let custom_v3_source_group = st.custom_v3_source_group.layout_in(cx).value_or(false);",
                "let show_inspector = st.show_inspector.layout_in(cx).value_or(true);",
                "let animate = st.animate.layout_in(cx).value_or(true);",
                "let use_backdrop = st.use_backdrop.layout_in(cx).value_or(true);",
                "let use_dither = st.use_dither.layout_in(cx).value_or(true);",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            LAUNCHER_UTILITY_WINDOW_DEMO,
            &[
                "always_on_top: LocalState<bool>,",
                "status: LocalState<Arc<str>>,",
                "LocalState::new_in(app.models_mut(), false)",
                "LocalState::new_in(app.models_mut(), Arc::from(\"Idle\"))",
                "let view_settings: LauncherUtilityWindowViewSettings = cx.data().selector_layout(",
                "(&st.always_on_top, &st.status),",
            ],
            &[
                "let always_on_top = cx.watch_model(&st.always_on_top).layout().value_or(false);",
                "cx.watch_model(&st.status)",
                "let always_on_top = st.always_on_top.layout_in(cx).value_or(false);",
                "let status = st.status.layout_in(cx).value_or_else(|| Arc::from(\"Idle\"));",
                "always_on_top: fret_runtime::Model<bool>,",
                "status: fret_runtime::Model<Arc<str>>,",
                "LocalState::from_model(app.models_mut().insert(",
                "let view_settings: LauncherUtilityWindowViewSettings = cx.data().selector(",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            WINDOW_HIT_TEST_PROBE_DEMO,
            &["let status = st.status.layout_in(cx).value_or_else(|| Arc::from(\"Idle\"));"],
            &["cx.watch_model(&st.status)"],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            LAUNCHER_UTILITY_WINDOW_MATERIALS_DEMO,
            &[
                "status: LocalState<Arc<str>>,",
                "LocalState::new_in(app.models_mut(), Arc::from(\"Idle\"))",
                "let status = st.status.layout_value_in(cx);",
            ],
            &[
                "cx.watch_model(&st.status)",
                "let status = st.status.layout_in(cx).value_or_else(|| Arc::from(\"Idle\"));",
                "let status = cx.data().selector_layout(&st.status, |status| status);",
                "status: fret_runtime::Model<Arc<str>>,",
                "LocalState::from_model(app.models_mut().insert(",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            WORKSPACE_SHELL_DEMO,
            &[
                "fn workspace_shell_command_button<'a, Cx>(",
                "Cx: fret::app::ElementContextAccess<'a, App>,",
                "let cx = cx.elements();",
                "let (prompt_open, prompt): (bool, Option<WorkspaceShellDirtyClosePrompt>) =",
                "cx.data().selector_model_layout(",
                "(&dirty_close_prompt_open, &dirty_close_prompt),",
                ".selector_model_layout(&tabstrip_two_row_pinned, |two_row_pinned| {",
            ],
            &[
                "let button = |cx: &mut fret_ui::ElementContext<'_, App>,",
                ".get_model_cloned(&dirty_close_prompt_open, Invalidation::Layout)",
                ".get_model_cloned(&dirty_close_prompt, Invalidation::Layout)",
                ".get_model_cloned(&tabstrip_two_row_pinned, Invalidation::Layout)",
            ],
        );
    }

    #[test]
    fn workspace_shell_demo_prefers_capability_first_command_button_helpers() {
        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            WORKSPACE_SHELL_DEMO,
            &[
                "fn workspace_shell_command_button<'a, Cx>(",
                "Cx: fret::app::ElementContextAccess<'a, App>,",
                "let cx = cx.elements();",
                "workspace_shell_command_button(",
            ],
            &["let button = |cx: &mut fret_ui::ElementContext<'_, App>,"],
        );
    }

    #[test]
    fn workspace_shell_demo_prefers_capability_first_editor_rail_helpers() {
        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            WORKSPACE_SHELL_DEMO,
            &[
                "fn workspace_shell_editor_rail<'a, Cx>(",
                "Cx: fret::app::ElementContextAccess<'a, App>,",
                "workspace_shell_editor_rail(",
                "InspectorPanel::new(None)",
                ".into_element_in(cx,",
                "PropertyGrid::new().into_element_in(cx,",
            ],
            &["fn workspace_shell_editor_rail(cx: &mut fret_ui::ElementContext<'_, App>,"],
        );
    }

    #[test]
    fn table_examples_prefer_local_state_menu_bridges_over_clone_model() {
        assert!(TABLE_DEMO.contains("table_state: LocalState<TableState>,"));
        assert!(
            TABLE_DEMO
                .contains("let table_state = LocalState::new_in(app.models_mut(), table_state);")
        );
        assert!(
            TABLE_DEMO
                .contains("let (selected, sorting) = table_state.layout_read_ref_in(cx, |st| {")
        );
        assert!(TABLE_DEMO.contains("fret_ui_kit::declarative::table::table_virtualized("));
        assert!(TABLE_DEMO.contains("&table_state,"));
        assert!(TABLE_DEMO.contains("let view_options_open = view_options_open.clone();"));
        assert!(TABLE_DEMO.contains("let header_menu_id_open = header_menu_id_open.clone();"));
        assert!(TABLE_DEMO.contains("let header_menu_name_open = header_menu_name_open.clone();"));
        assert!(TABLE_DEMO.contains("let header_menu_role_open = header_menu_role_open.clone();"));
        assert!(
            TABLE_DEMO.contains("let header_menu_score_open = header_menu_score_open.clone();")
        );
        assert!(TABLE_DEMO.contains("shadcn::DropdownMenu::from_open(open).build("));
        assert!(TABLE_DEMO.contains("shadcn::DropdownMenuCheckboxItem::new("));
        assert!(TABLE_DEMO.contains("&enable_grouping,"));
        assert!(TABLE_DEMO.contains("shadcn::DropdownMenuRadioGroup::new("));
        assert!(TABLE_DEMO.contains("&grouped_column_mode,"));
        assert!(TABLE_DEMO.contains("shadcn::ContextMenu::from_open(open).into_element("));
        assert!(TABLE_DEMO.contains(
            "let table_debug_ids =\n                                                    fret_ui_kit::declarative::table::TableDebugIds {"
        ));
        assert!(TABLE_DEMO.contains(
            "header_row_test_id: Some(Arc::<str>::from(\n                                                            \"table-demo-header-row\","
        ));
        assert!(TABLE_DEMO.contains(
            "header_cell_test_id_prefix: Some(\n                                                            Arc::<str>::from(\"table-demo-header-\"),"
        ));
        assert!(TABLE_DEMO.contains(
            "row_test_id_prefix: Some(Arc::<str>::from(\n                                                            \"table-demo-row-\","
        ));
        assert!(!TABLE_DEMO.contains("let view_options_open = view_options_open.clone_model();"));
        assert!(
            !TABLE_DEMO.contains("let header_menu_id_open = header_menu_id_open.clone_model();")
        );
        assert!(
            !TABLE_DEMO
                .contains("let header_menu_name_open = header_menu_name_open.clone_model();")
        );
        assert!(
            !TABLE_DEMO
                .contains("let header_menu_role_open = header_menu_role_open.clone_model();")
        );
        assert!(
            !TABLE_DEMO
                .contains("let header_menu_score_open = header_menu_score_open.clone_model();")
        );
        assert!(!TABLE_DEMO.contains("enable_grouping_state.clone_model();"));
        assert!(!TABLE_DEMO.contains("grouped_column_mode_state.clone_model();"));
        assert!(!TABLE_DEMO.contains("table_state: Model<TableState>,"));
        assert!(!TABLE_DEMO.contains("cx.observe_model(&table_state, Invalidation::Layout);"));
        assert!(!TABLE_DEMO.contains(".models().read(&table_state, |st|"));
    }

    #[test]
    fn table_demo_uses_structured_table_debug_ids() {
        assert!(TABLE_DEMO.contains(
            "let table_debug_ids =\n                                                    fret_ui_kit::declarative::table::TableDebugIds {"
        ));
        assert!(TABLE_DEMO.contains(
            "header_row_test_id: Some(Arc::<str>::from(\n                                                            \"table-demo-header-row\","
        ));
        assert!(TABLE_DEMO.contains(
            "header_cell_test_id_prefix: Some(\n                                                            Arc::<str>::from(\"table-demo-header-\"),"
        ));
        assert!(TABLE_DEMO.contains(
            "row_test_id_prefix: Some(Arc::<str>::from(\n                                                            \"table-demo-row-\","
        ));
        assert!(
            TABLE_DEMO
                .contains("Prefer table-owned diagnostics anchors over renderer-local markers.")
        );
    }

    #[test]
    fn datatable_examples_prefer_local_state_table_bridges() {
        assert!(DATATABLE_DEMO.contains("use fret::advanced::prelude::LocalState;"));
        assert!(DATATABLE_DEMO.contains("table_state: LocalState<TableState>,"));
        assert!(
            DATATABLE_DEMO
                .contains("let table_state = LocalState::new_in(app.models_mut(), table_state);")
        );
        assert!(
            DATATABLE_DEMO
                .contains("let (selected, sorting) = table_state.layout_read_ref_in(cx, |st| {")
        );
        assert!(DATATABLE_DEMO.contains("shadcn::DataTableToolbar::new("));
        assert!(
            DATATABLE_DEMO
                .contains("shadcn::DataTablePagination::new(&table_state, table_output.clone())")
        );
        assert!(DATATABLE_DEMO.contains("&table_state,"));
        assert!(!DATATABLE_DEMO.contains("table_state: Model<TableState>,"));
        assert!(!DATATABLE_DEMO.contains(".models().read(&table_state, |st|"));
    }

    #[test]
    fn datatable_demo_uses_structured_table_debug_ids() {
        assert!(
            DATATABLE_DEMO.contains(".debug_ids(fret_ui_kit::declarative::table::TableDebugIds {")
        );
        assert!(DATATABLE_DEMO.contains(
            "header_row_test_id: Some(Arc::<str>::from(\"datatable-demo-header-row\")),"
        ));
        assert!(DATATABLE_DEMO.contains(
            "header_cell_test_id_prefix: Some(Arc::<str>::from(\"datatable-demo-header-\")),"
        ));
        assert!(
            DATATABLE_DEMO
                .contains("row_test_id_prefix: Some(Arc::<str>::from(\"datatable-demo-row-\")),")
        );
    }

    #[test]
    fn table_stress_demo_uses_structured_table_debug_ids() {
        assert!(TABLE_STRESS_DEMO.contains(
            "let table_debug_ids =\n                                                    fret_ui_kit::declarative::table::TableDebugIds {"
        ));
        assert!(TABLE_STRESS_DEMO.contains(
            "header_row_test_id: Some(Arc::<str>::from(\n                                                            \"table-stress-header-row\","
        ));
        assert!(TABLE_STRESS_DEMO.contains(
            "header_cell_test_id_prefix: Some(\n                                                            Arc::<str>::from(\"table-stress-header-\"),"
        ));
        assert!(TABLE_STRESS_DEMO.contains(
            "row_test_id_prefix: Some(Arc::<str>::from(\n                                                            \"table-stress-row-\","
        ));
        assert!(
            TABLE_STRESS_DEMO
                .contains("Keep stress/perf diagnostics on table-owned layout wrappers.")
        );
        assert!(
            !TABLE_STRESS_DEMO
                .contains("fret_ui_kit::declarative::table::TableDebugIds::default()")
        );
    }

    #[test]
    fn embedded_viewport_driver_extensions_are_discoverable_from_advanced_prelude() {
        assert!(EMBEDDED_VIEWPORT_DEMO.contains(".drive_embedded_viewport()"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains(".drive_embedded_viewport()"));
        assert!(!EMBEDDED_VIEWPORT_DEMO.contains("EmbeddedViewportUiAppDriverExt"));
        assert!(!IMUI_EDITOR_PROOF_DEMO.contains("EmbeddedViewportUiAppDriverExt"));
    }

    #[test]
    fn first_party_examples_use_curated_shadcn_surface() {
        for src in [
            ASSETS_DEMO,
            ASYNC_PLAYGROUND_DEMO,
            CJK_CONFORMANCE_DEMO,
            COMPONENTS_GALLERY_DEMO,
            CUSTOM_EFFECT_V1_DEMO,
            CUSTOM_EFFECT_V2_DEMO,
            CUSTOM_EFFECT_V2_GLASS_CHROME_WEB_DEMO,
            CUSTOM_EFFECT_V2_IDENTITY_WEB_DEMO,
            CUSTOM_EFFECT_V2_LUT_WEB_DEMO,
            CUSTOM_EFFECT_V2_WEB_DEMO,
            CUSTOM_EFFECT_V3_DEMO,
            DOCKING_ARBITRATION_DEMO,
            DOCKING_DEMO,
            DROP_SHADOW_DEMO,
            EMBEDDED_VIEWPORT_DEMO,
            EMOJI_CONFORMANCE_DEMO,
            GENUI_DEMO,
            HELLO_COUNTER_DEMO,
            IME_SMOKE_DEMO,
            IMUI_EDITOR_PROOF_DEMO,
            IMUI_INTERACTION_SHOWCASE_DEMO,
            IMUI_SHADCN_ADAPTER_DEMO,
            LIQUID_GLASS_DEMO,
            MARKDOWN_DEMO,
            POSTPROCESS_THEME_DEMO,
            QUERY_ASYNC_TOKIO_DEMO,
            SIMPLE_TODO_DEMO,
            SONNER_DEMO,
        ] {
            assert_shadcn_surface_is_curated(src);
        }
    }

    #[test]
    fn parse_editor_theme_preset_key_accepts_supported_values() {
        assert_eq!(
            super::parse_editor_theme_preset_key("default"),
            Some(fret_ui_editor::theme::EditorThemePresetV1::Default)
        );
        assert_eq!(
            super::parse_editor_theme_preset_key(" imgui_like_dense "),
            Some(fret_ui_editor::theme::EditorThemePresetV1::ImguiLikeDense)
        );
    }

    #[test]
    fn parse_editor_theme_preset_key_rejects_empty_and_unknown_values() {
        assert_eq!(super::parse_editor_theme_preset_key(""), None);
        assert_eq!(super::parse_editor_theme_preset_key("neutral"), None);
    }
}
