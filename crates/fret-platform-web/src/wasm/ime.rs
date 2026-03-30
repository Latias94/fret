use std::cell::RefCell;
use std::rc::Rc;

use fret_core::Event;
use wasm_bindgen::JsCast as _;
use web_sys::{
    Document, Event as WebSysEvent, EventTarget, HtmlElement, HtmlTextAreaElement, InputEvent,
    KeyboardEvent, Node,
};

use crate::ime_dom_state::{
    DomControlIntent, DomInputDisposition, WebImeDomState, control_intent_for_beforeinput_type,
};

type WebChangeCallback = wasm_bindgen::closure::Closure<dyn FnMut(WebSysEvent)>;
type WebWaker = Rc<dyn Fn()>;

fn window() -> Option<web_sys::Window> {
    web_sys::window()
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
pub(super) struct WebImeDebugState {
    pub(super) dirty: bool,
    pub(super) snapshot: fret_core::input::WebImeBridgeDebugSnapshot,
}

#[cfg(debug_assertions)]
fn debug_truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect()
}

#[cfg(debug_assertions)]
fn debug_push_recent_event(debug: &Rc<RefCell<WebImeDebugState>>, event: impl AsRef<str>) {
    const MAX_EVENTS: usize = 24;
    const MAX_CHARS: usize = 160;

    let mut st = debug.borrow_mut();
    st.snapshot
        .recent_events
        .push(debug_truncate(event.as_ref(), MAX_CHARS));
    if st.snapshot.recent_events.len() > MAX_EVENTS {
        let excess = st.snapshot.recent_events.len().saturating_sub(MAX_EVENTS);
        st.snapshot.recent_events.drain(0..excess);
    }
    st.dirty = true;
}

#[cfg(debug_assertions)]
fn debug_update_textarea_metrics(
    textarea: &HtmlTextAreaElement,
    debug: &Rc<RefCell<WebImeDebugState>>,
) {
    let mut st = debug.borrow_mut();

    let (has_focus, active_tag) = textarea
        .owner_document()
        .and_then(|doc| doc.active_element())
        .map(|active: web_sys::Element| {
            let active_tag = active.tag_name();
            let textarea_node: Node = textarea.clone().unchecked_into();
            let active_node: Node = active.unchecked_into();
            let has_focus = active_node.is_same_node(Some(&textarea_node));
            (Some(has_focus), Some(active_tag))
        })
        .unwrap_or((None, None));
    st.snapshot.textarea_has_focus = has_focus;
    st.snapshot.active_element_tag = active_tag;

    let value = textarea.value();
    st.snapshot.textarea_value_chars = Some(value.chars().count());
    st.snapshot.textarea_selection_start_utf16 = textarea.selection_start().ok().flatten();
    st.snapshot.textarea_selection_end_utf16 = textarea.selection_end().ok().flatten();
    st.snapshot.textarea_client_width_px = Some(textarea.client_width());
    st.snapshot.textarea_client_height_px = Some(textarea.client_height());
    st.snapshot.textarea_scroll_width_px = Some(textarea.scroll_width());
    st.snapshot.textarea_scroll_height_px = Some(textarea.scroll_height());

    st.dirty = true;
}

#[cfg(debug_assertions)]
fn ime_console_debug_enabled() -> bool {
    let Some(win) = window() else {
        return false;
    };

    let key = wasm_bindgen::JsValue::from_str("__FRET_IME_DEBUG");
    if let Ok(v) = js_sys::Reflect::get(&win, &key) {
        if v.as_bool().unwrap_or(false) {
            return true;
        }
        if let Some(s) = v.as_string() {
            if s == "1" || s.eq_ignore_ascii_case("true") {
                return true;
            }
        }
    }

    // Avoid requiring `web-sys`'s `Location` feature: read `window.location.search` via `Reflect`.
    let search = js_sys::Reflect::get(&win, &wasm_bindgen::JsValue::from_str("location"))
        .ok()
        .and_then(|loc| js_sys::Reflect::get(&loc, &wasm_bindgen::JsValue::from_str("search")).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    search.contains("ime_debug=1") || search.contains("fret_ime_debug=1")
}

#[cfg(debug_assertions)]
fn ime_console_log(msg: impl AsRef<str>) {
    if !ime_console_debug_enabled() {
        return;
    }
    // Avoid requiring `web-sys`'s `console` feature: call `globalThis.console.log` via `Reflect`.
    let global = js_sys::global();
    let console = js_sys::Reflect::get(&global, &wasm_bindgen::JsValue::from_str("console"));
    let Ok(console) = console else {
        return;
    };
    let log = js_sys::Reflect::get(&console, &wasm_bindgen::JsValue::from_str("log"));
    let Ok(log) = log else {
        return;
    };
    let Ok(log) = log.dyn_into::<js_sys::Function>() else {
        return;
    };
    let _ = log.call1(&console, &wasm_bindgen::JsValue::from_str(msg.as_ref()));
}

#[cfg(debug_assertions)]
const IME_DEBUG_INJECT_BEFOREINPUT_KEY: &str = "__FRET_IME_DEBUG_INJECT_BEFOREINPUT";

#[cfg(debug_assertions)]
fn set_ime_debug_beforeinput_injector(
    injector: Option<&wasm_bindgen::closure::Closure<dyn FnMut(wasm_bindgen::JsValue)>>,
) {
    let Some(win) = window() else {
        return;
    };
    let key = wasm_bindgen::JsValue::from_str(IME_DEBUG_INJECT_BEFOREINPUT_KEY);
    match injector {
        Some(injector) => {
            let _ = js_sys::Reflect::set(&win, &key, injector.as_ref().unchecked_ref());
        }
        None => {
            let _ = js_sys::Reflect::delete_property(&win, &key);
        }
    }
}

pub(super) struct WebImeBridge {
    textarea: HtmlTextAreaElement,
    position_mode: WebImePositionMode,
    enabled: bool,
    dom_state: Rc<RefCell<WebImeDomState>>,
    queued_events: Rc<RefCell<Vec<Event>>>,
    waker: Option<WebWaker>,
    listeners: Vec<(String, WebChangeCallback)>,
    last_cursor_area: Option<fret_core::Rect>,
    #[cfg(debug_assertions)]
    debug: Rc<RefCell<WebImeDebugState>>,
    #[cfg(debug_assertions)]
    cursor_overlay: Option<HtmlElement>,
    #[cfg(debug_assertions)]
    debug_inject_beforeinput:
        Option<wasm_bindgen::closure::Closure<dyn FnMut(wasm_bindgen::JsValue)>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebImePositionMode {
    Fixed,
    Absolute,
}

impl std::fmt::Debug for WebImeBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dom_state = self.dom_state.borrow();
        f.debug_struct("WebImeBridge")
            .field("enabled", &self.enabled)
            .field("composing", &dom_state.composing())
            .field("suppress_next_input", &dom_state.suppress_next_input())
            .field("listeners", &self.listeners.len())
            .field("last_cursor_area", &self.last_cursor_area)
            .field("position_mode", &self.position_mode)
            .finish()
    }
}

impl WebImeBridge {
    pub(super) fn ensure(
        document: &Document,
        mount: Option<HtmlElement>,
        queued_events: Rc<RefCell<Vec<Event>>>,
        waker: Option<WebWaker>,
        #[cfg(debug_assertions)] debug: Rc<RefCell<WebImeDebugState>>,
    ) -> Option<Self> {
        #[cfg(debug_assertions)]
        let mount_kind: Option<&'static str> = mount.as_ref().map(|m| {
            if m.get_attribute("data-fret-ime-overlay").as_deref() == Some("1") {
                "overlay"
            } else {
                "mount"
            }
        });

        let Ok(el) = document.create_element("textarea") else {
            return None;
        };
        let Ok(textarea) = el.dyn_into::<HtmlTextAreaElement>() else {
            return None;
        };

        textarea.set_spellcheck(false);
        textarea.set_value("");
        textarea.set_tab_index(-1);
        textarea.set_rows(1);
        // Make the textarea extremely wide to reduce the chance of internal line wrapping during
        // IME composition updates (candidate UI jitter).
        textarea.set_cols(4096);
        textarea.set_wrap("off");

        let textarea_el: HtmlElement = textarea.clone().unchecked_into();
        let _ = textarea_el.set_attribute("autocapitalize", "off");
        let _ = textarea_el.set_attribute("autocomplete", "off");
        let _ = textarea_el.set_attribute("autocorrect", "off");
        let style = textarea_el.style();
        let position_mode = if mount.is_some() {
            let _ = style.set_property("position", "absolute");
            WebImePositionMode::Absolute
        } else {
            let _ = style.set_property("position", "fixed");
            WebImePositionMode::Fixed
        };
        let _ = style.set_property("left", "0px");
        let _ = style.set_property("top", "0px");
        // Keep the element effectively invisible but still "layout-real" so browser IME can anchor
        // composition UI reliably across platforms (ADR 0180).
        let _ = style.set_property("opacity", "0.001");
        // Avoid line wrapping during composition updates; some IMEs anchor their candidate UI to the
        // textarea caret position, so wrapping causes vertical jitter as the preedit string grows.
        let _ = style.set_property("width", "20000px");
        let _ = style.set_property("height", "20px");
        let _ = style.set_property("margin", "0");
        let _ = style.set_property("padding", "0");
        let _ = style.set_property("border", "0");
        let _ = style.set_property("outline", "none");
        let _ = style.set_property("resize", "none");
        let _ = style.set_property("overflow", "hidden");
        let _ = style.set_property("white-space", "pre");
        let _ = style.set_property("overflow-wrap", "normal");
        let _ = style.set_property("word-break", "keep-all");
        let _ = style.set_property("background", "transparent");
        let _ = style.set_property("color", "transparent");
        let _ = style.set_property("caret-color", "transparent");
        let _ = style.set_property("font-size", "16px");
        let _ = style.set_property("line-height", "20px");
        let _ = style.set_property("pointer-events", "none");
        let _ = style.set_property("z-index", "2147483647");
        let _ = textarea_el.set_attribute("aria-hidden", "true");

        #[cfg(debug_assertions)]
        let mut cursor_overlay: Option<HtmlElement> = None;

        if let Some(mount) = mount {
            // Only mutate inline styles for mounts we own.
            if mount.get_attribute("data-fret-ime-mount").as_deref() == Some("1") {
                let mstyle = mount.style();
                // If the runner provides a dedicated overlay element, keep it as an absolutely
                // positioned layer (sized to the canvas wrapper). Otherwise fall back to the older
                // "parent is the mount" strategy.
                if mount.get_attribute("data-fret-ime-overlay").as_deref() == Some("1") {
                    let _ = mstyle.set_property("position", "absolute");
                    let _ = mstyle.set_property("left", "0");
                    let _ = mstyle.set_property("top", "0");
                    let _ = mstyle.set_property("width", "100%");
                    let _ = mstyle.set_property("height", "100%");
                    let _ = mstyle.set_property("pointer-events", "none");
                    let _ = mstyle.set_property("overflow", "hidden");
                } else {
                    let _ = mstyle.set_property("position", "relative");
                    let _ = mstyle.set_property("margin", "0");
                    let _ = mstyle.set_property("padding", "0");
                    let _ = mstyle.set_property("border", "0");
                    let _ = mstyle.set_property("overflow", "hidden");
                }
            }
            let _ = mount.append_child(&textarea_el);

            #[cfg(debug_assertions)]
            {
                cursor_overlay = Self::ensure_cursor_overlay(document, Some(mount), position_mode);
            }
        } else if let Some(body) = document.body() {
            let _ = body.append_child(&textarea_el);

            #[cfg(debug_assertions)]
            {
                cursor_overlay = Self::ensure_cursor_overlay(document, None, position_mode);
            }
        }

        let dom_state = Rc::new(RefCell::new(WebImeDomState::default()));

        #[cfg(debug_assertions)]
        {
            let mut st = debug.borrow_mut();
            st.snapshot.enabled = false;
            st.snapshot.composing = false;
            st.snapshot.suppress_next_input = false;
            st.snapshot.position_mode = Some(
                match position_mode {
                    WebImePositionMode::Absolute => "absolute",
                    WebImePositionMode::Fixed => "fixed",
                }
                .to_string(),
            );
            st.snapshot.mount_kind = mount_kind
                .map(|v| v.to_string())
                .or_else(|| document.body().is_some().then_some("body".to_string()));
            st.snapshot.device_pixel_ratio =
                document.default_view().map(|v| v.device_pixel_ratio());
            st.dirty = true;
        }

        #[cfg(debug_assertions)]
        debug_update_textarea_metrics(&textarea, &debug);

        #[cfg(debug_assertions)]
        let debug_inject_beforeinput = Some(Self::make_debug_beforeinput_injector(
            textarea.clone(),
            dom_state.clone(),
            queued_events.clone(),
            waker.clone(),
            debug.clone(),
        ));

        let mut bridge = Self {
            textarea,
            position_mode,
            enabled: false,
            dom_state,
            queued_events,
            waker,
            listeners: Vec::new(),
            last_cursor_area: None,
            #[cfg(debug_assertions)]
            debug,
            #[cfg(debug_assertions)]
            cursor_overlay,
            #[cfg(debug_assertions)]
            debug_inject_beforeinput,
        };
        bridge.install_listeners();
        Some(bridge)
    }

    #[cfg(debug_assertions)]
    fn make_debug_beforeinput_injector(
        textarea: HtmlTextAreaElement,
        dom_state: Rc<RefCell<WebImeDomState>>,
        queued_events: Rc<RefCell<Vec<Event>>>,
        waker: Option<WebWaker>,
        debug: Rc<RefCell<WebImeDebugState>>,
    ) -> wasm_bindgen::closure::Closure<dyn FnMut(wasm_bindgen::JsValue)> {
        wasm_bindgen::closure::Closure::wrap(Box::new(move |value: wasm_bindgen::JsValue| {
            let Some(input_type) = value.as_string() else {
                return;
            };
            let Some(intent) = control_intent_for_beforeinput_type(input_type.as_str()) else {
                return;
            };

            textarea.set_value("");
            {
                let mut st = debug.borrow_mut();
                st.snapshot.beforeinput_seen = st.snapshot.beforeinput_seen.saturating_add(1);
                st.snapshot.last_input_type = Some(input_type.clone());
                st.snapshot.last_beforeinput_data = None;
                st.snapshot.composing = dom_state.borrow().composing();
                st.snapshot.suppress_next_input = dom_state.borrow().suppress_next_input();
                st.dirty = true;
            }
            debug_push_recent_event(
                &debug,
                format!("debug_beforeinput type={:?} synthetic=1", input_type),
            );

            queued_events.borrow_mut().push(Event::KeyDown {
                key: Self::key_code_for_control_intent(intent),
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            });
            if let Some(wake) = waker.as_ref() {
                wake();
            }

            debug_update_textarea_metrics(&textarea, &debug);
        }) as Box<dyn FnMut(wasm_bindgen::JsValue)>)
    }

    #[cfg(debug_assertions)]
    fn ensure_cursor_overlay(
        document: &Document,
        mount: Option<HtmlElement>,
        position_mode: WebImePositionMode,
    ) -> Option<HtmlElement> {
        let Ok(el) = document.create_element("div") else {
            return None;
        };
        let Ok(overlay) = el.dyn_into::<HtmlElement>() else {
            return None;
        };

        let _ = overlay.set_attribute("data-fret-ime-cursor-overlay", "1");
        let style = overlay.style();
        let _ = style.set_property(
            "position",
            match position_mode {
                WebImePositionMode::Absolute => "absolute",
                WebImePositionMode::Fixed => "fixed",
            },
        );
        let _ = style.set_property("left", "0px");
        let _ = style.set_property("top", "0px");
        let _ = style.set_property("width", "0px");
        let _ = style.set_property("height", "0px");
        let _ = style.set_property("pointer-events", "none");
        let _ = style.set_property("box-sizing", "border-box");
        let _ = style.set_property("outline", "1px solid rgba(255, 0, 0, 0.65)");
        let _ = style.set_property("background", "rgba(255, 0, 0, 0.08)");
        let _ = style.set_property("z-index", "2147483646");
        let _ = style.set_property("display", "none");

        if let Some(mount) = mount {
            let _ = mount.append_child(&overlay);
        } else if let Some(body) = document.body() {
            let _ = body.append_child(&overlay);
        } else {
            return None;
        }

        Some(overlay)
    }

    fn wake(&self) {
        if let Some(wake) = self.waker.as_ref() {
            wake();
        }
    }

    fn push_event(&self, event: Event) {
        self.queued_events.borrow_mut().push(event);
        self.wake();
    }

    fn key_code_for_control_intent(intent: DomControlIntent) -> fret_core::KeyCode {
        match intent {
            DomControlIntent::DeleteBackward => fret_core::KeyCode::Backspace,
            DomControlIntent::DeleteForward => fret_core::KeyCode::Delete,
            DomControlIntent::Enter => fret_core::KeyCode::Enter,
        }
    }

    fn control_intent_for_key(key: fret_core::KeyCode) -> Option<DomControlIntent> {
        match key {
            fret_core::KeyCode::Backspace => Some(DomControlIntent::DeleteBackward),
            fret_core::KeyCode::Delete => Some(DomControlIntent::DeleteForward),
            fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter => {
                Some(DomControlIntent::Enter)
            }
            _ => None,
        }
    }

    fn install_listeners(&mut self) {
        let target: EventTarget = self.textarea.clone().unchecked_into();

        // Key events: needed because the textarea becomes the focused element while IME is enabled.
        {
            let textarea = self.textarea.clone();
            let dom_state = self.dom_state.clone();
            let queue = self.queued_events.clone();
            let wake = self.waker.clone();
            #[cfg(debug_assertions)]
            let debug = self.debug.clone();
            let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: WebSysEvent| {
                let Ok(k) = e.dyn_into::<KeyboardEvent>() else {
                    return;
                };

                // Keep focus in the UI runtime; do not let the browser tab away.
                if k.key() == "Tab" {
                    k.prevent_default();
                }

                let alt_gr = k.get_modifier_state("AltGraph");
                let mut modifiers = fret_core::Modifiers {
                    shift: k.shift_key(),
                    ctrl: k.ctrl_key(),
                    alt: k.alt_key(),
                    alt_gr,
                    meta: k.meta_key(),
                };
                if modifiers.alt_gr {
                    modifiers.ctrl = false;
                    modifiers.alt = false;
                }

                let key = k
                    .code()
                    .parse::<fret_core::KeyCode>()
                    .unwrap_or(fret_core::KeyCode::Unidentified);

                #[cfg(debug_assertions)]
                {
                    let mut st = debug.borrow_mut();
                    st.snapshot.last_key_code = Some(key);
                    st.dirty = true;
                }

                // When IME is enabled we route editor shortcuts through the UI runtime. Prevent
                // the browser from applying default text editing to the hidden textarea (notably
                // paste), which would otherwise produce extra `input` events.
                if (modifiers.ctrl || modifiers.meta)
                    && matches!(
                        key,
                        fret_core::KeyCode::KeyA
                            | fret_core::KeyCode::KeyC
                            | fret_core::KeyCode::KeyV
                            | fret_core::KeyCode::KeyX
                            | fret_core::KeyCode::KeyY
                            | fret_core::KeyCode::KeyZ
                    )
                {
                    k.prevent_default();
                    dom_state.borrow_mut().on_shortcut_suppressed();
                    textarea.set_value("");
                    #[cfg(debug_assertions)]
                    {
                        let mut st = debug.borrow_mut();
                        st.snapshot.suppress_next_input = true;
                        st.dirty = true;
                    }
                }

                if !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.meta
                    && let Some(intent) = Self::control_intent_for_key(key)
                {
                    dom_state.borrow_mut().on_control_keydown(intent);
                }

                let event = Event::KeyDown {
                    key,
                    modifiers,
                    repeat: k.repeat(),
                };
                queue.borrow_mut().push(event);
                if let Some(wake) = wake.as_ref() {
                    wake();
                }
            })
                as Box<dyn FnMut(WebSysEvent)>);
            let _ = target.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
            self.listeners.push(("keydown".to_string(), cb));
        }

        {
            let dom_state = self.dom_state.clone();
            let queue = self.queued_events.clone();
            let wake = self.waker.clone();
            #[cfg(debug_assertions)]
            let debug = self.debug.clone();
            let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: WebSysEvent| {
                let Ok(k) = e.dyn_into::<KeyboardEvent>() else {
                    return;
                };

                let alt_gr = k.get_modifier_state("AltGraph");
                let mut modifiers = fret_core::Modifiers {
                    shift: k.shift_key(),
                    ctrl: k.ctrl_key(),
                    alt: k.alt_key(),
                    alt_gr,
                    meta: k.meta_key(),
                };
                if modifiers.alt_gr {
                    modifiers.ctrl = false;
                    modifiers.alt = false;
                }

                let key = k
                    .code()
                    .parse::<fret_core::KeyCode>()
                    .unwrap_or(fret_core::KeyCode::Unidentified);

                #[cfg(debug_assertions)]
                {
                    let mut st = debug.borrow_mut();
                    st.snapshot.last_key_code = Some(key);
                    st.dirty = true;
                }

                if let Some(intent) = Self::control_intent_for_key(key) {
                    dom_state.borrow_mut().on_control_keyup(intent);
                }

                let event = Event::KeyUp { key, modifiers };
                queue.borrow_mut().push(event);
                if let Some(wake) = wake.as_ref() {
                    wake();
                }
            })
                as Box<dyn FnMut(WebSysEvent)>);
            let _ = target.add_event_listener_with_callback("keyup", cb.as_ref().unchecked_ref());
            self.listeners.push(("keyup".to_string(), cb));
        }

        // Composition events → `Event::Ime`.
        {
            let dom_state = self.dom_state.clone();
            #[cfg(debug_assertions)]
            let debug = self.debug.clone();
            #[cfg(debug_assertions)]
            let textarea = self.textarea.clone();
            let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: WebSysEvent| {
                dom_state.borrow_mut().on_composition_start();
                #[cfg(debug_assertions)]
                {
                    let mut st = debug.borrow_mut();
                    st.snapshot.composing = true;
                    st.snapshot.composition_start_seen =
                        st.snapshot.composition_start_seen.saturating_add(1);
                    st.dirty = true;
                }
                #[cfg(debug_assertions)]
                debug_update_textarea_metrics(&textarea, &debug);
                #[cfg(debug_assertions)]
                debug_push_recent_event(&debug, "compositionstart");
            })
                as Box<dyn FnMut(WebSysEvent)>);
            let _ = target
                .add_event_listener_with_callback("compositionstart", cb.as_ref().unchecked_ref());
            self.listeners.push(("compositionstart".to_string(), cb));
        }

        {
            let textarea = self.textarea.clone();
            let dom_state = self.dom_state.clone();
            let queue = self.queued_events.clone();
            let wake = self.waker.clone();
            #[cfg(debug_assertions)]
            let debug = self.debug.clone();
            let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: WebSysEvent| {
                // Some browsers may fire update without start; treat as composing.
                dom_state.borrow_mut().on_composition_update();
                let text = textarea.value();
                let cursor = textarea
                    .selection_start()
                    .ok()
                    .flatten()
                    .zip(textarea.selection_end().ok().flatten())
                    .map(|(s, e)| {
                        let (start, end) = fret_core::utf::utf16_range_to_utf8_byte_range(
                            text.as_str(),
                            s as usize,
                            e as usize,
                        );
                        (start, end)
                    });

                queue
                    .borrow_mut()
                    .push(Event::Ime(fret_core::ImeEvent::Preedit { text, cursor }));
                if let Some(wake) = wake.as_ref() {
                    wake();
                }

                #[cfg(debug_assertions)]
                {
                    let mut st = debug.borrow_mut();
                    st.snapshot.composing = true;
                    st.snapshot.last_preedit_text = {
                        let text = textarea.value();
                        (!text.is_empty()).then(|| debug_truncate(text.as_str(), 64))
                    };
                    st.snapshot.last_preedit_cursor_utf16 = textarea
                        .selection_start()
                        .ok()
                        .flatten()
                        .zip(textarea.selection_end().ok().flatten())
                        .map(|(s, e)| (s, e));
                    st.snapshot.composition_update_seen =
                        st.snapshot.composition_update_seen.saturating_add(1);
                    st.dirty = true;
                }
                #[cfg(debug_assertions)]
                debug_update_textarea_metrics(&textarea, &debug);
                #[cfg(debug_assertions)]
                debug_push_recent_event(
                    &debug,
                    format!(
                        "compositionupdate preedit_chars={} sel_utf16={:?}..{:?}",
                        textarea.value().chars().count(),
                        textarea.selection_start().ok().flatten(),
                        textarea.selection_end().ok().flatten(),
                    ),
                );
            })
                as Box<dyn FnMut(WebSysEvent)>);
            let _ = target
                .add_event_listener_with_callback("compositionupdate", cb.as_ref().unchecked_ref());
            self.listeners.push(("compositionupdate".to_string(), cb));
        }

        {
            let textarea = self.textarea.clone();
            let dom_state = self.dom_state.clone();
            let queue = self.queued_events.clone();
            let wake = self.waker.clone();
            #[cfg(debug_assertions)]
            let debug = self.debug.clone();
            let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: WebSysEvent| {
                dom_state.borrow_mut().on_composition_end();

                let text = textarea.value();
                textarea.set_value("");

                let committed = sanitize_text_input(&text);
                if let Some(committed) = committed.clone() {
                    queue
                        .borrow_mut()
                        .push(Event::Ime(fret_core::ImeEvent::Commit(committed)));
                }
                queue
                    .borrow_mut()
                    .push(Event::Ime(fret_core::ImeEvent::Preedit {
                        text: String::new(),
                        cursor: None,
                    }));
                if let Some(wake) = wake.as_ref() {
                    wake();
                }

                #[cfg(debug_assertions)]
                {
                    let mut st = debug.borrow_mut();
                    st.snapshot.composing = false;
                    st.snapshot.suppress_next_input = true;
                    st.snapshot.last_commit_text =
                        committed.as_deref().map(|s| debug_truncate(s, 64));
                    st.snapshot.composition_end_seen =
                        st.snapshot.composition_end_seen.saturating_add(1);
                    st.dirty = true;
                }
                #[cfg(debug_assertions)]
                debug_update_textarea_metrics(&textarea, &debug);
                #[cfg(debug_assertions)]
                debug_push_recent_event(
                    &debug,
                    format!(
                        "compositionend commit={:?} suppress_next_input=1",
                        committed.as_deref()
                    ),
                );
            })
                as Box<dyn FnMut(WebSysEvent)>);
            let _ = target
                .add_event_listener_with_callback("compositionend", cb.as_ref().unchecked_ref());
            self.listeners.push(("compositionend".to_string(), cb));
        }

        // Input events → `Event::TextInput` for committed insertion.
        {
            let textarea = self.textarea.clone();
            let dom_state = self.dom_state.clone();
            let queue = self.queued_events.clone();
            let wake = self.waker.clone();
            #[cfg(debug_assertions)]
            let debug = self.debug.clone();
            let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: WebSysEvent| {
                let Ok(input) = e.dyn_into::<InputEvent>() else {
                    return;
                };

                match dom_state.borrow_mut().input_disposition() {
                    DomInputDisposition::IgnoreComposing => return,
                    DomInputDisposition::IgnoreSuppressed => {
                        textarea.set_value("");
                        #[cfg(debug_assertions)]
                        {
                            let mut st = debug.borrow_mut();
                            st.snapshot.suppress_next_input = false;
                            st.snapshot.suppressed_input_seen =
                                st.snapshot.suppressed_input_seen.saturating_add(1);
                            st.dirty = true;
                        }
                        #[cfg(debug_assertions)]
                        debug_update_textarea_metrics(&textarea, &debug);
                        return;
                    }
                    DomInputDisposition::Process => {}
                }

                #[cfg(debug_assertions)]
                {
                    let mut st = debug.borrow_mut();
                    st.snapshot.input_seen = st.snapshot.input_seen.saturating_add(1);
                    st.snapshot.last_input_type = Some(input.input_type());
                    let data = input.data().unwrap_or_default();
                    st.snapshot.last_input_data =
                        (!data.is_empty()).then(|| debug_truncate(&data, 64));
                    let commit = input.data().unwrap_or_default();
                    if !commit.is_empty() {
                        st.snapshot.last_commit_text = Some(debug_truncate(&commit, 64));
                    }
                    st.dirty = true;
                }

                // Prefer the explicit data payload; fall back to reading the textarea value.
                let mut text = input.data().unwrap_or_default();
                if text.is_empty() {
                    text = textarea.value();
                }
                textarea.set_value("");

                if let Some(text) = sanitize_text_input(&text) {
                    queue.borrow_mut().push(Event::TextInput(text));
                    if let Some(wake) = wake.as_ref() {
                        wake();
                    }
                }

                #[cfg(debug_assertions)]
                debug_update_textarea_metrics(&textarea, &debug);
                #[cfg(debug_assertions)]
                debug_push_recent_event(
                    &debug,
                    format!(
                        "input type={:?} data={:?}",
                        input.input_type(),
                        input.data().unwrap_or_default()
                    ),
                );
            })
                as Box<dyn FnMut(WebSysEvent)>);
            let _ = target.add_event_listener_with_callback("input", cb.as_ref().unchecked_ref());
            self.listeners.push(("input".to_string(), cb));
        }

        // Prefer `beforeinput` for simple insertions so we can keep the textarea empty and avoid
        // relying on the post-mutation `input` event for common typing paths (ADR 0180).
        {
            let textarea = self.textarea.clone();
            let dom_state = self.dom_state.clone();
            let queue = self.queued_events.clone();
            let wake = self.waker.clone();
            #[cfg(debug_assertions)]
            let debug = self.debug.clone();
            let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: WebSysEvent| {
                let Ok(input) = e.dyn_into::<InputEvent>() else {
                    return;
                };

                let disposition = dom_state
                    .borrow_mut()
                    .beforeinput_disposition(input.is_composing());
                match disposition {
                    DomInputDisposition::IgnoreComposing => return,
                    DomInputDisposition::IgnoreSuppressed => {
                        // If a command path already handled the edit (or a composition commit was
                        // already emitted via `compositionend`), prevent the DOM mutation so we
                        // don't get a follow-up `input` that would double-insert (ADR 0180).
                        dom_state.borrow_mut().on_beforeinput_handled();
                        input.prevent_default();
                        textarea.set_value("");
                        #[cfg(debug_assertions)]
                        {
                            let mut st = debug.borrow_mut();
                            st.snapshot.suppress_next_input = false;
                            st.snapshot.suppressed_input_seen =
                                st.snapshot.suppressed_input_seen.saturating_add(1);
                            st.dirty = true;
                        }
                        #[cfg(debug_assertions)]
                        debug_update_textarea_metrics(&textarea, &debug);
                        return;
                    }
                    DomInputDisposition::Process => {}
                }

                let input_type = input.input_type();
                #[cfg(debug_assertions)]
                {
                    let mut st = debug.borrow_mut();
                    st.snapshot.beforeinput_seen = st.snapshot.beforeinput_seen.saturating_add(1);
                    st.snapshot.last_input_type = Some(input_type.clone());
                    let data = input.data().unwrap_or_default();
                    st.snapshot.last_beforeinput_data =
                        (!data.is_empty()).then(|| debug_truncate(&data, 64));
                    st.dirty = true;
                }
                #[cfg(debug_assertions)]
                debug_push_recent_event(
                    &debug,
                    format!(
                        "beforeinput type={:?} composing={} data={:?}",
                        input_type,
                        input.is_composing() as u8,
                        input.data().unwrap_or_default()
                    ),
                );

                if let Some(intent) = control_intent_for_beforeinput_type(input_type.as_str()) {
                    let mut dom_state = dom_state.borrow_mut();
                    dom_state.on_beforeinput_handled();
                    input.prevent_default();
                    textarea.set_value("");

                    if dom_state.suppress_matching_control_beforeinput(intent) {
                        #[cfg(debug_assertions)]
                        {
                            let mut st = debug.borrow_mut();
                            st.snapshot.suppressed_input_seen =
                                st.snapshot.suppressed_input_seen.saturating_add(1);
                            st.dirty = true;
                        }
                        #[cfg(debug_assertions)]
                        debug_update_textarea_metrics(&textarea, &debug);
                        return;
                    }

                    drop(dom_state);
                    queue.borrow_mut().push(Event::KeyDown {
                        key: Self::key_code_for_control_intent(intent),
                        modifiers: fret_core::Modifiers::default(),
                        repeat: false,
                    });
                    if let Some(wake) = wake.as_ref() {
                        wake();
                    }

                    #[cfg(debug_assertions)]
                    debug_update_textarea_metrics(&textarea, &debug);
                    return;
                }

                if !input_type.starts_with("insert") {
                    return;
                }

                let data = input.data().unwrap_or_default();
                if data.is_empty() {
                    return;
                }

                if let Some(text) = sanitize_text_input(&data) {
                    dom_state.borrow_mut().on_beforeinput_handled();
                    input.prevent_default();
                    textarea.set_value("");
                    queue.borrow_mut().push(Event::TextInput(text));
                    if let Some(wake) = wake.as_ref() {
                        wake();
                    }
                }

                #[cfg(debug_assertions)]
                debug_update_textarea_metrics(&textarea, &debug);
            })
                as Box<dyn FnMut(WebSysEvent)>);
            let _ =
                target.add_event_listener_with_callback("beforeinput", cb.as_ref().unchecked_ref());
            self.listeners.push(("beforeinput".to_string(), cb));
        }
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;

        #[cfg(debug_assertions)]
        {
            let dom_state = self.dom_state.borrow();
            let mut st = self.debug.borrow_mut();
            st.snapshot.enabled = enabled;
            st.snapshot.composing = dom_state.composing();
            st.snapshot.suppress_next_input = dom_state.suppress_next_input();
            st.dirty = true;
        }
        #[cfg(debug_assertions)]
        debug_update_textarea_metrics(&self.textarea, &self.debug);
        #[cfg(debug_assertions)]
        debug_push_recent_event(&self.debug, format!("ime_allow enabled={}", enabled as u8));
        #[cfg(debug_assertions)]
        set_ime_debug_beforeinput_injector(
            enabled
                .then_some(self.debug_inject_beforeinput.as_ref())
                .flatten(),
        );

        if enabled {
            let focus_result = self.textarea.focus();
            #[cfg(debug_assertions)]
            {
                if let Err(err) = &focus_result {
                    ime_console_log(format!("ime_allow enabled=1 focus_err={err:?}"));
                } else {
                    ime_console_log("ime_allow enabled=1 focus_ok");
                }
                debug_update_textarea_metrics(&self.textarea, &self.debug);
            }
            self.push_event(Event::Ime(fret_core::ImeEvent::Enabled));
            return;
        }

        let blur_result = self.textarea.blur();
        #[cfg(debug_assertions)]
        {
            if let Err(err) = &blur_result {
                ime_console_log(format!("ime_allow enabled=0 blur_err={err:?}"));
            } else {
                ime_console_log("ime_allow enabled=0 blur_ok");
            }
        }
        self.textarea.set_value("");
        self.dom_state.borrow_mut().on_ime_disabled();

        #[cfg(debug_assertions)]
        {
            let mut st = self.debug.borrow_mut();
            st.snapshot.composing = false;
            st.snapshot.suppress_next_input = false;
            st.dirty = true;
        }
        #[cfg(debug_assertions)]
        debug_update_textarea_metrics(&self.textarea, &self.debug);

        self.push_event(Event::Ime(fret_core::ImeEvent::Disabled));
    }

    pub(super) fn set_cursor_area(&mut self, rect: fret_core::Rect) {
        self.last_cursor_area = Some(rect);
        let anchor_x = rect.origin.x.0 + rect.size.width.0 * 0.5;
        let anchor_y = rect.origin.y.0 + rect.size.height.0 * 0.5;
        #[cfg(debug_assertions)]
        {
            let mut st = self.debug.borrow_mut();
            st.snapshot.last_cursor_area = Some(rect);
            st.snapshot.last_cursor_anchor_px = Some((anchor_x, anchor_y));
            st.snapshot.cursor_area_set_seen = st.snapshot.cursor_area_set_seen.saturating_add(1);
            st.snapshot.device_pixel_ratio = self
                .textarea
                .owner_document()
                .and_then(|d| d.default_view())
                .map(|v| v.device_pixel_ratio());
            st.dirty = true;
        }
        #[cfg(debug_assertions)]
        debug_update_textarea_metrics(&self.textarea, &self.debug);
        #[cfg(debug_assertions)]
        debug_push_recent_event(
            &self.debug,
            format!(
                "cursor_area_set x={} y={} w={} h={} anchor=({},{})",
                rect.origin.x.0,
                rect.origin.y.0,
                rect.size.width.0,
                rect.size.height.0,
                anchor_x,
                anchor_y
            ),
        );
        let textarea_el: HtmlElement = self.textarea.clone().unchecked_into();
        let style = textarea_el.style();
        // Anchor the textarea to the *center* of the caret rect to better match how browsers place
        // IME candidate/composition UI (similar to egui's web text agent).
        let left_px = anchor_x.max(0.0).round();
        let top_px = anchor_y.max(0.0).round();
        let _ = style.set_property("left", &format!("{left_px}px"));
        let _ = style.set_property("top", &format!("{top_px}px"));
        // Keep textarea line metrics roughly in sync with the caret height to avoid vertical drift
        // between the app caret and browser IME UI across fonts/zoom levels.
        let caret_h = rect.size.height.0.max(1.0);
        let height_px = caret_h.max(20.0).round();
        let font_px = caret_h.clamp(10.0, 48.0).round();
        let _ = style.set_property("height", &format!("{height_px}px"));
        let _ = style.set_property("font-size", &format!("{font_px}px"));
        let _ = style.set_property("line-height", &format!("{height_px}px"));

        #[cfg(debug_assertions)]
        ime_console_log(format!(
            "ime_cursor_area rect=({:.1},{:.1} {:.1}x{:.1}) anchor=({left_px:.0},{top_px:.0}) font_px={font_px:.0} height_px={height_px:.0}",
            rect.origin.x.0, rect.origin.y.0, rect.size.width.0, rect.size.height.0,
        ));

        #[cfg(debug_assertions)]
        if let Some(overlay) = self.cursor_overlay.as_ref() {
            let style = overlay.style();
            let _ = style.set_property("display", "block");
            let overlay_left_px = rect.origin.x.0.max(0.0).round();
            let overlay_top_px = rect.origin.y.0.max(0.0).round();
            let _ = style.set_property("left", &format!("{overlay_left_px}px"));
            let _ = style.set_property("top", &format!("{overlay_top_px}px"));
            let _ = style.set_property("width", &format!("{}px", rect.size.width.0.max(0.0)));
            let _ = style.set_property("height", &format!("{}px", rect.size.height.0.max(0.0)));
        }
    }

    pub(super) fn request_virtual_keyboard(&mut self, visible: bool) {
        // v1: best-effort "show keyboard" request for platforms that require an explicit call.
        // On the web this maps to a best-effort `focus()` retry. We intentionally do not map
        // "hide keyboard" to `blur()` here, as blurring would also disable further typing while
        // the runtime still believes a text input is focused.
        if !visible || !self.enabled {
            return;
        }

        let focus_result = self.textarea.focus();
        #[cfg(debug_assertions)]
        {
            if let Err(err) = &focus_result {
                ime_console_log(format!("ime_request_keyboard visible=1 focus_err={err:?}"));
            } else {
                ime_console_log("ime_request_keyboard visible=1 focus_ok");
            }
            debug_update_textarea_metrics(&self.textarea, &self.debug);
            debug_push_recent_event(&self.debug, "ime_request_keyboard visible=1");
        }
        let _ = focus_result;
    }
}

fn sanitize_text_input(text: &str) -> Option<String> {
    // Contract: `Event::TextInput` represents committed insertion text and must not include
    // control characters. Keys like Backspace/Enter/Tab must be handled via `KeyDown` + commands.
    let filtered: String = text.chars().filter(|ch| !ch.is_control()).collect();
    if filtered.is_empty() {
        None
    } else {
        Some(filtered)
    }
}
