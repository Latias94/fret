use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast as _;
use wasm_bindgen::prelude::wasm_bindgen;
use winit::platform::web::WindowExtWeb;

use crate::RunnerError;

thread_local! {
    static LAST_POS: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

pub(crate) fn set(pos: Option<(f32, f32)>) {
    LAST_POS.with(|cell| cell.set(pos));
}

pub(crate) fn get() -> Option<(f32, f32)> {
    LAST_POS.with(|cell| cell.get())
}

fn pointer_offset(event: &web_sys::PointerEvent) -> (f32, f32) {
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

pub struct WebCursorListener {
    canvas: web_sys::HtmlCanvasElement,
    on_move: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>,
    on_leave: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>,
}

impl Drop for WebCursorListener {
    fn drop(&mut self) {
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

pub fn install_web_cursor_listener(
    window: &dyn winit::window::Window,
    wake: impl Fn() + 'static,
) -> Result<WebCursorListener, RunnerError> {
    let Some(canvas) = window.canvas() else {
        return Err(RunnerError::new("winit window has no canvas"));
    };
    let canvas: web_sys::HtmlCanvasElement = canvas.clone();

    let wake = Rc::new(wake);
    let wake_move = wake.clone();
    let on_move =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
            let (x, y) = pointer_offset(&event);
            set(Some((x, y)));
            wake_move();
        }) as Box<dyn FnMut(web_sys::PointerEvent)>);

    let wake_leave = wake.clone();
    let on_leave =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::PointerEvent| {
            set(None);
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
