//! Local view-owned state for the app-facing `View` authoring lane.

use std::any::Any;
#[cfg(feature = "shadcn")]
use std::sync::Arc;

use fret_runtime::{Model, ModelStore, ModelUpdateError};
use fret_ui::{ElementContext, Invalidation, UiHost};

use super::RenderContextAccess;

/// Default app-facing handle for view-owned local state.
///
/// `LocalState<T>` is the normal local-state story for app code on the `fret::app` lane.
/// The explicit raw-model and bridge helpers that still live on this type are intentionally
/// non-default:
///
/// - use `cx.state().local*` plus `layout_value(...)` / `paint_value(...)` for the default
///   app-authoring path,
/// - use [`AppUiRawModelExt::raw_model`] when code intentionally wants a raw `Model<T>` handle,
/// - use the bridge helpers below only when ownership or helper-context boundaries still require
///   direct `ModelStore`, `ElementContext`, or `Model<T>` access.
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
    /// This is the blessed constructor for driver/init/hybrid surfaces that already own
    /// `&mut ModelStore` (for example: manual window state, `UiAppDriver` init hooks, or
    /// render-root bridges that need a `LocalState<T>` handle before the first `AppUi` render).
    /// On the default `AppUi` lane, prefer `cx.state().local::<T>()` / `local_init(...)`.
    #[track_caller]
    pub fn new_in(models: &mut ModelStore, value: T) -> Self
    where
        T: Any,
    {
        Self {
            model: models.insert(value),
        }
    }

    /// Wrap an existing raw `Model<T>` handle as explicit `LocalState<T>` bridge state.
    ///
    /// This is primarily for advanced/manual surfaces that allocate tracked slots outside the
    /// default `AppUi` render loop (for example: manual `UiTree` drivers or hybrid runtime-owned
    /// window state) but still want to read/write that slot through the `LocalState<T>` helpers on
    /// the app-facing authoring lane.
    pub fn from_model(model: Model<T>) -> Self {
        Self { model }
    }

    /// Expose the underlying `Model<T>` as an explicit bridge.
    ///
    /// This exists for advanced/component/hybrid surfaces that intentionally still speak
    /// `Model<T>`. It is not the default app-authoring path.
    pub fn model(&self) -> &Model<T> {
        &self.model
    }

    /// Clone the underlying `Model<T>` as an explicit bridge.
    ///
    /// Prefer staying on `LocalState<T>` for default app code. Reach for this only when a widget,
    /// helper, or runtime-owned boundary intentionally needs a raw `Model<T>` handle.
    pub fn clone_model(&self) -> Model<T> {
        self.model.clone()
    }

    /// Read this local through an explicit `ModelStore` bridge.
    ///
    /// This is for code that already owns `ModelStore` and intentionally needs store-level access.
    /// Prefer tracked reads on `LocalState<T>` in ordinary render-time app code.
    pub fn read_in<R>(
        &self,
        models: &ModelStore,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any,
    {
        models.read(&self.model, f)
    }

    /// Query the underlying model revision through an explicit `ModelStore` bridge.
    ///
    /// This is primarily for advanced transactions, diagnostics, or helper surfaces that already
    /// operate on `ModelStore`.
    pub fn revision_in(&self, models: &ModelStore) -> Option<u64>
    where
        T: Any,
    {
        models.revision(&self.model)
    }

    /// Clone the current local value through an explicit `ModelStore` bridge read.
    ///
    /// This mirrors the render-time `watch(...).value_*` helpers for advanced store-side
    /// transactions that still need to read from `ModelStore` inside
    /// `cx.actions().models::<A>(...)`. It is not the normal render-loop read path for
    /// first-contact app code.
    pub fn value_in(&self, models: &ModelStore) -> Option<T>
    where
        T: Any + Clone,
    {
        self.read_in(models, Clone::clone).ok()
    }

    pub fn value_in_or(&self, models: &ModelStore, default: T) -> T
    where
        T: Any + Clone,
    {
        self.value_in(models).unwrap_or(default)
    }

    pub fn value_in_or_else(&self, models: &ModelStore, f: impl FnOnce() -> T) -> T
    where
        T: Any + Clone,
    {
        self.value_in(models).unwrap_or_else(f)
    }

    pub fn value_in_or_default(&self, models: &ModelStore) -> T
    where
        T: Any + Clone + Default,
    {
        self.value_in(models).unwrap_or_default()
    }

    /// Update this local slot through an explicit `ModelStore` transaction.
    ///
    /// This is a store-only write helper: it does **not** request redraw or mark the current
    /// view-cache root dirty by itself. Use it inside `cx.actions().models::<A>(...)` when the
    /// write participates in a broader model-store transaction, or prefer the grouped
    /// `cx.actions().local(&local).set::<A>(...)` / `.update::<A>(...)` /
    /// `.payload_update_if::<A>(...)` helpers when the local write itself should drive rerender.
    pub fn update_in(&self, models: &mut ModelStore, f: impl FnOnce(&mut T)) -> bool
    where
        T: Any,
    {
        models.update(&self.model, f).is_ok()
    }

    /// Update this local slot through an explicit `ModelStore` transaction and let the closure
    /// decide whether the write should count as `handled`.
    ///
    /// This is useful for tracked collections where the mutation may or may not actually change the
    /// slot (for example: toggle/remove by id). Missing model handles still return `false`.
    pub fn update_in_if(&self, models: &mut ModelStore, f: impl FnOnce(&mut T) -> bool) -> bool
    where
        T: Any,
    {
        models.update(&self.model, f).ok().unwrap_or(false)
    }

    /// Set this local slot through an explicit `ModelStore` transaction.
    ///
    /// Like `update_in(...)`, this only mutates the tracked slot; redraw + `notify()` remain the
    /// responsibility of the surrounding authoring surface unless a higher-level action helper
    /// owns the rerender rule.
    pub fn set_in(&self, models: &mut ModelStore, value: T) -> bool
    where
        T: Any,
    {
        self.update_in(models, move |slot| *slot = value)
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

    /// Observe/read this local from helper-heavy `ElementContext` surfaces.
    ///
    /// This is an explicit bridge for helpers that already operate on `ElementContext` and would
    /// otherwise have to drop down to `local.model()`. Prefer `watch(...)` on `AppUi` for the
    /// default app-authoring path.
    pub fn watch_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T>
    where
        T: Any,
    {
        WatchedState::new(cx, &self.model)
    }

    /// Convenience bridge over [`LocalState::watch_in`] for paint invalidation reads.
    pub fn paint_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T>
    where
        T: Any,
    {
        self.watch_in(cx).paint()
    }

    /// Read the current local value through a paint invalidation tracked read on helper-heavy
    /// `ElementContext` bridge surfaces.
    pub fn paint_value_in<'cx, 'm, 'a, H: UiHost>(&'m self, cx: &'cx mut ElementContext<'a, H>) -> T
    where
        T: Any + Clone,
    {
        self.paint_in(cx)
            .value()
            .expect("LocalState bridge code should always read initialized locals")
    }

    /// Read a derived value from this local through a paint invalidation tracked borrow on
    /// helper-heavy `ElementContext` bridge surfaces.
    pub fn paint_read_ref_in<'cx, 'm, 'a, H: UiHost, R>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
        f: impl FnOnce(&T) -> R,
    ) -> R
    where
        T: Any,
    {
        self.paint_in(cx)
            .read_ref(f)
            .expect("LocalState bridge code should always read initialized locals")
    }

    /// Convenience bridge over [`LocalState::watch_in`] for layout invalidation reads.
    pub fn layout_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T>
    where
        T: Any,
    {
        self.watch_in(cx).layout()
    }

    /// Read the current local value through a layout invalidation tracked read on helper-heavy
    /// `ElementContext` bridge surfaces.
    pub fn layout_value_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> T
    where
        T: Any + Clone,
    {
        self.layout_in(cx)
            .value()
            .expect("LocalState bridge code should always read initialized locals")
    }

    /// Read a derived value from this local through a layout invalidation tracked borrow on
    /// helper-heavy `ElementContext` bridge surfaces.
    pub fn layout_read_ref_in<'cx, 'm, 'a, H: UiHost, R>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
        f: impl FnOnce(&T) -> R,
    ) -> R
    where
        T: Any,
    {
        self.layout_in(cx)
            .read_ref(f)
            .expect("LocalState bridge code should always read initialized locals")
    }

    /// Convenience bridge over [`LocalState::watch_in`] for hit-test invalidation reads.
    pub fn hit_test_in<'cx, 'm, 'a, H: UiHost>(
        &'m self,
        cx: &'cx mut ElementContext<'a, H>,
    ) -> WatchedState<'cx, 'm, 'a, H, T>
    where
        T: Any,
    {
        self.watch_in(cx).hit_test()
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

impl<T> fret_ui_kit::declarative::form::IntoFormValueModel<T> for LocalState<T> {
    fn into_form_value_model(self) -> Model<T> {
        self.clone_model()
    }
}

impl<T> fret_ui_kit::declarative::form::IntoFormValueModel<T> for &LocalState<T> {
    fn into_form_value_model(self) -> Model<T> {
        self.clone_model()
    }
}

impl fret_ui_kit::declarative::table::IntoTableStateModel
    for LocalState<fret_ui_kit::headless::table::TableState>
{
    fn into_table_state_model(self) -> Model<fret_ui_kit::headless::table::TableState> {
        self.clone_model()
    }
}

impl fret_ui_kit::declarative::table::IntoTableStateModel
    for &LocalState<fret_ui_kit::headless::table::TableState>
{
    fn into_table_state_model(self) -> Model<fret_ui_kit::headless::table::TableState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoBoolModel for LocalState<bool> {
    fn into_bool_model(self) -> Model<bool> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoBoolModel for &LocalState<bool> {
    fn into_bool_model(self) -> Model<bool> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalBoolModel for LocalState<Option<bool>> {
    fn into_optional_bool_model(self) -> Model<Option<bool>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalBoolModel for &LocalState<Option<bool>> {
    fn into_optional_bool_model(self) -> Model<Option<bool>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFormStateModel
    for LocalState<fret_ui_kit::headless::form_state::FormState>
{
    fn into_form_state_model(self) -> Model<fret_ui_kit::headless::form_state::FormState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFormStateModel
    for &LocalState<fret_ui_kit::headless::form_state::FormState>
{
    fn into_form_state_model(self) -> Model<fret_ui_kit::headless::form_state::FormState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoCheckedStateModel
    for LocalState<fret_ui_headless::checked_state::CheckedState>
{
    fn into_checked_state_model(self) -> Model<fret_ui_headless::checked_state::CheckedState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoCheckedStateModel
    for &LocalState<fret_ui_headless::checked_state::CheckedState>
{
    fn into_checked_state_model(self) -> Model<fret_ui_headless::checked_state::CheckedState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoTextValueModel for LocalState<String> {
    fn into_text_value_model(self) -> Model<String> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoTextValueModel for &LocalState<String> {
    fn into_text_value_model(self) -> Model<String> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalTextValueModel for LocalState<Option<Arc<str>>> {
    fn into_optional_text_value_model(self) -> Model<Option<Arc<str>>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalTextValueModel for &LocalState<Option<Arc<str>>> {
    fn into_optional_text_value_model(self) -> Model<Option<Arc<str>>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoTextVecModel for LocalState<Vec<Arc<str>>> {
    fn into_text_vec_model(self) -> Model<Vec<Arc<str>>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoTextVecModel for &LocalState<Vec<Arc<str>>> {
    fn into_text_vec_model(self) -> Model<Vec<Arc<str>>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFloatValueModel for LocalState<f32> {
    fn into_float_value_model(self) -> Model<f32> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFloatValueModel for &LocalState<f32> {
    fn into_float_value_model(self) -> Model<f32> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalFloatValueModel for LocalState<Option<f32>> {
    fn into_optional_float_value_model(self) -> Model<Option<f32>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalFloatValueModel for &LocalState<Option<f32>> {
    fn into_optional_float_value_model(self) -> Model<Option<f32>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFloatVecModel for LocalState<Vec<f32>> {
    fn into_float_vec_model(self) -> Model<Vec<f32>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFloatVecModel for &LocalState<Vec<f32>> {
    fn into_float_vec_model(self) -> Model<Vec<f32>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoCalendarMonthModel
    for LocalState<fret_ui_kit::headless::calendar::CalendarMonth>
{
    fn into_calendar_month_model(self) -> Model<fret_ui_kit::headless::calendar::CalendarMonth> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoCalendarMonthModel
    for &LocalState<fret_ui_kit::headless::calendar::CalendarMonth>
{
    fn into_calendar_month_model(self) -> Model<fret_ui_kit::headless::calendar::CalendarMonth> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalDateModel for LocalState<Option<time::Date>> {
    fn into_optional_date_model(self) -> Model<Option<time::Date>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalDateModel for &LocalState<Option<time::Date>> {
    fn into_optional_date_model(self) -> Model<Option<time::Date>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoSolarHijriMonthModel
    for LocalState<fret_ui_shadcn::facade::SolarHijriMonth>
{
    fn into_solar_hijri_month_model(self) -> Model<fret_ui_shadcn::facade::SolarHijriMonth> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoSolarHijriMonthModel
    for &LocalState<fret_ui_shadcn::facade::SolarHijriMonth>
{
    fn into_solar_hijri_month_model(self) -> Model<fret_ui_shadcn::facade::SolarHijriMonth> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoU8ValueModel for LocalState<u8> {
    fn into_u8_value_model(self) -> Model<u8> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoU8ValueModel for &LocalState<u8> {
    fn into_u8_value_model(self) -> Model<u8> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoDateRangeSelectionModel
    for LocalState<fret_ui_kit::headless::calendar::DateRangeSelection>
{
    fn into_date_range_selection_model(
        self,
    ) -> Model<fret_ui_kit::headless::calendar::DateRangeSelection> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoDateRangeSelectionModel
    for &LocalState<fret_ui_kit::headless::calendar::DateRangeSelection>
{
    fn into_date_range_selection_model(
        self,
    ) -> Model<fret_ui_kit::headless::calendar::DateRangeSelection> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoDateVecModel for LocalState<Vec<time::Date>> {
    fn into_date_vec_model(self) -> Model<Vec<time::Date>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoDateVecModel for &LocalState<Vec<time::Date>> {
    fn into_date_vec_model(self) -> Model<Vec<time::Date>> {
        self.clone_model()
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
