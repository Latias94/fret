#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let demo = args
        .next()
        .unwrap_or_else(|| "components_gallery".to_string());

    if demo == "--list" || demo == "-l" {
        eprintln!(
            "Available demos:\n  simple-todo\n  todo_demo\n  components_gallery\n  genui_demo\n  emoji_conformance_demo\n  cjk_conformance_demo\n  chart_demo\n  chart_declarative_demo\n  chart_multi_axis_demo\n  echarts_demo\n  category_line_demo\n  horizontal_bars_demo\n  plot_demo\n  plot_image_demo\n  bars_demo\n  grouped_bars_demo\n  stacked_bars_demo\n  area_demo\n  candlestick_demo\n  error_bars_demo\n  heatmap_demo\n  histogram_demo\n  histogram2d_demo\n  shaded_demo\n  stairs_demo\n  stems_demo\n  linked_cursor_demo\n  inf_lines_demo\n  tags_demo\n  drag_demo\n  effects_demo"
        );
        return Ok(());
    }

    match demo.as_str() {
        "simple-todo" | "simple_todo" | "simple_todo_demo" => {
            fret_examples::simple_todo_demo::run()
        }
        "todo_demo" | "todo-demo" => fret_examples::todo_demo::run(),
        "components_gallery" => fret_examples::components_gallery::run(),
        "genui_demo" => fret_examples::genui_demo::run(),
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
        other => {
            eprintln!(
                "Unknown demo: {other}\nRun `cargo run -p fret-demo -- --list` to see available demos."
            );
            fret_examples::components_gallery::run()
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
