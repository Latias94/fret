use std::any::Any;

use fret_runtime::UiHost;
use fret_ui::{ElementContext, Invalidation};

/// Ergonomic helpers for observing-and-reading globals during declarative rendering.
///
/// This is intentionally a component-layer API (ADR 0066): it provides sugar on top of
/// `fret-ui`'s explicit `observe_global(..., Invalidation)` contract.
pub trait GlobalWatchExt {
    type WatchedGlobal<'cx, T: Any>
    where
        Self: 'cx;

    fn watch_global<'cx, T: Any>(&'cx mut self) -> Self::WatchedGlobal<'cx, T>;
}

impl<'a, H: UiHost> GlobalWatchExt for ElementContext<'a, H> {
    type WatchedGlobal<'cx, T: Any>
        = WatchedGlobal<'cx, 'a, H, T>
    where
        Self: 'cx;

    fn watch_global<'cx, T: Any>(&'cx mut self) -> Self::WatchedGlobal<'cx, T> {
        WatchedGlobal {
            cx: self,
            invalidation: Invalidation::Paint,
            _ty: std::marker::PhantomData,
        }
    }
}

#[must_use]
pub struct WatchedGlobal<'cx, 'a, H: UiHost, T: Any> {
    cx: &'cx mut ElementContext<'a, H>,
    invalidation: Invalidation,
    _ty: std::marker::PhantomData<T>,
}

impl<'cx, 'a, H: UiHost, T: Any> WatchedGlobal<'cx, 'a, H, T> {
    pub fn invalidation(mut self, invalidation: Invalidation) -> Self {
        self.invalidation = invalidation;
        self
    }

    pub fn paint(self) -> Self {
        self.invalidation(Invalidation::Paint)
    }

    pub fn layout(self) -> Self {
        self.invalidation(Invalidation::Layout)
    }

    pub fn hit_test(self) -> Self {
        self.invalidation(Invalidation::HitTest)
    }

    pub fn observe(self) {
        self.cx.observe_global::<T>(self.invalidation);
    }

    pub fn as_ref(self) -> Option<&'cx T> {
        self.cx.observe_global::<T>(self.invalidation);
        self.cx.app.global::<T>()
    }

    pub fn copied(self) -> Option<T>
    where
        T: Copy,
    {
        self.as_ref().copied()
    }

    pub fn cloned(self) -> Option<T>
    where
        T: Clone,
    {
        self.as_ref().cloned()
    }

    pub fn map<R>(self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.as_ref().map(f)
    }
}
