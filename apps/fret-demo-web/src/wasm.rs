use wasm_bindgen::prelude::*;

thread_local! {
    static RUNNER_HANDLE: std::cell::RefCell<Option<fret_launch::WebRunnerHandle>> =
        const { std::cell::RefCell::new(None) };
    static DESTROY_HOOK: std::cell::RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut()>>> =
        const { std::cell::RefCell::new(None) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Demo {
    ComponentsGallery,
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
    ShadedDemo,
    StairsDemo,
    StemsDemo,
    LinkedCursorDemo,
    InfLinesDemo,
    TagsDemo,
    DragDemo,
}

fn select_demo() -> Demo {
    let Some(window) = web_sys::window() else {
        return Demo::ComponentsGallery;
    };

    let location = window.location();

    let hash = location.hash().unwrap_or_default();
    let search = location.search().unwrap_or_default();

    if hash.contains("plot_demo") || search.contains("demo=plot_demo") {
        return Demo::PlotDemo;
    }
    if hash.contains("plot_image_demo") || search.contains("demo=plot_image_demo") {
        return Demo::PlotImageDemo;
    }
    if hash.contains("bars_demo") || search.contains("demo=bars_demo") {
        return Demo::BarsDemo;
    }
    if hash.contains("grouped_bars_demo") || search.contains("demo=grouped_bars_demo") {
        return Demo::GroupedBarsDemo;
    }
    if hash.contains("stacked_bars_demo") || search.contains("demo=stacked_bars_demo") {
        return Demo::StackedBarsDemo;
    }
    if hash.contains("area_demo") || search.contains("demo=area_demo") {
        return Demo::AreaDemo;
    }
    if hash.contains("candlestick_demo") || search.contains("demo=candlestick_demo") {
        return Demo::CandlestickDemo;
    }
    if hash.contains("error_bars_demo") || search.contains("demo=error_bars_demo") {
        return Demo::ErrorBarsDemo;
    }
    if hash.contains("heatmap_demo") || search.contains("demo=heatmap_demo") {
        return Demo::HeatmapDemo;
    }
    if hash.contains("histogram_demo") || search.contains("demo=histogram_demo") {
        return Demo::HistogramDemo;
    }
    if hash.contains("shaded_demo") || search.contains("demo=shaded_demo") {
        return Demo::ShadedDemo;
    }
    if hash.contains("stairs_demo") || search.contains("demo=stairs_demo") {
        return Demo::StairsDemo;
    }
    if hash.contains("stems_demo") || search.contains("demo=stems_demo") {
        return Demo::StemsDemo;
    }
    if hash.contains("linked_cursor_demo") || search.contains("demo=linked_cursor_demo") {
        return Demo::LinkedCursorDemo;
    }
    if hash.contains("inf_lines_demo") || search.contains("demo=inf_lines_demo") {
        return Demo::InfLinesDemo;
    }
    if hash.contains("tags_demo") || search.contains("demo=tags_demo") {
        return Demo::TagsDemo;
    }
    if hash.contains("drag_demo") || search.contains("demo=drag_demo") {
        return Demo::DragDemo;
    }

    Demo::ComponentsGallery
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let demo = select_demo();

    let handle = match demo {
        Demo::ComponentsGallery => {
            let app = fret_examples::components_gallery::build_app();
            let mut config = fret_examples::components_gallery::build_runner_config();
            config.main_window_title = "fret-demo components_gallery (web)".to_string();
            let driver = fret_examples::components_gallery::build_driver();
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
