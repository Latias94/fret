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
    const IMUI_EDITOR_PROOF_DEMO_COLLECTION_MODULE: &str =
        include_str!("imui_editor_proof_demo/collection.rs");
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
    const IMUI_IMGUI_PARITY_AUDIT_V2: &str =
        include_str!("../../../docs/workstreams/standalone/imui-imgui-parity-audit-v2.md");
    const IMUI_RESPONSE_STATUS_LIFECYCLE_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md");
    const IMUI_RESPONSE_STATUS_LIFECYCLE_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json");
    const IMUI_KEY_OWNER_SURFACE_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-key-owner-surface-v1/DESIGN.md");
    const IMUI_KEY_OWNER_SURFACE_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md"
    );
    const IMUI_KEY_OWNER_SURFACE_M2_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md"
    );
    const IMUI_KEY_OWNER_SURFACE_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md"
    );
    const IMUI_KEY_OWNER_SURFACE_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_PANE_PROOF_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-pane-proof-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md"
    );
    const IMUI_COLLECTION_PANE_PROOF_M2_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-pane-proof-v1/M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md"
    );
    const IMUI_COLLECTION_PANE_PROOF_M3_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-pane-proof-v1/M3_PANE_PROOF_CLOSURE_2026-04-21.md"
    );
    const IMUI_COLLECTION_PANE_PROOF_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md"
    );
    const IMUI_COLLECTION_PANE_PROOF_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-collection-pane-proof-v1/WORKSTREAM.json");
    const IMUI_CHILD_REGION_DEPTH_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-child-region-depth-v1/DESIGN.md");
    const IMUI_CHILD_REGION_DEPTH_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md"
    );
    const IMUI_CHILD_REGION_DEPTH_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-child-region-depth-v1/M1_TARGET_SURFACE_FREEZE_2026-04-22.md"
    );
    const IMUI_CHILD_REGION_DEPTH_M2_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-child-region-depth-v1/M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md"
    );
    const IMUI_CHILD_REGION_DEPTH_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md"
    );
    const IMUI_CHILD_REGION_DEPTH_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_BOX_SELECT_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-box-select-v1/DESIGN.md");
    const IMUI_COLLECTION_BOX_SELECT_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md"
    );
    const IMUI_COLLECTION_BOX_SELECT_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md"
    );
    const IMUI_COLLECTION_BOX_SELECT_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md"
    );
    const IMUI_COLLECTION_BOX_SELECT_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_KEYBOARD_OWNER_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md");
    const IMUI_COLLECTION_KEYBOARD_OWNER_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md"
    );
    const IMUI_COLLECTION_KEYBOARD_OWNER_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md"
    );
    const IMUI_COLLECTION_KEYBOARD_OWNER_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md"
    );
    const IMUI_COLLECTION_KEYBOARD_OWNER_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_DELETE_ACTION_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-delete-action-v1/DESIGN.md");
    const IMUI_COLLECTION_DELETE_ACTION_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md"
    );
    const IMUI_COLLECTION_DELETE_ACTION_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md"
    );
    const IMUI_COLLECTION_DELETE_ACTION_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md"
    );
    const IMUI_COLLECTION_DELETE_ACTION_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_CONTEXT_MENU_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-context-menu-v1/DESIGN.md");
    const IMUI_COLLECTION_CONTEXT_MENU_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_CONTEXT_MENU_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md"
    );
    const IMUI_COLLECTION_CONTEXT_MENU_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_CONTEXT_MENU_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-collection-context-menu-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_ZOOM_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-zoom-v1/DESIGN.md");
    const IMUI_COLLECTION_ZOOM_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-zoom-v1/M0_BASELINE_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_ZOOM_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-zoom-v1/M1_APP_OWNED_ZOOM_LAYOUT_SLICE_2026-04-23.md"
    );
    const IMUI_COLLECTION_ZOOM_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-zoom-v1/CLOSEOUT_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_ZOOM_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-collection-zoom-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_SELECT_ALL_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-select-all-v1/DESIGN.md");
    const IMUI_COLLECTION_SELECT_ALL_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-select-all-v1/M0_BASELINE_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_SELECT_ALL_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-select-all-v1/M1_APP_OWNED_SELECT_ALL_SLICE_2026-04-23.md"
    );
    const IMUI_COLLECTION_SELECT_ALL_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-select-all-v1/CLOSEOUT_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_SELECT_ALL_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-collection-select-all-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_RENAME_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-rename-v1/DESIGN.md");
    const IMUI_COLLECTION_RENAME_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_RENAME_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-rename-v1/M1_APP_OWNED_RENAME_SLICE_2026-04-23.md"
    );
    const IMUI_COLLECTION_RENAME_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_RENAME_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-collection-rename-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_INLINE_RENAME_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-inline-rename-v1/DESIGN.md");
    const IMUI_COLLECTION_INLINE_RENAME_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-inline-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_INLINE_RENAME_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-inline-rename-v1/M1_APP_OWNED_INLINE_RENAME_SLICE_2026-04-23.md"
    );
    const IMUI_COLLECTION_INLINE_RENAME_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_INLINE_RENAME_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-collection-inline-rename-v1/WORKSTREAM.json");
    const IMUI_COLLECTION_MODULARIZATION_DESIGN: &str = include_str!(
        "../../../docs/workstreams/imui-editor-proof-collection-modularization-v1/DESIGN.md"
    );
    const IMUI_COLLECTION_MODULARIZATION_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-proof-collection-modularization-v1/M0_BASELINE_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_MODULARIZATION_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-proof-collection-modularization-v1/M1_DEMO_LOCAL_COLLECTION_MODULE_SLICE_2026-04-23.md"
    );
    const IMUI_COLLECTION_MODULARIZATION_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_MODULARIZATION_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-editor-proof-collection-modularization-v1/WORKSTREAM.json"
    );
    const IMUI_COLLECTION_COMMAND_PACKAGE_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-command-package-v1/DESIGN.md");
    const IMUI_COLLECTION_COMMAND_PACKAGE_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-command-package-v1/M0_BASELINE_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_COMMAND_PACKAGE_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-command-package-v1/M1_APP_OWNED_DUPLICATE_COMMAND_SLICE_2026-04-23.md"
    );
    const IMUI_COLLECTION_COMMAND_PACKAGE_M2_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-command-package-v1/M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md"
    );
    const IMUI_COLLECTION_COMMAND_PACKAGE_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_COMMAND_PACKAGE_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-collection-command-package-v1/WORKSTREAM.json"
    );
    const IMUI_COLLECTION_SECOND_PROOF_SURFACE_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md");
    const IMUI_COLLECTION_SECOND_PROOF_SURFACE_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-second-proof-surface-v1/M0_BASELINE_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_SECOND_PROOF_SURFACE_M2_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-second-proof-surface-v1/M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md"
    );
    const IMUI_COLLECTION_SECOND_PROOF_SURFACE_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md"
    );
    const IMUI_COLLECTION_SECOND_PROOF_SURFACE_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-collection-second-proof-surface-v1/WORKSTREAM.json"
    );
    const IMUI_COLLECTION_HELPER_READINESS_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-collection-helper-readiness-v1/DESIGN.md");
    const IMUI_COLLECTION_HELPER_READINESS_TODO: &str =
        include_str!("../../../docs/workstreams/imui-collection-helper-readiness-v1/TODO.md");
    const IMUI_COLLECTION_HELPER_READINESS_MILESTONES: &str =
        include_str!("../../../docs/workstreams/imui-collection-helper-readiness-v1/MILESTONES.md");
    const IMUI_COLLECTION_HELPER_READINESS_EVIDENCE: &str = include_str!(
        "../../../docs/workstreams/imui-collection-helper-readiness-v1/EVIDENCE_AND_GATES.md"
    );
    const IMUI_COLLECTION_HELPER_READINESS_M1_AUDIT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-helper-readiness-v1/M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md"
    );
    const IMUI_COLLECTION_HELPER_READINESS_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-collection-helper-readiness-v1/CLOSEOUT_AUDIT_2026-04-24.md"
    );
    const IMUI_COLLECTION_HELPER_READINESS_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-collection-helper-readiness-v1/WORKSTREAM.json"
    );
    const IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-editor-notes-inspector-command-v1/DESIGN.md");
    const IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-inspector-command-v1/M1_APP_OWNED_SUMMARY_COMMAND_SLICE_2026-04-24.md"
    );
    const IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-inspector-command-v1/CLOSEOUT_AUDIT_2026-04-24.md"
    );
    const IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-inspector-command-v1/WORKSTREAM.json"
    );
    const IMUI_EDITOR_NOTES_DIRTY_STATUS_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-editor-notes-dirty-status-v1/DESIGN.md");
    const IMUI_EDITOR_NOTES_DIRTY_STATUS_TODO: &str =
        include_str!("../../../docs/workstreams/imui-editor-notes-dirty-status-v1/TODO.md");
    const IMUI_EDITOR_NOTES_DIRTY_STATUS_MILESTONES: &str =
        include_str!("../../../docs/workstreams/imui-editor-notes-dirty-status-v1/MILESTONES.md");
    const IMUI_EDITOR_NOTES_DIRTY_STATUS_EVIDENCE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-dirty-status-v1/EVIDENCE_AND_GATES.md"
    );
    const IMUI_EDITOR_NOTES_DIRTY_STATUS_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-dirty-status-v1/M1_APP_OWNED_DRAFT_STATUS_SLICE_2026-04-24.md"
    );
    const IMUI_EDITOR_NOTES_DIRTY_STATUS_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-dirty-status-v1/CLOSEOUT_AUDIT_2026-04-24.md"
    );
    const IMUI_EDITOR_NOTES_DIRTY_STATUS_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-editor-notes-dirty-status-v1/WORKSTREAM.json");
    const IMUI_NEXT_GAP_AUDIT_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-next-gap-audit-v1/DESIGN.md");
    const IMUI_NEXT_GAP_AUDIT_TODO: &str =
        include_str!("../../../docs/workstreams/imui-next-gap-audit-v1/TODO.md");
    const IMUI_NEXT_GAP_AUDIT_MILESTONES: &str =
        include_str!("../../../docs/workstreams/imui-next-gap-audit-v1/MILESTONES.md");
    const IMUI_NEXT_GAP_AUDIT_EVIDENCE: &str =
        include_str!("../../../docs/workstreams/imui-next-gap-audit-v1/EVIDENCE_AND_GATES.md");
    const IMUI_NEXT_GAP_AUDIT_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-next-gap-audit-v1/M1_NEXT_GAP_AUDIT_2026-04-24.md"
    );
    const IMUI_NEXT_GAP_AUDIT_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-next-gap-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md"
    );
    const IMUI_NEXT_GAP_AUDIT_WORKSTREAM: &str =
        include_str!("../../../docs/workstreams/imui-next-gap-audit-v1/WORKSTREAM.json");
    const IMUI_EDITOR_NOTES_DRAFT_ACTIONS_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-editor-notes-draft-actions-v1/DESIGN.md");
    const IMUI_EDITOR_NOTES_DRAFT_ACTIONS_TODO: &str =
        include_str!("../../../docs/workstreams/imui-editor-notes-draft-actions-v1/TODO.md");
    const IMUI_EDITOR_NOTES_DRAFT_ACTIONS_MILESTONES: &str =
        include_str!("../../../docs/workstreams/imui-editor-notes-draft-actions-v1/MILESTONES.md");
    const IMUI_EDITOR_NOTES_DRAFT_ACTIONS_EVIDENCE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-draft-actions-v1/EVIDENCE_AND_GATES.md"
    );
    const IMUI_EDITOR_NOTES_DRAFT_ACTIONS_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-draft-actions-v1/M1_APP_OWNED_DRAFT_ACTIONS_SLICE_2026-04-24.md"
    );
    const IMUI_EDITOR_NOTES_DRAFT_ACTIONS_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-draft-actions-v1/CLOSEOUT_AUDIT_2026-04-24.md"
    );
    const IMUI_EDITOR_NOTES_DRAFT_ACTIONS_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-editor-notes-draft-actions-v1/WORKSTREAM.json"
    );
    const IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_DESIGN: &str = include_str!(
        "../../../docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/DESIGN.md"
    );
    const IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_TODO: &str = include_str!(
        "../../../docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/TODO.md"
    );
    const IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_MILESTONES: &str = include_str!(
        "../../../docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/MILESTONES.md"
    );
    const IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_EVIDENCE: &str = include_str!(
        "../../../docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/EVIDENCE_AND_GATES.md"
    );
    const IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/M1_DRAFT_BUFFER_CONTRACT_AUDIT_2026-04-24.md"
    );
    const IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md"
    );
    const IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/WORKSTREAM.json"
    );
    const IMUI_TEXT_FIELD_RS: &str =
        include_str!("../../../ecosystem/fret-ui-editor/src/controls/text_field.rs");
    const IMUI_FACADE_INTERNAL_MODULARIZATION_DESIGN: &str =
        include_str!("../../../docs/workstreams/imui-facade-internal-modularization-v1/DESIGN.md");
    const IMUI_FACADE_INTERNAL_MODULARIZATION_M0_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-facade-internal-modularization-v1/M0_BASELINE_AUDIT_2026-04-21.md"
    );
    const IMUI_FACADE_INTERNAL_MODULARIZATION_M1_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-facade-internal-modularization-v1/M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md"
    );
    const IMUI_FACADE_INTERNAL_MODULARIZATION_M2_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md"
    );
    const IMUI_FACADE_INTERNAL_MODULARIZATION_M3_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md"
    );
    const IMUI_FACADE_INTERNAL_MODULARIZATION_M4_NOTE: &str = include_str!(
        "../../../docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md"
    );
    const IMUI_FACADE_INTERNAL_MODULARIZATION_CLOSEOUT: &str = include_str!(
        "../../../docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md"
    );
    const IMUI_FACADE_INTERNAL_MODULARIZATION_WORKSTREAM: &str = include_str!(
        "../../../docs/workstreams/imui-facade-internal-modularization-v1/WORKSTREAM.json"
    );
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
    const IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_NEXT_PRIORITY: &str = include_str!(
        "../../../docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md"
    );
    const IMUI_RESPONSE_STATUS_LIFECYCLE_TODO: &str =
        include_str!("../../../docs/workstreams/imui-response-status-lifecycle-v1/TODO.md");
    const WORKSTREAMS_INDEX_DOC: &str = include_str!("../../../docs/workstreams/README.md");
    const ROADMAP_DOC: &str = include_str!("../../../docs/roadmap.md");
    const TODO_TRACKER_DOC: &str = include_str!("../../../docs/todo-tracker.md");
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
    const DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_AUTOMATION_DECISION_NOTE: &str = include_str!(
        "../../../docs/workstreams/docking-multiwindow-imgui-parity/M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md"
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
    const IMUI_SHADCN_ADAPTER_DISCOVERABILITY_SCRIPT: &str = include_str!(
        "../../../tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-control-discoverability.json"
    );
    const IMUI_EDITOR_PROOF_APP_OWNER_AUDIT: &str = include_str!(
        "../../../docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/IMUI_EDITOR_PROOF_APP_OWNER_AUDIT_2026-04-16.md"
    );
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
        let legacy_page_sig = format!("fn {page_fn}(cx: &mut AppComponentCx<'_>,");
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

    fn assert_source_slice_keeps_raw_driver_owner(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                normalized.contains(&marker),
                "missing raw driver-owner marker: {marker}"
            );
        }
        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "unexpected non-driver marker present: {marker}"
            );
        }
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
        assert!(TODO_DEMO.contains("use fret::env::{"));
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
            "usefret::env::{ViewportQueryHysteresis,primary_pointer_can_hover,viewport_tailwind,viewport_width_at_least,};"
        ));
        assert!(!normalized.contains(
            "usefret_ui_kit::declarative::{ElementContextThemeExtas_,ViewportQueryHysteresis,primary_pointer_can_hover,viewport_tailwind,viewport_width_at_least,};"
        ));
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
    fn todo_demo_prefers_capability_first_landing_for_root_builders() {
        let render = source_slice(
            TODO_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn todo_page(",
        );
        let render = render.split_whitespace().collect::<String>();

        for marker in [
            "ui::text(\"Add a task to get started\").text_sm().text_color(ColorRef::Color(muted_foreground)).into_element_in(cx)",
            ".gap(Space::N1).items_center().into_element_in(cx)",
            "ui::text(format!(\"{active_count} {task_label} left\")).text_sm().text_color(ColorRef::Color(muted_foreground)).into_element_in(cx)",
            ".gap(Space::N1p5).w_full().into_element_in(cx)",
            "shadcn::ScrollArea::new([rows_body.into_element_in(cx)])",
            ".min_h_0().build().into_element_in(cx);",
            "let footer = if responsive.stack_footer {",
            "children }).gap(Space::N2).items_stretch().w_full().into_element_in(cx)",
            "children }).gap(Space::N3).items_center().justify_between().w_full().into_element_in(cx)",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                render.contains(&marker),
                "todo demo should keep the capability-first root marker: {marker}",
            );
        }

        for legacy in [
            "ui::text(\"Add a task to get started\").text_sm().text_color(ColorRef::Color(muted_foreground)).into_element(cx)",
            ".gap(Space::N1).items_center().into_element(cx)",
            "ui::text(format!(\"{active_count} {task_label} left\")).text_sm().text_color(ColorRef::Color(muted_foreground)).into_element(cx)",
            ".gap(Space::N1p5).w_full().into_element(cx)",
            "shadcn::ScrollArea::new([rows_body.into_element(cx)])",
            ".min_h_0().build().into_element(cx);",
            "children }).gap(Space::N2).items_stretch().w_full().into_element(cx)",
            "children }).gap(Space::N3).items_center().justify_between().w_full().into_element(cx)",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !render.contains(&legacy),
                "todo demo should stay off the legacy root marker: {legacy}",
            );
        }
    }

    #[test]
    fn async_playground_demo_prefers_app_render_context_helpers_and_root_capability_landing() {
        assert_app_facing_generic_helpers_prefer_app_render_context(
            ASYNC_PLAYGROUND_DEMO,
            &[
                "use fret::app::{AppRenderContext, RenderContextAccess as _};",
                "use fret_ui_kit::IntoUiElementInExt as _;",
                "fn tracked_query_inputs<'a, Cx>(",
                "fn header_bar<'a, Cx>(",
                "fn body<'a, Cx>(",
                "fn query_panel_for_mode<'a, Cx>(",
                "fn status_badge<'a, Cx>(",
                "Cx: AppRenderContext<'a>,",
                "cx.elements().pressable(",
                "let state = handle.read_layout(cx);",
                "locals.tabs.layout_read_ref(cx, |tab| match tab.as_deref() {",
                "config.fail_mode.layout_value(cx)",
            ],
            &[
                "fn tracked_query_inputs(cx: &mut AppComponentCx<'_>,",
                "fn header_bar(cx: &mut AppComponentCx<'_>,",
                "fn body(cx: &mut AppComponentCx<'_>,",
                "fn query_panel_for_mode(cx: &mut AppComponentCx<'_>,",
                "fn status_badge(cx: &mut AppComponentCx<'_>,",
                "handle.layout_query(cx).value_or_default()",
                "locals.tabs.layout_read_ref_in(cx, |tab| match tab.as_deref() {",
                "config.fail_mode.layout_value_in(cx)",
            ],
        );

        let render = source_slice(
            ASYNC_PLAYGROUND_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn header_bar<'a, Cx>(",
        );
        let render = render.split_whitespace().collect::<String>();

        for marker in [
            "let query_inputs = tracked_query_inputs(cx, &locals);",
            "let header = header_bar(cx, &locals, theme.clone(), global_slow, dark);",
            "let body = body(cx, &mut self.st, &locals, theme, global_slow, selected);",
            "ui::v_flex(|_cx| [header, body]).w_full().h_full().into_element_in(cx).into()",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                render.contains(&marker),
                "async playground should keep the capability-first root marker: {marker}",
            );
        }

        for legacy in [
            "let header = header_bar(cx, &locals, theme.clone(), global_slow, dark).into_element(cx);",
            "let body = body(cx, &mut self.st, &locals, theme, global_slow, selected).into_element(cx);",
            "ui::v_flex(|_cx| [header, body]).w_full().h_full().into_element(cx).into()",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !render.contains(&legacy),
                "async playground should stay off the legacy root marker: {legacy}",
            );
        }
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
    fn query_demos_prefer_capability_first_landing_for_root_detail_builders() {
        let query_demo = QUERY_DEMO.split_whitespace().collect::<String>();
        let query_async = QUERY_ASYNC_TOKIO_DEMO
            .split_whitespace()
            .collect::<String>();

        for marker in [
            "use fret_ui_kit::IntoUiElementInExt as _;",
            "}).gap(Space::N2).items_center().into_element_in(cx);",
            "}).gap(Space::N2).into_element_in(cx);",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                query_demo.contains(&marker),
                "query demo should keep the capability-first landing marker: {marker}",
            );
            assert!(
                query_async.contains(&marker),
                "query async tokio demo should keep the capability-first landing marker: {marker}",
            );
        }

        for legacy in [
            "}).gap(Space::N2).items_center().into_element(cx);",
            "}).gap(Space::N2).into_element(cx);",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !query_demo.contains(&legacy),
                "query demo should stay off the legacy landing marker: {legacy}",
            );
            assert!(
                !query_async.contains(&legacy),
                "query async tokio demo should stay off the legacy landing marker: {legacy}",
            );
        }
    }

    #[test]
    fn api_workbench_lite_demo_uses_query_for_sqlite_reads_and_mutation_for_explicit_submit() {
        assert!(API_WORKBENCH_LITE_DEMO.contains("use fret::app::prelude::*;"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("advanced::prelude::*"));
        assert!(
            API_WORKBENCH_LITE_DEMO.contains("fn init(_app: &mut App, window: WindowId) -> Self")
        );
        assert!(API_WORKBENCH_LITE_DEMO.contains("Cx: AppRenderContext<'a>,"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("fn shell_frame(\n    cx: &mut AppUi<'_, '_>,"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("fn request_panel(cx: &mut AppUi<'_, '_>,"));
        assert!(!API_WORKBENCH_LITE_DEMO.contains("fn response_panel(cx: &mut AppUi<'_, '_>,"));
        let api_workbench = API_WORKBENCH_LITE_DEMO
            .split_whitespace()
            .collect::<String>();
        assert!(
            api_workbench.contains(
                &"cx.app().global::<HistoryDbGlobal>()"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !api_workbench.contains(
                &"cx.app.global::<HistoryDbGlobal>()"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            api_workbench.contains(
                &"shadcn::Dialog::new(&locals.settings_open).into_element_in("
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !api_workbench.contains(
                &"shadcn::Dialog::new(&locals.settings_open).into_element("
                    .split_whitespace()
                    .collect::<String>()
            )
        );
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
        assert!(!HELLO_COUNTER_DEMO.contains("fn hello_counter_page(cx: &mut AppComponentCx<'_>,"));
        assert!(!HELLO_COUNTER_DEMO.contains(".test_id(TEST_ID_ROOT).into_element(cx).into()"));
        assert!(!HELLO_COUNTER_DEMO.contains("Theme::global(&*cx.app).snapshot()"));
    }

    #[test]
    fn hello_counter_demo_prefers_app_lane_text_builders_and_capability_first_landing() {
        let hello_counter = HELLO_COUNTER_DEMO.split_whitespace().collect::<String>();
        for marker in [
            "use fret_ui_kit::IntoUiElementInExt as _;",
            "ui::text(count.to_string())",
            "ui::text(status_text)",
            "ui::text_block(if step_valid {",
            ".submit_action(inc_cmd).into_element_in(cx)",
            ".max_w(Px(480.0)).into_element_in(cx)",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                hello_counter.contains(&marker),
                "hello counter demo should keep the app-lane marker: {marker}",
            );
        }

        for legacy in [
            "let count_text = cx.text_props(",
            "let status_line = cx.text_props(",
            "let step_help = cx.text_props(",
            ".submit_action(inc_cmd).into_element(cx)",
            ".max_w(Px(480.0)).into_element(cx)",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !hello_counter.contains(&legacy),
                "hello counter demo should stay off legacy raw marker: {legacy}",
            );
        }
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
    fn renderer_theme_bridge_proofs_keep_explicit_host_theme_reads() {
        for src in [POSTPROCESS_THEME_DEMO, LIQUID_GLASS_DEMO] {
            assert!(src.contains("Theme::global(&*cx.app).snapshot()"));
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
                "use fret::advanced::view::AppRenderDataExt as _;",
                "let show = cx.data().selector_model_layout(&st.show, |show| show);",
                "let theme = cx.theme().snapshot();",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-texture-imports-root\"),",
            ],
            &[
                "fn external_texture_imports_root(",
                "cx.observe_model(&st.show, Invalidation::Layout);",
                "cx.app.models().read(&st.show, |v| *v).unwrap_or(true)",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_TEXTURE_IMPORTS_WEB_DEMO,
            &[
                "use fret::advanced::view::AppRenderDataExt as _;",
                "let show = cx.data().selector_model_layout(&show_model, |show| show);",
                "let theme = cx.theme().snapshot();",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-texture-imports-web-root\"),",
                "make_panel(cx, fret_core::ViewportFit::Contain, \"ext-tex-web-contain\")",
            ],
            &[
                "fn external_texture_imports_web_root(",
                "cx.observe_model(&show_model, Invalidation::Layout);",
                "cx.app.models().read(&show_model, |v| *v).unwrap_or(true)",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_VIDEO_IMPORTS_AVF_DEMO,
            &[
                "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsAvfView) -> fret::Ui",
                "use fret::advanced::view::AppRenderDataExt as _;",
                "let show = cx.data().selector_model_layout(&st.show, |show| show);",
                "let theme = cx.theme().snapshot();",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-video-imports-avf-root\"),",
            ],
            &[
                "fn external_video_imports_avf_root(",
                "cx.observe_model(&st.show, Invalidation::Layout);",
                "cx.app.models().read(&st.show, |v| *v).unwrap_or(true)",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_VIDEO_IMPORTS_MF_DEMO,
            &[
                "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsMfView) -> fret::Ui",
                "use fret::advanced::view::AppRenderDataExt as _;",
                "let show = cx.data().selector_model_layout(&st.show, |show| show);",
                "let theme = cx.theme().snapshot();",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-video-imports-mf-root\"),",
            ],
            &[
                "fn external_video_imports_mf_root(",
                "cx.observe_model(&st.show, Invalidation::Layout);",
                "cx.app.models().read(&st.show, |v| *v).unwrap_or(true)",
                "Theme::global(&*cx.app).snapshot()",
            ],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            CHART_DECLARATIVE_DEMO,
            &[
                "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
                "let _ = self.engine.paint(cx).read_ref(|_| ());",
                "chart_canvas_panel_in(cx, props).into()",
            ],
            &[
                "fn chart_declarative_root(",
                "cx.elements().observe_model(&self.engine, Invalidation::Paint);",
                "chart_canvas_panel(cx.elements(), props).into()",
            ],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            NODE_GRAPH_DEMO,
            &[
                "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
                "self.surface.observe_in(cx);",
                "node_graph_surface_in(cx, props).into()",
            ],
            &[
                "fn node_graph_root(",
                "self.surface.observe(cx.elements());",
                "node_graph_surface(cx.elements(), props).into()",
            ],
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
                "let cx = cx.elements();",
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
                "let theme = cx.theme_snapshot();",
                "let theme_name = cx.theme().name.clone();",
                "let theme = cx.theme();",
                "let state_revision = table_state.layout(cx).revision().unwrap_or(0);",
                "let selected = tree_state.layout(cx).read_ref(|s| s.selected).ok().flatten();",
                "let checkbox_value = checkbox.layout(cx).copied_or(false);",
                "let selected_emoji_font = emoji_font_override.layout(cx).value_or_default();",
                "let last_action_value = last_action.layout(cx).value_or_else(",
            ],
            &[
                "move |cx: &mut ElementContext<'_, App>, col: &ColumnDef<u64>, row: &u64| {",
                ".render_root(\"components-gallery\", |cx| {",
                "cx.observe_model(&tree_state, Invalidation::Layout);",
                "cx.app.models().revision(&table_state).unwrap_or(0);",
                "cx.app.models().get_copied(&checkbox).unwrap_or(false);",
                "cx.app.models().get_cloned(&last_action);",
                "cx.app.models().read(&emoji_font_override, |v| v.clone())",
                "Theme::global(&*cx.app)",
            ],
        );
    }

    #[test]
    fn components_gallery_keeps_retained_render_and_driver_owner_split() {
        let normalized = COMPONENTS_GALLERY_DEMO
            .split_whitespace()
            .collect::<String>();
        for marker in [
            "impl ComponentsGalleryWindowState {",
            "fn selected_theme_preset(&self, app: &App) -> Option<Arc<str>> {",
            "app.models().get_cloned(&self.theme_preset).flatten()",
            "fn overlays_open(&self, app: &App) -> bool {",
            "app.models().get_copied(&self.select_open).unwrap_or(false)",
            "app.models().get_copied(&self.cmdk_open).unwrap_or(false)",
            "let preset = state.selected_theme_preset(app);",
            "let state_revision = table_state.layout(cx).revision().unwrap_or(0);",
            "let items_revision = 1 ^ state_revision.rotate_left(17);",
            "let items_value = app.models().get_cloned(&items).unwrap_or_default();",
            "let tree_state_value = app.models().get_cloned(&state).unwrap_or_default();",
            "let overlays_open = state.overlays_open(app);",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for legacy in ["cx.app.models().revision(&table_state).unwrap_or(0);"] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&legacy),
                "legacy marker still present: {legacy}"
            );
        }

        for marker in [
            "`components_gallery` is not one unresolved raw-model bucket anymore.",
            "retained render owner",
            "driver/event owner",
            "`table_state.layout(cx).revision()`",
            "`selected_theme_preset(app)`",
            "`overlays_open(app)`",
        ] {
            assert!(
                COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT.contains(marker),
                "components gallery owner audit should remain explicit: {marker}"
            );
        }
    }

    #[test]
    fn selected_raw_owner_examples_keep_escape_hatches_explicit() {
        let components_gallery = COMPONENTS_GALLERY_DEMO
            .split_whitespace()
            .collect::<String>();
        assert!(
            components_gallery.contains("letcx=cx.elements();"),
            "components gallery retained branch should spell the raw retained owner explicitly",
        );
        assert!(
            components_gallery.contains(
                &"let theme = cx.theme_snapshot();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "components gallery should keep theme snapshot reads on AppUi before entering the raw builder phase",
        );
        assert!(
            components_gallery.contains(
                &"let last_action_value = last_action.layout(cx).value_or_else(|| Arc::<str>::from(\"<none>\"));
                let cx = cx.elements();
                let theme_name = cx.theme().name.clone();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "components gallery should keep AppUi tracked reads ahead of an explicit raw builder phase",
        );

        assert!(EDITOR_NOTES_DEVICE_SHELL_DEMO.contains(
            "let (name_value, committed_notes, notes_outcome) = cx.data().selector_model_paint("
        ));
        for legacy in [
            ".watch_model(&asset.name_model)",
            ".watch_model(&asset.notes_model)",
            ".watch_model(&asset.notes_outcome_model)",
        ] {
            assert!(
                !EDITOR_NOTES_DEVICE_SHELL_DEMO.contains(legacy),
                "editor notes device shell should stay off legacy watch_model reads: {legacy}",
            );
        }

        let emoji_conformance = EMOJI_CONFORMANCE_DEMO
            .split_whitespace()
            .collect::<String>();
        assert!(
            emoji_conformance.contains(
                &"let cx = cx.elements();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "emoji conformance should keep raw text/builder authoring on an explicit elements lane",
        );

        let form_demo = FORM_DEMO.split_whitespace().collect::<String>();
        assert!(
            form_demo.contains(
                &"let cx = cx.elements();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "form demo should keep manual raw builder/container authoring on an explicit elements lane",
        );

        let date_picker_demo = DATE_PICKER_DEMO.split_whitespace().collect::<String>();
        assert!(
            date_picker_demo.contains(
                &"let cx = cx.elements();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "date picker demo should keep manual raw builder/container authoring on an explicit elements lane",
        );

        let imui_interaction_showcase = IMUI_INTERACTION_SHOWCASE_DEMO
            .split_whitespace()
            .collect::<String>();
        assert!(
            imui_interaction_showcase.contains(
                &"let cx = cx.elements();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "imui interaction showcase should keep advanced shell helper rendering on an explicit elements lane",
        );

        let postprocess_theme = POSTPROCESS_THEME_DEMO
            .split_whitespace()
            .collect::<String>();
        assert!(
            postprocess_theme.contains(
                &"let cx = cx.elements();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "postprocess theme demo should keep advanced renderer/theme helper rendering on an explicit elements lane",
        );
        assert!(
            postprocess_theme.contains(
                &"shadcn::raw::typography::h3(\"Custom effects unavailable\").into_element_in(cx)"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "postprocess theme demo should keep the unavailable fallback on the capability-first landing lane",
        );

        let drop_shadow_demo = DROP_SHADOW_DEMO.split_whitespace().collect::<String>();
        assert!(
            drop_shadow_demo.contains(
                &"let cx = cx.elements();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "drop shadow demo should keep advanced container/effect authoring on an explicit elements lane",
        );

        let ime_smoke_demo = IME_SMOKE_DEMO.split_whitespace().collect::<String>();
        assert!(
            ime_smoke_demo.contains(
                &"let cx = cx.elements();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "ime smoke demo should keep manual container/flex/text authoring on an explicit elements lane",
        );

        let sonner_demo = SONNER_DEMO.split_whitespace().collect::<String>();
        assert!(
            sonner_demo.contains(
                &"let cx = cx.elements();"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "sonner demo should keep manual flex/text/toaster authoring on an explicit elements lane",
        );

        for (label, src) in [
            ("custom effect v1", CUSTOM_EFFECT_V1_DEMO),
            ("custom effect v2", CUSTOM_EFFECT_V2_DEMO),
            ("custom effect v3", CUSTOM_EFFECT_V3_DEMO),
            ("liquid glass", LIQUID_GLASS_DEMO),
        ] {
            let normalized = src.split_whitespace().collect::<String>();
            assert!(
                normalized.contains(
                    &"view(cx.elements(), &mut st)"
                        .split_whitespace()
                        .collect::<String>()
                ),
                "{label} should enter its advanced raw view helper through an explicit elements lane",
            );
            assert!(
                !normalized.contains(&"view(cx, &mut st)".split_whitespace().collect::<String>()),
                "{label} should not rely on implicit AppUi -> ElementContext coercion",
            );
        }

        let genui_demo = GENUI_DEMO.split_whitespace().collect::<String>();
        assert!(
            genui_demo.contains(
                &"view(cx.elements(), &mut self.st)"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "genui demo should enter its advanced raw view helper through an explicit elements lane",
        );
        assert!(
            !genui_demo.contains(
                &"view(cx, &mut self.st)"
                    .split_whitespace()
                    .collect::<String>()
            ),
            "genui demo should not rely on implicit AppUi -> ElementContext coercion",
        );
    }

    #[test]
    fn markdown_demo_keeps_layout_query_authoring_on_app_ui_lane() {
        let markdown_demo = MARKDOWN_DEMO.split_whitespace().collect::<String>();
        for marker in [
            "cx.layout_query_bounds(anchor_id, Invalidation::Layout)",
            "cx.layout_query_bounds(viewport_region, Invalidation::Layout)",
            "cx.layout_query_region_with_id(props, move |_cx, id| {",
            "let scroll = cx.layout_query_region_with_id(",
            "pending_anchor.set_in(cx.app_mut().models_mut(), None);",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                markdown_demo.contains(&marker),
                "markdown demo should keep app-lane layout-query marker: {marker}",
            );
        }

        for legacy in [
            "pending_anchor.set_in(cx.app.models_mut(), None);",
            "cx.elements().layout_query_bounds(",
            "cx.elements().layout_query_region_with_id(",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !markdown_demo.contains(&legacy),
                "markdown demo should not regress to legacy layout-query authoring: {legacy}",
            );
        }
    }

    #[test]
    fn markdown_demo_prefers_capability_first_landing_for_root_and_layout_query_shells() {
        let markdown_demo = MARKDOWN_DEMO.split_whitespace().collect::<String>();

        for marker in [
            "use fret_ui_kit::IntoUiElementInExt as _;",
            "}).gap(Space::N3).wrap().items_center().into_element_in(cx);",
            "}).w_full().padding_px(padding_md).into_element_in(cx)])",
            ".refine_layout(LayoutRefinement::default().w_full().flex_1()).into_element_in(cx);",
            "}).w_full().h_full().gap(Space::N3).padding_px(padding_md).into_element_in(cx);",
            "ui::container(|_cx| [content]).bg(ColorRef::Color(theme.color_token(\"background\"))).w_full().h_full().into_element_in(cx).into()",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                markdown_demo.contains(&marker),
                "markdown demo should keep the capability-first landing marker: {marker}",
            );
        }

        for legacy in [
            "}).gap(Space::N3).wrap().items_center().into_element(cx);",
            "}).w_full().padding_px(padding_md).into_element(cx)])",
            ".refine_layout(LayoutRefinement::default().w_full().flex_1()).into_element(cx);",
            "}).w_full().h_full().gap(Space::N3).padding_px(padding_md).into_element(cx);",
            "}).bg(ColorRef::Color(theme.color_token(\"background\"))).w_full().h_full().into_element(cx).into()",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !markdown_demo.contains(&legacy),
                "markdown demo should stay off the legacy landing marker: {legacy}",
            );
        }
    }

    #[test]
    fn editor_notes_demo_keeps_reusable_panels_on_generic_element_context_access() {
        let editor_notes_demo = EDITOR_NOTES_DEMO.split_whitespace().collect::<String>();

        for marker in [
            "fn selection_button<'a, Cx>(",
            "pub(crate) fn render_selection_panel<'a, Cx>(",
            "pub(crate) fn render_center_panel<'a, Cx>(",
            "pub(crate) fn render_inspector_panel<'a, Cx>(",
            "Cx: fret::app::ElementContextAccess<'a, App>,",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                editor_notes_demo.contains(&marker),
                "editor notes demo should keep the generic reusable-panel marker: {marker}",
            );
        }

        for legacy in [
            "fn selection_button(cx: &mut AppUi<'_, '_>,",
            "fn render_selection_panel(cx: &mut AppUi<'_, '_>,",
            "fn render_center_panel(cx: &mut AppUi<'_, '_>,",
            "fn render_inspector_panel(cx: &mut AppUi<'_, '_>,",
            "fn render_selection_panel(cx: &mut AppComponentCx<'_>,",
            "fn render_center_panel(cx: &mut AppComponentCx<'_>,",
            "fn render_inspector_panel(cx: &mut AppComponentCx<'_>,",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !editor_notes_demo.contains(&legacy),
                "editor notes demo should stay off the legacy panel-owner marker: {legacy}",
            );
        }
    }

    #[test]
    fn editor_notes_demo_prefers_capability_first_landing_for_workspace_shell_root() {
        assert!(
            EDITOR_NOTES_DEMO
                .contains("use fret_ui_kit::{ColorRef, IntoUiElementInExt as _, Space};")
        );

        let render = source_slice(
            EDITOR_NOTES_DEMO,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "impl EditorNotesDemoView {",
        );
        let render = render.split_whitespace().collect::<String>();

        for marker in [
            ".h_full().into_element_in(cx).test_id(TEST_ID_LEFT_RAIL);",
            ".h_full().into_element_in(cx).test_id(TEST_ID_RIGHT_RAIL);",
            ".background(Some(theme.color_token(\"background\"))).into_element_in(cx);",
            "ui::container(|_cx| [frame]).p(Space::N4).size_full().into_element_in(cx).test_id(TEST_ID_ROOT).into()",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                render.contains(&marker),
                "editor notes demo should keep the capability-first root marker: {marker}",
            );
        }

        for legacy in [
            ".h_full().into_element(cx).test_id(TEST_ID_LEFT_RAIL);",
            ".h_full().into_element(cx).test_id(TEST_ID_RIGHT_RAIL);",
            ".background(Some(theme.color_token(\"background\"))).into_element(cx);",
            "ui::container(|_cx| [frame]).p(Space::N4).size_full().into_element(cx).test_id(TEST_ID_ROOT).into()",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !render.contains(&legacy),
                "editor notes demo should stay off the legacy root marker: {legacy}",
            );
        }
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
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "imui_in(cx, |ui| {",
                "ui.text(format!(\"Count: {count}\"));",
                "ui.checkbox_model(\"Enabled\", enabled_state.model())",
            ],
            &[
                "fret_imui::imui_in(cx, |ui| {",
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "fret_ui_kit::ui::text(format!(\"Count: {count}\"))",
            ],
        );

        assert_current_imui_teaching_surface(
            "imui_floating_windows_demo",
            IMUI_FLOATING_WINDOWS_DEMO,
            &[
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "imui_in(cx, |ui| {",
                "kit::WindowOptions::default()",
                "kit::FloatingWindowResizeOptions::default()",
                "ui.window_with_options(",
                "ui.combo_model_with_options(",
            ],
            &[
                "fret_imui::imui_in(cx, |ui| {",
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "fret_ui_kit::imui::WindowOptions::default()",
                "fret_ui_kit::imui::FloatingWindowResizeOptions::default()",
                "fret_ui_kit::imui::MenuItemOptions",
                "fret_ui_kit::imui::ComboModelOptions",
            ],
        );

        assert_current_imui_teaching_surface(
            "imui_response_signals_demo",
            IMUI_RESPONSE_SIGNALS_DEMO,
            &[
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "imui_in(cx, |ui| {",
                "kit::SliderOptions {",
                "kit::MenuBarOptions {",
                "kit::TabBarOptions {",
                "click.secondary_clicked()",
                "drag.drag_started()",
                "trigger.context_menu_requested()",
                "menu_lifecycle.activated()",
                "combo_resp.trigger.activated()",
                "combo_model_resp.deactivated_after_edit()",
            ],
            &[
                "fret_imui::imui_in(cx, |ui| {",
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "fret_ui_kit::imui::SliderOptions {",
                "fret_ui_kit::imui::InputTextOptions {",
                "fret_ui_kit::imui::MenuItemOptions {",
                "fret_ui_kit::imui::ComboOptions {",
                "fret_ui_kit::imui::SelectableOptions {",
                "fret_ui_kit::imui::ComboModelOptions {",
                "fret_ui_kit::imui::MenuBarOptions {",
                "fret_ui_kit::imui::BeginMenuOptions {",
                "fret_ui_kit::imui::BeginSubmenuOptions {",
                "fret_ui_kit::imui::TabBarOptions {",
                "fret_ui_kit::imui::TabItemOptions {",
            ],
        );

        assert_current_imui_teaching_surface(
            "imui_interaction_showcase_demo",
            IMUI_INTERACTION_SHOWCASE_DEMO,
            &[
                "Showcase surface for immediate-mode interaction affordances.",
                "Current proof/contract surface stays in `imui_response_signals_demo`.",
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "use fret_ui_shadcn::facade as shadcn;",
                "imui(cx, move |ui| {",
                "const TEST_ID_INSPECTOR",
                "TEST_ID_INSPECTOR_SUMMARY",
                "imui-interaction-showcase.inspector.flag.",
                "ShowcaseInspectorState::default",
                "render_response_inspector_card(",
                "record_showcase_response(",
                "pulse.press_holding()",
                "drag.drag_stopped()",
                "quick_actions.context_menu_requested()",
                "kit::ButtonOptions {",
                "kit::MenuBarOptions {",
                "kit::ChildRegionOptions {",
                "ui.begin_menu_with_options(",
                "ui.tab_bar_with_options(",
                "ui.begin_popup_context_menu(",
            ],
            &[
                "fret_imui::imui(cx, move |ui| {",
                "UiWriterImUiFacadeExt as _",
                "UiWriterUiKitExt as _",
                "fret_ui_kit::imui::ChildRegionOptions",
                "fret_ui_kit::imui::ScrollOptions",
                "fret_ui_kit::imui::ButtonOptions",
                "fret_ui_kit::imui::SliderOptions",
                "fret_ui_kit::imui::ComboModelOptions",
                "fret_ui_kit::imui::InputTextOptions",
                "fret_ui_kit::imui::MenuBarOptions",
                "fret_ui_kit::imui::BeginMenuOptions",
                "fret_ui_kit::imui::BeginSubmenuOptions",
                "fret_ui_kit::imui::MenuItemOptions",
                "fret_ui_kit::imui::TabBarOptions",
                "fret_ui_kit::imui::TabItemOptions",
            ],
        );

        assert_current_imui_teaching_surface(
            "imui_shadcn_adapter_demo",
            IMUI_SHADCN_ADAPTER_DEMO,
            &[
                "Product-validation IMUI surface for the shared control-chrome lane.",
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "imui_in(cx, |ui| {",
                "ui.add_ui(root);",
                "imui(cx, move |ui| {",
                "const TEST_ID_ROOT: &str = \"imui-shadcn-demo.root\";",
                "const TEST_ID_INCREMENT: &str = \"imui-shadcn-demo.controls.increment\";",
                "const TEST_ID_ENABLED: &str = \"imui-shadcn-demo.controls.enabled\";",
                "const TEST_ID_VALUE: &str = \"imui-shadcn-demo.controls.value\";",
                "const TEST_ID_MODE: &str = \"imui-shadcn-demo.controls.mode\";",
                "const TEST_ID_DRAFT: &str = \"imui-shadcn-demo.controls.draft\";",
                "summary_badge(",
                "kit::ButtonOptions {",
                "kit::SwitchOptions {",
                "kit::SliderOptions {",
                "kit::ComboModelOptions {",
                "kit::InputTextOptions {",
                "ui.combo_model_with_options(",
                "kit::TableColumn::fill(\"Signal\")",
                "kit::TableOptions {",
                "ui.table_with_options(",
                "kit::VirtualListOptions {",
                "kit::VirtualListMeasureMode::Fixed",
                "ui.virtual_list_with_options(",
            ],
            &[
                "fret_imui::imui_in(cx, |ui| {",
                "fret_imui::imui(cx, move |ui| {",
                "UiWriterImUiFacadeExt as _",
                "UiWriterUiKitExt as _",
                "fret_ui_kit::imui::ButtonOptions",
                "fret_ui_kit::imui::SwitchOptions",
                "fret_ui_kit::imui::SliderOptions",
                "fret_ui_kit::imui::ComboModelOptions",
                "fret_ui_kit::imui::InputTextOptions",
                "fret_ui_kit::imui::TableColumn",
                "fret_ui_kit::imui::TableOptions",
                "fret_ui_kit::imui::VirtualListOptions",
            ],
        );

        assert_current_imui_teaching_surface(
            "imui_node_graph_demo",
            IMUI_NODE_GRAPH_DEMO,
            &[
                "fret_imui::imui_in(cx, |ui| {",
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
                "use fret::imui::prelude::*;",
                "use fret_ui_editor::imui as editor_imui;",
                "use fret_ui_kit::imui::ImUiMultiSelectState;",
                "imui(cx, |ui| {",
                "imui(cx, move |ui| {",
                "imui_build(cx, out, |ui| {",
                "imui_build(cx, &mut out, move |ui| {",
                "imui_build(cx, out, f);",
                "editor_imui::property_grid(",
                "editor_imui::numeric_input(",
                "editor_imui::gradient_editor(",
            ],
            &[
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "fret_imui::imui(cx, |ui| {",
                "fret_imui::imui(cx, move |ui| {",
                "fret_imui::imui_build(cx, out, |ui| {",
                "fret_imui::imui_build(cx, &mut out, move |ui| {",
                "fret_imui::imui_build(cx, out, f);",
            ],
        );
    }

    #[test]
    fn imui_editor_proof_keeps_app_owned_sortable_and_dock_helpers_explicit() {
        let normalized = IMUI_EDITOR_PROOF_DEMO
            .split_whitespace()
            .collect::<String>();
        for marker in [
            "Sortable math stays app-owned. `imui` only provides typed payloads + drop positions.",
            "fn proof_outliner_items_snapshot(",
            "app.models().read(model, |items| items.clone()).unwrap_or_default()",
            "fn proof_outliner_order_line_for_model(",
            "proof_outliner_order_line(items)",
            "let outliner_items = proof_outliner_items_snapshot(ui.cx_mut().app, &outliner_items_model);",
            "let outliner_order = proof_outliner_order_line_for_model(ui.cx_mut().app, &outliner_items_model);",
            "fn embedded_target_for_window(app: &KernelApp, window: AppWindowId) -> fret_core::RenderTargetId {",
            "let target = embedded_target_for_window(app, window);",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for legacy in [
            "ui.cx_mut().app.models().read(&outliner_items_model, |items| items.clone())",
            "ui.cx_mut().app.models().read(&outliner_items_model, |items| { proof_outliner_order_line(items) })",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&legacy),
                "legacy marker still present: {legacy}"
            );
        }

        let dock_bootstrap = source_slice(
            IMUI_EDITOR_PROOF_DEMO,
            "fn ensure_dock_graph_inner(",
            "struct WindowBootstrapService {",
        )
        .split_whitespace()
        .collect::<String>();
        assert!(
            dock_bootstrap.contains(
                &"let target = embedded_target_for_window(app, window);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !dock_bootstrap.contains(
                &"embedded::models(app, window).and_then(|m| app.models().read(&m.target, |v| *v).ok()).unwrap_or_default()"
                    .split_whitespace()
                    .collect::<String>()
            )
        );

        for marker in [
            "outliner reorder math and dock bootstrap still belong to explicit app-owned helpers",
            "`proof_outliner_items_snapshot(...)`",
            "`proof_outliner_order_line_for_model(...)`",
            "`embedded_target_for_window(...)`",
            "do not justify new framework surface",
        ] {
            assert!(
                IMUI_EDITOR_PROOF_APP_OWNER_AUDIT.contains(marker),
                "imui editor proof app-owner audit should remain explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_examples_docs_name_the_golden_pair_and_reference_roster() {
        for marker in [
            "Immediate-mode sidecar (when you intentionally want the IMUI lane):",
            "First-party authoring policy: use the root `fret::imui` lane",
            "`use fret::imui::prelude::*;`",
            "`use fret::imui::{kit::..., prelude::*};`",
            "deliberate exception is `imui_node_graph_demo`",
            "compatibility-only retained-bridge",
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
            "On the explicit `fret::imui` lane, `imui(...)` is now the safe default",
            "`use fret::imui::prelude::*;`.",
            "`imui_raw(...)` is the advanced seam",
            "`imui_action_basics` demonstrates the explicit layout-host + raw shape on the root `fret::imui`",
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
    fn imui_shadcn_adapter_demo_keeps_control_discoverability_proof_surface() {
        let demo = IMUI_SHADCN_ADAPTER_DEMO
            .split_whitespace()
            .collect::<String>();
        for marker in [
            "constTEST_ID_ROOT:&str=\"imui-shadcn-demo.root\";",
            "constTEST_ID_CONTROL_CARD:&str=\"imui-shadcn-demo.controls.card\";",
            "constTEST_ID_SUMMARY_CARD:&str=\"imui-shadcn-demo.summary.card\";",
            "constTEST_ID_INSPECTOR_CARD:&str=\"imui-shadcn-demo.inspector.card\";",
            "constTEST_ID_SUMMARY_COUNT:&str=\"imui-shadcn-demo.summary.count\";",
            "constTEST_ID_SUMMARY_ENABLED:&str=\"imui-shadcn-demo.summary.enabled\";",
            "constTEST_ID_SUMMARY_MODE:&str=\"imui-shadcn-demo.summary.mode\";",
            "constTEST_ID_SUMMARY_DRAFT:&str=\"imui-shadcn-demo.summary.draft\";",
            "test_id:Some(Arc::from(TEST_ID_INCREMENT))",
            "test_id:Some(Arc::from(TEST_ID_ENABLED))",
            "test_id:Some(Arc::from(TEST_ID_VALUE))",
            "test_id:Some(Arc::from(TEST_ID_MODE))",
            "test_id:Some(Arc::from(TEST_ID_DRAFT))",
            "summary_badge(",
        ] {
            assert!(
                demo.contains(marker),
                "imui shadcn adapter demo should keep the compact control-discoverability proof markers: {marker}"
            );
        }

        let script = IMUI_SHADCN_ADAPTER_DISCOVERABILITY_SCRIPT
            .split_whitespace()
            .collect::<String>();
        for marker in [
            "\"name\":\"imui-shadcn-adapter-control-discoverability\"",
            "\"id\":\"imui-shadcn-demo.root\"",
            "\"id\":\"imui-shadcn-demo.controls.increment.chrome\"",
            "\"id\":\"imui-shadcn-demo.inspector.card\"",
            "\"id\":\"imui-shadcn-demo.controls.enabled.chrome\"",
            "\"id\":\"imui-shadcn-demo.controls.value.chrome\"",
            "\"id\":\"imui-shadcn-demo.controls.mode.chrome\"",
            "\"id\":\"imui-shadcn-demo.controls.draft\"",
            "\"kind\":\"bounds_min_size\"",
            "\"kind\":\"bounds_non_overlapping\"",
            "\"id\":\"imui-shadcn-demo.controls.mode.option.1\"",
            "\"text\":\"mode:Beta\"",
            "\"text\":\"draft:stagingreview\"",
            "\"type\":\"capture_layout_sidecar\"",
            "\"type\":\"capture_screenshot\"",
        ] {
            assert!(
                script.contains(marker),
                "adapter discoverability script should keep the bounded screenshot/layout proof markers: {marker}"
            );
        }
    }

    #[test]
    fn imui_interaction_showcase_demo_avoids_fixed_compact_lab_width_workaround() {
        let demo = IMUI_INTERACTION_SHOWCASE_DEMO
            .split_whitespace()
            .collect::<String>();

        for marker in [
            "constSHOWCASE_COMPACT_RAIL_MIN_WIDTH:Px=Px(272.0);",
            "constSHOWCASE_COMPACT_RAIL_MAX_WIDTH:Px=Px(352.0);",
            "constSHOWCASE_REGULAR_SIDE_COLUMN_WIDTH:Px=Px(336.0);",
            ".basis(LengthRefinement::Fraction(0.32))",
            ".min_w(SHOWCASE_COMPACT_RAIL_MIN_WIDTH)",
            ".max_w(SHOWCASE_COMPACT_RAIL_MAX_WIDTH)",
            ".w_px(SHOWCASE_REGULAR_SIDE_COLUMN_WIDTH)",
        ] {
            assert!(
                demo.contains(marker),
                "interaction showcase should keep the compact rail layout without a fixed-width workaround: {marker}"
            );
        }

        for marker in [
            "constSHOWCASE_SIDE_COLUMN_WIDTH:Px=Px(320.0);",
            "side_column_width:Px,",
            ".w_px(responsive.side_column_width)",
            "assert_eq!(layout.side_column_width,SHOWCASE_SIDE_COLUMN_WIDTH);",
        ] {
            assert!(
                !demo.contains(marker),
                "interaction showcase should not keep the old fixed compact rail workaround: {marker}"
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
    fn immediate_mode_workstream_freezes_the_p0_key_owner_surface_follow_on() {
        for marker in [
            "focused `activate_shortcut`",
            "`SetNextItemShortcut()` / `SetItemKeyOwner()`",
            "`crates/fret-ui` must remain unchanged unless stronger ADR-backed evidence appears.",
            "`apps/fret-examples/src/imui_response_signals_demo.rs`",
            "`ecosystem/fret-imui/src/tests/interaction.rs`",
            "Global keymap / command routing semantics remain fixed input, not negotiable scope here.",
            "Do not reopen `ResponseExt` lifecycle vocabulary, collection/pane proof breadth, or richer",
        ] {
            assert!(
                IMUI_KEY_OWNER_SURFACE_DESIGN.contains(marker),
                "the immediate-mode workstream should keep the P0 key-owner surface design explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-key-owner-surface-v1\"",
            "\"status\": \"closed\"",
            "\"follow_on_of\": \"imui-editor-grade-product-closure-v1\"",
            "\"path\": \"docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md\"",
            "\"path\": \"docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md\"",
            "immediate_mode_workstream_freezes_the_p0_key_owner_surface_follow_on",
            "imui_response_signals_demo",
        ] {
            assert!(
                IMUI_KEY_OWNER_SURFACE_WORKSTREAM.contains(marker),
                "the key-owner surface lane state should keep the follow-on marker: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-key-owner-surface-v1/` now records the closed key-owner /",
            "item-local shortcut ownership follow-on",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the key-owner follow-on explicit: {marker}"
            );
        }

        for marker in [
            "Keep `apps/fret-examples/src/imui_response_signals_demo.rs` as the current proof/contract surface",
            "do not promote a new dedicated key-owner proof demo yet.",
            "menu_item_command_uses_command_metadata_shortcut_and_gating",
            "combo_model_activate_shortcut_is_scoped_to_focused_trigger",
            "runtime keymap / IME arbitration",
        ] {
            assert!(
                IMUI_KEY_OWNER_SURFACE_M1_NOTE.contains(marker),
                "the key-owner lane should keep the M1 proof roster explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_key_owner_surface_m2_no_new_surface_verdict_is_explicit() {
        for marker in [
            "M2 closes on a no-new-surface verdict.",
            "There is still no stronger first-party consumer pressure for a broader key-owner surface.",
            "Do not add a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade.",
            "Do not add a broader item-local shortcut registration seam.",
            "reopen this question only if stronger first-party proof exceeds the current demo/test",
        ] {
            assert!(
                IMUI_KEY_OWNER_SURFACE_M2_NOTE.contains(marker),
                "the key-owner lane should keep the M2 no-new-surface verdict explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "The current helper-local shortcut seams already close the first-party key-owner demand for this cycle",
            "There is still no stronger first-party consumer pressure for a broader key-owner surface",
            "do not add a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade here by",
            "start a different narrow lane with stronger first-party proof if future pressure still",
        ] {
            assert!(
                IMUI_KEY_OWNER_SURFACE_CLOSEOUT.contains(marker),
                "the key-owner lane should keep its closeout verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"status\": \"closed\"",
            "\"path\": \"docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md\"",
            "\"path\": \"docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md\"",
            "\"default_action\": \"close\"",
            "immediate_mode_key_owner_surface_m2_no_new_surface_verdict_is_explicit",
        ] {
            assert!(
                IMUI_KEY_OWNER_SURFACE_WORKSTREAM.contains(marker),
                "the key-owner lane state should keep the M2 closeout markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-key-owner-surface-v1/` now records the closed key-owner /",
            "item-local shortcut ownership follow-on",
            "the current helper-local",
            "first-party proof warrants a different narrow lane, and",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the key-owner closeout verdict explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p0_p1_collection_pane_proof_follow_on() {
        for marker in [
            "`apps/fret-examples/src/imui_editor_proof_demo.rs`",
            "`apps/fret-examples/src/workspace_shell_demo.rs`",
            "`apps/fret-examples/src/editor_notes_demo.rs`",
            "Keep `apps/fret-examples/src/imui_editor_proof_demo.rs` as the current collection-first proof",
            "Keep `apps/fret-examples/src/workspace_shell_demo.rs` as the current pane-first proof",
            "Keep `apps/fret-examples/src/editor_notes_demo.rs` as the supporting minimal pane rail proof.",
            "Do not introduce a dedicated asset-grid/file-browser proof demo yet.",
            "Do not introduce a narrower child-region-only proof demo yet.",
            "key ownership",
            "promoted shell helpers",
            "runner/backend multi-window parity",
            "broader menu/tab policy",
        ] {
            assert!(
                IMUI_COLLECTION_PANE_PROOF_M1_NOTE.contains(marker),
                "the collection/pane proof follow-on should keep the frozen M1 proof roster explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-pane-proof-v1\"",
            "\"status\": \"closed\"",
            "\"follow_on_of\": \"imui-editor-grade-product-closure-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md\"",
            "immediate_mode_workstream_freezes_the_p0_p1_collection_pane_proof_follow_on",
            "workspace_shell_demo",
            "editor_notes_demo",
        ] {
            assert!(
                IMUI_COLLECTION_PANE_PROOF_WORKSTREAM.contains(marker),
                "the collection/pane proof lane state should keep the M1 source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-pane-proof-v1/` now records the closed collection-first /",
            "pane-first proof pair with a no-helper-widening verdict",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection/pane proof follow-on explicit: {marker}"
            );
        }

        for marker in [
            "M4 closes on a no-helper-widening verdict.",
            "do not add helper widening, a narrower pane-only demo, or a narrower pane-only diagnostics path",
            "Treat `imui-collection-pane-proof-v1` as:",
        ] {
            assert!(
                IMUI_COLLECTION_PANE_PROOF_CLOSEOUT.contains(marker),
                "the collection/pane proof lane should keep its closeout verdict explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p0_imui_facade_internal_modularization_follow_on() {
        for marker in [
            "keep the public `fret-ui-kit::imui` surface stable while restructuring internals",
            "`ecosystem/fret-ui-kit/src/imui.rs` still mixes the module hub",
            "`ecosystem/fret-ui-kit/src/imui/options.rs` and `ecosystem/fret-ui-kit/src/imui/response.rs`",
            "The first implementation slice should stay structural:",
            "Do not widen `crates/fret-ui`",
        ] {
            assert!(
                IMUI_FACADE_INTERNAL_MODULARIZATION_DESIGN.contains(marker),
                "the internal modularization lane should keep the design boundary explicit: {marker}"
            );
        }

        for marker in [
            "`ecosystem/fret-ui-kit/src/imui.rs`: 2209 lines",
            "`ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`: 1027 lines",
            "M0 chooses this first implementation slice:",
            "modularize `options.rs`",
            "modularize `response.rs`",
        ] {
            assert!(
                IMUI_FACADE_INTERNAL_MODULARIZATION_M0_NOTE.contains(marker),
                "the internal modularization lane should keep the baseline audit explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-facade-internal-modularization-v1\"",
            "\"status\": \"closed\"",
            "\"follow_on_of\": \"imui-editor-grade-product-closure-v1\"",
            "\"path\": \"docs/workstreams/imui-facade-internal-modularization-v1/M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md\"",
            "\"path\": \"docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md\"",
            "\"path\": \"docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md\"",
            "\"path\": \"docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md\"",
            "\"path\": \"docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md\"",
            "immediate_mode_workstream_freezes_the_p0_imui_facade_internal_modularization_follow_on",
            "imui_response_contract_smoke",
            "cargo nextest run -p fret-imui --no-fail-fast",
        ] {
            assert!(
                IMUI_FACADE_INTERNAL_MODULARIZATION_WORKSTREAM.contains(marker),
                "the internal modularization lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`options.rs` now re-exports smaller private owner files",
            "`response.rs` now re-exports smaller private owner files",
            "no public type names changed",
            "`ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`",
            "`ecosystem/fret-ui-kit/src/imui.rs`",
        ] {
            assert!(
                IMUI_FACADE_INTERNAL_MODULARIZATION_M1_NOTE.contains(marker),
                "the internal modularization lane should keep the first landed slice explicit: {marker}"
            );
        }

        for marker in [
            "`interaction_runtime.rs` now re-exports the same helper family over five private owner files",
            "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/models.rs`",
            "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/disabled.rs`",
            "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle.rs`",
            "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs`",
            "`ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag.rs`",
            "hover/lifecycle/drag/disabled bookkeeping are reviewable as separate owners",
            "`ecosystem/fret-ui-kit/src/imui.rs`",
        ] {
            assert!(
                IMUI_FACADE_INTERNAL_MODULARIZATION_M2_NOTE.contains(marker),
                "the internal modularization lane should keep the M2 interaction-runtime slice explicit: {marker}"
            );
        }

        for marker in [
            "`ecosystem/fret-ui-kit/src/imui.rs` now re-imports smaller owner files for support helpers",
            "`ecosystem/fret-ui-kit/src/imui/facade_support.rs`",
            "`ecosystem/fret-ui-kit/src/imui/floating_options.rs`",
            "`UiWriterUiKitExt`",
            "`ImUiFacade` / `UiWriterImUiFacadeExt` writer glue",
        ] {
            assert!(
                IMUI_FACADE_INTERNAL_MODULARIZATION_M3_NOTE.contains(marker),
                "the internal modularization lane should keep the M3 root-hub slice explicit: {marker}"
            );
        }

        for marker in [
            "`ecosystem/fret-ui-kit/src/imui/facade_writer.rs`",
            "`ImUiFacade`",
            "`UiWriterImUiFacadeExt`",
            "`ecosystem/fret-ui-kit/src/imui.rs`: 125 lines",
            "one dedicated owner file",
        ] {
            assert!(
                IMUI_FACADE_INTERNAL_MODULARIZATION_M4_NOTE.contains(marker),
                "the internal modularization lane should keep the M4 writer-glue slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed",
            "This lane is closed.",
            "`options.rs` -> smaller private owner files",
            "`interaction_runtime.rs` -> owner files under `interaction_runtime/`",
            "`facade_writer.rs`",
            "Do not reopen this lane by default.",
        ] {
            assert!(
                IMUI_FACADE_INTERNAL_MODULARIZATION_CLOSEOUT.contains(marker),
                "the internal modularization lane should keep the closeout verdict explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-facade-internal-modularization-v1/DESIGN.md`",
            "`docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md`",
            "`docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`",
            "`docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md`",
            "`docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md`",
            "`docs/workstreams/imui-facade-internal-modularization-v1/WORKSTREAM.json`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the active internal modularization lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the active internal modularization lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the active internal modularization lane: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_collection_pane_proof_m2_collection_first_asset_browser_slice_is_explicit() {
        for marker in [
            "Collection-first asset browser proof",
            "Background drag now draws a marquee and updates grid selection app-locally while shared helper widening stays deferred until another first-party proof surface exists.",
            "imui-editor-proof.authoring.imui.collection.order-toggle",
            "imui-editor-proof.authoring.imui.collection.browser",
            "imui-editor-proof.authoring.imui.collection.grid",
            "imui_editor_proof_demo.model.authoring_parity.collection_selection",
            "imui_editor_proof_demo.model.authoring_parity.collection_box_select",
            "imui_editor_proof_demo.model.authoring_parity.collection_reverse_order",
            "imui_editor_proof_demo.model.authoring_parity.collection_drop_status",
            "ui.id(asset.id.clone(), |ui| {",
        ] {
            assert!(
                IMUI_EDITOR_PROOF_DEMO.contains(marker),
                "imui_editor_proof_demo should keep the M2 collection-first asset browser proof explicit: {marker}"
            );
        }

        for marker in [
            "Keep `apps/fret-examples/src/imui_editor_proof_demo.rs` as the collection-first M2 proof surface.",
            "Close M2 with an in-demo asset-browser/file-browser proof instead of a new dedicated demo.",
            "Marquee / box-select stays deferred for M2.",
            "`ecosystem/fret-imui/src/tests/interaction.rs` now proves selected collection drag payloads survive visible order flips.",
        ] {
            assert!(
                IMUI_COLLECTION_PANE_PROOF_M2_NOTE.contains(marker),
                "the M2 collection proof note should keep the collection-first closure explicit: {marker}"
            );
        }

        for marker in [
            "\"path\": \"docs/workstreams/imui-collection-pane-proof-v1/M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md\"",
            "\"name\": \"collection-focused-interaction\"",
            "collection_drag_payload_preserves_selected_keys_across_order_flip",
        ] {
            assert!(
                IMUI_COLLECTION_PANE_PROOF_WORKSTREAM.contains(marker),
                "the collection/pane proof lane state should keep the M2 collection-first gates explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_collection_box_select_follow_on() {
        for marker in [
            "land one app-owned background marquee / box-select slice on the existing proof surface",
            "The frozen two-surface proof budget blocks a new public `fret-ui-kit::imui` helper here.",
            "The first correct target is:",
            "background-only marquee / box-select slice inside",
            "Do not begin by designing a shared helper surface.",
        ] {
            assert!(
                IMUI_COLLECTION_BOX_SELECT_DESIGN.contains(marker),
                "the collection box-select design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection/pane proof lane explicitly deferred marquee / box-select for M2.",
            "The frozen two-surface proof budget blocks a new public `fret-ui-kit::imui` helper here.",
            "The current proof surface already has the right ingredients for a narrow app-owned box-select",
            "Dear ImGui treats box-select as part of collection depth",
        ] {
            assert!(
                IMUI_COLLECTION_BOX_SELECT_M0_NOTE.contains(marker),
                "the collection box-select baseline audit should keep the new-lane justification explicit: {marker}"
            );
        }

        for marker in [
            "Background drag now draws a marquee overlay and updates collection selection app-locally.",
            "Selection stays normalized to visible collection order",
            "Plain background click clears the selection;",
            "baseline set.",
            "No new public `fret-ui-kit::imui` box-select helper is admitted in this lane.",
        ] {
            assert!(
                IMUI_COLLECTION_BOX_SELECT_M1_NOTE.contains(marker),
                "the collection box-select M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-collection-box-select-v1` as:",
            "a closeout record for the landed app-owned background marquee / box-select slice",
            "Start a different narrower follow-on only if stronger first-party proof shows either:",
        ] {
            assert!(
                IMUI_COLLECTION_BOX_SELECT_CLOSEOUT.contains(marker),
                "the collection box-select closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-box-select-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-pane-proof-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md\"",
            "\"path\": \"docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md\"",
            "\"path\": \"docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md\"",
            "immediate_mode_workstream_freezes_the_p1_collection_box_select_follow_on",
            "proof_collection_drag_rect_normalizes_drag_direction",
            "proof_collection_box_select_replace_uses_visible_collection_order",
            "proof_collection_box_select_append_preserves_baseline_and_adds_hits",
            "imui_editor_collection_box_select_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_BOX_SELECT_WORKSTREAM.contains(marker),
                "the collection box-select lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-box-select-v1/` now records the closed",
            "background-only box-select slice in `imui_editor_proof_demo`",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection box-select follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-box-select-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
            "`docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`",
            "`docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection box-select lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection box-select lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection box-select lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json`"),
            "the workstream index should list the collection box-select lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC
                .contains("`docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json`"),
            "the todo tracker should list the collection box-select lane state file explicitly"
        );
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_collection_keyboard_owner_follow_on() {
        for marker in [
            "The generic key-owner lane stays closed; this lane is collection proof depth, not generic helper growth.",
            "The first landable target is therefore narrow:",
            "make the collection scope itself a focusable keyboard owner in the proof demo,",
            "`Arrow` / `Home` / `End` to move the active tile in visible order,",
            "Do not start by designing a shared helper or a new generic shortcut facade.",
        ] {
            assert!(
                IMUI_COLLECTION_KEYBOARD_OWNER_DESIGN.contains(marker),
                "the collection keyboard-owner design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection box-select lane explicitly deferred collection keyboard-owner depth.",
            "The generic key-owner lane already closed on a no-new-surface verdict and should stay closed.",
            "The current proof surface already has the right ingredients for a narrow app-owned keyboard",
            "The smallest credible slice is still narrower than \"full parity\"",
        ] {
            assert!(
                IMUI_COLLECTION_KEYBOARD_OWNER_M0_NOTE.contains(marker),
                "the collection keyboard-owner baseline audit should keep the new-lane justification explicit: {marker}"
            );
        }

        for marker in [
            "The collection scope now owns a focusable keyboard region locally in the proof demo.",
            "`Arrow` / `Home` / `End` now move the active tile",
            "`Shift+Arrow` / `Shift+Home` / `Shift+End` now extend the selected range",
            "`Escape` now clears the selected set while keeping the current keyboard location app-defined.",
            "No new generic `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale facade is admitted here.",
        ] {
            assert!(
                IMUI_COLLECTION_KEYBOARD_OWNER_M1_NOTE.contains(marker),
                "the collection keyboard-owner M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-collection-keyboard-owner-v1` as:",
            "generic key-owner no-new-surface verdict remains closed",
            "No reopening of the generic key-owner lane.",
        ] {
            assert!(
                IMUI_COLLECTION_KEYBOARD_OWNER_CLOSEOUT.contains(marker),
                "the collection keyboard-owner closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-keyboard-owner-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-box-select-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md\"",
            "\"path\": \"docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md\"",
            "\"path\": \"docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md\"",
            "immediate_mode_workstream_freezes_the_p1_collection_keyboard_owner_follow_on",
            "proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile",
            "proof_collection_keyboard_shift_navigation_extends_range_from_anchor",
            "proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile",
            "proof_collection_keyboard_ignores_primary_modifier_shortcuts",
            "imui_editor_collection_keyboard_owner_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_KEYBOARD_OWNER_WORKSTREAM.contains(marker),
                "the collection keyboard-owner lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-keyboard-owner-v1/` now records the closed",
            "app-owned collection keyboard-owner slice in `imui_editor_proof_demo`",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection keyboard-owner follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
            "`docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`",
            "`docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection keyboard-owner lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection keyboard-owner lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection keyboard-owner lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json`"),
            "the workstream index should list the collection keyboard-owner lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC
                .contains("`docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json`"),
            "the todo tracker should list the collection keyboard-owner lane state file explicitly"
        );
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_collection_delete_action_follow_on() {
        for marker in [
            "The closed collection keyboard-owner lane already deferred collection action semantics.",
            "The first landable target is therefore narrow:",
            "make `Delete` / `Backspace` remove the current selected set in visible collection order,",
            "add one explicit button-owned affordance for the same action,",
            "Do not start by designing a shared collection command facade or helper.",
        ] {
            assert!(
                IMUI_COLLECTION_DELETE_ACTION_DESIGN.contains(marker),
                "the collection delete-action design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection keyboard-owner lane explicitly deferred collection action semantics.",
            "The proof-budget rule and runtime contract posture remain unchanged for this lane.",
            "The current proof surface already has the right ingredients for a narrow app-owned delete slice:",
            "Dear ImGui keeps delete requests at the collection proof surface rather than using them as a reason to widen unrelated runtime or shared-helper contracts.",
        ] {
            assert!(
                IMUI_COLLECTION_DELETE_ACTION_M0_NOTE.contains(marker),
                "the collection delete-action baseline audit should keep the new-lane justification explicit: {marker}"
            );
        }

        for marker in [
            "The collection proof now supports one app-owned delete-selected action slice.",
            "`Delete` / `Backspace` now remove the selected set from the stored asset model.",
            "The explicit action button reuses the same delete helper instead of forking policy.",
            "Remaining assets, selection, and keyboard active tile now reflow app-locally after deletion.",
            "No new public `fret-ui-kit::imui` collection action helper is admitted in this lane.",
        ] {
            assert!(
                IMUI_COLLECTION_DELETE_ACTION_M1_NOTE.contains(marker),
                "the collection delete-action M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-collection-delete-action-v1` as:",
            "a closeout record for the landed app-owned collection delete-selected slice",
            "No reopening of the generic key-owner lane or the closed keyboard-owner folder.",
        ] {
            assert!(
                IMUI_COLLECTION_DELETE_ACTION_CLOSEOUT.contains(marker),
                "the collection delete-action closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-delete-action-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-keyboard-owner-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md\"",
            "\"path\": \"docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md\"",
            "\"path\": \"docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md\"",
            "immediate_mode_workstream_freezes_the_p1_collection_delete_action_follow_on",
            "proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item",
            "proof_collection_delete_selection_picks_previous_visible_item_at_end",
            "imui_editor_collection_delete_action_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_DELETE_ACTION_WORKSTREAM.contains(marker),
                "the collection delete-action lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-delete-action-v1/` now records the closed",
            "app-owned collection delete-selected slice in `imui_editor_proof_demo`",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection delete-action follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
            "`docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`",
            "`docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection delete-action lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection delete-action lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection delete-action lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json`"),
            "the workstream index should list the collection delete-action lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC
                .contains("`docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json`"),
            "the todo tracker should list the collection delete-action lane state file explicitly"
        );
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_collection_context_menu_follow_on() {
        for marker in [
            "The closed collection delete-action lane already deferred context-menu action breadth.",
            "The first landable target is therefore narrow:",
            "reuse the current app-owned delete helper inside one shared collection popup scope,",
            "support right-click on both assets and collection background,",
            "Do not start by designing a shared collection context-menu helper or broader command surface.",
        ] {
            assert!(
                IMUI_COLLECTION_CONTEXT_MENU_DESIGN.contains(marker),
                "the collection context-menu design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection delete-action lane explicitly deferred context-menu breadth.",
            "The current proof surface already has the right ingredients for a narrow app-owned collection context menu:",
            "The menu/popup helper floor already exists generically, so this lane is not a justification to widen shared helper ownership.",
            "Dear ImGui keeps the asset-browser context menu at the proof surface and routes delete through the same selection model instead of inventing a separate command contract.",
        ] {
            assert!(
                IMUI_COLLECTION_CONTEXT_MENU_M0_NOTE.contains(marker),
                "the collection context-menu baseline audit should keep the new-lane justification explicit: {marker}"
            );
        }

        for marker in [
            "The collection proof now supports one shared popup scope for app-owned quick actions.",
            "Right-click on an unselected asset now replaces selection with that asset before opening the popup.",
            "Right-click on collection background now opens the same popup without widening helper surface.",
            "The popup reuses the existing delete helper instead of forking collection action policy.",
            "No new public `fret-ui-kit::imui` collection context-menu helper is admitted in this lane.",
        ] {
            assert!(
                IMUI_COLLECTION_CONTEXT_MENU_M1_NOTE.contains(marker),
                "the collection context-menu M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-collection-context-menu-v1` as:",
            "a closeout record for the landed app-owned collection context-menu slice",
            "No reopening of the closed delete-action lane or the generic menu/key-owner lanes.",
        ] {
            assert!(
                IMUI_COLLECTION_CONTEXT_MENU_CLOSEOUT.contains(marker),
                "the collection context-menu closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-context-menu-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-delete-action-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md\"",
            "immediate_mode_workstream_freezes_the_p1_collection_context_menu_follow_on",
            "proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile",
            "proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile",
            "imui_editor_collection_context_menu_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_CONTEXT_MENU_WORKSTREAM.contains(marker),
                "the collection context-menu lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-context-menu-v1/` now records the closed",
            "app-owned collection context-menu slice in `imui_editor_proof_demo`",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection context-menu follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-context-menu-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
            "`docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`",
            "`docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection context-menu lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection context-menu lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection context-menu lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-collection-context-menu-v1/WORKSTREAM.json`"),
            "the workstream index should list the collection context-menu lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC
                .contains("`docs/workstreams/imui-collection-context-menu-v1/WORKSTREAM.json`"),
            "the todo tracker should list the collection context-menu lane state file explicitly"
        );
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_collection_zoom_follow_on() {
        for marker in [
            "The closed collection context-menu lane already deferred collection zoom/layout depth.",
            "The first landable target is therefore narrow:",
            "derive collection layout metrics from viewport width plus an app-owned zoom model,",
            "route primary+wheel through one collection-scope zoom policy,",
            "Do not start by designing a shared collection zoom helper or runtime-owned layout contract.",
        ] {
            assert!(
                IMUI_COLLECTION_ZOOM_DESIGN.contains(marker),
                "the collection zoom design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection context-menu lane explicitly deferred collection zoom/layout depth.",
            "The current proof surface already has the right ingredients for a narrow app-owned collection zoom slice:",
            "The scroll handle and wheel hooks already exist generically, so this lane is not a justification to widen shared helper ownership.",
            "Dear ImGui keeps asset-browser zoom and layout recomputation at the proof surface instead of turning them into a generic runtime contract.",
        ] {
            assert!(
                IMUI_COLLECTION_ZOOM_M0_NOTE.contains(marker),
                "the collection zoom baseline audit should keep the new-lane justification explicit: {marker}"
            );
        }

        for marker in [
            "The collection proof now derives layout metrics from viewport width plus app-owned zoom state.",
            "Primary+Wheel now adjusts tile extent without widening generic IMUI helper ownership.",
            "Keyboard grid navigation now reads the derived layout columns instead of a frozen constant.",
            "The zoom slice reuses the existing child-region scroll handle to keep hovered rows anchored while columns change.",
            "No new public `fret-ui-kit::imui` collection zoom helper is admitted in this lane.",
        ] {
            assert!(
                IMUI_COLLECTION_ZOOM_M1_NOTE.contains(marker),
                "the collection zoom M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-collection-zoom-v1` as:",
            "a closeout record for the landed app-owned collection zoom/layout slice",
            "No reopening of the closed context-menu lane or wider generic layout/helper questions.",
        ] {
            assert!(
                IMUI_COLLECTION_ZOOM_CLOSEOUT.contains(marker),
                "the collection zoom closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-zoom-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-context-menu-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-zoom-v1/M0_BASELINE_AUDIT_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-zoom-v1/M1_APP_OWNED_ZOOM_LAYOUT_SLICE_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-zoom-v1/CLOSEOUT_AUDIT_2026-04-23.md\"",
            "immediate_mode_workstream_freezes_the_p1_collection_zoom_follow_on",
            "proof_collection_layout_metrics_fall_back_before_viewport_binding_exists",
            "proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor",
            "proof_collection_zoom_request_ignores_non_primary_wheel",
            "imui_editor_collection_zoom_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_ZOOM_WORKSTREAM.contains(marker),
                "the collection zoom lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-zoom-v1/` now records the closed",
            "app-owned collection zoom/layout slice in `imui_editor_proof_demo`",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection zoom follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-zoom-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-zoom-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
            "`docs/workstreams/imui-collection-zoom-v1/M1_APP_OWNED_ZOOM_LAYOUT_SLICE_2026-04-23.md`",
            "`docs/workstreams/imui-collection-zoom-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection zoom lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection zoom lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection zoom lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-collection-zoom-v1/WORKSTREAM.json`"),
            "the workstream index should list the collection zoom lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC.contains("`docs/workstreams/imui-collection-zoom-v1/WORKSTREAM.json`"),
            "the todo tracker should list the collection zoom lane state file explicitly"
        );
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_collection_select_all_follow_on() {
        for marker in [
            "The closed collection zoom lane already deferred collection select-all breadth.",
            "The first landable target is therefore narrow:",
            "route Primary+A through one collection-scope select-all policy,",
            "select all visible assets in current visible order,",
            "Do not start by designing a shared collection select-all helper or broader command surface.",
        ] {
            assert!(
                IMUI_COLLECTION_SELECT_ALL_DESIGN.contains(marker),
                "the collection select-all design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection zoom lane explicitly deferred collection select-all breadth.",
            "The current proof surface already has the right ingredients for a narrow app-owned collection select-all slice:",
            "The collection-scope key-owner and visible-order helpers already exist locally, so this lane is not a justification to widen shared helper ownership.",
            "Dear ImGui keeps Ctrl+A selection breadth in the multi-select proof surface instead of turning it into a generic runtime contract.",
        ] {
            assert!(
                IMUI_COLLECTION_SELECT_ALL_M0_NOTE.contains(marker),
                "the collection select-all baseline audit should keep the new-lane justification explicit: {marker}"
            );
        }

        for marker in [
            "The collection proof now supports one app-owned select-all shortcut slice.",
            "Primary+A now selects all visible assets within the focused collection scope.",
            "Select-all keeps the current active tile when possible instead of widening generic key-owner ownership.",
            "The popup/menu surface stays unchanged in this lane.",
            "No new public `fret-ui-kit::imui` collection select-all helper is admitted in this lane.",
        ] {
            assert!(
                IMUI_COLLECTION_SELECT_ALL_M1_NOTE.contains(marker),
                "the collection select-all M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-collection-select-all-v1` as:",
            "a closeout record for the landed app-owned collection select-all slice",
            "No reopening of the closed zoom lane or wider generic key-owner/helper questions.",
        ] {
            assert!(
                IMUI_COLLECTION_SELECT_ALL_CLOSEOUT.contains(marker),
                "the collection select-all closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-select-all-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-zoom-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-select-all-v1/M0_BASELINE_AUDIT_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-select-all-v1/M1_APP_OWNED_SELECT_ALL_SLICE_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-select-all-v1/CLOSEOUT_AUDIT_2026-04-23.md\"",
            "immediate_mode_workstream_freezes_the_p1_collection_select_all_follow_on",
            "proof_collection_select_all_selection_uses_visible_order_and_preserves_active_tile",
            "proof_collection_select_all_selection_falls_back_to_first_visible_asset",
            "proof_collection_select_all_shortcut_matches_primary_a_only",
            "imui_editor_collection_select_all_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_SELECT_ALL_WORKSTREAM.contains(marker),
                "the collection select-all lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-select-all-v1/` now records the closed",
            "app-owned collection select-all slice in `imui_editor_proof_demo`",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection select-all follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-select-all-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-select-all-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
            "`docs/workstreams/imui-collection-select-all-v1/M1_APP_OWNED_SELECT_ALL_SLICE_2026-04-23.md`",
            "`docs/workstreams/imui-collection-select-all-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection select-all lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection select-all lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection select-all lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-collection-select-all-v1/WORKSTREAM.json`"),
            "the workstream index should list the collection select-all lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC
                .contains("`docs/workstreams/imui-collection-select-all-v1/WORKSTREAM.json`"),
            "the todo tracker should list the collection select-all lane state file explicitly"
        );
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_collection_rename_follow_on() {
        for marker in [
            "The closed collection select-all lane already deferred rename breadth.",
            "The first landable target is therefore narrow:",
            "route F2 through the existing collection-scope keyboard owner,",
            "open one app-owned rename modal from the current active asset or context-menu selection,",
            "Do not start by designing a shared collection rename helper or generic inline-edit surface.",
        ] {
            assert!(
                IMUI_COLLECTION_RENAME_DESIGN.contains(marker),
                "the collection rename design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection select-all lane explicitly deferred rename breadth.",
            "The current proof surface already has the right ingredients for a narrow app-owned collection rename slice:",
            "The current proof already has popup and text-input seams, so this lane is not a justification to widen shared helper ownership.",
            "Dear ImGui keeps rename breadth close to the current proof surface instead of turning it into a generic runtime contract.",
        ] {
            assert!(
                IMUI_COLLECTION_RENAME_M0_NOTE.contains(marker),
                "the collection rename baseline audit should keep the new-lane justification explicit: {marker}"
            );
        }

        for marker in [
            "The collection proof now supports one app-owned rename slice.",
            "F2 and the existing context-menu entry now open one app-owned rename modal for the active collection asset.",
            "Committing rename updates the visible label while preserving stable asset ids and collection order.",
            "The popup stays product-owned and uses the existing input/popup seams instead of widening `fret-ui-kit::imui`.",
            "No new public `fret-ui-kit::imui` collection rename helper is admitted in this lane.",
        ] {
            assert!(
                IMUI_COLLECTION_RENAME_M1_NOTE.contains(marker),
                "the collection rename M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-collection-rename-v1` as:",
            "a closeout record for the landed app-owned collection rename slice",
            "No reopening of the closed select-all lane or wider generic key-owner/helper questions.",
        ] {
            assert!(
                IMUI_COLLECTION_RENAME_CLOSEOUT.contains(marker),
                "the collection rename closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-rename-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-select-all-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-rename-v1/M1_APP_OWNED_RENAME_SLICE_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md\"",
            "immediate_mode_workstream_freezes_the_p1_collection_rename_follow_on",
            "proof_collection_begin_rename_session_prefers_active_visible_asset",
            "proof_collection_begin_rename_session_falls_back_to_first_visible_asset",
            "proof_collection_rename_shortcut_matches_plain_f2_only",
            "proof_collection_commit_rename_updates_label_without_touching_order_or_ids",
            "imui_editor_collection_rename_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_RENAME_WORKSTREAM.contains(marker),
                "the collection rename lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-rename-v1/` now records the closed",
            "app-owned collection rename slice in `imui_editor_proof_demo`",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection rename follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-rename-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
            "`docs/workstreams/imui-collection-rename-v1/M1_APP_OWNED_RENAME_SLICE_2026-04-23.md`",
            "`docs/workstreams/imui-collection-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection rename lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection rename lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection rename lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-collection-rename-v1/WORKSTREAM.json`"),
            "the workstream index should list the collection rename lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC
                .contains("`docs/workstreams/imui-collection-rename-v1/WORKSTREAM.json`"),
            "the todo tracker should list the collection rename lane state file explicitly"
        );
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_collection_inline_rename_follow_on() {
        for marker in [
            "The closed collection rename lane already landed modal/dialog breadth and left inline product depth open.",
            "The first landable target is therefore still narrow:",
            "route F2 plus the existing context-menu entry through one app-owned inline rename session,",
            "render the editor inside the existing active asset tile,",
            "Do not reopen the closed modal lane by widening `fret-ui-kit::imui` with a generic inline-edit helper.",
        ] {
            assert!(
                IMUI_COLLECTION_INLINE_RENAME_DESIGN.contains(marker),
                "the collection inline-rename design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection rename lane already landed modal/dialog rename breadth.",
            "The current proof surface already has the right ingredients for a narrow app-owned inline rename slice:",
            "The repo already has an editor-owned inline text-entry control we can embed locally without widening `fret-ui-kit::imui`.",
            "Dear ImGui-class collection/product depth now points at inline rename posture more than another popup contract.",
        ] {
            assert!(
                IMUI_COLLECTION_INLINE_RENAME_M0_NOTE.contains(marker),
                "the collection inline-rename baseline audit should keep the new-lane justification explicit: {marker}"
            );
        }

        for marker in [
            "The collection proof now supports one app-owned inline rename slice.",
            "F2 and the existing context-menu entry now start one app-owned inline rename editor for the active collection asset.",
            "The inline editor uses `TextField` plus a proof-local focus handoff instead of widening `fret-ui-kit::imui`.",
            "Committing rename still updates the visible label while preserving stable asset ids and collection order.",
            "No new public `fret-ui-kit::imui` inline-edit or collection rename helper is admitted in this lane.",
        ] {
            assert!(
                IMUI_COLLECTION_INLINE_RENAME_M1_NOTE.contains(marker),
                "the collection inline-rename M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-collection-inline-rename-v1` as:",
            "a closeout record for the landed app-owned collection inline rename slice",
            "No reopening of the closed modal rename lane or wider generic key-owner/helper questions.",
        ] {
            assert!(
                IMUI_COLLECTION_INLINE_RENAME_CLOSEOUT.contains(marker),
                "the collection inline-rename closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-inline-rename-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-rename-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-inline-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-inline-rename-v1/M1_APP_OWNED_INLINE_RENAME_SLICE_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md\"",
            "immediate_mode_workstream_freezes_the_p1_collection_inline_rename_follow_on",
            "proof_collection_commit_rename_rejects_empty_trimmed_label",
            "imui_editor_collection_rename_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_INLINE_RENAME_WORKSTREAM.contains(marker),
                "the collection inline-rename lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-inline-rename-v1/` now records the closed",
            "app-owned collection inline rename slice in `imui_editor_proof_demo`",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection inline-rename follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-inline-rename-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-inline-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
            "`docs/workstreams/imui-collection-inline-rename-v1/M1_APP_OWNED_INLINE_RENAME_SLICE_2026-04-23.md`",
            "`docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection inline-rename lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection inline-rename lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection inline-rename lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-collection-inline-rename-v1/WORKSTREAM.json`"),
            "the workstream index should list the collection inline-rename lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC
                .contains("`docs/workstreams/imui-collection-inline-rename-v1/WORKSTREAM.json`"),
            "the todo tracker should list the collection inline-rename lane state file explicitly"
        );
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_collection_modularization_follow_on() {
        for marker in [
            "The closed collection inline rename lane already landed the current app-owned collection product depth, but the host proof still kept too much collection implementation in one file.",
            "The first correct target is therefore structural rather than behavioral:",
            "move the collection proof into `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`,",
            "keep the host file on `mod collection;` plus one render call and drag-asset delegation,",
            "Do not widen `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` for a demo-local maintenance problem.",
        ] {
            assert!(
                IMUI_COLLECTION_MODULARIZATION_DESIGN.contains(marker),
                "the collection modularization design should keep the structural target explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection inline rename lane already landed the current app-owned collection product depth.",
            "The current collection proof now spans enough owner-local helpers and models that host-file shape is a real maintenance concern.",
            "A demo-local `collection.rs` module is sufficient to reduce that pressure without widening any public surface.",
            "The frozen proof-budget rule still blocks shared helper growth from one proof surface.",
        ] {
            assert!(
                IMUI_COLLECTION_MODULARIZATION_M0_NOTE.contains(marker),
                "the collection modularization baseline audit should keep the owner/problem framing explicit: {marker}"
            );
        }

        for marker in [
            "The collection proof now lives in one demo-local `collection.rs` module under `imui_editor_proof_demo`.",
            "The host file now routes collection rendering through `collection::render_collection_first_asset_browser_proof(ui)` and uses `collection::authoring_parity_collection_assets()` for the drag-chip seed set.",
            "Collection unit tests now live beside the module and the new modularization surface test freezes the host/module boundary explicitly.",
            "Existing collection surface tests now read `collection.rs` for behavior anchors instead of pretending the host still owns the implementation inline.",
            "No new public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API is admitted in this lane.",
        ] {
            assert!(
                IMUI_COLLECTION_MODULARIZATION_M1_NOTE.contains(marker),
                "the collection modularization M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-editor-proof-collection-modularization-v1` as:",
            "a closeout record for the landed demo-local collection module slice",
            "No reopening of the closed inline-rename lane or premature shared-helper growth from one proof surface.",
        ] {
            assert!(
                IMUI_COLLECTION_MODULARIZATION_CLOSEOUT.contains(marker),
                "the collection modularization closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-editor-proof-collection-modularization-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-inline-rename-v1\"",
            "\"path\": \"docs/workstreams/imui-editor-proof-collection-modularization-v1/M0_BASELINE_AUDIT_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-editor-proof-collection-modularization-v1/M1_DEMO_LOCAL_COLLECTION_MODULE_SLICE_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md\"",
            "immediate_mode_workstream_freezes_the_p1_collection_modularization_follow_on",
            "proof_collection_drag_rect_normalizes_drag_direction",
            "proof_collection_commit_rename_rejects_empty_trimmed_label",
            "imui_editor_collection_modularization_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_MODULARIZATION_WORKSTREAM.contains(marker),
                "the collection modularization lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-editor-proof-collection-modularization-v1/` now records the closed demo-local collection module slice",
            "resets the default next non-multi-window priority to broader app-owned command-package breadth",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection modularization follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-editor-proof-collection-modularization-v1/DESIGN.md`",
            "`docs/workstreams/imui-editor-proof-collection-modularization-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
            "`docs/workstreams/imui-editor-proof-collection-modularization-v1/M1_DEMO_LOCAL_COLLECTION_MODULE_SLICE_2026-04-23.md`",
            "`docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection modularization lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection modularization lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection modularization lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC.contains(
                "`docs/workstreams/imui-editor-proof-collection-modularization-v1/WORKSTREAM.json`"
            ),
            "the workstream index should list the collection modularization lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC.contains(
                "`docs/workstreams/imui-editor-proof-collection-modularization-v1/WORKSTREAM.json`"
            ),
            "the todo tracker should list the collection modularization lane state file explicitly"
        );

        for marker in [
            "mod collection;",
            "collection::render_collection_first_asset_browser_proof(ui);",
            "collection::authoring_parity_collection_assets()",
        ] {
            assert!(
                IMUI_EDITOR_PROOF_DEMO.contains(marker),
                "the host proof should keep the collection boundary explicit after modularization: {marker}"
            );
        }

        for marker in [
            "fn proof_collection_assets_in_visible_order(",
            "fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
            "struct ProofCollectionAsset {",
            "fn proof_collection_drag_rect_normalizes_drag_direction()",
        ] {
            assert!(
                !IMUI_EDITOR_PROOF_DEMO.contains(marker),
                "the host proof should not keep the collection implementation inline after modularization: {marker}"
            );
        }

        for marker in [
            "pub(super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
            "pub(super) fn render_collection_first_asset_browser_proof(",
            "ui: &mut fret_imui::ImUi<'_, '_, KernelApp>,",
            "#[cfg(test)]",
            "fn proof_collection_drag_rect_normalizes_drag_direction() {",
        ] {
            assert!(
                IMUI_EDITOR_PROOF_DEMO_COLLECTION_MODULE.contains(marker),
                "the demo-local collection module should keep the modularized implementation and test floor explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_closes_the_p1_collection_command_package_follow_on() {
        for marker in [
            "historical execution reference (closed lane)",
            "The closed collection modularization lane already proved the host file can stay slim without widening any public surface.",
            "The first landable target is therefore command-package breadth rather than helper growth:",
            "land `Primary+D` duplicate-selected on the existing proof surface,",
            "route the same duplicate command through the explicit button and context menu,",
            "`docs/workstreams/imui-collection-second-proof-surface-v1/`",
            "Do not introduce a shared `collection_commands(...)` or `duplicate_selected(...)` helper in `fret-ui-kit::imui`.",
        ] {
            assert!(
                IMUI_COLLECTION_COMMAND_PACKAGE_DESIGN.contains(marker),
                "the collection command-package design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The closed collection modularization lane already reset the default next non-multi-window priority to broader app-owned command-package depth.",
            "The current collection proof already has enough local substrate for a first duplicate-selected slice: stable ids, selection owner, context menu, button affordance, and status readouts.",
            "A proof-local command status model is sufficient for this lane; system clipboard, platform reveal, or generic command buses are unnecessary.",
            "The frozen proof-budget rule still blocks shared helper growth from one proof surface.",
        ] {
            assert!(
                IMUI_COLLECTION_COMMAND_PACKAGE_M0_NOTE.contains(marker),
                "the collection command-package baseline audit should keep the owner/problem framing explicit: {marker}"
            );
        }

        for marker in [
            "The collection proof now lands the first app-owned command-package slice with `Primary+D` duplicate-selected.",
            "The same duplicate command now routes through keyboard, the explicit button, and the collection context menu.",
            "Duplicate results now reselect the copied set, preserve an active copied tile when possible, and publish app-owned command status feedback.",
            "No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.",
        ] {
            assert!(
                IMUI_COLLECTION_COMMAND_PACKAGE_M1_NOTE.contains(marker),
                "the collection command-package M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "The collection proof now routes the existing inline rename command through an explicit",
            "`Rename active asset` button in addition to `F2` and the collection context menu.",
            "Button and context-menu rename activation now share one demo-local app helper for the render",
            "No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.",
        ] {
            assert!(
                IMUI_COLLECTION_COMMAND_PACKAGE_M2_NOTE.contains(marker),
                "the collection command-package M2 note should keep the landed rename-trigger slice explicit: {marker}"
            );
        }

        for marker in [
            "Treat `imui-collection-command-package-v1` as:",
            "duplicate-selected plus explicit rename-trigger breadth is coherent enough",
            "the default next non-multi-window follow-on is now",
            "`imui-collection-second-proof-surface-v1`, not a third command verb in this lane.",
            "No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.",
            "Do not reopen this folder just to add one more verb.",
        ] {
            assert!(
                IMUI_COLLECTION_COMMAND_PACKAGE_CLOSEOUT.contains(marker),
                "the collection command-package closeout should keep the closed verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-command-package-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-editor-proof-collection-modularization-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-command-package-v1/M0_BASELINE_AUDIT_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-command-package-v1/M1_APP_OWNED_DUPLICATE_COMMAND_SLICE_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-command-package-v1/M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-second-proof-surface-v1/WORKSTREAM.json\"",
            "immediate_mode_workstream_closes_the_p1_collection_command_package_follow_on",
            "proof_collection_duplicate_shortcut_matches_primary_d_only",
            "proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy",
            "proof_collection_begin_rename_session_prefers_active_visible_asset",
            "proof_collection_rename_shortcut_matches_plain_f2_only",
            "imui_editor_collection_command_package_surface",
            "imui_editor_collection_rename_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_COMMAND_PACKAGE_WORKSTREAM.contains(marker),
                "the collection command-package lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-command-package-v1/` now records the closed",
            "lands duplicate-selected plus explicit rename-trigger slices in",
            "moves the next non-multi-window priority to a second proof surface.",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the collection command-package follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-command-package-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-command-package-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
            "`docs/workstreams/imui-collection-command-package-v1/M1_APP_OWNED_DUPLICATE_COMMAND_SLICE_2026-04-23.md`",
            "`docs/workstreams/imui-collection-command-package-v1/M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md`",
            "`docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection command-package lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection command-package lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection command-package lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-collection-command-package-v1/WORKSTREAM.json`"),
            "the workstream index should list the collection command-package lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC
                .contains("`docs/workstreams/imui-collection-command-package-v1/WORKSTREAM.json`"),
            "the todo tracker should list the collection command-package lane state file explicitly"
        );
    }

    #[test]
    fn immediate_mode_workstream_closes_the_p1_collection_second_proof_surface_follow_on() {
        for marker in [
            "Status: historical execution reference (closed lane)",
            "this lane starts immediately after the bounded command-package lane",
            "freeze and then land a materially different second collection proof surface",
            "`CLOSEOUT_AUDIT_2026-04-23.md` now closes this lane on a no-helper-widening verdict.",
            "`apps/fret-examples/src/editor_notes_demo.rs`",
            "`apps/fret-examples/src/workspace_shell_demo.rs`",
            "This work should not be forced back into",
            "`imui-collection-command-package-v1`.",
            "This lane also should not create a new dedicated asset-grid/file-browser demo.",
            "Prefer existing demos, with `editor_notes_demo.rs` as the primary candidate and",
            "`workspace_shell_demo.rs` as supporting proof.",
            "Widening `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.",
        ] {
            assert!(
                IMUI_COLLECTION_SECOND_PROOF_SURFACE_DESIGN.contains(marker),
                "the second proof-surface design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "`imui_editor_proof_demo` plus the closed command-package lane still count as only one current",
            "`editor_notes_demo.rs` is the smallest materially different shell-mounted second proof",
            "`workspace_shell_demo.rs` remains supporting evidence for shell-mounted proof pressure",
            "No dedicated asset-grid/file-browser demo should be introduced yet.",
            "The frozen proof-budget rule still blocks shared helper growth until a second real proof",
        ] {
            assert!(
                IMUI_COLLECTION_SECOND_PROOF_SURFACE_M0_NOTE.contains(marker),
                "the second proof-surface baseline audit should keep the owner/problem framing explicit: {marker}"
            );
        }

        for marker in [
            "`editor_notes_demo.rs` now carries the first materially different shell-mounted collection proof",
            "the old single-purpose button group is now an explicit `Scene collection` surface",
            "a stable collection summary test id",
            "a stable collection list test id",
            "app-owned row labels that include title, role, and active/available state",
            "no `fret-ui-kit::imui` collection helper widening",
        ] {
            assert!(
                IMUI_COLLECTION_SECOND_PROOF_SURFACE_M2_NOTE.contains(marker),
                "the second proof-surface M2 note should keep the landed shell-mounted surface explicit: {marker}"
            );
        }

        for marker in [
            "Treat `imui-collection-second-proof-surface-v1` as:",
            "explicit evidence that the second collection proof surface now exists outside",
            "a no-helper-widening verdict for this cycle",
            "the two surfaces do not yet demand the same reusable helper shape",
            "No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.",
            "Do not reopen this folder just because the second proof now exists.",
        ] {
            assert!(
                IMUI_COLLECTION_SECOND_PROOF_SURFACE_CLOSEOUT.contains(marker),
                "the second proof-surface closeout should keep the no-helper-widening verdict explicit: {marker}"
            );
        }

        for marker in [
            "const TEST_ID_COLLECTION: &str = \"editor-notes-demo.collection\";",
            "const TEST_ID_COLLECTION_SUMMARY: &str = \"editor-notes-demo.collection.summary\";",
            "const TEST_ID_COLLECTION_LIST: &str = \"editor-notes-demo.collection.list\";",
            "fn editor_collection_row_label(",
            "fn editor_collection_status_label(",
            "shadcn::CardTitle::new(\"Scene collection\")",
            "Shell-mounted collection proof: choose an editor-owned surface",
            "editor_collection_row_label(",
            "ui::text(editor_collection_status_label(selected))",
            ".test_id(TEST_ID_COLLECTION)",
        ] {
            assert!(
                EDITOR_NOTES_DEMO.contains(marker),
                "editor_notes_demo should keep the shell-mounted second collection proof explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-second-proof-surface-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-command-package-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md\"",
            "\"path\": \"docs/workstreams/imui-collection-second-proof-surface-v1/M0_BASELINE_AUDIT_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-second-proof-surface-v1/M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md\"",
            "\"path\": \"docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md\"",
            "Reopen this closeout only if fresh evidence names an exact shared helper",
            "immediate_mode_workstream_closes_the_p1_collection_second_proof_surface_follow_on",
            "editor_notes_demo",
            "workspace_shell_demo",
            "editor_notes_editor_rail_surface",
            "workspace_shell_pane_proof_surface",
            "workspace_shell_editor_rail_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_SECOND_PROOF_SURFACE_WORKSTREAM.contains(marker),
                "the second proof-surface lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-second-proof-surface-v1/` now records the closed",
            "lands the `Scene collection` left-rail",
            "does not yet prove that both collection proof surfaces",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the second proof-surface follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-second-proof-surface-v1/WORKSTREAM.json`",
            "`docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-second-proof-surface-v1/M0_BASELINE_AUDIT_2026-04-23.md`",
            "`docs/workstreams/imui-collection-second-proof-surface-v1/M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md`",
            "`docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`",
            "`docs/workstreams/imui-collection-second-proof-surface-v1/EVIDENCE_AND_GATES.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the collection second proof-surface lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the collection second proof-surface lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the collection second proof-surface lane: {marker}"
            );
        }

        for marker in [
            "closed primary surface: `apps/fret-examples/src/editor_notes_demo.rs`",
            "no-helper-widening verdict:",
            "the two collection proof surfaces do not yet demand the same reusable helper shape",
            "do not reopen shared collection helpers directly from this lane.",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_NEXT_PRIORITY.contains(marker),
                "the priority audit should move the default next priority to the second proof surface: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-command-package-v1/` now closes the bounded app-owned",
            "`docs/workstreams/imui-collection-second-proof-surface-v1/` now lands a smaller",
            "What still remains is narrower Dear ImGui-class collection depth: a future helper-readiness proposal",
            "### R3) Keep shared collection helpers closed after the second proof",
            "the app-owned collection command package and smaller shell-mounted second proof are now closed",
        ] {
            assert!(
                IMUI_IMGUI_PARITY_AUDIT_V2.contains(marker),
                "the ImGui parity audit should now point at second proof-surface pressure: {marker}"
            );
        }

        assert!(
            !WORKSTREAMS_INDEX_DOC.contains(
                "imui-collection-second-proof-surface-v1/` — first n/a, latest n/a, 0 markdown docs"
            ),
            "the workstream index should not leave the second proof-surface lane as an empty placeholder"
        );
    }

    #[test]
    fn immediate_mode_workstream_closes_the_p1_collection_helper_readiness_follow_on() {
        for marker in [
            "Status: active narrow audit lane",
            "can fresh first-party evidence name an exact shared",
            "No `fret-imui` facade widening.",
            "No `fret-ui-kit::imui` public helper implementation until the audit names an exact helper.",
            "both proof surfaces need the same helper shape",
            "If no candidate passes, close this lane as another no-helper-widening verdict",
        ] {
            assert!(
                IMUI_COLLECTION_HELPER_READINESS_DESIGN.contains(marker),
                "the helper-readiness design should keep the audit boundary explicit: {marker}"
            );
        }

        for marker in [
            "Create the helper-readiness follow-on instead of reopening the closed second proof-surface lane.",
            "Compare the asset-browser grid and shell-mounted `Scene collection` outline for reusable helper pressure.",
            "Classify each candidate seam as helper-ready, app-owned policy, recipe policy, or not worth extracting.",
            "current evidence still says app-owned collection",
        ] {
            assert!(
                IMUI_COLLECTION_HELPER_READINESS_TODO.contains(marker),
                "the helper-readiness TODO should keep the current audit tasks explicit: {marker}"
            );
        }

        for marker in [
            "M0 - Lane Opened",
            "Kept `imui-collection-second-proof-surface-v1` closed.",
            "M1 - Candidate Seam Audit",
            "Status: complete",
            "M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md",
            "M2 - Verdict Or Split",
            "CLOSEOUT_AUDIT_2026-04-24.md",
            "Status: complete",
            "closes the lane with no helper widening",
            "follow-on for one exact helper shape",
        ] {
            assert!(
                IMUI_COLLECTION_HELPER_READINESS_MILESTONES.contains(marker),
                "the helper-readiness milestones should keep the audit sequence explicit: {marker}"
            );
        }

        for marker in [
            "keep helper-readiness separate from helper implementation",
            "Collection-first asset-browser proof",
            "Shell-mounted `Scene collection` proof",
            "immediate_mode_workstream_closes_the_p1_collection_helper_readiness_follow_on",
            "Do not add a shared `collection(...)`, `collection_list(...)`, or `collection_commands(...)`",
            "generic collection command helpers remain app-owned policy",
            "`CLOSEOUT_AUDIT_2026-04-24.md` closes the lane on a no-helper-widening verdict.",
        ] {
            assert!(
                IMUI_COLLECTION_HELPER_READINESS_EVIDENCE.contains(marker),
                "the helper-readiness evidence doc should keep gates and non-goals explicit: {marker}"
            );
        }

        for marker in [
            "No shared collection helper is helper-ready yet.",
            "The shared part is currently vocabulary and test-id discipline, not a reusable helper contract.",
            "`collection(...)` container helper",
            "`collection_list(...)` / `collection_rows(...)`",
            "`collection_commands(...)`",
            "Stable collection test-id convention",
            "Keep shared helper widening closed for M1.",
            "prefer docs/recipe naming guidance over public helper implementation",
        ] {
            assert!(
                IMUI_COLLECTION_HELPER_READINESS_M1_AUDIT.contains(marker),
                "the helper-readiness M1 audit should keep the no-helper-ready verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-collection-helper-readiness-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-second-proof-surface-v1\"",
            "\"path\": \"docs/workstreams/imui-collection-helper-readiness-v1/DESIGN.md\"",
            "\"path\": \"docs/workstreams/imui-collection-helper-readiness-v1/M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md\"",
            "\"path\": \"docs/workstreams/imui-collection-helper-readiness-v1/CLOSEOUT_AUDIT_2026-04-24.md\"",
            "\"path\": \"docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md\"",
            "collection-helper-readiness-closeout-source-policy",
            "imui_editor_proof_demo",
            "editor_notes_demo",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_COLLECTION_HELPER_READINESS_WORKSTREAM.contains(marker),
                "the helper-readiness workstream state should keep source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-collection-helper-readiness-v1/DESIGN.md`",
            "`docs/workstreams/imui-collection-helper-readiness-v1/TODO.md`",
            "`docs/workstreams/imui-collection-helper-readiness-v1/MILESTONES.md`",
            "`docs/workstreams/imui-collection-helper-readiness-v1/CLOSEOUT_AUDIT_2026-04-24.md`",
            "`docs/workstreams/imui-collection-helper-readiness-v1/EVIDENCE_AND_GATES.md`",
            "`docs/workstreams/imui-collection-helper-readiness-v1/WORKSTREAM.json`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the helper-readiness lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the helper-readiness lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the helper-readiness lane: {marker}"
            );
        }

        for marker in [
            "Closed narrow P1 collection helper-readiness closeout record:",
            "records the closed helper-readiness follow-on after second proof-surface closeout",
            "closes without `fret-ui-kit::imui` helper widening because both proof surfaces do",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should keep the helper-readiness scope explicit: {marker}"
            );
        }

        for marker in [
            "Treat `imui-collection-helper-readiness-v1` as a closed no-helper-widening verdict.",
            "No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.",
            "The overlap is stable evidence vocabulary and test-id discipline, not runtime/helper behavior.",
            "Do not reopen this folder for generic collection helper implementation.",
            "one exact helper shape",
        ] {
            assert!(
                IMUI_COLLECTION_HELPER_READINESS_CLOSEOUT.contains(marker),
                "the helper-readiness closeout should keep the no-helper-widening verdict explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_closes_the_p1_editor_notes_inspector_command_follow_on() {
        for marker in [
            "Status: closed narrow P1 lane",
            "deepen app-owned editor-grade behavior in an existing",
            "Add one explicit inspector-local command affordance to `editor_notes_demo.rs`.",
            "No generic command palette or command bus.",
            "No platform clipboard integration.",
            "Add a `Copy asset summary` inspector command",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_DESIGN.contains(marker),
                "the editor-notes inspector command design should keep the app-owned scope explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-editor-notes-inspector-command-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-helper-readiness-v1\"",
            "\"path\": \"docs/workstreams/imui-editor-notes-inspector-command-v1/M1_APP_OWNED_SUMMARY_COMMAND_SLICE_2026-04-24.md\"",
            "\"path\": \"docs/workstreams/imui-editor-notes-inspector-command-v1/CLOSEOUT_AUDIT_2026-04-24.md\"",
            "editor-notes-inspector-command-closeout-source-policy",
            "editor_notes_demo",
            "editor_notes_editor_rail_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_WORKSTREAM.contains(marker),
                "the editor-notes inspector command lane state should keep source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`editor_notes_demo.rs` now carries one inspector-local command/status loop:",
            "`Copy asset summary` is rendered inside the existing `InspectorPanel` / `PropertyGrid` surface.",
            "The command updates an app-owned summary status model for the selected asset.",
            "no generic command palette",
            "no platform clipboard integration",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_M1_NOTE.contains(marker),
                "the editor-notes inspector command M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "const TEST_ID_SUMMARY_COMMAND: &str = \"editor-notes-demo.inspector.summary-command\";",
            "const TEST_ID_SUMMARY_STATUS: &str = \"editor-notes-demo.inspector.summary-status\";",
            "fn editor_asset_summary_command_status(",
            "summary_status_model: Model<String>",
            "shadcn::Button::new(\"Copy asset summary\")",
            ".test_id(TEST_ID_SUMMARY_COMMAND)",
            ".test_id(TEST_ID_SUMMARY_STATUS)",
        ] {
            assert!(
                EDITOR_NOTES_DEMO.contains(marker),
                "editor_notes_demo should keep the inspector-local command/status slice explicit: {marker}"
            );
        }

        for marker in [
            "Treat `imui-editor-notes-inspector-command-v1` as a closed app-owned inspector command proof.",
            "The first `Copy asset summary` slice is coherent enough to close the lane",
            "Do not reopen this folder for generic command, clipboard, inspector, or IMUI helper implementation.",
            "the exact app-owned editor behavior still missing",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_CLOSEOUT.contains(marker),
                "the editor-notes inspector command closeout should keep the closed app-owned verdict explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_closes_the_p1_editor_notes_dirty_status_follow_on() {
        for marker in [
            "Status: closed narrow P1 lane",
            "local dirty/clean feedback for",
            "Add app-owned dirty/clean status copy to the existing notes inspector surface.",
            "No workspace dirty-close prompt.",
            "No document persistence or save command.",
            "Add a visible `Draft status` row",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DIRTY_STATUS_DESIGN.contains(marker),
                "the editor-notes dirty-status design should keep the app-owned scope explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-editor-notes-dirty-status-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-editor-notes-inspector-command-v1\"",
            "\"path\": \"docs/workstreams/imui-editor-notes-dirty-status-v1/M1_APP_OWNED_DRAFT_STATUS_SLICE_2026-04-24.md\"",
            "\"path\": \"docs/workstreams/imui-editor-notes-dirty-status-v1/CLOSEOUT_AUDIT_2026-04-24.md\"",
            "editor-notes-dirty-status-closeout-source-policy",
            "editor_notes_demo",
            "editor_notes_editor_rail_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DIRTY_STATUS_WORKSTREAM.contains(marker),
                "the editor-notes dirty-status lane state should keep source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`editor_notes_demo.rs` now carries one inspector-local draft status row:",
            "`Draft status` is rendered inside the existing `InspectorPanel` / `PropertyGrid` surface.",
            "The status is derived from the existing notes outcome plus committed line-count label.",
            "No workspace dirty-close prompt.",
            "No save/persistence command.",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DIRTY_STATUS_M1_NOTE.contains(marker),
                "the editor-notes dirty-status M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "const TEST_ID_NOTES_DRAFT_STATUS: &str = \"editor-notes-demo.inspector.notes.draft-status\";",
            "fn editor_notes_draft_status_label(",
            "cx.text(\"Draft status\")",
            ".test_id(TEST_ID_NOTES_DRAFT_STATUS)",
            "Clean draft · {committed_label}",
            "Draft canceled · preserved editor text · {committed_label}",
            "Draft preserved until commit · {committed_label}",
        ] {
            assert!(
                EDITOR_NOTES_DEMO.contains(marker),
                "editor_notes_demo should keep the inspector-local draft-status slice explicit: {marker}"
            );
        }

        for marker in [
            "Treat `imui-editor-notes-dirty-status-v1` as a closed app-owned draft-status proof.",
            "The first `Draft status` row is coherent enough to close the lane",
            "Do not reopen this folder for workspace dirty-close, persistence,",
            "the exact app-owned editor behavior still missing",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DIRTY_STATUS_CLOSEOUT.contains(marker),
                "the editor-notes dirty-status closeout should keep the closed app-owned verdict explicit: {marker}"
            );
        }

        for marker in [
            "- [x] Land one app-owned draft/dirty status row in `editor_notes_demo.rs`.",
            "- [x] Add source-policy and surface-test markers.",
            "- [x] Close this lane after the first status row.",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DIRTY_STATUS_TODO.contains(marker),
                "the editor-notes dirty-status TODO should record the closed slice: {marker}"
            );
        }

        for marker in [
            "## M1 - Draft Status Slice",
            "Status: complete",
            "## M2 - Closeout Verdict",
            "Status: complete",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DIRTY_STATUS_MILESTONES.contains(marker),
                "the editor-notes dirty-status milestones should record the completed closeout: {marker}"
            );
        }

        for marker in [
            "M1_APP_OWNED_DRAFT_STATUS_SLICE_2026-04-24.md",
            "CLOSEOUT_AUDIT_2026-04-24.md",
            "editor-notes-dirty-status-closeout-source-policy",
            "cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DIRTY_STATUS_EVIDENCE.contains(marker),
                "the editor-notes dirty-status evidence doc should name the canonical gates: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_closes_the_p1_imui_next_gap_audit() {
        for marker in [
            "Status: closed narrow P1 audit lane",
            "The recent IMUI lanes closed collection helper readiness",
            "Rank the next locally testable, non-macOS-dependent IMUI follow-on candidates.",
            "No `fret-ui-kit::imui`, `fret-imui`, `fret-authoring`, or `crates/fret-ui` API widening.",
            "No macOS-only or multi-window runner implementation.",
            "Close this audit with a ranked next-gap decision",
        ] {
            assert!(
                IMUI_NEXT_GAP_AUDIT_DESIGN.contains(marker),
                "the IMUI next-gap audit design should keep the narrow audit scope explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-next-gap-audit-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-editor-notes-dirty-status-v1\"",
            "\"path\": \"docs/workstreams/imui-next-gap-audit-v1/M1_NEXT_GAP_AUDIT_2026-04-24.md\"",
            "\"path\": \"docs/workstreams/imui-next-gap-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md\"",
            "imui-next-gap-audit-source-policy",
            "imui-editor-notes-draft-actions-v1",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_NEXT_GAP_AUDIT_WORKSTREAM.contains(marker),
                "the IMUI next-gap audit lane state should keep source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "Recommended: `imui-editor-notes-draft-actions-v1`.",
            "add app-owned `Commit draft` / `Discard draft` or equivalent draft action affordances",
            "Multi-window/tear-off remains important but is not the right next local slice",
            "Public IMUI helper growth still needs stronger two-surface proof.",
            "Parked: generic IMUI/public helper widening.",
            "without persistence, dirty-close,",
        ] {
            assert!(
                IMUI_NEXT_GAP_AUDIT_M1_NOTE.contains(marker),
                "the IMUI next-gap M1 audit should keep the ranking explicit: {marker}"
            );
        }

        for marker in [
            "Treat `imui-next-gap-audit-v1` as a closed decision record.",
            "implementation should start `imui-editor-notes-draft-actions-v1`",
            "Do not reopen `imui-editor-grade-product-closure-v1` for implementation-heavy work.",
            "Keep macOS/multi-window/tear-off work parked in runner/backend-owned lanes",
            "Gate with `editor_notes_editor_rail_surface` plus a source-policy test.",
        ] {
            assert!(
                IMUI_NEXT_GAP_AUDIT_CLOSEOUT.contains(marker),
                "the IMUI next-gap closeout should keep the follow-on decision explicit: {marker}"
            );
        }

        for marker in [
            "- [x] Rank locally testable next-gap candidates.",
            "- [x] Record a recommended next lane and explicit non-goals.",
            "## M1 - Next-Gap Audit",
            "Status: complete",
            "immediate_mode_workstream_closes_the_p1_imui_next_gap_audit",
        ] {
            assert!(
                IMUI_NEXT_GAP_AUDIT_TODO.contains(marker)
                    || IMUI_NEXT_GAP_AUDIT_MILESTONES.contains(marker)
                    || IMUI_NEXT_GAP_AUDIT_EVIDENCE.contains(marker),
                "the IMUI next-gap audit execution docs should record closure markers: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_closes_the_p1_editor_notes_draft_actions_follow_on() {
        for marker in [
            "Status: closed narrow P1 lane",
            "deepen app-owned editor-note draft actions",
            "Add app-owned draft action affordances to the existing editor-notes inspector surface.",
            "No `TextField` draft-buffer API widening.",
            "No `fret-ui-kit::imui`, `fret-imui`, `fret-authoring`, or `crates/fret-ui` API changes.",
            "These are action/status affordances, not hidden-buffer commit/discard APIs.",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DRAFT_ACTIONS_DESIGN.contains(marker),
                "the editor-notes draft-actions design should keep the app-owned scope explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-editor-notes-draft-actions-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-next-gap-audit-v1\"",
            "editor-notes-draft-actions-closeout-source-policy",
            "editor_notes_demo",
            "editor_notes_editor_rail_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DRAFT_ACTIONS_WORKSTREAM.contains(marker),
                "the editor-notes draft-actions lane state should keep source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "const TEST_ID_DRAFT_READY_COMMAND: &str = \"editor-notes-demo.inspector.notes.mark-draft-ready\";",
            "const TEST_ID_DRAFT_CLEAR_COMMAND: &str = \"editor-notes-demo.inspector.notes.clear-draft-marker\";",
            "fn editor_notes_draft_action_status(",
            "cx.text(\"Draft actions\")",
            "shadcn::Button::new(\"Mark draft ready\")",
            "shadcn::Button::new(\"Clear draft marker\")",
            ".test_id(TEST_ID_DRAFT_READY_COMMAND)",
            ".test_id(TEST_ID_DRAFT_CLEAR_COMMAND)",
            "Draft marked ready",
            "Draft marker cleared",
            "local inspector state only",
        ] {
            assert!(
                EDITOR_NOTES_DEMO.contains(marker),
                "editor_notes_demo should keep the inspector-local draft-actions slice explicit: {marker}"
            );
        }

        for marker in [
            "`editor_notes_demo.rs` now carries one inspector-local draft action row:",
            "`Mark draft ready` updates app-owned outcome/status feedback.",
            "`Clear draft marker` updates app-owned outcome/status feedback.",
            "The status copy is explicitly local inspector state only.",
            "No `TextField` draft-buffer API widening.",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DRAFT_ACTIONS_M1_NOTE.contains(marker),
                "the editor-notes draft-actions M1 note should keep the landed slice explicit: {marker}"
            );
        }

        for marker in [
            "Treat `imui-editor-notes-draft-actions-v1` as a closed app-owned draft action proof.",
            "The first `Draft actions` row is coherent enough to close the lane",
            "Do not reopen this folder for persistence, workspace dirty-close prompts, command-bus integration,",
            "preserved `TextField` draft buffer",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DRAFT_ACTIONS_CLOSEOUT.contains(marker),
                "the editor-notes draft-actions closeout should keep the closed app-owned verdict explicit: {marker}"
            );
        }

        for marker in [
            "- [x] Add app-owned draft action buttons to `editor_notes_demo.rs`.",
            "## M1 - App-Owned Draft Actions",
            "Status: complete",
            "immediate_mode_workstream_closes_the_p1_editor_notes_draft_actions_follow_on",
        ] {
            assert!(
                IMUI_EDITOR_NOTES_DRAFT_ACTIONS_TODO.contains(marker)
                    || IMUI_EDITOR_NOTES_DRAFT_ACTIONS_MILESTONES.contains(marker)
                    || IMUI_EDITOR_NOTES_DRAFT_ACTIONS_EVIDENCE.contains(marker),
                "the editor-notes draft-actions execution docs should record startup markers: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_closes_the_p1_textfield_draft_buffer_contract_audit() {
        for marker in [
            "Status: closed narrow P1 audit lane",
            "whether Fret should expose a",
            "Audit the current `TextField` buffered draft implementation.",
            "No `TextFieldOptions` or `TextField` API changes.",
            "No public model handles for the internal draft buffer.",
            "Close with an explicit verdict on whether to expose a draft-buffer contract now",
        ] {
            assert!(
                IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_DESIGN.contains(marker),
                "the TextField draft-buffer contract audit design should keep the audit scope explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-textfield-draft-buffer-contract-audit-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-editor-notes-draft-actions-v1\"",
            "textfield-draft-buffer-contract-audit-source-policy",
            "ecosystem/fret-ui-editor/src/controls/text_field.rs",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_WORKSTREAM.contains(marker),
                "the TextField draft-buffer contract audit state should keep source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "The preserved draft buffer is an internal keyed local model.",
            "draft = buffered.then(|| draft_model(cx))",
            "Commit/cancel behavior is tied to internal focus/session state.",
            "Do not expose a public `TextField` preserved draft-buffer API now.",
            "A future API-proof lane must provide all of the following",
        ] {
            assert!(
                IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_M1_NOTE.contains(marker),
                "the TextField draft-buffer M1 audit should keep the no-public-API finding explicit: {marker}"
            );
        }

        for marker in [
            "Treat `imui-textfield-draft-buffer-contract-audit-v1` as a closed no-public-API verdict.",
            "Do not expose a `TextField` preserved draft-buffer contract yet.",
            "Public draft model handles from `TextFieldOptions`.",
            "Generic app-facing commit/discard buttons wired into hidden `TextField` draft state.",
            "Start a new API-proof lane only when a proof surface truly needs external preserved-draft commit/discard.",
        ] {
            assert!(
                IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_CLOSEOUT.contains(marker)
                    || IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_WORKSTREAM.contains(marker),
                "the TextField draft-buffer closeout should keep the closed verdict explicit: {marker}"
            );
        }

        for marker in [
            "fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String>",
            "cx.local_model(String::new)",
            "fn commit_buffered_text_field(",
            "fn cancel_buffered_text_field(",
            "TextFieldBlurBehavior::PreserveDraft => BufferedTextFieldPendingBlurPlan::Clear",
            "install_buffered_text_field_blur_handler",
        ] {
            assert!(
                IMUI_TEXT_FIELD_RS.contains(marker),
                "TextField implementation should keep the audited internal draft-buffer mechanics visible: {marker}"
            );
        }

        for marker in [
            "- [x] Decide whether to expose a public draft-buffer API now.",
            "## M1 - Draft Buffer Contract Audit",
            "Status: complete",
            "immediate_mode_workstream_closes_the_p1_textfield_draft_buffer_contract_audit",
        ] {
            assert!(
                IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_TODO.contains(marker)
                    || IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_MILESTONES.contains(marker)
                    || IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_EVIDENCE.contains(marker),
                "the TextField draft-buffer audit execution docs should record closure markers: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_collection_pane_proof_m3_pane_first_workspace_shell_slice_is_explicit() {
        for marker in [
            "struct WorkspaceShellPaneProofState {",
            "fn workspace_shell_pane_proof<'a, Cx>(",
            "use fret::{imui::prelude::*, shadcn, shadcn::themes::ShadcnColorScheme};",
            "imui_build(cx, out, move |ui| {",
            "workspace-shell-pane-{}-proof.shell",
            "workspace-shell-pane-{}-proof.toolbar",
            "workspace-shell-pane-{}-proof.tabs",
            "workspace-shell-pane-{}-proof.inspector",
            "workspace-shell-pane-{}-proof.status",
            "Decision: keep the current `child_region` seam for M3.",
            "vec![workspace_shell_pane_proof(",
        ] {
            assert!(
                WORKSPACE_SHELL_DEMO.contains(marker),
                "workspace_shell_demo should keep the M3 pane-first proof explicit: {marker}"
            );
        }

        for marker in [
            "Keep `apps/fret-examples/src/workspace_shell_demo.rs` as the pane-first M3 proof surface.",
            "Close M3 with a shell-mounted pane proof inside the existing workspace shell demo.",
            "Keep `ecosystem/fret-ui-kit/src/imui/child_region.rs` unchanged for M3.",
            "No narrower pane-only diagnostics path is required at M3 because the existing workspace shell diag floor remains sufficient.",
        ] {
            assert!(
                IMUI_COLLECTION_PANE_PROOF_M3_NOTE.contains(marker),
                "the M3 pane proof note should keep the pane-first closure explicit: {marker}"
            );
        }

        for marker in [
            "\"path\": \"docs/workstreams/imui-collection-pane-proof-v1/M3_PANE_PROOF_CLOSURE_2026-04-21.md\"",
            "\"name\": \"pane-proof-source-policy\"",
            "\"name\": \"pane-proof-surface-floor\"",
            "workspace_shell_pane_proof_surface",
        ] {
            assert!(
                IMUI_COLLECTION_PANE_PROOF_WORKSTREAM.contains(marker),
                "the collection/pane proof lane state should keep the M3 pane-first gates explicit: {marker}"
            );
        }
    }

    #[test]
    fn immediate_mode_workstream_freezes_the_p1_child_region_depth_follow_on() {
        for marker in [
            "The current helper is intentionally small.",
            "`ChildRegionOptions` currently exposes only:",
            "Frame and padding policy",
            "Axis-specific resize",
            "Axis-specific auto-resize",
            "Do not start with resize or auto-resize implementation immediately.",
            "Reopening collection-first or pane-first proof breadth.",
        ] {
            assert!(
                IMUI_CHILD_REGION_DEPTH_DESIGN.contains(marker),
                "the child-region depth design should keep the target-surface framing explicit: {marker}"
            );
        }

        for marker in [
            "The basic pane-proof question is already closed",
            "the remaining question is child-region depth, not first-party pane proof absence.",
            "Child-specific menu composition is no longer the leading blocker",
            "The owner split is already clear enough to avoid runtime drift",
            "Not every Dear ImGui child flag should be cloned",
        ] {
            assert!(
                IMUI_CHILD_REGION_DEPTH_M0_NOTE.contains(marker),
                "the child-region depth baseline audit should keep the new-lane justification explicit: {marker}"
            );
        }

        for marker in [
            "do not clone Dear ImGui's `size_arg` grammar into generic IMUI.",
            "frame/padding posture is the first credible generic child-region candidate for M2.",
            "axis-specific resize should stay out of generic `child_region` for now.",
            "auto-resize / always-auto-resize should stay deferred for now.",
            "do not admit a `BeginChild() -> bool`-style return contract in generic IMUI.",
        ] {
            assert!(
                IMUI_CHILD_REGION_DEPTH_M1_NOTE.contains(marker),
                "the child-region depth M1 note should keep the target-surface verdict explicit: {marker}"
            );
        }

        for marker in [
            "M2 lands `ChildRegionChrome::{Framed, Bare}` as the only admitted generic child-region depth slice",
            "default `ChildRegionChrome::Framed`",
            "opt-in `ChildRegionChrome::Bare`",
            "keep resize / auto-resize / focus-boundary flattening / begin-return posture out of generic",
        ] {
            assert!(
                IMUI_CHILD_REGION_DEPTH_M2_NOTE.contains(marker),
                "the child-region depth M2 note should keep the chrome-slice verdict explicit: {marker}"
            );
        }

        for marker in [
            "Status: closed closeout record",
            "Treat `imui-child-region-depth-v1` as:",
            "a closeout record for the landed `ChildRegionChrome::{Framed, Bare}` slice,",
            "start a different narrower follow-on only if stronger first-party proof shows the current",
        ] {
            assert!(
                IMUI_CHILD_REGION_DEPTH_CLOSEOUT.contains(marker),
                "the child-region depth closeout should keep the shipped verdict explicit: {marker}"
            );
        }

        for marker in [
            "\"slug\": \"imui-child-region-depth-v1\"",
            "\"status\": \"closed\"",
            "\"scope_kind\": \"closeout\"",
            "\"follow_on_of\": \"imui-collection-pane-proof-v1\"",
            "\"path\": \"docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md\"",
            "\"path\": \"docs/workstreams/imui-child-region-depth-v1/M1_TARGET_SURFACE_FREEZE_2026-04-22.md\"",
            "\"path\": \"docs/workstreams/imui-child-region-depth-v1/M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md\"",
            "\"path\": \"docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md\"",
            "immediate_mode_workstream_freezes_the_p1_child_region_depth_follow_on",
            "imui_adapter_seam_smoke",
            "child_region_helper_stacks_content_and_forwards_scroll_options",
            "child_region_helper_can_host_menu_bar_and_popup_menu",
            "child_region_helper_can_switch_between_framed_and_bare_chrome",
            "workspace_shell_pane_proof_surface",
            "\"default_action\": \"start_follow_on\"",
        ] {
            assert!(
                IMUI_CHILD_REGION_DEPTH_WORKSTREAM.contains(marker),
                "the child-region depth lane state should keep the source-policy markers explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-child-region-depth-v1/` now records the closed child-region",
            "depth verdict: the bounded `ChildRegionChrome::{Framed, Bare}` slice is landed, while",
        ] {
            assert!(
                IMUI_EDITOR_GRADE_PRODUCT_CLOSURE_TODO.contains(marker),
                "the umbrella lane should keep the child-region depth follow-on explicit: {marker}"
            );
        }

        for marker in [
            "`docs/workstreams/imui-child-region-depth-v1/DESIGN.md`",
            "`docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md`",
            "`docs/workstreams/imui-child-region-depth-v1/M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`",
            "`docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`",
        ] {
            assert!(
                WORKSTREAMS_INDEX_DOC.contains(marker),
                "the workstream index should list the child-region depth lane: {marker}"
            );
            assert!(
                ROADMAP_DOC.contains(marker),
                "the roadmap should list the child-region depth lane: {marker}"
            );
            assert!(
                TODO_TRACKER_DOC.contains(marker),
                "the todo tracker should list the child-region depth lane: {marker}"
            );
        }

        assert!(
            WORKSTREAMS_INDEX_DOC
                .contains("`docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json`"),
            "the workstream index should list the child-region depth lane state file explicitly"
        );
        assert!(
            TODO_TRACKER_DOC
                .contains("`docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json`"),
            "the todo tracker should list the child-region depth lane state file explicitly"
        );
        assert!(
            ROADMAP_DOC.contains(
                "`docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`"
            ),
            "the roadmap should point to the child-region depth closeout explicitly"
        );
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
            "Campaign manifests still only gate on stable `requires_capabilities`",
            "Do not add an automated mixed-DPI gate in this lane yet.",
            "Keep the bounded P3 campaign generic and portable across single-monitor and mixed-DPI hosts.",
            "the real Windows mixed-DPI acceptance pair as the only remaining open proof item",
            "start a narrow diagnostics follow-on",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_MIXED_DPI_AUTOMATION_DECISION_NOTE
                    .contains(marker),
                "the docking parity lane should keep the mixed-DPI automation decision explicit: {marker}"
            );
        }

        for marker in [
            "M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md",
            "do not add a mixed-DPI-only automated gate",
            "Manual acceptance run on a real mixed-DPI setup",
            "Result: no, not honestly in this lane yet.",
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_TODO_DOC.contains(marker),
                "the mixed-DPI TODO state should keep the automation decision explicit: {marker}"
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
        ] {
            assert!(
                DOCKING_MULTIWINDOW_IMGUI_PARITY_WORKSTREAM.contains(marker),
                "the docking parity lane should keep the Wayland compositor runbook reachable: {marker}"
            );
        }

        for marker in [
            "Run this only on a Linux native Wayland session.",
            "`XDG_SESSION_TYPE=wayland`",
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
    fn imui_hello_demo_prefers_root_fret_imui_facade_lane() {
        assert!(
            IMUI_HELLO_DEMO
                .contains("use fret::{FretApp, advanced::prelude::*, imui::prelude::*};")
        );
        assert!(IMUI_HELLO_DEMO.contains("imui_in(cx, |ui| {"));
        assert!(IMUI_HELLO_DEMO.contains("ui.text(format!(\"Count: {count}\"));"));
        assert!(IMUI_HELLO_DEMO.contains("ui.button(\"Increment\").clicked()"));
        assert!(!IMUI_HELLO_DEMO.contains("fret_imui::imui_in(cx, |ui| {"));
        assert!(!IMUI_HELLO_DEMO.contains("use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;"));
        assert!(!IMUI_HELLO_DEMO.contains("use fret_ui_kit::imui::UiWriterUiKitExt as _;"));
    }

    #[test]
    fn imui_floating_windows_demo_prefers_root_fret_imui_facade_lane() {
        assert!(
            IMUI_FLOATING_WINDOWS_DEMO
                .contains("use fret::{FretApp, advanced::prelude::*, imui::prelude::*};")
        );
        assert!(IMUI_FLOATING_WINDOWS_DEMO.contains("imui_in(cx, |ui| {"));
        assert!(IMUI_FLOATING_WINDOWS_DEMO.contains("kit::WindowOptions::default()"));
        assert!(IMUI_FLOATING_WINDOWS_DEMO.contains("kit::FloatingWindowResizeOptions::default()"));
        assert!(IMUI_FLOATING_WINDOWS_DEMO.contains("kit::MenuItemOptions {"));
        assert!(IMUI_FLOATING_WINDOWS_DEMO.contains("kit::ComboModelOptions {"));
        assert!(!IMUI_FLOATING_WINDOWS_DEMO.contains("use fret_imui::prelude::UiWriter;"));
        assert!(!IMUI_FLOATING_WINDOWS_DEMO.contains("fret_imui::imui_in(cx, |ui| {"));
        assert!(
            !IMUI_FLOATING_WINDOWS_DEMO
                .contains("use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;")
        );
        assert!(
            !IMUI_FLOATING_WINDOWS_DEMO.contains("use fret_ui_kit::imui::UiWriterUiKitExt as _;")
        );
    }

    #[test]
    fn imui_response_signals_demo_prefers_root_fret_imui_facade_lane() {
        assert!(
            IMUI_RESPONSE_SIGNALS_DEMO
                .contains("use fret::{FretApp, advanced::prelude::*, imui::prelude::*};")
        );
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("imui_in(cx, |ui| {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::SliderOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::InputTextOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::MenuItemOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::ComboOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::SelectableOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::ComboModelOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::MenuBarOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::BeginMenuOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::BeginSubmenuOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::TabBarOptions {"));
        assert!(IMUI_RESPONSE_SIGNALS_DEMO.contains("kit::TabItemOptions {"));
        assert!(!IMUI_RESPONSE_SIGNALS_DEMO.contains("fret_imui::imui_in(cx, |ui| {"));
        assert!(
            !IMUI_RESPONSE_SIGNALS_DEMO
                .contains("use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;")
        );
        assert!(
            !IMUI_RESPONSE_SIGNALS_DEMO.contains("use fret_ui_kit::imui::UiWriterUiKitExt as _;")
        );
    }

    #[test]
    fn imui_interaction_showcase_demo_prefers_root_fret_imui_facade_lane() {
        assert!(
            IMUI_INTERACTION_SHOWCASE_DEMO
                .contains("use fret::{FretApp, advanced::prelude::*, imui::prelude::*};")
        );
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("imui(cx, move |ui| {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::ButtonOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::ButtonArrowDirection::Left"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::RadioOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::SliderOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::ComboModelOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::InputTextOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::MenuBarOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::BeginMenuOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::BeginSubmenuOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::MenuItemOptions::default()"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::TabBarOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::TabItemOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::ChildRegionOptions {"));
        assert!(IMUI_INTERACTION_SHOWCASE_DEMO.contains("kit::ScrollOptions {"));
        assert!(!IMUI_INTERACTION_SHOWCASE_DEMO.contains("fret_imui::imui(cx, move |ui| {"));
        assert!(
            !IMUI_INTERACTION_SHOWCASE_DEMO
                .contains("use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;")
        );
        assert!(
            !IMUI_INTERACTION_SHOWCASE_DEMO
                .contains("use fret_ui_kit::imui::UiWriterUiKitExt as _;")
        );
    }

    #[test]
    fn imui_shadcn_adapter_demo_prefers_root_fret_imui_facade_lane() {
        assert!(
            IMUI_SHADCN_ADAPTER_DEMO
                .contains("use fret::{FretApp, advanced::prelude::*, imui::prelude::*};")
        );
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("imui_in(cx, |ui| {"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("imui(cx, move |ui| {"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("kit::ButtonOptions {"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("kit::SwitchOptions {"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("kit::SliderOptions {"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("kit::ComboModelOptions {"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("kit::InputTextOptions {"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("kit::TableColumn::fill(\"Signal\")"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("kit::TableOptions {"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("kit::VirtualListOptions {"));
        assert!(IMUI_SHADCN_ADAPTER_DEMO.contains("kit::VirtualListMeasureMode::Fixed"));
        assert!(!IMUI_SHADCN_ADAPTER_DEMO.contains("fret_imui::imui_in(cx, |ui| {"));
        assert!(!IMUI_SHADCN_ADAPTER_DEMO.contains("fret_imui::imui(cx, move |ui| {"));
        assert!(
            !IMUI_SHADCN_ADAPTER_DEMO
                .contains("use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;")
        );
        assert!(
            !IMUI_SHADCN_ADAPTER_DEMO.contains("use fret_ui_kit::imui::UiWriterUiKitExt as _;")
        );
    }

    #[test]
    fn imui_editor_proof_demo_prefers_root_fret_imui_entry_surface() {
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("use fret::imui::prelude::*;"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("use fret_ui_editor::imui as editor_imui;"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("use fret_ui_kit::imui::ImUiMultiSelectState;"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("imui(cx, |ui| {"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("imui(cx, move |ui| {"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("imui_build(cx, out, |ui| {"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("imui_build(cx, &mut out, move |ui| {"));
        assert!(IMUI_EDITOR_PROOF_DEMO.contains("imui_build(cx, out, f);"));
        assert!(!IMUI_EDITOR_PROOF_DEMO.contains("fret_imui::imui(cx, |ui| {"));
        assert!(!IMUI_EDITOR_PROOF_DEMO.contains("fret_imui::imui(cx, move |ui| {"));
        assert!(!IMUI_EDITOR_PROOF_DEMO.contains("fret_imui::imui_build(cx, out, |ui| {"));
        assert!(
            !IMUI_EDITOR_PROOF_DEMO.contains("fret_imui::imui_build(cx, &mut out, move |ui| {")
        );
        assert!(!IMUI_EDITOR_PROOF_DEMO.contains("fret_imui::imui_build(cx, out, f);"));
        assert!(
            !IMUI_EDITOR_PROOF_DEMO.contains("use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;")
        );
        assert!(!IMUI_EDITOR_PROOF_DEMO.contains("use fret_ui_kit::imui::UiWriterUiKitExt as _;"));
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
    fn workspace_shell_demo_prefers_root_fret_imui_entry_surface() {
        assert!(
            WORKSPACE_SHELL_DEMO.contains(
                "use fret::{imui::prelude::*, shadcn, shadcn::themes::ShadcnColorScheme};"
            )
        );
        assert!(WORKSPACE_SHELL_DEMO.contains("imui_build(cx, out, move |ui| {"));
        assert!(!WORKSPACE_SHELL_DEMO.contains("fret_imui::imui_build(cx, out, move |ui| {"));
        assert!(!WORKSPACE_SHELL_DEMO.contains("UiWriterImUiFacadeExt as _"));
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
    fn direct_leaf_visibility_reads_use_grouped_selector_model_layout() {
        for src in [
            CUSTOM_EFFECT_V2_WEB_DEMO,
            CUSTOM_EFFECT_V2_GLASS_CHROME_WEB_DEMO,
            CUSTOM_EFFECT_V2_IDENTITY_WEB_DEMO,
            CUSTOM_EFFECT_V2_LUT_WEB_DEMO,
            EXTERNAL_TEXTURE_IMPORTS_DEMO,
            EXTERNAL_TEXTURE_IMPORTS_WEB_DEMO,
            EXTERNAL_VIDEO_IMPORTS_AVF_DEMO,
            EXTERNAL_VIDEO_IMPORTS_MF_DEMO,
        ] {
            let normalized = src.split_whitespace().collect::<String>();
            for marker in [
                "use fret::advanced::view::AppRenderDataExt as _;",
                "selector_model_layout(",
            ] {
                let marker = marker.split_whitespace().collect::<String>();
                assert!(normalized.contains(&marker), "missing marker: {marker}");
            }
            for legacy in [
                "cx.observe_model(&show, Invalidation::Layout);",
                "cx.observe_model(&show_model, Invalidation::Layout);",
                "cx.observe_model(&st.show, Invalidation::Layout);",
                "cx.app.models().read(&show, |v| *v).unwrap_or(true)",
                "cx.app.models().read(&show_model, |v| *v).unwrap_or(true)",
                "cx.app.models().read(&st.show, |v| *v).unwrap_or(true)",
            ] {
                let legacy = legacy.split_whitespace().collect::<String>();
                assert!(
                    !normalized.contains(&legacy),
                    "legacy marker still present: {legacy}"
                );
            }
        }
    }

    #[test]
    fn stress_render_roots_use_grouped_selector_model_layout() {
        for src in [VIRTUAL_LIST_STRESS_DEMO, CANVAS_DATAGRID_STRESS_DEMO] {
            let normalized = src.split_whitespace().collect::<String>();
            for marker in [
                "use fret::advanced::view::AppRenderDataExt as _;",
                "cx.data().selector_model_layout(",
            ] {
                let marker = marker.split_whitespace().collect::<String>();
                assert!(normalized.contains(&marker), "missing marker: {marker}");
            }
            for legacy in [
                "cx.observe_model(&state.tall_rows_enabled, Invalidation::Layout);",
                "cx.observe_model(&state.reversed, Invalidation::Layout);",
                "cx.observe_model(&state.items_revision, Invalidation::Layout);",
                "app.models().read(&state.tall_rows_enabled, |v| *v).unwrap_or(false);",
                "app.models().read(&state.reversed, |v| *v).unwrap_or(false);",
                "app.models().read(&state.items_revision, |v| *v).unwrap_or(0);",
                "cx.observe_model(&state.variable_sizes, Invalidation::Layout);",
                "cx.observe_model(&state.clamp_rows, Invalidation::Layout);",
                "cx.observe_model(&state.revision, Invalidation::Layout);",
                "cx.observe_model(&state.grid_output, Invalidation::Layout);",
                "app.models().read(&state.variable_sizes, |v| *v).unwrap_or(false);",
                "app.models().read(&state.clamp_rows, |v| *v).unwrap_or(false);",
                "app.models().read(&state.revision, |v| *v).unwrap_or(1);",
                "cx.app.models().read(&state.grid_output, |v| *v).unwrap_or_default();",
            ] {
                let legacy = legacy.split_whitespace().collect::<String>();
                assert!(
                    !normalized.contains(&legacy),
                    "legacy marker still present: {legacy}"
                );
            }
        }
    }

    #[test]
    fn genui_message_lane_uses_state_owned_model_helpers() {
        let normalized = GENUI_DEMO.split_whitespace().collect::<String>();
        for marker in [
            "impl GenUiState {",
            "fn clear_action_queue(&self, app: &mut KernelApp) {",
            "fn queued_invocations(",
            "fn auto_apply_enabled(&self, app: &KernelApp) -> bool {",
            "fn auto_fix_enabled(&self, app: &KernelApp) -> bool {",
            "fn editor_text_value(&self, app: &KernelApp) -> String {",
            "fn stream_text_value(&self, app: &KernelApp) -> String {",
            "fn stream_patch_only_enabled(&self, app: &KernelApp) -> bool {",
            "let auto_apply = state.auto_apply_enabled(app);",
            "let invocations = state.queued_invocations(app);",
            "state.clear_action_queue(app);",
            "let text = state.editor_text_value(app);",
            "let auto_fix = state.auto_fix_enabled(app);",
            "let text = state.stream_text_value(app);",
            "let patch_only = state.stream_patch_only_enabled(app);",
        ] {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for legacy in [
            "state.auto_apply_standard_actions.value_in_or(app.models(), true);",
            "app.models().read(&state.action_queue, |q| q.invocations.clone())",
            "state.editor_text.value_in_or_default(app.models());",
            "state.auto_fix_on_apply.value_in_or(app.models(), true);",
            "state.stream_text.value_in_or_default(app.models());",
            "state.stream_patch_only.value_in_or(app.models(), false);",
            "app.models_mut().update(&state.action_queue, |q| q.invocations.clear());",
        ] {
            let legacy = legacy.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&legacy),
                "legacy marker still present: {legacy}"
            );
        }
    }

    #[test]
    fn imui_immediate_mode_examples_use_local_state_bridge_reads() {
        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_HELLO_DEMO,
            &[
                "let enabled = enabled_state.paint_value_in(ui.cx_mut());",
                "checkbox_model(\"Enabled\", enabled_state.model())",
            ],
            &["enabled_state.value_in(ui.cx_mut().app.models())"],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_INTERACTION_SHOWCASE_DEMO,
            &[
                "bookmark_slot.layout_value_in(ui.cx_mut())",
                "tool_mode.layout_value_in(ui.cx_mut())",
                "autosave_enabled.layout_value_in(ui.cx_mut())",
                "exposure_value.layout_value_in(ui.cx_mut())",
                "review_mode.layout_value_in(ui.cx_mut())",
                "selected_tab.layout_value_in(ui.cx_mut())",
                "context_toggle.layout_value_in(ui.cx_mut())",
                "fn push_showcase_event(",
                "let id = next_id.value_in_or_default(app.models());",
            ],
            &[
                "bookmark_slot.value_in(ui.cx_mut().app.models())",
                "tool_mode.value_in(ui.cx_mut().app.models())",
                "autosave_enabled.value_in(ui.cx_mut().app.models())",
                "exposure_value.value_in(ui.cx_mut().app.models())",
                "review_mode.value_in(ui.cx_mut().app.models())",
                "selected_tab.value_in(ui.cx_mut().app.models())",
                "context_toggle.value_in(ui.cx_mut().app.models())",
            ],
        );
    }

    #[test]
    fn driver_owned_example_loops_keep_raw_model_store_reads() {
        let embedded_record = source_slice(
            EMBEDDED_VIEWPORT_DEMO,
            "fn record_embedded_viewport(",
            "pub fn run() -> anyhow::Result<()> {",
        );
        assert_source_slice_keeps_raw_driver_owner(
            embedded_record,
            &[
                "embedded::models(app, window)",
                "app.models().read(&m.clicks, |v| *v).ok()",
            ],
            &["selector_model_layout(", "layout_value_in("],
        );

        let texture_record = source_slice(
            EXTERNAL_TEXTURE_IMPORTS_DEMO,
            "fn record_engine_frame(",
            "pub fn run() -> anyhow::Result<()> {",
        );
        assert_source_slice_keeps_raw_driver_owner(
            texture_record,
            &["let show = app.models().read(&st.view.show, |v| *v).unwrap_or(true);"],
            &["selector_model_layout(", "layout_value_in("],
        );

        let texture_web_record = source_slice(
            EXTERNAL_TEXTURE_IMPORTS_WEB_DEMO,
            "fn record_engine_frame(",
            "fn handle_event(",
        );
        assert_source_slice_keeps_raw_driver_owner(
            texture_web_record,
            &["let show = app.models().read(&state.show, |v| *v).unwrap_or(true);"],
            &["selector_model_layout(", "layout_value_in("],
        );

        let video_avf_record = source_slice(
            EXTERNAL_VIDEO_IMPORTS_AVF_DEMO,
            "fn record_engine_frame(",
            "pub fn run() -> anyhow::Result<()> {",
        );
        assert_source_slice_keeps_raw_driver_owner(
            video_avf_record,
            &["let show = app.models().read(&st.view.show, |v| *v).unwrap_or(true);"],
            &["selector_model_layout(", "layout_value_in("],
        );

        let video_mf_record = source_slice(
            EXTERNAL_VIDEO_IMPORTS_MF_DEMO,
            "fn record_engine_frame(",
            "pub fn run() -> anyhow::Result<()> {",
        );
        assert_source_slice_keeps_raw_driver_owner(
            video_mf_record,
            &["let show = app.models().read(&st.view.show, |v| *v).unwrap_or(true);"],
            &["selector_model_layout(", "layout_value_in("],
        );

        let workspace_command = source_slice(
            WORKSPACE_SHELL_DEMO,
            "fn handle_command(",
            "fn handle_event(",
        );
        assert_source_slice_keeps_raw_driver_owner(
            workspace_command,
            &["let prompt = app.models().get_cloned(&state.dirty_close_prompt).flatten();"],
            &["selector_model_layout(", "layout_value_in("],
        );

        let utility_command = source_slice(
            LAUNCHER_UTILITY_WINDOW_DEMO,
            "fn on_command(",
            "fn on_event(",
        );
        assert_source_slice_keeps_raw_driver_owner(
            utility_command,
            &["let next = !st.always_on_top.value_in_or(app.models(), false);"],
            &["selector_model_layout(", "layout_value_in("],
        );

        let plot_animate = source_slice(
            PLOT_STRESS_DEMO,
            "fn maybe_animate_bounds(",
            "fn gpu_ready(",
        );
        assert_source_slice_keeps_raw_driver_owner(
            plot_animate,
            &["let animate = app.models().read(&state.animate, |v| *v).unwrap_or(false);"],
            &["selector_model_layout(", "layout_value_in("],
        );

        let plot_render = source_slice(
            PLOT_STRESS_DEMO,
            "fn render(driver: &mut PlotStressDriver, context: WinitRenderContext<'_, PlotStressWindowState>) {",
            "fn window_create_spec(",
        );
        assert_source_slice_keeps_raw_driver_owner(
            plot_render,
            &["let animate = app.models().read(&state.animate, |v| *v).unwrap_or(false);"],
            &["selector_model_layout(", "layout_value_in("],
        );
    }

    #[test]
    fn asset_helper_entrypoints_prefer_ui_assets_capability_adapters() {
        let assets_demo = ASSETS_DEMO.split_whitespace().collect::<String>();
        assert!(assets_demo.contains(
            &"use fret_ui_assets::ui::{image_stats_in, svg_stats_in, use_rgba8_image_state_in};"
                .split_whitespace()
                .collect::<String>()
        ));
        assert!(assets_demo.contains(
            &"use_rgba8_image_state_in(cx, 96, 96, checker_rgba.as_slice(), ImageColorSpace::Srgb);"
                .split_whitespace()
                .collect::<String>()
        ));
        assert!(
            assets_demo.contains(
                &"let image_stats = image_stats_in(cx);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            assets_demo.contains(
                &"let svg_stats = svg_stats_in(cx);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !assets_demo.contains(
                &"image_asset_state::use_rgba8_image_state(cx.app, cx.window,"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !assets_demo.contains(
                &"UiAssets::image_stats(cx.app);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !assets_demo.contains(
                &"UiAssets::svg_stats(cx.app);"
                    .split_whitespace()
                    .collect::<String>()
            )
        );

        let markdown_demo = MARKDOWN_DEMO.split_whitespace().collect::<String>();
        assert!(
            markdown_demo.contains(
                &"use fret_ui_assets::ui::use_rgba8_image_state_in;"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            markdown_demo.contains(
                &"let (_key, image, _status) = use_rgba8_image_state_in(cx,"
                    .split_whitespace()
                    .collect::<String>()
            )
        );
        assert!(
            !markdown_demo.contains(
                &"image_asset_state::use_rgba8_image_state("
                    .split_whitespace()
                    .collect::<String>()
            )
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
