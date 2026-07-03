use std::any::Any;

use fret_runtime::{Model, ModelStore, ModelUpdateError};
use fret_ui::{ElementContext, UiHost};

use super::{LocalState, WatchedState};

/// Explicit raw `Model<T>` bridge for advanced/component/hybrid surfaces.
///
/// This trait intentionally stays off `fret::app::prelude::*`. Import it from
/// `fret::advanced` or `fret::advanced::prelude::*` when a surface still needs to wrap, expose, or
/// clone a raw model handle.
pub trait LocalStateRawModelExt<T> {
    /// Wrap an existing raw `Model<T>` handle as explicit `LocalState<T>` bridge state.
    fn from_model(model: Model<T>) -> Self;

    /// Expose the underlying `Model<T>` as an explicit bridge.
    fn model(&self) -> &Model<T>;

    /// Clone the underlying `Model<T>` as an explicit bridge.
    fn clone_model(&self) -> Model<T>;
}

impl<T> LocalStateRawModelExt<T> for LocalState<T> {
    fn from_model(model: Model<T>) -> Self {
        Self { model }
    }

    fn model(&self) -> &Model<T> {
        &self.model
    }

    fn clone_model(&self) -> Model<T> {
        self.model.clone()
    }
}

/// Explicit `ModelStore` bridge for advanced transactions and manual/hybrid surfaces.
///
/// Keep default app writes on grouped helpers such as `cx.actions().local(...)` or
/// `cx.actions().locals_with(...)`; import this trait only when the surrounding code already owns
/// `ModelStore`.
pub trait LocalStateModelStoreExt<T> {
    fn read_in<R>(
        &self,
        models: &ModelStore,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any;

    fn revision_in(&self, models: &ModelStore) -> Option<u64>
    where
        T: Any;

    fn value_in(&self, models: &ModelStore) -> Option<T>
    where
        T: Any + Clone;

    fn value_in_or(&self, models: &ModelStore, default: T) -> T
    where
        T: Any + Clone;

    fn value_in_or_else(&self, models: &ModelStore, f: impl FnOnce() -> T) -> T
    where
        T: Any + Clone;

    fn value_in_or_default(&self, models: &ModelStore) -> T
    where
        T: Any + Clone + Default;

    fn update_in(&self, models: &mut ModelStore, f: impl FnOnce(&mut T)) -> bool
    where
        T: Any;

    fn update_in_if(&self, models: &mut ModelStore, f: impl FnOnce(&mut T) -> bool) -> bool
    where
        T: Any;

    fn set_in(&self, models: &mut ModelStore, value: T) -> bool
    where
        T: Any;
}

impl<T> LocalStateModelStoreExt<T> for LocalState<T> {
    fn read_in<R>(
        &self,
        models: &ModelStore,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any,
    {
        models.read(&self.model, f)
    }

    fn revision_in(&self, models: &ModelStore) -> Option<u64>
    where
        T: Any,
    {
        models.revision(&self.model)
    }

    fn value_in(&self, models: &ModelStore) -> Option<T>
    where
        T: Any + Clone,
    {
        self.read_in(models, Clone::clone).ok()
    }

    fn value_in_or(&self, models: &ModelStore, default: T) -> T
    where
        T: Any + Clone,
    {
        self.value_in(models).unwrap_or(default)
    }

    fn value_in_or_else(&self, models: &ModelStore, f: impl FnOnce() -> T) -> T
    where
        T: Any + Clone,
    {
        self.value_in(models).unwrap_or_else(f)
    }

    fn value_in_or_default(&self, models: &ModelStore) -> T
    where
        T: Any + Clone + Default,
    {
        self.value_in(models).unwrap_or_default()
    }

    fn update_in(&self, models: &mut ModelStore, f: impl FnOnce(&mut T)) -> bool
    where
        T: Any,
    {
        models.update(&self.model, f).is_ok()
    }

    fn update_in_if(&self, models: &mut ModelStore, f: impl FnOnce(&mut T) -> bool) -> bool
    where
        T: Any,
    {
        models.update(&self.model, f).ok().unwrap_or(false)
    }

    fn set_in(&self, models: &mut ModelStore, value: T) -> bool
    where
        T: Any,
    {
        self.update_in(models, move |slot| *slot = value)
    }
}

/// Explicit `ElementContext` bridge for helper-heavy component or advanced surfaces.
///
/// Default app code should prefer `state.layout_value(cx)` / `state.paint_value(cx)` through
/// `AppUi`. This trait is intentionally omitted from `fret::app::prelude::*` and reexported from
/// `fret::advanced::prelude::*`; import it only when a helper already owns an `ElementContext`.
pub trait LocalStateElementContextExt<T: Any> {
    fn watch_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T>;

    fn paint_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T>;

    fn paint_value_in<'cx, 'm, 'a, H: UiHost>(&'m self, cx: &'cx mut ElementContext<'a, H>) -> T
    where
        T: Clone;

    fn paint_read_ref_in<'cx, 'm, 'a, H: UiHost, R>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
        f: impl FnOnce(&T) -> R,
    ) -> R;

    fn layout_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T>;

    fn layout_value_in<'cx, 'm, 'a, H: UiHost>(&'m self, cx: &'cx mut ElementContext<'a, H>) -> T
    where
        T: Clone;

    fn layout_read_ref_in<'cx, 'm, 'a, H: UiHost, R>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
        f: impl FnOnce(&T) -> R,
    ) -> R;

    fn hit_test_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T>;
}

impl<T: Any> LocalStateElementContextExt<T> for LocalState<T> {
    fn watch_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T> {
        WatchedState::new(cx, &self.model)
    }

    fn paint_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T> {
        self.watch_in(cx).paint()
    }

    fn paint_value_in<'cx, 'm, 'a, H: UiHost>(&'m self, cx: &'cx mut ElementContext<'a, H>) -> T
    where
        T: Clone,
    {
        self.paint_in(cx)
            .value()
            .expect("LocalState bridge code should always read initialized locals")
    }

    fn paint_read_ref_in<'cx, 'm, 'a, H: UiHost, R>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
        f: impl FnOnce(&T) -> R,
    ) -> R {
        self.paint_in(cx)
            .read_ref(f)
            .expect("LocalState bridge code should always read initialized locals")
    }

    fn layout_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T> {
        self.watch_in(cx).layout()
    }

    fn layout_value_in<'cx, 'm, 'a, H: UiHost>(&'m self, cx: &'cx mut ElementContext<'a, H>) -> T
    where
        T: Clone,
    {
        self.layout_in(cx)
            .value()
            .expect("LocalState bridge code should always read initialized locals")
    }

    fn layout_read_ref_in<'cx, 'm, 'a, H: UiHost, R>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
        f: impl FnOnce(&T) -> R,
    ) -> R {
        self.layout_in(cx)
            .read_ref(f)
            .expect("LocalState bridge code should always read initialized locals")
    }

    fn hit_test_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T> {
        self.watch_in(cx).hit_test()
    }
}
