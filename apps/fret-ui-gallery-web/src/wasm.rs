use wasm_bindgen::prelude::*;

thread_local! {
    static RUNNER_HANDLE: std::cell::RefCell<Option<fret_launch::WebRunnerHandle>> =
        const { std::cell::RefCell::new(None) };
    static DESTROY_HOOK: std::cell::RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut()>>> =
        const { std::cell::RefCell::new(None) };
}

fn run_web_app<D: fret_launch::WinitAppDriver + 'static>(
    config: fret_launch::WinitRunnerConfig,
    mut app: fret_app::App,
    driver: D,
) -> Result<fret_launch::WebRunnerHandle, fret_launch::RunnerError> {
    fret_bootstrap::install_default_text_interaction_settings(&mut app);
    fret_launch::run_app_with_handle(config, app, driver)
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let app = fret_ui_gallery::build_app();
    let mut config = fret_ui_gallery::build_runner_config();
    config.main_window_title = "fret-ui-gallery (web)".to_string();
    let driver = fret_ui_gallery::build_driver();

    let handle = run_web_app(config, app, driver).map_err(|e| JsValue::from_str(&e.to_string()))?;

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
        crate::fret_ui_gallery_destroy();
    }) as Box<dyn FnMut()>);

    let _ = js_sys::Reflect::set(
        window.as_ref(),
        &JsValue::from_str("fret_ui_gallery_destroy"),
        hook.as_ref().unchecked_ref(),
    );

    DESTROY_HOOK.with(|slot| {
        slot.borrow_mut().replace(hook);
    });
}

/// Debug-only teardown hook for restarting the UI gallery in a single page session.
///
/// On web targets, we cannot reliably close a browser tab/window, so "close" is implemented as
/// stopping the runner instance.
#[wasm_bindgen]
pub fn fret_ui_gallery_destroy() {
    RUNNER_HANDLE.with(|slot| {
        if let Some(handle) = slot.borrow_mut().take() {
            handle.destroy();
        }
    });
}
