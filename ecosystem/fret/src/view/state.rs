use std::any::Any;
use std::hash::Hash;

use fret_ui::UiHost;

use super::{AppUi, LocalState, WatchedState};

/// Grouped LocalState-first helpers for the default app authoring surface.
#[doc(hidden)]
pub struct AppUiState<'view, 'cx, 'a, H: UiHost> {
    pub(super) cx: &'view mut AppUi<'cx, 'a, H>,
}

impl<'view, 'cx, 'a, H: UiHost> AppUiState<'view, 'cx, 'a, H> {
    #[track_caller]
    pub fn local<T>(self) -> LocalState<T>
    where
        T: Any + Default,
    {
        self.cx.local_with(T::default)
    }

    #[track_caller]
    pub fn local_keyed<K: Hash, T>(self, key: K) -> LocalState<T>
    where
        T: Any + Default,
    {
        self.cx.keyed(key, |cx| cx.local_with(T::default))
    }

    #[track_caller]
    pub fn local_init<T>(self, init: impl FnOnce() -> T) -> LocalState<T>
    where
        T: Any,
    {
        self.cx.local_with(init)
    }

    pub fn watch<T: Any>(
        self,
        local: &'view LocalState<T>,
    ) -> WatchedState<'view, 'view, 'a, H, T> {
        self.cx.watch_local(local)
    }
}
