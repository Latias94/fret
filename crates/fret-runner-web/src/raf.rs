use web_sys::wasm_bindgen::JsCast as _;

pub fn request_animation_frame(mut f: impl FnMut(f64) + 'static) -> Result<i32, ()> {
    let window = web_sys::window().ok_or(())?;
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |ts: f64| {
        f(ts);
    }) as Box<dyn FnMut(f64)>);
    let id = window
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .map_err(|_| ())?;
    closure.forget();
    Ok(id)
}

pub fn cancel_animation_frame(id: i32) -> Result<(), ()> {
    let window = web_sys::window().ok_or(())?;
    window.cancel_animation_frame(id).map_err(|_| ())
}

pub fn set_timeout_ms(f: impl FnOnce() + 'static, timeout_ms: i32) -> Result<i32, ()> {
    let window = web_sys::window().ok_or(())?;
    let mut f = Some(f);
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        if let Some(f) = f.take() {
            f();
        }
    }) as Box<dyn FnMut()>);
    let id = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            timeout_ms,
        )
        .map_err(|_| ())?;
    closure.forget();
    Ok(id)
}
