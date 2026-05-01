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
