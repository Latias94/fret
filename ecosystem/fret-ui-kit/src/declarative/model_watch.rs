use std::any::Any;

use fret_runtime::{Model, ModelUpdateError};
use fret_ui::{ElementContext, Invalidation, UiHost};

/// Ergonomic helpers for observing-and-reading models during declarative rendering.
///
/// This is intentionally a component-layer API (ADR 0066): it provides sugar on top of
/// `fret-ui`'s explicit `observe_model(..., Invalidation)` contract (ADR 0051).
pub trait ModelWatchExt {
    type WatchedModel<'cx, 'm, T: Any>
    where
        Self: 'cx;

    fn watch_model<'cx, 'm, T: Any>(
        &'cx mut self,
        model: &'m Model<T>,
    ) -> Self::WatchedModel<'cx, 'm, T>;
}

impl<'a, H: UiHost> ModelWatchExt for ElementContext<'a, H> {
    type WatchedModel<'cx, 'm, T: Any>
        = WatchedModel<'cx, 'm, 'a, H, T>
    where
        Self: 'cx;

    fn watch_model<'cx, 'm, T: Any>(
        &'cx mut self,
        model: &'m Model<T>,
    ) -> Self::WatchedModel<'cx, 'm, T> {
        WatchedModel {
            cx: self,
            model,
            invalidation: Invalidation::Paint,
        }
    }
}

#[must_use]
pub struct WatchedModel<'cx, 'm, 'a, H: UiHost, T: Any> {
    cx: &'cx mut ElementContext<'a, H>,
    model: &'m Model<T>,
    invalidation: Invalidation,
}

impl<'cx, 'm, 'a, H: UiHost, T: Any> WatchedModel<'cx, 'm, 'a, H, T> {
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
        self.cx.observe_model(self.model, self.invalidation);
    }

    pub fn revision(self) -> Option<u64> {
        self.cx.observe_model(self.model, self.invalidation);
        self.cx.app.models().revision(self.model)
    }

    pub fn copied(self) -> Option<T>
    where
        T: Copy,
    {
        self.cx.get_model_copied(self.model, self.invalidation)
    }

    pub fn copied_or(self, default: T) -> T
    where
        T: Copy,
    {
        self.copied().unwrap_or(default)
    }

    pub fn copied_or_default(self) -> T
    where
        T: Copy + Default,
    {
        self.copied().unwrap_or_default()
    }

    pub fn cloned(self) -> Option<T>
    where
        T: Clone,
    {
        self.cx.get_model_cloned(self.model, self.invalidation)
    }

    pub fn cloned_or(self, default: T) -> T
    where
        T: Clone,
    {
        self.cloned().unwrap_or(default)
    }

    pub fn cloned_or_else(self, f: impl FnOnce() -> T) -> T
    where
        T: Clone,
    {
        self.cloned().unwrap_or_else(f)
    }

    pub fn cloned_or_default(self) -> T
    where
        T: Clone + Default,
    {
        self.cloned().unwrap_or_default()
    }

    pub fn read_ref<R>(self, f: impl FnOnce(&T) -> R) -> Result<R, ModelUpdateError> {
        self.cx.read_model_ref(self.model, self.invalidation, f)
    }

    pub fn read<R>(self, f: impl FnOnce(&mut H, &T) -> R) -> Result<R, ModelUpdateError> {
        self.cx.read_model(self.model, self.invalidation, f)
    }
}
