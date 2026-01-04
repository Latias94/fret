use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let app = fret_demo::components_gallery::build_app();
    let mut config = fret_demo::components_gallery::build_runner_config();
    config.main_window_title = "fret-demo components_gallery (web)".to_string();

    let driver = fret_demo::components_gallery::build_driver();
    fret_launch::run_app(config, app, driver).map_err(|e| JsValue::from_str(&e.to_string()))
}
