//! App-facing async/background-work helpers.

use std::{any::Any, marker::PhantomData, sync::Arc};

use fret_core::AppWindowId;
use fret_runtime::{InboxDrain, InboxDrainHost, InboxDrainRegistry, ModelId};

use super::LocalState;

/// Main-thread-safe handle to a `LocalState<T>` slot for inbox drainers.
///
/// `LocalState<T>` itself contains the runtime model handle and is intentionally main-thread only.
/// Inbox drainers must be `Send + Sync`, so this compact handle records only the stable model id
/// needed once the runner invokes the drainer back at a driver boundary.
pub struct InboxLocal<T> {
    id: ModelId,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Copy for InboxLocal<T> {}

impl<T> Clone for InboxLocal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Convert app-local state into an inbox-drainer-safe handle.
pub fn inbox_local<T>(local: &LocalState<T>) -> InboxLocal<T> {
    InboxLocal {
        id: local.model.id(),
        _phantom: PhantomData,
    }
}

/// App-facing context passed to an inbox drainer callback.
///
/// This hides `InboxDrainHost`, `ModelStore`, and raw `ModelId` access from default app examples
/// while preserving the runner-boundary update contract from ADR 0175.
pub struct AppInboxCx<'a> {
    host: &'a mut dyn InboxDrainHost,
    window: Option<AppWindowId>,
    changed: bool,
    redraw_requested: bool,
}

impl<'a> AppInboxCx<'a> {
    pub fn window_id(&self) -> Option<AppWindowId> {
        self.window
    }

    pub fn request_redraw(&mut self) {
        let Some(window) = self.window else {
            return;
        };
        self.host.request_redraw(window);
        self.redraw_requested = true;
    }

    pub fn set_local<T>(&mut self, local: &InboxLocal<T>, value: T) -> bool
    where
        T: Any,
    {
        self.update_local(local, move |slot| *slot = value)
    }

    pub fn update_local<T>(&mut self, local: &InboxLocal<T>, update: impl FnOnce(&mut T)) -> bool
    where
        T: Any,
    {
        let mut applied = false;
        let updated = self
            .host
            .models_mut()
            .update_any(local.id, |any| {
                let Some(value) = any.downcast_mut::<T>() else {
                    return;
                };
                update(value);
                applied = true;
            })
            .is_ok()
            && applied;

        if updated {
            self.changed = true;
        }

        updated
    }

    fn finish(mut self) {
        if self.changed && !self.redraw_requested {
            self.request_redraw();
        }
    }
}

/// Adapt a typed message callback into the low-level `InboxDrainer::new(...)` callback shape.
pub fn inbox_drain_apply<M>(
    apply: impl Fn(&mut AppInboxCx<'_>, M) + Send + Sync + 'static,
) -> impl Fn(&mut dyn InboxDrainHost, Option<AppWindowId>, M) + Send + Sync + 'static {
    move |host, window, msg| {
        let mut cx = AppInboxCx {
            host,
            window,
            changed: false,
            redraw_requested: false,
        };
        apply(&mut cx, msg);
        cx.finish();
    }
}

/// App-level async work hooks for the default app facade.
pub trait AppAsyncWorkExt {
    fn dispatcher(&self) -> Option<fret_runtime::DispatcherHandle>;

    fn register_inbox_drainer<D>(&mut self, drainer: D)
    where
        D: InboxDrain;
}

impl AppAsyncWorkExt for crate::app::App {
    fn dispatcher(&self) -> Option<fret_runtime::DispatcherHandle> {
        self.global::<fret_runtime::DispatcherHandle>().cloned()
    }

    fn register_inbox_drainer<D>(&mut self, drainer: D)
    where
        D: InboxDrain,
    {
        self.with_global_mut_untracked(InboxDrainRegistry::default, |registry, _app| {
            registry.register(Arc::new(drainer));
        });
    }
}
