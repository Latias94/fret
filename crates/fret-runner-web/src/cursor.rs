use std::cell::Cell;

use web_sys::wasm_bindgen::JsCast as _;

/// Minimal error type for web runner utilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerError {
    message: String,
}

impl RunnerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RunnerError {}

/// Finds a canvas element by id.
pub fn canvas_by_id(id: &str) -> Result<web_sys::HtmlCanvasElement, RunnerError> {
    let window = web_sys::window().ok_or_else(|| RunnerError::new("window is not available"))?;
    let document = window
        .document()
        .ok_or_else(|| RunnerError::new("document is not available"))?;
    let el = document
        .get_element_by_id(id)
        .ok_or_else(|| RunnerError::new("canvas element not found"))?;
    el.dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| RunnerError::new("element is not a canvas"))
}

thread_local! {
    static LAST_CURSOR_OFFSET_PX: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

pub fn last_cursor_offset_px() -> Option<(f32, f32)> {
    LAST_CURSOR_OFFSET_PX.with(|cell| cell.get())
}

fn set_last_cursor_offset_px(pos: Option<(f32, f32)>) {
    LAST_CURSOR_OFFSET_PX.with(|cell| cell.set(pos));
}

fn pointer_offset_px(event: &web_sys::PointerEvent) -> (f32, f32) {
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    extern "C" {
        type PointerEventExt;

        #[wasm_bindgen(method, getter, js_name = offsetX)]
        fn offset_x(this: &PointerEventExt) -> f64;

        #[wasm_bindgen(method, getter, js_name = offsetY)]
        fn offset_y(this: &PointerEventExt) -> f64;
    }

    let event: &PointerEventExt = event.unchecked_ref();
    (event.offset_x() as f32, event.offset_y() as f32)
}

pub struct WebCursorListener {
    canvas: web_sys::HtmlCanvasElement,
    on_move: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>,
    on_leave: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>,
}

impl Drop for WebCursorListener {
    fn drop(&mut self) {
        use wasm_bindgen::JsCast as _;

        let _ = self.canvas.remove_event_listener_with_callback(
            "pointermove",
            self.on_move.as_ref().unchecked_ref(),
        );
        let _ = self.canvas.remove_event_listener_with_callback(
            "pointerleave",
            self.on_leave.as_ref().unchecked_ref(),
        );
    }
}

/// Installs a best-effort cursor position listener on the provided canvas.
///
/// This is currently used by winit-based wasm runners as an escape hatch when the event loop
/// does not provide cursor updates while idle.
pub fn install_canvas_cursor_listener(
    canvas: web_sys::HtmlCanvasElement,
    wake: impl Fn() + 'static,
) -> Result<WebCursorListener, RunnerError> {
    let wake = std::rc::Rc::new(wake);
    let wake_move = wake.clone();
    let on_move =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
            let (x, y) = pointer_offset_px(&event);
            set_last_cursor_offset_px(Some((x, y)));
            wake_move();
        }) as Box<dyn FnMut(web_sys::PointerEvent)>);

    let wake_leave = wake.clone();
    let on_leave =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
            let (x, y) = pointer_offset_px(&event);
            set_last_cursor_offset_px(Some((x, y)));
            wake_leave();
        }) as Box<dyn FnMut(web_sys::PointerEvent)>);

    canvas
        .add_event_listener_with_callback("pointermove", on_move.as_ref().unchecked_ref())
        .map_err(|_| RunnerError::new("failed to add pointermove listener"))?;
    canvas
        .add_event_listener_with_callback("pointerleave", on_leave.as_ref().unchecked_ref())
        .map_err(|_| RunnerError::new("failed to add pointerleave listener"))?;

    Ok(WebCursorListener {
        canvas,
        on_move,
        on_leave,
    })
}
