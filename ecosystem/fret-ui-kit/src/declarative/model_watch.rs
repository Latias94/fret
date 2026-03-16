use std::any::Any;

#[cfg(feature = "state-query")]
use fret_query::{QueryHandle, QueryState};
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

/// Handle-first tracked-read helpers for helper-heavy `ElementContext` surfaces.
///
/// This stays in the component/declarative layer for the same reason as `ModelWatchExt`: it is
/// sugar over the explicit `observe_model(..., Invalidation)` contract, not a new runtime
/// mechanism.
pub trait TrackedModelExt<T: Any> {
    fn watch_in<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, T>;

    fn paint_in<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, T> {
        self.watch_in(cx).paint()
    }

    fn layout_in<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, T> {
        self.watch_in(cx).layout()
    }

    fn hit_test_in<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, T> {
        self.watch_in(cx).hit_test()
    }
}

impl<T: Any> TrackedModelExt<T> for Model<T> {
    fn watch_in<'cx, 'a, H: UiHost>(
        &self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedModel<'cx, '_, 'a, H, T> {
        WatchedModel {
            cx,
            model: self,
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

    /// Default post-v1 read path: clone/copy the tracked value without choosing between
    /// `copied_*` and `cloned_*` at every call site.
    pub fn value(self) -> Option<T>
    where
        T: Clone,
    {
        self.cloned()
    }

    pub fn value_or(self, default: T) -> T
    where
        T: Clone,
    {
        self.value().unwrap_or(default)
    }

    pub fn value_or_else(self, f: impl FnOnce() -> T) -> T
    where
        T: Clone,
    {
        self.value().unwrap_or_else(f)
    }

    pub fn value_or_default(self) -> T
    where
        T: Clone + Default,
    {
        self.value().unwrap_or_default()
    }

    pub fn read_ref<R>(self, f: impl FnOnce(&T) -> R) -> Result<R, ModelUpdateError> {
        self.cx.read_model_ref(self.model, self.invalidation, f)
    }

    pub fn read<R>(self, f: impl FnOnce(&mut H, &T) -> R) -> Result<R, ModelUpdateError> {
        self.cx.read_model(self.model, self.invalidation, f)
    }
}

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
        WatchedModel {
            cx,
            model: self.model(),
            invalidation: Invalidation::Paint,
        }
    }
}
