use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use fret_core::{
    Event, ImeEvent, KeyCode, Modifiers, MouseButton, MouseButtons, Point, PointerEvent, Px, Rect,
    Scene, Size, UiServices,
};
use fret_render::{ClearColor, RenderSceneParams, Renderer, SurfaceState, WgpuContext};
use fret_runtime::{Effect, FontCatalog, FontCatalogCache, PlatformCapabilities, TimerToken};
use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsCast as _;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    AddEventListenerOptions, CompositionEvent, Document, Event as WebSysEvent, EventTarget,
    HtmlCanvasElement, HtmlElement, HtmlTextAreaElement, InputEvent, KeyboardEvent, Node,
    PointerEvent as WebPointerEvent, WheelEvent, Window,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerError {
    message: String,
}

impl RunnerError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn to_js_value(&self) -> JsValue {
        JsValue::from_str(&self.message)
    }
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RunnerError {}

fn window() -> Result<Window, RunnerError> {
    web_sys::window().ok_or_else(|| RunnerError::new("window is not available"))
}

fn is_web_input_focused(document: &Document, canvas_node: &Node, ime_node: &Node) -> bool {
    let Some(active) = document.active_element() else {
        return false;
    };
    let active_node: Node = active.unchecked_into();
    active_node.is_same_node(Some(canvas_node)) || active_node.is_same_node(Some(ime_node))
}

pub fn canvas_by_id(id: &str) -> Result<HtmlCanvasElement, RunnerError> {
    let window = window()?;
    let document = window
        .document()
        .ok_or_else(|| RunnerError::new("document is not available"))?;
    let el = document
        .get_element_by_id(id)
        .ok_or_else(|| RunnerError::new("canvas element not found"))?;
    el.dyn_into::<HtmlCanvasElement>()
        .map_err(|_| RunnerError::new("element is not a canvas"))
}

fn resize_canvas_to_display_size(
    canvas: &HtmlCanvasElement,
) -> Result<(u32, u32, f32), RunnerError> {
    let window = window()?;
    let dpr = window.device_pixel_ratio();
    let scale = dpr as f32;

    let css_w = canvas.client_width().max(0) as f64;
    let css_h = canvas.client_height().max(0) as f64;

    let mut pixel_w = (css_w * dpr).round() as u32;
    let mut pixel_h = (css_h * dpr).round() as u32;

    if pixel_w == 0 {
        pixel_w = canvas.width().max(1);
    }
    if pixel_h == 0 {
        pixel_h = canvas.height().max(1);
    }

    if canvas.width() != pixel_w {
        canvas.set_width(pixel_w);
    }
    if canvas.height() != pixel_h {
        canvas.set_height(pixel_h);
    }

    Ok((pixel_w, pixel_h, scale))
}

fn request_animation_frame(
    window: &Window,
    callback: &Closure<dyn FnMut(f64)>,
) -> Result<i32, RunnerError> {
    window
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map_err(|_| RunnerError::new("requestAnimationFrame failed"))
}

#[derive(Debug, Default)]
struct WebInputQueue {
    events: Vec<Event>,
}

impl WebInputQueue {
    fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    fn take(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}

fn modifiers_from_keyboard_event(event: &KeyboardEvent) -> Modifiers {
    Modifiers {
        shift: event.shift_key(),
        ctrl: event.ctrl_key(),
        alt: event.alt_key(),
        alt_gr: event.get_modifier_state("AltGraph"),
        meta: event.meta_key(),
    }
}

fn modifiers_from_pointer_event(event: &WebPointerEvent) -> Modifiers {
    Modifiers {
        shift: event.shift_key(),
        ctrl: event.ctrl_key(),
        alt: event.alt_key(),
        alt_gr: false,
        meta: event.meta_key(),
    }
}

fn key_code_from_dom_code(code: &str) -> KeyCode {
    KeyCode::from_token(code).unwrap_or(KeyCode::Unknown)
}

fn mouse_buttons_from_dom_buttons(buttons: u16) -> MouseButtons {
    let buttons_u = buttons;
    MouseButtons {
        left: (buttons_u & 1) != 0,
        right: (buttons_u & 2) != 0,
        middle: (buttons_u & 4) != 0,
    }
}

fn mouse_button_from_dom_button(button: i16) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        3 => MouseButton::Back,
        4 => MouseButton::Forward,
        other => MouseButton::Other(other.max(0) as u16),
    }
}

fn point_from_dom_offset_xy(offset_x: i32, offset_y: i32) -> fret_core::Point {
    fret_core::Point::new(Px(offset_x as f32), Px(offset_y as f32))
}

fn wheel_delta_from_dom(event: &WheelEvent) -> fret_core::Point {
    // DOM WheelEvent:
    // - deltaMode 0: pixels
    // - deltaMode 1: lines
    // - deltaMode 2: pages
    // `fret-core` wheel delta follows winit semantics: positive y means wheel up.
    let mode = event.delta_mode();
    let (scale_x, scale_y) = match mode {
        0 => (1.0, 1.0),
        1 => (16.0, 16.0),
        2 => (800.0, 800.0),
        _ => (1.0, 1.0),
    };
    let dx = (event.delta_x() as f32) * scale_x;
    let dy = (event.delta_y() as f32) * scale_y;
    fret_core::Point::new(Px(dx), Px(-dy))
}

pub struct WebInput {
    queue: Rc<RefCell<WebInputQueue>>,
    activation_callback: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
    _listeners: WebInputListeners,
}

impl WebInput {
    pub fn new(canvas: HtmlCanvasElement, prevent_default: bool) -> Result<Self, RunnerError> {
        let window = window()?;
        let queue = Rc::new(RefCell::new(WebInputQueue::default()));
        let activation_callback = Rc::new(RefCell::new(None));
        let listeners = WebInputListeners::install(
            window,
            canvas,
            queue.clone(),
            prevent_default,
            activation_callback.clone(),
        )?;
        Ok(Self {
            queue,
            activation_callback,
            _listeners: listeners,
        })
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        self.queue.borrow_mut().take()
    }

    pub fn set_user_activation_callback(&mut self, cb: Option<Rc<dyn Fn()>>) {
        *self.activation_callback.borrow_mut() = cb;
    }

    pub fn set_ime_allowed(&mut self, enabled: bool) {
        self._listeners.set_ime_allowed(enabled);
    }

    pub fn set_ime_cursor_area(&mut self, rect: Rect) {
        self._listeners.set_ime_cursor_area(rect);
    }
}

struct WebInputListeners {
    window: Window,
    canvas: HtmlCanvasElement,
    ime_textarea: HtmlTextAreaElement,
    ime_enabled: Rc<Cell<bool>>,

    on_pointer_move: Closure<dyn FnMut(WebPointerEvent)>,
    on_pointer_down: Closure<dyn FnMut(WebPointerEvent)>,
    on_pointer_up: Closure<dyn FnMut(WebPointerEvent)>,
    on_pointer_cancel: Closure<dyn FnMut(WebPointerEvent)>,
    on_wheel: Closure<dyn FnMut(WheelEvent)>,

    on_key_down: Closure<dyn FnMut(KeyboardEvent)>,
    on_key_up: Closure<dyn FnMut(KeyboardEvent)>,
    on_window_resize: Closure<dyn FnMut(WebSysEvent)>,

    on_composition_start: Closure<dyn FnMut(CompositionEvent)>,
    on_composition_update: Closure<dyn FnMut(CompositionEvent)>,
    on_composition_end: Closure<dyn FnMut(CompositionEvent)>,

    on_before_input: Closure<dyn FnMut(InputEvent)>,
}

impl WebInputListeners {
    fn set_ime_allowed(&self, enabled: bool) {
        self.ime_enabled.set(enabled);
        let el: HtmlElement = self.ime_textarea.clone().unchecked_into();
        if enabled {
            let _ = el.focus();
        } else {
            let _ = el.blur();
        }
    }

    fn set_ime_cursor_area(&self, rect: Rect) {
        if !self.ime_enabled.get() {
            return;
        }

        let dom_rect = self.canvas.get_bounding_client_rect();
        let left = dom_rect.left() + self.window.scroll_x().unwrap_or(0.0) + rect.origin.x.0 as f64;
        let top = dom_rect.top() + self.window.scroll_y().unwrap_or(0.0) + rect.origin.y.0 as f64;
        let w = rect.size.width.0.max(1.0) as f64;
        let h = rect.size.height.0.max(1.0) as f64;

        let el: HtmlElement = self.ime_textarea.clone().unchecked_into();
        let style = el.style();
        let _ = style.set_property("left", &format!("{left}px"));
        let _ = style.set_property("top", &format!("{top}px"));
        let _ = style.set_property("width", &format!("{w}px"));
        let _ = style.set_property("height", &format!("{h}px"));
        let _ = self.ime_textarea.set_selection_range(0, 0);
    }

    fn install(
        window: Window,
        canvas: HtmlCanvasElement,
        queue: Rc<RefCell<WebInputQueue>>,
        prevent_default: bool,
        activation_callback: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
    ) -> Result<Self, RunnerError> {
        let canvas_el: HtmlElement = canvas.clone().unchecked_into();
        canvas_el.set_tab_index(0);
        if prevent_default {
            let _ = canvas_el.style().set_property("touch-action", "none");
        }

        let document = window
            .document()
            .ok_or_else(|| RunnerError::new("document is not available"))?;
        let ime_textarea: HtmlTextAreaElement = document
            .create_element("textarea")
            .map_err(|_| RunnerError::new("failed to create IME textarea"))?
            .dyn_into()
            .map_err(|_| RunnerError::new("failed to cast IME textarea"))?;

        let ime_el: HtmlElement = ime_textarea.clone().unchecked_into();
        let style = ime_el.style();
        let _ = style.set_property("position", "absolute");
        let _ = style.set_property("opacity", "0");
        let _ = style.set_property("left", "0px");
        let _ = style.set_property("top", "0px");
        let _ = style.set_property("width", "1px");
        let _ = style.set_property("height", "1px");
        let _ = style.set_property("border", "0");
        let _ = style.set_property("padding", "0");
        let _ = style.set_property("margin", "0");
        let _ = style.set_property("background", "transparent");
        let _ = style.set_property("color", "transparent");
        let _ = style.set_property("outline", "none");
        let _ = style.set_property("resize", "none");
        let _ = style.set_property("overflow", "hidden");
        let _ = style.set_property("white-space", "pre");

        let _ = ime_el.set_attribute("aria-hidden", "true");
        let _ = ime_el.set_attribute("autocomplete", "off");
        let _ = ime_el.set_attribute("autocapitalize", "off");
        let _ = ime_el.set_attribute("spellcheck", "false");

        if let Some(body) = document.body() {
            let _ = body.append_child(&ime_el);
        }

        let ime_enabled = Rc::new(Cell::new(false));

        let queue_move = queue.clone();
        let on_pointer_move = Closure::wrap(Box::new(move |event: WebPointerEvent| {
            if prevent_default {
                event.prevent_default();
            }

            let position = point_from_dom_offset_xy(event.offset_x(), event.offset_y());
            let modifiers = modifiers_from_pointer_event(&event);
            let buttons = mouse_buttons_from_dom_buttons(event.buttons());

            if let Ok(mut q) = queue_move.try_borrow_mut() {
                q.push(Event::Pointer(PointerEvent::Move {
                    position,
                    buttons,
                    modifiers,
                }));
            }
        }) as Box<dyn FnMut(WebPointerEvent)>);

        let queue_down = queue.clone();
        let canvas_focus: HtmlElement = canvas.clone().unchecked_into();
        let ime_focus: HtmlElement = ime_textarea.clone().unchecked_into();
        let ime_enabled_for_down = ime_enabled.clone();
        let activation_for_pointer_down = activation_callback.clone();
        let canvas_capture = canvas.clone();
        let on_pointer_down = Closure::wrap(Box::new(move |event: WebPointerEvent| {
            if ime_enabled_for_down.get() {
                let _ = ime_focus.focus();
            } else {
                let _ = canvas_focus.focus();
            }
            let _ = canvas_capture.set_pointer_capture(event.pointer_id());

            if prevent_default {
                event.prevent_default();
            }

            let position = point_from_dom_offset_xy(event.offset_x(), event.offset_y());
            let modifiers = modifiers_from_pointer_event(&event);
            let button = mouse_button_from_dom_button(event.button());

            if let Ok(mut q) = queue_down.try_borrow_mut() {
                q.push(Event::Pointer(PointerEvent::Down {
                    position,
                    button,
                    modifiers,
                }));
            }

            let cb = activation_for_pointer_down
                .try_borrow()
                .ok()
                .and_then(|v| v.as_ref().cloned());
            if let Some(cb) = cb {
                cb();
            }
        }) as Box<dyn FnMut(WebPointerEvent)>);

        let queue_up = queue.clone();
        let canvas_release = canvas.clone();
        let activation_for_pointer_up = activation_callback.clone();
        let on_pointer_up = Closure::wrap(Box::new(move |event: WebPointerEvent| {
            let _ = canvas_release.release_pointer_capture(event.pointer_id());
            if prevent_default {
                event.prevent_default();
            }

            let position = point_from_dom_offset_xy(event.offset_x(), event.offset_y());
            let modifiers = modifiers_from_pointer_event(&event);
            let button = mouse_button_from_dom_button(event.button());

            if let Ok(mut q) = queue_up.try_borrow_mut() {
                q.push(Event::Pointer(PointerEvent::Up {
                    position,
                    button,
                    modifiers,
                }));
            }

            let cb = activation_for_pointer_up
                .try_borrow()
                .ok()
                .and_then(|v| v.as_ref().cloned());
            if let Some(cb) = cb {
                cb();
            }
        }) as Box<dyn FnMut(WebPointerEvent)>);

        let queue_cancel = queue.clone();
        let canvas_cancel_release = canvas.clone();
        let on_pointer_cancel = Closure::wrap(Box::new(move |event: WebPointerEvent| {
            let _ = canvas_cancel_release.release_pointer_capture(event.pointer_id());
            let position = point_from_dom_offset_xy(event.offset_x(), event.offset_y());
            let modifiers = modifiers_from_pointer_event(&event);
            let buttons = mouse_buttons_from_dom_buttons(event.buttons());
            if let Ok(mut q) = queue_cancel.try_borrow_mut() {
                q.push(Event::Pointer(PointerEvent::Move {
                    position,
                    buttons,
                    modifiers,
                }));
            }
        }) as Box<dyn FnMut(WebPointerEvent)>);

        let queue_wheel = queue.clone();
        let on_wheel = Closure::wrap(Box::new(move |event: WheelEvent| {
            if prevent_default {
                event.prevent_default();
            }

            let position = point_from_dom_offset_xy(event.offset_x(), event.offset_y());
            let delta = wheel_delta_from_dom(&event);
            let modifiers = Modifiers {
                shift: event.shift_key(),
                ctrl: event.ctrl_key(),
                alt: event.alt_key(),
                alt_gr: false,
                meta: event.meta_key(),
            };

            if let Ok(mut q) = queue_wheel.try_borrow_mut() {
                q.push(Event::Pointer(PointerEvent::Wheel {
                    position,
                    delta,
                    modifiers,
                }));
            }
        }) as Box<dyn FnMut(WheelEvent)>);

        let queue_key_down = queue.clone();
        let document_for_keys = document.clone();
        let canvas_node_for_keys: Node = canvas.clone().unchecked_into();
        let ime_node_for_keys: Node = ime_textarea.clone().unchecked_into();
        let activation_for_key_down = activation_callback.clone();
        let on_key_down = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if !is_web_input_focused(
                &document_for_keys,
                &canvas_node_for_keys,
                &ime_node_for_keys,
            ) {
                return;
            }

            if prevent_default {
                event.prevent_default();
            }

            let key = key_code_from_dom_code(&event.code());
            let modifiers = modifiers_from_keyboard_event(&event);
            let repeat = event.repeat();

            if let Ok(mut q) = queue_key_down.try_borrow_mut() {
                q.push(Event::KeyDown {
                    key,
                    modifiers,
                    repeat,
                });
            }

            let cb = activation_for_key_down
                .try_borrow()
                .ok()
                .and_then(|v| v.as_ref().cloned());
            if let Some(cb) = cb {
                cb();
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);

        let queue_key_up = queue.clone();
        let document_for_keys = document.clone();
        let canvas_node_for_keys: Node = canvas.clone().unchecked_into();
        let ime_node_for_keys: Node = ime_textarea.clone().unchecked_into();
        let activation_for_key_up = activation_callback.clone();
        let on_key_up = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if !is_web_input_focused(
                &document_for_keys,
                &canvas_node_for_keys,
                &ime_node_for_keys,
            ) {
                return;
            }

            if prevent_default {
                event.prevent_default();
            }

            let key = key_code_from_dom_code(&event.code());
            let modifiers = modifiers_from_keyboard_event(&event);

            if let Ok(mut q) = queue_key_up.try_borrow_mut() {
                q.push(Event::KeyUp { key, modifiers });
            }

            let cb = activation_for_key_up
                .try_borrow()
                .ok()
                .and_then(|v| v.as_ref().cloned());
            if let Some(cb) = cb {
                cb();
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);

        let queue_resize = queue.clone();
        let canvas_for_resize = canvas.clone();
        let on_window_resize = Closure::wrap(Box::new(move |_event: WebSysEvent| {
            let dpr = web_sys::window()
                .map(|w| w.device_pixel_ratio())
                .unwrap_or(1.0);
            let width = canvas_for_resize.client_width().max(0) as f32;
            let height = canvas_for_resize.client_height().max(0) as f32;

            if let Ok(mut q) = queue_resize.try_borrow_mut() {
                q.push(Event::WindowScaleFactorChanged(dpr as f32));
                q.push(Event::WindowResized {
                    width: Px(width),
                    height: Px(height),
                });
            }
        }) as Box<dyn FnMut(WebSysEvent)>);

        let queue_comp_start = queue.clone();
        let on_composition_start = Closure::wrap(Box::new(move |event: CompositionEvent| {
            if prevent_default {
                event.prevent_default();
            }

            if let Ok(mut q) = queue_comp_start.try_borrow_mut() {
                q.push(Event::Ime(ImeEvent::Enabled));
                if let Some(text) = event.data() {
                    if !text.is_empty() {
                        q.push(Event::Ime(ImeEvent::Preedit { text, cursor: None }));
                    }
                }
            }
        }) as Box<dyn FnMut(CompositionEvent)>);

        let queue_comp_update = queue.clone();
        let on_composition_update = Closure::wrap(Box::new(move |event: CompositionEvent| {
            if prevent_default {
                event.prevent_default();
            }

            if let Ok(mut q) = queue_comp_update.try_borrow_mut() {
                q.push(Event::Ime(ImeEvent::Preedit {
                    text: event.data().unwrap_or_default(),
                    cursor: None,
                }));
            }
        }) as Box<dyn FnMut(CompositionEvent)>);

        let queue_comp_end = queue.clone();
        let on_composition_end = Closure::wrap(Box::new(move |event: CompositionEvent| {
            if prevent_default {
                event.prevent_default();
            }

            if let Ok(mut q) = queue_comp_end.try_borrow_mut() {
                let text = event.data().unwrap_or_default();
                if !text.is_empty() {
                    q.push(Event::Ime(ImeEvent::Commit(text)));
                }
                q.push(Event::Ime(ImeEvent::Disabled));
            }
        }) as Box<dyn FnMut(CompositionEvent)>);

        let queue_before_input = queue.clone();
        let on_before_input = Closure::wrap(Box::new(move |event: InputEvent| {
            if prevent_default {
                event.prevent_default();
            }

            if event.is_composing() {
                return;
            }

            let Some(text) = event.data() else {
                return;
            };
            if text.is_empty() {
                return;
            }

            if let Ok(mut q) = queue_before_input.try_borrow_mut() {
                q.push(Event::TextInput(text));
            }
        }) as Box<dyn FnMut(InputEvent)>);

        let canvas_target: EventTarget = canvas.clone().unchecked_into();
        let window_target: EventTarget = window.clone().unchecked_into();
        let ime_target: EventTarget = ime_textarea.clone().unchecked_into();

        if let Ok(mut q) = queue.try_borrow_mut() {
            q.push(Event::WindowScaleFactorChanged(
                window.device_pixel_ratio() as f32
            ));
            q.push(Event::WindowResized {
                width: Px(canvas.client_width().max(0) as f32),
                height: Px(canvas.client_height().max(0) as f32),
            });
        }

        canvas_target
            .add_event_listener_with_callback(
                "pointermove",
                on_pointer_move.as_ref().unchecked_ref(),
            )
            .map_err(|_| RunnerError::new("failed to register pointermove listener"))?;
        canvas_target
            .add_event_listener_with_callback(
                "pointerdown",
                on_pointer_down.as_ref().unchecked_ref(),
            )
            .map_err(|_| RunnerError::new("failed to register pointerdown listener"))?;
        canvas_target
            .add_event_listener_with_callback("pointerup", on_pointer_up.as_ref().unchecked_ref())
            .map_err(|_| RunnerError::new("failed to register pointerup listener"))?;
        canvas_target
            .add_event_listener_with_callback(
                "pointercancel",
                on_pointer_cancel.as_ref().unchecked_ref(),
            )
            .map_err(|_| RunnerError::new("failed to register pointercancel listener"))?;
        let wheel_opts = AddEventListenerOptions::new();
        wheel_opts.set_passive(false);
        canvas_target
            .add_event_listener_with_callback_and_add_event_listener_options(
                "wheel",
                on_wheel.as_ref().unchecked_ref(),
                &wheel_opts,
            )
            .map_err(|_| RunnerError::new("failed to register wheel listener"))?;

        window_target
            .add_event_listener_with_callback("keydown", on_key_down.as_ref().unchecked_ref())
            .map_err(|_| RunnerError::new("failed to register keydown listener"))?;
        window_target
            .add_event_listener_with_callback("keyup", on_key_up.as_ref().unchecked_ref())
            .map_err(|_| RunnerError::new("failed to register keyup listener"))?;
        window_target
            .add_event_listener_with_callback("resize", on_window_resize.as_ref().unchecked_ref())
            .map_err(|_| RunnerError::new("failed to register resize listener"))?;

        canvas_target
            .add_event_listener_with_callback(
                "compositionstart",
                on_composition_start.as_ref().unchecked_ref(),
            )
            .map_err(|_| RunnerError::new("failed to register compositionstart listener"))?;
        canvas_target
            .add_event_listener_with_callback(
                "compositionupdate",
                on_composition_update.as_ref().unchecked_ref(),
            )
            .map_err(|_| RunnerError::new("failed to register compositionupdate listener"))?;
        canvas_target
            .add_event_listener_with_callback(
                "compositionend",
                on_composition_end.as_ref().unchecked_ref(),
            )
            .map_err(|_| RunnerError::new("failed to register compositionend listener"))?;

        canvas_target
            .add_event_listener_with_callback(
                "beforeinput",
                on_before_input.as_ref().unchecked_ref(),
            )
            .map_err(|_| RunnerError::new("failed to register beforeinput listener"))?;

        let _ = ime_target.add_event_listener_with_callback(
            "compositionstart",
            on_composition_start.as_ref().unchecked_ref(),
        );
        let _ = ime_target.add_event_listener_with_callback(
            "compositionupdate",
            on_composition_update.as_ref().unchecked_ref(),
        );
        let _ = ime_target.add_event_listener_with_callback(
            "compositionend",
            on_composition_end.as_ref().unchecked_ref(),
        );
        let _ = ime_target.add_event_listener_with_callback(
            "beforeinput",
            on_before_input.as_ref().unchecked_ref(),
        );

        Ok(Self {
            window,
            canvas,
            ime_textarea,
            ime_enabled,

            on_pointer_move,
            on_pointer_down,
            on_pointer_up,
            on_pointer_cancel,
            on_wheel,

            on_key_down,
            on_key_up,
            on_window_resize,

            on_composition_start,
            on_composition_update,
            on_composition_end,

            on_before_input,
        })
    }

    fn uninstall(&self) {
        let canvas_target: EventTarget = self.canvas.clone().unchecked_into();
        let window_target: EventTarget = self.window.clone().unchecked_into();
        let ime_target: EventTarget = self.ime_textarea.clone().unchecked_into();

        let _ = canvas_target.remove_event_listener_with_callback(
            "pointermove",
            self.on_pointer_move.as_ref().unchecked_ref(),
        );
        let _ = canvas_target.remove_event_listener_with_callback(
            "pointerdown",
            self.on_pointer_down.as_ref().unchecked_ref(),
        );
        let _ = canvas_target.remove_event_listener_with_callback(
            "pointerup",
            self.on_pointer_up.as_ref().unchecked_ref(),
        );
        let _ = canvas_target.remove_event_listener_with_callback(
            "pointercancel",
            self.on_pointer_cancel.as_ref().unchecked_ref(),
        );
        let _ = canvas_target
            .remove_event_listener_with_callback("wheel", self.on_wheel.as_ref().unchecked_ref());

        let _ = window_target.remove_event_listener_with_callback(
            "keydown",
            self.on_key_down.as_ref().unchecked_ref(),
        );
        let _ = window_target
            .remove_event_listener_with_callback("keyup", self.on_key_up.as_ref().unchecked_ref());
        let _ = window_target.remove_event_listener_with_callback(
            "resize",
            self.on_window_resize.as_ref().unchecked_ref(),
        );

        let _ = canvas_target.remove_event_listener_with_callback(
            "compositionstart",
            self.on_composition_start.as_ref().unchecked_ref(),
        );
        let _ = canvas_target.remove_event_listener_with_callback(
            "compositionupdate",
            self.on_composition_update.as_ref().unchecked_ref(),
        );
        let _ = canvas_target.remove_event_listener_with_callback(
            "compositionend",
            self.on_composition_end.as_ref().unchecked_ref(),
        );

        let _ = canvas_target.remove_event_listener_with_callback(
            "beforeinput",
            self.on_before_input.as_ref().unchecked_ref(),
        );

        let _ = ime_target.remove_event_listener_with_callback(
            "compositionstart",
            self.on_composition_start.as_ref().unchecked_ref(),
        );
        let _ = ime_target.remove_event_listener_with_callback(
            "compositionupdate",
            self.on_composition_update.as_ref().unchecked_ref(),
        );
        let _ = ime_target.remove_event_listener_with_callback(
            "compositionend",
            self.on_composition_end.as_ref().unchecked_ref(),
        );
        let _ = ime_target.remove_event_listener_with_callback(
            "beforeinput",
            self.on_before_input.as_ref().unchecked_ref(),
        );

        let ime_el: HtmlElement = self.ime_textarea.clone().unchecked_into();
        if let Some(parent) = ime_el.parent_node() {
            let _ = parent.remove_child(&ime_el);
        }
    }
}

impl Drop for WebInputListeners {
    fn drop(&mut self) {
        self.uninstall();
    }
}

pub struct WebEffectPump {
    canvas: HtmlCanvasElement,
    queued_events: Rc<RefCell<Vec<Event>>>,
    fired_timeouts: Rc<RefCell<Vec<TimerToken>>>,
    timers: HashMap<TimerToken, WebTimer>,
    file_dialogs: Rc<RefCell<WebFileDialogState>>,
}

struct WebTimer {
    id: i32,
    repeat: Option<Duration>,
    callback: Closure<dyn FnMut()>,
}

#[derive(Default)]
struct WebFileDialogState {
    next_token: u64,
    selections: HashMap<fret_runtime::FileDialogToken, Vec<web_sys::File>>,
}

impl WebFileDialogState {
    fn allocate_token(&mut self) -> fret_runtime::FileDialogToken {
        let next = self.next_token.max(1);
        let token = fret_runtime::FileDialogToken(next);
        self.next_token = next.saturating_add(1);
        token
    }
}

impl WebEffectPump {
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        Self {
            canvas,
            queued_events: Rc::new(RefCell::new(Vec::new())),
            fired_timeouts: Rc::new(RefCell::new(Vec::new())),
            timers: HashMap::new(),
            file_dialogs: Rc::new(RefCell::new(WebFileDialogState::default())),
        }
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut *self.queued_events.borrow_mut())
    }

    pub fn tick(&mut self) {
        self.collect_fired_timeouts();
    }

    pub fn handle_effects<H>(
        &mut self,
        app: &mut H,
        runner: &mut WebRunner,
        input: &mut WebInput,
        redraw_window: fret_core::AppWindowId,
        effects: impl IntoIterator<Item = Effect>,
    ) -> Vec<Effect>
    where
        H: fret_runtime::GlobalsHost + fret_runtime::EffectSink,
    {
        let mut unhandled: Vec<Effect> = Vec::new();
        for effect in effects {
            match effect {
                Effect::Redraw(_) | Effect::RequestAnimationFrame(_) => {}
                Effect::SetTimer {
                    token,
                    after,
                    repeat,
                    ..
                } => self.schedule_timer(token, after, repeat),
                Effect::CancelTimer { token } => self.cancel_timer(token),
                Effect::CursorSetIcon { icon, .. } => self.set_cursor_icon(icon),
                Effect::ImeAllow { enabled, .. } => input.set_ime_allowed(enabled),
                Effect::ImeSetCursorArea { rect, .. } => input.set_ime_cursor_area(rect),
                Effect::TextAddFonts { fonts } => {
                    let added = runner.add_fonts(fonts);
                    if added == 0 {
                        continue;
                    }

                    let prev_rev = app.global::<FontCatalog>().map(|c| c.revision).unwrap_or(0);
                    let revision = prev_rev.saturating_add(1);
                    let families = runner.all_font_names();
                    let cache = FontCatalogCache::from_families(revision, &families);
                    app.set_global::<FontCatalog>(FontCatalog { families, revision });
                    app.set_global::<FontCatalogCache>(cache);

                    // See desktop runner rationale: this forces global-change observation even if
                    // config value is unchanged.
                    let config = app
                        .global::<fret_core::TextFontFamilyConfig>()
                        .cloned()
                        .unwrap_or_default();
                    app.set_global::<fret_core::TextFontFamilyConfig>(config);

                    app.request_redraw(redraw_window);
                }
                Effect::ClipboardSetText { text } => {
                    let caps = app
                        .global::<PlatformCapabilities>()
                        .cloned()
                        .unwrap_or_default();
                    if !caps.clipboard.text {
                        continue;
                    }
                    let Some(window) = web_sys::window() else {
                        continue;
                    };
                    let clipboard = window.navigator().clipboard();
                    spawn_local(async move {
                        let _ = JsFuture::from(clipboard.write_text(&text)).await;
                    });
                }
                Effect::ClipboardGetText { token, .. } => {
                    let caps = app
                        .global::<PlatformCapabilities>()
                        .cloned()
                        .unwrap_or_default();
                    if !caps.clipboard.text {
                        self.queued_events
                            .borrow_mut()
                            .push(Event::ClipboardTextUnavailable { token });
                        continue;
                    }

                    let Some(window) = web_sys::window() else {
                        self.queued_events
                            .borrow_mut()
                            .push(Event::ClipboardTextUnavailable { token });
                        continue;
                    };
                    let clipboard = window.navigator().clipboard();
                    let queue = self.queued_events.clone();
                    spawn_local(async move {
                        let result = JsFuture::from(clipboard.read_text()).await;
                        let event = match result {
                            Ok(v) => Event::ClipboardText {
                                token,
                                text: v.as_string().unwrap_or_default(),
                            },
                            Err(_) => Event::ClipboardTextUnavailable { token },
                        };
                        let _ = queue.try_borrow_mut().map(|mut q| q.push(event));
                    });
                }
                Effect::OpenUrl { url } => {
                    let caps = app
                        .global::<PlatformCapabilities>()
                        .cloned()
                        .unwrap_or_default();
                    if !caps.shell.open_url {
                        continue;
                    }
                    let Some(window) = web_sys::window() else {
                        continue;
                    };
                    let _ = window.open_with_url(&url);
                }
                Effect::FileDialogOpen { options, .. } => {
                    let caps = app
                        .global::<PlatformCapabilities>()
                        .cloned()
                        .unwrap_or_default();
                    if !caps.fs.file_dialogs {
                        continue;
                    }

                    let Some(window) = web_sys::window() else {
                        continue;
                    };
                    let Some(document) = window.document() else {
                        continue;
                    };
                    let Ok(el) = document.create_element("input") else {
                        continue;
                    };
                    let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() else {
                        continue;
                    };

                    input.set_type("file");
                    input.set_multiple(options.multiple);

                    let accept = {
                        let mut parts: Vec<String> = Vec::new();
                        for filter in &options.filters {
                            for ext in &filter.extensions {
                                let ext = ext.trim().trim_start_matches('.');
                                if ext.is_empty() {
                                    continue;
                                }
                                parts.push(format!(".{ext}"));
                            }
                        }
                        parts.join(",")
                    };
                    if !accept.is_empty() {
                        input.set_accept(&accept);
                    }

                    let input_el: HtmlElement = input.clone().unchecked_into();
                    let style = input_el.style();
                    let _ = style.set_property("position", "absolute");
                    let _ = style.set_property("left", "0px");
                    let _ = style.set_property("top", "0px");
                    let _ = style.set_property("opacity", "0");
                    let _ = style.set_property("width", "1px");
                    let _ = style.set_property("height", "1px");
                    let _ = style.set_property("pointer-events", "none");
                    let _ = input_el.set_attribute("aria-hidden", "true");

                    if let Some(body) = document.body() {
                        let _ = body.append_child(&input_el);
                    }

                    let queue = self.queued_events.clone();
                    let state = self.file_dialogs.clone();
                    let input_target: EventTarget = input.clone().unchecked_into();
                    let input_target_for_cb = input_target.clone();
                    let input_for_cb = input.clone();
                    let input_node_for_cb: Node = input.clone().unchecked_into();

                    let callback_cell: Rc<RefCell<Option<Closure<dyn FnMut(WebSysEvent)>>>> =
                        Rc::new(RefCell::new(None));
                    let callback_cell_for_cb = callback_cell.clone();

                    let on_change = Closure::wrap(Box::new(move |_event: WebSysEvent| {
                        if let Some(parent) = input_node_for_cb.parent_node() {
                            let _ = parent.remove_child(&input_node_for_cb);
                        }

                        if let Ok(holder) = callback_cell_for_cb.try_borrow() {
                            if let Some(cb) = holder.as_ref() {
                                let _ = input_target_for_cb.remove_event_listener_with_callback(
                                    "change",
                                    cb.as_ref().unchecked_ref(),
                                );
                            }
                        }
                        callback_cell_for_cb.borrow_mut().take();

                        let mut selected: Vec<web_sys::File> = Vec::new();
                        if let Some(files) = input_for_cb.files() {
                            for i in 0..files.length() {
                                if let Some(file) = files.item(i) {
                                    selected.push(file);
                                }
                            }
                        }

                        if selected.is_empty() {
                            let _ = queue
                                .try_borrow_mut()
                                .map(|mut q| q.push(Event::FileDialogCanceled));
                            return;
                        }

                        let (token, files_meta) = {
                            let mut st = state.borrow_mut();
                            let token = st.allocate_token();
                            let files_meta = selected
                                .iter()
                                .map(|f| fret_core::ExternalDragFile { name: f.name() })
                                .collect::<Vec<_>>();
                            st.selections.insert(token, selected);
                            (token, files_meta)
                        };

                        let selection = fret_core::FileDialogSelection {
                            token,
                            files: files_meta,
                        };
                        let _ = queue
                            .try_borrow_mut()
                            .map(|mut q| q.push(Event::FileDialogSelection(selection)));
                    })
                        as Box<dyn FnMut(WebSysEvent)>);

                    *callback_cell.borrow_mut() = Some(on_change);
                    if let Ok(holder) = callback_cell.try_borrow() {
                        if let Some(cb) = holder.as_ref() {
                            let _ = input_target.add_event_listener_with_callback(
                                "change",
                                cb.as_ref().unchecked_ref(),
                            );
                        }
                    }

                    input.click();
                }
                Effect::FileDialogReadAll { token, .. } => {
                    self.file_dialog_read_all(
                        token,
                        fret_core::ExternalDropReadLimits {
                            max_total_bytes: 64 * 1024 * 1024,
                            max_file_bytes: 32 * 1024 * 1024,
                            max_files: 64,
                        },
                    );
                }
                Effect::FileDialogReadAllWithLimits { token, limits, .. } => {
                    let cap = fret_core::ExternalDropReadLimits {
                        max_total_bytes: 64 * 1024 * 1024,
                        max_file_bytes: 32 * 1024 * 1024,
                        max_files: 64,
                    };
                    self.file_dialog_read_all(token, limits.capped_by(cap));
                }
                Effect::FileDialogRelease { token } => {
                    self.file_dialogs.borrow_mut().selections.remove(&token);
                }
                other => unhandled.push(other),
            }
        }
        unhandled
    }

    fn file_dialog_read_all(
        &self,
        token: fret_runtime::FileDialogToken,
        limits: fret_core::ExternalDropReadLimits,
    ) {
        let files = self.file_dialogs.borrow().selections.get(&token).cloned();
        let Some(files) = files else {
            return;
        };

        let queue = self.queued_events.clone();
        spawn_local(async move {
            let mut out_files: Vec<fret_core::ExternalDropFileData> = Vec::new();
            let mut errors: Vec<fret_core::ExternalDropReadError> = Vec::new();
            let mut total: u64 = 0;

            for file in files.into_iter().take(limits.max_files) {
                let name = file.name();
                let reported_size = file.size() as u64;

                if reported_size > limits.max_file_bytes {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: format!(
                            "file too large ({} bytes > max_file_bytes {})",
                            reported_size, limits.max_file_bytes
                        ),
                    });
                    continue;
                }

                if total >= limits.max_total_bytes {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: format!(
                            "selection too large (total {} >= max_total_bytes {})",
                            total, limits.max_total_bytes
                        ),
                    });
                    break;
                }

                let buf = match JsFuture::from(file.array_buffer()).await {
                    Ok(v) => v,
                    Err(err) => {
                        errors.push(fret_core::ExternalDropReadError {
                            name,
                            message: format!("read failed: {err:?}"),
                        });
                        continue;
                    }
                };
                let bytes = Uint8Array::new(&buf).to_vec();

                if bytes.len() as u64 > limits.max_file_bytes {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: format!(
                            "file too large ({} bytes > max_file_bytes {})",
                            bytes.len(),
                            limits.max_file_bytes
                        ),
                    });
                    continue;
                }

                let next_total = total.saturating_add(bytes.len() as u64);
                if next_total > limits.max_total_bytes {
                    errors.push(fret_core::ExternalDropReadError {
                        name,
                        message: format!(
                            "selection too large (next_total {} > max_total_bytes {})",
                            next_total, limits.max_total_bytes
                        ),
                    });
                    break;
                }
                total = next_total;

                out_files.push(fret_core::ExternalDropFileData { name, bytes });
            }

            let data = fret_core::FileDialogDataEvent {
                token,
                files: out_files,
                errors,
            };
            let _ = queue
                .try_borrow_mut()
                .map(|mut q| q.push(Event::FileDialogData(data)));
        });
    }

    fn ms(d: Duration) -> i32 {
        (d.as_millis().min(i32::MAX as u128) as i32).max(0)
    }

    fn schedule_timer(&mut self, token: TimerToken, after: Duration, repeat: Option<Duration>) {
        let Some(window) = web_sys::window() else {
            return;
        };

        if let Some(existing) = self.timers.remove(&token) {
            window.clear_timeout_with_handle(existing.id);
        }

        let queue = self.queued_events.clone();
        let fired = self.fired_timeouts.clone();
        let callback = Closure::wrap(Box::new(move || {
            if let Ok(mut q) = queue.try_borrow_mut() {
                q.push(Event::Timer { token });
            }
            let _ = fired.try_borrow_mut().map(|mut v| v.push(token));
        }) as Box<dyn FnMut()>);

        let id = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                Self::ms(after),
            )
            .unwrap_or(0);

        self.timers.insert(
            token,
            WebTimer {
                id,
                repeat,
                callback,
            },
        );
    }

    fn cancel_timer(&mut self, token: TimerToken) {
        let Some(window) = web_sys::window() else {
            return;
        };

        if let Some(timer) = self.timers.remove(&token) {
            window.clear_timeout_with_handle(timer.id);
        }
    }

    fn collect_fired_timeouts(&mut self) {
        let tokens = std::mem::take(&mut *self.fired_timeouts.borrow_mut());
        if tokens.is_empty() {
            return;
        }

        let Some(window) = web_sys::window() else {
            return;
        };

        for token in tokens {
            let Some(timer) = self.timers.remove(&token) else {
                continue;
            };

            window.clear_timeout_with_handle(timer.id);

            let Some(repeat) = timer.repeat else {
                continue;
            };

            let id = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    timer.callback.as_ref().unchecked_ref(),
                    Self::ms(repeat),
                )
                .unwrap_or(0);
            self.timers.insert(
                token,
                WebTimer {
                    id,
                    repeat: Some(repeat),
                    callback: timer.callback,
                },
            );
        }
    }

    fn set_cursor_icon(&mut self, icon: fret_core::CursorIcon) {
        let cursor = match icon {
            fret_core::CursorIcon::Default => "default",
            fret_core::CursorIcon::Pointer => "pointer",
            fret_core::CursorIcon::Text => "text",
            fret_core::CursorIcon::ColResize => "col-resize",
            fret_core::CursorIcon::RowResize => "row-resize",
        };

        let canvas_el: HtmlElement = self.canvas.clone().unchecked_into();
        let _ = canvas_el.style().set_property("cursor", cursor);
    }
}

pub struct WebRunner {
    canvas: HtmlCanvasElement,
    ctx: WgpuContext,
    surface_state: SurfaceState<'static>,
    renderer: Renderer,
    clear: ClearColor,
    scene: Scene,
}

impl WebRunner {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, RunnerError> {
        let (width, height, _scale) = resize_canvas_to_display_size(&canvas)?;
        let (ctx, surface) =
            WgpuContext::new_with_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
                .await
                .map_err(|e| RunnerError::new(format!("{e}")))?;

        let surface_state = SurfaceState::new(&ctx.adapter, &ctx.device, surface, width, height)
            .map_err(|e| RunnerError::new(format!("{e}")))?;

        let renderer = Renderer::new(&ctx.adapter, &ctx.device);

        Ok(Self {
            canvas,
            ctx,
            surface_state,
            renderer,
            clear: ClearColor::default(),
            scene: Scene::default(),
        })
    }

    pub async fn from_canvas_id(canvas_id: &str) -> Result<Self, RunnerError> {
        let canvas = canvas_by_id(canvas_id)?;
        Self::new(canvas).await
    }

    pub fn set_clear_color(&mut self, clear: ClearColor) {
        self.clear = clear;
    }

    pub fn services_mut(&mut self) -> &mut dyn UiServices {
        &mut self.renderer
    }

    pub fn services_and_scene_mut(&mut self) -> (&mut dyn UiServices, &mut Scene) {
        (&mut self.renderer, &mut self.scene)
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    /// Returns UI layout bounds (in CSS pixels) and device scale factor (DPR).
    pub fn ui_bounds_and_scale(&self) -> Result<(Rect, f32), RunnerError> {
        let window = window()?;
        let dpr = window.device_pixel_ratio().max(1.0) as f32;

        let mut w = self.canvas.client_width().max(0) as f32;
        let mut h = self.canvas.client_height().max(0) as f32;

        if w == 0.0 {
            w = (self.canvas.width().max(1) as f32) / dpr;
        }
        if h == 0.0 {
            h = (self.canvas.height().max(1) as f32) / dpr;
        }

        Ok((
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(w), Px(h))),
            dpr,
        ))
    }

    pub fn render_once(&mut self) -> Result<(), RunnerError> {
        let (width, height, scale_factor) = resize_canvas_to_display_size(&self.canvas)?;
        let (cur_w, cur_h) = self.surface_state.size();
        if (width, height) != (cur_w, cur_h) {
            self.surface_state
                .resize(&self.ctx.device, width.max(1), height.max(1));
        }

        let (frame, view) = self
            .surface_state
            .get_current_frame_view()
            .map_err(|e| RunnerError::new(format!("{e:?}")))?;

        let cmd = self.renderer.render_scene(
            &self.ctx.device,
            &self.ctx.queue,
            RenderSceneParams {
                format: self.surface_state.format(),
                target_view: &view,
                scene: &self.scene,
                clear: self.clear,
                scale_factor,
                viewport_size: self.surface_state.size(),
            },
        );
        self.ctx.queue.submit([cmd]);
        frame.present();
        Ok(())
    }

    pub fn add_fonts(&mut self, fonts: impl IntoIterator<Item = Vec<u8>>) -> usize {
        self.renderer.add_fonts(fonts)
    }

    pub fn all_font_names(&self) -> Vec<String> {
        self.renderer.all_font_names()
    }
}

struct RafLoopState {
    active: bool,
    callback: Option<Closure<dyn FnMut(f64)>>,
    raf_id: Option<i32>,
    window: Window,
}

pub struct RafLoop {
    state: Rc<RefCell<RafLoopState>>,
}

impl RafLoop {
    pub fn stop(&mut self) {
        let mut state = self.state.borrow_mut();
        state.active = false;
        if let Some(id) = state.raf_id.take() {
            let _ = state.window.cancel_animation_frame(id);
        }
        state.callback.take();
    }
}

impl Drop for RafLoop {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn start_raf_loop(runner: WebRunner) -> Result<RafLoop, RunnerError> {
    start_raf_loop_shared(Rc::new(RefCell::new(runner)))
}

fn start_raf_loop_shared(runner: Rc<RefCell<WebRunner>>) -> Result<RafLoop, RunnerError> {
    let window = window()?;
    let window_for_cb = window.clone();
    let window_for_first = window_for_cb.clone();

    let state = Rc::new(RefCell::new(RafLoopState {
        active: true,
        callback: None,
        raf_id: None,
        window,
    }));

    let state_for_cb = state.clone();
    let runner_for_cb = runner.clone();

    let callback = Closure::wrap(Box::new(move |_ts: f64| {
        if !state_for_cb.borrow().active {
            return;
        }

        if let Ok(mut r) = runner_for_cb.try_borrow_mut() {
            let _ = r.render_once();
        }

        let next_id = {
            let state = state_for_cb.borrow();
            state
                .callback
                .as_ref()
                .and_then(|cb| request_animation_frame(&window_for_cb, cb).ok())
        };
        if let Some(id) = next_id {
            state_for_cb.borrow_mut().raf_id = Some(id);
        }
    }) as Box<dyn FnMut(f64)>);

    state.borrow_mut().callback = Some(callback);
    let first_id = {
        let state_ref = state.borrow();
        state_ref
            .callback
            .as_ref()
            .and_then(|cb| request_animation_frame(&window_for_first, cb).ok())
    };
    if let Some(id) = first_id {
        state.borrow_mut().raf_id = Some(id);
    }

    Ok(RafLoop { state })
}

#[wasm_bindgen]
pub async fn render_one_frame(canvas_id: &str) -> Result<(), JsValue> {
    WebRunner::from_canvas_id(canvas_id)
        .await
        .and_then(|mut runner| runner.render_once())
        .map_err(|e| e.to_js_value())
}

#[wasm_bindgen]
pub struct WebRunnerHandle {
    runner: Rc<RefCell<WebRunner>>,
    loop_handle: Option<RafLoop>,
    input: Option<WebInput>,
}

#[wasm_bindgen]
impl WebRunnerHandle {
    #[wasm_bindgen(js_name = create)]
    pub async fn create(canvas_id: String) -> Result<WebRunnerHandle, JsValue> {
        let runner = WebRunner::from_canvas_id(&canvas_id)
            .await
            .map_err(|e| e.to_js_value())?;
        Ok(WebRunnerHandle {
            runner: Rc::new(RefCell::new(runner)),
            loop_handle: None,
            input: None,
        })
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        if self.loop_handle.is_some() {
            return Ok(());
        }
        let loop_handle =
            start_raf_loop_shared(self.runner.clone()).map_err(|e| e.to_js_value())?;
        self.loop_handle = Some(loop_handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut handle) = self.loop_handle.take() {
            handle.stop();
        }
    }

    #[wasm_bindgen(js_name = installInputListeners)]
    pub fn install_input_listeners(&mut self, prevent_default: bool) -> Result<(), JsValue> {
        if self.input.is_some() {
            return Ok(());
        }

        let canvas = self
            .runner
            .try_borrow()
            .map_err(|_| JsValue::from_str("runner is already borrowed"))?
            .canvas
            .clone();

        let input = WebInput::new(canvas, prevent_default).map_err(|e| e.to_js_value())?;
        self.input = Some(input);
        Ok(())
    }

    #[wasm_bindgen(js_name = removeInputListeners)]
    pub fn remove_input_listeners(&mut self) {
        self.input.take();
    }

    #[wasm_bindgen(js_name = drainInputEventsDebug)]
    pub fn drain_input_events_debug(&mut self) -> Array {
        let events = self
            .input
            .as_mut()
            .map(|i| i.take_events())
            .unwrap_or_default();
        events
            .into_iter()
            .map(|e| JsValue::from_str(&format!("{e:?}")))
            .collect()
    }

    #[wasm_bindgen(js_name = renderOnce)]
    pub fn render_once(&mut self) -> Result<(), JsValue> {
        self.runner
            .try_borrow_mut()
            .map_err(|_| JsValue::from_str("runner is already borrowed"))?
            .render_once()
            .map_err(|e| e.to_js_value())
    }

    #[wasm_bindgen(js_name = addFont)]
    pub fn add_font(&mut self, bytes: Uint8Array) -> Result<usize, JsValue> {
        let bytes = bytes.to_vec();
        let mut runner = self
            .runner
            .try_borrow_mut()
            .map_err(|_| JsValue::from_str("runner is already borrowed"))?;
        Ok(runner.add_fonts([bytes]))
    }

    #[wasm_bindgen(js_name = addFonts)]
    pub fn add_fonts(&mut self, fonts: Array) -> Result<usize, JsValue> {
        let mut bytes_vec: Vec<Vec<u8>> = Vec::with_capacity(fonts.length() as usize);
        for value in fonts.iter() {
            let bytes: Uint8Array = value
                .dyn_into()
                .map_err(|_| JsValue::from_str("fonts must be Uint8Array[]"))?;
            bytes_vec.push(bytes.to_vec());
        }

        let mut runner = self
            .runner
            .try_borrow_mut()
            .map_err(|_| JsValue::from_str("runner is already borrowed"))?;
        Ok(runner.add_fonts(bytes_vec))
    }

    #[wasm_bindgen(js_name = allFontNames)]
    pub fn all_font_names(&self) -> Result<Array, JsValue> {
        let runner = self
            .runner
            .try_borrow()
            .map_err(|_| JsValue::from_str("runner is already borrowed"))?;
        let names = runner.all_font_names();
        Ok(names.into_iter().map(JsValue::from).collect())
    }

    #[wasm_bindgen(js_name = setClearColorRgba)]
    pub fn set_clear_color_rgba(&mut self, r: f64, g: f64, b: f64, a: f64) -> Result<(), JsValue> {
        let mut runner = self
            .runner
            .try_borrow_mut()
            .map_err(|_| JsValue::from_str("runner is already borrowed"))?;
        runner.set_clear_color(ClearColor(wgpu::Color { r, g, b, a }));
        Ok(())
    }
}

impl WebRunnerHandle {
    /// Returns queued platform events since the last call.
    pub fn take_input_events(&mut self) -> Vec<Event> {
        self.input
            .as_mut()
            .map(|i| i.take_events())
            .unwrap_or_default()
    }
}

#[wasm_bindgen]
pub fn start_clear_loop(canvas_id: &str) -> Result<(), JsValue> {
    let canvas_id = canvas_id.to_string();

    spawn_local(async move {
        let Ok(runner) = WebRunner::from_canvas_id(&canvas_id).await else {
            return;
        };

        let Ok(handle) = start_raf_loop(runner) else {
            return;
        };
        std::mem::forget(handle);
    });

    Ok(())
}
