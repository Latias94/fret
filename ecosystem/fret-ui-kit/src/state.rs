//! Optional state-stack adapters for reusable component helpers.
//!
//! Base primitives stay value-first. This module is the explicit feature-gated seam for helpers
//! that need to speak to selector/query crates.

#[cfg(feature = "state-query")]
use fret_query::{QueryHandle, QueryState};
#[cfg(feature = "state-query")]
use fret_ui::{ElementContext, Invalidation, UiHost};

#[cfg(feature = "state-query")]
use crate::declarative::model_watch::WatchedModel;

#[cfg(feature = "state-query")]
pub trait QueryHandleWatchExt<T: 'static> {
    fn watch_query<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, QueryState<T>>;

    fn paint_query<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, QueryState<T>> {
        self.watch_query(cx).paint()
    }

    fn layout_query<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, QueryState<T>> {
        self.watch_query(cx).layout()
    }

    fn hit_test_query<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, QueryState<T>> {
        self.watch_query(cx).hit_test()
    }
}

#[cfg(feature = "state-query")]
impl<T: 'static> QueryHandleWatchExt<T> for QueryHandle<T> {
    fn watch_query<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, QueryState<T>> {
        WatchedModel::new(cx, self.model(), Invalidation::Paint)
    }
}
