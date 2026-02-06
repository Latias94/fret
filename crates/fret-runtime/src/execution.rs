use std::{sync::Arc, time::Duration};

use fret_core::AppWindowId;

use crate::ExecCapabilities;
use crate::effect::Effect;
use crate::model::ModelStore;

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

pub trait InboxDrainHost {
    fn request_redraw(&mut self, window: AppWindowId);
    fn push_effect(&mut self, effect: Effect);
    fn models_mut(&mut self) -> &mut ModelStore;
}

pub trait InboxDrain: Send + Sync + 'static {
    fn drain(&self, host: &mut dyn InboxDrainHost, window: Option<AppWindowId>) -> bool;
}

#[derive(Default)]
pub struct InboxDrainRegistry {
    drainers: Vec<Arc<dyn InboxDrain>>,
}

impl InboxDrainRegistry {
    pub fn register(&mut self, drainer: Arc<dyn InboxDrain>) {
        self.drainers.push(drainer);
    }

    pub fn drain_all(&self, host: &mut dyn InboxDrainHost, window: Option<AppWindowId>) -> bool {
        let mut did_work = false;
        for drainer in &self.drainers {
            did_work |= drainer.drain(host, window);
        }
        did_work
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestHost {
        models: ModelStore,
        redraws: Vec<AppWindowId>,
        effects: Vec<Effect>,
    }

    impl InboxDrainHost for TestHost {
        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraws.push(window);
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    #[test]
    fn inbox_drain_registry_invokes_drainers_and_reports_progress() {
        struct Drainer;
        impl InboxDrain for Drainer {
            fn drain(&self, _host: &mut dyn InboxDrainHost, _window: Option<AppWindowId>) -> bool {
                true
            }
        }

        let mut host = TestHost::default();
        let mut reg = InboxDrainRegistry::default();
        reg.register(Arc::new(Drainer));

        assert!(reg.drain_all(&mut host, None));
    }
}
