use std::{
    collections::VecDeque,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
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
    alive: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
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
                alive: Arc::new(AtomicBool::new(true)),
                generation: Arc::new(AtomicU64::new(0)),
                event_loop_proxy: Mutex::new(None),
                main_queue: Mutex::new(VecDeque::new()),
                delayed: Mutex::new(Vec::new()),
            }),
        }
    }

    pub(super) fn handle(&self) -> fret_runtime::DispatcherHandle {
        self.inner.clone()
    }

    pub(super) fn shutdown(&self) {
        self.inner.alive.store(false, Ordering::SeqCst);

        if let Ok(mut proxy) = self.inner.event_loop_proxy.lock() {
            *proxy = None;
        }

        if let Ok(mut delayed) = self.inner.delayed.lock() {
            delayed.clear();
        }
        if let Ok(mut queue) = self.inner.main_queue.lock() {
            queue.clear();
        }
    }

    #[cfg(any(feature = "hotpatch-subsecond", test))]
    pub(super) fn hot_reload_boundary(&self) {
        if !self.inner.alive.load(Ordering::SeqCst) {
            return;
        }
        self.inner.generation.fetch_add(1, Ordering::SeqCst);

        if let Ok(mut delayed) = self.inner.delayed.lock() {
            delayed.clear();
        }
        if let Ok(mut queue) = self.inner.main_queue.lock() {
            queue.clear();
        }
    }

    pub(super) fn set_event_loop_proxy(&self, proxy: EventLoopProxy) {
        if let Ok(mut slot) = self.inner.event_loop_proxy.lock() {
            *slot = Some(proxy);
        }
    }

    pub(super) fn next_deadline(&self) -> Option<Instant> {
        if !self.inner.alive.load(Ordering::SeqCst) {
            return None;
        }
        let Ok(tasks) = self.inner.delayed.lock() else {
            return None;
        };
        tasks.iter().map(|t| t.deadline).min()
    }

    pub(super) fn drain_turn(&self, now: Instant) -> bool {
        if !self.inner.alive.load(Ordering::SeqCst) {
            return false;
        }
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
        if !self.alive.load(Ordering::SeqCst) {
            return;
        }
        let expected_gen = self.generation.load(Ordering::SeqCst);
        let alive = self.alive.clone();
        let generation = self.generation.clone();
        let task = Box::new(move || {
            if !alive.load(Ordering::SeqCst) {
                return;
            }
            if generation.load(Ordering::SeqCst) != expected_gen {
                return;
            }
            task();
        });
        if let Ok(mut queue) = self.main_queue.lock() {
            queue.push_back(task);
        }
        self.wake(None);
    }

    fn dispatch_background(&self, task: Runnable, _priority: DispatchPriority) {
        if !self.alive.load(Ordering::SeqCst) {
            return;
        }
        let expected_gen = self.generation.load(Ordering::SeqCst);
        let alive = self.alive.clone();
        let generation = self.generation.clone();
        std::thread::spawn(move || {
            if !alive.load(Ordering::SeqCst) {
                return;
            }
            if generation.load(Ordering::SeqCst) != expected_gen {
                return;
            }
            task();
        });
    }

    fn dispatch_after(&self, delay: Duration, task: Runnable) {
        if !self.alive.load(Ordering::SeqCst) {
            return;
        }
        let expected_gen = self.generation.load(Ordering::SeqCst);
        let alive = self.alive.clone();
        let generation = self.generation.clone();
        let task = Box::new(move || {
            if !alive.load(Ordering::SeqCst) {
                return;
            }
            if generation.load(Ordering::SeqCst) != expected_gen {
                return;
            }
            task();
        });
        let deadline = Instant::now() + delay;
        if let Ok(mut delayed) = self.delayed.lock() {
            delayed.push(DelayedTask { deadline, task });
        }
        self.wake(None);
    }

    fn wake(&self, _window: Option<fret_core::AppWindowId>) {
        if !self.alive.load(Ordering::SeqCst) {
            return;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn shutdown_prevents_future_dispatch_and_draining() {
        let dispatcher = DesktopDispatcher::new(ExecCapabilities::default());
        let ran = Arc::new(AtomicUsize::new(0));

        {
            let ran = ran.clone();
            dispatcher
                .handle()
                .dispatch_on_main_thread(Box::new(move || {
                    ran.fetch_add(1, Ordering::SeqCst);
                }));
        }
        assert!(dispatcher.drain_turn(Instant::now()));
        assert_eq!(ran.load(Ordering::SeqCst), 1);

        dispatcher.shutdown();

        {
            let ran = ran.clone();
            dispatcher
                .handle()
                .dispatch_on_main_thread(Box::new(move || {
                    ran.fetch_add(1, Ordering::SeqCst);
                }));
        }
        assert!(!dispatcher.drain_turn(Instant::now()));
        assert_eq!(ran.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn shutdown_prevents_dispatch_after_callbacks() {
        let dispatcher = DesktopDispatcher::new(ExecCapabilities::default());
        let ran = Arc::new(AtomicUsize::new(0));

        {
            let ran = ran.clone();
            dispatcher.handle().dispatch_after(
                Duration::from_millis(0),
                Box::new(move || {
                    ran.fetch_add(1, Ordering::SeqCst);
                }),
            );
        }
        assert!(dispatcher.drain_turn(Instant::now()));
        assert_eq!(ran.load(Ordering::SeqCst), 1);

        dispatcher.shutdown();

        {
            let ran = ran.clone();
            dispatcher.handle().dispatch_after(
                Duration::from_millis(0),
                Box::new(move || {
                    ran.fetch_add(1, Ordering::SeqCst);
                }),
            );
        }
        assert!(!dispatcher.drain_turn(Instant::now()));
        assert_eq!(ran.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn hot_reload_boundary_fences_queued_tasks() {
        let dispatcher = DesktopDispatcher::new(ExecCapabilities::default());
        let ran = Arc::new(AtomicUsize::new(0));

        {
            let ran = ran.clone();
            dispatcher
                .handle()
                .dispatch_on_main_thread(Box::new(move || {
                    ran.fetch_add(1, Ordering::SeqCst);
                }));
        }

        let queued = {
            let mut q = dispatcher.inner.main_queue.lock().unwrap();
            q.pop_front().unwrap()
        };

        dispatcher.hot_reload_boundary();
        queued();
        assert_eq!(ran.load(Ordering::SeqCst), 0);

        {
            let ran = ran.clone();
            dispatcher.handle().dispatch_after(
                Duration::from_millis(0),
                Box::new(move || {
                    ran.fetch_add(1, Ordering::SeqCst);
                }),
            );
        }

        let delayed = {
            let mut d = dispatcher.inner.delayed.lock().unwrap();
            d.pop().unwrap().task
        };

        dispatcher.hot_reload_boundary();
        delayed();
        assert_eq!(ran.load(Ordering::SeqCst), 0);
    }
}
