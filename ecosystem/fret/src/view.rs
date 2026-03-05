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
use fret_runtime::Model;
use fret_ui::action::{OnCommand, OnCommandAvailability};
use fret_ui::element::Elements;
use fret_ui::{ElementContext, UiHost};
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

    /// View-local state hook backed by an app-owned model.
    ///
    /// v1: returns a `Model<T>` so action handlers can update state via `UiActionHost::models_mut`.
    #[track_caller]
    pub fn use_state<T>(&mut self) -> Model<T>
    where
        T: Any + Default,
    {
        let callsite = std::panic::Location::caller();
        let key = (callsite.file(), callsite.line(), callsite.column());

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

            // Two-phase init to avoid capturing `cx` in `with_state` init closures.
            let existing = cx.with_state(UseStateSlot::<T>::default, |slot| slot.model.clone());
            if let Some(model) = existing {
                return model;
            }

            let model = cx.app.models_mut().insert(T::default());
            cx.with_state(UseStateSlot::<T>::default, |slot| {
                if slot.model.is_none() {
                    slot.model = Some(model.clone());
                }
                slot.model.clone().expect("state slot must contain a model after init")
            })
        })
    }

    /// Keyed variant of [`use_state`], intended for loops.
    #[track_caller]
    pub fn use_state_keyed<K: Hash, T>(&mut self, key: K) -> Model<T>
    where
        T: Any + Default,
    {
        self.keyed(key, |cx| cx.use_state::<T>())
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
