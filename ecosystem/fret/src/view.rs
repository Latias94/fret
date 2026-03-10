//! View authoring runtime (ecosystem-level).
//!
//! This module provides a cohesive authoring loop aligned with ADR 0308:
//! - a stateful `View` object renders into the existing declarative IR (`Elements`),
//! - views can register typed action handlers (action-first),
//! - hook-style helpers compose existing mechanism contracts (models + observation).
//!
//! v1 notes:
//! - `use_state<T>()` currently returns a `Model<T>` allocated in the app-owned model store.
//!   This keeps event handlers object-safe (they only receive `UiActionHost`) while still
//!   providing view-local state ergonomics.
//! - The view runtime is intentionally additive and lives in `ecosystem/fret` (not kernel).

use std::any::Any;
use std::hash::Hash;

use fret_app::App;
use fret_core::AppWindowId;
use fret_runtime::{Model, ModelStore, ModelUpdateError};
use fret_ui::action::{OnCommand, OnCommandAvailability};
use fret_ui::element::Elements;
use fret_ui::{ElementContext, Invalidation, UiHost};
#[cfg(feature = "state-query")]
use std::future::Future;

/// A stateful view object that renders into the existing declarative IR (`Elements`).
pub trait View: 'static {
    /// Initialize the view for a specific window.
    fn init(app: &mut App, window: AppWindowId) -> Self
    where
        Self: Sized;

    /// Render the view into declarative elements.
    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements;
}

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
    pub fn model(&self) -> &Model<T> {
        &self.model
    }

    pub fn clone_model(&self) -> Model<T> {
        self.model.clone()
    }

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

    pub fn revision_in(&self, models: &ModelStore) -> Option<u64>
    where
        T: Any,
    {
        models.revision(&self.model)
    }

    /// Clone the current local value through an explicit `ModelStore` read.
    ///
    /// This mirrors the render-time `watch(...).value_*` helpers for multi-state transactions that
    /// still need to read from `ModelStore` inside `on_action_notify_models::<A>(...)`.
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
    /// view-cache root dirty by itself. Use it inside `on_action_notify_models::<A>(...)` when the
    /// write participates in a broader model-store transaction, or use `update_action(...)` /
    /// `ViewCx::on_action_notify_local_*` when the local write itself should drive rerender.
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
    /// responsibility of the surrounding authoring surface unless you use the action-aware helpers.
    pub fn set_in(&self, models: &mut ModelStore, value: T) -> bool
    where
        T: Any,
    {
        self.update_in(models, move |slot| *slot = value)
    }

    /// Update this local slot from an action dispatch and participate in the tracked-write
    /// rerender rule (`request_redraw(window)` + `notify(action_cx)`) when the write succeeds.
    pub fn update_action(
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
    pub fn update_action_if(
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
    pub fn set_action(
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

    pub fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut ViewCx<'view_cx, 'a, H>,
    ) -> WatchedLocal<'watch, 'watch, 'a, H, T>
    where
        T: Any,
    {
        cx.watch_local(self)
    }
}

/// A narrow, LocalState-focused transaction wrapper used to keep the default authoring surface
/// free of direct `ModelStore` plumbing.
///
/// This is intentionally *not* a general-purpose model transaction API. If you need to coordinate
/// across shared `Model<T>` graphs, use `ViewCx::on_action_notify_models` directly.
pub struct LocalTxn<'a> {
    models: &'a mut ModelStore,
}

impl<'a> LocalTxn<'a> {
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

#[must_use]
pub struct WatchedState<'cx, 'm, 'a, H: UiHost, T: Any> {
    cx: &'cx mut ElementContext<'a, H>,
    model: &'m Model<T>,
    invalidation: Invalidation,
}

pub type WatchedLocal<'cx, 'm, 'a, H, T> = WatchedState<'cx, 'm, 'a, H, T>;
pub type WatchedModel<'cx, 'm, 'a, H, T> = WatchedState<'cx, 'm, 'a, H, T>;

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
pub trait TrackedStateExt<T: Any> {
    fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut ViewCx<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T>;

    fn paint<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut ViewCx<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        self.watch(cx).paint()
    }

    fn layout<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut ViewCx<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        self.watch(cx).layout()
    }

    fn hit_test<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut ViewCx<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        self.watch(cx).hit_test()
    }
}

impl<T: Any> TrackedStateExt<T> for LocalState<T> {
    fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut ViewCx<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        cx.watch_local(self)
    }
}

impl<T: Any> TrackedStateExt<T> for Model<T> {
    fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut ViewCx<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, T> {
        WatchedState::new(cx.cx, self)
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::IntoTextValueModel for LocalState<String> {
    fn into_text_value_model(self) -> Model<String> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::IntoTextValueModel for &LocalState<String> {
    fn into_text_value_model(self) -> Model<String> {
        self.clone_model()
    }
}

#[cfg(feature = "state-query")]
impl<T: 'static> TrackedStateExt<fret_query::QueryState<T>> for fret_query::QueryHandle<T> {
    fn watch<'watch, 'view_cx, 'a, H: UiHost>(
        &'watch self,
        cx: &'watch mut ViewCx<'view_cx, 'a, H>,
    ) -> WatchedState<'watch, 'watch, 'a, H, fret_query::QueryState<T>> {
        WatchedState::new(cx.cx, self.model())
    }
}

/// Per-frame view construction context passed to [`View::render`].
///
/// This is a thin wrapper over [`ElementContext`] that:
/// - provides hook-style helpers (`use_state`, keyed),
/// - collects action handlers for installation at a chosen root element.
pub struct ViewCx<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    action_root: fret_ui::GlobalElementId,
    action_handlers: crate::actions::ActionHandlerTable,
    action_handlers_used: bool,
}

/// Grouped LocalState-first helpers for the default app authoring surface.
pub struct AppUiState<'view, 'cx, 'a, H: UiHost> {
    cx: &'view mut ViewCx<'cx, 'a, H>,
}

impl<'view, 'cx, 'a, H: UiHost> AppUiState<'view, 'cx, 'a, H> {
    #[track_caller]
    pub fn local<T>(self) -> LocalState<T>
    where
        T: Any + Default,
    {
        self.cx.use_local::<T>()
    }

    #[track_caller]
    pub fn local_keyed<K: Hash, T>(self, key: K) -> LocalState<T>
    where
        T: Any + Default,
    {
        self.cx.use_local_keyed::<K, T>(key)
    }

    #[track_caller]
    pub fn local_init<T>(self, init: impl FnOnce() -> T) -> LocalState<T>
    where
        T: Any,
    {
        self.cx.use_local_with(init)
    }

    pub fn watch<T: Any>(
        self,
        local: &'view LocalState<T>,
    ) -> WatchedLocal<'view, 'view, 'a, H, T> {
        self.cx.watch_local(local)
    }
}

/// Grouped action/effect registration helpers for the default app authoring surface.
pub struct AppUiActions<'view, 'cx, 'a, H: UiHost> {
    cx: &'view mut ViewCx<'cx, 'a, H>,
}

impl<'view, 'cx, 'a, H: UiHost> AppUiActions<'view, 'cx, 'a, H> {
    pub fn local_update<A, T>(self, local: &LocalState<T>, update: impl Fn(&mut T) + 'static)
    where
        A: crate::TypedAction,
        T: Any,
    {
        self.cx.on_action_notify_local_update::<A, T>(local, update);
    }

    pub fn local_set<A, T>(self, local: &LocalState<T>, value: T)
    where
        A: crate::TypedAction,
        T: Any + Clone,
    {
        self.cx.on_action_notify_local_set::<A, T>(local, value);
    }

    pub fn toggle_local_bool<A>(self, local: &LocalState<bool>)
    where
        A: crate::TypedAction,
    {
        self.cx.on_action_notify_toggle_local_bool::<A>(local);
    }

    pub fn models<A>(self, f: impl Fn(&mut fret_runtime::ModelStore) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        self.cx.on_action_notify_models::<A>(f);
    }

    pub fn locals<A>(self, f: impl for<'m> Fn(&mut LocalTxn<'m>) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        self.cx.on_action_notify_locals::<A>(f);
    }

    pub fn transient<A>(self, transient_key: u64)
    where
        A: crate::TypedAction,
    {
        self.cx.on_action_notify_transient::<A>(transient_key);
    }

    pub fn payload_local_update_if<A, T>(
        self,
        local: &LocalState<T>,
        update: impl Fn(&mut T, A::Payload) -> bool + 'static,
    ) where
        A: crate::actions::TypedPayloadAction,
        T: Any,
    {
        self.cx
            .on_payload_action_notify_local_update_if::<A, T>(local, update);
    }

    pub fn payload_locals<A>(
        self,
        f: impl for<'m> Fn(&mut LocalTxn<'m>, A::Payload) -> bool + 'static,
    ) where
        A: crate::actions::TypedPayloadAction,
    {
        self.cx.on_payload_action_notify_locals::<A>(f);
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
        self.cx.on_action_availability::<A>(f);
    }
}

/// Grouped selector/query helpers for the default app authoring surface.
pub struct AppUiData<'view, 'cx, 'a, H: UiHost> {
    #[allow(dead_code)]
    cx: &'view mut ViewCx<'cx, 'a, H>,
}

impl<'view, 'cx, 'a, H: UiHost> AppUiData<'view, 'cx, 'a, H> {
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
        self.cx.use_selector(deps, compute)
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
        self.cx.use_selector_keyed(key, deps, compute)
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
        self.cx.use_query(key, policy, fetch)
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
        self.cx.use_query_async(key, policy, fetch)
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
        self.cx.use_query_async_local(key, policy, fetch)
    }
}

/// Grouped render-time effect helpers for the default app authoring surface.
pub struct AppUiEffects<'view, 'cx, 'a, H: UiHost> {
    cx: &'view mut ViewCx<'cx, 'a, H>,
}

impl<'view, 'cx, 'a, H: UiHost> AppUiEffects<'view, 'cx, 'a, H> {
    pub fn take_transient(self, key: u64) -> bool {
        self.cx.take_transient_on_action_root(key)
    }
}

impl<'cx, 'a, H: UiHost> ViewCx<'cx, 'a, H> {
    pub fn new(cx: &'cx mut ElementContext<'a, H>, action_root: fret_ui::GlobalElementId) -> Self {
        Self {
            cx,
            action_root,
            action_handlers: crate::actions::ActionHandlerTable::new(),
            action_handlers_used: false,
        }
    }

    /// The element that owns view-level action handlers for this render pass.
    pub fn action_root(&self) -> fret_ui::GlobalElementId {
        self.action_root
    }

    /// Consume a transient event recorded for this view's action root.
    ///
    /// This is a lightweight scheduling primitive intended for “app effects” that need to run in
    /// the next render pass (e.g. query invalidation) without allocating a dedicated model.
    pub fn take_transient_on_action_root(&mut self, key: u64) -> bool {
        self.cx.take_transient_for(self.action_root, key)
    }

    /// Access the underlying element context.
    pub fn elements(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
    }

    /// Grouped state/local helpers for the default app authoring surface.
    pub fn state(&mut self) -> AppUiState<'_, 'cx, 'a, H> {
        AppUiState { cx: self }
    }

    /// Grouped typed action registration helpers for the default app authoring surface.
    pub fn actions(&mut self) -> AppUiActions<'_, 'cx, 'a, H> {
        AppUiActions { cx: self }
    }

    /// Grouped selector/query helpers for the default app authoring surface.
    pub fn data(&mut self) -> AppUiData<'_, 'cx, 'a, H> {
        AppUiData { cx: self }
    }

    /// Grouped render-time effect helpers for the default app authoring surface.
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
        f: impl for<'b> FnOnce(&mut ViewCx<'b, 'a, H>) -> R,
    ) -> R {
        let action_root = self.action_root;
        let action_handlers = std::mem::take(&mut self.action_handlers);
        let action_handlers_used = self.action_handlers_used;

        let (out, action_handlers, action_handlers_used) = self.cx.keyed(key, |cx| {
            let mut nested = ViewCx {
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
                cx.with_state(UseStateSlot::<T>::default, |slot| {
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

            let existing = cx.with_state(UseStateSlot::<T>::default, |slot| slot.model.clone());
            if let Some(model) = existing {
                return model;
            }

            let init = init.take().expect("use_state init closure is available");
            let model = cx.app.models_mut().insert(init());
            cx.with_state(UseStateSlot::<T>::default, |slot| {
                if slot.model.is_none() {
                    slot.model = Some(model.clone());
                }
                slot.model.clone().expect("state slot must contain a model after init")
            })
        })
    }

    /// View-local state hook backed by an app-owned model.
    ///
    /// v1: returns a `Model<T>` so action handlers can update state via `UiActionHost::models_mut`.
    #[track_caller]
    pub fn use_state<T>(&mut self) -> Model<T>
    where
        T: Any + Default,
    {
        self.use_state_with(T::default)
    }

    /// Keyed variant of [`use_state`], intended for loops.
    #[track_caller]
    pub fn use_state_keyed<K: Hash, T>(&mut self, key: K) -> Model<T>
    where
        T: Any + Default,
    {
        self.keyed(key, |cx| cx.use_state::<T>())
    }

    /// v2 prototype: a model-backed local-state wrapper with a lower-noise authoring surface.
    #[track_caller]
    pub fn use_local<T>(&mut self) -> LocalState<T>
    where
        T: Any + Default,
    {
        self.use_local_with(T::default)
    }

    /// v2 prototype: keyed local-state variant intended for loops.
    #[track_caller]
    pub fn use_local_keyed<K: Hash, T>(&mut self, key: K) -> LocalState<T>
    where
        T: Any + Default,
    {
        self.keyed(key, |cx| cx.use_local::<T>())
    }

    /// v2 prototype: local state with explicit initializer.
    #[track_caller]
    pub fn use_local_with<T>(&mut self, init: impl FnOnce() -> T) -> LocalState<T>
    where
        T: Any,
    {
        LocalState {
            model: self.use_state_with(init),
        }
    }

    /// Observe and read a model-backed local state handle.
    pub fn watch_local<'m, T: Any>(
        &'m mut self,
        local: &'m LocalState<T>,
    ) -> WatchedLocal<'m, 'm, 'a, H, T> {
        WatchedState::new(self.cx, local.model())
    }

    /// Derived state hook backed by `ecosystem/fret-selector` (UI feature).
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn use_selector<Deps, TValue>(
        &mut self,
        deps: impl FnOnce(&mut ElementContext<'a, H>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'a, H>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(self.cx, deps, compute)
    }

    /// Keyed derived state hook backed by `ecosystem/fret-selector` (UI feature).
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn use_selector_keyed<K: Hash, Deps, TValue>(
        &mut self,
        key: K,
        deps: impl FnOnce(&mut ElementContext<'a, H>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'a, H>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector_keyed(
            self.cx, key, deps, compute,
        )
    }

    /// Async query hook backed by `ecosystem/fret-query` (UI feature).
    #[cfg(feature = "state-query")]
    pub fn use_query<T: Any + Send + Sync + 'static>(
        &mut self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Result<T, fret_query::QueryError>
        + Send
        + 'static,
    ) -> fret_query::QueryHandle<T> {
        fret_query::ui::QueryElementContextExt::use_query(self.cx, key, policy, fetch)
    }

    /// Async query hook (async fetch) backed by `ecosystem/fret-query` (UI feature).
    #[cfg(feature = "state-query")]
    pub fn use_query_async<T, Fut>(
        &mut self,
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

    /// Async query hook (non-Send async fetch) backed by `ecosystem/fret-query` (UI feature).
    #[cfg(feature = "state-query")]
    pub fn use_query_async_local<T, Fut>(
        &mut self,
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

    /// Register a typed unit action handler (v1: adapter over `OnCommand`).
    pub fn on_action<A: crate::TypedAction>(
        &mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
    ) {
        self.action_handlers_used = true;
        let next = std::mem::take(&mut self.action_handlers).on::<A>(f);
        self.action_handlers = next;
    }

    /// Register a typed unit action handler that requests redraw + notifies on `handled=true`.
    ///
    /// This is a small ergonomics helper: most action handlers that mutate models/state need both
    /// `request_redraw(window)` and `notify(action_cx)` to participate in the view-cache closure.
    pub fn on_action_notify<A: crate::TypedAction>(
        &mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
    ) {
        self.on_action::<A>(move |host, action_cx| {
            let handled = f(host, action_cx);
            if handled {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
            handled
        });
    }

    /// Register a typed unit action handler that updates a model and participates in the view-cache
    /// closure (`request_redraw` + `notify`) when the update succeeds.
    ///
    /// This is the recommended helper for simple state mutations (counter increments, toggles,
    /// flags, and small structs).
    pub fn on_action_notify_model_update<A, T>(
        &mut self,
        model: Model<T>,
        update: impl Fn(&mut T) + 'static,
    ) where
        A: crate::TypedAction,
        T: Any,
    {
        self.on_action_notify::<A>(move |host, _action_cx| {
            host.models_mut().update(&model, |v| update(v)).is_ok()
        });
    }

    /// Register a typed unit action handler that sets a model to a fixed value and participates in
    /// the view-cache closure (`request_redraw` + `notify`) when the update succeeds.
    pub fn on_action_notify_model_set<A, T>(&mut self, model: Model<T>, value: T)
    where
        A: crate::TypedAction,
        T: Any + Clone,
    {
        self.on_action_notify_model_update::<A, T>(model, move |v| *v = value.clone());
    }

    /// Convenience helper: register a typed unit action handler that toggles a `Model<bool>`.
    pub fn on_action_notify_toggle_bool<A: crate::TypedAction>(&mut self, model: Model<bool>) {
        self.on_action_notify_model_update::<A, bool>(model, |v| *v = !*v);
    }

    /// Register a typed unit action handler that updates a `LocalState<T>` and participates in the
    /// view-cache closure (`request_redraw` + `notify`) when the update succeeds.
    pub fn on_action_notify_local_update<A, T>(
        &mut self,
        local: &LocalState<T>,
        update: impl Fn(&mut T) + 'static,
    ) where
        A: crate::TypedAction,
        T: Any,
    {
        let local = LocalState::clone(local);
        self.on_action::<A>(move |host, action_cx| {
            local.update_action(host, action_cx, |value| update(value))
        });
    }

    /// Register a typed unit action handler that sets a `LocalState<T>` to a fixed value and
    /// participates in the view-cache closure (`request_redraw` + `notify`) when the write succeeds.
    pub fn on_action_notify_local_set<A, T>(&mut self, local: &LocalState<T>, value: T)
    where
        A: crate::TypedAction,
        T: Any + Clone,
    {
        let local = LocalState::clone(local);
        self.on_action::<A>(move |host, action_cx| {
            local.set_action(host, action_cx, value.clone())
        });
    }

    /// Convenience helper: register a typed unit action handler that toggles a `LocalState<bool>`.
    pub fn on_action_notify_toggle_local_bool<A: crate::TypedAction>(
        &mut self,
        local: &LocalState<bool>,
    ) {
        let local = LocalState::clone(local);
        self.on_action::<A>(move |host, action_cx| {
            local.update_action(host, action_cx, |value| *value = !*value)
        });
    }

    /// Register a typed unit action handler that runs a model-store transaction and participates
    /// in the view-cache closure (`request_redraw` + `notify`) when `handled=true`.
    ///
    /// This helper is intended for common cases that touch multiple models (e.g. read a draft
    /// string, push to a list, clear the draft, bump an id counter).
    pub fn on_action_notify_models<A: crate::TypedAction>(
        &mut self,
        f: impl Fn(&mut fret_runtime::ModelStore) -> bool + 'static,
    ) {
        self.on_action_notify::<A>(move |host, _action_cx| f(host.models_mut()))
    }

    /// Register a typed unit action handler that runs a LocalState-focused transaction and
    /// participates in the view-cache closure (`request_redraw` + `notify`) when `handled=true`.
    ///
    /// This keeps the default authoring surface free of direct `ModelStore` references for the
    /// common case where the transaction only touches view-owned `LocalState<T>` slots.
    pub fn on_action_notify_locals<A: crate::TypedAction>(
        &mut self,
        f: impl for<'m> Fn(&mut LocalTxn<'m>) -> bool + 'static,
    ) {
        self.on_action_notify_models::<A>(move |models| {
            let mut tx = LocalTxn { models };
            f(&mut tx)
        })
    }

    /// Register a typed action handler that records a transient event for this dispatch cycle.
    ///
    /// This is a convenience wrapper over `UiActionHost::record_transient_event`, commonly used
    /// to schedule “app effects” that must be applied in `render()` (because the handler only
    /// receives a restricted `UiActionHost`).
    pub fn on_action_notify_transient<A: crate::TypedAction>(&mut self, transient_key: u64) {
        self.on_action_notify::<A>(move |host, action_cx| {
            host.record_transient_event(action_cx, transient_key);
            true
        });
    }

    /// Register a typed payload action handler (v2 prototype; ADR 0312).
    ///
    /// Notes:
    /// - Payload is pointer/programmatic-only in v2 (keymap/palette/menus remain unit actions).
    /// - Missing/invalid payload should be treated as “not handled” by default.
    pub fn on_payload_action<A: crate::actions::TypedPayloadAction>(
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

    /// Register a typed payload action handler that requests redraw + notifies on `handled=true`.
    pub fn on_payload_action_notify<A: crate::actions::TypedPayloadAction>(
        &mut self,
        f: impl Fn(
            &mut dyn fret_ui::action::UiFocusActionHost,
            fret_ui::action::ActionCx,
            A::Payload,
        ) -> bool
        + 'static,
    ) {
        self.on_payload_action::<A>(move |host, action_cx, payload| {
            let handled = f(host, action_cx, payload);
            if handled {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
            handled
        });
    }

    /// Register a typed payload action handler that mutates `LocalState<T>` and participates in
    /// the tracked-write rerender rule when the closure returns `handled=true`.
    ///
    /// This is intentionally narrow: it exists for keyed-list / payload-row style mutations where
    /// the remaining visible boilerplate is mostly `host.models_mut()` + `LocalState` cloning at
    /// the root action table.
    pub fn on_payload_action_notify_local_update_if<A, T>(
        &mut self,
        local: &LocalState<T>,
        update: impl Fn(&mut T, A::Payload) -> bool + 'static,
    ) where
        A: crate::actions::TypedPayloadAction,
        T: Any,
    {
        let local = LocalState::clone(local);
        self.on_payload_action::<A>(move |host, action_cx, payload| {
            local.update_action_if(host, action_cx, |value| update(value, payload))
        });
    }

    /// Register a typed payload action handler that runs a LocalState-focused transaction and
    /// participates in the view-cache closure (`request_redraw` + `notify`) when `handled=true`.
    ///
    /// This keeps the default authoring surface free of direct `ModelStore` references for the
    /// common case where the handler only coordinates view-owned `LocalState<T>` slots.
    pub fn on_payload_action_notify_locals<A: crate::actions::TypedPayloadAction>(
        &mut self,
        f: impl for<'m> Fn(&mut LocalTxn<'m>, A::Payload) -> bool + 'static,
    ) {
        self.on_payload_action_notify::<A>(move |host, _action_cx, payload| {
            let mut tx = LocalTxn {
                models: host.models_mut(),
            };
            f(&mut tx, payload)
        })
    }

    /// Register a typed unit action availability handler.
    pub fn on_action_availability<A: crate::TypedAction>(
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

    pub(crate) fn take_action_handlers(self) -> Option<(OnCommand, OnCommandAvailability)> {
        if !self.action_handlers_used {
            return None;
        }
        Some(self.action_handlers.build())
    }
}

impl<'cx, 'a, H: UiHost> std::ops::Deref for ViewCx<'cx, 'a, H> {
    type Target = ElementContext<'a, H>;

    fn deref(&self) -> &Self::Target {
        self.cx
    }
}

impl<'cx, 'a, H: UiHost> std::ops::DerefMut for ViewCx<'cx, 'a, H> {
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
pub fn view_init_window<V: View>(app: &mut App, window: AppWindowId) -> ViewWindowState<V> {
    ViewWindowState {
        view: V::init(app, window),
        cached_handlers: None,
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[doc(hidden)]
pub fn view_view<'a, V: View>(
    cx: &mut ElementContext<'a, App>,
    st: &mut ViewWindowState<V>,
) -> Elements {
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
            let mut vcx = ViewCx::new(cx, action_root);
            st.cached_handlers = None;
            let out = st.view.render(&mut vcx);

            if let Some((on_command, on_command_availability)) = vcx.take_action_handlers() {
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
    _app: &mut App,
    _window: AppWindowId,
    ui: &mut fret_ui::UiTree<App>,
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
    use super::LocalState;
    use fret_core::AppWindowId;
    use fret_runtime::{Effect, ModelStore, TimerToken};
    use fret_ui::action::{ActionCx, UiActionHost, UiFocusActionHost};

    #[derive(Default)]
    struct FakeHost {
        models: ModelStore,
        redraws: Vec<AppWindowId>,
        notifies: Vec<ActionCx>,
        next_timer: u64,
    }

    impl UiActionHost for FakeHost {
        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }

        fn push_effect(&mut self, _effect: Effect) {}

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

    #[cfg(feature = "shadcn")]
    #[test]
    fn local_state_supports_text_value_widgets() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(String::from("hello")),
        };

        let _input = fret_ui_shadcn::Input::new(&local);
        let _textarea = fret_ui_shadcn::Textarea::new(&local);
    }
}
