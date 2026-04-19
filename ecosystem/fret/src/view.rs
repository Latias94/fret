//! View authoring runtime (ecosystem-level).
//!
//! This module provides a cohesive authoring loop aligned with ADR 0308:
//! - a stateful `View` object renders into the app-facing `Ui` alias (backed by the existing
//!   declarative IR),
//! - views can register typed action handlers (action-first),
//! - hook-style helpers compose existing mechanism contracts (models + observation).
//!
//! v1 notes:
//! - the explicit raw-model hook seam (`AppUiRawModelExt::raw_model<T>()`) currently returns a
//!   `Model<T>` allocated in the app-owned model store. This keeps event handlers object-safe
//!   (they only receive `UiActionHost`) while still providing view-local state ergonomics.
//! - The view runtime is intentionally additive and lives in `ecosystem/fret` (not kernel).

use std::any::Any;
#[cfg(any(feature = "state-mutation", feature = "state-query"))]
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::{Model, ModelStore, ModelUpdateError};
use fret_ui::action::{ActionCx, OnActivate, OnCommand, OnCommandAvailability, UiActionHost};
use fret_ui::{ElementContext, Invalidation, UiHost};

/// A stateful view object that renders into the existing declarative IR (`Ui`).
pub trait View: 'static {
    /// Initialize the view for a specific window.
    fn init(app: &mut crate::app::App, window: crate::WindowId) -> Self
    where
        Self: Sized;

    /// Render the view into declarative UI.
    fn render(&mut self, cx: &mut crate::AppUi<'_, '_>) -> crate::Ui;
}

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
    model: Model<T>,
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
    fn update_action(
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
    fn update_action_if(
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
    fn set_action(
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
    models: &'a mut ModelStore,
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
    fn new(cx: &'cx mut ElementContext<'a, H>, model: &'m Model<T>) -> Self {
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

/// Explicit render-authoring helper capability for app-facing extracted helper functions.
///
/// This keeps helper signatures on one named lane without forcing them to accept the full `AppUi`
/// surface or a raw `ElementContext<'_, H>` type directly.
pub trait RenderContextAccess<'a, H: UiHost + 'a>: fret_ui::ElementContextAccess<'a, H> {
    fn app<'b>(&'b mut self) -> &'b H
    where
        'a: 'b,
    {
        &*self.elements().app
    }

    fn app_mut<'b>(&'b mut self) -> &'b mut H
    where
        'a: 'b,
    {
        &mut *self.elements().app
    }

    fn window_id(&mut self) -> AppWindowId {
        self.elements().window
    }

    fn environment_viewport_bounds(&mut self, invalidation: Invalidation) -> fret_core::Rect {
        self.elements().environment_viewport_bounds(invalidation)
    }

    fn with_theme<R>(&mut self, f: impl FnOnce(&fret_ui::Theme) -> R) -> R {
        f(self.elements().theme())
    }

    fn theme_snapshot(&mut self) -> fret_ui::ThemeSnapshot {
        self.with_theme(|theme| theme.snapshot())
    }
}

impl<'a, H: UiHost + 'a, T> RenderContextAccess<'a, H> for T where
    T: fret_ui::ElementContextAccess<'a, H>
{
}

/// Named default extracted-helper render lane for ordinary `fret` app code.
///
/// This is the app-facing façade over `RenderContextAccess<'a, crate::app::App>` so new helper
/// signatures can name the default lane directly without reopening the raw `UiCx` alias or
/// spelling the generic host parameter at every callsite.
pub trait AppRenderContext<'a>: RenderContextAccess<'a, crate::app::App> {}

impl<'a, T> AppRenderContext<'a> for T where T: RenderContextAccess<'a, crate::app::App> {}

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
    for LocalState<fret_ui_kit::primitives::checkbox::CheckedState>
{
    fn into_checked_state_model(self) -> Model<fret_ui_kit::primitives::checkbox::CheckedState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoCheckedStateModel
    for &LocalState<fret_ui_kit::primitives::checkbox::CheckedState>
{
    fn into_checked_state_model(self) -> Model<fret_ui_kit::primitives::checkbox::CheckedState> {
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

/// App-facing layout-phase convenience reads for query handles on the default `fret` lane.
///
/// This intentionally collapses only the repeated `layout(...).value_or_default()` fallback for the
/// ordinary app path. Query creation (`key`, `policy`, `fetch`) and lifecycle branching
/// (`status` / `data` / `error`) stay explicit.
#[cfg(feature = "state-query")]
pub trait QueryHandleReadLayoutExt<T: 'static> {
    fn read_layout<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> fret_query::QueryState<T>
    where
        Cx: RenderContextAccess<'a, H>;
}

#[cfg(feature = "state-query")]
impl<T: 'static> QueryHandleReadLayoutExt<T> for fret_query::QueryHandle<T> {
    fn read_layout<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> fret_query::QueryState<T>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        TrackedStateExt::layout(self, cx).value_or_default()
    }
}

/// App-facing layout-phase convenience reads for mutation handles on the default `fret` lane.
#[cfg(feature = "state-mutation")]
pub trait MutationHandleReadLayoutExt<TIn: 'static, TOut: 'static> {
    fn read_layout<'a, H: UiHost + 'a, Cx>(
        &self,
        cx: &mut Cx,
    ) -> fret_mutation::MutationState<TIn, TOut>
    where
        Cx: RenderContextAccess<'a, H>;
}

#[cfg(feature = "state-mutation")]
impl<TIn: 'static, TOut: 'static> MutationHandleReadLayoutExt<TIn, TOut>
    for fret_mutation::MutationHandle<TIn, TOut>
{
    fn read_layout<'a, H: UiHost + 'a, Cx>(
        &self,
        cx: &mut Cx,
    ) -> fret_mutation::MutationState<TIn, TOut>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        TrackedStateExt::layout(self, cx).value_or_default()
    }
}

/// LocalState-aware selector dependency helpers for the explicit `fret::selector` lane.
///
/// This keeps `fret-selector` portable while still letting LocalState-first app code build
/// dependency signatures without bouncing through `clone_model()` or `local.model()`.
#[cfg(feature = "state-selector")]
pub(crate) trait LocalSelectorDepsBuilderExt {
    fn local_rev_invalidation<T: Any>(
        &mut self,
        local: &LocalState<T>,
        invalidation: Invalidation,
    ) -> &mut Self;
}

#[cfg(feature = "state-selector")]
impl<'cx, 'a, H: UiHost> LocalSelectorDepsBuilderExt
    for fret_selector::ui::DepsBuilder<'cx, 'a, H>
{
    fn local_rev_invalidation<T: Any>(
        &mut self,
        local: &LocalState<T>,
        invalidation: Invalidation,
    ) -> &mut Self {
        self.model_rev_invalidation(local.model(), invalidation)
    }
}

#[cfg(feature = "state-selector")]
fn local_selector_value_in<T: Any + Clone, H: UiHost>(
    local: &LocalState<T>,
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> T {
    let value = match invalidation {
        Invalidation::Paint => local.paint_in(cx).value(),
        Invalidation::Layout => local.layout_in(cx).value(),
        Invalidation::HitTest | Invalidation::HitTestOnly => local.hit_test_in(cx).value(),
    };
    value.expect("LocalState-first selector inputs should always resolve a tracked value")
}

#[cfg(feature = "state-selector")]
fn model_selector_value_in<T: Any + Clone, H: UiHost>(
    model: &Model<T>,
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> T {
    let value = match invalidation {
        Invalidation::Paint => cx.get_model_cloned(model, Invalidation::Paint),
        Invalidation::Layout => cx.get_model_cloned(model, Invalidation::Layout),
        Invalidation::HitTest | Invalidation::HitTestOnly => {
            cx.get_model_cloned(model, Invalidation::HitTest)
        }
    };
    value.expect("Model selector inputs should always resolve a tracked value")
}

/// App-facing LocalState selector inputs for the grouped `cx.data()` lane.
///
/// This trait is intentionally hidden from docs because app authors should use the methods on
/// `cx.data()` rather than naming the trait directly.
#[cfg(feature = "state-selector")]
#[doc(hidden)]
pub trait LocalSelectorLayoutInputs<'a, H: UiHost>: Copy {
    type Values;

    fn deps_in(
        self,
        cx: &mut ElementContext<'a, H>,
        invalidation: Invalidation,
    ) -> fret_selector::DepsSignature;

    fn values_in(self, cx: &mut ElementContext<'a, H>, invalidation: Invalidation) -> Self::Values;
}

#[cfg(feature = "state-selector")]
impl<'a, H: UiHost, T: Any + Clone> LocalSelectorLayoutInputs<'a, H> for &LocalState<T> {
    type Values = T;

    fn deps_in(
        self,
        cx: &mut ElementContext<'a, H>,
        invalidation: Invalidation,
    ) -> fret_selector::DepsSignature {
        let mut deps = fret_selector::ui::DepsBuilder::new(cx);
        deps.local_rev_invalidation(self, invalidation);
        deps.finish()
    }

    fn values_in(self, cx: &mut ElementContext<'a, H>, invalidation: Invalidation) -> Self::Values {
        local_selector_value_in(self, cx, invalidation)
    }
}

#[cfg(feature = "state-selector")]
macro_rules! impl_local_selector_inputs_tuple {
    ($(($($name:ident:$idx:tt),+)),+ $(,)?) => {
        $(
            impl<'a, H: UiHost, $($name: Any + Clone),+> LocalSelectorLayoutInputs<'a, H>
                for ($(&LocalState<$name>,)+)
            {
                type Values = ($($name,)+);

                fn deps_in(
                    self,
                    cx: &mut ElementContext<'a, H>,
                    invalidation: Invalidation,
                ) -> fret_selector::DepsSignature {
                    let mut deps = fret_selector::ui::DepsBuilder::new(cx);
                    $(deps.local_rev_invalidation(self.$idx, invalidation);)+
                    deps.finish()
                }

                fn values_in(
                    self,
                    cx: &mut ElementContext<'a, H>,
                    invalidation: Invalidation,
                ) -> Self::Values {
                    ($(local_selector_value_in(self.$idx, cx, invalidation),)+)
                }
            }
        )+
    };
}

#[cfg(feature = "state-selector")]
impl_local_selector_inputs_tuple!(
    (A:0, B:1),
    (A:0, B:1, C:2),
    (A:0, B:1, C:2, D:3),
    (A:0, B:1, C:2, D:3, E:4),
    (A:0, B:1, C:2, D:3, E:4, F:5),
);

/// Explicit shared-`Model<T>` selector inputs for grouped derived reads on the app-facing lane.
///
/// This keeps the advanced/manual path out of raw `selector(deps, compute)` boilerplate when the
/// state already lives in an explicit `Model<T>` bag instead of `LocalState<T>`.
#[cfg(feature = "state-selector")]
#[doc(hidden)]
pub trait ModelSelectorInputs<'a, H: UiHost>: Copy {
    type Values;

    fn deps_in(
        self,
        cx: &mut ElementContext<'a, H>,
        invalidation: Invalidation,
    ) -> fret_selector::DepsSignature;

    fn values_in(self, cx: &mut ElementContext<'a, H>, invalidation: Invalidation) -> Self::Values;
}

#[cfg(feature = "state-selector")]
impl<'a, H: UiHost, T: Any + Clone> ModelSelectorInputs<'a, H> for &Model<T> {
    type Values = T;

    fn deps_in(
        self,
        cx: &mut ElementContext<'a, H>,
        invalidation: Invalidation,
    ) -> fret_selector::DepsSignature {
        let mut deps = fret_selector::ui::DepsBuilder::new(cx);
        deps.model_rev_invalidation(self, invalidation);
        deps.finish()
    }

    fn values_in(self, cx: &mut ElementContext<'a, H>, invalidation: Invalidation) -> Self::Values {
        model_selector_value_in(self, cx, invalidation)
    }
}

#[cfg(feature = "state-selector")]
macro_rules! impl_model_selector_inputs_tuple {
    ($(($($name:ident:$idx:tt),+)),+ $(,)?) => {
        $(
            impl<'a, H: UiHost, $($name: Any + Clone),+> ModelSelectorInputs<'a, H>
                for ($(&Model<$name>,)+)
            {
                type Values = ($($name,)+);

                fn deps_in(
                    self,
                    cx: &mut ElementContext<'a, H>,
                    invalidation: Invalidation,
                ) -> fret_selector::DepsSignature {
                    let mut deps = fret_selector::ui::DepsBuilder::new(cx);
                    $(deps.model_rev_invalidation(self.$idx, invalidation);)+
                    deps.finish()
                }

                fn values_in(
                    self,
                    cx: &mut ElementContext<'a, H>,
                    invalidation: Invalidation,
                ) -> Self::Values {
                    ($(model_selector_value_in(self.$idx, cx, invalidation),)+)
                }
            }
        )+
    };
}

#[cfg(feature = "state-selector")]
impl_model_selector_inputs_tuple!(
    (A:0, B:1),
    (A:0, B:1, C:2),
    (A:0, B:1, C:2, D:3),
    (A:0, B:1, C:2, D:3, E:4),
    (A:0, B:1, C:2, D:3, E:4, F:5),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8, J:9),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8, J:9, K:10),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8, J:9, K:10, L:11),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8, J:9, K:10, L:11, M:12),
);

/// Per-frame view construction context passed to [`View::render`].
///
/// This is a thin wrapper over [`ElementContext`] that:
/// - provides grouped default-path helpers (`state`, `actions`, `data`, `effects`),
/// - collects action handlers for installation at a chosen root element,
/// - and keeps the component/internal identity lane behind an explicit `elements()` escape hatch.
///
/// The default app lane intentionally does not expose helper-local slot/model primitives such as
/// `slot_state(...)` or `local_model(...)` directly.
///
/// ```compile_fail
/// use fret::AppUi;
///
/// fn wrong(cx: &mut AppUi<'_, '_>) {
///     let _ = cx.local_model(|| false);
/// }
/// ```
///
/// Reach for `cx.state().local*` on the default app lane, or call `cx.elements()` explicitly when
/// advanced/component-heavy code intentionally needs the lower-level `ElementContext` substrate.
pub struct AppUi<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    action_root: fret_ui::GlobalElementId,
    action_handlers: crate::actions::ActionHandlerTable,
    action_handlers_used: bool,
}

#[doc(hidden)]
pub trait AppUiComponentLaneRequiresExplicitElementsEscapeHatch {}

/// Explicit raw-model state hooks that intentionally stay off the default app authoring surface.
///
/// This trait is intentionally omitted from `fret::app::prelude::*` and reexported from
/// `fret::advanced::prelude::*`.
///
/// Import it explicitly when advanced code still wants a stable callsite-keyed raw `Model<T>`
/// handle rather than the grouped `cx.state().local*` surface. For loop/dynamic callsites, wrap
/// `raw_model::<T>()` in `cx.keyed(...)` instead of relying on a separate keyed alias.
pub trait AppUiRawModelExt {
    #[track_caller]
    fn raw_model<T>(&mut self) -> Model<T>
    where
        T: Any + Default;
}

impl<'cx, 'a, H: UiHost> AppUiRawModelExt for AppUi<'cx, 'a, H> {
    #[track_caller]
    fn raw_model<T>(&mut self) -> Model<T>
    where
        T: Any + Default,
    {
        self.raw_model_with(T::default)
    }
}

/// Explicit raw action-registration hooks that intentionally stay off the default app authoring
/// surface.
///
/// This trait is intentionally omitted from `fret::app::prelude::*` and reexported from
/// `fret::advanced::prelude::*`.
///
/// Import it explicitly when advanced/manual-assembly code intentionally wants raw typed handler
/// registration rather than the grouped `cx.actions()` helpers. Model/local mutation shorthands
/// stay on the grouped default lane or on explicit store transactions; this trait keeps only the
/// raw host-facing registration hooks.
pub trait AppUiRawActionNotifyExt {
    /// Register a typed unit action handler that requests redraw + notifies on `handled=true`.
    ///
    /// This is a small ergonomics helper: most action handlers that mutate models/state need both
    /// `request_redraw(window)` and `notify(action_cx)` to participate in the view-cache closure.
    fn on_action_notify<A: crate::TypedAction>(
        &mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
    );

    /// Register a typed payload action handler that requests redraw + notifies on `handled=true`.
    fn on_payload_action_notify<A: crate::actions::TypedPayloadAction>(
        &mut self,
        f: impl Fn(
            &mut dyn fret_ui::action::UiFocusActionHost,
            fret_ui::action::ActionCx,
            A::Payload,
        ) -> bool
        + 'static,
    );
}

impl<'cx, 'a, H: UiHost> AppUiRawActionNotifyExt for AppUi<'cx, 'a, H> {
    fn on_action_notify<A: crate::TypedAction>(
        &mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
    ) {
        self.register_action_handler::<A>(move |host, action_cx| {
            let handled = f(host, action_cx);
            if handled {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
            handled
        });
    }

    fn on_payload_action_notify<A: crate::actions::TypedPayloadAction>(
        &mut self,
        f: impl Fn(
            &mut dyn fret_ui::action::UiFocusActionHost,
            fret_ui::action::ActionCx,
            A::Payload,
        ) -> bool
        + 'static,
    ) {
        self.register_payload_action_handler::<A>(move |host, action_cx, payload| {
            let handled = f(host, action_cx, payload);
            if handled {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
            handled
        });
    }
}

/// Grouped LocalState-first helpers for the default app authoring surface.
#[doc(hidden)]
pub struct AppUiState<'view, 'cx, 'a, H: UiHost> {
    cx: &'view mut AppUi<'cx, 'a, H>,
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

/// Grouped action/effect registration helpers for the default app authoring surface.
#[doc(hidden)]
pub struct AppUiActions<'view, 'cx, 'a, H: UiHost> {
    cx: &'view mut AppUi<'cx, 'a, H>,
}

#[doc(hidden)]
pub struct AppUiActionLocal<'view, 'cx, 'a, H: UiHost, T> {
    cx: &'view mut AppUi<'cx, 'a, H>,
    local: LocalState<T>,
}

/// Grouped action/effect registration helpers for extracted `AppComponentCx` child builders on the default
/// app surface.
#[doc(hidden)]
pub struct UiCxActions<'cx, 'a> {
    cx: &'cx mut ElementContext<'a, crate::app::App>,
}

#[doc(hidden)]
pub struct UiCxActionLocal<'cx, 'a, T> {
    cx: &'cx mut ElementContext<'a, crate::app::App>,
    local: LocalState<T>,
}

#[doc(hidden)]
pub struct AppUiLocalsWith<'view, 'cx, 'a, H: UiHost, C> {
    cx: &'view mut AppUi<'cx, 'a, H>,
    captures: C,
}

#[doc(hidden)]
pub struct UiCxLocalsWith<'cx, 'a, C> {
    cx: &'cx mut ElementContext<'a, crate::app::App>,
    captures: C,
}

impl<'view, 'cx, 'a, H: UiHost, C> AppUiLocalsWith<'view, 'cx, 'a, H, C>
where
    C: Clone + 'static,
{
    pub fn on<A>(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>, C) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        let captures = self.captures;
        self.cx.on_action_notify::<A>(move |host, _action_cx| {
            let mut tx = LocalStateTxn {
                models: host.models_mut(),
            };
            f(&mut tx, captures.clone())
        });
    }
}

impl<'view, 'cx, 'a, H: UiHost, T> AppUiActionLocal<'view, 'cx, 'a, H, T>
where
    T: Any,
{
    pub fn update<A>(self, update: impl Fn(&mut T) + 'static)
    where
        A: crate::TypedAction,
    {
        let local = self.local;
        self.cx
            .register_action_handler::<A>(move |host, action_cx| {
                local.update_action(host, action_cx, |value| update(value))
            });
    }

    pub fn set<A>(self, value: T)
    where
        A: crate::TypedAction,
        T: Clone,
    {
        let local = self.local;
        self.cx
            .register_action_handler::<A>(move |host, action_cx| {
                local.set_action(host, action_cx, value.clone())
            });
    }

    pub fn payload_update_if<A>(self, update: impl Fn(&mut T, A::Payload) -> bool + 'static)
    where
        A: crate::actions::TypedPayloadAction,
    {
        let local = self.local;
        self.cx
            .register_payload_action_handler::<A>(move |host, action_cx, payload| {
                local.update_action_if(host, action_cx, |value| update(value, payload))
            });
    }
}

impl<'view, 'cx, 'a, H: UiHost> AppUiActionLocal<'view, 'cx, 'a, H, bool> {
    pub fn toggle_bool<A>(self)
    where
        A: crate::TypedAction,
    {
        let local = self.local;
        self.cx
            .register_action_handler::<A>(move |host, action_cx| {
                local.update_action(host, action_cx, |value| *value = !*value)
            });
    }
}

impl<'cx, 'a, C> UiCxLocalsWith<'cx, 'a, C>
where
    C: Clone + 'static,
{
    pub fn on<A>(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>, C) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        let captures = self.captures;
        uicx_on_action_notify::<A>(self.cx, move |host, _action_cx| {
            let mut tx = LocalStateTxn {
                models: host.models_mut(),
            };
            f(&mut tx, captures.clone())
        });
    }
}

impl<'cx, 'a, T> UiCxActionLocal<'cx, 'a, T>
where
    T: Any,
{
    pub fn update<A>(self, update: impl Fn(&mut T) + 'static)
    where
        A: crate::TypedAction,
    {
        let local = self.local;
        uicx_on_action::<A>(self.cx, move |host, action_cx| {
            local.update_action(host, action_cx, |value| update(value))
        });
    }

    pub fn set<A>(self, value: T)
    where
        A: crate::TypedAction,
        T: Clone,
    {
        let local = self.local;
        uicx_on_action::<A>(self.cx, move |host, action_cx| {
            local.set_action(host, action_cx, value.clone())
        });
    }

    pub fn payload_update_if<A>(self, update: impl Fn(&mut T, A::Payload) -> bool + 'static)
    where
        A: crate::actions::TypedPayloadAction,
    {
        let local = self.local;
        uicx_on_payload_action::<A>(self.cx, move |host, action_cx, payload| {
            local.update_action_if(host, action_cx, |value| update(value, payload))
        });
    }
}

impl<'cx, 'a> UiCxActionLocal<'cx, 'a, bool> {
    pub fn toggle_bool<A>(self)
    where
        A: crate::TypedAction,
    {
        let local = self.local;
        uicx_on_action::<A>(self.cx, move |host, action_cx| {
            local.update_action(host, action_cx, |value| *value = !*value)
        });
    }
}

/// Contract for app-facing widgets that expose an activation-only callback slot.
///
/// This stays in `ecosystem/fret` because it is authoring sugar, not runtime mechanism.
pub trait AppActivateSurface: Sized {
    fn on_activate(self, on_activate: OnActivate) -> Self;
}

/// Thin app-facing sugar for activation-only widget surfaces.
///
/// Prefer widget-native `.action(...)` / `.action_payload(...)` whenever a stable action slot
/// already exists. Activation-only surfaces can still stay on the same action-first vocabulary via
/// `.action(act::Save)` / `.action_payload(act::Remove, payload)` plus `.listen(...)` as the
/// imperative escape hatch.
pub trait AppActivateExt: AppActivateSurface {
    fn action<A>(self, _action: A) -> Self
    where
        A: crate::TypedAction,
    {
        <Self as AppActivateSurface>::on_activate(self, dispatch_action_listener::<A>())
    }

    fn action_payload<A>(self, _action: A, payload: A::Payload) -> Self
    where
        A: crate::actions::TypedPayloadAction,
        A::Payload: Clone,
    {
        <Self as AppActivateSurface>::on_activate(
            self,
            dispatch_payload_action_listener::<A>(payload),
        )
    }

    fn listen(self, f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> Self {
        <Self as AppActivateSurface>::on_activate(self, action_listener(f))
    }
}

impl<T> AppActivateExt for T where T: AppActivateSurface {}

// Keep the default bridge table empty: first-party widgets should prefer native
// `.action(...)` / `.action_payload(...)` / widget-owned `.on_activate(...)` surfaces.

fn dispatch_action_listener<A>() -> OnActivate
where
    A: crate::TypedAction,
{
    let action = A::action_id();
    Arc::new(move |host, action_cx, reason| {
        host.record_pending_command_dispatch_source(action_cx, &action, reason);
        host.dispatch_command(Some(action_cx.window), action.clone());
    })
}

fn dispatch_payload_action_listener<A>(payload: A::Payload) -> OnActivate
where
    A: crate::actions::TypedPayloadAction,
    A::Payload: Clone,
{
    let action = A::action_id();
    Arc::new(move |host, action_cx, reason| {
        host.record_pending_command_dispatch_source(action_cx, &action, reason);
        host.record_pending_action_payload(action_cx, &action, Box::new(payload.clone()));
        host.dispatch_command(Some(action_cx.window), action.clone());
    })
}

fn action_listener(f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> OnActivate {
    Arc::new(move |host, action_cx, _reason| f(host, action_cx))
}

#[derive(Default)]
struct UiCxActionHooksFrameSlot {
    frame_id: Option<fret_runtime::FrameId>,
}

struct UiCxActionHooksOwner;
struct ViewRuntimeActionHooksOwner;
struct AppUiRenderRootActionHooksOwner;

fn prepare_uicx_action_hooks(cx: &mut ElementContext<'_, crate::app::App>) {
    let frame_id = cx.frame_id;
    let action_root = cx.root_id();
    let needs_reset = cx.root_state(UiCxActionHooksFrameSlot::default, |slot| {
        if slot.frame_id == Some(frame_id) {
            return false;
        }
        slot.frame_id = Some(frame_id);
        true
    });
    if needs_reset {
        cx.action_clear_on_command_for_owner::<UiCxActionHooksOwner>(action_root);
        cx.action_clear_on_command_availability_for_owner::<UiCxActionHooksOwner>(action_root);
    }
}

fn uicx_on_action<A>(
    cx: &mut ElementContext<'_, crate::app::App>,
    f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, ActionCx) -> bool + 'static,
) where
    A: crate::TypedAction,
{
    prepare_uicx_action_hooks(cx);
    let action_root = cx.root_id();
    let action = A::action_id();
    cx.action_add_on_command_for_owner::<UiCxActionHooksOwner>(
        action_root,
        Arc::new(move |host, action_cx, command| {
            if command != action {
                return false;
            }
            f(host, action_cx)
        }),
    );
}

fn uicx_on_action_notify<A>(
    cx: &mut ElementContext<'_, crate::app::App>,
    f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, ActionCx) -> bool + 'static,
) where
    A: crate::TypedAction,
{
    uicx_on_action::<A>(cx, move |host, action_cx| {
        let handled = f(host, action_cx);
        if handled {
            host.request_redraw(action_cx.window);
            host.notify(action_cx);
        }
        handled
    });
}

fn uicx_on_payload_action<A>(
    cx: &mut ElementContext<'_, crate::app::App>,
    f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, ActionCx, A::Payload) -> bool + 'static,
) where
    A: crate::actions::TypedPayloadAction,
{
    prepare_uicx_action_hooks(cx);
    let action_root = cx.root_id();
    let action = A::action_id();
    cx.action_add_on_command_for_owner::<UiCxActionHooksOwner>(
        action_root,
        Arc::new(move |host, action_cx, command| {
            if command != action {
                return false;
            }
            let Some(payload_any) = host.consume_pending_action_payload(action_cx.window, &action)
            else {
                return false;
            };
            let Ok(payload) = payload_any.downcast::<A::Payload>() else {
                return false;
            };
            f(host, action_cx, *payload)
        }),
    );
}

fn uicx_on_action_availability<A>(
    cx: &mut ElementContext<'_, crate::app::App>,
    f: impl Fn(
        &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
        fret_ui::action::CommandAvailabilityActionCx,
    ) -> fret_ui::CommandAvailability
    + 'static,
) where
    A: crate::TypedAction,
{
    prepare_uicx_action_hooks(cx);
    let action_root = cx.root_id();
    let action = A::action_id();
    cx.action_add_on_command_availability_for_owner::<UiCxActionHooksOwner>(
        action_root,
        Arc::new(move |host, action_cx, command| {
            if command != action {
                return fret_ui::CommandAvailability::NotHandled;
            }
            f(host, action_cx)
        }),
    );
}

impl<'view, 'cx, 'a, H: UiHost> AppUiActions<'view, 'cx, 'a, H> {
    /// Build a widget-local activation listener without reopening the raw `Arc<dyn Fn...>` seam.
    pub fn listen(self, f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> OnActivate {
        action_listener(f)
    }

    /// Bind a typed action handler against one `LocalState<T>` slot without repeating that handle
    /// on every helper family.
    pub fn local<T>(self, local: &LocalState<T>) -> AppUiActionLocal<'view, 'cx, 'a, H, T>
    where
        T: Any,
    {
        AppUiActionLocal {
            cx: self.cx,
            local: LocalState::clone(local),
        }
    }

    pub fn models<A>(self, f: impl Fn(&mut fret_runtime::ModelStore) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        self.cx
            .on_action_notify::<A>(move |host, _action_cx| f(host.models_mut()));
    }

    /// Coordinate shared `Model<T>` graphs through a typed payload action without reopening the
    /// raw payload-carrier namespace.
    ///
    /// Prefer `cx.actions().local(&local).payload_update_if::<A>(...)` when the write stays on
    /// `LocalState<T>`. Use this when the payload targets shared `Model<T>` graphs or
    /// view-external state that already lives in `ModelStore`.
    pub fn payload_models<A>(
        self,
        f: impl Fn(&mut fret_runtime::ModelStore, A::Payload) -> bool + 'static,
    ) where
        A: crate::actions::TypedPayloadAction,
    {
        self.cx
            .on_payload_action_notify::<A>(move |host, _action_cx, payload| {
                f(host.models_mut(), payload)
            });
    }

    /// Clone the provided `LocalState<T>` handles into a hidden builder so the call site can pass
    /// `(&draft_state, &next_id_state, ...)` directly, then register the typed action via
    /// `.on::<A>(...)` without repeating a `LocalState::clone(...)` prelude at every call site.
    ///
    /// Borrowed captures are often the right default in real render bodies because the same local
    /// handles are still used later for reads, widget binding, or other action registration. Use
    /// owned `LocalState<T>` values only when the handles are not needed again in the same scope.
    ///
    /// This keeps action identity and the `LocalStateTxn` boundary explicit while trimming the
    /// common multi-slot capture ceremony on the default app lane.
    pub fn locals_with<C>(self, captures: C) -> AppUiLocalsWith<'view, 'cx, 'a, H, C::Owned>
    where
        C: LocalActionCapture,
    {
        AppUiLocalsWith {
            cx: self.cx,
            captures: captures.capture_owned(),
        }
    }

    pub fn transient<A>(self, transient_key: u64)
    where
        A: crate::TypedAction,
    {
        self.cx.on_action_notify::<A>(move |host, action_cx| {
            host.record_transient_event(action_cx, transient_key);
            true
        });
    }

    pub fn availability<A>(
        self,
        f: impl Fn(
            &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
            fret_ui::action::CommandAvailabilityActionCx,
        ) -> fret_ui::CommandAvailability
        + 'static,
    ) where
        A: crate::TypedAction,
    {
        self.cx.register_action_availability_handler::<A>(f);
    }
}

impl<'cx, 'a> UiCxActions<'cx, 'a> {
    /// Build a widget-local activation listener without reopening the raw `Arc<dyn Fn...>` seam.
    pub fn listen(self, f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> OnActivate {
        action_listener(f)
    }

    /// Bind a typed action handler against one `LocalState<T>` slot without repeating that handle
    /// on every helper family.
    pub fn local<T>(self, local: &LocalState<T>) -> UiCxActionLocal<'cx, 'a, T>
    where
        T: Any,
    {
        UiCxActionLocal {
            cx: self.cx,
            local: LocalState::clone(local),
        }
    }

    pub fn models<A>(self, f: impl Fn(&mut fret_runtime::ModelStore) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        uicx_on_action_notify::<A>(self.cx, move |host, _action_cx| f(host.models_mut()));
    }

    /// Coordinate shared `Model<T>` graphs through a typed payload action without reopening the
    /// raw payload-carrier namespace.
    ///
    /// Prefer `cx.actions().local(&local).payload_update_if::<A>(...)` when the write stays on
    /// `LocalState<T>`. Use this when the payload targets shared `Model<T>` graphs or
    /// view-external state that already lives in `ModelStore`.
    pub fn payload_models<A>(
        self,
        f: impl Fn(&mut fret_runtime::ModelStore, A::Payload) -> bool + 'static,
    ) where
        A: crate::actions::TypedPayloadAction,
    {
        uicx_on_payload_action::<A>(self.cx, move |host, action_cx, payload| {
            let handled = f(host.models_mut(), payload);
            if handled {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
            handled
        });
    }

    /// Clone the provided `LocalState<T>` handles into a hidden builder so helper-heavy `AppComponentCx`
    /// code can pass `(&draft_state, &next_id_state, ...)` directly, then register the typed
    /// action via `.on::<A>(...)` without reopening a `LocalState::clone(...)` prelude.
    pub fn locals_with<C>(self, captures: C) -> UiCxLocalsWith<'cx, 'a, C::Owned>
    where
        C: LocalActionCapture,
    {
        UiCxLocalsWith {
            cx: self.cx,
            captures: captures.capture_owned(),
        }
    }

    pub fn transient<A>(self, transient_key: u64)
    where
        A: crate::TypedAction,
    {
        uicx_on_action_notify::<A>(self.cx, move |host, action_cx| {
            host.record_transient_event(action_cx, transient_key);
            true
        });
    }

    pub fn availability<A>(
        self,
        f: impl Fn(
            &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
            fret_ui::action::CommandAvailabilityActionCx,
        ) -> fret_ui::CommandAvailability
        + 'static,
    ) where
        A: crate::TypedAction,
    {
        uicx_on_action_availability::<A>(self.cx, f);
    }
}

/// Grouped selector/query helpers for the default app authoring surface.
#[doc(hidden)]
pub struct AppUiData<'view, 'cx, 'a, H: UiHost> {
    #[allow(dead_code)]
    cx: &'view mut AppUi<'cx, 'a, H>,
}

#[cfg(feature = "state-mutation")]
fn take_mutation_completion_state_in<H: UiHost, TIn: 'static, TOut: 'static>(
    cx: &mut ElementContext<'_, H>,
    effect_key: u64,
    handle: &fret_mutation::MutationHandle<TIn, TOut>,
) -> Option<fret_mutation::MutationState<TIn, TOut>> {
    let completion_token = handle.completion_token();
    let state = cx
        .get_model_cloned(handle.model(), Invalidation::Layout)
        .unwrap_or_default();
    if !(state.is_success() || state.is_error()) {
        return None;
    }

    let Some(completion_token) = completion_token else {
        return None;
    };

    let fresh = cx.keyed_slot_state(
        (effect_key, handle.model().id()),
        Option::<std::num::NonZeroU64>::default,
        |last_seen| {
            if *last_seen == Some(completion_token) {
                false
            } else {
                *last_seen = Some(completion_token);
                true
            }
        },
    );
    if !fresh {
        return None;
    }

    Some(state)
}

#[cfg(feature = "state-mutation")]
fn take_mutation_completion_in<H: UiHost, TIn: 'static, TOut: 'static>(
    cx: &mut ElementContext<'_, H>,
    effect_key: u64,
    handle: &fret_mutation::MutationHandle<TIn, TOut>,
) -> bool {
    take_mutation_completion_state_in(cx, effect_key, handle).is_some()
}

#[cfg(feature = "state-mutation")]
fn take_mutation_success_in<H: UiHost, TIn: 'static, TOut: 'static>(
    cx: &mut ElementContext<'_, H>,
    effect_key: u64,
    handle: &fret_mutation::MutationHandle<TIn, TOut>,
) -> bool {
    let success_token = handle.success_token();
    let state = cx
        .get_model_cloned(handle.model(), Invalidation::Layout)
        .unwrap_or_default();
    if !state.is_success() {
        return false;
    }

    let Some(success_token) = success_token else {
        return false;
    };

    cx.keyed_slot_state(
        (effect_key, handle.model().id()),
        Option::<std::num::NonZeroU64>::default,
        |last_seen| {
            if *last_seen == Some(success_token) {
                false
            } else {
                *last_seen = Some(success_token);
                true
            }
        },
    )
}

#[cfg(feature = "state-query")]
fn query_snapshot_entry_for_key<T: Any + Send + Sync + 'static>(
    snapshot: fret_query::QueryClientSnapshot,
    key: fret_query::QueryKey<T>,
) -> Option<fret_query::QuerySnapshotEntry> {
    let type_name = std::any::type_name::<T>();
    snapshot.entries.into_iter().find(|entry| {
        entry.namespace == key.namespace()
            && entry.hash == key.hash()
            && entry.type_name == type_name
    })
}

impl<'view, 'cx, 'a, H: UiHost> AppUiData<'view, 'cx, 'a, H> {
    /// Default LocalState-first selector path for app-facing derived values that affect layout.
    ///
    /// Use this when the deps are view-owned `LocalState<T>` slots. Keep raw `selector(...)` for
    /// explicit shared `Model<T>` signatures, global tokens, or custom dependency builders.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_layout<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: LocalSelectorLayoutInputs<'a, H>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
        )
    }

    /// Grouped selector path for explicit shared `Model<T>` bags that affect layout.
    ///
    /// Use this when the deps intentionally stay as shared `Model<T>` handles on manual/advanced
    /// surfaces. Prefer `selector_layout(...)` when the inputs are view-owned `LocalState<T>`.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_model_layout<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: ModelSelectorInputs<'a, H>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
        )
    }

    /// Grouped selector path for explicit shared `Model<T>` bags that affect paint-time derived
    /// values.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_model_paint<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: ModelSelectorInputs<'a, H>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Paint),
            move |cx| compute(inputs.values_in(cx, Invalidation::Paint)),
        )
    }

    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector<Deps, TValue>(
        self,
        deps: impl FnOnce(&mut ElementContext<'a, H>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'a, H>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(self.cx.cx, deps, compute)
    }

    #[cfg(feature = "state-query")]
    pub fn query<T: Any + Send + Sync + 'static>(
        self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Result<T, fret_query::QueryError>
        + Send
        + 'static,
    ) -> fret_query::QueryHandle<T> {
        fret_query::ui::QueryElementContextExt::use_query(self.cx.cx, key, policy, fetch)
    }

    #[cfg(feature = "state-query")]
    pub fn query_async<T, Fut>(
        self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Fut + Send + 'static,
    ) -> fret_query::QueryHandle<T>
    where
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, fret_query::QueryError>> + Send + 'static,
    {
        fret_query::ui::QueryElementContextExt::use_query_async(self.cx.cx, key, policy, fetch)
    }

    #[cfg(feature = "state-query")]
    pub fn query_async_local<T, Fut>(
        self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Fut + 'static,
    ) -> fret_query::QueryHandle<T>
    where
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, fret_query::QueryError>> + 'static,
    {
        fret_query::ui::QueryElementContextExt::use_query_async_local(
            self.cx.cx, key, policy, fetch,
        )
    }

    /// Grouped query-client snapshot read for app-facing diagnostics or status chrome on `AppUi`.
    ///
    /// Keep raw `fret::query::with_query_client(...)` for pure app/driver code that does not have
    /// a grouped `cx.data()` surface.
    #[cfg(feature = "state-query")]
    pub fn query_snapshot(self) -> Option<fret_query::QueryClientSnapshot> {
        fret_query::with_query_client(self.cx.cx.app, |client, _app| client.snapshot())
    }

    /// Find one typed query snapshot entry on the grouped app data lane without dropping back to
    /// raw query-client plumbing.
    #[cfg(feature = "state-query")]
    pub fn query_snapshot_entry<T: Any + Send + Sync + 'static>(
        self,
        key: fret_query::QueryKey<T>,
    ) -> Option<fret_query::QuerySnapshotEntry> {
        self.query_snapshot()
            .and_then(|snapshot| query_snapshot_entry_for_key(snapshot, key))
    }

    /// Cancel one inflight query task from the grouped app data lane while keeping redraw ownership
    /// local to `AppUi`.
    #[cfg(feature = "state-query")]
    pub fn cancel_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>) {
        let _ = fret_query::with_query_client(self.cx.cx.app, |client, app| {
            client.cancel_inflight(app, key);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
    }

    #[cfg(feature = "state-mutation")]
    pub fn mutation_async<TIn, TOut, Fut>(
        self,
        policy: fret_mutation::MutationPolicy,
        submit: impl Fn(fret_mutation::CancellationToken, Arc<TIn>) -> Fut + Send + Sync + 'static,
    ) -> fret_mutation::MutationHandle<TIn, TOut>
    where
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<TOut, fret_mutation::MutationError>> + Send + 'static,
    {
        fret_mutation::ui::MutationElementContextExt::use_mutation_async(self.cx.cx, policy, submit)
    }

    #[cfg(feature = "state-mutation")]
    pub fn mutation_async_local<TIn, TOut, Fut>(
        self,
        policy: fret_mutation::MutationPolicy,
        submit: impl Fn(fret_mutation::CancellationToken, Arc<TIn>) -> Fut + 'static,
    ) -> fret_mutation::MutationHandle<TIn, TOut>
    where
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<TOut, fret_mutation::MutationError>> + 'static,
    {
        fret_mutation::ui::MutationElementContextExt::use_mutation_async_local(
            self.cx.cx, policy, submit,
        )
    }

    /// Consume a mutation completion exactly once for one `(effect_key, handle)` pair on the
    /// default app data lane.
    ///
    /// This covers both success and error terminal states when app code needs to materialize the
    /// latest terminal result into ordinary `LocalState<T>` or shared models without replaying the
    /// same completion on later renders. Prefer `update_after_mutation_completion(...)` when this
    /// once-only gate immediately drives app-owned model updates.
    #[cfg(feature = "state-mutation")]
    pub fn take_mutation_completion<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
    ) -> bool {
        take_mutation_completion_in(self.cx.cx, effect_key, handle)
    }

    /// Update ordinary app-owned `LocalState<T>` or shared models exactly once after a mutation
    /// reaches a fresh terminal completion on the default app data lane.
    ///
    /// Prefer this over pairing `handle.read_layout(cx)` with `take_mutation_completion(...)`
    /// when the only goal is to project the latest terminal result into other app-owned state.
    #[cfg(feature = "state-mutation")]
    pub fn update_after_mutation_completion<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        apply: impl FnOnce(
            &mut fret_runtime::ModelStore,
            fret_mutation::MutationState<TIn, TOut>,
        ) -> bool,
    ) -> bool {
        let Some(state) = take_mutation_completion_state_in(self.cx.cx, effect_key, handle) else {
            return false;
        };
        let changed = apply(self.cx.cx.app.models_mut(), state);
        if changed {
            self.cx.cx.app.request_redraw(self.cx.cx.window);
        }
        changed
    }

    /// Consume a mutation success exactly once for one `(effect_key, handle)` pair on the default
    /// app data lane.
    ///
    /// This keeps terminal mutation state reviewable via `read_layout(cx)` while avoiding repeated
    /// render-triggered follow-up work after the same completion.
    #[cfg(feature = "state-mutation")]
    pub fn take_mutation_success<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
    ) -> bool {
        take_mutation_success_in(self.cx.cx, effect_key, handle)
    }

    /// Default grouped invalidation path for app-facing query state when the caller is already on
    /// `AppUi`.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>) {
        let _ = fret_query::with_query_client(self.cx.cx.app, |client, app| {
            client.invalidate(app, key);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
    }

    /// Default grouped namespace invalidation path for app-facing query state when the caller is
    /// already on `AppUi`.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query_namespace(self, namespace: &'static str) {
        let _ = fret_query::with_query_client(self.cx.cx.app, |client, _app| {
            client.invalidate_namespace(namespace);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
    }

    /// Invalidate one query key exactly once after a mutation reports success on the default app
    /// data lane.
    #[cfg(all(feature = "state-query", feature = "state-mutation"))]
    pub fn invalidate_query_after_mutation_success<
        TIn: 'static,
        TOut: 'static,
        T: Any + Send + Sync + 'static,
    >(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        key: fret_query::QueryKey<T>,
    ) -> bool {
        if !take_mutation_success_in(self.cx.cx, effect_key, handle) {
            return false;
        }

        let _ = fret_query::with_query_client(self.cx.cx.app, |client, app| {
            client.invalidate(app, key);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
        true
    }

    /// Invalidate one query namespace exactly once after a mutation reports success on the
    /// default app data lane.
    #[cfg(all(feature = "state-query", feature = "state-mutation"))]
    pub fn invalidate_query_namespace_after_mutation_success<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        namespace: &'static str,
    ) -> bool {
        if !take_mutation_success_in(self.cx.cx, effect_key, handle) {
            return false;
        }

        let _ = fret_query::with_query_client(self.cx.cx.app, |client, _app| {
            client.invalidate_namespace(namespace);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
        true
    }
}

/// Grouped selector/query helpers for extracted `AppComponentCx` child builders on the default app surface.
#[doc(hidden)]
pub struct UiCxData<'cx, 'a> {
    #[allow(dead_code)]
    cx: &'cx mut ElementContext<'a, crate::app::App>,
}

impl<'cx, 'a> UiCxData<'cx, 'a> {
    /// Default LocalState-first selector path for extracted `AppComponentCx` helpers on the app-facing lane.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_layout<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: LocalSelectorLayoutInputs<'a, crate::app::App>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
        )
    }

    /// Grouped selector path for explicit shared `Model<T>` bags on extracted `AppComponentCx` helpers when
    /// the derived value affects layout.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_model_layout<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: ModelSelectorInputs<'a, crate::app::App>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
        )
    }

    /// Grouped selector path for explicit shared `Model<T>` bags on extracted `AppComponentCx` helpers when
    /// the derived value affects paint.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_model_paint<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: ModelSelectorInputs<'a, crate::app::App>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Paint),
            move |cx| compute(inputs.values_in(cx, Invalidation::Paint)),
        )
    }

    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector<Deps, TValue>(
        self,
        deps: impl FnOnce(&mut ElementContext<'a, crate::app::App>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'a, crate::app::App>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(self.cx, deps, compute)
    }

    #[cfg(feature = "state-query")]
    pub fn query<T: Any + Send + Sync + 'static>(
        self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Result<T, fret_query::QueryError>
        + Send
        + 'static,
    ) -> fret_query::QueryHandle<T> {
        fret_query::ui::QueryElementContextExt::use_query(self.cx, key, policy, fetch)
    }

    #[cfg(feature = "state-query")]
    pub fn query_async<T, Fut>(
        self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Fut + Send + 'static,
    ) -> fret_query::QueryHandle<T>
    where
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, fret_query::QueryError>> + Send + 'static,
    {
        fret_query::ui::QueryElementContextExt::use_query_async(self.cx, key, policy, fetch)
    }

    #[cfg(feature = "state-query")]
    pub fn query_async_local<T, Fut>(
        self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Fut + 'static,
    ) -> fret_query::QueryHandle<T>
    where
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, fret_query::QueryError>> + 'static,
    {
        fret_query::ui::QueryElementContextExt::use_query_async_local(self.cx, key, policy, fetch)
    }

    /// Grouped query-client snapshot read for extracted `AppComponentCx` app-facing diagnostics helpers.
    ///
    /// Keep raw `fret::query::with_query_client(...)` for pure app/driver code that does not have
    /// a grouped `cx.data()` surface.
    #[cfg(feature = "state-query")]
    pub fn query_snapshot(self) -> Option<fret_query::QueryClientSnapshot> {
        fret_query::with_query_client(self.cx.app, |client, _app| client.snapshot())
    }

    /// Find one typed query snapshot entry from an extracted `AppComponentCx` helper on the grouped app
    /// data lane.
    #[cfg(feature = "state-query")]
    pub fn query_snapshot_entry<T: Any + Send + Sync + 'static>(
        self,
        key: fret_query::QueryKey<T>,
    ) -> Option<fret_query::QuerySnapshotEntry> {
        self.query_snapshot()
            .and_then(|snapshot| query_snapshot_entry_for_key(snapshot, key))
    }

    /// Cancel one inflight query task from an extracted `AppComponentCx` helper while keeping redraw
    /// ownership on the grouped app data lane.
    #[cfg(feature = "state-query")]
    pub fn cancel_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>) {
        let _ = fret_query::with_query_client(self.cx.app, |client, app| {
            client.cancel_inflight(app, key);
        });
        self.cx.app.request_redraw(self.cx.window);
    }

    #[cfg(feature = "state-mutation")]
    pub fn mutation_async<TIn, TOut, Fut>(
        self,
        policy: fret_mutation::MutationPolicy,
        submit: impl Fn(fret_mutation::CancellationToken, Arc<TIn>) -> Fut + Send + Sync + 'static,
    ) -> fret_mutation::MutationHandle<TIn, TOut>
    where
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<TOut, fret_mutation::MutationError>> + Send + 'static,
    {
        fret_mutation::ui::MutationElementContextExt::use_mutation_async(self.cx, policy, submit)
    }

    #[cfg(feature = "state-mutation")]
    pub fn mutation_async_local<TIn, TOut, Fut>(
        self,
        policy: fret_mutation::MutationPolicy,
        submit: impl Fn(fret_mutation::CancellationToken, Arc<TIn>) -> Fut + 'static,
    ) -> fret_mutation::MutationHandle<TIn, TOut>
    where
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<TOut, fret_mutation::MutationError>> + 'static,
    {
        fret_mutation::ui::MutationElementContextExt::use_mutation_async_local(
            self.cx, policy, submit,
        )
    }

    /// Consume a mutation completion exactly once for one `(effect_key, handle)` pair inside an
    /// extracted `AppComponentCx` helper.
    ///
    /// Prefer `update_after_mutation_completion(...)` when this once-only gate immediately drives
    /// app-owned model updates.
    #[cfg(feature = "state-mutation")]
    pub fn take_mutation_completion<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
    ) -> bool {
        take_mutation_completion_in(self.cx, effect_key, handle)
    }

    /// Update ordinary app-owned `LocalState<T>` or shared models exactly once after a mutation
    /// reaches a fresh terminal completion inside an extracted `AppComponentCx` helper.
    #[cfg(feature = "state-mutation")]
    pub fn update_after_mutation_completion<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        apply: impl FnOnce(
            &mut fret_runtime::ModelStore,
            fret_mutation::MutationState<TIn, TOut>,
        ) -> bool,
    ) -> bool {
        let Some(state) = take_mutation_completion_state_in(self.cx, effect_key, handle) else {
            return false;
        };
        let changed = apply(self.cx.app.models_mut(), state);
        if changed {
            self.cx.app.request_redraw(self.cx.window);
        }
        changed
    }

    /// Consume a mutation success exactly once for one `(effect_key, handle)` pair inside an
    /// extracted `AppComponentCx` helper.
    #[cfg(feature = "state-mutation")]
    pub fn take_mutation_success<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
    ) -> bool {
        take_mutation_success_in(self.cx, effect_key, handle)
    }

    /// Grouped invalidation helper for extracted `AppComponentCx` app-facing helpers.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>) {
        let _ = fret_query::with_query_client(self.cx.app, |client, app| {
            client.invalidate(app, key);
        });
        self.cx.app.request_redraw(self.cx.window);
    }

    /// Grouped namespace invalidation helper for extracted `AppComponentCx` app-facing helpers.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query_namespace(self, namespace: &'static str) {
        let _ = fret_query::with_query_client(self.cx.app, |client, _app| {
            client.invalidate_namespace(namespace);
        });
        self.cx.app.request_redraw(self.cx.window);
    }

    /// Invalidate one query key exactly once after a mutation reports success inside an extracted
    /// `AppComponentCx` helper.
    #[cfg(all(feature = "state-query", feature = "state-mutation"))]
    pub fn invalidate_query_after_mutation_success<
        TIn: 'static,
        TOut: 'static,
        T: Any + Send + Sync + 'static,
    >(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        key: fret_query::QueryKey<T>,
    ) -> bool {
        if !take_mutation_success_in(self.cx, effect_key, handle) {
            return false;
        }

        let _ = fret_query::with_query_client(self.cx.app, |client, app| {
            client.invalidate(app, key);
        });
        self.cx.app.request_redraw(self.cx.window);
        true
    }

    /// Invalidate one query namespace exactly once after a mutation reports success inside an
    /// extracted `AppComponentCx` helper.
    #[cfg(all(feature = "state-query", feature = "state-mutation"))]
    pub fn invalidate_query_namespace_after_mutation_success<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        namespace: &'static str,
    ) -> bool {
        if !take_mutation_success_in(self.cx, effect_key, handle) {
            return false;
        }

        let _ = fret_query::with_query_client(self.cx.app, |client, _app| {
            client.invalidate_namespace(namespace);
        });
        self.cx.app.request_redraw(self.cx.window);
        true
    }
}

/// Brings the grouped `data()` namespace to extracted `AppComponentCx` helper functions.
pub trait UiCxDataExt<'a> {
    /// Discover selector/query helpers through `cx.data()` rather than naming the carrier type
    /// directly.
    fn data(&mut self) -> UiCxData<'_, 'a>;
}

impl<'a, Cx> UiCxDataExt<'a> for Cx
where
    Cx: RenderContextAccess<'a, crate::app::App>,
{
    fn data(&mut self) -> UiCxData<'_, 'a> {
        UiCxData {
            cx: self.elements(),
        }
    }
}

/// Brings the grouped `actions()` namespace to extracted `AppComponentCx` helper functions.
pub trait UiCxActionsExt<'a> {
    /// Discover grouped action helpers through `cx.actions()` rather than naming the carrier type
    /// directly.
    fn actions(&mut self) -> UiCxActions<'_, 'a>;
}

impl<'a, Cx> UiCxActionsExt<'a> for Cx
where
    Cx: RenderContextAccess<'a, crate::app::App>,
{
    fn actions(&mut self) -> UiCxActions<'_, 'a> {
        UiCxActions {
            cx: self.elements(),
        }
    }
}

/// Grouped render-time effect helpers for the default app authoring surface.
#[doc(hidden)]
pub struct AppUiEffects<'view, 'cx, 'a, H: UiHost> {
    cx: &'view mut AppUi<'cx, 'a, H>,
}

impl<'view, 'cx, 'a, H: UiHost> AppUiEffects<'view, 'cx, 'a, H> {
    pub fn take_transient(self, key: u64) -> bool {
        self.cx.cx.take_transient_for(self.cx.action_root, key)
    }
}

impl<'cx, 'a, H: UiHost> fret_ui::ElementContextAccess<'a, H> for AppUi<'cx, 'a, H> {
    fn elements(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
    }
}

impl<'cx, 'a, H: UiHost> fret_ui_kit::command::ElementCommandGatingExt for AppUi<'cx, 'a, H> {
    fn command_is_enabled(&self, command: &fret_runtime::CommandId) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::command_is_enabled(
            &*self.cx, command,
        )
    }

    fn command_is_enabled_with_fallback_input_context(
        &self,
        command: &fret_runtime::CommandId,
        fallback_input_ctx: fret_runtime::InputContext,
    ) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::command_is_enabled_with_fallback_input_context(
            &*self.cx,
            command,
            fallback_input_ctx,
        )
    }

    fn dispatch_command_if_enabled(&mut self, command: fret_runtime::CommandId) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::dispatch_command_if_enabled(
            self.cx,
            command,
        )
    }

    fn dispatch_command_if_enabled_with_fallback_input_context(
        &mut self,
        command: fret_runtime::CommandId,
        fallback_input_ctx: fret_runtime::InputContext,
    ) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::dispatch_command_if_enabled_with_fallback_input_context(
            self.cx,
            command,
            fallback_input_ctx,
        )
    }

    fn action_is_enabled(&self, action: &fret_runtime::ActionId) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::action_is_enabled(
            &*self.cx, action,
        )
    }

    fn dispatch_action_if_enabled(&mut self, action: fret_runtime::ActionId) -> bool {
        <ElementContext<'a, H> as fret_ui_kit::command::ElementCommandGatingExt>::dispatch_action_if_enabled(
            self.cx,
            action,
        )
    }
}

impl<'cx, 'a, H: UiHost> fret_ui_kit::declarative::ElementContextThemeExt for AppUi<'cx, 'a, H> {
    fn with_theme<R>(&mut self, f: impl FnOnce(&fret_ui::Theme) -> R) -> R {
        f(self.cx.theme())
    }

    fn theme_snapshot(&mut self) -> fret_ui::ThemeSnapshot {
        self.cx.theme().snapshot()
    }
}

impl<'cx, 'a, H: UiHost> AppUi<'cx, 'a, H> {
    pub(crate) fn new(
        cx: &'cx mut ElementContext<'a, H>,
        action_root: fret_ui::GlobalElementId,
    ) -> Self {
        Self {
            cx,
            action_root,
            action_handlers: crate::actions::ActionHandlerTable::new(),
            action_handlers_used: false,
        }
    }

    /// Access the underlying element context explicitly.
    ///
    /// This is the escape hatch for advanced/component-heavy code that intentionally needs the
    /// lower-level identity/state substrate (`scope`, `slot_state`, `local_model`, `state_for`,
    /// etc.). Default app-facing code should prefer `state()`, `actions()`, `data()`, `effects()`,
    /// and `keyed()` first.
    pub fn elements(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
    }

    /// Borrow the current app/host explicitly from the default app-facing render lane.
    pub fn app(&self) -> &H {
        self.cx.app
    }

    /// Borrow the current app/host mutably from the default app-facing render lane.
    pub fn app_mut(&mut self) -> &mut H {
        self.cx.app
    }

    /// Read the current window id without reopening the broader `ElementContext` surface.
    pub fn window_id(&self) -> AppWindowId {
        self.cx.window
    }

    /// Request the next animation frame from the default app-facing render lane.
    ///
    /// Use this for frame-driven progression that must continue without fresh input events.
    pub fn request_animation_frame(&mut self) {
        self.cx.request_animation_frame();
    }

    /// Toggle the continuous-frames lease for the current view root without reopening
    /// `ElementContext`.
    ///
    /// Use this for app-facing surfaces that need ongoing frame delivery while a mode remains
    /// active. Advanced/component code can still opt into the lower-level helper directly.
    pub fn set_continuous_frames(&mut self, enabled: bool) {
        fret_ui_kit::declarative::scheduling::set_continuous_frames(self.cx, enabled);
    }

    /// Observe the committed bounds for a layout-query region from the default app-facing lane.
    pub fn layout_query_bounds(
        &mut self,
        region: fret_ui::GlobalElementId,
        invalidation: Invalidation,
    ) -> Option<fret_core::Rect> {
        self.cx.layout_query_bounds(region, invalidation)
    }

    /// Create a layout-query region on the default app-facing render lane and pass its region id.
    ///
    /// The nested builder keeps the same grouped action-registration surface as the outer `AppUi`
    /// scope instead of reopening the raw `ElementContext` lane.
    #[track_caller]
    pub fn layout_query_region_with_id<I>(
        &mut self,
        props: fret_ui::element::LayoutQueryRegionProps,
        f: impl for<'b> FnOnce(&mut AppUi<'b, 'a, H>, fret_ui::GlobalElementId) -> I,
    ) -> fret_ui::element::AnyElement
    where
        I: IntoIterator<Item = fret_ui::element::AnyElement>,
    {
        let action_root = self.action_root;
        let mut carried_action_handlers = Some(std::mem::take(&mut self.action_handlers));
        let mut carried_action_handlers_used = self.action_handlers_used;

        let out = self.cx.layout_query_region_with_id(props, |cx, id| {
            let action_handlers = carried_action_handlers
                .take()
                .expect("AppUi layout_query_region_with_id should carry handlers once");
            let mut nested = AppUi {
                cx,
                action_root,
                action_handlers,
                action_handlers_used: carried_action_handlers_used,
            };
            let built = f(&mut nested, id);
            carried_action_handlers = Some(nested.action_handlers);
            carried_action_handlers_used = nested.action_handlers_used;
            built
        });

        self.action_handlers = carried_action_handlers
            .take()
            .expect("AppUi layout_query_region_with_id should restore handlers");
        self.action_handlers_used = carried_action_handlers_used;
        out
    }

    /// Create a layout-query region on the default app-facing render lane.
    #[track_caller]
    pub fn layout_query_region<I>(
        &mut self,
        props: fret_ui::element::LayoutQueryRegionProps,
        f: impl for<'b> FnOnce(&mut AppUi<'b, 'a, H>) -> I,
    ) -> fret_ui::element::AnyElement
    where
        I: IntoIterator<Item = fret_ui::element::AnyElement>,
    {
        self.layout_query_region_with_id(props, |cx, _id| f(cx))
    }

    /// Read the committed viewport bounds from the default app-facing render lane.
    pub fn environment_viewport_bounds(&mut self, invalidation: Invalidation) -> fret_core::Rect {
        self.cx.environment_viewport_bounds(invalidation)
    }

    // Lane-sealing barriers for the default app surface.
    //
    // Keep these helpers callable-but-unusable on `AppUi` itself so method resolution stops here
    // instead of falling through the `Deref` to `ElementContext`. Advanced code can still opt into
    // the lower-level substrate explicitly via `cx.elements()`.

    #[doc(hidden)]
    pub fn scope<R>(&mut self, _f: impl FnOnce(&mut Self) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.keyed(...) or cx.elements().scope(...)")
    }

    #[doc(hidden)]
    pub fn named<R>(&mut self, _name: &str, _f: impl FnOnce(&mut Self) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.keyed(...) or cx.elements().named(...)")
    }

    #[doc(hidden)]
    pub fn root_state<S: Any, R>(&mut self, _init: impl FnOnce() -> S, _f: impl FnOnce(&mut S) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local* or cx.elements().root_state(...)")
    }

    #[doc(hidden)]
    pub fn with_state<S: Any, R>(&mut self, _init: impl FnOnce() -> S, _f: impl FnOnce(&mut S) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local* or cx.elements().root_state(...)")
    }

    #[doc(hidden)]
    pub fn slot_state<S: Any, R>(&mut self, _init: impl FnOnce() -> S, _f: impl FnOnce(&mut S) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local* or cx.elements().slot_state(...)")
    }

    #[doc(hidden)]
    pub fn keyed_slot_state<K: Hash, S: Any, R>(
        &mut self,
        _key: K,
        _init: impl FnOnce() -> S,
        _f: impl FnOnce(&mut S) -> R,
    ) where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local_keyed(...) or cx.elements().keyed_slot_state(...)")
    }

    #[doc(hidden)]
    pub fn slot_id(&mut self)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().slot_id(...)")
    }

    #[doc(hidden)]
    pub fn keyed_slot_id<K: Hash>(&mut self, _key: K)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().keyed_slot_id(...)")
    }

    #[doc(hidden)]
    pub fn local_model<T: Any>(&mut self, _init: impl FnOnce() -> T)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local_init(...) or cx.elements().local_model(...)")
    }

    #[doc(hidden)]
    pub fn local_model_keyed<K: Hash, T: Any>(&mut self, _key: K, _init: impl FnOnce() -> T)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local_keyed(...) or cx.elements().local_model_keyed(...)")
    }

    #[doc(hidden)]
    pub fn state_for<S: Any, R>(
        &mut self,
        _element: fret_ui::GlobalElementId,
        _init: impl FnOnce() -> S,
        _f: impl FnOnce(&mut S) -> R,
    ) where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().state_for(...)")
    }

    #[doc(hidden)]
    pub fn with_state_for<S: Any, R>(
        &mut self,
        _element: fret_ui::GlobalElementId,
        _init: impl FnOnce() -> S,
        _f: impl FnOnce(&mut S) -> R,
    ) where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().state_for(...)")
    }

    #[doc(hidden)]
    pub fn model_for<T: Any>(
        &mut self,
        _element: fret_ui::GlobalElementId,
        _init: impl FnOnce() -> T,
    ) where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().model_for(...)")
    }

    /// Grouped state/local helpers for the default app authoring surface.
    ///
    /// Discover this namespace through `cx.state()` rather than naming the returned carrier type
    /// directly. The grouped surface owns `local`, `local_keyed`, `local_init`, and `watch`.
    pub fn state(&mut self) -> AppUiState<'_, 'cx, 'a, H> {
        AppUiState { cx: self }
    }

    /// Grouped typed action registration helpers for the default app authoring surface.
    ///
    /// Discover this namespace through `cx.actions()` rather than naming the returned carrier
    /// type directly. The grouped surface owns widget-local action glue, one-slot local writes,
    /// coordinated `locals_with((...)).on::<A>(...)`, keyed payload writes, transients, and
    /// availability hooks.
    pub fn actions(&mut self) -> AppUiActions<'_, 'cx, 'a, H> {
        AppUiActions { cx: self }
    }

    /// Grouped selector/query helpers for the default app authoring surface.
    ///
    /// Discover this namespace through `cx.data()` rather than naming the returned carrier type
    /// directly. The grouped surface owns selector helpers, query creation, mutation creation,
    /// and query invalidation / mutation-success handoff helpers with the redraw shell included.
    pub fn data(&mut self) -> AppUiData<'_, 'cx, 'a, H> {
        AppUiData { cx: self }
    }

    /// Grouped render-time effect helpers for the default app authoring surface.
    ///
    /// Discover this namespace through `cx.effects()` rather than naming the returned carrier type
    /// directly. The grouped surface currently owns transient consumption.
    pub fn effects(&mut self) -> AppUiEffects<'_, 'cx, 'a, H> {
        AppUiEffects { cx: self }
    }

    /// Create a keyed scope for hooks and element state.
    ///
    /// Use this in loops, or prefer keyed hook variants (`use_*_keyed`) once they exist.
    #[track_caller]
    pub fn keyed<K: Hash, R>(
        &mut self,
        key: K,
        f: impl for<'b> FnOnce(&mut AppUi<'b, 'a, H>) -> R,
    ) -> R {
        let action_root = self.action_root;
        let action_handlers = std::mem::take(&mut self.action_handlers);
        let action_handlers_used = self.action_handlers_used;

        let (out, action_handlers, action_handlers_used) = self.cx.keyed(key, |cx| {
            let mut nested = AppUi {
                cx,
                action_root,
                action_handlers,
                action_handlers_used,
            };
            let out = f(&mut nested);
            (out, nested.action_handlers, nested.action_handlers_used)
        });

        self.action_handlers = action_handlers;
        self.action_handlers_used = action_handlers_used;
        out
    }

    fn register_action_handler<A: crate::TypedAction>(
        &mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
    ) {
        self.action_handlers_used = true;
        let next = std::mem::take(&mut self.action_handlers).on::<A>(f);
        self.action_handlers = next;
    }

    fn register_payload_action_handler<A: crate::actions::TypedPayloadAction>(
        &mut self,
        f: impl Fn(
            &mut dyn fret_ui::action::UiFocusActionHost,
            fret_ui::action::ActionCx,
            A::Payload,
        ) -> bool
        + 'static,
    ) {
        self.action_handlers_used = true;
        let next = std::mem::take(&mut self.action_handlers).on_payload::<A>(f);
        self.action_handlers = next;
    }

    fn register_action_availability_handler<A: crate::TypedAction>(
        &mut self,
        f: impl Fn(
            &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
            fret_ui::action::CommandAvailabilityActionCx,
        ) -> fret_ui::CommandAvailability
        + 'static,
    ) {
        self.action_handlers_used = true;
        let next = std::mem::take(&mut self.action_handlers).availability::<A>(f);
        self.action_handlers = next;
    }

    #[track_caller]
    fn raw_model_with<T>(&mut self, init: impl FnOnce() -> T) -> Model<T>
    where
        T: Any,
    {
        let callsite = std::panic::Location::caller();

        #[cfg(debug_assertions)]
        {
            if self.cx.note_repeated_call_in_render_evaluation_at(callsite) {
                eprintln!(
                    "raw_model called multiple times in the same render pass at the same callsite ({}:{}:{}); wrap in `cx.keyed(...)` to avoid state collisions",
                    callsite.file(),
                    callsite.line(),
                    callsite.column()
                );
            }
        }

        self.cx.local_model_at(callsite, init)
    }

    #[track_caller]
    fn local_with<T>(&mut self, init: impl FnOnce() -> T) -> LocalState<T>
    where
        T: Any,
    {
        LocalState {
            model: self.raw_model_with(init),
        }
    }

    /// Internal substrate for app-facing local tracked reads.
    pub(crate) fn watch_local<'m, T: Any>(
        &'m mut self,
        local: &'m LocalState<T>,
    ) -> WatchedState<'m, 'm, 'a, H, T> {
        WatchedState::new(self.cx, local.model())
    }

    pub(crate) fn take_action_handlers(self) -> Option<(OnCommand, OnCommandAvailability)> {
        if !self.action_handlers_used {
            return None;
        }
        Some(self.action_handlers.build())
    }
}

// `AppUi` intentionally does not implement `Deref<Target = ElementContext<...>>`.
// Keep the default app-facing render-authoring lane separate from raw `ElementContext` so
// advanced/manual builder ownership stays explicit at `cx.elements()`. See ADR 0319 and the
// corresponding workstream before widening this boundary again.

#[doc(hidden)]
pub struct ViewWindowState<V: View> {
    pub view: V,
    pub(crate) cached_handlers: Option<(OnCommand, OnCommandAvailability)>,
    pub(crate) cached_action_root: Option<fret_ui::GlobalElementId>,
}

/// Keepalive state for manual `render_root(...)` surfaces that opt into `AppUi`.
///
/// This mirrors the cached action-handler lifecycle used by the `View` runtime so manual `UiTree`
/// / `FnDriver` hosts can reuse the same grouped `AppUi` authoring surface without reintroducing
/// low-level action-hook bookkeeping at each call site.
#[derive(Default)]
pub struct AppUiRenderRootState {
    cached_handlers: Option<(OnCommand, OnCommandAvailability)>,
    cached_action_root: Option<fret_ui::GlobalElementId>,
}

fn clear_app_ui_action_handlers_for_owner<Owner: 'static, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    action_root: fret_ui::GlobalElementId,
) {
    cx.action_clear_on_command_for_owner::<Owner>(action_root);
    cx.action_clear_on_command_availability_for_owner::<Owner>(action_root);
}

fn install_app_ui_action_handlers_for_owner<Owner: 'static, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    action_root: fret_ui::GlobalElementId,
    handlers: &Option<(OnCommand, OnCommandAvailability)>,
) {
    if let Some((on_command, on_command_availability)) = handlers.clone() {
        cx.action_on_command_for_owner::<Owner>(action_root, on_command);
        cx.action_on_command_availability_for_owner::<Owner>(action_root, on_command_availability);
    }
}

fn render_app_ui_with_cached_handlers<'a, Owner: 'static, H: UiHost + 'static>(
    cx: &mut ElementContext<'a, H>,
    action_root_name: &str,
    cached_handlers: &mut Option<(OnCommand, OnCommandAvailability)>,
    cached_action_root: &mut Option<fret_ui::GlobalElementId>,
    render: impl for<'cx, 'el> FnOnce(&mut AppUi<'cx, 'el, H>) -> crate::Ui,
) -> crate::Ui {
    let mut render = Some(render);
    cx.named(action_root_name, |cx| {
        if let Some(action_root) = *cached_action_root {
            clear_app_ui_action_handlers_for_owner::<Owner, _>(cx, action_root);

            // Ensure handlers remain installed even when the view-cache root is reused (render
            // skipped).
            install_app_ui_action_handlers_for_owner::<Owner, _>(cx, action_root, cached_handlers);
        }

        cx.view_cache(
            fret_ui::element::ViewCacheProps {
                contained_layout: true,
                cache_key: 0,
                ..fret_ui::element::ViewCacheProps::default()
            },
            |cx| {
                let action_root = cx.root_id();
                clear_app_ui_action_handlers_for_owner::<Owner, _>(cx, action_root);

                let mut app_ui = AppUi::new(cx, action_root);
                let render = render.take().expect("AppUi render closure should run once");
                let out = render(&mut app_ui);
                *cached_handlers = app_ui.take_action_handlers();
                *cached_action_root = Some(action_root);

                install_app_ui_action_handlers_for_owner::<Owner, _>(
                    cx,
                    action_root,
                    cached_handlers,
                );

                out
            },
        )
        .into()
    })
}

#[doc(hidden)]
pub fn view_init_window<V: View>(
    app: &mut fret_app::App,
    window: AppWindowId,
) -> ViewWindowState<V> {
    ViewWindowState {
        view: V::init(app, window),
        cached_handlers: None,
        cached_action_root: None,
    }
}

#[doc(hidden)]
pub fn view_view<'a, V: View>(
    cx: &mut ElementContext<'a, fret_app::App>,
    st: &mut ViewWindowState<V>,
) -> crate::Ui {
    let ViewWindowState {
        view,
        cached_handlers,
        cached_action_root,
    } = st;
    render_app_ui_with_cached_handlers::<ViewRuntimeActionHooksOwner, _>(
        cx,
        "__fret.view.action_root",
        cached_handlers,
        cached_action_root,
        |app_ui| view.render(app_ui),
    )
}

/// Render a manual declarative root through the grouped `AppUi` authoring surface.
///
/// This is the explicit advanced/manual-assembly bridge for `UiTree` / `FnDriver` code that still
/// owns its own window state but wants the same `AppUi` + `LocalState` authoring lane that the
/// higher-level `View` runtime uses.
pub fn render_root_with_app_ui<'a, H: UiHost + 'static>(
    cx: fret_ui::declarative::RenderRootContext<'a, H>,
    root_name: &str,
    state: &mut AppUiRenderRootState,
    render: impl for<'cx, 'el> FnOnce(&mut AppUi<'cx, 'el, H>) -> crate::Ui,
) -> fret_core::NodeId {
    let fret_ui::declarative::RenderRootContext {
        ui,
        app,
        services,
        window,
        bounds,
    } = cx;
    fret_ui::declarative::render_root(ui, app, services, window, bounds, root_name, |cx| {
        render_app_ui_with_cached_handlers::<AppUiRenderRootActionHooksOwner, _>(
            cx,
            "__fret.advanced.view.render_root_with_app_ui.action_root",
            &mut state.cached_handlers,
            &mut state.cached_action_root,
            render,
        )
    })
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[doc(hidden)]
pub fn view_record_engine_frame<V: View>(
    _app: &mut fret_app::App,
    _window: AppWindowId,
    ui: &mut fret_ui::UiTree<fret_app::App>,
    _state: &mut ViewWindowState<V>,
    _context: &crate::kernel::render::WgpuContext,
    _renderer: &mut crate::kernel::render::Renderer,
    _scale_factor: f32,
    _tick_id: fret_runtime::TickId,
    _frame_id: fret_runtime::FrameId,
) -> fret_launch::EngineFrameUpdate {
    if !ui.view_cache_enabled() {
        ui.set_view_cache_enabled(true);
    }
    fret_launch::EngineFrameUpdate::default()
}

#[cfg(test)]
mod tests {
    use super::{
        AppActivateExt, AppActivateSurface, AppUiRenderRootState, LocalActionCapture, LocalState,
        LocalStateTxn, OnActivate, UiCxActionsExt as _, View, ViewWindowState, action_listener,
        dispatch_action_listener, dispatch_payload_action_listener, render_root_with_app_ui,
        view_init_window, view_view,
    };
    use std::any::Any;
    #[cfg(feature = "state-mutation")]
    use std::cell::RefCell;
    #[cfg(feature = "state-mutation")]
    use std::future::Future;
    #[cfg(feature = "state-mutation")]
    use std::pin::Pin;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    #[cfg(feature = "state-mutation")]
    use std::task::{Context, Poll, Waker};
    const VIEW_RS_SOURCE: &str = include_str!("view.rs");
    use fret_core::{
        AppWindowId, FrameId, Modifiers, MouseButton, NodeId, Point, PointerEvent, PointerType, Px,
        Rect, Size, TextConstraints, TextMetrics, WindowMetricsService,
    };
    use fret_runtime::{
        ActionId, CommandId, Effect, ModelStore, TickId, TimerToken,
        WindowPendingActionPayloadService,
    };
    #[cfg(feature = "state-mutation")]
    use fret_runtime::{
        DispatchPriority, Dispatcher, DispatcherHandle, ExecCapabilities, InboxDrainRegistry,
        Runnable,
    };
    use fret_ui::action::{ActionCx, ActivateReason, UiActionHost, UiFocusActionHost};
    use fret_ui::declarative::render_root;
    use fret_ui::{UiTree, element::Length};

    #[derive(Default)]
    struct FakeUiServices;

    impl fret_core::TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    impl fret_core::MaterialService for FakeUiServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    #[cfg(feature = "state-mutation")]
    #[derive(Default)]
    struct TestDispatcher;

    #[cfg(feature = "state-mutation")]
    impl Dispatcher for TestDispatcher {
        fn dispatch_on_main_thread(&self, runnable: Runnable) {
            runnable();
        }

        fn dispatch_background(&self, runnable: Runnable, _priority: DispatchPriority) {
            runnable();
        }

        fn dispatch_after(&self, _delay: std::time::Duration, runnable: Runnable) {
            runnable();
        }

        fn wake(&self, _window: Option<AppWindowId>) {}

        fn exec_capabilities(&self) -> ExecCapabilities {
            ExecCapabilities::default()
        }
    }

    #[cfg(feature = "state-mutation")]
    #[derive(Default)]
    struct ReadyOnlySpawner;

    #[cfg(feature = "state-mutation")]
    impl fret_mutation::FutureSpawner for ReadyOnlySpawner {
        fn spawn_send(&self, mut fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
            let mut cx = Context::from_waker(Waker::noop());
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(()) => {}
                Poll::Pending => panic!("test mutation future should complete immediately"),
            }
        }

        fn spawn_local(&self, mut fut: Pin<Box<dyn Future<Output = ()> + 'static>>) -> bool {
            let mut cx = Context::from_waker(Waker::noop());
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(()) => true,
                Poll::Pending => panic!("test mutation future should complete immediately"),
            }
        }
    }

    #[cfg(feature = "state-mutation")]
    fn drain_inboxes(app: &mut crate::app::App, window: AppWindowId) -> bool {
        app.with_global_mut_untracked(InboxDrainRegistry::default, |registry, app| {
            registry.drain_all(app, Some(window))
        })
    }

    struct DispatchAction;
    impl fret_runtime::TypedAction for DispatchAction {
        fn action_id() -> ActionId {
            ActionId::from("test.dispatch_action.v1")
        }
    }

    struct DispatchPayloadAction;
    impl fret_runtime::TypedAction for DispatchPayloadAction {
        fn action_id() -> ActionId {
            ActionId::from("test.dispatch_payload_action.v1")
        }
    }
    impl crate::actions::TypedPayloadAction for DispatchPayloadAction {
        type Payload = u64;
    }

    struct RuntimeIncrementAction;
    impl fret_runtime::TypedAction for RuntimeIncrementAction {
        fn action_id() -> ActionId {
            ActionId::from("test.locals_with.runtime.increment.v1")
        }
    }

    struct RuntimePayloadAppendAction;
    impl fret_runtime::TypedAction for RuntimePayloadAppendAction {
        fn action_id() -> ActionId {
            ActionId::from("test.uicx.payload_models.append.v1")
        }
    }
    impl crate::actions::TypedPayloadAction for RuntimePayloadAppendAction {
        type Payload = u64;
    }

    #[derive(Default)]
    struct DummyActivateSurface {
        on_activate: Option<OnActivate>,
    }

    impl AppActivateSurface for DummyActivateSurface {
        fn on_activate(mut self, on_activate: OnActivate) -> Self {
            self.on_activate = Some(on_activate);
            self
        }
    }

    #[derive(Default)]
    struct FakeHost {
        models: ModelStore,
        redraws: Vec<AppWindowId>,
        notifies: Vec<ActionCx>,
        effects: Vec<Effect>,
        dispatch_sources: Vec<(ActionCx, CommandId, ActivateReason)>,
        payloads: Vec<(ActionCx, ActionId, Box<dyn Any + Send + Sync>)>,
        next_timer: u64,
    }

    impl UiActionHost for FakeHost {
        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraws.push(window);
        }

        fn next_timer_token(&mut self) -> TimerToken {
            let current = self.next_timer;
            self.next_timer = self.next_timer.saturating_add(1);
            TimerToken(current)
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            fret_runtime::ClipboardToken::default()
        }

        fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
            fret_runtime::ShareSheetToken::default()
        }

        fn notify(&mut self, cx: ActionCx) {
            self.notifies.push(cx);
        }

        fn record_pending_command_dispatch_source(
            &mut self,
            cx: ActionCx,
            command: &CommandId,
            reason: ActivateReason,
        ) {
            self.dispatch_sources.push((cx, command.clone(), reason));
        }

        fn record_pending_action_payload(
            &mut self,
            cx: ActionCx,
            action: &ActionId,
            payload: Box<dyn Any + Send + Sync>,
        ) {
            self.payloads.push((cx, action.clone(), payload));
        }
    }

    impl UiFocusActionHost for FakeHost {
        fn request_focus(&mut self, _target: fret_ui::GlobalElementId) {}
    }

    struct RuntimeLocalsWithView {
        count: Option<LocalState<u32>>,
        touched: Option<LocalState<bool>>,
        renders: Arc<AtomicUsize>,
        last_seen_count: Arc<AtomicUsize>,
    }

    impl View for RuntimeLocalsWithView {
        fn init(_app: &mut crate::app::App, _window: crate::WindowId) -> Self {
            Self {
                count: None,
                touched: None,
                renders: Arc::new(AtomicUsize::new(0)),
                last_seen_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn render(&mut self, cx: &mut crate::AppUi<'_, '_>) -> crate::Ui {
            self.renders.fetch_add(1, Ordering::SeqCst);

            if self.count.is_none() {
                self.count = Some(cx.state().local_init(|| 0u32));
            }
            if self.touched.is_none() {
                self.touched = Some(cx.state().local_init(|| false));
            }

            let count = self
                .count
                .as_ref()
                .expect("count local should exist")
                .clone();
            let touched = self
                .touched
                .as_ref()
                .expect("touched local should exist")
                .clone();

            cx.actions()
                .locals_with((&count, &touched))
                .on::<RuntimeIncrementAction>(|tx, (count, touched)| {
                    let incremented = tx.update_if(&count, |value| {
                        *value += 1;
                        true
                    });
                    let flagged = tx.set(&touched, true);
                    incremented || flagged
                });

            self.last_seen_count
                .store(count.layout_value(cx) as usize, Ordering::SeqCst);

            let mut props = fret_ui::element::ContainerProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            let cx = cx.elements();
            cx.container(props, |_cx| Vec::new()).into()
        }
    }

    fn render_runtime_view(
        ui: &mut UiTree<crate::app::App>,
        app: &mut crate::app::App,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        st: &mut ViewWindowState<RuntimeLocalsWithView>,
    ) -> NodeId {
        let root = render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "locals-with-runtime",
            |cx| view_view(cx, st),
        );
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn seed_runtime_window_metrics(
        app: &mut crate::app::App,
        window: AppWindowId,
        bounds: Rect,
        scale_factor: f32,
    ) {
        app.with_global_mut_untracked(WindowMetricsService::default, |svc, _app| {
            svc.set_inner_size(window, bounds.size);
            svc.set_scale_factor(window, scale_factor);
            svc.set_focused(window, true);
        });
        app.with_global_mut_untracked(fret_ui::elements::ElementRuntime::new, |rt, _app| {
            rt.set_window_primary_pointer_type(window, PointerType::Unknown);
        });
    }

    fn render_runtime_view_semantics<V: View>(
        ui: &mut UiTree<crate::app::App>,
        app: &mut crate::app::App,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        scale_factor: f32,
        frame_id: u64,
        root_name: &str,
        st: &mut ViewWindowState<V>,
    ) -> fret_core::SemanticsSnapshot {
        app.set_tick_id(TickId(frame_id));
        app.set_frame_id(FrameId(frame_id));
        seed_runtime_window_metrics(app, window, bounds, scale_factor);

        let root = render_root(ui, app, services, window, bounds, root_name, |cx| {
            view_view(cx, st)
        });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, scale_factor);
        ui.semantics_snapshot()
            .expect("runtime semantics snapshot")
            .clone()
    }

    fn snapshot_test_ids(snapshot: &fret_core::SemanticsSnapshot) -> Vec<String> {
        let mut ids: Vec<String> = snapshot
            .nodes
            .iter()
            .filter_map(|node| node.test_id.as_ref().map(ToString::to_string))
            .collect();
        ids.sort();
        ids
    }

    struct RuntimeToggleGroupFooterView {
        filter: fret_runtime::Model<Option<Arc<str>>>,
        action_flag: Option<LocalState<bool>>,
    }

    impl View for RuntimeToggleGroupFooterView {
        fn init(app: &mut crate::app::App, _window: crate::WindowId) -> Self {
            Self {
                filter: app.models_mut().insert(Some(Arc::from("all"))),
                action_flag: None,
            }
        }

        fn render(&mut self, cx: &mut crate::AppUi<'_, '_>) -> crate::Ui {
            if self.action_flag.is_none() {
                self.action_flag = Some(cx.state().local_init(|| false));
            }
            let action_flag = self
                .action_flag
                .as_ref()
                .expect("action flag should exist")
                .clone();
            cx.actions()
                .local(&action_flag)
                .update::<RuntimeIncrementAction>(|value| *value = !*value);
            let _flag = action_flag.layout_value(cx);

            let viewport = cx.environment_viewport_bounds(fret_ui::Invalidation::Layout);
            let compact = viewport.size.width.0 < 560.0;

            let cx = cx.elements();

            let filters = crate::shadcn::ToggleGroup::single(self.filter.clone())
                .deselectable(false)
                .items([
                    crate::shadcn::ToggleGroupItem::new("all", [cx.text("All")])
                        .test_id("runtime.toggle.filter.all"),
                    crate::shadcn::ToggleGroupItem::new("active", [cx.text("Active")])
                        .test_id("runtime.toggle.filter.active"),
                    crate::shadcn::ToggleGroupItem::new("completed", [cx.text("Completed")])
                        .test_id("runtime.toggle.filter.completed"),
                ])
                .into_element(cx);

            let clear = crate::shadcn::Button::new("Clear")
                .test_id("runtime.toggle.clear")
                .into_element(cx);
            let body = cx.text("Body").test_id("runtime.toggle.body");

            let footer = if compact {
                let clear_row = cx
                    .flex(
                        fret_ui::element::FlexProps {
                            direction: fret_core::Axis::Horizontal,
                            ..Default::default()
                        },
                        move |_cx| vec![clear],
                    )
                    .test_id("runtime.toggle.clear_row");
                cx.flex(
                    fret_ui::element::FlexProps {
                        direction: fret_core::Axis::Vertical,
                        ..Default::default()
                    },
                    move |_cx| vec![filters, clear_row],
                )
                .test_id("runtime.toggle.footer.compact")
            } else {
                cx.flex(
                    fret_ui::element::FlexProps {
                        direction: fret_core::Axis::Horizontal,
                        ..Default::default()
                    },
                    move |_cx| vec![filters, clear],
                )
                .test_id("runtime.toggle.footer.roomy")
            };

            let mut page_props = fret_ui::element::FlexProps::default();
            page_props.direction = fret_core::Axis::Vertical;
            page_props.layout.size.width = Length::Fill;
            page_props.layout.size.height = Length::Fill;

            cx.flex(page_props, move |_cx| vec![body, footer]).into()
        }
    }

    struct ManualRuntimeLocalsWithRoot {
        app_ui_root: AppUiRenderRootState,
        count: Option<LocalState<u32>>,
        touched: Option<LocalState<bool>>,
        renders: Arc<AtomicUsize>,
        last_seen_count: Arc<AtomicUsize>,
    }

    impl Default for ManualRuntimeLocalsWithRoot {
        fn default() -> Self {
            Self {
                app_ui_root: AppUiRenderRootState::default(),
                count: None,
                touched: None,
                renders: Arc::new(AtomicUsize::new(0)),
                last_seen_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    fn render_manual_runtime_view(
        ui: &mut UiTree<crate::app::App>,
        app: &mut crate::app::App,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        st: &mut ManualRuntimeLocalsWithRoot,
    ) -> NodeId {
        let ManualRuntimeLocalsWithRoot {
            app_ui_root,
            count,
            touched,
            renders,
            last_seen_count,
        } = st;
        let root = render_root_with_app_ui(
            fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
            "manual-locals-with-runtime",
            app_ui_root,
            |cx| {
                renders.fetch_add(1, Ordering::SeqCst);

                if count.is_none() {
                    *count = Some(cx.state().local_init(|| 0u32));
                }
                if touched.is_none() {
                    *touched = Some(cx.state().local_init(|| false));
                }

                let count = count.as_ref().expect("count local should exist").clone();
                let touched = touched
                    .as_ref()
                    .expect("touched local should exist")
                    .clone();

                cx.actions()
                    .locals_with((&count, &touched))
                    .on::<RuntimeIncrementAction>(|tx, (count, touched)| {
                        let incremented = tx.update_if(&count, |value| {
                            *value += 1;
                            true
                        });
                        let flagged = tx.set(&touched, true);
                        incremented || flagged
                    });

                last_seen_count.store(count.layout_value(cx) as usize, Ordering::SeqCst);

                let mut props = fret_ui::element::ContainerProps::default();
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;

                let cx = cx.elements();
                cx.container(props, |_cx| Vec::new()).into()
            },
        );
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn first_leaf(ui: &UiTree<crate::app::App>, mut node: NodeId) -> NodeId {
        loop {
            let children = ui.children(node);
            if children.is_empty() {
                return node;
            }
            node = children[0];
        }
    }

    #[test]
    fn local_state_value_in_helpers_clone_store_values() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(String::from("hello")),
        };

        assert_eq!(local.value_in(&host.models), Some(String::from("hello")));
        assert_eq!(
            local.value_in_or(&host.models, String::from("fallback")),
            String::from("hello")
        );
        assert_eq!(
            LocalState {
                model: host.models.insert(String::new()),
            }
            .value_in_or_default(&host.models),
            String::new()
        );
    }

    #[test]
    fn local_state_from_model_wraps_existing_raw_handle() {
        let mut host = FakeHost::default();
        let model = host.models.insert(String::from("hello"));
        let local = LocalState::from_model(model.clone());

        assert_eq!(local.model(), &model);
        assert_eq!(local.value_in(&host.models), Some(String::from("hello")));
    }

    #[test]
    fn local_state_new_in_allocates_without_exposing_raw_model_handle() {
        let mut host = FakeHost::default();
        let local = LocalState::new_in(&mut host.models, String::from("hello"));

        assert_eq!(local.value_in(&host.models), Some(String::from("hello")));
    }

    #[test]
    fn local_state_borrowed_read_helpers_project_without_clone_noise() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);
        let mut services = FakeUiServices;
        let local = LocalState::from_model(app.models_mut().insert(vec![1u32, 2, 3]));

        let root = render_root_with_app_ui(
            fret_ui::declarative::RenderRootContext::new(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
            ),
            "local-state-borrowed-read",
            &mut AppUiRenderRootState::default(),
            |cx| {
                let layout_len = local.layout_read_ref(cx, |values| values.len());
                let paint_len = local.paint_read_ref(cx, |values| values.len());
                assert_eq!(layout_len, 3);
                assert_eq!(paint_len, 3);
                let cx = cx.elements();
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                })
                .into()
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    #[test]
    fn local_state_bridge_read_helpers_project_without_clone_noise() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);
        let mut services = FakeUiServices;
        let local = LocalState::from_model(app.models_mut().insert(vec![1u32, 2, 3]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "local-state-bridge-read",
            |cx| {
                let layout_values = local.layout_value_in(cx);
                let layout_len = local.layout_read_ref_in(cx, |values| values.len());
                let paint_values = local.paint_value_in(cx);
                let paint_len = local.paint_read_ref_in(cx, |values| values.len());
                assert_eq!(layout_values, vec![1u32, 2, 3]);
                assert_eq!(layout_len, 3);
                assert_eq!(paint_values, vec![1u32, 2, 3]);
                assert_eq!(paint_len, 3);
                vec![
                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into(),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    #[test]
    fn local_state_txn_value_reads_initialized_locals_without_fallback_noise() {
        let mut host = FakeHost::default();
        let draft = LocalState {
            model: host.models.insert(String::from("draft")),
        };
        let next_id = LocalState {
            model: host.models.insert(7u64),
        };

        let tx = LocalStateTxn {
            models: &mut host.models,
        };

        assert_eq!(tx.value(&draft), String::from("draft"));
        assert_eq!(tx.value(&next_id), 7u64);
    }

    #[test]
    fn local_action_capture_clones_local_state_handles_from_refs() {
        let mut host = FakeHost::default();
        let draft = LocalState {
            model: host.models.insert(String::from("draft")),
        };
        let next_id = LocalState {
            model: host.models.insert(7u64),
        };

        let (draft_capture, next_id_capture) = (&draft, &next_id).capture_owned();

        assert_eq!(
            draft_capture.value_in(&host.models),
            Some(String::from("draft"))
        );
        assert_eq!(next_id_capture.value_in(&host.models), Some(7u64));
    }

    #[test]
    fn local_action_capture_supports_wide_local_tuples() {
        let mut host = FakeHost::default();
        let a = LocalState {
            model: host.models.insert(1u64),
        };
        let b = LocalState {
            model: host.models.insert(2u64),
        };
        let c = LocalState {
            model: host.models.insert(3u64),
        };
        let d = LocalState {
            model: host.models.insert(4u64),
        };
        let e = LocalState {
            model: host.models.insert(5u64),
        };
        let f = LocalState {
            model: host.models.insert(6u64),
        };
        let g = LocalState {
            model: host.models.insert(7u64),
        };
        let h = LocalState {
            model: host.models.insert(8u64),
        };

        let captures = (&a, &b, &c, &d, &e, &f, &g, &h).capture_owned();

        assert_eq!(captures.0.value_in(&host.models), Some(1u64));
        assert_eq!(captures.1.value_in(&host.models), Some(2u64));
        assert_eq!(captures.2.value_in(&host.models), Some(3u64));
        assert_eq!(captures.3.value_in(&host.models), Some(4u64));
        assert_eq!(captures.4.value_in(&host.models), Some(5u64));
        assert_eq!(captures.5.value_in(&host.models), Some(6u64));
        assert_eq!(captures.6.value_in(&host.models), Some(7u64));
        assert_eq!(captures.7.value_in(&host.models), Some(8u64));
    }

    #[test]
    fn local_state_update_in_if_returns_closure_handled_state() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(vec![1u64, 2, 3]),
        };

        assert!(local.update_in_if(&mut host.models, |values| {
            let before = values.len();
            values.retain(|value| *value != 2);
            values.len() != before
        }));
        assert_eq!(
            host.models
                .read(local.model(), |values| values.clone())
                .unwrap(),
            vec![1, 3]
        );
        assert!(!local.update_in_if(&mut host.models, |values| {
            let before = values.len();
            values.retain(|value| *value != 99);
            values.len() != before
        }));
    }

    #[test]
    fn local_state_update_action_requests_redraw_and_notify() {
        let mut host = FakeHost::default();
        let model = host.models.insert(1i32);
        let local = LocalState {
            model: model.clone(),
        };
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(42),
        };

        assert!(local.update_action(&mut host, action_cx, |value| *value += 1));
        assert_eq!(host.models.read(&model, |value| *value).unwrap(), 2);
        assert_eq!(host.redraws, vec![action_cx.window]);
        assert_eq!(host.notifies, vec![action_cx]);
    }

    #[test]
    fn local_state_update_action_if_only_notifies_when_handled() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(vec![1u64, 2, 3]),
        };
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(7),
        };

        assert!(local.update_action_if(&mut host, action_cx, |values| {
            let before = values.len();
            values.retain(|value| *value != 2);
            values.len() != before
        }));
        assert_eq!(host.redraws, vec![action_cx.window]);
        assert_eq!(host.notifies, vec![action_cx]);

        host.redraws.clear();
        host.notifies.clear();
        assert!(!local.update_action_if(&mut host, action_cx, |values| {
            let before = values.len();
            values.retain(|value| *value != 99);
            values.len() != before
        }));
        assert!(host.redraws.is_empty());
        assert!(host.notifies.is_empty());
    }

    #[test]
    fn local_state_update_action_if_can_use_payload_from_closure() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(vec![1u64, 2, 3]),
        };
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(9),
        };

        assert!(local.update_action_if(&mut host, action_cx, |values| {
            let remove_id = 2u64;
            let before = values.len();
            values.retain(|value| *value != remove_id);
            values.len() != before
        }));
        assert_eq!(host.redraws, vec![action_cx.window]);
        assert_eq!(host.notifies, vec![action_cx]);
    }

    #[test]
    fn locals_with_runtime_dispatch_updates_locals_and_rerenders_cached_view() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);

        let mut services = FakeUiServices;
        let mut st = view_init_window::<RuntimeLocalsWithView>(&mut app, window);

        app.set_frame_id(FrameId(1));
        let root = render_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        ui.set_focus(Some(first_leaf(&ui, root)));
        assert!(
            st.cached_handlers.is_some(),
            "view render should install cached action handlers before command dispatch"
        );
        assert!(
            st.cached_action_root.is_some(),
            "view render should cache the concrete action root used for runtime dispatch"
        );
        assert_eq!(st.view.renders.load(Ordering::SeqCst), 1);
        assert_eq!(st.view.last_seen_count.load(Ordering::SeqCst), 0);

        app.set_frame_id(FrameId(2));
        render_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        assert_eq!(
            st.view.renders.load(Ordering::SeqCst),
            1,
            "expected the view-cache root to reuse the previous frame before any notify-driven invalidation"
        );

        let command = <RuntimeIncrementAction as fret_runtime::TypedAction>::action_id();
        assert!(ui.dispatch_command(&mut app, &mut services, &command));
        assert_eq!(
            st.view
                .count
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(1)
        );
        assert_eq!(
            st.view
                .touched
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(true)
        );
        assert!(
            app.flush_effects()
                .iter()
                .any(|effect| matches!(effect, Effect::Redraw(redraw) if *redraw == window)),
            "locals_with action dispatch should request a redraw through the runtime host"
        );

        app.set_frame_id(FrameId(3));
        render_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        assert_eq!(
            st.view.renders.load(Ordering::SeqCst),
            2,
            "notify should force the cached view root to rerender on the next frame"
        );
        assert_eq!(st.view.last_seen_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn view_runtime_cache_enable_transition_keeps_toggle_group_footer_semantics_after_compact_resize()
     {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let roomy_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(660.0)),
        );
        let compact_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(560.0)),
        );
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let mut services = FakeUiServices;
        let mut st = view_init_window::<RuntimeToggleGroupFooterView>(&mut app, window);

        let _frame1 = render_runtime_view_semantics(
            &mut ui,
            &mut app,
            &mut services,
            window,
            roomy_bounds,
            2.0,
            1,
            "runtime-toggle-group-footer",
            &mut st,
        );

        ui.set_view_cache_enabled(true);

        let mut failures: Vec<String> = Vec::new();
        for frame_id in 2..=8 {
            let snapshot = render_runtime_view_semantics(
                &mut ui,
                &mut app,
                &mut services,
                window,
                compact_bounds,
                2.0,
                frame_id,
                "runtime-toggle-group-footer",
                &mut st,
            );
            let ids = snapshot_test_ids(&snapshot);
            for expected in [
                "runtime.toggle.body",
                "runtime.toggle.footer.compact",
                "runtime.toggle.clear_row",
                "runtime.toggle.clear",
                "runtime.toggle.filter.all",
                "runtime.toggle.filter.active",
                "runtime.toggle.filter.completed",
            ] {
                if !ids.iter().any(|id| id == expected) {
                    let cache_roots = ui.debug_cache_root_stats();
                    let removed = ui.debug_removed_subtrees();
                    failures.push(format!(
                        "frame{frame_id} should keep {expected} after runtime cache-enable transition; ids={ids:?}; cache_roots={cache_roots:?}; removed={removed:?}"
                    ));
                }
            }
        }

        if !failures.is_empty() {
            panic!("{}", failures.join("\n"));
        }
    }

    #[test]
    fn manual_render_root_with_app_ui_keeps_handlers_and_local_state_alive() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);

        let mut services = FakeUiServices;
        let mut st = ManualRuntimeLocalsWithRoot::default();

        app.set_frame_id(FrameId(1));
        let root =
            render_manual_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        ui.set_focus(Some(first_leaf(&ui, root)));
        assert!(
            st.app_ui_root.cached_handlers.is_some(),
            "manual AppUi root should install cached action handlers before command dispatch"
        );
        assert!(
            st.app_ui_root.cached_action_root.is_some(),
            "manual AppUi root should cache the concrete action root used for runtime dispatch"
        );
        assert_eq!(st.renders.load(Ordering::SeqCst), 1);
        assert_eq!(st.last_seen_count.load(Ordering::SeqCst), 0);

        app.set_frame_id(FrameId(2));
        render_manual_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        assert_eq!(
            st.renders.load(Ordering::SeqCst),
            1,
            "expected the manual AppUi root to reuse the previous frame before any notify-driven invalidation"
        );

        let command = <RuntimeIncrementAction as fret_runtime::TypedAction>::action_id();
        assert!(ui.dispatch_command(&mut app, &mut services, &command));
        assert_eq!(
            st.count
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(1)
        );
        assert_eq!(
            st.touched
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(true)
        );
        assert!(
            app.flush_effects()
                .iter()
                .any(|effect| matches!(effect, Effect::Redraw(redraw) if *redraw == window)),
            "manual AppUi root dispatch should request a redraw through the runtime host"
        );

        app.set_frame_id(FrameId(3));
        render_manual_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        assert_eq!(
            st.renders.load(Ordering::SeqCst),
            2,
            "notify should force the cached manual AppUi root to rerender on the next frame"
        );
        assert_eq!(st.last_seen_count.load(Ordering::SeqCst), 1);
    }

    #[cfg(feature = "state-mutation")]
    #[test]
    fn app_ui_data_update_after_mutation_completion_projects_terminal_state_once() {
        fn render_frame(
            app: &mut crate::app::App,
            ui: &mut UiTree<crate::app::App>,
            services: &mut FakeUiServices,
            window: AppWindowId,
            bounds: Rect,
            st: &mut AppUiRenderRootState,
            handle_cell: &RefCell<Option<fret_mutation::MutationHandle<u8, u8>>>,
            count_cell: &RefCell<Option<LocalState<u32>>>,
            status_cell: &RefCell<Option<LocalState<String>>>,
            frame_id: u64,
        ) {
            app.set_frame_id(FrameId(frame_id));
            let root = render_root_with_app_ui(
                fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
                "mutation-completion-update",
                st,
                |cx| {
                    let applied_count = cx.state().local_init(|| 0u32);
                    let applied_status = cx.state().local_init(|| "Idle".to_string());
                    let handle = cx.data().mutation_async(
                        fret_mutation::MutationPolicy::default(),
                        |_token, input: Arc<u8>| async move {
                            if *input == 0 {
                                Err(fret_mutation::MutationError::transient("boom"))
                            } else {
                                Ok(*input)
                            }
                        },
                    );
                    if handle_cell.borrow().is_none() {
                        *handle_cell.borrow_mut() = Some(handle.clone());
                    }
                    if count_cell.borrow().is_none() {
                        *count_cell.borrow_mut() = Some(applied_count.clone());
                    }
                    if status_cell.borrow().is_none() {
                        *status_cell.borrow_mut() = Some(applied_status.clone());
                    }
                    let _ = cx.data().update_after_mutation_completion(
                        0xF123_2002,
                        &handle,
                        |models, st| {
                            let mut changed = false;
                            changed = applied_count
                                .update_in(models, |value| *value = value.saturating_add(1))
                                || changed;
                            let next_status = if st.is_success() {
                                "Success".to_string()
                            } else {
                                "Error".to_string()
                            };
                            changed = applied_status.set_in(models, next_status) || changed;
                            changed
                        },
                    );

                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into()
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        }

        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        let dispatcher: DispatcherHandle = Arc::new(TestDispatcher);
        app.set_global::<DispatcherHandle>(dispatcher);
        let spawner: fret_mutation::FutureSpawnerHandle = Arc::new(ReadyOnlySpawner);
        app.set_global::<fret_mutation::FutureSpawnerHandle>(spawner);

        let mut services = FakeUiServices;
        let mut st = AppUiRenderRootState::default();
        let handle_cell = RefCell::new(None::<fret_mutation::MutationHandle<u8, u8>>);
        let count_cell = RefCell::new(None::<LocalState<u32>>);
        let status_cell = RefCell::new(None::<LocalState<String>>);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &count_cell,
            &status_cell,
            1,
        );

        let handle = handle_cell
            .borrow()
            .as_ref()
            .expect("mutation handle should be captured")
            .clone();
        let applied_count = count_cell
            .borrow()
            .as_ref()
            .expect("applied_count local should be captured")
            .clone();
        let applied_status = status_cell
            .borrow()
            .as_ref()
            .expect("applied_status local should be captured")
            .clone();

        assert_eq!(applied_count.value_in_or_default(app.models_mut()), 0);
        assert_eq!(applied_status.value_in_or_default(app.models_mut()), "Idle");

        assert!(handle.submit(app.models_mut(), window, 0));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &count_cell,
            &status_cell,
            2,
        );
        assert_eq!(applied_count.value_in_or_default(app.models_mut()), 1);
        assert_eq!(
            applied_status.value_in_or_default(app.models_mut()),
            "Error"
        );

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &count_cell,
            &status_cell,
            3,
        );
        assert_eq!(
            applied_count.value_in_or_default(app.models_mut()),
            1,
            "same terminal completion should not reapply the projection"
        );

        assert!(handle.submit(app.models_mut(), window, 1));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &count_cell,
            &status_cell,
            4,
        );
        assert_eq!(applied_count.value_in_or_default(app.models_mut()), 2);
        assert_eq!(
            applied_status.value_in_or_default(app.models_mut()),
            "Success"
        );
    }

    #[cfg(feature = "state-mutation")]
    #[test]
    fn app_ui_data_take_mutation_completion_only_fires_once_per_terminal_state() {
        fn render_frame(
            app: &mut crate::app::App,
            ui: &mut UiTree<crate::app::App>,
            services: &mut FakeUiServices,
            window: AppWindowId,
            bounds: Rect,
            st: &mut AppUiRenderRootState,
            handle_cell: &RefCell<Option<fret_mutation::MutationHandle<u8, u8>>>,
            completions_seen: &Arc<AtomicUsize>,
            frame_id: u64,
        ) {
            app.set_frame_id(FrameId(frame_id));
            let root = render_root_with_app_ui(
                fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
                "mutation-completion-once",
                st,
                |cx| {
                    let handle = cx.data().mutation_async(
                        fret_mutation::MutationPolicy::default(),
                        |_token, input: Arc<u8>| async move {
                            if *input == 0 {
                                Err(fret_mutation::MutationError::transient("boom"))
                            } else {
                                Ok(*input)
                            }
                        },
                    );
                    if handle_cell.borrow().is_none() {
                        *handle_cell.borrow_mut() = Some(handle.clone());
                    }
                    if cx.data().take_mutation_completion(0xF123_2000, &handle) {
                        completions_seen.fetch_add(1, Ordering::SeqCst);
                    }

                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into()
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        }

        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        let dispatcher: DispatcherHandle = Arc::new(TestDispatcher);
        app.set_global::<DispatcherHandle>(dispatcher);
        let spawner: fret_mutation::FutureSpawnerHandle = Arc::new(ReadyOnlySpawner);
        app.set_global::<fret_mutation::FutureSpawnerHandle>(spawner);

        let mut services = FakeUiServices;
        let mut st = AppUiRenderRootState::default();
        let completions_seen = Arc::new(AtomicUsize::new(0));
        let handle_cell = RefCell::new(None::<fret_mutation::MutationHandle<u8, u8>>);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            1,
        );
        assert_eq!(completions_seen.load(Ordering::SeqCst), 0);

        let handle = handle_cell
            .borrow()
            .as_ref()
            .expect("mutation handle should be captured")
            .clone();

        assert!(handle.submit(app.models_mut(), window, 0));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            2,
        );
        assert_eq!(completions_seen.load(Ordering::SeqCst), 1);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            3,
        );
        assert_eq!(
            completions_seen.load(Ordering::SeqCst),
            1,
            "same terminal completion should not retrigger on later renders"
        );

        assert!(handle.retry_last(app.models_mut(), window));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            4,
        );
        assert_eq!(
            completions_seen.load(Ordering::SeqCst),
            2,
            "retrying the same stored input should still surface a fresh completion"
        );
    }

    #[cfg(feature = "state-mutation")]
    #[test]
    fn app_ui_data_take_mutation_success_only_fires_once_per_completion() {
        fn render_frame(
            app: &mut crate::app::App,
            ui: &mut UiTree<crate::app::App>,
            services: &mut FakeUiServices,
            window: AppWindowId,
            bounds: Rect,
            st: &mut AppUiRenderRootState,
            handle_cell: &RefCell<Option<fret_mutation::MutationHandle<(), ()>>>,
            completions_seen: &Arc<AtomicUsize>,
            frame_id: u64,
        ) {
            app.set_frame_id(FrameId(frame_id));
            let root = render_root_with_app_ui(
                fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
                "mutation-success-once",
                st,
                |cx| {
                    let handle = cx.data().mutation_async(
                        fret_mutation::MutationPolicy::default(),
                        |_token, _input: Arc<()>| async { Ok(()) },
                    );
                    if handle_cell.borrow().is_none() {
                        *handle_cell.borrow_mut() = Some(handle.clone());
                    }
                    if cx.data().take_mutation_success(0xF123_2001, &handle) {
                        completions_seen.fetch_add(1, Ordering::SeqCst);
                    }

                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into()
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        }

        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        let dispatcher: DispatcherHandle = Arc::new(TestDispatcher);
        app.set_global::<DispatcherHandle>(dispatcher);
        let spawner: fret_mutation::FutureSpawnerHandle = Arc::new(ReadyOnlySpawner);
        app.set_global::<fret_mutation::FutureSpawnerHandle>(spawner);

        let mut services = FakeUiServices;
        let mut st = AppUiRenderRootState::default();
        let completions_seen = Arc::new(AtomicUsize::new(0));
        let handle_cell = RefCell::new(None::<fret_mutation::MutationHandle<(), ()>>);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            1,
        );
        assert_eq!(completions_seen.load(Ordering::SeqCst), 0);

        let handle = handle_cell
            .borrow()
            .as_ref()
            .expect("mutation handle should be captured")
            .clone();

        assert!(handle.submit(app.models_mut(), window, ()));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            2,
        );
        assert_eq!(completions_seen.load(Ordering::SeqCst), 1);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            3,
        );
        assert_eq!(
            completions_seen.load(Ordering::SeqCst),
            1,
            "same mutation completion should not retrigger on later renders"
        );

        assert!(handle.submit(app.models_mut(), window, ()));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            4,
        );
        assert_eq!(
            completions_seen.load(Ordering::SeqCst),
            2,
            "a new successful completion should retrigger exactly once"
        );
    }

    #[test]
    fn raw_model_with_reuses_element_context_local_model_substrate() {
        let api_source = VIEW_RS_SOURCE
            .split("\nmod tests {")
            .next()
            .expect("view.rs test module marker should exist");

        assert!(api_source.contains("self.cx.local_model_at(callsite, init)"));
        assert!(
            api_source.contains("self.cx.note_repeated_call_in_render_evaluation_at(callsite)")
        );
        assert!(!api_source.contains("struct RawModelSlot<T>"));
        assert!(!api_source.contains("struct RawModelRenderPassDiagnostics"));
        assert!(!api_source.contains("fn note_raw_model_call_in_render_pass("));
    }

    #[test]
    fn uicx_payload_models_runtime_dispatch_updates_shared_models_and_requests_redraw() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);

        let mut services = FakeUiServices;
        let selected_rows = app.models_mut().insert(Vec::<u64>::new());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "uicx-payload-models-runtime",
            |cx| {
                cx.actions().payload_models::<RuntimePayloadAppendAction>({
                    let selected_rows = selected_rows.clone();
                    move |models, row_id| {
                        models
                            .update(&selected_rows, |rows| rows.push(row_id))
                            .is_ok()
                    }
                });

                vec![
                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into(),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let command = <RuntimePayloadAppendAction as fret_runtime::TypedAction>::action_id();
        app.with_global_mut(WindowPendingActionPayloadService::default, |svc, app| {
            svc.record(window, app.tick_id(), command.clone(), Box::new(41u64));
        });

        assert!(
            ui.dispatch_command(&mut app, &mut services, &command),
            "payload_models dispatch should be handled when a pending payload is present"
        );
        assert_eq!(
            app.models()
                .read(&selected_rows, |rows| rows.clone())
                .ok()
                .unwrap_or_default(),
            vec![41u64]
        );
        assert!(
            app.flush_effects()
                .iter()
                .any(|effect| matches!(effect, Effect::Redraw(redraw) if *redraw == window)),
            "handled payload_models dispatch should request redraw"
        );
    }

    #[cfg(feature = "shadcn")]
    #[test]
    fn checkbox_action_payload_round_trips_through_uicx_payload_models() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);

        let mut services = FakeUiServices;
        let checkbox_checked = app.models_mut().insert(false);
        let selected_rows = app.models_mut().insert(Vec::<u64>::new());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "uicx-payload-models-checkbox",
            |cx| {
                cx.actions().payload_models::<RuntimePayloadAppendAction>({
                    let selected_rows = selected_rows.clone();
                    move |models, row_id| {
                        models
                            .update(&selected_rows, |rows| rows.push(row_id))
                            .is_ok()
                    }
                });

                vec![
                    fret_ui_shadcn::facade::Checkbox::new(checkbox_checked.clone())
                        .test_id("payload-checkbox")
                        .action(
                            <RuntimePayloadAppendAction as fret_runtime::TypedAction>::action_id(),
                        )
                        .action_payload(41u64)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let checkbox = snap
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("payload-checkbox"))
            .expect("checkbox semantics node");
        let position = Point::new(
            Px(checkbox.bounds.origin.x.0 + checkbox.bounds.size.width.0 * 0.5),
            Px(checkbox.bounds.origin.y.0 + checkbox.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let mut saw_command = false;
        for effect in app.flush_effects() {
            match effect {
                Effect::Command {
                    window: Some(target_window),
                    command,
                } if target_window == window => {
                    saw_command = true;
                    assert!(
                        ui.dispatch_command(&mut app, &mut services, &command),
                        "checkbox payload command should be handled by UiCx payload_models"
                    );
                }
                other => app.push_effect(other),
            }
        }

        assert!(saw_command, "checkbox click should emit an Effect::Command");
        assert_eq!(
            app.models()
                .read(&selected_rows, |rows| rows.clone())
                .ok()
                .unwrap_or_default(),
            vec![41u64]
        );
    }

    #[test]
    fn dispatch_listener_queues_a_command_effect() {
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(17),
        };

        let dispatch = dispatch_action_listener::<DispatchAction>();
        dispatch(&mut host, action_cx, ActivateReason::Pointer);

        assert_eq!(
            host.effects,
            vec![Effect::Command {
                window: Some(action_cx.window),
                command: <DispatchAction as fret_runtime::TypedAction>::action_id(),
            }]
        );
        assert_eq!(
            host.dispatch_sources,
            vec![(
                action_cx,
                <DispatchAction as fret_runtime::TypedAction>::action_id(),
                ActivateReason::Pointer
            )]
        );
    }

    #[test]
    fn dispatch_payload_listener_records_payload_before_dispatch() {
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(23),
        };

        let dispatch = dispatch_payload_action_listener::<DispatchPayloadAction>(42);
        dispatch(&mut host, action_cx, ActivateReason::Keyboard);

        assert_eq!(
            host.effects,
            vec![Effect::Command {
                window: Some(action_cx.window),
                command: <DispatchPayloadAction as fret_runtime::TypedAction>::action_id(),
            }]
        );
        assert_eq!(host.payloads.len(), 1);
        assert_eq!(host.payloads[0].0, action_cx);
        assert_eq!(
            host.payloads[0].1,
            <DispatchPayloadAction as fret_runtime::TypedAction>::action_id()
        );
        assert_eq!(host.payloads[0].2.downcast_ref::<u64>().copied(), Some(42));
    }

    #[test]
    fn action_listener_hides_activate_reason_for_simple_widget_glue() {
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(31),
        };

        let listener = action_listener(move |host, cx| {
            host.request_redraw(cx.window);
            host.notify(cx);
        });
        listener(&mut host, action_cx, ActivateReason::Keyboard);

        assert_eq!(host.redraws, vec![action_cx.window]);
        assert_eq!(host.notifies, vec![action_cx]);
    }

    #[test]
    fn app_activate_surface_contract_can_store_activation_handlers() {
        let widget = DummyActivateSurface::default().on_activate(action_listener(|host, cx| {
            host.request_redraw(cx.window);
        }));
        assert!(widget.on_activate.is_some());
    }

    #[test]
    fn app_activate_ext_action_alias_dispatches_without_turbofish() {
        let widget = DummyActivateSurface::default().action(DispatchAction);
        let dispatch = widget
            .on_activate
            .expect("action alias should store an activation handler");
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(77),
        };

        dispatch(&mut host, action_cx, ActivateReason::Pointer);

        assert_eq!(
            host.effects,
            vec![Effect::Command {
                window: Some(action_cx.window),
                command: <DispatchAction as fret_runtime::TypedAction>::action_id(),
            }]
        );
    }

    #[test]
    fn app_activate_ext_action_payload_alias_records_payload_without_turbofish() {
        let widget = DummyActivateSurface::default().action_payload(DispatchPayloadAction, 9);
        let dispatch = widget
            .on_activate
            .expect("action_payload alias should store an activation handler");
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(88),
        };

        dispatch(&mut host, action_cx, ActivateReason::Keyboard);

        assert_eq!(host.payloads.len(), 1);
        assert_eq!(host.payloads[0].0, action_cx);
        assert_eq!(
            host.payloads[0].1,
            <DispatchPayloadAction as fret_runtime::TypedAction>::action_id()
        );
        assert_eq!(host.payloads[0].2.downcast_ref::<u64>().copied(), Some(9));
    }

    #[cfg(feature = "shadcn")]
    #[test]
    fn local_state_supports_text_value_widgets() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(String::from("hello")),
        };

        let _input = fret_ui_shadcn::facade::Input::new(&local);
        let _textarea = fret_ui_shadcn::facade::Textarea::new(&local);
    }

    #[test]
    fn grouped_authoring_surfaces_replace_flat_app_ui_helpers() {
        let api_source = VIEW_RS_SOURCE
            .split("\nmod tests {")
            .next()
            .expect("view.rs test module marker should exist");
        assert!(!api_source.contains("pub fn use_state<"));
        assert!(!api_source.contains("pub fn use_state_keyed<"));
        assert!(!api_source.contains("fn use_state_keyed<"));
        assert!(!api_source.contains("pub fn raw_model<"));
        assert!(!api_source.contains("pub fn use_local<"));
        assert!(!api_source.contains("pub fn use_local_keyed<"));
        assert!(!api_source.contains("pub fn use_local_with<"));
        assert!(!api_source.contains("pub fn on_action_notify_local_update<"));
        assert!(!api_source.contains("pub fn on_action_notify_local_set<"));
        assert!(!api_source.contains("pub fn on_action_notify_toggle_local_bool<"));
        assert!(!api_source.contains("pub fn on_action_notify_models<"));
        assert!(!api_source.contains("pub fn on_action_notify_locals<"));
        assert!(!api_source.contains("pub fn on_action_notify_transient<"));
        assert!(!api_source.contains("fn on_action<A: crate::TypedAction>("));
        assert!(
            !api_source.contains("fn on_payload_action<A: crate::actions::TypedPayloadAction>(")
        );
        assert!(!api_source.contains("fn on_action_availability<A: crate::TypedAction>("));
        assert!(!api_source.contains("fn on_action_notify_model_update<"));
        assert!(!api_source.contains("fn on_action_notify_model_set<"));
        assert!(!api_source.contains("fn on_action_notify_toggle_bool<"));
        assert!(!api_source.contains("pub fn on_payload_action_notify_local_update_if<"));
        assert!(!api_source.contains("pub fn on_payload_action_notify_locals<"));
        assert!(!api_source.contains("pub struct AppUiPayloadActions<"));
        assert!(!api_source.contains("pub struct UiCxPayloadActions<"));
        assert!(!api_source.contains("pub fn payload_locals<"));
        assert!(!api_source.contains("pub fn payload<A>(self) -> AppUiPayloadActions"));
        assert!(!api_source.contains("pub fn payload<A>(self) -> UiCxPayloadActions"));
        assert!(!api_source.contains("pub fn local_update_if<T>("));
        assert!(!api_source.contains(
            "pub fn locals(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>, A::Payload) -> bool + 'static)"
        ));
        assert!(
            api_source.contains("pub fn value<T: Any + Clone>(&self, local: &LocalState<T>) -> T")
        );
        assert!(
            api_source
                .contains("pub fn layout_value<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> T")
        );
        assert!(
            api_source
                .contains("pub fn paint_value<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> T")
        );
        assert!(api_source.contains("pub fn layout_read_ref<'a, H: UiHost + 'a, Cx, R>("));
        assert!(api_source.contains("pub fn paint_read_ref<'a, H: UiHost + 'a, Cx, R>("));
        assert!(api_source.contains("pub fn layout_value_in<'cx, 'm, 'a, H: UiHost>("));
        assert!(api_source.contains("pub fn layout_read_ref_in<'cx, 'm, 'a, H: UiHost, R>("));
        assert!(api_source.contains("pub fn paint_value_in<'cx, 'm, 'a, H: UiHost>("));
        assert!(api_source.contains("pub fn paint_read_ref_in<'cx, 'm, 'a, H: UiHost, R>("));
        assert!(!api_source.contains("pub fn use_selector<"));
        assert!(!api_source.contains("pub fn use_selector_keyed<"));
        assert!(!api_source.contains("pub fn use_query<"));
        assert!(!api_source.contains("pub fn use_query_async<"));
        assert!(!api_source.contains("pub fn use_query_async_local<"));
        assert!(!api_source.contains("pub fn take_transient_on_action_root("));
        assert!(!api_source.contains("pub type WatchedModel"));
        assert!(!api_source.contains("pub type WatchedLocal"));
        assert!(!api_source.contains("pub fn update_action("));
        assert!(!api_source.contains("pub fn update_action_if("));
        assert!(!api_source.contains("pub fn set_action("));
        assert!(!api_source.contains("pub trait LocalSelectorDepsBuilderExt"));
        assert!(api_source.contains("pub(crate) trait LocalSelectorDepsBuilderExt"));
        assert!(api_source.contains("pub trait LocalSelectorLayoutInputs"));
        assert!(api_source.contains("pub trait ModelSelectorInputs"));
        assert!(api_source.contains("pub trait QueryHandleReadLayoutExt<T: 'static>"));
        assert!(!api_source.contains("pub trait AppUiRawStateExt"));
        assert!(api_source.contains("pub trait AppUiRawModelExt"));
        assert!(api_source.contains("pub trait AppUiRawActionNotifyExt"));
        assert!(
            api_source.contains("pub trait AppUiComponentLaneRequiresExplicitElementsEscapeHatch")
        );
        assert!(api_source.contains("pub trait RenderContextAccess<'a, H: UiHost + 'a>"));
        assert!(api_source.contains("pub trait UiCxDataExt"));
        assert!(api_source.contains("pub trait UiCxActionsExt"));
        assert!(!api_source.contains("pub fn watch_local<'m, T: Any>("));
        assert!(api_source.contains("pub(crate) fn watch_local<'m, T: Any>("));
        assert!(!api_source.contains("pub fn action_root(&self) -> fret_ui::GlobalElementId"));
        assert!(!api_source.contains("pub fn new(cx: &'cx mut ElementContext<'a, H>, action_root: fret_ui::GlobalElementId) -> Self"));
        assert!(api_source.contains("pub(crate) fn new("));
        assert!(api_source.contains("pub fn actions(&mut self) -> AppUiActions"));
        assert!(api_source.contains(
            "impl<'cx, 'a, H: UiHost> fret_ui_kit::command::ElementCommandGatingExt for AppUi<'cx, 'a, H> {"
        ));
        assert!(api_source.contains(
            "pub fn request_animation_frame(&mut self) {\n        self.cx.request_animation_frame();\n    }"
        ));
        assert!(api_source.contains("pub fn layout_query_bounds("));
        assert!(api_source.contains("pub fn layout_query_region_with_id<I>("));
        assert!(api_source.contains("pub fn layout_query_region<I>("));
        assert!(api_source.contains(
            "let mut carried_action_handlers = Some(std::mem::take(&mut self.action_handlers));"
        ));
        assert!(api_source.contains("self.cx.layout_query_region_with_id(props, |cx, id| {"));
        assert!(api_source.contains("pub fn scope<R>(&mut self, _f: impl FnOnce(&mut Self) -> R)"));
        assert!(
            api_source.contains(
                "pub fn named<R>(&mut self, _name: &str, _f: impl FnOnce(&mut Self) -> R)"
            )
        );
        assert!(api_source.contains(
            "pub fn slot_state<S: Any, R>(&mut self, _init: impl FnOnce() -> S, _f: impl FnOnce(&mut S) -> R)"
        ));
        assert!(api_source.contains(
            "pub fn local_model<T: Any>(&mut self, _init: impl FnOnce() -> T)\n    where"
        ));
        assert!(api_source.contains(
            "pub fn local_model_keyed<K: Hash, T: Any>(&mut self, _key: K, _init: impl FnOnce() -> T)"
        ));
        assert!(api_source.contains("pub fn state_for<S: Any, R>("));
        assert!(
            api_source.contains("Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,")
        );
        assert!(api_source.contains(
            "pub fn local<T>(self, local: &LocalState<T>) -> AppUiActionLocal<'view, 'cx, 'a, H, T>"
        ));
        assert!(!api_source.contains("pub fn local_update<A, T>("));
        assert!(!api_source.contains("pub fn local_set<A, T>("));
        assert!(!api_source.contains("pub fn toggle_local_bool<A>("));
        assert!(!api_source.contains("pub fn payload_local_update_if<A, T>("));
        assert!(api_source.contains("fn read_layout<'a, H: UiHost + 'a, Cx>("));
        assert!(api_source.contains("pub fn selector_layout<Inputs, TValue>("));
        assert!(api_source.contains("pub fn selector_model_layout<Inputs, TValue>("));
        assert!(api_source.contains("pub fn selector_model_paint<Inputs, TValue>("));
        assert!(!api_source.contains("pub fn selector_layout_keyed<K: Hash, Inputs, TValue>("));
        assert!(!api_source.contains("pub fn selector_keyed<K: Hash, Deps, TValue>("));
        assert!(
            api_source
                .contains("pub fn query_snapshot(self) -> Option<fret_query::QueryClientSnapshot>")
        );
        assert!(
            api_source.contains("pub fn query_snapshot_entry<T: Any + Send + Sync + 'static>(")
        );
        assert!(api_source.contains("pub fn cancel_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>)"));
        assert!(api_source.contains("pub fn invalidate_query<T: Any + Send + Sync + 'static>("));
        assert!(
            api_source.contains("pub fn invalidate_query_namespace(self, namespace: &'static str)")
        );
        assert!(
            api_source.contains("pub fn take_mutation_completion<TIn: 'static, TOut: 'static>(")
        );
        assert!(
            api_source
                .contains("pub fn update_after_mutation_completion<TIn: 'static, TOut: 'static>(")
        );
        assert!(api_source.contains("pub fn take_mutation_success<TIn: 'static, TOut: 'static>("));
        assert!(api_source.contains("pub fn invalidate_query_after_mutation_success<"));
        assert!(api_source.contains(
            "pub fn invalidate_query_namespace_after_mutation_success<TIn: 'static, TOut: 'static>("
        ));
        assert!(api_source.contains("pub trait AppActivateSurface"));
        assert!(api_source.contains("pub trait AppActivateExt"));
        assert!(!api_source.contains("pub trait AppActivateCxMarker"));
        assert!(!api_source.contains("AppActivateCxMarker for AppUi"));
        assert!(!api_source.contains("AppActivateCxMarker for ElementContext"));
        assert!(api_source.contains("fn action<A>(self, _action: A) -> Self"));
        assert!(
            api_source
                .contains("fn action_payload<A>(self, _action: A, payload: A::Payload) -> Self")
        );
        assert!(api_source.contains(
            "fn listen(self, f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> Self"
        ));
        assert!(!api_source.contains("pub fn action<A>(self, _action: A) -> OnActivate"));
        assert!(!api_source.contains(
            "pub fn action_payload<A>(self, _action: A, payload: A::Payload) -> OnActivate"
        ));
        assert!(!api_source.contains("fn dispatch<A>(self) -> Self"));
        assert!(!api_source.contains("fn dispatch_payload<A>(self, payload: A::Payload) -> Self"));
        assert!(!api_source.contains("pub fn dispatch<A>(self) -> OnActivate"));
        assert!(
            !api_source
                .contains("pub fn dispatch_payload<A>(self, payload: A::Payload) -> OnActivate")
        );
        assert!(api_source.contains("pub fn listen("));
        assert!(api_source.contains(
            "pub fn payload_models<A>(\n        self,\n        f: impl Fn(&mut fret_runtime::ModelStore, A::Payload) -> bool + 'static,"
        ));
        assert!(
            api_source.contains(
                "#[doc(hidden)]\npub struct AppUiActionLocal<'view, 'cx, 'a, H: UiHost, T>"
            )
        );
        assert!(api_source.contains("#[doc(hidden)]\npub struct UiCxActionLocal<'cx, 'a, T>"));
        assert!(api_source.contains("pub fn update<A>(self, update: impl Fn(&mut T) + 'static)"));
        assert!(api_source.contains("pub fn set<A>(self, value: T)"));
        assert!(api_source.contains("pub fn toggle_bool<A>(self)"));
        assert!(api_source.contains("pub fn payload_update_if<A>("));
        assert!(api_source.contains("pub fn locals_with<C>("));
        assert!(api_source.contains(
            "pub fn on<A>(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>, C) -> bool + 'static)"
        ));
        assert!(!api_source.contains(
            "pub fn locals<A>(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>) -> bool + 'static)"
        ));
        assert!(!api_source.contains("pub fn listener("));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_shadcn::facade::Button"));
        assert!(
            !api_source
                .contains("impl AppActivateSurface for fret_ui_shadcn::facade::SidebarMenuButton")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_shadcn::facade::Badge"));
        assert!(
            !api_source
                .contains("impl AppActivateSurface for fret_ui_shadcn::raw::extras::BannerAction")
        );
        assert!(
            !api_source
                .contains("impl AppActivateSurface for fret_ui_shadcn::raw::extras::BannerClose")
        );
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_shadcn::raw::extras::Ticker")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_material3::Card"));
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_material3::DialogAction")
        );
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_material3::TopAppBarAction")
        );
        assert!(api_source.contains("pub fn data(&mut self) -> AppUiData"));
        assert!(api_source.contains("pub fn effects(&mut self) -> AppUiEffects"));
        assert!(!api_source.contains("pub trait AppActionCxSurface"));
        assert!(!api_source.contains("pub trait AppActionCxExt"));
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_ai::WorkflowControlsButton")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::MessageAction"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::ArtifactClose"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::CheckpointTrigger"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::ArtifactAction"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::ConfirmationAction"));
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_ai::ConversationDownload")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::PromptInputButton"));
        assert!(
            !api_source
                .contains("impl AppActivateSurface for fret_ui_ai::WebPreviewNavigationButton")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::Attachment"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::QueueItemAction"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::Test"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::FileTreeAction"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::Suggestion"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::MessageBranch"));
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_ai::TerminalClearButton")
        );
    }

    #[test]
    fn structural_grouped_carriers_stay_hidden_from_first_contact_rustdoc() {
        let api_source = VIEW_RS_SOURCE
            .split("\nmod tests {")
            .next()
            .expect("view.rs test module marker should exist");

        assert!(api_source.contains("#[doc(hidden)]\npub struct LocalStateTxn<'a>"));
        assert!(api_source.contains("#[doc(hidden)]\npub trait LocalActionCapture"));
        assert!(
            api_source.contains(
                "#[doc(hidden)]\npub struct AppUiLocalsWith<'view, 'cx, 'a, H: UiHost, C>"
            )
        );
        assert!(api_source.contains("#[doc(hidden)]\npub struct UiCxLocalsWith<'cx, 'a, C>"));
        assert!(
            api_source.contains("#[doc(hidden)]\npub struct AppUiState<'view, 'cx, 'a, H: UiHost>")
        );
        assert!(
            api_source
                .contains("#[doc(hidden)]\npub struct AppUiActions<'view, 'cx, 'a, H: UiHost>")
        );
        assert!(api_source.contains("#[doc(hidden)]\npub struct UiCxActions<'cx, 'a>"));
        assert!(
            api_source.contains("#[doc(hidden)]\npub struct AppUiData<'view, 'cx, 'a, H: UiHost>")
        );
        assert!(api_source.contains("#[doc(hidden)]\npub struct UiCxData<'cx, 'a>"));
        assert!(
            api_source
                .contains("#[doc(hidden)]\npub struct AppUiEffects<'view, 'cx, 'a, H: UiHost>")
        );
        assert!(!api_source.contains("#[doc(hidden)]\npub struct LocalState<T>"));
        assert!(!api_source.contains("#[doc(hidden)]\npub trait TrackedStateExt<T: Any>"));
        assert!(!api_source.contains("#[doc(hidden)]\npub trait AppActivateExt"));
    }

    #[test]
    fn tracked_read_builder_stays_visible_while_structural_carriers_hide() {
        let api_source = VIEW_RS_SOURCE
            .split("\nmod tests {")
            .next()
            .expect("view.rs test module marker should exist");

        assert!(
            api_source
                .contains("#[must_use]\npub struct WatchedState<'cx, 'm, 'a, H: UiHost, T: Any>")
        );
        assert!(!api_source.contains(
            "#[doc(hidden)]\n#[must_use]\npub struct WatchedState<'cx, 'm, 'a, H: UiHost, T: Any>"
        ));
        assert!(api_source.contains(
            "Prefer `LocalState::layout_value(...)` / `paint_value(...)` for ordinary initialized app-lane"
        ));
    }

    #[test]
    fn local_state_docs_classify_default_and_bridge_surfaces() {
        let api_source = VIEW_RS_SOURCE
            .split("\nmod tests {")
            .next()
            .expect("view.rs test module marker should exist");
        assert!(api_source.contains("Default app-facing handle for view-owned local state."));
        assert!(api_source.contains("Insert a new view-owned local slot into an existing"));
        assert!(api_source.contains("Expose the underlying `Model<T>` as an explicit bridge."));
        assert!(api_source.contains("Clone the underlying `Model<T>` as an explicit bridge."));
        assert!(api_source.contains("Read this local through an explicit `ModelStore` bridge."));
        assert!(api_source.contains(
            "Query the underlying model revision through an explicit `ModelStore` bridge."
        ));
        assert!(api_source.contains(
            "Clone the current local value through an explicit `ModelStore` bridge read."
        ));
        assert!(
            api_source
                .contains("Update this local slot through an explicit `ModelStore` transaction.")
        );
        assert!(
            api_source
                .contains("Set this local slot through an explicit `ModelStore` transaction.")
        );
        assert!(api_source.contains(
            "Read the current local value through a layout invalidation tracked read on the default app"
        ));
        assert!(api_source.contains(
            "Read a derived value from this local through a layout invalidation tracked borrow on the"
        ));
        assert!(api_source.contains(
            "Read the current local value through a paint invalidation tracked read on the default app"
        ));
        assert!(api_source.contains(
            "Read a derived value from this local through a paint invalidation tracked borrow on the"
        ));
        assert!(
            api_source
                .contains("Observe/read this local from helper-heavy `ElementContext` surfaces.")
        );
        assert!(api_source.contains(
            "Read the current local value through a paint invalidation tracked read on helper-heavy"
        ));
        assert!(api_source.contains(
            "Read a derived value from this local through a paint invalidation tracked borrow on"
        ));
        assert!(api_source.contains(
            "Read the current local value through a layout invalidation tracked read on helper-heavy"
        ));
        assert!(api_source.contains(
            "Read a derived value from this local through a layout invalidation tracked borrow on"
        ));
        assert!(api_source.contains(
            "This trait is intentionally omitted from `fret::app::prelude::*` and reexported from"
        ));
        assert!(api_source.contains("`fret::advanced::prelude::*`."));
    }

    #[test]
    fn app_ui_keeps_raw_element_lane_explicit() {
        let api_source = VIEW_RS_SOURCE
            .split("\nmod tests {")
            .next()
            .expect("view.rs test module marker should exist");
        assert!(api_source.contains(
            "`AppUi` intentionally does not implement `Deref<Target = ElementContext<...>>`."
        ));
        assert!(api_source.contains("app-facing render-authoring"));
        assert!(api_source.contains("raw `ElementContext`"));
        assert!(!api_source.contains("std::ops::Deref for AppUi"));
        assert!(!api_source.contains("std::ops::DerefMut for AppUi"));
    }

    #[test]
    fn app_ui_keeps_command_gating_and_animation_frame_surface_without_deref() {
        fn assert_command_gating_impl<T: fret_ui_kit::command::ElementCommandGatingExt>() {}

        assert_command_gating_impl::<crate::AppUi<'static, 'static>>();

        let _request_animation_frame: fn(&mut crate::AppUi<'static, 'static>) =
            crate::AppUi::request_animation_frame;
        let _set_continuous_frames: fn(&mut crate::AppUi<'static, 'static>, bool) =
            crate::AppUi::set_continuous_frames;
        let _layout_query_bounds: fn(
            &mut crate::AppUi<'static, 'static>,
            fret_ui::GlobalElementId,
            fret_ui::Invalidation,
        ) -> Option<fret_core::Rect> = crate::AppUi::layout_query_bounds;
        let _layout_query_region_with_id: fn(
            &mut crate::AppUi<'static, 'static>,
            fret_ui::element::LayoutQueryRegionProps,
            for<'b> fn(
                &mut crate::AppUi<'b, 'static>,
                fret_ui::GlobalElementId,
            ) -> std::vec::Vec<fret_ui::element::AnyElement>,
        ) -> fret_ui::element::AnyElement = crate::AppUi::layout_query_region_with_id::<
            std::vec::Vec<fret_ui::element::AnyElement>,
        >;
        let _layout_query_region: fn(
            &mut crate::AppUi<'static, 'static>,
            fret_ui::element::LayoutQueryRegionProps,
            for<'b> fn(
                &mut crate::AppUi<'b, 'static>,
            ) -> std::vec::Vec<fret_ui::element::AnyElement>,
        ) -> fret_ui::element::AnyElement =
            crate::AppUi::layout_query_region::<std::vec::Vec<fret_ui::element::AnyElement>>;
    }
}
