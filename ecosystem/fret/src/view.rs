//! View authoring runtime (ecosystem-level).
//!
//! This module provides a cohesive authoring loop aligned with ADR 0308:
//! - a stateful `View` object renders into the app-facing `Ui` alias (backed by the existing
//!   declarative IR),
//! - views can register typed action handlers (action-first),
//! - hook-style helpers compose existing mechanism contracts (models + observation).
//!
//! v1 notes:
//! - the explicit raw-model hook seam (`AppUiRawStateExt::use_state<T>()`) currently returns a
//!   `Model<T>` allocated in the app-owned model store. This keeps event handlers object-safe
//!   (they only receive `UiActionHost`) while still providing view-local state ergonomics.
//! - The view runtime is intentionally additive and lives in `ecosystem/fret` (not kernel).

use std::any::Any;
use std::hash::Hash;
use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::{Model, ModelStore, ModelUpdateError};
use fret_ui::action::{ActionCx, OnActivate, OnCommand, OnCommandAvailability, UiActionHost};
use fret_ui::{ElementContext, Invalidation, UiHost};
#[cfg(feature = "state-query")]
use std::future::Future;

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
/// - use [`AppUiRawStateExt::use_state`] when code intentionally wants a raw `Model<T>` handle,
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
    /// `cx.actions().local_*` / `payload_local_update_if::<A>(...)` helpers when the local write
    /// itself should drive rerender.
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
    pub fn layout_value<'view_cx, 'a, H: UiHost>(&self, cx: &mut AppUi<'view_cx, 'a, H>) -> T
    where
        T: Any + Clone,
    {
        self.layout(cx)
            .value()
            .expect("LocalState-first app code should always read initialized locals")
    }

    /// Read the current local value through a paint invalidation tracked read on the default app
    /// surface.
    ///
    /// Keep raw `watch(...).paint().value_*` when you intentionally want the explicit builder; use
    /// this for ordinary initialized app locals that only need the paint-phase value.
    pub fn paint_value<'view_cx, 'a, H: UiHost>(&self, cx: &mut AppUi<'view_cx, 'a, H>) -> T
    where
        T: Any + Clone,
    {
        self.paint(cx)
            .value()
            .expect("LocalState-first app code should always read initialized locals")
    }

    pub fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut AppUi<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>
    where
        T: Any,
    {
        cx.watch_local(self)
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
    /// `locals::<A>(...)` transactions can read with `tx.value(&local)` instead of reopening
    /// fallback noise at every call site.
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

/// Shared read-side ergonomics for both `LocalState<T>` and explicit `Model<T>` handles.
///
/// Prefer `LocalState::layout_value(...)` / `paint_value(...)` for ordinary initialized app-lane
/// locals, or the shorter tracked-read chains such as `state.layout(cx).value_*` /
/// `state.paint(cx).value_*` when you intentionally want the explicit builder. Keep raw
/// `watch(cx)` when you need custom invalidation, `observe()`, `revision()`, or direct `read*()`
/// access on the tracked-read builder.
pub trait TrackedStateExt<T: Any> {
    fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut AppUi<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>;

    fn paint<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut AppUi<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        self.watch(cx).paint()
    }

    fn layout<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut AppUi<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        self.watch(cx).layout()
    }

    fn hit_test<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut AppUi<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        self.watch(cx).hit_test()
    }
}

impl<T: Any> TrackedStateExt<T> for LocalState<T> {
    fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut AppUi<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        cx.watch_local(self)
    }
}

impl<T: Any> TrackedStateExt<T> for Model<T> {
    fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut AppUi<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        WatchedState::new(cx.cx, self)
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

#[cfg(feature = "state-query")]
impl<T: 'static> TrackedStateExt<fret_query::QueryState<T>> for fret_query::QueryHandle<T> {
    fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut AppUi<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, fret_query::QueryState<T>> {
        WatchedState::new(cx.cx, self.model())
    }
}

/// App-facing layout-phase convenience reads for query handles on the default `fret` lane.
///
/// This intentionally collapses only the repeated `layout(...).value_or_default()` fallback for the
/// ordinary app path. Query creation (`key`, `policy`, `fetch`) and lifecycle branching
/// (`status` / `data` / `error`) stay explicit.
#[cfg(feature = "state-query")]
pub trait QueryHandleReadLayoutExt<T: 'static> {
    fn read_layout<'view_cx, 'a, H: UiHost>(
        &self,
        cx: &mut AppUi<'view_cx, 'a, H>,
    ) -> fret_query::QueryState<T>;
}

#[cfg(feature = "state-query")]
impl<T: 'static> QueryHandleReadLayoutExt<T> for fret_query::QueryHandle<T> {
    fn read_layout<'view_cx, 'a, H: UiHost>(
        &self,
        cx: &mut AppUi<'view_cx, 'a, H>,
    ) -> fret_query::QueryState<T> {
        TrackedStateExt::layout(self, cx).value_or_default()
    }
}

/// LocalState-aware selector dependency helpers for the explicit `fret::selector` lane.
///
/// This keeps `fret-selector` portable while still letting LocalState-first app code build
/// dependency signatures without bouncing through `clone_model()` or `local.model()`.
#[cfg(feature = "state-selector")]
pub(crate) trait LocalSelectorDepsBuilderExt {
    fn local_rev<T: Any>(&mut self, local: &LocalState<T>) -> &mut Self;

    fn local_rev_invalidation<T: Any>(
        &mut self,
        local: &LocalState<T>,
        invalidation: Invalidation,
    ) -> &mut Self;

    fn local_paint_rev<T: Any>(&mut self, local: &LocalState<T>) -> &mut Self {
        self.local_rev_invalidation(local, Invalidation::Paint)
    }

    fn local_layout_rev<T: Any>(&mut self, local: &LocalState<T>) -> &mut Self {
        self.local_rev_invalidation(local, Invalidation::Layout)
    }

    fn local_hit_test_rev<T: Any>(&mut self, local: &LocalState<T>) -> &mut Self {
        self.local_rev_invalidation(local, Invalidation::HitTest)
    }
}

#[cfg(feature = "state-selector")]
impl<'cx, 'a, H: UiHost> LocalSelectorDepsBuilderExt
    for fret_selector::ui::DepsBuilder<'cx, 'a, H>
{
    fn local_rev<T: Any>(&mut self, local: &LocalState<T>) -> &mut Self {
        self.local_rev_invalidation(local, Invalidation::Paint)
    }

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

/// Per-frame view construction context passed to [`View::render`].
///
/// This is a thin wrapper over [`ElementContext`] that:
/// - provides grouped default-path helpers (`state`, `actions`, `data`, `effects`),
/// - collects action handlers for installation at a chosen root element.
pub struct AppUi<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    action_root: fret_ui::GlobalElementId,
    action_handlers: crate::actions::ActionHandlerTable,
    action_handlers_used: bool,
}

/// Explicit raw-model state hooks that intentionally stay off the default app authoring surface.
///
/// This trait is intentionally omitted from `fret::app::prelude::*` and reexported from
/// `fret::advanced::prelude::*`.
///
/// Import it explicitly when advanced code still wants stable callsite-keyed `Model<T>`
/// allocation rather than the grouped `cx.state().local*` surface.
pub trait AppUiRawStateExt {
    #[track_caller]
    fn use_state<T>(&mut self) -> Model<T>
    where
        T: Any + Default;

    #[track_caller]
    fn use_state_keyed<K: Hash, T>(&mut self, key: K) -> Model<T>
    where
        T: Any + Default;
}

impl<'cx, 'a, H: UiHost> AppUiRawStateExt for AppUi<'cx, 'a, H> {
    #[track_caller]
    fn use_state<T>(&mut self) -> Model<T>
    where
        T: Any + Default,
    {
        self.use_state_with(T::default)
    }

    #[track_caller]
    fn use_state_keyed<K: Hash, T>(&mut self, key: K) -> Model<T>
    where
        T: Any + Default,
    {
        self.keyed(key, |cx| AppUiRawStateExt::use_state::<T>(cx))
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

/// Grouped action/effect registration helpers for extracted `UiCx` child builders on the default
/// app surface.
#[doc(hidden)]
pub struct UiCxActions<'cx, 'a> {
    cx: &'cx mut ElementContext<'a, crate::app::App>,
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
        cx.command_clear_on_command_for(action_root);
        cx.command_clear_on_command_availability_for(action_root);
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
    cx.command_add_on_command_for(
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
    cx.command_add_on_command_for(
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
    cx.command_add_on_command_availability_for(
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
    /// Build a widget-local activation handler using the same action-first vocabulary as widgets
    /// that already expose `.action(...)`.
    pub fn action<A>(self, _action: A) -> OnActivate
    where
        A: crate::TypedAction,
    {
        dispatch_action_listener::<A>()
    }

    /// Build a widget-local activation handler that dispatches a typed payload action while
    /// keeping the action marker on the call site.
    pub fn action_payload<A>(self, _action: A, payload: A::Payload) -> OnActivate
    where
        A: crate::actions::TypedPayloadAction,
        A::Payload: Clone,
    {
        dispatch_payload_action_listener::<A>(payload)
    }

    /// Build a widget-local activation listener without reopening the raw `Arc<dyn Fn...>` seam.
    pub fn listen(self, f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> OnActivate {
        action_listener(f)
    }

    pub fn local_update<A, T>(self, local: &LocalState<T>, update: impl Fn(&mut T) + 'static)
    where
        A: crate::TypedAction,
        T: Any,
    {
        let local = LocalState::clone(local);
        self.cx
            .register_action_handler::<A>(move |host, action_cx| {
                local.update_action(host, action_cx, |value| update(value))
            });
    }

    pub fn local_set<A, T>(self, local: &LocalState<T>, value: T)
    where
        A: crate::TypedAction,
        T: Any + Clone,
    {
        let local = LocalState::clone(local);
        self.cx
            .register_action_handler::<A>(move |host, action_cx| {
                local.set_action(host, action_cx, value.clone())
            });
    }

    pub fn toggle_local_bool<A>(self, local: &LocalState<bool>)
    where
        A: crate::TypedAction,
    {
        let local = LocalState::clone(local);
        self.cx
            .register_action_handler::<A>(move |host, action_cx| {
                local.update_action(host, action_cx, |value| *value = !*value)
            });
    }

    pub fn models<A>(self, f: impl Fn(&mut fret_runtime::ModelStore) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        self.cx
            .on_action_notify::<A>(move |host, _action_cx| f(host.models_mut()));
    }

    /// Coordinate multiple `LocalState<T>` slots through one store transaction.
    ///
    /// Inside the callback, prefer `tx.value(...)` for ordinary initialized locals, keep
    /// `tx.value_or(...)` / `tx.value_or_else(...)` for explicit fallback cases, and use
    /// `tx.set(...)`, `tx.update(...)`, and `tx.update_if(...)` for writes rather than naming the
    /// transaction carrier type directly.
    pub fn locals<A>(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        self.cx.on_action_notify::<A>(move |host, _action_cx| {
            let mut tx = LocalStateTxn {
                models: host.models_mut(),
            };
            f(&mut tx)
        });
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

    pub fn payload_local_update_if<A, T>(
        self,
        local: &LocalState<T>,
        update: impl Fn(&mut T, A::Payload) -> bool + 'static,
    ) where
        A: crate::actions::TypedPayloadAction,
        T: Any,
    {
        let local = LocalState::clone(local);
        self.cx
            .register_payload_action_handler::<A>(move |host, action_cx, payload| {
                local.update_action_if(host, action_cx, |value| update(value, payload))
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
    /// Build a widget-local activation handler using the same action-first vocabulary as widgets
    /// that already expose `.action(...)`.
    pub fn action<A>(self, _action: A) -> OnActivate
    where
        A: crate::TypedAction,
    {
        dispatch_action_listener::<A>()
    }

    /// Build a widget-local activation handler that dispatches a typed payload action while
    /// keeping the action marker on the call site.
    pub fn action_payload<A>(self, _action: A, payload: A::Payload) -> OnActivate
    where
        A: crate::actions::TypedPayloadAction,
        A::Payload: Clone,
    {
        dispatch_payload_action_listener::<A>(payload)
    }

    /// Build a widget-local activation listener without reopening the raw `Arc<dyn Fn...>` seam.
    pub fn listen(self, f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> OnActivate {
        action_listener(f)
    }

    pub fn local_update<A, T>(self, local: &LocalState<T>, update: impl Fn(&mut T) + 'static)
    where
        A: crate::TypedAction,
        T: Any,
    {
        let local = LocalState::clone(local);
        uicx_on_action::<A>(self.cx, move |host, action_cx| {
            local.update_action(host, action_cx, |value| update(value))
        });
    }

    pub fn local_set<A, T>(self, local: &LocalState<T>, value: T)
    where
        A: crate::TypedAction,
        T: Any + Clone,
    {
        let local = LocalState::clone(local);
        uicx_on_action::<A>(self.cx, move |host, action_cx| {
            local.set_action(host, action_cx, value.clone())
        });
    }

    pub fn toggle_local_bool<A>(self, local: &LocalState<bool>)
    where
        A: crate::TypedAction,
    {
        let local = LocalState::clone(local);
        uicx_on_action::<A>(self.cx, move |host, action_cx| {
            local.update_action(host, action_cx, |value| *value = !*value)
        });
    }

    pub fn models<A>(self, f: impl Fn(&mut fret_runtime::ModelStore) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        uicx_on_action_notify::<A>(self.cx, move |host, _action_cx| f(host.models_mut()));
    }

    /// Coordinate multiple `LocalState<T>` slots through one store transaction.
    ///
    /// Inside the callback, prefer `tx.value(...)` for ordinary initialized locals, keep
    /// `tx.value_or(...)` / `tx.value_or_else(...)` for explicit fallback cases, and use
    /// `tx.set(...)`, `tx.update(...)`, and `tx.update_if(...)` for writes rather than naming the
    /// transaction carrier type directly.
    pub fn locals<A>(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        uicx_on_action_notify::<A>(self.cx, move |host, _action_cx| {
            let mut tx = LocalStateTxn {
                models: host.models_mut(),
            };
            f(&mut tx)
        });
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

    pub fn payload_local_update_if<A, T>(
        self,
        local: &LocalState<T>,
        update: impl Fn(&mut T, A::Payload) -> bool + 'static,
    ) where
        A: crate::actions::TypedPayloadAction,
        T: Any,
    {
        let local = LocalState::clone(local);
        uicx_on_payload_action::<A>(self.cx, move |host, action_cx, payload| {
            local.update_action_if(host, action_cx, |value| update(value, payload))
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

    /// Keyed LocalState-first selector path for repeated callsites (lists/loops) on the default
    /// app-facing lane.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_layout_keyed<K: Hash, Inputs, TValue>(
        self,
        key: K,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: LocalSelectorLayoutInputs<'a, H>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector_keyed(
            self.cx.cx,
            key,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
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

    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_keyed<K: Hash, Deps, TValue>(
        self,
        key: K,
        deps: impl FnOnce(&mut ElementContext<'a, H>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'a, H>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector_keyed(
            self.cx.cx, key, deps, compute,
        )
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
}

/// Grouped selector/query helpers for extracted `UiCx` child builders on the default app surface.
#[doc(hidden)]
pub struct UiCxData<'cx, 'a> {
    #[allow(dead_code)]
    cx: &'cx mut ElementContext<'a, crate::app::App>,
}

impl<'cx, 'a> UiCxData<'cx, 'a> {
    /// Default LocalState-first selector path for extracted `UiCx` helpers on the app-facing lane.
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

    /// Keyed LocalState-first selector path for repeated extracted `UiCx` helper callsites.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_layout_keyed<K: Hash, Inputs, TValue>(
        self,
        key: K,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: LocalSelectorLayoutInputs<'a, crate::app::App>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector_keyed(
            self.cx,
            key,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
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

    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_keyed<K: Hash, Deps, TValue>(
        self,
        key: K,
        deps: impl FnOnce(&mut ElementContext<'a, crate::app::App>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'a, crate::app::App>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector_keyed(
            self.cx, key, deps, compute,
        )
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

    /// Grouped invalidation helper for extracted `UiCx` app-facing helpers.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>) {
        let _ = fret_query::with_query_client(self.cx.app, |client, app| {
            client.invalidate(app, key);
        });
        self.cx.app.request_redraw(self.cx.window);
    }

    /// Grouped namespace invalidation helper for extracted `UiCx` app-facing helpers.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query_namespace(self, namespace: &'static str) {
        let _ = fret_query::with_query_client(self.cx.app, |client, _app| {
            client.invalidate_namespace(namespace);
        });
        self.cx.app.request_redraw(self.cx.window);
    }
}

/// Brings the grouped `data()` namespace to extracted `UiCx` helper functions.
pub trait UiCxDataExt<'a> {
    /// Discover selector/query helpers through `cx.data()` rather than naming the carrier type
    /// directly.
    fn data(&mut self) -> UiCxData<'_, 'a>;
}

impl<'a> UiCxDataExt<'a> for ElementContext<'a, crate::app::App> {
    fn data(&mut self) -> UiCxData<'_, 'a> {
        UiCxData { cx: self }
    }
}

/// Brings the grouped `actions()` namespace to extracted `UiCx` helper functions.
pub trait UiCxActionsExt<'a> {
    /// Discover grouped action helpers through `cx.actions()` rather than naming the carrier type
    /// directly.
    fn actions(&mut self) -> UiCxActions<'_, 'a>;
}

impl<'a> UiCxActionsExt<'a> for ElementContext<'a, crate::app::App> {
    fn actions(&mut self) -> UiCxActions<'_, 'a> {
        UiCxActions { cx: self }
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

    /// Access the underlying element context.
    pub fn elements(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
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
    /// coordinated `locals::<A>(...)`, keyed payload writes, transients, and availability hooks.
    pub fn actions(&mut self) -> AppUiActions<'_, 'cx, 'a, H> {
        AppUiActions { cx: self }
    }

    /// Grouped selector/query helpers for the default app authoring surface.
    ///
    /// Discover this namespace through `cx.data()` rather than naming the returned carrier type
    /// directly. The grouped surface owns selector helpers, query creation, and query
    /// invalidation with the redraw shell included.
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
    fn use_state_with<T>(&mut self, init: impl FnOnce() -> T) -> Model<T>
    where
        T: Any,
    {
        let callsite = std::panic::Location::caller();
        let key = (callsite.file(), callsite.line(), callsite.column());
        let mut init = Some(init);

        self.cx.keyed(key, |cx| {
            struct UseStateSlot<T> {
                model: Option<Model<T>>,
                #[cfg(debug_assertions)]
                last_frame_id: u64,
                #[cfg(debug_assertions)]
                calls_in_frame: u32,
            }

            impl<T> Default for UseStateSlot<T> {
                fn default() -> Self {
                    Self {
                        model: None,
                        #[cfg(debug_assertions)]
                        last_frame_id: 0,
                        #[cfg(debug_assertions)]
                        calls_in_frame: 0,
                    }
                }
            }

            #[cfg(debug_assertions)]
            {
                let frame_id = cx.frame_id.0;
                cx.root_state(UseStateSlot::<T>::default, |slot| {
                    if slot.last_frame_id != frame_id {
                        slot.last_frame_id = frame_id;
                        slot.calls_in_frame = 0;
                    }
                    slot.calls_in_frame = slot.calls_in_frame.saturating_add(1);
                    if slot.calls_in_frame == 2 {
                        eprintln!(
                            "use_state called multiple times per frame at the same callsite ({}:{}:{}); wrap in `cx.keyed(...)` or use `use_state_keyed(...)` to avoid state collisions",
                            callsite.file(),
                            callsite.line(),
                            callsite.column()
                        );
                    }
                });
            }

            let existing = cx.root_state(UseStateSlot::<T>::default, |slot| slot.model.clone());
            if let Some(model) = existing {
                return model;
            }

            let init = init.take().expect("use_state init closure is available");
            let model = cx.app.models_mut().insert(init());
            cx.root_state(UseStateSlot::<T>::default, |slot| {
                if slot.model.is_none() {
                    slot.model = Some(model.clone());
                }
                slot.model.clone().expect("state slot must contain a model after init")
            })
        })
    }

    fn local_with<T>(&mut self, init: impl FnOnce() -> T) -> LocalState<T>
    where
        T: Any,
    {
        LocalState {
            model: self.use_state_with(init),
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

impl<'cx, 'a, H: UiHost> std::ops::Deref for AppUi<'cx, 'a, H> {
    type Target = ElementContext<'a, H>;

    fn deref(&self) -> &Self::Target {
        self.cx
    }
}

impl<'cx, 'a, H: UiHost> std::ops::DerefMut for AppUi<'cx, 'a, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.cx
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[doc(hidden)]
pub struct ViewWindowState<V: View> {
    pub view: V,
    pub(crate) cached_handlers: Option<(OnCommand, OnCommandAvailability)>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[doc(hidden)]
pub fn view_init_window<V: View>(
    app: &mut fret_app::App,
    window: AppWindowId,
) -> ViewWindowState<V> {
    ViewWindowState {
        view: V::init(app, window),
        cached_handlers: None,
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[doc(hidden)]
pub fn view_view<'a, V: View>(
    cx: &mut ElementContext<'a, fret_app::App>,
    st: &mut ViewWindowState<V>,
) -> crate::Ui {
    let action_root = cx.root_id();

    // Ensure handlers remain installed even when the view-cache root is reused (render skipped).
    if let Some((on_command, on_command_availability)) = st.cached_handlers.clone() {
        cx.command_on_command_for(action_root, on_command);
        cx.command_on_command_availability_for(action_root, on_command_availability);
    }

    let root = cx.view_cache(
        fret_ui::element::ViewCacheProps {
            contained_layout: true,
            cache_key: 0,
            ..fret_ui::element::ViewCacheProps::default()
        },
        |cx| {
            let mut app_ui = AppUi::new(cx, action_root);
            st.cached_handlers = None;
            let out = st.view.render(&mut app_ui);

            if let Some((on_command, on_command_availability)) = app_ui.take_action_handlers() {
                st.cached_handlers = Some((on_command.clone(), on_command_availability.clone()));
                cx.command_on_command_for(action_root, on_command);
                cx.command_on_command_availability_for(action_root, on_command_availability);
            }

            out
        },
    );

    root.into()
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
        AppActivateExt, AppActivateSurface, LocalState, LocalStateTxn, OnActivate, action_listener,
        dispatch_action_listener, dispatch_payload_action_listener,
    };
    use std::any::Any;
    const VIEW_RS_SOURCE: &str = include_str!("view.rs");
    use fret_core::AppWindowId;
    use fret_runtime::{ActionId, CommandId, Effect, ModelStore, TimerToken};
    use fret_ui::action::{ActionCx, ActivateReason, UiActionHost, UiFocusActionHost};

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
        assert!(api_source.contains(
            "pub fn layout_value<'view_cx, 'a, H: UiHost>(&self, cx: &mut AppUi<'view_cx, 'a, H>) -> T"
        ));
        assert!(api_source.contains(
            "pub fn paint_value<'view_cx, 'a, H: UiHost>(&self, cx: &mut AppUi<'view_cx, 'a, H>) -> T"
        ));
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
        assert!(api_source.contains("pub trait QueryHandleReadLayoutExt<T: 'static>"));
        assert!(api_source.contains("pub trait AppUiRawStateExt"));
        assert!(api_source.contains("pub trait AppUiRawActionNotifyExt"));
        assert!(api_source.contains("pub trait UiCxDataExt"));
        assert!(api_source.contains("pub trait UiCxActionsExt"));
        assert!(!api_source.contains("pub fn watch_local<'m, T: Any>("));
        assert!(api_source.contains("pub(crate) fn watch_local<'m, T: Any>("));
        assert!(!api_source.contains("pub fn action_root(&self) -> fret_ui::GlobalElementId"));
        assert!(!api_source.contains("pub fn new(cx: &'cx mut ElementContext<'a, H>, action_root: fret_ui::GlobalElementId) -> Self"));
        assert!(api_source.contains("pub(crate) fn new("));
        assert!(api_source.contains("pub fn actions(&mut self) -> AppUiActions"));
        assert!(api_source.contains("fn read_layout<'view_cx, 'a, H: UiHost>("));
        assert!(api_source.contains("pub fn selector_layout<Inputs, TValue>("));
        assert!(api_source.contains("pub fn selector_layout_keyed<K: Hash, Inputs, TValue>("));
        assert!(api_source.contains("pub fn invalidate_query<T: Any + Send + Sync + 'static>("));
        assert!(
            api_source.contains("pub fn invalidate_query_namespace(self, namespace: &'static str)")
        );
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
        assert!(api_source.contains("pub fn action<A>(self, _action: A) -> OnActivate"));
        assert!(api_source.contains(
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
        assert!(api_source.contains("Expose the underlying `Model<T>` as an explicit bridge."));
        assert!(api_source.contains("Clone the underlying `Model<T>` as an explicit bridge."));
        assert!(api_source.contains("Read this local through an explicit `ModelStore` bridge."));
        assert!(api_source.contains(
            "Read the current local value through a layout invalidation tracked read on the default app"
        ));
        assert!(api_source.contains(
            "Read the current local value through a paint invalidation tracked read on the default app"
        ));
        assert!(
            api_source
                .contains("Observe/read this local from helper-heavy `ElementContext` surfaces.")
        );
        assert!(api_source.contains(
            "This trait is intentionally omitted from `fret::app::prelude::*` and reexported from"
        ));
        assert!(api_source.contains("`fret::advanced::prelude::*`."));
    }
}
