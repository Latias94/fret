use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use fret_runtime::{DispatchPriority, Dispatcher, ExecCapabilities, Runnable};
use winit::event_loop::EventLoopProxy;

#[derive(Clone)]
pub(super) struct DesktopDispatcher {
    inner: Arc<DesktopDispatcherInner>,
}

struct DesktopDispatcherInner {
    exec: ExecCapabilities,
    event_loop_proxy: Mutex<Option<EventLoopProxy>>,
    main_queue: Mutex<VecDeque<Runnable>>,
    delayed: Mutex<Vec<DelayedTask>>,
}

struct DelayedTask {
    deadline: Instant,
    task: Runnable,
}

impl DesktopDispatcher {
    pub(super) fn new(exec: ExecCapabilities) -> Self {
        Self {
            inner: Arc::new(DesktopDispatcherInner {
                exec,
                event_loop_proxy: Mutex::new(None),
                main_queue: Mutex::new(VecDeque::new()),
                delayed: Mutex::new(Vec::new()),
            }),
        }
    }

    pub(super) fn handle(&self) -> fret_runtime::DispatcherHandle {
        self.inner.clone()
    }

    pub(super) fn set_event_loop_proxy(&self, proxy: EventLoopProxy) {
        if let Ok(mut slot) = self.inner.event_loop_proxy.lock() {
            *slot = Some(proxy);
        }
    }

    pub(super) fn next_deadline(&self) -> Option<Instant> {
        let Ok(tasks) = self.inner.delayed.lock() else {
            return None;
        };
        tasks.iter().map(|t| t.deadline).min()
    }

    pub(super) fn drain_turn(&self, now: Instant) -> bool {
        let mut ready: Vec<Runnable> = Vec::new();

        if let Ok(mut delayed) = self.inner.delayed.lock() {
            let mut pending = Vec::new();
            for task in delayed.drain(..) {
                if task.deadline <= now {
                    ready.push(task.task);
                } else {
                    pending.push(task);
                }
            }
            *delayed = pending;
        }

        if let Ok(mut queue) = self.inner.main_queue.lock() {
            ready.extend(queue.drain(..));
        }

        if ready.is_empty() {
            return false;
        }

        for task in ready {
            task();
        }

        true
    }
}

impl Dispatcher for DesktopDispatcherInner {
    fn dispatch_on_main_thread(&self, task: Runnable) {
        if let Ok(mut queue) = self.main_queue.lock() {
            queue.push_back(task);
        }
        self.wake(None);
    }

    fn dispatch_background(&self, task: Runnable, _priority: DispatchPriority) {
        std::thread::spawn(task);
    }

    fn dispatch_after(&self, delay: Duration, task: Runnable) {
        let deadline = Instant::now() + delay;
        if let Ok(mut delayed) = self.delayed.lock() {
            delayed.push(DelayedTask { deadline, task });
        }
        self.wake(None);
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
