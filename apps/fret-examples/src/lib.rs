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
pub use fret_examples_imui::imui_floating_windows_demo;
#[cfg(not(target_arch = "wasm32"))]
pub use fret_examples_imui::imui_hello_demo;
#[cfg(not(target_arch = "wasm32"))]
pub use fret_examples_imui::imui_interaction_showcase_demo;
#[cfg(all(not(target_arch = "wasm32"), feature = "node-graph-demos-legacy"))]
pub mod imui_node_graph_demo;
#[cfg(not(target_arch = "wasm32"))]
pub use fret_examples_imui::imui_response_signals_demo;
#[cfg(not(target_arch = "wasm32"))]
pub use fret_examples_imui::imui_shadcn_adapter_demo;
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
    const IMUI_EDITOR_PROOF_DEMO_COLLECTION_MODULE: &str =
        include_str!("imui_editor_proof_demo/collection.rs");
    const IMUI_FLOATING_WINDOWS_DEMO: &str =
        include_str!("../../fret-examples-imui/src/imui_floating_windows_demo.rs");
    const IMUI_HELLO_DEMO: &str = include_str!("../../fret-examples-imui/src/imui_hello_demo.rs");
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
    const IMUI_IMGUI_PARITY_AUDIT_V2: &str =
        include_str!("../../../docs/workstreams/standalone/imui-imgui-parity-audit-v2.md");
    const IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO: &str =
        include_str!("../../../docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md");
    const IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_NEXT_PRIORITY: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md"
    );
    const IMUI_RESPONSE_STATUS_LIFECYCLE_TODO: &str =
        include_str!("../../../docs/workstreams/imui-response-status-lifecycle-v1/TODO.md");
    const WORKSTREAMS_INDEX_DOC: &str = include_str!("../../../docs/workstreams/README.md");
    const ROADMAP_DOC: &str = include_str!("../../../docs/roadmap.md");
    const TODO_TRACKER_DOC: &str = include_str!("../../../docs/todo-tracker.md");
    const FIRST_FRAME_SMOKE_DEMO: &str = include_str!("first_frame_smoke_demo.rs");
    const FRET_LAUNCH_DESKTOP_APP_HANDLER: &str =
        include_str!("../../../crates/fret-launch/src/runner/desktop/runner/app_handler.rs");
    const FRET_LAUNCH_DESKTOP_WINDOW_LIFECYCLE: &str =
        include_str!("../../../crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs");
    const FRET_LAUNCH_RUNNER_SCHEDULING_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/WORKSTREAM.json"
    );
    const FRET_LAUNCH_RUNNER_SCHEDULING_EVIDENCE: &str = include_str!(
        "../../../docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/EVIDENCE_AND_GATES.md"
    );
    const FRET_LAUNCH_RUNNER_SCHEDULING_FIRST_FRAME_NOTE: &str = include_str!(
        "../../../docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/M6_FIRST_FRAME_BOOTSTRAP_CLOSURE_2026-04-26.md"
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
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_AUTOMATION_DECISION_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_MONITOR_SCALE_GATE_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_REAL_HOST_ACCEPTANCE_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_WINDOWS_PLACEMENT_GATE_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M8_WINDOWS_TEAROFF_PLACEMENT_CAPTURE_GATE_2026-04-26.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_WINDOWS_CURSOR_CONTINUITY_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M9_WINDOWS_TEAROFF_CURSOR_CONTINUITY_FIX_2026-04-26.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_WINDOW_STYLE_OPACITY_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M10_WINDOW_STYLE_OPACITY_CAPABILITY_2026-04-26.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_WAYLAND_DEGRADATION_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md"
    );
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_WAYLAND_ACCEPTANCE_RUNBOOK_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md"
    );
    const DIAG_MONITOR_TOPOLOGY_ENVIRONMENT_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/diag-monitor-topology-environment-v1/WORKSTREAM.json"
    );
    const DIAG_MONITOR_TOPOLOGY_ENVIRONMENT_DESIGN: &str =
        include_str!("../../../docs/workstreams/diag-monitor-topology-environment-v1/DESIGN.md");
    const DIAG_MONITOR_TOPOLOGY_ENVIRONMENT_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_DESIGN: &str =
        include_str!("../../../docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md");
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_BASELINE: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/BASELINE_AUDIT_2026-04-20.md"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M1_DECISION: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/M1_FIRST_SOURCE_AND_TIMING_DECISION_2026-04-20.md"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M2_DECISION: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M2_FOUNDATION: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/M2_ENVIRONMENT_SOURCE_CATALOG_FOUNDATION_2026-04-20.md"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M3_PUBLICATION: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/M3_HOST_MONITOR_TOPOLOGY_LAUNCH_TIME_PUBLICATION_AND_CAMPAIGN_PROVENANCE_2026-04-20.md"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M4_TRANSPORT_QUERY: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/M4_TRANSPORT_SESSION_ENVIRONMENT_SOURCE_QUERY_FOUNDATION_2026-04-20.md"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M5_ADMISSION: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/M5_REQUIRES_ENVIRONMENT_HOST_MONITOR_TOPOLOGY_ADMISSION_2026-04-20.md"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/CLOSEOUT_AUDIT_2026-04-20.md"
    );
    const DIAG_ENVIRONMENT_PREDICATE_CONTRACT_EVIDENCE_GATES: &str = include_str!(
        "../../../docs/workstreams/diag-environment-predicate-contract-v1/EVIDENCE_AND_GATES.md"
    );
    const RUNNER_MONITOR_TOPOLOGY_DIAGNOSTICS: &str =
        include_str!("../../../crates/fret-runtime/src/runner_monitor_topology_diagnostics.rs");
    const ELEMENT_RUNTIME_DIAGNOSTICS_RS: &str = include_str!(
        "../../../ecosystem/fret-bootstrap/src/ui_diagnostics/element_runtime_diagnostics.rs"
    );
    const RUNTIME_FONT_CATALOG_RS: &str =
        include_str!("../../../crates/fret-runtime/src/font_catalog.rs");
    const DIAG_CAMPAIGNS_RS: &str =
        include_str!("../../../crates/fret-diag/src/registry/campaigns.rs");
    const DIAG_CAMPAIGN_RS: &str = include_str!("../../../crates/fret-diag/src/diag_campaign.rs");
    const DIAG_LIB_RS: &str = include_str!("../../../crates/fret-diag/src/lib.rs");
    const DIAG_DEVTOOLS_RS: &str = include_str!("../../../crates/fret-diag/src/devtools.rs");
    const DIAG_FS_TRANSPORT_RS: &str =
        include_str!("../../../crates/fret-diag/src/transport/fs.rs");
    const DIAG_PROTOCOL_RS: &str = include_str!("../../../crates/fret-diag-protocol/src/lib.rs");
    const UI_DIAGNOSTICS_DEVTOOLS_WS_RS: &str = include_str!(
        "../../../ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs"
    );
    const UI_DIAGNOSTICS_WS_BRIDGE_RS: &str =
        include_str!("../../../ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs");
    const ENVIRONMENT_QUERIES_ADR: &str =
        include_str!("../../../docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md");
    const RESOURCE_LOADING_WORKSTREAM_README: &str =
        include_str!("../../../docs/workstreams/resource-loading-fearless-refactor-v1/README.md");
    const UI_DIAGNOSTICS_SERVICE_RS: &str =
        include_str!("../../../ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs");
    const UI_DIAGNOSTICS_FS_TRIGGERS_RS: &str =
        include_str!("../../../ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs");
    const UI_DIAGNOSTICS_BUNDLE_RS: &str =
        include_str!("../../../ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs");
    const DIAG_EXTENSIBILITY_DETERMINISM_DOC: &str = include_str!(
        "../../../docs/workstreams/diag-extensibility-and-capabilities-v1/determinism.md"
    );
    const UI_DIAGNOSTICS_BUNDLES_DOC: &str =
        include_str!("../../../docs/ui-diagnostics-and-scripted-tests.md");
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_DOC: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md"
    );
    const MACOS_DOCKING_MULTIWINDOW_IMGUI_PARITY_DOC: &str = include_str!(
        "../../../docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md"
    );
    const IMUI_P3_WINDOWS_PLACEMENT_CAMPAIGN: &str =
        include_str!("../../../tools/diag-campaigns/imui-p3-windows-placement-real-host.json");
    const IMUI_P3_WINDOWS_PLACEMENT_SCRIPT: &str = include_str!(
        "../../../tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-windows-tearoff-placement-capture.debug.json"
    );
    const IMUI_SHADCN_ADAPTER_DEMO: &str =
        include_str!("../../fret-examples-imui/src/imui_shadcn_adapter_demo.rs");
    const IME_SMOKE_DEMO: &str = include_str!("ime_smoke_demo.rs");
    const EDITOR_NOTES_DEVICE_SHELL_DEMO: &str = include_str!("editor_notes_device_shell_demo.rs");
    const LAUNCHER_UTILITY_WINDOW_DEMO: &str = include_str!("launcher_utility_window_demo.rs");
    const LAUNCHER_UTILITY_WINDOW_MATERIALS_DEMO: &str =
        include_str!("launcher_utility_window_materials_demo.rs");
    const LIQUID_GLASS_DEMO: &str = include_str!("liquid_glass_demo.rs");
    const MARKDOWN_DEMO: &str = include_str!("markdown_demo.rs");
    const COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT: &str = include_str!(
        "../../../docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT_2026-04-16.md"
    );
    const NODE_GRAPH_DEMO: &str = include_str!("node_graph_demo.rs");
    const PLOT_STRESS_DEMO: &str = include_str!("plot_stress_demo.rs");
    const POSTPROCESS_THEME_DEMO: &str = include_str!("postprocess_theme_demo.rs");
    const QUERY_ASYNC_TOKIO_DEMO: &str = include_str!("query_async_tokio_demo.rs");
    const QUERY_DEMO: &str = include_str!("query_demo.rs");
    const SIMPLE_TODO_DEMO: &str = include_str!("simple_todo_demo.rs");
    const SONNER_DEMO: &str = include_str!("sonner_demo.rs");
    const TABLE_DEMO: &str = include_str!("table_demo.rs");
    const TABLE_STRESS_DEMO: &str = include_str!("table_stress_demo.rs");
    const TEXT_HEAVY_MEMORY_DEMO: &str = include_str!("text_heavy_memory_demo.rs");
    const TODO_DEMO: &str = include_str!("todo_demo.rs");
    const VIRTUAL_LIST_STRESS_DEMO: &str = include_str!("virtual_list_stress_demo.rs");
    const WINDOW_HIT_TEST_PROBE_DEMO: &str = include_str!("window_hit_test_probe_demo.rs");
    const WORKSPACE_SHELL_DEMO: &str = include_str!("workspace_shell_demo.rs");

    fn assert_avoids_legacy_conversion_names(src: &str) {
        assert!(!src.contains("UiIntoElement"));
        assert!(!src.contains("UiHostBoundIntoElement"));
        assert!(!src.contains("UiChildIntoElement"));
        assert!(!src.contains("UiBuilderHostBoundIntoElementExt"));
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

    fn assert_advanced_helpers_prefer_app_component_cx(
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

    fn assert_app_facing_concrete_helpers_prefer_app_render_cx(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(normalized.contains("AppRenderCx<'_>"));
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

    fn assert_app_facing_generic_helpers_prefer_app_render_context(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(normalized.contains("AppRenderContext<'a>"));
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
    fn first_frame_bootstrap_smoke_locks_runner_wake_paths() {
        let smoke = FIRST_FRAME_SMOKE_DEMO
            .split_whitespace()
            .collect::<String>();
        for marker in [
            "scene.push(SceneOp::Quad {",
            "state.frames_drawn = state.frames_drawn.saturating_add(1);",
            "if state.frames_drawn < 3 {",
            "app.push_effect(Effect::RequestAnimationFrame(window));",
            "app.push_effect(Effect::Window(WindowRequest::Close(window)));",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(smoke.contains(&marker), "missing smoke marker: {marker}");
        }

        let insert_window = source_slice(
            FRET_LAUNCH_DESKTOP_WINDOW_LIFECYCLE,
            "pub(super) fn insert_window(",
            "pub(super) fn close_window",
        );
        let insert_window_normalized = insert_window.split_whitespace().collect::<String>();
        for marker in [
            "self.window_registry.insert(winit_id, id);",
            "self.request_window_redraw_with_reason(",
            "fret_runtime::RunnerFrameDriveReason::SurfaceBootstrap,",
            "self.raf_windows.request(id);",
            "window may appear blank until another event arrives",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                insert_window_normalized.contains(&marker),
                "missing normal bootstrap marker: {marker}"
            );
        }

        let deferred_surface = source_slice(
            FRET_LAUNCH_DESKTOP_APP_HANDLER,
            "fn try_create_missing_surfaces(&mut self) {",
            "impl<D: WinitAppDriver> ApplicationHandler for WinitRunner<D> {",
        );
        let deferred_surface_normalized = deferred_surface.split_whitespace().collect::<String>();
        for marker in [
            "let mut redraw_bootstrap_windows: Vec<fret_core::AppWindowId> = Vec::new();",
            "redraw_bootstrap_windows.push(app_window);",
            "self.request_window_redraw_with_reason(",
            "fret_runtime::RunnerFrameDriveReason::SurfaceBootstrap,",
            "self.raf_windows.request(app_window);",
            "deferred surface creation also gets a one-shot RAF",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                deferred_surface_normalized.contains(&marker),
                "missing deferred bootstrap marker: {marker}"
            );
        }
        assert!(
            !deferred_surface_normalized.contains(
                &"state.window.request_redraw();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "deferred bootstrap must not bypass the redraw helper"
        );
        assert!(
            !deferred_surface_normalized.contains(
                &"self.record_frame_drive_reason( app_window,"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "SurfaceBootstrap diagnostics should be paired with the redraw helper"
        );

        let about_to_wait = source_slice(
            FRET_LAUNCH_DESKTOP_APP_HANDLER,
            "fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {",
            "fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {",
        );
        let about_to_wait_normalized = about_to_wait.split_whitespace().collect::<String>();
        for marker in [
            "next_raf_deadline.get_or_insert_with(|| now + self.config.frame_interval);",
            "let flushed_raf_this_turn = raf_deadline.is_some_and(|deadline| now >= deadline);",
            "self.flush_raf_redraw_requests();",
            "if wants_poll || flushed_raf_this_turn {",
            "event_loop.set_control_flow(ControlFlow::Poll);",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                about_to_wait_normalized.contains(&marker),
                "missing RAF deadline wake marker: {marker}"
            );
        }

        let workstream: serde_json::Value =
            serde_json::from_str(FRET_LAUNCH_RUNNER_SCHEDULING_WORKSTREAM)
                .expect("workstream state should be valid JSON");
        assert_eq!(workstream["status"], "maintenance");
        assert_eq!(workstream["scope_kind"], "execution");
        assert!(
            workstream["problem"]
                .as_str()
                .unwrap_or_default()
                .contains("pointer movement or hover"),
            "workstream state should name the blank-until-hover invariant"
        );

        for marker in [
            "first_frame_smoke_demo",
            "SurfaceBootstrap",
            "request_window_redraw_with_reason",
            "one-shot RAF",
            "blank-start reports",
        ] {
            assert!(
                FRET_LAUNCH_RUNNER_SCHEDULING_EVIDENCE.contains(marker)
                    || FRET_LAUNCH_RUNNER_SCHEDULING_FIRST_FRAME_NOTE.contains(marker),
                "first-frame workstream docs should name marker: {marker}"
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
            "references below to `DW-P0-dpi-006` as the current open blocker are historical",
            "do not reopen generic `imui` helper growth or widen `crates/fret-ui`",
            "`tools/diag-campaigns/imui-p3-multiwindow-parity.json` as the bounded P3 regression entry",
            "M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md",
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
            "The \"no mixed-DPI campaign yet\" portion is superseded",
            "real-host acceptance pair",
            "\"pre-crossing\" bundle",
            "\"post-crossing\" bundle",
            "`DW-P0-dpi-006` is no longer open",
            "M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md",
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
            "docking-arbitration-demo-multiwindow-drag-back-monitor-scale-sweep.debug.json",
            "target/fret-diag/docking-multiwindow-imgui-parity/mixed-dpi-real-host",
            "multiwindow-drag-back-monitor-sweep-after-tearoff",
            "multiwindow-drag-back-monitor-sweep-after-lowest-scale-monitor",
            "multiwindow-drag-back-monitor-sweep-after-highest-scale-monitor",
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
            "imui-p3-mixed-dpi-real-host",
            "M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md",
            "Manual acceptance run on a real mixed-DPI setup",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the mixed-DPI TODO runbook should remain explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_mixed_dpi_automation_decision() {
        for marker in [
            "\"role\": \"status\"",
            "M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the mixed-DPI automation decision reachable: {marker}"
            );
        }

        for marker in [
            "`bundle.json.env.scale_factors_seen` is not a host monitor-topology contract",
            "`mixed_dpi_signal_observed` remains drag evidence, not a preflight capability",
            "historical decision note, superseded",
            "`host.monitor_topology` + `host_monitor_topology` admission",
            "Keep the bounded P3 campaign generic",
            "dedicated real-host mixed-DPI campaign",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_AUTOMATION_DECISION_NOTE
                    .contains(marker),
                "the docking parity lane should keep the mixed-DPI automation decision explicit: {marker}"
            );
        }

        for marker in [
            "M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md",
            "M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md",
            "M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md",
            "Manual acceptance run on a real mixed-DPI setup",
            "host.monitor_topology",
            "Result: yes for this narrow source-scoped shape",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the mixed-DPI TODO state should keep the automation decision explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_mixed_dpi_monitor_scale_gate() {
        for marker in [
            "\"role\": \"status\"",
            "M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md",
            "imui-p3-mixed-dpi-real-host",
            "docking-arbitration-demo-multiwindow-drag-back-monitor-scale-sweep.debug.json",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the mixed-DPI monitor-scale gate reachable: {marker}"
            );
        }

        for marker in [
            "`host.monitor_topology` + `host_monitor_topology` admission",
            "set_cursor_at_host_monitor",
            "lowest-scale monitor",
            "highest-scale monitor",
            "Keep `imui-p3-multiwindow-parity` generic and portable.",
            "Add `imui-p3-mixed-dpi-real-host` as the dedicated mixed-DPI real-host campaign.",
            "`mixed_dpi_signal_observed: true`",
            "at least two observed scale factors",
            "M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md",
            "closes `DW-P0-dpi-006`",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_MONITOR_SCALE_GATE_NOTE.contains(marker),
                "the mixed-DPI monitor-scale gate note should keep the current posture explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_mixed_dpi_real_host_acceptance() {
        for marker in [
            "\"role\": \"status\"",
            "M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md",
            "target/release/docking_arbitration_demo.exe",
            "imui-p3-mixed-dpi-real-host",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the mixed-DPI accepted run reachable: {marker}"
            );
        }

        for marker in [
            "Status: accepted real-host evidence; closes `DW-P0-dpi-006`",
            "scale factor `1.25`",
            "scale factor `1.50`",
            "multiwindow-drag-back-monitor-sweep-after-lowest-scale-monitor",
            "mixed_dpi=true scale_factors=1.250, 1.500",
            "canonical_ok=true",
            "floatings=[]",
            "Keep the dedicated",
            "generic `imui-p3-multiwindow-parity`",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_REAL_HOST_ACCEPTANCE_NOTE
                    .contains(marker),
                "the mixed-DPI accepted-run note should keep bounded evidence explicit: {marker}"
            );
        }

        for marker in [
            "- [x] DW-P0-dpi-006 Mixed-DPI multi-monitor follow",
            "Real-host acceptance evidence is now recorded",
            "M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md",
            "post-crossing bundle reports `mixed_dpi_signal_observed: true`",
            "final bundle reports one window, `canonical_ok=true`, and `floatings=[]`",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the docking parity TODO should keep the mixed-DPI acceptance closure explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_windows_placement_capture_gate() {
        for marker in [
            "\"role\": \"status\"",
            "M8_WINDOWS_TEAROFF_PLACEMENT_CAPTURE_GATE_2026-04-26.md",
            "M9_WINDOWS_TEAROFF_CURSOR_CONTINUITY_FIX_2026-04-26.md",
            "imui-p3-windows-placement-real-host",
            "docking-arbitration-demo-windows-tearoff-placement-capture.debug.json",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the Windows placement gate reachable: {marker}"
            );
        }

        for marker in [
            "moving_window_outer_pos_physical_px",
            "moving_window_client_origin_screen_physical_px",
            "moving_window_cursor_grab_error_abs_max_logical_px",
            "move_grab_delta",
            "move_grab_error",
            "move_origin_src=platform",
            "windows-tearoff-placement-after-tearoff-initial",
            "windows-tearoff-placement-after-tearoff-settled",
            "M9 used that surface to identify and fix the remaining",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WINDOWS_PLACEMENT_GATE_NOTE.contains(marker),
                "the Windows placement gate note should keep the bounded evidence surface explicit: {marker}"
            );
        }

        for marker in [
            "Status: accepted real-host fix for `DW-P1-win-002`",
            "move_grab_error=786.7",
            "move_grab_error=0.0",
            "crates/fret-launch/src/runner/desktop/runner/diag_cursor_override.rs",
            "dock_drag_migration_remaps_drag_pointer_until_after_pointer_down",
            "1777187535921-windows-tearoff-placement-after-tearoff-settled",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WINDOWS_CURSOR_CONTINUITY_NOTE.contains(marker),
                "the Windows cursor-continuity fix note should keep acceptance evidence explicit: {marker}"
            );
        }

        for marker in [
            "\"id\": \"imui-p3-windows-placement-real-host\"",
            "\"source_id\": \"platform.capabilities\"",
            "\"platform_is\": \"windows\"",
            "\"ui_window_tear_off_is\": true",
        ] {
            assert!(
                IMUI_P3_WINDOWS_PLACEMENT_CAMPAIGN.contains(marker),
                "the Windows placement campaign should keep platform admission explicit: {marker}"
            );
        }

        for marker in [
            "\"FRET_DOCK_TEAROFF_FOLLOW_IN_DIAG\": \"1\"",
            "windows-tearoff-placement-after-tearoff-initial",
            "windows-tearoff-placement-after-tearoff-settled",
            "docking-arbitration-demo-windows-tearoff-placement-capture",
        ] {
            assert!(
                IMUI_P3_WINDOWS_PLACEMENT_SCRIPT.contains(marker),
                "the Windows placement script should keep the capture labels explicit: {marker}"
            );
        }

        for marker in [
            "`crates/fret-runtime/src/drag.rs` (`diag_moving_window_*`)",
            "`crates/fret-launch/src/runner/desktop/runner/diag_cursor_override.rs`",
            "`crates/fret-diag/src/commands/dock_routing.rs` (`move_grab_delta`)",
            "M9_WINDOWS_TEAROFF_CURSOR_CONTINUITY_FIX_2026-04-26.md",
            "move_grab_error=0.0",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the Windows placement TODO should keep the new proof surface explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_window_style_opacity_capability() {
        for marker in [
            "M10_WINDOW_STYLE_OPACITY_CAPABILITY_2026-04-26.md",
            "ui.window.opacity",
            "crates/fret-runtime/src/runner_window_style_diagnostics.rs",
            "opacity_alpha_u8",
            "DW-P2-style-001",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the window-style opacity closure reachable: {marker}"
            );
        }

        for marker in [
            "Status: accepted source-level closure for `DW-P2-style-001`",
            "`WindowStyleRequest::opacity`",
            "`PlatformCapabilities.ui.window_opacity`",
            "`ui.window.opacity`",
            "`WindowOpacity(255)`",
            "`window_style_effective_is` predicates can assert `opacity_alpha_u8`",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WINDOW_STYLE_OPACITY_NOTE.contains(marker),
                "the window-style opacity note should keep the source-level closure explicit: {marker}"
            );
        }

        for marker in [
            "- [x] DW-P2-style-001 DockFloating window style requests",
            "M10_WINDOW_STYLE_OPACITY_CAPABILITY_2026-04-26.md",
            "`ui.window.opacity`",
            "effective `opacity_alpha_u8`",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the docking parity TODO should keep the style closure explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_wayland_degradation_policy_slice() {
        for marker in [
            "\"role\": \"status\"",
            "M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md",
            "linux_windowing_capability_posture",
            "request_float_degrades_to_in_window_when_window_hover_detection_is_none",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the Wayland degradation slice reachable: {marker}"
            );
        }

        for marker in [
            "Wayland degradation is an owner-split question first",
            "keep `ui.multi_window=true`",
            "`ui.window_tear_off=false`",
            "`ui.window_hover_detection=none`",
            "in-window floating fallback instead of `CreateWindowKind::DockFloating`",
            "Manual compositor acceptance is a different proof step from source-policy freeze",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WAYLAND_DEGRADATION_NOTE.contains(marker),
                "the docking parity lane should keep the Wayland degradation policy explicit: {marker}"
            );
        }

        for marker in [
            "DW-P1-linux-003 Wayland-safe degradation policy for follow-mode.",
            "M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md",
            "Wayland keeps `ui.multi_window=true`",
            "Docking runtime fallback is now explicitly locked for `window_hover_detection == None`.",
            "Manual Wayland compositor acceptance remains open.",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the docking parity TODO should keep the Wayland degradation progress explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_wayland_compositor_acceptance_runbook() {
        for marker in [
            "\"role\": \"next\"",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
            "docking-arbitration-demo-wayland-degrade-no-os-tearoff.json",
            "imui-p3-wayland-real-host",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the Wayland compositor runbook reachable: {marker}"
            );
        }

        for marker in [
            "Run this only on a Linux native Wayland session.",
            "`XDG_SESSION_TYPE=wayland`",
            "`platform.capabilities`",
            "`imui-p3-wayland-real-host`",
            "`docking-arbitration-demo-wayland-degrade-no-os-tearoff.json`",
            "`known_window_count_is(n=1)`",
            "`diag windows`",
            "`diag dock-graph`",
            "`[effect-window-create]`",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WAYLAND_ACCEPTANCE_RUNBOOK_NOTE.contains(marker),
                "the docking parity lane should keep the Wayland compositor acceptance runbook explicit: {marker}"
            );
        }

        for marker in [
            "Real-host acceptance runbook is now explicit",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
            "docking-arbitration-demo-wayland-degrade-no-os-tearoff.json",
            "tools/diag-campaigns/imui-p3-wayland-real-host.json",
            "Campaign admission now uses the launch-time `platform.capabilities` environment source",
            "`diag windows`",
            "`diag dock-graph`",
            "Manual Wayland compositor acceptance remains open.",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the docking parity TODO should keep the Wayland acceptance package explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p3_mixed_dpi_monitor_topology_follow_on() {
        for marker in [
            "\"slug\": \"diag-monitor-topology-environment-v1\"",
            "\"status\": \"closed\"",
            "\"follow_on_of\": \"docking-multiwindow-imgui-parity\"",
            "RunnerMonitorTopologyDiagnosticsStore",
            "monitor_topology",
            "\"default_action\": \"stay_closed\"",
        ] {
            assert!(
                DIAG_MONITOR_TOPOLOGY_ENVIRONMENT_WORKSTREAM.contains(marker),
                "the monitor-topology follow-on should keep the first-open state marker reachable: {marker}"
            );
        }

        for marker in [
            "It does not reopen the broad docking parity lane.",
            "It does not add mixed-DPI-only campaign gates.",
            "`scale_factors_seen` remains the last-known per-window scale-factor evidence",
            "environment predicates remain a future follow-on.",
        ] {
            assert!(
                DIAG_MONITOR_TOPOLOGY_ENVIRONMENT_DESIGN.contains(marker),
                "the monitor-topology design note should keep the owner split explicit: {marker}"
            );
        }

        for marker in [
            "RunnerMonitorTopologyDiagnosticsStore",
            "RunnerMonitorTopologySnapshotV1",
            "update_snapshot",
            "clear_snapshot",
        ] {
            assert!(
                RUNNER_MONITOR_TOPOLOGY_DIAGNOSTICS.contains(marker),
                "the runtime diagnostics store should keep the host monitor-topology contract explicit: {marker}"
            );
        }

        for marker in [
            "sync_runner_monitor_topology_from_app",
            "RunnerMonitorTopologyDiagnosticsStore",
            "refresh_environment_source_files",
            "published_host_monitor_topology",
            "environment_sources_catalog_written",
        ] {
            assert!(
                UI_DIAGNOSTICS_SERVICE_RS.contains(marker),
                "diagnostics should keep syncing the runtime monitor-topology source explicitly: {marker}"
            );
        }

        for marker in [
            "refresh_environment_source_files",
            "HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1",
            "FILESYSTEM_HOST_MONITOR_TOPOLOGY_ENVIRONMENT_PAYLOAD_FILE_NAME_V1",
            "EnvironmentSourceAvailabilityV1::LaunchTime",
        ] {
            assert!(
                UI_DIAGNOSTICS_FS_TRIGGERS_RS.contains(marker),
                "filesystem diagnostics publication should keep the launch-time source sidecars explicit: {marker}"
            );
        }

        for marker in [
            "pub monitor_topology: Option<UiDiagnosticsMonitorTopologyV1>",
            "scale_factors_seen",
            "env_fingerprint_exports_host_monitor_topology_without_reclassifying_scale_factors_seen",
        ] {
            assert!(
                UI_DIAGNOSTICS_BUNDLE_RS.contains(marker),
                "the bundle env fingerprint should keep the monitor-topology split explicit: {marker}"
            );
        }

        for marker in [
            "`monitor_topology` is the host environment fingerprint.",
            "`scale_factors_seen` remains run-observed per-window evidence",
        ] {
            assert!(
                DIAG_EXTENSIBILITY_DETERMINISM_DOC.contains(marker),
                "the diagnostics determinism note should keep the environment boundary explicit: {marker}"
            );
        }

        for marker in [
            "`bundle.json.env.monitor_topology` is the host monitor inventory",
            "`bundle.json.env.scale_factors_seen` remains the last-known per-window scale factors observed",
            "Do not treat `scale_factors_seen` as host monitor topology",
        ] {
            assert!(
                UI_DIAGNOSTICS_BUNDLES_DOC.contains(marker),
                "the living diagnostics doc should keep the monitor-topology guidance explicit: {marker}"
            );
        }

        for marker in [
            "RunnerMonitorTopologyDiagnosticsStore",
            "bundle.json.env.monitor_topology",
            "`crates/fret-diag` still does not grow environment predicates or mixed-DPI-only campaign",
        ] {
            assert!(
                DIAG_MONITOR_TOPOLOGY_ENVIRONMENT_CLOSEOUT.contains(marker),
                "the closeout note should keep the shipped verdict explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_diag_environment_predicate_taxonomy() {
        for marker in [
            "\"slug\": \"diag-environment-predicate-contract-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"diag-monitor-topology-environment-v1\"",
            "\"default_action\": \"stay_closed\"",
            "the first honest `requires_environment` contract for `host.monitor_topology`",
            "M1_FIRST_SOURCE_AND_TIMING_DECISION_2026-04-20.md",
            "M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md",
            "M2_ENVIRONMENT_SOURCE_CATALOG_FOUNDATION_2026-04-20.md",
            "M3_HOST_MONITOR_TOPOLOGY_LAUNCH_TIME_PUBLICATION_AND_CAMPAIGN_PROVENANCE_2026-04-20.md",
            "M4_TRANSPORT_SESSION_ENVIRONMENT_SOURCE_QUERY_FOUNDATION_2026-04-20.md",
            "M5_REQUIRES_ENVIRONMENT_HOST_MONITOR_TOPOLOGY_ADMISSION_2026-04-20.md",
            "CLOSEOUT_AUDIT_2026-04-20.md",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_WORKSTREAM.contains(marker),
                "the environment-predicate lane should keep the closed workstream markers explicit: {marker}"
            );
        }

        for marker in [
            "It does not force a generic runtime `EnvironmentSnapshot` abstraction.",
            "`ElementEnvironmentSnapshotV1` is the per-window reactive UI environment surface.",
            "`RendererFontEnvironmentSnapshot` is a renderer/resource-loading provenance surface.",
            "`UiDiagnosticsEnvFingerprintV1` is the diagnostics-run environment fingerprint.",
            "`crates/fret-diag`",
            "`ecosystem/fret-bootstrap` now publishes `environment.sources.json`",
            "`environment.source.host.monitor_topology.json`",
            "`environment_source_catalog_provenance`",
            "`environment.sources.get` / `environment.sources.get_ack`",
            "Campaign manifests may now declare `requires_environment`.",
            "launch-time probe for tool-launched filesystem runs.",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_DESIGN.contains(marker),
                "the design note should keep the taxonomy and owner split explicit: {marker}"
            );
        }

        for marker in [
            "The repo already has three environment lanes with different purposes",
            "Do not generalize them into one erased runtime family yet",
            "The current automation preflight contract is still `requires_capabilities` only",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_BASELINE.contains(marker),
                "the baseline audit should keep the no-premature-abstraction verdict explicit: {marker}"
            );
        }

        for marker in [
            "The first candidate source for environment predicates is `host.monitor_topology`.",
            "Do not freeze `requires_environment` syntax yet.",
            "Do not overload `capabilities.json` with environment fingerprints.",
            "Current campaign preflight happens before launch",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M1_DECISION.contains(marker),
                "the first-source decision should keep the timing and syntax deferral explicit: {marker}"
            );
        }

        for marker in [
            "`environment.sources.json`",
            "`FilesystemEnvironmentSourcesV1`",
            "`EnvironmentSourceCatalogProvenance`",
            "`preflight_filesystem_sidecar`",
            "`post_run_only`",
            "`host.monitor_topology` is the first admitted source id",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M2_DECISION.contains(marker),
                "the provenance-and-availability decision should keep the catalog contract explicit: {marker}"
            );
        }

        for marker in [
            "`EnvironmentSourceAvailabilityV1`",
            "`FilesystemEnvironmentSourcesV1`",
            "`EnvironmentSourceCatalogProvenance`",
            "`environment.sources.json`",
            "This slice intentionally did not:",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M2_FOUNDATION.contains(marker),
                "the implementation note should keep the landed foundation scope explicit: {marker}"
            );
        }

        for marker in [
            "`environment.sources.json` at the diagnostics `out_dir`",
            "`environment.source.host.monitor_topology.json`",
            "`environment_sources_path`",
            "`environment_source_catalog_provenance`",
            "`environment_sources`",
            "Campaign preflight still only evaluates `requires_capabilities` before launch.",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M3_PUBLICATION.contains(marker),
                "the launch-time publication note should keep the runtime/publication boundary explicit: {marker}"
            );
        }

        for marker in [
            "`environment.sources.get`",
            "`environment.sources.get_ack`",
            "`devtools.environment_sources`",
            "`preflight_transport_session`",
            "Static session descriptors are the wrong owner for dynamic environment sources",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M4_TRANSPORT_QUERY.contains(marker),
                "the transport-session note should keep the explicit query boundary visible: {marker}"
            );
        }

        for marker in [
            "Campaign manifests now support `requires_environment`",
            "`source_id: \"host.monitor_topology\"`",
            "`predicate.kind: \"host_monitor_topology\"`",
            "`check.environment.json`",
            "launch-time probe for tool-launched filesystem runs",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_M5_ADMISSION.contains(marker),
                "the admission note should keep the shipped grammar and execution slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: Closed",
            "The shipped outcome is:",
            "ADR 0246 does not currently justify a second admitted source id.",
            "renderer font provenance should remain outside `requires_environment`.",
            "`scale_factors_seen` is not a second admitted source candidate.",
            "Until that evidence exists, do not widen `requires_environment`",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_CLOSEOUT.contains(marker),
                "the closeout note should keep the no-second-source verdict explicit: {marker}"
            );
        }

        for marker in [
            "pub struct ElementEnvironmentSnapshotV1",
            "pub viewport_bounds: RectV1",
            "pub safe_area_insets: Option<UiEdgesV1>",
        ] {
            assert!(
                ELEMENT_RUNTIME_DIAGNOSTICS_RS.contains(marker),
                "the per-window environment snapshot surface should stay explicit: {marker}"
            );
        }

        for marker in [
            "pub struct RendererFontEnvironmentSnapshot",
            "pub revision: u64",
            "pub text_font_stack_key: Option<u64>",
        ] {
            assert!(
                RUNTIME_FONT_CATALOG_RS.contains(marker),
                "the renderer font environment surface should stay explicit: {marker}"
            );
        }

        for marker in [
            "pub struct UiDiagnosticsEnvFingerprintV1",
            "pub monitor_topology: Option<UiDiagnosticsMonitorTopologyV1>",
            "pub scale_factors_seen: Vec<f32>",
        ] {
            assert!(
                UI_DIAGNOSTICS_BUNDLE_RS.contains(marker),
                "the diagnostics-run environment fingerprint should stay explicit: {marker}"
            );
        }

        for marker in [
            "pub requires_capabilities: Vec<String>",
            "pub requires_environment: Vec<CampaignEnvironmentRequirementDefinition>",
            "requires_environment: Vec<CampaignEnvironmentRequirementDefinition>",
            "normalize_lowercase_string_list(manifest.requires_capabilities)",
            "CampaignManifestEnvironmentPredicateV1",
        ] {
            assert!(
                DIAG_CAMPAIGNS_RS.contains(marker),
                "campaign orchestration should keep capabilities as the current preflight contract: {marker}"
            );
        }

        for marker in [
            "maybe_execute_campaign_capability_preflight",
            "maybe_execute_campaign_environment_admission",
            "check.environment.json",
            "CampaignEnvironmentAcquisition::LaunchTimeProbe",
            "execute_campaign_start_plan(start_plan)?;",
            "read_filesystem_capabilities_with_provenance(&ctx.resolved_out_dir)",
            "populate_environment_source_summary_artifacts",
            "environment_sources_path",
            "environment_source_catalog_provenance",
            "environment_sources",
        ] {
            assert!(
                DIAG_CAMPAIGN_RS.contains(marker),
                "the campaign execution path should keep preflight timing explicit: {marker}"
            );
        }

        for marker in [
            "pub(crate) fn read_filesystem_capabilities_with_provenance",
            "CapabilitySource::filesystem",
            "capabilities.json",
            "EnvironmentSourceCatalogProvenance",
            "read_filesystem_environment_sources_with_provenance",
            "environment.sources.json",
            "PublishedEnvironmentSourceArtifact",
            "read_filesystem_published_environment_sources_with_provenance",
            "read_transport_published_environment_sources",
            "read_transport_host_monitor_topology_environment_payload",
            "query_transport_environment_sources",
            "FILESYSTEM_HOST_MONITOR_TOPOLOGY_ENVIRONMENT_PAYLOAD_FILE_NAME_V1",
        ] {
            assert!(
                DIAG_LIB_RS.contains(marker),
                "the diagnostics library should keep capability-source provenance explicit: {marker}"
            );
        }

        for marker in [
            "pub fn environment_sources_get",
            "environment.sources.get",
            "wait_for_environment_sources_get_ack",
        ] {
            assert!(
                DIAG_DEVTOOLS_RS.contains(marker),
                "the devtools helper should keep the explicit environment-source request surface visible: {marker}"
            );
        }

        for marker in [
            "pub enum EnvironmentSourceAvailabilityV1",
            "pub struct FilesystemEnvironmentSourceV1",
            "pub struct FilesystemEnvironmentSourcesV1",
            "pub struct DevtoolsEnvironmentSourcesGetV1",
            "pub struct DevtoolsEnvironmentSourcesGetAckV1",
            "HOST_MONITOR_TOPOLOGY_ENVIRONMENT_SOURCE_ID_V1",
            "FILESYSTEM_HOST_MONITOR_TOPOLOGY_ENVIRONMENT_PAYLOAD_FILE_NAME_V1",
            "pub struct HostMonitorTopologyEnvironmentPayloadV1",
        ] {
            assert!(
                DIAG_PROTOCOL_RS.contains(marker),
                "the diagnostics protocol should keep the environment-source catalog types explicit: {marker}"
            );
        }

        for marker in [
            "let path = self.cfg.out_dir.join(\"capabilities.json\")",
            "FilesystemCapabilitiesV1",
            "self.last_emitted_capabilities",
        ] {
            assert!(
                DIAG_FS_TRANSPORT_RS.contains(marker),
                "the filesystem transport should keep capabilities sidecar ownership explicit: {marker}"
            );
        }

        for marker in [
            "build_environment_sources_get_ack_v1",
            "\"environment.sources.get\"",
            "\"environment.sources.get_ack\"",
            "EnvironmentSourceAvailabilityV1::PreflightTransportSession",
        ] {
            assert!(
                UI_DIAGNOSTICS_DEVTOOLS_WS_RS.contains(marker),
                "the runtime WS handler should keep the explicit transport-session source query visible: {marker}"
            );
        }

        for marker in [
            "\"environment_sources\".to_string()",
            "\"devtools.environment_sources\".to_string()",
        ] {
            assert!(
                UI_DIAGNOSTICS_WS_BRIDGE_RS.contains(marker),
                "the runtime WS capability advertisement should keep the query support capability visible: {marker}"
            );
        }

        for marker in [
            "environment query",
            "Per-window (keyed by `AppWindowId`).",
            "a **committed** per-window environment snapshot",
        ] {
            assert!(
                ENVIRONMENT_QUERIES_ADR.contains(marker),
                "the environment-queries ADR should keep the per-window contract explicit: {marker}"
            );
        }

        for marker in [
            "`fret_runtime::RendererFontEnvironmentSnapshot` tracks a monotonic `revision`",
            "resource-loading predicates can now gate that inventory by revision, source lane, and",
        ] {
            assert!(
                RESOURCE_LOADING_WORKSTREAM_README.contains(marker),
                "the resource-loading lane should keep the renderer-font environment contract explicit: {marker}"
            );
        }

        for marker in [
            "`debug.environment` remains a per-window runtime/debug surface, not a campaign preflight",
            "host-environment predicates belong to the dedicated follow-on lane",
        ] {
            assert!(
                DIAG_EXTENSIBILITY_DETERMINISM_DOC.contains(marker),
                "the diagnostics determinism note should keep the preflight boundary explicit: {marker}"
            );
        }

        for marker in [
            "Campaign manifests may also declare `requires_environment`.",
            "`source_id: \"host.monitor_topology\"`",
            "`predicate.kind: \"host_monitor_topology\"`",
            "`requires_capabilities` remains capabilities-only.",
            "Do not scrape `debug.environment` or other debug-only snapshot lanes as a substitute",
            "`environment.source.host.monitor_topology.json`",
            "`environment.sources.get` / `environment.sources.get_ack`",
            "`environment_source_catalog_provenance`",
        ] {
            assert!(
                UI_DIAGNOSTICS_BUNDLES_DOC.contains(marker),
                "the living diagnostics doc should keep the environment-predicate boundary explicit: {marker}"
            );
        }

        for marker in [
            "source-scoped `requires_environment` grammar",
            "cargo nextest run -p fret-diag --lib environment_admission --no-fail-fast",
            "`post_run_only` environment sources are evidence-only and must not drive preflight.",
            "`host.monitor_topology` now has a launch-time filesystem publication lane",
            "`check.environment.json`",
        ] {
            assert!(
                DIAG_ENVIRONMENT_PREDICATE_CONTRACT_EVIDENCE_GATES.contains(marker),
                "the evidence-and-gates note should keep the catalog and availability outcome explicit: {marker}"
            );
        }
    }

    #[test]
    fn advanced_helper_contexts_prefer_app_component_cx() {
        assert_advanced_helpers_prefer_app_component_cx(
            ASSETS_DEMO,
            &[
                "fn render_view<'a, Cx>(cx: &mut Cx) -> Ui",
                "Cx: fret::app::RenderContextAccess<'a, KernelApp>",
                "let theme = cx.theme_snapshot();",
                "let cx = cx.elements();",
                "render_view(cx)",
                "fn assets_page<C>(cx: &mut AppComponentCx<'_>, theme: &ThemeSnapshot, card: C) -> Ui",
                "C: IntoUiElement<KernelApp>",
                "fn render_image_panel(",
                "theme: &ThemeSnapshot,",
                "stats: fret_ui_assets::image_asset_cache::ImageAssetStats,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn render_svg_panel(",
                "svg: Option<fret_core::SvgId>,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
            ],
            &[
                "fn render_view(cx: &mut AppComponentCx<'_>) -> Ui",
                "render_view(cx.elements())",
                "fn assets_page<C>(cx: &mut AppComponentCx<'_>, theme: &Theme, card: C) -> Ui",
                "fn render_image_panel(cx: &mut AppComponentCx<'_>, theme: &Theme,",
                "fn render_svg_panel(cx: &mut AppComponentCx<'_>, theme: &Theme,",
                "fn render_image_panel(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_svg_panel(cx: &mut ElementContext<'_, KernelApp>,",
            ],
        );

        assert_advanced_helpers_prefer_app_component_cx(
            GENUI_DEMO,
            &[
                "fn genui_page<L, R>(cx: &mut AppComponentCx<'_>, theme: ThemeSnapshot, left: L, right: R) -> Ui",
                "L: IntoUiElement<KernelApp>,",
                "R: IntoUiElement<KernelApp>,",
                "genui_page(cx, theme, left, right)",
            ],
            &["let page = ui::container(move |cx| {"],
        );

        assert_advanced_helpers_prefer_app_component_cx(
            IMUI_EDITOR_PROOF_DEMO,
            &[
                "fn render_authoring_parity_surface(cx: &mut AppComponentCx<'_>,",
                "fn render_authoring_parity_shared_state(cx: &mut AppComponentCx<'_>,",
                "fn render_authoring_parity_declarative_group(cx: &mut AppComponentCx<'_>,",
                "fn render_authoring_parity_imui_group(cx: &mut AppComponentCx<'_>,",
            ],
            &[
                "fn render_authoring_parity_surface(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_authoring_parity_shared_state(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_authoring_parity_declarative_group(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_authoring_parity_imui_group(cx: &mut ElementContext<'_, KernelApp>,",
            ],
        );

        assert_advanced_helpers_prefer_app_component_cx(
            MARKDOWN_DEMO,
            &["let spinner_box = |cx: &mut AppComponentCx<'_>|"],
            &["let spinner_box = |cx: &mut fret_ui::ElementContext<'_, KernelApp>|"],
        );

        assert_advanced_helpers_prefer_app_component_cx(
            CUSTOM_EFFECT_V1_DEMO,
            &[
                "fn watch_first_f32(cx: &mut AppComponentCx<'_>,",
                "fn stage(cx: &mut AppComponentCx<'_>, enabled: bool, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn lens_row(cx: &mut AppComponentCx<'_>, enabled: bool, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn inspector(cx: &mut AppComponentCx<'_>, st: &mut CustomEffectV1State, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> impl IntoUiElement<KernelApp> + use<>",
                "let label_row = |cx: &mut AppComponentCx<'_>, label: &str, value: String|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut AppComponentCx<'_>, enabled: bool, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> AnyElement",
                "fn lens_row(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_row(cx: &mut AppComponentCx<'_>, enabled: bool, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut AppComponentCx<'_>, st: &mut CustomEffectV1State, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32,) -> AnyElement",
                "let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
            ],
        );

        assert_advanced_helpers_prefer_app_component_cx(
            CUSTOM_EFFECT_V2_DEMO,
            &[
                "fn watch_first_f32(cx: &mut AppComponentCx<'_>,",
                "fn stage(cx: &mut AppComponentCx<'_>, enabled: bool, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn lens_row(cx: &mut AppComponentCx<'_>, enabled: bool, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn inspector(cx: &mut AppComponentCx<'_>, st: &mut CustomEffectV2State, sampling_value: &str, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32,) -> impl IntoUiElement<KernelApp> + use<>",
                "let label_row = |cx: &mut AppComponentCx<'_>, label: &str, value: String|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut AppComponentCx<'_>, enabled: bool, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool,) -> AnyElement",
                "fn lens_row(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_row(cx: &mut AppComponentCx<'_>, enabled: bool, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool,) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut AppComponentCx<'_>, st: &mut CustomEffectV2State, sampling_value: &str, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32,) -> AnyElement",
                "let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
            ],
        );

        assert_advanced_helpers_prefer_app_component_cx(
            CUSTOM_EFFECT_V3_DEMO,
            &[
                "fn stage(cx: &mut AppComponentCx<'_>, st: &mut State, enabled: bool, show_user0_probe: bool, use_non_filterable_user0: bool, lens_effect: EffectId, user0_probe_effect: Option<EffectId>, show_user1_probe: bool, use_non_filterable_user1: bool, user1_probe_effect: Option<EffectId>, user01_probe_effect: Option<EffectId>, user0_image: Option<ImageId>, user1_image: Option<ImageId>,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn stage_controls(cx: &mut AppComponentCx<'_>, st: &mut State, enabled: bool, show_user0_probe: bool, show_user1_probe: bool, use_non_filterable_user0: bool, use_non_filterable_user1: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn animated_backdrop(cx: &mut AppComponentCx<'_>) -> impl IntoUiElement<KernelApp> + use<>",
                "fn lens_row(cx: &mut AppComponentCx<'_>, enabled: bool, show_user0_probe: bool, lens_effect: EffectId, user0_probe_effect: Option<EffectId>, show_user1_probe: bool, user1_probe_effect: Option<EffectId>, user01_probe_effect: Option<EffectId>, user0_image: Option<ImageId>, user1_image: Option<ImageId>,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn lens_shell(cx: &mut AppComponentCx<'_>, title: &'static str, radius: Px, lens_w: Px, lens_h: Px, with_effect: Option<EffectChain>,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn custom_effect_user01_probe_lens(cx: &mut AppComponentCx<'_>,",
            ],
            &[
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut AppComponentCx<'_>, st: &mut State, enabled: bool, show_user0_probe: bool, use_non_filterable_user0: bool, lens_effect: EffectId, user0_probe_effect: Option<EffectId>, show_user1_probe: bool, use_non_filterable_user1: bool, user1_probe_effect: Option<EffectId>, user01_probe_effect: Option<EffectId>, user0_image: Option<ImageId>, user1_image: Option<ImageId>,) -> AnyElement",
                "fn stage_controls(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage_controls(cx: &mut AppComponentCx<'_>, st: &mut State, enabled: bool, show_user0_probe: bool, show_user1_probe: bool, use_non_filterable_user0: bool, use_non_filterable_user1: bool,) -> AnyElement",
                "fn animated_backdrop(cx: &mut ElementContext<'_, KernelApp>) -> AnyElement",
                "fn animated_backdrop(cx: &mut AppComponentCx<'_>) -> AnyElement",
                "fn lens_row(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_row(cx: &mut AppComponentCx<'_>, enabled: bool, show_user0_probe: bool, lens_effect: EffectId, user0_probe_effect: Option<EffectId>, show_user1_probe: bool, user1_probe_effect: Option<EffectId>, user01_probe_effect: Option<EffectId>, user0_image: Option<ImageId>, user1_image: Option<ImageId>,) -> AnyElement",
                "fn lens_shell(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_shell(cx: &mut AppComponentCx<'_>, title: &'static str, radius: Px, lens_w: Px, lens_h: Px, with_effect: Option<EffectChain>,) -> AnyElement",
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
                "fn plain_lens(cx: &mut AppComponentCx<'_>, title: &'static str, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
                "fn custom_effect_lens(cx: &mut AppComponentCx<'_>, title: &'static str, effect: EffectId, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
                "fn custom_effect_user0_probe_lens(cx: &mut AppComponentCx<'_>, title: &'static str, effect: EffectId, user0_image: ImageId, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
                "fn custom_effect_user1_probe_lens(cx: &mut AppComponentCx<'_>, title: &'static str, effect: EffectId, user1_image: ImageId, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
                "fn custom_effect_user01_probe_lens(cx: &mut AppComponentCx<'_>, title: &'static str, effect: EffectId, user0_image: ImageId, user1_image: ImageId, radius: Px, lens_w: Px, lens_h: Px) -> AnyElement",
            ],
        );

        assert_advanced_helpers_prefer_app_component_cx(
            LIQUID_GLASS_DEMO,
            &[
                "fn watch_first_f32(cx: &mut AppComponentCx<'_>,",
                "let mk_card = |cx: &mut AppComponentCx<'_>,",
                "|cx: &mut AppComponentCx<'_>, label: &str, value: String|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "let mk_card = |cx: &mut ElementContext<'_, KernelApp>,",
                "|cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
            ],
        );

        assert_advanced_helpers_prefer_app_component_cx(
            POSTPROCESS_THEME_DEMO,
            &[
                "fn watch_first_f32(cx: &mut AppComponentCx<'_>,",
                "fn inspector(cx: &mut AppComponentCx<'_>, st: &mut ThemePostprocessState, theme: &str, chromatic_offset_px: f32, scanline_strength: f32, scanline_spacing_px: f32, vignette_strength: f32, grain_strength: f32, grain_scale: f32, retro_pixel_scale: f32, retro_dither: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "let label_row = |cx: &mut AppComponentCx<'_>, label: &str, value: String|",
                "fn stage(cx: &mut AppComponentCx<'_>, enabled: bool, compare: bool, theme: &str, effect: EffectId, chromatic_offset_px: f32, scanline_strength: f32, scanline_spacing_px: f32, vignette_strength: f32, grain_strength: f32, grain_scale: f32, retro_pixel_scale: f32, retro_dither: bool,) -> impl IntoUiElement<KernelApp> + use<>",
                "fn stage_body(",
                "postprocess_applied: bool,",
                "label: &str,",
                "fn stage_cards(cx: &mut AppComponentCx<'_>) -> impl IntoUiElement<KernelApp> + use<>",
                "let card = |cx: &mut AppComponentCx<'_>, title: &str, subtitle: &str|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut AppComponentCx<'_>, st: &mut ThemePostprocessState, theme: &str, chromatic_offset_px: f32, scanline_strength: f32, scanline_spacing_px: f32, vignette_strength: f32, grain_strength: f32, grain_scale: f32, retro_pixel_scale: f32, retro_dither: bool,) -> AnyElement",
                "let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut AppComponentCx<'_>, enabled: bool, compare: bool, theme: &str, effect: EffectId, chromatic_offset_px: f32, scanline_strength: f32, scanline_spacing_px: f32, vignette_strength: f32, grain_strength: f32, grain_scale: f32, retro_pixel_scale: f32, retro_dither: bool,) -> AnyElement",
                "fn stage_body(cx: &mut AppComponentCx<'_>, postprocess_applied: bool, label: &str) -> AnyElement",
                "fn stage_cards(cx: &mut AppComponentCx<'_>) -> AnyElement",
                "fn stage_body(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage_cards(cx: &mut ElementContext<'_, KernelApp>) -> AnyElement",
                "let card = |cx: &mut ElementContext<'_, KernelApp>, title: &str, subtitle: &str|",
            ],
        );

        assert_app_facing_generic_helpers_prefer_app_render_context(
            ASYNC_PLAYGROUND_DEMO,
            &[
                "use fret::app::{AppRenderContext, RenderContextAccess as _};",
                "use fret_ui_kit::IntoUiElementInExt as _;",
                "fn header_bar<'a, Cx>(",
                "fn body<'a, Cx>(",
                "fn catalog_panel<'a, Cx>(",
                "fn catalog_item<'a, Cx>(",
                "fn main_panel<'a, Cx>(",
                "fn inspector_panel<'a, Cx>(",
                "fn policy_editor<'a, Cx>(",
                "fn query_panel_for_mode<'a, Cx>(",
                "fn query_inputs_row<'a, Cx>(",
                "fn query_result_view<'a, Cx>(",
                "fn active_mode<'a, Cx>(",
                "fn query_policy<'a, Cx>(",
                "fn query_fail_mode<'a, Cx>(",
                "fn tracked_query_inputs<'a, Cx>(",
                "fn query_key_for_id<'a, Cx>(",
                "fn status_badge<'a, Cx>(",
                "fn snapshot_entry_for_key<'a, Cx>(",
                "Cx: AppRenderContext<'a>,",
                "cx.elements().pressable(",
                "let state = handle.read_layout(cx);",
            ],
            &[
                "fn header_bar(cx: &mut AppComponentCx<'_>,",
                "fn body(cx: &mut AppComponentCx<'_>,",
                "fn catalog_panel(cx: &mut AppComponentCx<'_>,",
                "fn catalog_item(cx: &mut AppComponentCx<'_>,",
                "fn main_panel(cx: &mut AppComponentCx<'_>,",
                "fn inspector_panel(cx: &mut AppComponentCx<'_>,",
                "fn policy_editor(cx: &mut AppComponentCx<'_>,",
                "fn query_panel_for_mode(cx: &mut AppComponentCx<'_>,",
                "fn query_inputs_row(cx: &mut AppComponentCx<'_>,",
                "fn query_result_view(cx: &mut AppComponentCx<'_>,",
                "fn active_mode(cx: &mut AppComponentCx<'_>,",
                "fn query_policy(cx: &mut AppComponentCx<'_>,",
                "fn query_fail_mode(cx: &mut AppComponentCx<'_>,",
                "fn tracked_query_inputs(cx: &mut AppComponentCx<'_>,",
                "fn query_key_for_id(cx: &mut AppComponentCx<'_>,",
                "fn status_badge(cx: &mut AppComponentCx<'_>,",
                "fn snapshot_entry_for_key(cx: &mut AppComponentCx<'_>,",
                "handle.layout_query(cx).value_or_default()",
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
                "fn lens_shell(cx: &mut AppComponentCx<'_>, label: Arc<str>, radius: Px, body: AnyElement) -> AnyElement",
                "fn plain_lens(cx: &mut AppComponentCx<'_>, label: impl Into<Arc<str>>, radius: Px) -> AnyElement",
                "fn custom_effect_lens(cx: &mut AppComponentCx<'_>, label: impl Into<Arc<str>>, effect: EffectId, blur_radius_px: f32, blur_downsample: f32, refraction_height_px: f32, refraction_amount_px: f32, depth_effect: f32, chromatic_aberration: f32, corner_radius_px: f32, grain_strength: f32, grain_scale: f32) -> AnyElement",
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
                "fn lens_shell(cx: &mut AppComponentCx<'_>, label: Arc<str>, radius: Px, body: AnyElement) -> AnyElement",
                "fn plain_lens(cx: &mut AppComponentCx<'_>, label: impl Into<Arc<str>>, radius: Px) -> AnyElement",
                "fn custom_effect_lens(cx: &mut AppComponentCx<'_>, label: impl Into<Arc<str>>, effect: EffectId, input_image: Option<ImageId>, sampling: ImageSamplingHint, uv_span: f32, input_strength: f32, rim_strength: f32, blur_radius_px: f32, debug_input: bool) -> AnyElement",
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
    }

    #[test]
    fn closure_local_app_facing_helpers_can_use_app_render_cx_alias() {
        assert_app_facing_concrete_helpers_prefer_app_render_cx(
            HELLO_WORLD_COMPARE_DEMO,
            &[
                "use fret_ui_kit::IntoUiElementInExt as _;",
                "cx.set_continuous_frames(self.flags.uses_continuous_frames_lease());",
                "let swatch = |_cx: &mut AppRenderCx<'_>, fill_rgb: u32, border_rgb: u32|",
                "fn hello_world_compare_root<'a, Cx>(",
                "Cx: fret::app::ElementContextAccess<'a, KernelApp>",
                "ui::text(\"Hello, World!\")",
                ".text_size_px(Px(24.0))",
                ".font_semibold()",
                ".text_align(TextAlign::Center)",
                ".nowrap()",
                ".into_element_in(cx)",
                "panel_bg: Color,",
                "children: Vec<AnyElement>)",
                "hello_world_compare_root(cx, panel_bg, children)",
            ],
            &[
                "set_continuous_frames(cx, self.flags.uses_continuous_frames_lease());",
                "let swatch = |_cx: &mut AppComponentCx<'_>, fill_rgb: u32, border_rgb: u32|",
                "let swatch = |cx: &mut ElementContext<'_, KernelApp>,",
                "let swatch = |cx: &mut AppComponentCx<'_>, fill_rgb: u32, border_rgb: u32| -> AnyElement",
                "fn hello_world_compare_root(cx: &mut AppComponentCx<'_>, panel_bg: Color, children: Vec<AnyElement>) -> Ui",
                "let cx = cx.elements();",
                "cx.text_props(TextProps {",
                ".into_element(cx)",
                "hello_world_compare_root(cx.elements(), panel_bg, children)",
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
    fn embedded_viewport_demo_prefers_capability_first_landing_with_explicit_panel_owner() {
        assert!(EMBEDDED_VIEWPORT_DEMO.contains("use fret_ui_kit::IntoUiElementInExt as _;"));

        let render = source_slice(
            EMBEDDED_VIEWPORT_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn embedded_viewport_page<'a, Cx, C>(",
        );
        let render = render.split_whitespace().collect::<String>();

        for marker in [
            ".gap(Space::N1).into_element_in(cx);",
            "[ui::text(\"640×360\").into_element_in(cx)]",
            "[ui::text(\"960×540\").into_element_in(cx)]",
            "[ui::text(\"1280×720\").into_element_in(cx)]",
            ".refine_layout(LayoutRefinement::default().flex_none()).into_element_in(cx);",
            ".panel(cx.elements(), embedded::EmbeddedViewportPanelProps {",
            ".max_w(Px(980.0)).into_element_in(cx);",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                render.contains(&marker),
                "embedded viewport demo should keep the capability-first/app-lane marker: {marker}",
            );
        }

        for legacy in [
            ".gap(Space::N1).into_element(cx);",
            "[cx.text(\"640×360\")]",
            "[cx.text(\"960×540\")]",
            "[cx.text(\"1280×720\")]",
            ".panel(cx, embedded::EmbeddedViewportPanelProps {",
            ".max_w(Px(980.0)).into_element(cx);",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !render.contains(&legacy),
                "embedded viewport demo should stay off the legacy marker: {legacy}",
            );
        }
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
                "use fret::app::{AppRenderContext, RenderContextAccess as _};",
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
            EMBEDDED_VIEWPORT_DEMO,
            &[
                "let theme = cx.theme_snapshot();",
                "fn embedded_viewport_page<'a, Cx, C>(",
                "Cx: fret::app::ElementContextAccess<'a, KernelApp>",
                "let cx = cx.elements();",
                "embedded_viewport_page(cx, theme, viewport_card, diag_enabled())",
            ],
            &["embedded_viewport_page(cx.elements(), theme, viewport_card, diag_enabled())"],
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
            IMAGE_HEAVY_MEMORY_DEMO,
            &[
                "fn render_view<'a, Cx>(cx: &mut Cx) -> Ui",
                "Cx: fret::app::ElementContextAccess<'a, KernelApp>",
                "let cx = cx.elements();",
                "render_view(cx)",
            ],
            &[
                "fn render_view(cx: &mut AppComponentCx<'_>) -> Ui",
                "render_view(cx.elements())",
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
                "fn tracked_query_inputs<'a, Cx>(cx: &mut Cx, locals: &AsyncPlaygroundLocals) -> QueryKeyInputs",
                "Cx: AppRenderContext<'a>,",
                "let query_inputs = tracked_query_inputs(cx, &locals);",
                "locals.tabs.layout_read_ref(cx, |tab| match tab.as_deref() {",
                "let policy_settings: QueryPolicySettings = cx.data().selector_layout(",
                "let state = handle.read_layout(cx);",
                "config.fail_mode.layout_value(cx)",
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
                "locals.tabs.layout_read_ref_in(cx, |tab| match tab.as_deref() {",
                "let policy_settings: QueryPolicySettings = cx.data().selector(",
                "cx.data().selector_layout(&config.fail_mode, |fail_mode| fail_mode)",
                "config.fail_mode.layout_in(cx).value_or_default()",
                "handle.layout_query(cx).value_or_default()",
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
                "fn render_view<'a, Cx>(cx: &mut Cx) -> ViewElements",
                "Cx: fret::app::ElementContextAccess<'a, KernelApp>",
                "let cx = cx.elements();",
                "render_view(cx)",
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
                "render_view(cx.elements())",
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
    fn selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref() {
        let embedded_render = source_slice(
            EMBEDDED_VIEWPORT_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn embedded_viewport_page<'a, Cx, C>(",
        );
        let embedded_render = embedded_render.split_whitespace().collect::<String>();
        assert!(
            embedded_render.contains(
                &"let window = cx.window_id();"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            embedded_render.contains(
                &"embedded::models(cx.app(), window)"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            embedded_render.contains(
                &"embedded::ensure_models(cx.app_mut(), window)"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !embedded_render.contains(
                &"let window = cx.window;"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !embedded_render.contains(
                &"embedded::models(&*cx.app, window)"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !embedded_render.contains(
                &"embedded::ensure_models(cx.app, window)"
                    .split_whitespace()
                    .collect::<String>()
            )
        );

        let async_render = source_slice(
            ASYNC_PLAYGROUND_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn header_bar<'a, Cx>(",
        );
        let async_render = async_render.split_whitespace().collect::<String>();
        assert!(
            async_render.contains(
                &"apply_theme(cx.app_mut(), dark);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            async_render.contains(
                &"cx.data().invalidate_query(key);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            async_render.contains(
                &"cx.data().cancel_query(key);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            async_render.contains(
                &"cx.data().invalidate_query_namespace(ns);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !async_render.contains(
                &"apply_theme(cx.app, dark);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !async_render.contains(
                &"with_query_client(cx.app_mut(), |client, app|"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !async_render.contains(
                &"with_query_client(cx.app_mut(), |client, _app|"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !async_render.contains(
                &"with_query_client(cx.app, |client, app|"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !async_render.contains(
                &"with_query_client(cx.app, |client, _app|"
                    .split_whitespace()
                    .collect::<String>()
            )
        );

        let markdown_render = source_slice(
            MARKDOWN_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn checkerboard_rgba8(",
        );
        let markdown_render = markdown_render.split_whitespace().collect::<String>();
        assert!(
            markdown_render.contains(
                &"cx.data().invalidate_query_namespace(REMOTE_IMAGE_NAMESPACE);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !markdown_render.contains(&"with_query_client(".split_whitespace().collect::<String>())
        );

        let api_workbench_render = source_slice(
            API_WORKBENCH_LITE_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn bind_actions(",
        );
        let api_workbench_render = api_workbench_render.split_whitespace().collect::<String>();
        assert!(
            api_workbench_render.contains(
                &"cx.app().global::<HistoryDbGlobal>()"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            api_workbench_render.contains(
                &"shadcn::Dialog::new(&locals.settings_open).into_element_in("
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !api_workbench_render.contains(
                &"cx.app.global::<HistoryDbGlobal>()"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !api_workbench_render.contains(
                &"shadcn::Dialog::new(&locals.settings_open).into_element("
                    .split_whitespace()
                    .collect::<String>()
            )
        );

        assert!(
            EMOJI_CONFORMANCE_DEMO
                .split_whitespace()
                .collect::<String>()
                .contains(
                    &"cx.app().global::<FontCatalogCache>()"
                        .split_whitespace()
                        .collect::<String>()
                )
        );
        assert!(
            !EMOJI_CONFORMANCE_DEMO
                .split_whitespace()
                .collect::<String>()
                .contains(
                    &"cx.app.global::<FontCatalogCache>()"
                        .split_whitespace()
                        .collect::<String>()
                )
        );

        let postprocess_render = source_slice(
            POSTPROCESS_THEME_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn srgb(",
        );
        let postprocess_render = postprocess_render.split_whitespace().collect::<String>();
        assert!(
            postprocess_render.contains(
                &"cx.app().global::<DemoEffect>()"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !postprocess_render.contains(
                &"cx.app.global::<DemoEffect>()"
                    .split_whitespace()
                    .collect::<String>()
            )
        );

        let genui_render = source_slice(
            GENUI_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut GenUiState) -> ViewElements {",
        );
        let genui_render = genui_render.split_whitespace().collect::<String>();
        assert!(
            genui_render.contains(
                &"Self::handle_msg(cx.app_mut(), &mut self.st, Msg::ClearActions);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            genui_render.contains(
                &"Self::handle_msg(cx.app_mut(), &mut self.st, Msg::AutoApplyToggled);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !genui_render.contains(
                &"Self::handle_msg(cx.app, &mut self.st, Msg::ClearActions);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !genui_render.contains(
                &"Self::handle_msg(cx.app, &mut self.st, Msg::AutoApplyToggled);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );

        let runtime_sample_fn = source_slice(
            HELLO_WORLD_COMPARE_DEMO,
            "fn update_runtime_frame_sample_state(cx: &mut AppUi<'_, '_>) {",
            "fn capture_runtime_frame_sample_json(",
        );
        let runtime_sample_fn = runtime_sample_fn.split_whitespace().collect::<String>();
        assert!(
            runtime_sample_fn.contains(
                &"let window = cx.window_id();"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            runtime_sample_fn.contains(
                &"state.last_frame_id = cx.app().frame_id().0;"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            runtime_sample_fn.contains(
                &"capture_element_runtime_frame_sample(cx.app_mut(),"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !runtime_sample_fn.contains(
                &"let window = cx.window;"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !runtime_sample_fn.contains(
                &"state.last_frame_id = cx.app.frame_id().0;"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !runtime_sample_fn.contains(
                &"capture_element_runtime_frame_sample(cx.app, window);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
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
