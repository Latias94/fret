use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

use fret_runtime::{DispatchPriority, Dispatcher, DispatcherHandle, ExecCapabilities, Runnable};
use web_sys::js_sys;
use web_sys::wasm_bindgen::JsCast as _;
use web_sys::wasm_bindgen::closure::Closure;
use winit::event_loop::EventLoopProxy;

fn window() -> Option<web_sys::Window> {
    web_sys::window()
}

fn duration_ms(delay: Duration) -> i32 {
    let ms = delay.as_millis();
    i32::try_from(ms.min(i32::MAX as u128)).unwrap_or(i32::MAX)
}

#[derive(Clone)]
pub(super) struct WebDispatcher {
    inner: Arc<WebDispatcherInner>,
}

struct WebDispatcherInner {
    exec: ExecCapabilities,
    event_loop_proxy: Arc<Mutex<Option<EventLoopProxy>>>,
    main_queue: Arc<Mutex<VecDeque<Runnable>>>,
}

impl WebDispatcher {
    pub(super) fn new(exec: ExecCapabilities) -> Self {
        Self {
            inner: Arc::new(WebDispatcherInner {
                exec,
                event_loop_proxy: Arc::new(Mutex::new(None)),
                main_queue: Arc::new(Mutex::new(VecDeque::new())),
            }),
        }
    }

    pub(super) fn handle(&self) -> DispatcherHandle {
        self.inner.clone()
    }

    pub(super) fn set_event_loop_proxy(&self, proxy: EventLoopProxy) {
        if let Ok(mut slot) = self.inner.event_loop_proxy.lock() {
            *slot = Some(proxy);
        }
    }

    pub(super) fn drain_turn(&self) -> bool {
        let mut tasks: Vec<Runnable> = Vec::new();
        if let Ok(mut q) = self.inner.main_queue.lock() {
            tasks.extend(q.drain(..));
        }

        if tasks.is_empty() {
            return false;
        }

        for task in tasks {
            task();
        }
        true
    }
}

impl Dispatcher for WebDispatcherInner {
    fn dispatch_on_main_thread(&self, task: Runnable) {
        if let Ok(mut q) = self.main_queue.lock() {
            q.push_back(task);
        }
        self.wake(None);
    }

    fn dispatch_background(&self, task: Runnable, _priority: DispatchPriority) {
        wasm_bindgen_futures::spawn_local(async move {
            task();
        });
    }

    fn dispatch_after(&self, delay: Duration, task: Runnable) {
        let Some(window) = window() else {
            return;
        };

        let main_queue = self.main_queue.clone();
        let event_loop_proxy = self.event_loop_proxy.clone();

        let callback = Closure::once_into_js(move || {
            if let Ok(mut q) = main_queue.lock() {
                q.push_back(task);
            }

            if let Ok(proxy) = event_loop_proxy.lock()
                && let Some(proxy) = proxy.as_ref()
            {
                proxy.wake_up();
            }
        });

        let function: js_sys::Function = callback.unchecked_into();
        let _ = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&function, duration_ms(delay));
    }

    fn wake(&self, _window: Option<fret_core::AppWindowId>) {
        let Ok(proxy) = self.event_loop_proxy.lock() else {
            return;
        };
        let Some(proxy) = proxy.as_ref() else {
            return;
        };
        proxy.wake_up();
    }

    fn exec_capabilities(&self) -> ExecCapabilities {
        self.exec
    }
}
