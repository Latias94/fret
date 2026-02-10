use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fret_core::{
    Event, ExternalDragFile, ExternalDropFileData, ExternalDropReadError, ExternalDropReadLimits,
    FileDialogDataEvent, FileDialogSelection,
};
use wasm_bindgen::JsCast as _;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{EventTarget, HtmlElement, HtmlInputElement, Node};

use super::{WebChangeCallback, WebSysEvent, WebWaker, document};

#[derive(Debug, Default)]
pub(crate) struct WebFileDialogState {
    next_token: u64,
    pub(crate) selections: HashMap<fret_runtime::FileDialogToken, Vec<web_sys::File>>,
}

impl WebFileDialogState {
    fn allocate_token(&mut self) -> fret_runtime::FileDialogToken {
        let next = self.next_token.max(1);
        let token = fret_runtime::FileDialogToken(next);
        self.next_token = next.saturating_add(1);
        token
    }
}

pub(crate) fn open(
    options: &fret_core::FileDialogOptions,
    state: Rc<RefCell<WebFileDialogState>>,
    queued_events: Rc<RefCell<Vec<Event>>>,
    waker: Option<WebWaker>,
) {
    let Some(document) = document() else {
        return;
    };
    let Ok(el) = document.create_element("input") else {
        return;
    };
    let Ok(input) = el.dyn_into::<HtmlInputElement>() else {
        return;
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

    let input_target: EventTarget = input.clone().unchecked_into();
    let input_target_for_cb = input_target.clone();
    let input_for_cb = input.clone();
    let input_node_for_cb: Node = input.clone().unchecked_into();

    let callback_cell: Rc<RefCell<Option<WebChangeCallback>>> = Rc::new(RefCell::new(None));
    let callback_cell_for_cb = callback_cell.clone();

    let on_change = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: WebSysEvent| {
        if let Some(parent) = input_node_for_cb.parent_node() {
            let _ = parent.remove_child(&input_node_for_cb);
        }

        if let Ok(holder) = callback_cell_for_cb.try_borrow()
            && let Some(cb) = holder.as_ref()
        {
            let _ = input_target_for_cb
                .remove_event_listener_with_callback("change", cb.as_ref().unchecked_ref());
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
            let _ = queued_events
                .try_borrow_mut()
                .map(|mut q| q.push(Event::FileDialogCanceled));
            if let Some(wake) = waker.as_ref() {
                wake();
            }
            return;
        }

        let (token, files_meta) = {
            let mut st = state.borrow_mut();
            let token = st.allocate_token();
            let files_meta = selected
                .iter()
                .map(|f| ExternalDragFile {
                    name: f.name(),
                    size_bytes: Some(f.size() as u64),
                    media_type: {
                        let ty = f.type_();
                        (!ty.is_empty()).then_some(ty)
                    },
                })
                .collect::<Vec<_>>();
            st.selections.insert(token, selected);
            (token, files_meta)
        };

        let selection = FileDialogSelection {
            token,
            files: files_meta,
        };
        let _ = queued_events
            .try_borrow_mut()
            .map(|mut q| q.push(Event::FileDialogSelection(selection)));
        if let Some(wake) = waker.as_ref() {
            wake();
        }
    }) as Box<dyn FnMut(WebSysEvent)>);

    *callback_cell.borrow_mut() = Some(on_change);
    if let Ok(holder) = callback_cell.try_borrow()
        && let Some(cb) = holder.as_ref()
    {
        let _ =
            input_target.add_event_listener_with_callback("change", cb.as_ref().unchecked_ref());
    }

    input.click();
}

pub(crate) fn read_all(
    token: fret_runtime::FileDialogToken,
    limits: ExternalDropReadLimits,
    state: Rc<RefCell<WebFileDialogState>>,
    queued_events: Rc<RefCell<Vec<Event>>>,
    waker: Option<WebWaker>,
) {
    let files = state.borrow().selections.get(&token).cloned();
    let Some(files) = files else {
        return;
    };

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
        let _ = queued_events.try_borrow_mut().map(|mut q| q.push(event));
        if let Some(wake) = waker.as_ref() {
            wake();
        }
    });
}

pub(crate) fn release(
    token: fret_runtime::FileDialogToken,
    state: &Rc<RefCell<WebFileDialogState>>,
) {
    state.borrow_mut().selections.remove(&token);
}
