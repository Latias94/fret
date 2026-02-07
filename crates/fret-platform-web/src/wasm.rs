use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use fret_core::{
    AppWindowId, Event, ExternalDragFile, ExternalDropFileData, ExternalDropReadError,
    ExternalDropReadLimits, FileDialogDataEvent, FileDialogSelection, TimerToken,
};
use fret_runtime::{Effect, PlatformCapabilities};
use wasm_bindgen::JsCast as _;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    Document, Event as WebSysEvent, EventTarget, HtmlElement, HtmlInputElement,
    HtmlTextAreaElement, InputEvent, KeyboardEvent, Node,
};

use crate::ime_dom_state::{DomInputDisposition, WebImeDomState};

type WebChangeCallback = wasm_bindgen::closure::Closure<dyn FnMut(WebSysEvent)>;
type WebWaker = Rc<dyn Fn()>;

fn window() -> Option<web_sys::Window> {
    web_sys::window()
}

fn document() -> Option<Document> {
    window().and_then(|w| w.document())
}

/// Web-specific platform services for `fret-runtime::Effect`s that require browser APIs.
///
/// This crate intentionally does *not* implement input/event mapping; use `winit` for that.
pub struct WebPlatformServices {
    queued_events: Rc<RefCell<Vec<Event>>>,
    fired_timeouts: Rc<RefCell<Vec<TimerToken>>>,
    timers: HashMap<TimerToken, WebTimer>,
    file_dialogs: Rc<RefCell<WebFileDialogState>>,
    ime_active_window: Option<AppWindowId>,
    ime: HashMap<AppWindowId, WebImeBridge>,
    last_ime_cursor_area: HashMap<AppWindowId, fret_core::Rect>,
    ime_mounts: HashMap<AppWindowId, HtmlElement>,
    #[cfg(debug_assertions)]
    ime_debug: Rc<RefCell<WebImeDebugState>>,
    waker: Option<WebWaker>,
}

impl Default for WebPlatformServices {
    fn default() -> Self {
        Self {
            queued_events: Rc::new(RefCell::new(Vec::new())),
            fired_timeouts: Rc::new(RefCell::new(Vec::new())),
            timers: HashMap::new(),
            file_dialogs: Rc::new(RefCell::new(WebFileDialogState::default())),
            ime_active_window: None,
            ime: HashMap::new(),
            last_ime_cursor_area: HashMap::new(),
            ime_mounts: HashMap::new(),
            #[cfg(debug_assertions)]
            ime_debug: Rc::new(RefCell::new(WebImeDebugState::default())),
            waker: None,
        }
    }
}

impl std::fmt::Debug for WebPlatformServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebPlatformServices")
            .field("queued_events", &"<Rc<RefCell<Vec<Event>>>>")
            .field("fired_timeouts", &"<Rc<RefCell<Vec<TimerToken>>>>")
            .field("timers", &self.timers)
            .field("file_dialogs", &"<Rc<RefCell<WebFileDialogState>>>")
            .field("ime_active_window", &self.ime_active_window)
            .field("ime", &self.ime)
            .field("ime_mounts", &self.ime_mounts.len())
            .finish()
    }
}

#[derive(Debug)]
struct WebTimer {
    id: i32,
    repeat: Option<Duration>,
    callback: wasm_bindgen::closure::Closure<dyn FnMut()>,
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct WebImeDebugState {
    dirty: bool,
    snapshot: fret_core::input::WebImeBridgeDebugSnapshot,
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

struct WebImeBridge {
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
    fn ensure(
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
        // composition UI reliably across platforms (ADR 0195).
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
        };
        bridge.install_listeners();
        Some(bridge)
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
        // relying on the post-mutation `input` event for common typing paths (ADR 0195).
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
                match dom_state
                    .borrow_mut()
                    .beforeinput_disposition(input.is_composing())
                {
                    DomInputDisposition::IgnoreComposing => return,
                    DomInputDisposition::IgnoreSuppressed => {
                        // If a command path already handled the edit (or a composition commit was
                        // already emitted via `compositionend`), prevent the DOM mutation so we
                        // don't get a follow-up `input` that would double-insert (ADR 0195).
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
                if !input_type.starts_with("insert") {
                    return;
                }

                let data = input.data().unwrap_or_default();
                if data.is_empty() {
                    return;
                }

                if let Some(text) = sanitize_text_input(&data) {
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

    fn set_enabled(&mut self, enabled: bool) {
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

    fn set_cursor_area(&mut self, rect: fret_core::Rect) {
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
}

#[derive(Debug, Default)]
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

impl WebPlatformServices {
    /// Register a per-window DOM container used to mount the hidden IME textarea (ADR 0195).
    pub fn register_ime_mount(&mut self, window: AppWindowId, mount: HtmlElement) {
        self.ime_mounts.insert(window, mount);
    }

    pub fn unregister_ime_mount(&mut self, window: AppWindowId) {
        self.ime_mounts.remove(&window);
    }

    pub fn set_waker(&mut self, waker: impl Fn() + 'static) {
        self.waker = Some(Rc::new(waker));
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
        effects: impl IntoIterator<Item = Effect>,
    ) -> Vec<Effect>
    where
        H: fret_runtime::GlobalsHost,
    {
        let mut unhandled: Vec<Effect> = Vec::new();
        for effect in effects {
            match effect {
                Effect::ImeAllow { window, enabled } => {
                    let Some(document) = document() else {
                        continue;
                    };
                    if enabled {
                        if let Some(active) = self.ime_active_window
                            && active != window
                            && let Some(bridge) = self.ime.get_mut(&active)
                        {
                            bridge.set_enabled(false);
                        }
                        self.ime_active_window = Some(window);
                    } else if self.ime_active_window == Some(window) {
                        self.ime_active_window = None;
                    }

                    if !self.ime.contains_key(&window) {
                        let mount = self.ime_mounts.get(&window).cloned();
                        #[cfg(not(debug_assertions))]
                        {
                            if let Some(bridge) = WebImeBridge::ensure(
                                &document,
                                mount,
                                self.queued_events.clone(),
                                self.waker.clone(),
                            ) {
                                self.ime.insert(window, bridge);
                            }
                        }
                        #[cfg(debug_assertions)]
                        {
                            if let Some(bridge) = WebImeBridge::ensure(
                                &document,
                                mount,
                                self.queued_events.clone(),
                                self.waker.clone(),
                                self.ime_debug.clone(),
                            ) {
                                self.ime.insert(window, bridge);
                            }
                        }

                        if let Some(bridge) = self.ime.get_mut(&window)
                            && let Some(rect) = self.last_ime_cursor_area.get(&window).copied()
                        {
                            bridge.set_cursor_area(rect);
                        }
                    }

                    if let Some(bridge) = self.ime.get_mut(&window) {
                        bridge.set_enabled(enabled);
                    }
                }
                Effect::ImeSetCursorArea { window, rect } => {
                    self.last_ime_cursor_area.insert(window, rect);
                    #[cfg(debug_assertions)]
                    {
                        if self.ime_active_window.is_none() {
                            let mut st = self.ime_debug.borrow_mut();
                            st.snapshot.last_cursor_area = Some(rect);
                            st.snapshot.cursor_area_set_seen =
                                st.snapshot.cursor_area_set_seen.saturating_add(1);
                            st.dirty = true;
                        }
                    }

                    if self.ime_active_window == Some(window)
                        && let Some(bridge) = self.ime.get_mut(&window)
                    {
                        bridge.set_cursor_area(rect);
                    }
                }
                Effect::SetTimer {
                    token,
                    after,
                    repeat,
                    ..
                } => {
                    let Some(window) = window() else {
                        continue;
                    };

                    let queue = self.fired_timeouts.clone();
                    let wake = self.waker.clone();
                    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                        let _ = queue.try_borrow_mut().map(|mut q| q.push(token));
                        if let Some(wake) = wake.as_ref() {
                            wake();
                        }
                    })
                        as Box<dyn FnMut()>);

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
                Effect::CancelTimer { token } => {
                    let Some(window) = window() else {
                        continue;
                    };
                    let Some(timer) = self.timers.remove(&token) else {
                        continue;
                    };
                    window.clear_timeout_with_handle(timer.id);
                }
                Effect::ClipboardSetText { text } => {
                    let caps = app
                        .global::<PlatformCapabilities>()
                        .cloned()
                        .unwrap_or_default();
                    if !caps.clipboard.text {
                        continue;
                    }
                    let Some(window) = window() else {
                        continue;
                    };
                    let clipboard = window.navigator().clipboard();
                    let wake = self.waker.clone();
                    spawn_local(async move {
                        let _ = JsFuture::from(clipboard.write_text(&text)).await;
                        if let Some(wake) = wake.as_ref() {
                            wake();
                        }
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

                    let Some(window) = window() else {
                        self.queued_events
                            .borrow_mut()
                            .push(Event::ClipboardTextUnavailable { token });
                        continue;
                    };
                    let clipboard = window.navigator().clipboard();
                    let queue = self.queued_events.clone();
                    let wake = self.waker.clone();
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
                        if let Some(wake) = wake.as_ref() {
                            wake();
                        }
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
                    let Some(window) = window() else {
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

                    let Some(document) = document() else {
                        continue;
                    };
                    let Ok(el) = document.create_element("input") else {
                        continue;
                    };
                    let Ok(input) = el.dyn_into::<HtmlInputElement>() else {
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
                    let wake = self.waker.clone();

                    let callback_cell: Rc<RefCell<Option<WebChangeCallback>>> =
                        Rc::new(RefCell::new(None));
                    let callback_cell_for_cb = callback_cell.clone();

                    let on_change = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e| {
                        if let Some(parent) = input_node_for_cb.parent_node() {
                            let _ = parent.remove_child(&input_node_for_cb);
                        }

                        if let Ok(holder) = callback_cell_for_cb.try_borrow()
                            && let Some(cb) = holder.as_ref()
                        {
                            let _ = input_target_for_cb.remove_event_listener_with_callback(
                                "change",
                                cb.as_ref().unchecked_ref(),
                            );
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
                            if let Some(wake) = wake.as_ref() {
                                wake();
                            }
                            return;
                        }

                        let (token, files_meta) = {
                            let mut st = state.borrow_mut();
                            let token = st.allocate_token();
                            let files_meta = selected
                                .iter()
                                .map(|f| ExternalDragFile { name: f.name() })
                                .collect::<Vec<_>>();
                            st.selections.insert(token, selected);
                            (token, files_meta)
                        };

                        let selection = FileDialogSelection {
                            token,
                            files: files_meta,
                        };
                        let _ = queue
                            .try_borrow_mut()
                            .map(|mut q| q.push(Event::FileDialogSelection(selection)));
                        if let Some(wake) = wake.as_ref() {
                            wake();
                        }
                    })
                        as Box<dyn FnMut(WebSysEvent)>);

                    *callback_cell.borrow_mut() = Some(on_change);
                    if let Ok(holder) = callback_cell.try_borrow()
                        && let Some(cb) = holder.as_ref()
                    {
                        let _ = input_target.add_event_listener_with_callback(
                            "change",
                            cb.as_ref().unchecked_ref(),
                        );
                    }

                    input.click();
                }
                Effect::FileDialogReadAll { token, .. } => {
                    self.file_dialog_read_all(
                        token,
                        ExternalDropReadLimits {
                            max_total_bytes: 64 * 1024 * 1024,
                            max_file_bytes: 32 * 1024 * 1024,
                            max_files: 64,
                        },
                    );
                }
                Effect::FileDialogReadAllWithLimits { token, limits, .. } => {
                    let cap = ExternalDropReadLimits {
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

        #[cfg(debug_assertions)]
        {
            let dirty = self.ime_debug.borrow().dirty;
            if dirty {
                let snapshot = {
                    let mut st = self.ime_debug.borrow_mut();
                    st.dirty = false;
                    st.snapshot.clone()
                };
                app.set_global(snapshot);
            }
        }

        unhandled
    }

    fn file_dialog_read_all(
        &self,
        token: fret_runtime::FileDialogToken,
        limits: ExternalDropReadLimits,
    ) {
        let files = self.file_dialogs.borrow().selections.get(&token).cloned();
        let Some(files) = files else {
            return;
        };

        let queue = self.queued_events.clone();
        let wake = self.waker.clone();
        spawn_local(async move {
            let mut out_files: Vec<ExternalDropFileData> = Vec::new();
            let mut errors: Vec<ExternalDropReadError> = Vec::new();
            let mut total: u64 = 0;

            for file in files.into_iter().take(limits.max_files) {
                let name = file.name();
                if limits.max_file_bytes > 0 && (file.size() as u64) > limits.max_file_bytes {
                    errors.push(ExternalDropReadError {
                        name,
                        message: "file exceeds max_file_bytes".to_string(),
                    });
                    continue;
                }

                let promise = file.array_buffer();
                let Ok(buf) = JsFuture::from(promise).await else {
                    errors.push(ExternalDropReadError {
                        name,
                        message: "failed to read file array_buffer".to_string(),
                    });
                    continue;
                };
                let array = js_sys::Uint8Array::new(&buf);
                let bytes = array.to_vec();

                total = total.saturating_add(bytes.len() as u64);
                if limits.max_total_bytes > 0 && total > limits.max_total_bytes {
                    errors.push(ExternalDropReadError {
                        name,
                        message: "total exceeds max_total_bytes".to_string(),
                    });
                    break;
                }

                out_files.push(ExternalDropFileData { name, bytes });
            }

            let event = Event::FileDialogData(FileDialogDataEvent {
                token,
                files: out_files,
                errors,
            });
            let _ = queue.try_borrow_mut().map(|mut q| q.push(event));
            if let Some(wake) = wake.as_ref() {
                wake();
            }
        });
    }

    fn collect_fired_timeouts(&mut self) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let fired = std::mem::take(&mut *self.fired_timeouts.borrow_mut());
        for token in fired {
            let Some(timer) = self.timers.remove(&token) else {
                continue;
            };

            self.queued_events.borrow_mut().push(Event::Timer { token });
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

    fn ms(duration: Duration) -> i32 {
        let ms = duration.as_millis().min(i32::MAX as u128);
        i32::try_from(ms).unwrap_or(i32::MAX)
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
