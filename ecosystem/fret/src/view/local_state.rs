//! Local view-owned state for the app-facing `View` authoring lane.

use std::any::Any;

use fret_runtime::{Model, ModelStore, ModelUpdateError};
use fret_ui::{ElementContext, Invalidation, UiHost};

use super::RenderContextAccess;

mod adapters;
mod bridges;
pub use bridges::{LocalStateElementContextExt, LocalStateModelStoreExt, LocalStateRawModelExt};

/// Default app-facing handle for view-owned local state.
///
/// `LocalState<T>` is the normal local-state story for app code on the `fret::app` lane.
/// The explicit raw-model and bridge helpers that still live on this type are intentionally
/// non-default:
///
/// - use `cx.state().local*` plus `layout_value(...)` / `paint_value(...)` for the default
///   app-authoring path,
/// - use [`AppUiRawModelExt::raw_model`] when code intentionally wants a raw `Model<T>` handle,
/// - use `fret::advanced::raw` bridge helpers only when ownership or helper-context boundaries
///   still require direct `ModelStore`, `ElementContext`, or `Model<T>` access.
pub struct LocalState<T> {
    pub(super) model: Model<T>,
}

impl<T> Clone for LocalState<T> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
        }
    }
}

impl<T> LocalState<T> {
    /// Insert a new view-owned local slot into an existing `ModelStore`.
    ///
    /// This is the crate-internal constructor behind the public app and advanced raw seams.
    /// Default app code should use `AppLocalStateExt::local_state(...)` during `View::init` or
    /// `cx.state().local::<T>()` / `local_init(...)` during render. Manual surfaces that already
    /// own `&mut ModelStore` should use `fret::advanced::raw::local_state_in(...)`.
    #[track_caller]
    pub(crate) fn new_in(models: &mut ModelStore, value: T) -> Self
    where
        T: Any,
    {
        Self {
            model: models.insert(value),
        }
    }

    /// Update this local slot from an action dispatch and participate in the tracked-write
    /// rerender rule (`request_redraw(window)` + `notify(action_cx)`) when the write succeeds.
    pub(super) fn update_action(
        &self,
        host: &mut dyn fret_ui::action::UiFocusActionHost,
        action_cx: fret_ui::action::ActionCx,
        f: impl FnOnce(&mut T),
    ) -> bool
    where
        T: Any,
    {
        let handled = self.update_in(host.models_mut(), f);
        if handled {
            host.request_redraw(action_cx.window);
            host.notify(action_cx);
        }
        handled
    }

    /// Like `update_action(...)`, but the closure decides whether the mutation should count as
    /// `handled` before triggering redraw + `notify()`.
    pub(super) fn update_action_if(
        &self,
        host: &mut dyn fret_ui::action::UiFocusActionHost,
        action_cx: fret_ui::action::ActionCx,
        f: impl FnOnce(&mut T) -> bool,
    ) -> bool
    where
        T: Any,
    {
        let handled = self.update_in_if(host.models_mut(), f);
        if handled {
            host.request_redraw(action_cx.window);
            host.notify(action_cx);
        }
        handled
    }

    /// Set this local slot from an action dispatch and participate in the tracked-write rerender
    /// rule (`request_redraw(window)` + `notify(action_cx)`) when the write succeeds.
    pub(super) fn set_action(
        &self,
        host: &mut dyn fret_ui::action::UiFocusActionHost,
        action_cx: fret_ui::action::ActionCx,
        value: T,
    ) -> bool
    where
        T: Any,
    {
        self.update_action(host, action_cx, move |slot| *slot = value)
    }

    /// Read the current local value through a layout invalidation tracked read on the default app
    /// surface.
    ///
    /// `LocalState<T>` on the app lane always owns an inserted slot, so this keeps the invalidation
    /// phase explicit without repeating fallback noise at the call site.
    pub fn layout_value<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> T
    where
        T: Any + Clone,
        Cx: RenderContextAccess<'a, H>,
    {
        self.layout(cx)
            .value()
            .expect("LocalState-first app code should always read initialized locals")
    }

    /// Read a derived value from this local through a layout invalidation tracked borrow on the
    /// default app surface.
    ///
    /// Use this when app code only needs a projection (for example: `len()`, membership checks, or
    /// lightweight formatting) and should not clone the entire `T` just to compute that result.
    /// Keep raw `layout(cx).read_ref(...)` when you intentionally want the explicit tracked-read
    /// builder.
    pub fn layout_read_ref<'a, H: UiHost + 'a, Cx, R>(
        &self,
        cx: &mut Cx,
        f: impl FnOnce(&T) -> R,
    ) -> R
    where
        T: Any,
        Cx: RenderContextAccess<'a, H>,
    {
        self.layout(cx)
            .read_ref(f)
            .expect("LocalState-first app code should always read initialized locals")
    }

    /// Read the current local value through a paint invalidation tracked read on the default app
    /// surface.
    ///
    /// Keep raw `watch(...).paint().value_*` when you intentionally want the explicit builder; use
    /// this for ordinary initialized app locals that only need the paint-phase value.
    pub fn paint_value<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> T
    where
        T: Any + Clone,
        Cx: RenderContextAccess<'a, H>,
    {
        self.paint(cx)
            .value()
            .expect("LocalState-first app code should always read initialized locals")
    }

    /// Read a derived value from this local through a paint invalidation tracked borrow on the
    /// default app surface.
    ///
    /// Use this when paint-time app code only needs a projection and should not clone the whole
    /// slot. Keep raw `paint(cx).read_ref(...)` when you intentionally want the explicit
    /// tracked-read builder.
    pub fn paint_read_ref<'a, H: UiHost + 'a, Cx, R>(
        &self,
        cx: &mut Cx,
        f: impl FnOnce(&T) -> R,
    ) -> R
    where
        T: Any,
        Cx: RenderContextAccess<'a, H>,
    {
        self.paint(cx)
            .read_ref(f)
            .expect("LocalState-first app code should always read initialized locals")
    }

    pub fn watch<'watch, 'a, H: UiHost + 'a, Cx>(
        &'watch self,
        cx: &'watch mut Cx,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>
    where
        T: Any,
        Cx: RenderContextAccess<'a, H>,
    {
        WatchedState::new(cx.elements(), &self.model)
    }
}

/// App-facing constructor for local state created during `View::init`.
///
/// This keeps default examples on the app facade when they need a `LocalState<T>` handle before
/// the first `AppUi` render. Use `cx.state().local*` inside render; use
/// `fret::advanced::raw::local_state_in(...)` for advanced/manual surfaces that already own a raw
/// `ModelStore`.
pub trait AppLocalStateExt {
    #[track_caller]
    fn local_state<T>(&mut self, value: T) -> LocalState<T>
    where
        T: Any;
}

impl AppLocalStateExt for crate::app::App {
    #[track_caller]
    fn local_state<T>(&mut self, value: T) -> LocalState<T>
    where
        T: Any,
    {
        LocalState::new_in(self.models_mut(), value)
    }
}

/// App-facing transaction entry for local state writes outside render.
///
/// Use this when app-owned callbacks or function-driver hooks have `&mut App` but should still
/// stay on the LocalState-first surface instead of reopening raw `ModelStore` access. Render-time
/// action handlers should usually prefer `cx.actions().local(...)` or
/// `cx.actions().locals_with(...)`.
pub trait AppLocalStateTxnExt {
    fn local_state_txn<R>(&mut self, f: impl FnOnce(&mut LocalStateTxn<'_>) -> R) -> R;
}

impl AppLocalStateTxnExt for crate::app::App {
    fn local_state_txn<R>(&mut self, f: impl FnOnce(&mut LocalStateTxn<'_>) -> R) -> R {
        let mut tx = LocalStateTxn {
            models: self.models_mut(),
        };
        f(&mut tx)
    }
}

/// Action-host transaction entry for local state writes in app-facing activation callbacks.
///
/// Use this for widget or ecosystem callbacks that receive `&mut dyn UiActionHost` rather than
/// `&mut App`. It keeps callback code on the LocalState-first surface without reopening the raw
/// `ModelStore` bridge.
pub trait UiActionHostLocalStateTxnExt {
    fn local_state_txn<R>(&mut self, f: impl FnOnce(&mut LocalStateTxn<'_>) -> R) -> R;
}

impl<H> UiActionHostLocalStateTxnExt for H
where
    H: fret_ui::action::UiActionHost + ?Sized,
{
    fn local_state_txn<R>(&mut self, f: impl FnOnce(&mut LocalStateTxn<'_>) -> R) -> R {
        let mut tx = LocalStateTxn {
            models: self.models_mut(),
        };
        f(&mut tx)
    }
}

/// A narrow, LocalState-focused transaction wrapper used to keep the default authoring surface
/// free of direct `ModelStore` plumbing.
///
/// This is intentionally *not* a general-purpose model transaction API. If you need to coordinate
/// across shared `Model<T>` graphs, use `cx.actions().models::<A>(...)` directly.
#[doc(hidden)]
pub struct LocalStateTxn<'a> {
    pub(super) models: &'a mut ModelStore,
}

impl<'a> LocalStateTxn<'a> {
    /// Borrow an existing action `ModelStore` as a local-state transaction.
    ///
    /// This is for mixed handlers that already receive `ModelStore` for shared model or mutation
    /// coordination but still want app-local reads/writes to stay on the LocalState surface.
    pub fn with_model_store<R>(
        models: &mut ModelStore,
        f: impl FnOnce(&mut LocalStateTxn<'_>) -> R,
    ) -> R {
        let mut tx = LocalStateTxn { models };
        f(&mut tx)
    }

    /// Read the current value from an initialized local slot.
    ///
    /// `LocalState<T>` on the app lane always owns an inserted model slot, so ordinary
    /// `locals_with((...)).on::<A>(...)` transactions can read with `tx.value(&local)` instead of
    /// reopening fallback noise at every call site.
    pub fn value<T: Any + Clone>(&self, local: &LocalState<T>) -> T {
        local
            .value_in(self.models)
            .expect("LocalState-first action transactions should always read initialized locals")
    }

    pub fn value_or<T: Any + Clone>(&self, local: &LocalState<T>, default: T) -> T {
        local.value_in_or(self.models, default)
    }

    pub fn value_or_else<T: Any + Clone>(&self, local: &LocalState<T>, f: impl FnOnce() -> T) -> T {
        local.value_in_or_else(self.models, f)
    }

    pub fn value_or_default<T: Any + Clone + Default>(&self, local: &LocalState<T>) -> T {
        local.value_in_or_default(self.models)
    }

    pub fn read_ref<T: Any, R>(
        &self,
        local: &LocalState<T>,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, ModelUpdateError> {
        local.read_in(self.models, f)
    }

    pub fn set<T: Any>(&mut self, local: &LocalState<T>, value: T) -> bool {
        local.set_in(self.models, value)
    }

    pub fn update<T: Any>(&mut self, local: &LocalState<T>, f: impl FnOnce(&mut T)) -> bool {
        local.update_in(self.models, f)
    }

    pub fn update_if<T: Any>(
        &mut self,
        local: &LocalState<T>,
        f: impl FnOnce(&mut T) -> bool,
    ) -> bool {
        local.update_in_if(self.models, f)
    }

    /// Update an existing shared `Model<T>` while staying inside a LocalState-first action
    /// transaction.
    ///
    /// Prefer [`Self::update`] for ordinary view-owned state. Use this only when a default app
    /// surface already receives a shared model handle from a parent component and should not reopen
    /// the broader `ModelStore` action API just to coordinate one shared readout.
    pub fn update_shared_model<T: Any>(
        &mut self,
        model: &Model<T>,
        f: impl FnOnce(&mut T),
    ) -> bool {
        self.models.update(model, f).is_ok()
    }
}

/// Hidden capture helper for `locals_with(...)`.
///
/// This lets first-party and downstream app code pass `&LocalState<T>` handles directly into a
/// registered action closure without repeating a `LocalState::clone(...)` prelude at every call
/// site.
#[doc(hidden)]
pub trait LocalActionCapture {
    type Owned: Clone + 'static;

    fn capture_owned(&self) -> Self::Owned;
}

impl<T: Any> LocalActionCapture for LocalState<T> {
    type Owned = LocalState<T>;

    fn capture_owned(&self) -> Self::Owned {
        self.clone()
    }
}

impl<T: Any> LocalActionCapture for &LocalState<T> {
    type Owned = LocalState<T>;

    fn capture_owned(&self) -> Self::Owned {
        (*self).clone()
    }
}

macro_rules! impl_local_action_capture_tuple {
    ($(($($name:ident $idx:tt),+)),+ $(,)?) => {
        $(
            impl<$($name),+> LocalActionCapture for ($($name,)+)
            where
                $($name: LocalActionCapture,)+
            {
                type Owned = ($($name::Owned,)+);

                fn capture_owned(&self) -> Self::Owned {
                    ($(self.$idx.capture_owned(),)+)
                }
            }
        )+
    };
}

impl_local_action_capture_tuple!(
    (A 0),
    (A 0, B 1),
    (A 0, B 1, C 2),
    (A 0, B 1, C 2, D 3),
    (A 0, B 1, C 2, D 3, E 4),
    (A 0, B 1, C 2, D 3, E 4, F 5),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7),
);

/// Explicit tracked-read builder returned by `watch(...)` / `layout(...)` / `paint(...)`.
///
/// Unlike the grouped namespace carrier types, this stays visible on purpose: it owns the
/// user-facing tracked-read chain (`paint/layout/hit_test`, `value_*`, `observe`, `revision`,
/// `read_ref`, `read`) rather than acting as a purely structural callback or namespace wrapper.
#[must_use]
pub struct WatchedState<'cx, 'm, 'a, H: UiHost, T: Any> {
    cx: &'cx mut ElementContext<'a, H>,
    model: &'m Model<T>,
    invalidation: Invalidation,
}

impl<'cx, 'm, 'a, H: UiHost, T: Any> WatchedState<'cx, 'm, 'a, H, T> {
    pub(super) fn new(cx: &'cx mut ElementContext<'a, H>, model: &'m Model<T>) -> Self {
        Self {
            cx,
            model,
            invalidation: Invalidation::Paint,
        }
    }

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

/// Shared read-side ergonomics for both `LocalState<T>` and explicit `Model<T>` handles.
///
/// Prefer `LocalState::layout_value(...)` / `paint_value(...)` for ordinary initialized app-lane
/// locals, or the shorter tracked-read chains such as `state.layout(cx).value_*` /
/// `state.paint(cx).value_*` when you intentionally want the explicit builder. Keep raw
/// `watch(cx)` when you need custom invalidation, `observe()`, `revision()`, or direct `read*()`
/// access on the tracked-read builder.
pub trait TrackedStateExt<T: Any> {
    fn watch<'watch, 'a, H: UiHost + 'a, Cx>(
        &'watch self,
        cx: &'watch mut Cx,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>
    where
        Cx: RenderContextAccess<'a, H>;

    fn paint<'watch, 'a, H: UiHost + 'a, Cx>(
        &'watch self,
        cx: &'watch mut Cx,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        self.watch(cx).paint()
    }

    fn layout<'watch, 'a, H: UiHost + 'a, Cx>(
        &'watch self,
        cx: &'watch mut Cx,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        self.watch(cx).layout()
    }

    fn hit_test<'watch, 'a, H: UiHost + 'a, Cx>(
        &'watch self,
        cx: &'watch mut Cx,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        self.watch(cx).hit_test()
    }
}

impl<T: Any> TrackedStateExt<T> for LocalState<T> {
    fn watch<'watch, 'a, H: UiHost + 'a, Cx>(
        &'watch self,
        cx: &'watch mut Cx,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        WatchedState::new(cx.elements(), &self.model)
    }
}

impl<T: Any> TrackedStateExt<T> for Model<T> {
    fn watch<'watch, 'a, H: UiHost + 'a, Cx>(
        &'watch self,
        cx: &'watch mut Cx,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        WatchedState::new(cx.elements(), self)
    }
}

#[cfg(feature = "state-query")]
impl<T: 'static> TrackedStateExt<fret_query::QueryState<T>> for fret_query::QueryHandle<T> {
    fn watch<'watch, 'a, H: UiHost + 'a, Cx>(
        &'watch self,
        cx: &'watch mut Cx,
    ) -> WatchedState<'watch, 'watch, 'a, H, fret_query::QueryState<T>>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        WatchedState::new(cx.elements(), self.model())
    }
}

#[cfg(feature = "state-mutation")]
impl<TIn: 'static, TOut: 'static> TrackedStateExt<fret_mutation::MutationState<TIn, TOut>>
    for fret_mutation::MutationHandle<TIn, TOut>
{
    fn watch<'watch, 'a, H: UiHost + 'a, Cx>(
        &'watch self,
        cx: &'watch mut Cx,
    ) -> WatchedState<'watch, 'watch, 'a, H, fret_mutation::MutationState<TIn, TOut>>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        WatchedState::new(cx.elements(), self.model())
    }
}
