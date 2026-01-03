use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use fret_core::{
    Event, ExternalDragFile, ExternalDropFileData, ExternalDropReadError, ExternalDropReadLimits,
    FileDialogDataEvent, FileDialogSelection, TimerToken,
};
use fret_runtime::{Effect, PlatformCapabilities};
use wasm_bindgen::JsCast as _;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{Document, Event as WebSysEvent, EventTarget, HtmlElement, HtmlInputElement, Node};

type WebChangeCallback = wasm_bindgen::closure::Closure<dyn FnMut(WebSysEvent)>;

fn window() -> Option<web_sys::Window> {
    web_sys::window()
}

fn document() -> Option<Document> {
    window().and_then(|w| w.document())
}

/// Web-specific platform services for `fret-runtime::Effect`s that require browser APIs.
///
/// This crate intentionally does *not* implement input/event mapping; use `winit` for that.
#[derive(Debug, Default)]
pub struct WebPlatformServices {
    queued_events: Rc<RefCell<Vec<Event>>>,
    fired_timeouts: Rc<RefCell<Vec<TimerToken>>>,
    timers: HashMap<TimerToken, WebTimer>,
    file_dialogs: Rc<RefCell<WebFileDialogState>>,
}

#[derive(Debug)]
struct WebTimer {
    id: i32,
    repeat: Option<Duration>,
    callback: wasm_bindgen::closure::Closure<dyn FnMut()>,
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
                    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                        let _ = queue.try_borrow_mut().map(|mut q| q.push(token));
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

                    let Some(window) = window() else {
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
