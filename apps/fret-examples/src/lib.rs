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
/// Use this on manual/non-`FretApp` surfaces that do not ride the `fret` facade's optional editor
/// integration middleware. The ordering stays explicit when `WindowMetricsService` changes can
/// trigger a host-theme reset: sync the host theme first, then replay the installed editor preset.
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
    const DOCKING_ARBITRATION_DEMO: &str = include_str!("docking_arbitration_demo.rs");
    const DOCKING_DEMO: &str = include_str!("docking_demo.rs");
    const DROP_SHADOW_DEMO: &str = include_str!("drop_shadow_demo.rs");
    const ECHARTS_DEMO: &str = include_str!("echarts_demo.rs");
    const EMBEDDED_VIEWPORT_DEMO: &str = include_str!("embedded_viewport_demo.rs");
    const EMPTY_IDLE_DEMO: &str = include_str!("empty_idle_demo.rs");
    const EMOJI_CONFORMANCE_DEMO: &str = include_str!("emoji_conformance_demo.rs");
    const EXTERNAL_TEXTURE_IMPORTS_DEMO: &str = include_str!("external_texture_imports_demo.rs");
    const EXTERNAL_TEXTURE_IMPORTS_WEB_DEMO: &str =
        include_str!("external_texture_imports_web_demo.rs");
    const EXTERNAL_VIDEO_IMPORTS_AVF_DEMO: &str =
        include_str!("external_video_imports_avf_demo.rs");
    const EXTERNAL_VIDEO_IMPORTS_MF_DEMO: &str = include_str!("external_video_imports_mf_demo.rs");
    const EXTRAS_MARQUEE_PERF_DEMO: &str = include_str!("extras_marquee_perf_demo.rs");
    const GENUI_DEMO: &str = include_str!("genui_demo.rs");
    const HELLO_COUNTER_DEMO: &str = include_str!("hello_counter_demo.rs");
    const HELLO_WORLD_COMPARE_DEMO: &str = include_str!("hello_world_compare_demo.rs");
    const IMAGE_HEAVY_MEMORY_DEMO: &str = include_str!("image_heavy_memory_demo.rs");
    const IMUI_EDITOR_PROOF_DEMO: &str = include_str!("imui_editor_proof_demo.rs");
    const IMUI_FLOATING_WINDOWS_DEMO: &str = include_str!("imui_floating_windows_demo.rs");
    const IMUI_HELLO_DEMO: &str = include_str!("imui_hello_demo.rs");
    const IMUI_NODE_GRAPH_DEMO: &str = include_str!("imui_node_graph_demo.rs");
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
    const TEXT_HEAVY_MEMORY_DEMO: &str = include_str!("text_heavy_memory_demo.rs");
    const TODO_DEMO: &str = include_str!("todo_demo.rs");
    const WINDOW_HIT_TEST_PROBE_DEMO: &str = include_str!("window_hit_test_probe_demo.rs");

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

    fn assert_uses_default_app_surface(src: &str) {
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(!src.contains("advanced::prelude::*"));
        assert!(!src.contains("KernelApp"));
        assert!(!src.contains("AppWindowId"));
        assert!(src.contains("fn init(_app: &mut App, _window: WindowId) -> Self"));
        assert!(src.contains("ui::single(cx, todo_page(theme, card))"));
        assert!(
            src.contains(
                "fn todo_page(theme: ThemeSnapshot, content: impl UiChild) -> impl UiChild"
            )
        );
        assert!(!src.contains("fn todo_page(cx: &mut UiCx<'_>,"));
        assert!(!src.contains("let card = card.into_element(cx);"));
        assert!(!src.contains("todo_page(theme, card).into_element(cx).into()"));
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

    fn assert_prefers_grouped_data_surface(src: &str) {
        assert!(
            src.contains("cx.data().selector(")
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
            IMUI_NODE_GRAPH_DEMO,
            IMUI_RESPONSE_SIGNALS_DEMO,
            IMUI_SHADCN_ADAPTER_DEMO,
            LAUNCHER_UTILITY_WINDOW_DEMO,
            LAUNCHER_UTILITY_WINDOW_MATERIALS_DEMO,
            LIQUID_GLASS_DEMO,
            MARKDOWN_DEMO,
            NODE_GRAPH_DEMO,
            POSTPROCESS_THEME_DEMO,
            QUERY_ASYNC_TOKIO_DEMO,
            QUERY_DEMO,
            TEXT_HEAVY_MEMORY_DEMO,
            WINDOW_HIT_TEST_PROBE_DEMO,
        ] {
            assert_uses_advanced_surface(src);
        }
    }

    #[test]
    fn todo_demo_prefers_default_app_surface() {
        assert_uses_default_app_surface(TODO_DEMO);
        assert_avoids_legacy_conversion_names(TODO_DEMO);
        assert!(
            TODO_DEMO
                .contains("bind_todo_actions(cx, &draft_state, &next_id_state, &todos_state);")
        );
        assert!(TODO_DEMO.contains("fn bind_todo_actions("));
        assert!(TODO_DEMO.contains("ui::v_flex(move |cx| ui::single(cx, content))"));
        assert!(!TODO_DEMO.contains("ui::v_flex(move |cx| ui::children![cx; content])"));
    }

    #[test]
    fn simple_todo_demo_prefers_typed_row_helper_surface() {
        assert!(SIMPLE_TODO_DEMO.contains("ui::for_each_keyed_with_cx("));
        assert!(SIMPLE_TODO_DEMO.contains("fn todo_row("));
        assert!(SIMPLE_TODO_DEMO.contains(") -> impl fret_ui_kit::IntoUiElement<App> + use<> {"));
        assert!(SIMPLE_TODO_DEMO.contains("shadcn::CardContent::new(ui::single(cx, content))"));
        assert!(SIMPLE_TODO_DEMO.contains("ui::v_flex(move |cx| ui::single(cx, card))"));
        assert!(!SIMPLE_TODO_DEMO.contains(") -> fret_ui::element::AnyElement {"));
        assert!(!SIMPLE_TODO_DEMO.contains("shadcn::CardContent::new(ui::children![cx; content])"));
        assert!(!SIMPLE_TODO_DEMO.contains("ui::v_flex(move |cx| ui::children![cx; card])"));
    }

    #[test]
    fn hello_counter_demo_prefers_root_helper_surface() {
        assert!(HELLO_COUNTER_DEMO.contains("ui::single(cx, hello_counter_page(theme, card))"));
        assert!(HELLO_COUNTER_DEMO.contains(
            "fn hello_counter_page(theme: ThemeSnapshot, card: impl UiChild) -> impl UiChild"
        ));
        assert!(!HELLO_COUNTER_DEMO.contains("fn hello_counter_page(cx: &mut UiCx<'_>,"));
        assert!(!HELLO_COUNTER_DEMO.contains(".test_id(TEST_ID_ROOT).into_element(cx).into()"));
    }

    #[test]
    fn low_level_interop_examples_keep_direct_leaf_root_contracts() {
        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_TEXTURE_IMPORTS_DEMO,
            &[
                "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalTextureImportsView) -> fret::Ui",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-texture-imports-root\"),",
            ],
            &["fn external_texture_imports_root("],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_TEXTURE_IMPORTS_WEB_DEMO,
            &[
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-texture-imports-web-root\"),",
                "make_panel(cx, fret_core::ViewportFit::Contain, \"ext-tex-web-contain\")",
            ],
            &["fn external_texture_imports_web_root("],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_VIDEO_IMPORTS_AVF_DEMO,
            &[
                "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsAvfView) -> fret::Ui",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-video-imports-avf-root\"),",
            ],
            &["fn external_video_imports_avf_root("],
        );

        assert_low_level_interop_examples_keep_direct_leaf_roots(
            EXTERNAL_VIDEO_IMPORTS_MF_DEMO,
            &[
                "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsMfView) -> fret::Ui",
                "cx.viewport_surface_props(ViewportSurfaceProps {",
                ".test_id(\"external-video-imports-mf-root\"),",
            ],
            &["fn external_video_imports_mf_root("],
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
            SIMPLE_TODO_DEMO,
            &[
                "fn simple_todo_page<C>(",
                "cx: &mut fret_ui::ElementContext<'_, App>,",
                "theme: fret_ui::ThemeSnapshot,",
                "card: C,",
                ") -> impl fret_ui_kit::IntoUiElement<App> + use<C>",
                "C: fret_ui_kit::IntoUiElement<App>,",
                "ui::children![cx; simple_todo_page(cx, theme, card)]",
            ],
            &["let page = ui::container(|cx| {"],
        );

        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            CJK_CONFORMANCE_DEMO,
            &[
                "fn cjk_conformance_page<C>(",
                "cx: &mut fret_ui::ElementContext<'_, App>,",
                "theme: fret_ui::ThemeSnapshot,",
                "card: C,",
                ") -> impl fret_ui_kit::IntoUiElement<App> + use<C>",
                "C: fret_ui_kit::IntoUiElement<App>,",
                "ui::children![cx; cjk_conformance_page(cx, theme, card)]",
                "ui::v_flex(move |cx| ui::single(cx, card))",
            ],
            &[
                "let page = ui::container(|cx| {",
                "ui::v_flex(move |_cx| [card])",
            ],
        );

        assert_manual_ui_tree_helpers_prefer_typed_root_helpers(
            EMOJI_CONFORMANCE_DEMO,
            &[
                "fn emoji_conformance_page<C>(",
                "cx: &mut fret_ui::ElementContext<'_, App>,",
                "theme: fret_ui::ThemeSnapshot,",
                "card: C,",
                ") -> impl fret_ui_kit::IntoUiElement<App> + use<C>",
                "C: fret_ui_kit::IntoUiElement<App>,",
                "ui::children![cx; emoji_conformance_page(cx, theme, card)]",
                "ui::v_flex(move |cx| ui::single(cx, card))",
            ],
            &[
                "let page = ui::container(|cx| {",
                "ui::v_flex(move |_cx| [card])",
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
    fn app_facing_docking_examples_prefer_fret_docking_facade() {
        for src in [CONTAINER_QUERIES_DOCKING_DEMO, DOCKING_DEMO] {
            assert!(src.contains("use fret::docking::{"));
            assert!(!src.contains("use fret_docking::{"));
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
            ],
            &[],
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
            QUERY_DEMO,
            &[
                "fn query_page<C>(cx: &mut UiCx<'_>, theme: ThemeSnapshot, card: C) -> Ui",
                "C: IntoUiElement<KernelApp>,",
                "query_page(cx.elements(), theme, card)",
            ],
            &[],
        );

        assert_advanced_helpers_prefer_uicx(
            QUERY_ASYNC_TOKIO_DEMO,
            &[
                "fn query_page<C>(cx: &mut UiCx<'_>, theme: ThemeSnapshot, card: C) -> Ui",
                "C: IntoUiElement<KernelApp>,",
                "query_page(cx.elements(), theme, card)",
            ],
            &[],
        );

        assert_advanced_helpers_prefer_uicx(
            HELLO_WORLD_COMPARE_DEMO,
            &[
                "let swatch = |_cx: &mut UiCx<'_>, fill_rgb: u32, border_rgb: u32|",
                "fn hello_world_compare_root(",
                "cx: &mut UiCx<'_>,",
                "panel_bg: Color,",
                "children: Vec<AnyElement>,",
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
                "fn active_mode(cx: &mut UiCx<'_>, st: &AsyncPlaygroundState) -> FetchMode",
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
                "fn active_mode(cx: &mut ElementContext<'_, KernelApp>, st: &AsyncPlaygroundState) -> FetchMode",
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
                "fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls,) -> impl IntoUiElement<App> + use<>",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls,) -> impl IntoUiElement<App> + use<>",
                "items.push(Self::stage_tile(",
                ".into_element(cx),",
                "let inspector = Self::inspector(cx, &controls).into_element(cx);",
                "Self::lens(cx, &controls_for_lens).into_element(cx)",
            ],
            &[
                "fn stage_tile(cx: &mut ElementContext<'_, App>, color: fret_core::Color, left: Px, top: Px, w: Px, h: Px, corner_radius_px: Px,) -> AnyElement",
                "fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement",
            ],
        );

        assert_default_app_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V2_WEB_DEMO,
            &[
                "fn stage_tile(",
                ") -> impl IntoUiElement<App> + use<>",
                "fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls,) -> impl IntoUiElement<App> + use<>",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls,) -> impl IntoUiElement<App> + use<>",
                "items.push(Self::stage_tile(",
                ".into_element(cx),",
                "let inspector = Self::inspector(cx, &controls).into_element(cx);",
                "Self::lens(cx, &controls_for_lens).into_element(cx)",
            ],
            &[
                "fn stage_tile(cx: &mut ElementContext<'_, App>, color: fret_core::Color, left: Px, top: Px, w: Px, h: Px, corner_radius_px: Px,) -> AnyElement",
                "fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement",
            ],
        );

        assert_default_app_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V2_LUT_WEB_DEMO,
            &[
                "fn stage_tile(",
                ") -> impl IntoUiElement<App> + use<>",
                "fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls,) -> impl IntoUiElement<App> + use<>",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls,) -> impl IntoUiElement<App> + use<>",
                "items.push(Self::stage_tile(",
                ".into_element(cx),",
                "let inspector = Self::inspector(cx, &controls).into_element(cx);",
                "Self::lens(cx, &controls_for_lens).into_element(cx)",
            ],
            &[
                "fn stage_tile(cx: &mut ElementContext<'_, App>, color: fret_core::Color, left: Px, top: Px, w: Px, h: Px, corner_radius_px: Px,) -> AnyElement",
                "fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement",
                "fn inspector(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement",
            ],
        );

        assert_default_app_generic_helpers_prefer_into_ui_element(
            CUSTOM_EFFECT_V2_GLASS_CHROME_WEB_DEMO,
            &[
                "fn label_row(cx: &mut ElementContext<'_, App>, label: &str, value: String,) -> impl IntoUiElement<App> + use<>",
                "fn stage_tile(",
                ") -> impl IntoUiElement<App> + use<>",
                "fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls,) -> impl IntoUiElement<App> + use<>",
                "fn controls_panel(cx: &mut ElementContext<'_, App>, controls: &DemoControls,) -> impl IntoUiElement<App> + use<>",
                "Self::label_row(cx, \"Uv span\", format!(\"{uv_span:.2}\")).into_element(cx)",
                "items.push(Self::stage_tile(",
                ".into_element(cx),",
                "let inspector = Self::controls_panel(cx, &controls).into_element(cx);",
                "items.push(Self::lens(cx, &controls_for_stage).into_element(cx));",
            ],
            &[
                "fn label_row(cx: &mut ElementContext<'_, App>, label: &str, value: String) -> AnyElement",
                "fn stage_tile(cx: &mut ElementContext<'_, App>, color: fret_core::Color, left: Px, top: Px, w: Px, h: Px, corner_radius_px: Px,) -> AnyElement",
                "fn lens(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement",
                "fn controls_panel(cx: &mut ElementContext<'_, App>, controls: &DemoControls) -> AnyElement",
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
                "cx.actions().locals::<act::Inc>({",
                "cx.actions().locals::<act::Dec>({",
                "cx.actions().local_set::<act::Reset, i64>(&count_state, 0);",
            ],
            &[
                "cx.use_local_with(|| 0i64)",
                "cx.on_action_notify_models::<act::Inc>",
                "cx.on_action_notify_local_set::<act::Reset, i64>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            QUERY_DEMO,
            &[
                "let fail_mode_state = cx.state().local_init(|| false);",
                "if cx.effects().take_transient(TRANSIENT_INVALIDATE_KEY)",
                "cx.actions().toggle_local_bool::<act::ToggleFailMode>(&fail_mode_state);",
                "cx.actions().transient::<act::Invalidate>(TRANSIENT_INVALIDATE_KEY);",
            ],
            &[
                "cx.use_local_with(|| false)",
                "cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_KEY)",
                "cx.on_action_notify_toggle_local_bool::<act::ToggleFailMode>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            QUERY_ASYNC_TOKIO_DEMO,
            &[
                "let fail_mode_state = cx.state().local_init(|| false);",
                "if cx.effects().take_transient(TRANSIENT_INVALIDATE_KEY)",
                "cx.actions().toggle_local_bool::<act::ToggleFailMode>(&fail_mode_state);",
                "cx.actions().transient::<act::Invalidate>(TRANSIENT_INVALIDATE_KEY);",
            ],
            &[
                "cx.use_local_with(|| false)",
                "cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_KEY)",
                "cx.on_action_notify_toggle_local_bool::<act::ToggleFailMode>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            TODO_DEMO,
            &[
                "let draft_state = cx.state().local::<String>();",
                "let next_id_state = cx.state().local_init(|| 4u64);",
                "let todos = todos_state.layout(cx).value_or_default();",
                "let draft_value = draft_state.layout(cx).value_or_default();",
                "cx.actions().locals::<act::Add>({",
                "cx.actions().locals::<act::ClearDone>({",
                "cx.actions().payload_local_update_if::<act::Toggle, Vec<TodoRow>>(",
                "cx.actions().payload_local_update_if::<act::Remove, Vec<TodoRow>>(",
            ],
            &[
                "cx.use_local::<String>()",
                "cx.on_action_notify_models::<act::Add>",
                "cx.on_payload_action_notify_local_update_if::<act::Toggle, Vec<TodoRow>>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            EMBEDDED_VIEWPORT_DEMO,
            &[
                "let size_preset_state = cx.state().local_init(|| 1usize);",
                "let preset = size_preset_state.layout(cx).value_or_default();",
                "cx.actions().local_set::<act::PickSize640, usize>(&size_preset_state, 0);",
            ],
            &[
                "cx.use_local_with(|| 1usize)",
                "cx.on_action_notify_local_set::<act::PickSize640, usize>",
            ],
        );
    }

    #[test]
    fn selected_advanced_examples_prefer_grouped_state_actions_and_effects() {
        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_HELLO_DEMO,
            &[
                "let count_state = cx.state().local_init(|| 0u32);",
                "let enabled_state = cx.state().local_init(|| false);",
                "let count = count_state.layout(cx).value_or_default();",
                "let enabled = enabled_state.paint(cx).value_or_default();",
            ],
            &["cx.use_local_with(|| 0u32)", "cx.use_local_with(|| false)"],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            IMUI_RESPONSE_SIGNALS_DEMO,
            &[
                "let left_clicks = cx.state().local_init(|| 0u32);",
                "let drag_offset = cx.state().local_init(Point::default);",
                "let last_anchor_value = last_context_menu_anchor.layout(cx).value_or_default();",
            ],
            &[
                "cx.use_local_with(|| 0u32)",
                "cx.use_local_with(Point::default)",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            ASYNC_PLAYGROUND_DEMO,
            &[
                "if cx.effects().take_transient(TRANSIENT_INVALIDATE_SELECTED)",
                "cx.actions().models::<act::SelectTip>({",
                "cx.actions().transient::<act::InvalidateSelected>(TRANSIENT_INVALIDATE_SELECTED);",
            ],
            &[
                "cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_SELECTED)",
                "cx.on_action_notify_models::<act::SelectTip>",
                "cx.on_action_notify_transient::<act::InvalidateSelected>",
            ],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V1_DEMO,
            &["cx.actions().models::<act::Reset>({"],
            &["cx.on_action_notify_models::<act::Reset>"],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V2_DEMO,
            &["cx.actions().models::<act::Reset>({"],
            &["cx.on_action_notify_models::<act::Reset>"],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            CUSTOM_EFFECT_V3_DEMO,
            &["cx.actions().models::<act::Reset>({"],
            &["cx.on_action_notify_models::<act::Reset>"],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            POSTPROCESS_THEME_DEMO,
            &["cx.actions().models::<act::Reset>({"],
            &["cx.on_action_notify_models::<act::Reset>"],
        );

        assert_selected_view_runtime_examples_prefer_grouped_helpers(
            LIQUID_GLASS_DEMO,
            &[
                "cx.actions().models::<act::Reset>({",
                "cx.actions().models::<act::ToggleInspector>({",
            ],
            &[
                "cx.on_action_notify_models::<act::Reset>",
                "cx.on_action_notify_models::<act::ToggleInspector>",
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
                "cx.actions().payload::<act::ToggleCodeBlockExpand>().models({",
            ],
            &[
                "cx.take_transient_on_action_root(TRANSIENT_REFRESH_REMOTE_IMAGES)",
                "cx.on_action_notify_transient::<act::RefreshRemoteImages>",
                "cx.on_payload_action_notify::<act::ToggleCodeBlockExpand>",
            ],
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
