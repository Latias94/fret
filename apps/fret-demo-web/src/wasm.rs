use wasm_bindgen::prelude::*;

thread_local! {
    static RUNNER_HANDLE: std::cell::RefCell<Option<fret_launch::WebRunnerHandle>> =
        const { std::cell::RefCell::new(None) };
    static DESTROY_HOOK: std::cell::RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut()>>> =
        const { std::cell::RefCell::new(None) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Demo {
    UiGallery,
    ComponentsGallery,
    EmojiConformanceDemo,
    CjkConformanceDemo,
    ChartDemo,
    ChartMultiAxisDemo,
    HorizontalBarsDemo,
    PlotDemo,
    PlotImageDemo,
    BarsDemo,
    GroupedBarsDemo,
    StackedBarsDemo,
    AreaDemo,
    CandlestickDemo,
    ErrorBarsDemo,
    HeatmapDemo,
    HistogramDemo,
    Histogram2DDemo,
    ShadedDemo,
    StairsDemo,
    StemsDemo,
    LinkedCursorDemo,
    InfLinesDemo,
    TagsDemo,
    DragDemo,
}

fn demo_from_id(id: &str) -> Option<Demo> {
    match id {
        "ui_gallery" => Some(Demo::UiGallery),
        "components_gallery" => Some(Demo::ComponentsGallery),
        "emoji_conformance_demo" => Some(Demo::EmojiConformanceDemo),
        "cjk_conformance_demo" => Some(Demo::CjkConformanceDemo),
        "chart_demo" => Some(Demo::ChartDemo),
        "chart_multi_axis_demo" => Some(Demo::ChartMultiAxisDemo),
        "horizontal_bars_demo" => Some(Demo::HorizontalBarsDemo),
        "plot_demo" => Some(Demo::PlotDemo),
        "plot_image_demo" => Some(Demo::PlotImageDemo),
        "bars_demo" => Some(Demo::BarsDemo),
        "grouped_bars_demo" => Some(Demo::GroupedBarsDemo),
        "stacked_bars_demo" => Some(Demo::StackedBarsDemo),
        "area_demo" => Some(Demo::AreaDemo),
        "candlestick_demo" => Some(Demo::CandlestickDemo),
        "error_bars_demo" => Some(Demo::ErrorBarsDemo),
        "heatmap_demo" => Some(Demo::HeatmapDemo),
        "histogram_demo" => Some(Demo::HistogramDemo),
        "histogram2d_demo" => Some(Demo::Histogram2DDemo),
        "shaded_demo" => Some(Demo::ShadedDemo),
        "stairs_demo" => Some(Demo::StairsDemo),
        "stems_demo" => Some(Demo::StemsDemo),
        "linked_cursor_demo" => Some(Demo::LinkedCursorDemo),
        "inf_lines_demo" => Some(Demo::InfLinesDemo),
        "tags_demo" => Some(Demo::TagsDemo),
        "drag_demo" => Some(Demo::DragDemo),
        _ => None,
    }
}

fn select_demo() -> Demo {
    let Some(location) = fret_router::web::current_location() else {
        return Demo::ComponentsGallery;
    };

    let hash = location.hash;
    let search = location.search;

    if let Some(demo) = fret_router::first_query_value_from_search_or_hash(&search, &hash, "demo")
        .and_then(|id| demo_from_id(id.as_str()))
    {
        return demo;
    }

    if let Some(demo) = fret_router::hash_token(&hash).and_then(|id| demo_from_id(id.as_str())) {
        return demo;
    }

    // Keep compatibility with older hash forms that matched by substring.
    for id in [
        "ui_gallery",
        "chart_demo",
        "chart_multi_axis_demo",
        "horizontal_bars_demo",
        "plot_demo",
        "plot_image_demo",
        "bars_demo",
        "grouped_bars_demo",
        "stacked_bars_demo",
        "area_demo",
        "candlestick_demo",
        "error_bars_demo",
        "heatmap_demo",
        "histogram_demo",
        "histogram2d_demo",
        "shaded_demo",
        "stairs_demo",
        "stems_demo",
        "linked_cursor_demo",
        "inf_lines_demo",
        "tags_demo",
        "drag_demo",
        "emoji_conformance_demo",
        "cjk_conformance_demo",
        "components_gallery",
    ] {
        if fret_router::hash_contains_token(&hash, id) {
            if let Some(demo) = demo_from_id(id) {
                return demo;
            }
        }
    }

    Demo::ComponentsGallery
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let demo = select_demo();

    let handle = match demo {
        Demo::UiGallery => {
            let app = fret_ui_gallery::build_app();
            let mut config = fret_ui_gallery::build_runner_config();
            config.main_window_title = "fret-ui-gallery (web)".to_string();
            let driver = fret_ui_gallery::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::ComponentsGallery => {
            let app = fret_examples::components_gallery::build_app();
            let mut config = fret_examples::components_gallery::build_runner_config();
            config.main_window_title = "fret-demo components_gallery (web)".to_string();
            let driver = fret_examples::components_gallery::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::EmojiConformanceDemo => {
            let app = fret_examples::emoji_conformance_demo::build_app();
            let mut config = fret_examples::emoji_conformance_demo::build_runner_config();
            config.main_window_title = "fret-demo emoji_conformance_demo (web)".to_string();
            let driver = fret_examples::emoji_conformance_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::CjkConformanceDemo => {
            let app = fret_examples::cjk_conformance_demo::build_app();
            let mut config = fret_examples::cjk_conformance_demo::build_runner_config();
            config.main_window_title = "fret-demo cjk_conformance_demo (web)".to_string();
            let driver = fret_examples::cjk_conformance_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::ChartDemo => {
            let app = fret_examples::chart_demo::build_app();
            let mut config = fret_examples::chart_demo::build_runner_config();
            config.main_window_title = "fret-demo chart_demo (web)".to_string();
            let driver = fret_examples::chart_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::ChartMultiAxisDemo => {
            let app = fret_examples::chart_multi_axis_demo::build_app();
            let mut config = fret_examples::chart_multi_axis_demo::build_runner_config();
            config.main_window_title = "fret-demo chart_multi_axis_demo (web)".to_string();
            let driver = fret_examples::chart_multi_axis_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::HorizontalBarsDemo => {
            let app = fret_examples::horizontal_bars_demo::build_app();
            let mut config = fret_examples::horizontal_bars_demo::build_runner_config();
            config.main_window_title = "fret-demo horizontal_bars_demo (web)".to_string();
            let driver = fret_examples::horizontal_bars_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::PlotDemo => {
            let app = fret_examples::plot_demo::build_app();
            let mut config = fret_examples::plot_demo::build_runner_config();
            config.main_window_title = "fret-demo plot_demo (web)".to_string();
            let driver = fret_examples::plot_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::PlotImageDemo => {
            let app = fret_examples::plot_image_demo::build_app();
            let mut config = fret_examples::plot_image_demo::build_runner_config();
            config.main_window_title = "fret-demo plot_image_demo (web)".to_string();
            let driver = fret_examples::plot_image_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::BarsDemo => {
            let app = fret_examples::bars_demo::build_app();
            let mut config = fret_examples::bars_demo::build_runner_config();
            config.main_window_title = "fret-demo bars_demo (web)".to_string();
            let driver = fret_examples::bars_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::GroupedBarsDemo => {
            let app = fret_examples::grouped_bars_demo::build_app();
            let mut config = fret_examples::grouped_bars_demo::build_runner_config();
            config.main_window_title = "fret-demo grouped_bars_demo (web)".to_string();
            let driver = fret_examples::grouped_bars_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::StackedBarsDemo => {
            let app = fret_examples::stacked_bars_demo::build_app();
            let mut config = fret_examples::stacked_bars_demo::build_runner_config();
            config.main_window_title = "fret-demo stacked_bars_demo (web)".to_string();
            let driver = fret_examples::stacked_bars_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::AreaDemo => {
            let app = fret_examples::area_demo::build_app();
            let mut config = fret_examples::area_demo::build_runner_config();
            config.main_window_title = "fret-demo area_demo (web)".to_string();
            let driver = fret_examples::area_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::CandlestickDemo => {
            let app = fret_examples::candlestick_demo::build_app();
            let mut config = fret_examples::candlestick_demo::build_runner_config();
            config.main_window_title = "fret-demo candlestick_demo (web)".to_string();
            let driver = fret_examples::candlestick_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::ErrorBarsDemo => {
            let app = fret_examples::error_bars_demo::build_app();
            let mut config = fret_examples::error_bars_demo::build_runner_config();
            config.main_window_title = "fret-demo error_bars_demo (web)".to_string();
            let driver = fret_examples::error_bars_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::HeatmapDemo => {
            let app = fret_examples::heatmap_demo::build_app();
            let mut config = fret_examples::heatmap_demo::build_runner_config();
            config.main_window_title = "fret-demo heatmap_demo (web)".to_string();
            let driver = fret_examples::heatmap_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::HistogramDemo => {
            let app = fret_examples::histogram_demo::build_app();
            let mut config = fret_examples::histogram_demo::build_runner_config();
            config.main_window_title = "fret-demo histogram_demo (web)".to_string();
            let driver = fret_examples::histogram_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::Histogram2DDemo => {
            let app = fret_examples::histogram2d_demo::build_app();
            let mut config = fret_examples::histogram2d_demo::build_runner_config();
            config.main_window_title = "fret-demo histogram2d_demo (web)".to_string();
            let driver = fret_examples::histogram2d_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::ShadedDemo => {
            let app = fret_examples::shaded_demo::build_app();
            let mut config = fret_examples::shaded_demo::build_runner_config();
            config.main_window_title = "fret-demo shaded_demo (web)".to_string();
            let driver = fret_examples::shaded_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::StairsDemo => {
            let app = fret_examples::stairs_demo::build_app();
            let mut config = fret_examples::stairs_demo::build_runner_config();
            config.main_window_title = "fret-demo stairs_demo (web)".to_string();
            let driver = fret_examples::stairs_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::StemsDemo => {
            let app = fret_examples::stems_demo::build_app();
            let mut config = fret_examples::stems_demo::build_runner_config();
            config.main_window_title = "fret-demo stems_demo (web)".to_string();
            let driver = fret_examples::stems_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::LinkedCursorDemo => {
            let app = fret_examples::linked_cursor_demo::build_app();
            let mut config = fret_examples::linked_cursor_demo::build_runner_config();
            config.main_window_title = "fret-demo linked_cursor_demo (web)".to_string();
            let driver = fret_examples::linked_cursor_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::InfLinesDemo => {
            let app = fret_examples::inf_lines_demo::build_app();
            let mut config = fret_examples::inf_lines_demo::build_runner_config();
            config.main_window_title = "fret-demo inf_lines_demo (web)".to_string();
            let driver = fret_examples::inf_lines_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::TagsDemo => {
            let app = fret_examples::tags_demo::build_app();
            let mut config = fret_examples::tags_demo::build_runner_config();
            config.main_window_title = "fret-demo tags_demo (web)".to_string();
            let driver = fret_examples::tags_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
        Demo::DragDemo => {
            let app = fret_examples::drag_demo::build_app();
            let mut config = fret_examples::drag_demo::build_runner_config();
            config.main_window_title = "fret-demo drag_demo (web)".to_string();
            let driver = fret_examples::drag_demo::build_driver();
            fret_launch::run_app_with_handle(config, app, driver)
        }
    }
    .map_err(|e| JsValue::from_str(&e.to_string()))?;

    RUNNER_HANDLE.with(|slot| {
        slot.borrow_mut().replace(handle);
    });

    install_global_debug_destroy_hook();
    Ok(())
}

fn install_global_debug_destroy_hook() {
    use wasm_bindgen::JsCast as _;

    let Some(window) = web_sys::window() else {
        return;
    };

    let hook = wasm_bindgen::closure::Closure::wrap(Box::new(|| {
        crate::fret_demo_destroy();
    }) as Box<dyn FnMut()>);

    let _ = js_sys::Reflect::set(
        window.as_ref(),
        &JsValue::from_str("fret_demo_destroy"),
        hook.as_ref().unchecked_ref(),
    );

    DESTROY_HOOK.with(|slot| {
        slot.borrow_mut().replace(hook);
    });
}

/// Debug-only teardown hook for restarting the demo in a single page session.
///
/// On web targets, we cannot reliably close a browser tab/window, so "close" is implemented as
/// stopping the runner instance.
#[wasm_bindgen]
pub fn fret_demo_destroy() {
    RUNNER_HANDLE.with(|slot| {
        if let Some(handle) = slot.borrow_mut().take() {
            handle.destroy();
        }
    });
}
