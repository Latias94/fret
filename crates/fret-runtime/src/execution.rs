use std::{sync::Arc, time::Duration};

use fret_core::AppWindowId;

use crate::ExecCapabilities;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchPriority {
    Low,
    Normal,
    High,
}

pub type Runnable = Box<dyn FnOnce() + Send + 'static>;

/// Portable execution surface provided by the runner.
///
/// This is a semantic contract (ADR 0190). Implementations live in platform runners and must:
///
/// - Preserve the main-thread mutation invariant (UI/runtime state is main-thread only).
/// - Keep the servicing thread/queue mapping stable for the lifetime of the process.
/// - Provide a `wake()` that advances the runner to the next driver boundary.
pub trait Dispatcher: Send + Sync + 'static {
    /// Schedule work to run on the UI/main thread.
    fn dispatch_on_main_thread(&self, task: Runnable);

    /// Schedule work to run off the UI thread where available.
    ///
    /// On constrained platforms (e.g. wasm without threads) this may be implemented as
    /// best-effort cooperative execution, but must still preserve the main-thread mutation
    /// invariant.
    fn dispatch_background(&self, task: Runnable, priority: DispatchPriority);

    /// Schedule delayed work.
    ///
    /// Implementations must share the same timer substrate/time base as effect timers to avoid
    /// split-brain scheduling (ADR 0190 / ADR 0034).
    fn dispatch_after(&self, delay: Duration, task: Runnable);

    /// Request that the runner reaches the next driver boundary promptly.
    ///
    /// Runners should coalesce repeated calls and may use `window` as a hint for multi-window
    /// implementations.
    fn wake(&self, window: Option<AppWindowId>);

    /// Execution capabilities exposed by this dispatcher implementation.
    fn exec_capabilities(&self) -> ExecCapabilities;
}

pub type DispatcherHandle = Arc<dyn Dispatcher>;
