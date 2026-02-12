use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fret_core::{AppWindowId, Event, ExternalDropReadLimits, TimerToken};
use fret_runtime::{Effect, PlatformCapabilities};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::Event as WebSysEvent;
use web_sys::{Document, HtmlElement};

mod file_dialog;
mod ime;
mod timers;

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
    timers: HashMap<TimerToken, timers::WebTimer>,
    file_dialogs: Rc<RefCell<file_dialog::WebFileDialogState>>,
    ime_active_window: Option<AppWindowId>,
    ime: HashMap<AppWindowId, ime::WebImeBridge>,
    last_ime_cursor_area: HashMap<AppWindowId, fret_core::Rect>,
    ime_mounts: HashMap<AppWindowId, HtmlElement>,
    #[cfg(debug_assertions)]
    ime_debug: Rc<RefCell<ime::WebImeDebugState>>,
    waker: Option<WebWaker>,
}

impl Default for WebPlatformServices {
    fn default() -> Self {
        Self {
            queued_events: Rc::new(RefCell::new(Vec::new())),
            fired_timeouts: Rc::new(RefCell::new(Vec::new())),
            timers: HashMap::new(),
            file_dialogs: Rc::new(RefCell::new(file_dialog::WebFileDialogState::default())),
            ime_active_window: None,
            ime: HashMap::new(),
            last_ime_cursor_area: HashMap::new(),
            ime_mounts: HashMap::new(),
            #[cfg(debug_assertions)]
            ime_debug: Rc::new(RefCell::new(ime::WebImeDebugState::default())),
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

impl WebPlatformServices {
    /// Register a per-window DOM container used to mount the hidden IME textarea (ADR 0180).
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
        timers::collect_fired_timeouts(&self.queued_events, &self.fired_timeouts, &mut self.timers);
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
                            if let Some(bridge) = ime::WebImeBridge::ensure(
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
                            if let Some(bridge) = ime::WebImeBridge::ensure(
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
                Effect::ImeRequestVirtualKeyboard { window, visible } => {
                    if self.ime_active_window == Some(window)
                        && let Some(bridge) = self.ime.get_mut(&window)
                    {
                        bridge.request_virtual_keyboard(visible);
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
                    timers::set_timer(
                        token,
                        after,
                        repeat,
                        &self.fired_timeouts,
                        &self.waker,
                        &mut self.timers,
                    );
                }
                Effect::CancelTimer { token } => {
                    timers::cancel_timer(token, &mut self.timers);
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
                Effect::OpenUrl { url, target, rel } => {
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
                    if let Some(target) = target {
                        let features = rel.as_deref().unwrap_or_default();
                        let _ =
                            window.open_with_url_and_target_and_features(&url, &target, features);
                    } else {
                        let _ = window.open_with_url(&url);
                    }
                }
                Effect::FileDialogOpen { options, .. } => {
                    let caps = app
                        .global::<PlatformCapabilities>()
                        .cloned()
                        .unwrap_or_default();
                    if !caps.fs.file_dialogs {
                        continue;
                    }

                    file_dialog::open(
                        &options,
                        self.file_dialogs.clone(),
                        self.queued_events.clone(),
                        self.waker.clone(),
                    );
                }
                Effect::FileDialogReadAll { token, .. } => {
                    file_dialog::read_all(
                        token,
                        ExternalDropReadLimits {
                            max_total_bytes: 64 * 1024 * 1024,
                            max_file_bytes: 32 * 1024 * 1024,
                            max_files: 64,
                        },
                        self.file_dialogs.clone(),
                        self.queued_events.clone(),
                        self.waker.clone(),
                    );
                }
                Effect::FileDialogReadAllWithLimits { token, limits, .. } => {
                    let cap = ExternalDropReadLimits {
                        max_total_bytes: 64 * 1024 * 1024,
                        max_file_bytes: 32 * 1024 * 1024,
                        max_files: 64,
                    };
                    file_dialog::read_all(
                        token,
                        limits.capped_by(cap),
                        self.file_dialogs.clone(),
                        self.queued_events.clone(),
                        self.waker.clone(),
                    );
                }
                Effect::FileDialogRelease { token } => {
                    file_dialog::release(token, &self.file_dialogs);
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
}
