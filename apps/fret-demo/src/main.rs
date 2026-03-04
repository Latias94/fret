#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let demo = args
        .next()
        .unwrap_or_else(|| "components_gallery".to_string());

    if demo == "--list" || demo == "-l" {
        eprintln!(
            "Available demos:\n  simple-todo\n  todo_demo\n  components_gallery\n  emoji_conformance_demo\n  cjk_conformance_demo\n  chart_demo\n  chart_declarative_demo\n  chart_multi_axis_demo\n  echarts_demo\n  category_line_demo\n  horizontal_bars_demo\n  plot_demo\n  plot_image_demo\n  bars_demo\n  grouped_bars_demo\n  stacked_bars_demo\n  area_demo\n  candlestick_demo\n  error_bars_demo\n  heatmap_demo\n  histogram_demo\n  histogram2d_demo\n  shaded_demo\n  stairs_demo\n  stems_demo\n  linked_cursor_demo\n  inf_lines_demo\n  tags_demo\n  drag_demo\n  effects_demo\n  launcher_utility_window_demo\n  launcher_utility_window_materials_demo\n  window_hit_test_probe_demo"
        );

        #[cfg(feature = "legacy-mvu-demos")]
        eprintln!(
            "\nLegacy MVU demos (feature `legacy-mvu-demos`):\n  todo_demo_legacy\n  query_demo_legacy\n  query_async_tokio_demo\n  async_playground_demo\n  genui_demo\n  hello_counter_demo\n  markdown_demo\n  embedded_viewport_demo\n  drop_shadow_demo\n  liquid_glass_demo\n  custom_effect_v1_demo\n  custom_effect_v2_demo\n  custom_effect_v3_demo\n  postprocess_theme_demo"
        );

        #[cfg(not(feature = "legacy-mvu-demos"))]
        eprintln!(
            "\nLegacy MVU demos are disabled by default.\nRe-run with: `cargo run -p fret-demo --features legacy-mvu-demos -- --list`"
        );
        return Ok(());
    }

    match demo.as_str() {
        "simple-todo" | "simple_todo" | "simple_todo_demo" => {
            fret_examples::simple_todo_demo::run()
        }
        "todo_demo" | "todo-demo" => fret_examples::todo_demo::run(),
        "query_demo" | "query-demo" => fret_examples::query_demo::run(),
        "components_gallery" => fret_examples::components_gallery::run(),
        "emoji_conformance_demo" => fret_examples::emoji_conformance_demo::run(),
        "cjk_conformance_demo" => fret_examples::cjk_conformance_demo::run(),
        "chart_demo" => fret_examples::chart_demo::run(),
        "chart_declarative_demo" => fret_examples::chart_declarative_demo::run(),
        "chart_multi_axis_demo" => fret_examples::chart_multi_axis_demo::run(),
        "echarts_demo" => fret_examples::echarts_demo::run(),
        "category_line_demo" => fret_examples::category_line_demo::run(),
        "horizontal_bars_demo" => fret_examples::horizontal_bars_demo::run(),
        "plot_demo" => fret_examples::plot_demo::run(),
        "plot_image_demo" => fret_examples::plot_image_demo::run(),
        "bars_demo" => fret_examples::bars_demo::run(),
        "grouped_bars_demo" => fret_examples::grouped_bars_demo::run(),
        "stacked_bars_demo" => fret_examples::stacked_bars_demo::run(),
        "area_demo" => fret_examples::area_demo::run(),
        "candlestick_demo" => fret_examples::candlestick_demo::run(),
        "error_bars_demo" => fret_examples::error_bars_demo::run(),
        "heatmap_demo" => fret_examples::heatmap_demo::run(),
        "histogram_demo" => fret_examples::histogram_demo::run(),
        "histogram2d_demo" => fret_examples::histogram2d_demo::run(),
        "shaded_demo" => fret_examples::shaded_demo::run(),
        "stairs_demo" => fret_examples::stairs_demo::run(),
        "stems_demo" => fret_examples::stems_demo::run(),
        "linked_cursor_demo" => fret_examples::linked_cursor_demo::run(),
        "inf_lines_demo" => fret_examples::inf_lines_demo::run(),
        "tags_demo" => fret_examples::tags_demo::run(),
        "drag_demo" => fret_examples::drag_demo::run(),
        "effects_demo" => fret_examples::effects_demo::run(),
        "launcher_utility_window_demo" | "launcher-utility-window-demo" => {
            fret_examples::launcher_utility_window_demo::run()
        }
        "launcher_utility_window_materials_demo" | "launcher-utility-window-materials-demo" => {
            fret_examples::launcher_utility_window_materials_demo::run()
        }
        "window_hit_test_probe_demo" | "window-hit-test-probe-demo" => {
            fret_examples::window_hit_test_probe_demo::run()
        }
        other => {
            if is_legacy_mvu_demo_name(other) {
                #[cfg(feature = "legacy-mvu-demos")]
                if let Some(result) = run_legacy_mvu_demo(other) {
                    return result;
                }

                eprintln!(
                    "Demo `{other}` is a legacy MVU demo and is disabled by default.\nRe-run with: `cargo run -p fret-demo --features legacy-mvu-demos -- {other}`"
                );
                return fret_examples::components_gallery::run();
            }

            eprintln!(
                "Unknown demo: {other}\nRun `cargo run -p fret-demo -- --list` to see available demos."
            );
            fret_examples::components_gallery::run()
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "legacy-mvu-demos"))]
fn run_legacy_mvu_demo(demo: &str) -> Option<anyhow::Result<()>> {
    let result = match demo {
        "todo_demo_legacy" | "todo-demo-legacy" => fret_examples::todo_demo_legacy::run(),
        "query_demo_legacy" | "query-demo-legacy" => fret_examples::query_demo_legacy::run(),
        "query_async_tokio_demo" | "query-async-tokio-demo" => {
            fret_examples::query_async_tokio_demo::run()
        }
        "async_playground_demo" | "async-playground-demo" => {
            fret_examples::async_playground_demo::run()
        }
        "genui_demo" => fret_examples::genui_demo::run(),
        "hello_counter_demo" | "hello-counter-demo" => fret_examples::hello_counter_demo::run(),
        "query_demo" | "query-demo" => fret_examples::query_demo::run(),
        "markdown_demo" | "markdown-demo" => fret_examples::markdown_demo::run(),
        "embedded_viewport_demo" | "embedded-viewport-demo" => {
            fret_examples::embedded_viewport_demo::run()
        }
        "drop_shadow_demo" | "drop-shadow-demo" => fret_examples::drop_shadow_demo::run(),
        "liquid_glass_demo" | "liquid-glass-demo" => fret_examples::liquid_glass_demo::run(),
        "custom_effect_v1_demo" | "custom-effect-v1-demo" => {
            fret_examples::custom_effect_v1_demo::run()
        }
        "custom_effect_v2_demo" | "custom-effect-v2-demo" => {
            fret_examples::custom_effect_v2_demo::run()
        }
        "custom_effect_v3_demo" | "custom-effect-v3-demo" => {
            fret_examples::custom_effect_v3_demo::run()
        }
        "postprocess_theme_demo" | "postprocess-theme-demo" => {
            fret_examples::postprocess_theme_demo::run()
        }
        _ => return None,
    };
    Some(result)
}

#[cfg(not(target_arch = "wasm32"))]
fn is_legacy_mvu_demo_name(demo: &str) -> bool {
    matches!(
        demo,
        "todo_demo_legacy"
            | "todo-demo-legacy"
            | "query_demo_legacy"
            | "query-demo-legacy"
            | "query_async_tokio_demo"
            | "query-async-tokio-demo"
            | "async_playground_demo"
            | "async-playground-demo"
            | "genui_demo"
            | "hello_counter_demo"
            | "hello-counter-demo"
            | "markdown_demo"
            | "markdown-demo"
            | "embedded_viewport_demo"
            | "embedded-viewport-demo"
            | "drop_shadow_demo"
            | "drop-shadow-demo"
            | "liquid_glass_demo"
            | "liquid-glass-demo"
            | "custom_effect_v1_demo"
            | "custom-effect-v1-demo"
            | "custom_effect_v2_demo"
            | "custom-effect-v2-demo"
            | "custom_effect_v3_demo"
            | "custom-effect-v3-demo"
            | "postprocess_theme_demo"
            | "postprocess-theme-demo"
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}
