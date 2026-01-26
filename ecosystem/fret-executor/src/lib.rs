//! Ecosystem execution utilities (Inbox + light executor helpers).
//!
//! This crate intentionally lives under `ecosystem/*` so it can evolve without forcing kernel
//! churn in the core framework crates. The stable contract is the runner-provided
//! `fret_runtime::Dispatcher` (ADR 0190).

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use fret_core::AppWindowId;
use fret_runtime::{DispatchPriority, DispatcherHandle, Runnable};

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

/// Thin helper over a runner-provided `Dispatcher`.
#[derive(Clone)]
pub struct Executors {
    dispatcher: DispatcherHandle,
}

impl Executors {
    pub fn new(dispatcher: DispatcherHandle) -> Self {
        Self { dispatcher }
    }

    pub fn dispatch_on_main_thread(&self, task: Runnable) {
        self.dispatcher.dispatch_on_main_thread(task);
    }

    pub fn dispatch_background(&self, task: Runnable, priority: DispatchPriority) {
        self.dispatcher.dispatch_background(task, priority);
    }

    pub fn dispatch_background_to_inbox<M: Send + 'static>(
        &self,
        window: Option<AppWindowId>,
        inbox: InboxSender<M>,
        task: impl FnOnce() -> M + Send + 'static,
    ) {
        let dispatcher = self.dispatcher.clone();
        self.dispatcher.dispatch_background(
            Box::new(move || {
                let msg = task();
                let _ = inbox.send(msg);
                dispatcher.wake(window);
            }),
            DispatchPriority::Normal,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
