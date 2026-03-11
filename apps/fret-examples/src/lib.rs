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
    fret::run_native_with_compat_driver(config, app, driver).map_err(anyhow::Error::from)
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
    fret::run_native_with_fn_driver_with_hooks(
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
    const CUSTOM_EFFECT_V3_DEMO: &str = include_str!("custom_effect_v3_demo.rs");
    const DOCKING_ARBITRATION_DEMO: &str = include_str!("docking_arbitration_demo.rs");
    const DOCKING_DEMO: &str = include_str!("docking_demo.rs");
    const DROP_SHADOW_DEMO: &str = include_str!("drop_shadow_demo.rs");
    const ECHARTS_DEMO: &str = include_str!("echarts_demo.rs");
    const EMBEDDED_VIEWPORT_DEMO: &str = include_str!("embedded_viewport_demo.rs");
    const EMPTY_IDLE_DEMO: &str = include_str!("empty_idle_demo.rs");
    const EMOJI_CONFORMANCE_DEMO: &str = include_str!("emoji_conformance_demo.rs");
    const EXTERNAL_TEXTURE_IMPORTS_DEMO: &str = include_str!("external_texture_imports_demo.rs");
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

    fn assert_prefers_grouped_data_surface(src: &str) {
        assert!(
            src.contains("cx.data().selector(")
                || src.contains("cx.data().query(")
                || src.contains("cx.data().query_async(")
                || src.contains("cx.data().query_async_local(")
        );
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
            HELLO_COUNTER_DEMO,
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
            TODO_DEMO,
            WINDOW_HIT_TEST_PROBE_DEMO,
        ] {
            assert_uses_advanced_surface(src);
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
        for src in [QUERY_ASYNC_TOKIO_DEMO, QUERY_DEMO, TODO_DEMO] {
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
    fn advanced_helper_contexts_prefer_uicx_aliases() {
        assert_advanced_helpers_prefer_uicx(
            CUSTOM_EFFECT_V1_DEMO,
            &[
                "fn watch_first_f32(cx: &mut UiCx<'_>,",
                "fn stage(cx: &mut UiCx<'_>,",
                "fn lens_row(cx: &mut UiCx<'_>,",
                "fn inspector(cx: &mut UiCx<'_>,",
                "let label_row = |cx: &mut UiCx<'_>, label: &str, value: String|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_row(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut ElementContext<'_, KernelApp>,",
                "let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            CUSTOM_EFFECT_V2_DEMO,
            &[
                "fn watch_first_f32(cx: &mut UiCx<'_>,",
                "fn stage(cx: &mut UiCx<'_>,",
                "fn lens_row(cx: &mut UiCx<'_>,",
                "fn inspector(cx: &mut UiCx<'_>,",
                "let label_row = |cx: &mut UiCx<'_>, label: &str, value: String|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_row(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut ElementContext<'_, KernelApp>,",
                "let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            CUSTOM_EFFECT_V3_DEMO,
            &[
                "fn stage(cx: &mut UiCx<'_>,",
                "fn stage_controls(cx: &mut UiCx<'_>,",
                "fn animated_backdrop(cx: &mut UiCx<'_>) -> AnyElement",
                "fn lens_row(cx: &mut UiCx<'_>,",
                "fn lens_shell(cx: &mut UiCx<'_>,",
                "fn custom_effect_user01_probe_lens(cx: &mut UiCx<'_>,",
            ],
            &[
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage_controls(cx: &mut ElementContext<'_, KernelApp>,",
                "fn animated_backdrop(cx: &mut ElementContext<'_, KernelApp>) -> AnyElement",
                "fn lens_row(cx: &mut ElementContext<'_, KernelApp>,",
                "fn lens_shell(cx: &mut ElementContext<'_, KernelApp>,",
                "fn custom_effect_user01_probe_lens(cx: &mut ElementContext<'_, KernelApp>,",
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
                "fn inspector(cx: &mut UiCx<'_>,",
                "let label_row = |cx: &mut UiCx<'_>, label: &str, value: String|",
                "fn stage(cx: &mut UiCx<'_>,",
                "fn stage_body(cx: &mut UiCx<'_>,",
                "fn stage_cards(cx: &mut UiCx<'_>) -> AnyElement",
                "let card = |cx: &mut UiCx<'_>, title: &str, subtitle: &str|",
            ],
            &[
                "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
                "fn inspector(cx: &mut ElementContext<'_, KernelApp>,",
                "let label_row = |cx: &mut ElementContext<'_, KernelApp>, label: &str, value: String|",
                "fn stage(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage_body(cx: &mut ElementContext<'_, KernelApp>,",
                "fn stage_cards(cx: &mut ElementContext<'_, KernelApp>) -> AnyElement",
                "let card = |cx: &mut ElementContext<'_, KernelApp>, title: &str, subtitle: &str|",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            ASYNC_PLAYGROUND_DEMO,
            &[
                "fn header_bar(cx: &mut UiCx<'_>,",
                "fn body(cx: &mut UiCx<'_>,",
                "fn catalog_panel(cx: &mut UiCx<'_>,",
                "fn main_panel(cx: &mut UiCx<'_>,",
                "fn query_panel_for_mode(cx: &mut UiCx<'_>,",
                "fn active_mode(cx: &mut UiCx<'_>, st: &AsyncPlaygroundState) -> FetchMode",
                "fn status_badge(cx: &mut UiCx<'_>, diag: Option<&QueryDiag>) -> AnyElement",
                "fn snapshot_entry_for_key(cx: &mut UiCx<'_>,",
            ],
            &[
                "fn header_bar(cx: &mut ElementContext<'_, KernelApp>,",
                "fn body(cx: &mut ElementContext<'_, KernelApp>,",
                "fn catalog_panel(cx: &mut ElementContext<'_, KernelApp>,",
                "fn main_panel(cx: &mut ElementContext<'_, KernelApp>,",
                "fn query_panel_for_mode(cx: &mut ElementContext<'_, KernelApp>,",
                "fn active_mode(cx: &mut ElementContext<'_, KernelApp>, st: &AsyncPlaygroundState) -> FetchMode",
                "fn status_badge(cx: &mut ElementContext<'_, KernelApp>, diag: Option<&QueryDiag>) -> AnyElement",
                "fn snapshot_entry_for_key(cx: &mut ElementContext<'_, KernelApp>,",
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
}
