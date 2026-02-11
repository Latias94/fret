//! Ecosystem execution utilities (Inbox + light executor helpers).
//!
//! This crate intentionally lives under `ecosystem/*` so it can evolve without forcing kernel
//! churn in the core framework crates. The stable contract is the runner-provided
//! `fret_runtime::Dispatcher` (ADR 0175).

use std::{
    collections::VecDeque,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
    time::Duration,
};

use fret_core::AppWindowId;
use fret_runtime::{DispatchPriority, DispatcherHandle, InboxDrain, InboxDrainHost, Runnable};

#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct BackgroundTask {
    token: CancellationToken,
    cancel_on_drop: bool,
}

impl BackgroundTask {
    pub fn cancel(&self) {
        self.token.cancel();
    }

    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    pub fn detach(mut self) {
        self.cancel_on_drop = false;
    }
}

impl Drop for BackgroundTask {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.token.cancel();
        }
    }
}

#[derive(Debug)]
pub struct ForegroundTask {
    inner: BackgroundTask,
    _not_send: PhantomData<Rc<()>>,
}

impl ForegroundTask {
    pub fn cancel(&self) {
        self.inner.cancel();
    }

    pub fn token(&self) -> CancellationToken {
        self.inner.token()
    }

    pub fn detach(mut self) {
        self.inner.cancel_on_drop = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxOverflowStrategy {
    /// Reject new messages once the inbox reaches capacity.
    DropNewest,
    /// Drop the oldest queued message to make room for the new one.
    DropOldest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InboxConfig {
    pub capacity: usize,
    pub overflow: InboxOverflowStrategy,
}

impl Default for InboxConfig {
    fn default() -> Self {
        Self {
            capacity: 1024,
            overflow: InboxOverflowStrategy::DropOldest,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InboxStats {
    pub dropped_newest: u64,
    pub dropped_oldest: u64,
}

#[derive(Debug)]
struct InboxInner<M> {
    config: InboxConfig,
    queue: VecDeque<M>,
    stats: InboxStats,
}

/// Bounded, portable inbox for data-only messages.
///
/// Intended usage:
/// - background producer sends messages into the inbox,
/// - producer calls `dispatcher.wake(...)` (or `dispatch_on_main_thread`) to prompt a driver boundary,
/// - UI/main thread drains at an explicit flush point and applies updates.
pub struct Inbox<M> {
    inner: Arc<Mutex<InboxInner<M>>>,
}

impl<M> Clone for Inbox<M> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<M> Inbox<M> {
    pub fn new(config: InboxConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(InboxInner {
                config,
                queue: VecDeque::new(),
                stats: InboxStats::default(),
            })),
        }
    }

    pub fn sender(&self) -> InboxSender<M> {
        InboxSender {
            inner: self.inner.clone(),
        }
    }

    pub fn drain(&self) -> Vec<M> {
        let mut out = Vec::new();
        let Ok(mut inner) = self.inner.lock() else {
            return out;
        };
        while let Some(msg) = inner.queue.pop_front() {
            out.push(msg);
        }
        out
    }

    pub fn stats(&self) -> InboxStats {
        let Ok(inner) = self.inner.lock() else {
            return InboxStats::default();
        };
        inner.stats
    }
}

#[derive(Clone)]
pub struct InboxSender<M> {
    inner: Arc<Mutex<InboxInner<M>>>,
}

impl<M> InboxSender<M> {
    /// Attempts to enqueue a message according to the configured backpressure strategy.
    ///
    /// Returns `true` if the message was enqueued.
    pub fn send(&self, msg: M) -> bool {
        let Ok(mut inner) = self.inner.lock() else {
            return false;
        };

        if inner.queue.len() < inner.config.capacity {
            inner.queue.push_back(msg);
            return true;
        }

        match inner.config.overflow {
            InboxOverflowStrategy::DropNewest => {
                inner.stats.dropped_newest = inner.stats.dropped_newest.saturating_add(1);
                false
            }
            InboxOverflowStrategy::DropOldest => {
                let _ = inner.queue.pop_front();
                inner.stats.dropped_oldest = inner.stats.dropped_oldest.saturating_add(1);
                inner.queue.push_back(msg);
                true
            }
        }
    }
}

/// Adapter that turns an [`Inbox`] into a runner-drainable [`InboxDrain`] implementation.
///
/// Register instances in `fret_runtime::InboxDrainRegistry` so runners can drain inboxes at a
/// driver boundary (ADR 0175).
pub struct InboxDrainer<M> {
    inbox: Inbox<M>,
    window_hint: Option<AppWindowId>,
    apply: Arc<dyn Fn(&mut dyn InboxDrainHost, Option<AppWindowId>, M) + Send + Sync>,
}

impl<M> InboxDrainer<M> {
    pub fn new(
        inbox: Inbox<M>,
        apply: impl Fn(&mut dyn InboxDrainHost, Option<AppWindowId>, M) + Send + Sync + 'static,
    ) -> Self {
        Self {
            inbox,
            window_hint: None,
            apply: Arc::new(apply),
        }
    }

    pub fn with_window_hint(mut self, window: AppWindowId) -> Self {
        self.window_hint = Some(window);
        self
    }
}

impl<M: Send + 'static> InboxDrain for InboxDrainer<M> {
    fn drain(&self, host: &mut dyn InboxDrainHost, window: Option<AppWindowId>) -> bool {
        let window = self.window_hint.or(window);
        let drained = self.inbox.drain();
        if drained.is_empty() {
            return false;
        }

        for msg in drained {
            (self.apply)(host, window, msg);
        }

        true
    }
}

/// Thin helper over a runner-provided `Dispatcher`.
#[derive(Clone)]
pub struct Executors {
    dispatcher: DispatcherHandle,
}

impl Executors {
    pub fn new(dispatcher: DispatcherHandle) -> Self {
        Self { dispatcher }
    }

    /// Spawn a `Send` future and deliver its output into an inbox, waking the runner on success.
    ///
    /// This is intended for integrating async runtimes (tokio/async-std/etc.) with Fret's
    /// driver-boundary inbox model (ADR 0175), without forcing a specific runtime on the kernel.
    pub fn spawn_future_to_inbox<M, Fut>(
        &self,
        spawner: &dyn FutureSpawner,
        window: Option<AppWindowId>,
        inbox: InboxSender<M>,
        task: impl FnOnce(CancellationToken) -> Fut + Send + 'static,
    ) -> BackgroundTask
    where
        M: Send + 'static,
        Fut: Future<Output = M> + Send + 'static,
    {
        let token = CancellationToken {
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let task_token = token.clone();
        let dispatcher = self.dispatcher.clone();

        spawner.spawn_send(Box::pin(async move {
            if task_token.is_cancelled() {
                return;
            }

            let msg = task(task_token.clone()).await;

            if task_token.is_cancelled() {
                return;
            }

            if inbox.send(msg) {
                dispatcher.wake(window);
            }
        }));

        BackgroundTask {
            token,
            cancel_on_drop: true,
        }
    }

    /// Spawn a `!Send` future (usually wasm `spawn_local`) and deliver its output into an inbox.
    ///
    /// Returns `None` if the provided spawner does not support local futures.
    pub fn spawn_local_future_to_inbox<M, Fut>(
        &self,
        spawner: &dyn FutureSpawner,
        window: Option<AppWindowId>,
        inbox: InboxSender<M>,
        task: impl FnOnce(CancellationToken) -> Fut + 'static,
    ) -> Option<BackgroundTask>
    where
        M: Send + 'static,
        Fut: Future<Output = M> + 'static,
    {
        let token = CancellationToken {
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let task_token = token.clone();
        let dispatcher = self.dispatcher.clone();

        let fut = Box::pin(async move {
            if task_token.is_cancelled() {
                return;
            }

            let msg = task(task_token.clone()).await;

            if task_token.is_cancelled() {
                return;
            }

            if inbox.send(msg) {
                dispatcher.wake(window);
            }
        });

        if !spawner.spawn_local(fut) {
            return None;
        }

        Some(BackgroundTask {
            token,
            cancel_on_drop: true,
        })
    }

    pub fn dispatch_on_main_thread(&self, task: Runnable) {
        self.spawn_on_main_thread(move |_| task()).detach();
    }

    pub fn dispatch_background(&self, task: Runnable, priority: DispatchPriority) {
        self.spawn_background(priority, move |_| task()).detach();
    }

    pub fn dispatch_background_to_inbox<M: Send + 'static>(
        &self,
        window: Option<AppWindowId>,
        inbox: InboxSender<M>,
        task: impl FnOnce() -> M + Send + 'static,
    ) {
        self.spawn_background_to_inbox(window, inbox, move |_| task())
            .detach();
    }

    pub fn spawn_on_main_thread(
        &self,
        task: impl FnOnce(CancellationToken) + Send + 'static,
    ) -> ForegroundTask {
        let token = CancellationToken {
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let task_token = token.clone();
        self.dispatcher.dispatch_on_main_thread(Box::new(move || {
            if task_token.is_cancelled() {
                return;
            }
            task(task_token);
        }));

        ForegroundTask {
            inner: BackgroundTask {
                token,
                cancel_on_drop: true,
            },
            _not_send: PhantomData,
        }
    }

    pub fn spawn_background(
        &self,
        priority: DispatchPriority,
        task: impl FnOnce(CancellationToken) + Send + 'static,
    ) -> BackgroundTask {
        let token = CancellationToken {
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let task_token = token.clone();
        self.dispatcher.dispatch_background(
            Box::new(move || {
                if task_token.is_cancelled() {
                    return;
                }
                task(task_token);
            }),
            priority,
        );

        BackgroundTask {
            token,
            cancel_on_drop: true,
        }
    }

    pub fn spawn_after(
        &self,
        delay: Duration,
        task: impl FnOnce(CancellationToken) + Send + 'static,
    ) -> ForegroundTask {
        let token = CancellationToken {
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let task_token = token.clone();
        self.dispatcher.dispatch_after(
            delay,
            Box::new(move || {
                if task_token.is_cancelled() {
                    return;
                }
                task(task_token);
            }),
        );

        ForegroundTask {
            inner: BackgroundTask {
                token,
                cancel_on_drop: true,
            },
            _not_send: PhantomData,
        }
    }

    pub fn spawn_background_to_inbox<M: Send + 'static>(
        &self,
        window: Option<AppWindowId>,
        inbox: InboxSender<M>,
        task: impl FnOnce(CancellationToken) -> M + Send + 'static,
    ) -> BackgroundTask {
        let token = CancellationToken {
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let task_token = token.clone();
        let dispatcher = self.dispatcher.clone();

        self.dispatcher.dispatch_background(
            Box::new(move || {
                if task_token.is_cancelled() {
                    return;
                }

                let msg = task(task_token.clone());

                if task_token.is_cancelled() {
                    return;
                }

                if inbox.send(msg) {
                    dispatcher.wake(window);
                }
            }),
            DispatchPriority::Normal,
        );

        BackgroundTask {
            token,
            cancel_on_drop: true,
        }
    }
}

pub type FutureSpawnerHandle = Arc<dyn FutureSpawner>;

/// Runtime-provided future spawner used for async ecosystem adapters.
///
/// Fret does not require a specific async runtime (Tokio/async-std/etc.). Instead, apps or runners
/// can install a `FutureSpawnerHandle` as a global and ecosystem crates can consume it.
pub trait FutureSpawner: Send + Sync + 'static {
    /// Spawn a `Send` future.
    fn spawn_send(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>);

    /// Spawn a `!Send` future (usually only available on wasm via `spawn_local`).
    ///
    /// Returns `true` if the future was accepted/spawned.
    fn spawn_local(&self, fut: Pin<Box<dyn Future<Output = ()> + 'static>>) -> bool {
        let _ = fut;
        false
    }
}

#[cfg(feature = "tokio")]
#[derive(Clone)]
pub struct TokioSpawner {
    handle: tokio::runtime::Handle,
}

#[cfg(feature = "tokio")]
impl TokioSpawner {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        Self { handle }
    }

    pub fn try_current() -> Result<Self, tokio::runtime::TryCurrentError> {
        Ok(Self::new(tokio::runtime::Handle::try_current()?))
    }
}

#[cfg(feature = "tokio")]
impl FutureSpawner for TokioSpawner {
    fn spawn_send(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        let _ = self.handle.spawn(fut);
    }
}

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, Default)]
pub struct WasmSpawner;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
impl FutureSpawner for WasmSpawner {
    fn spawn_send(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        wasm_bindgen_futures::spawn_local(fut);
    }

    fn spawn_local(&self, fut: Pin<Box<dyn Future<Output = ()> + 'static>>) -> bool {
        wasm_bindgen_futures::spawn_local(fut);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    use fret_runtime::{Dispatcher, ExecCapabilities};
    use pollster::block_on;

    #[derive(Default)]
    struct TestDispatcher {
        main: Mutex<Vec<Runnable>>,
        background: Mutex<Vec<Runnable>>,
        after: Mutex<Vec<(Duration, Runnable)>>,
        wakes: AtomicUsize,
    }

    impl TestDispatcher {
        fn drain_main(&self) {
            let tasks = {
                let Ok(mut guard) = self.main.lock() else {
                    return;
                };
                std::mem::take(&mut *guard)
            };
            for task in tasks {
                task();
            }
        }

        fn drain_background(&self) {
            let tasks = {
                let Ok(mut guard) = self.background.lock() else {
                    return;
                };
                std::mem::take(&mut *guard)
            };
            for task in tasks {
                task();
            }
        }

        fn drain_after(&self) {
            let tasks = {
                let Ok(mut guard) = self.after.lock() else {
                    return;
                };
                std::mem::take(&mut *guard)
            };
            for (_delay, task) in tasks {
                task();
            }
        }
    }

    impl Dispatcher for TestDispatcher {
        fn dispatch_on_main_thread(&self, task: Runnable) {
            let Ok(mut guard) = self.main.lock() else {
                return;
            };
            guard.push(task);
        }

        fn dispatch_background(&self, task: Runnable, _priority: DispatchPriority) {
            let Ok(mut guard) = self.background.lock() else {
                return;
            };
            guard.push(task);
        }

        fn dispatch_after(&self, delay: Duration, task: Runnable) {
            let Ok(mut guard) = self.after.lock() else {
                return;
            };
            guard.push((delay, task));
        }

        fn wake(&self, _window: Option<AppWindowId>) {
            self.wakes.fetch_add(1, AtomicOrdering::Relaxed);
        }

        fn exec_capabilities(&self) -> ExecCapabilities {
            ExecCapabilities::default()
        }
    }

    #[test]
    fn inbox_drop_newest_rejects_when_full() {
        let inbox = Inbox::new(InboxConfig {
            capacity: 2,
            overflow: InboxOverflowStrategy::DropNewest,
        });
        let sender = inbox.sender();

        assert!(sender.send(1));
        assert!(sender.send(2));
        assert!(!sender.send(3));

        assert_eq!(inbox.drain(), vec![1, 2]);
        assert_eq!(
            inbox.stats(),
            InboxStats {
                dropped_newest: 1,
                dropped_oldest: 0
            }
        );
    }

    #[test]
    fn inbox_drop_oldest_eviction_keeps_latest() {
        let inbox = Inbox::new(InboxConfig {
            capacity: 2,
            overflow: InboxOverflowStrategy::DropOldest,
        });
        let sender = inbox.sender();

        assert!(sender.send(1));
        assert!(sender.send(2));
        assert!(sender.send(3));

        assert_eq!(inbox.drain(), vec![2, 3]);
        assert_eq!(
            inbox.stats(),
            InboxStats {
                dropped_newest: 0,
                dropped_oldest: 1
            }
        );
    }

    #[test]
    fn dropped_background_task_suppresses_inbox_and_wake() {
        let dispatcher = Arc::new(TestDispatcher::default());
        let ex = Executors::new(dispatcher.clone());

        let inbox = Inbox::new(InboxConfig {
            capacity: 4,
            overflow: InboxOverflowStrategy::DropOldest,
        });
        let sender = inbox.sender();

        let ran = Arc::new(AtomicUsize::new(0));
        {
            let ran = ran.clone();
            let task = ex.spawn_background_to_inbox(None, sender, move |_| {
                ran.fetch_add(1, AtomicOrdering::Relaxed);
                123
            });
            drop(task);
        }

        dispatcher.drain_background();

        assert_eq!(ran.load(AtomicOrdering::Relaxed), 0);
        assert!(inbox.drain().is_empty());
        assert_eq!(dispatcher.wakes.load(AtomicOrdering::Relaxed), 0);
    }

    #[test]
    fn detached_background_task_runs_and_wakes() {
        let dispatcher = Arc::new(TestDispatcher::default());
        let ex = Executors::new(dispatcher.clone());

        let inbox = Inbox::new(InboxConfig {
            capacity: 4,
            overflow: InboxOverflowStrategy::DropOldest,
        });
        let sender = inbox.sender();

        let ran = Arc::new(AtomicUsize::new(0));
        {
            let ran = ran.clone();
            ex.spawn_background_to_inbox(None, sender, move |_| {
                ran.fetch_add(1, AtomicOrdering::Relaxed);
                456
            })
            .detach();
        }

        dispatcher.drain_background();

        assert_eq!(ran.load(AtomicOrdering::Relaxed), 1);
        assert_eq!(inbox.drain(), vec![456]);
        assert_eq!(dispatcher.wakes.load(AtomicOrdering::Relaxed), 1);
    }

    #[test]
    fn cancelled_after_task_suppresses_callback() {
        let dispatcher = Arc::new(TestDispatcher::default());
        let ex = Executors::new(dispatcher.clone());

        let ran = Arc::new(AtomicUsize::new(0));
        let task = {
            let ran = ran.clone();
            ex.spawn_after(Duration::from_secs(1), move |_| {
                ran.fetch_add(1, AtomicOrdering::Relaxed);
            })
        };

        task.cancel();
        dispatcher.drain_after();

        assert_eq!(ran.load(AtomicOrdering::Relaxed), 0);
    }

    #[test]
    fn dropped_foreground_task_suppresses_callback() {
        let dispatcher = Arc::new(TestDispatcher::default());
        let ex = Executors::new(dispatcher.clone());

        let ran = Arc::new(AtomicUsize::new(0));
        {
            let ran = ran.clone();
            let task = ex.spawn_on_main_thread(move |_| {
                ran.fetch_add(1, AtomicOrdering::Relaxed);
            });
            drop(task);
        }

        dispatcher.drain_main();

        assert_eq!(ran.load(AtomicOrdering::Relaxed), 0);
    }

    #[derive(Default)]
    struct TestSpawner {
        send: Mutex<Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
    }

    impl TestSpawner {
        fn drain_send(&self) -> Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> {
            let mut guard = self.send.lock().unwrap();
            std::mem::take(&mut *guard)
        }
    }

    impl FutureSpawner for TestSpawner {
        fn spawn_send(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
            self.send.lock().unwrap().push(fut);
        }
    }

    #[test]
    fn dropped_future_task_suppresses_inbox_and_wake() {
        let dispatcher = Arc::new(TestDispatcher::default());
        let ex = Executors::new(dispatcher.clone());
        let spawner = TestSpawner::default();

        let inbox = Inbox::new(InboxConfig {
            capacity: 4,
            overflow: InboxOverflowStrategy::DropOldest,
        });
        let sender = inbox.sender();

        let ran = Arc::new(AtomicUsize::new(0));
        {
            let ran = ran.clone();
            let task = ex.spawn_future_to_inbox(&spawner, None, sender, move |_| async move {
                ran.fetch_add(1, AtomicOrdering::Relaxed);
                123
            });
            drop(task);
        }

        for task in spawner.drain_send() {
            block_on(task);
        }

        assert_eq!(ran.load(AtomicOrdering::Relaxed), 0);
        assert!(inbox.drain().is_empty());
        assert_eq!(dispatcher.wakes.load(AtomicOrdering::Relaxed), 0);
    }

    #[test]
    fn detached_future_task_runs_and_wakes() {
        let dispatcher = Arc::new(TestDispatcher::default());
        let ex = Executors::new(dispatcher.clone());
        let spawner = TestSpawner::default();

        let inbox = Inbox::new(InboxConfig {
            capacity: 4,
            overflow: InboxOverflowStrategy::DropOldest,
        });
        let sender = inbox.sender();

        let ran = Arc::new(AtomicUsize::new(0));
        {
            let ran = ran.clone();
            ex.spawn_future_to_inbox(&spawner, None, sender, move |_| async move {
                ran.fetch_add(1, AtomicOrdering::Relaxed);
                456
            })
            .detach();
        }

        for task in spawner.drain_send() {
            block_on(task);
        }

        assert_eq!(ran.load(AtomicOrdering::Relaxed), 1);
        assert_eq!(inbox.drain(), vec![456]);
        assert_eq!(dispatcher.wakes.load(AtomicOrdering::Relaxed), 1);
    }
}
