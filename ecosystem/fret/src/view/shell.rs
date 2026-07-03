use std::any::Any;
use std::hash::Hash;

use fret_core::AppWindowId;
use fret_runtime::Model;
use fret_ui::action::{OnCommand, OnCommandAvailability};
use fret_ui::{ElementContext, UiHost};

use super::{
    AppUiActions, AppUiData, AppUiEffects, AppUiState, LocalState, LocalStateRawModelExt as _,
    WatchedState,
};

/// Per-frame view construction context passed to [`super::View::render`].
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
    pub(super) cx: &'cx mut ElementContext<'a, H>,
    pub(super) action_root: fret_ui::GlobalElementId,
    pub(super) action_handlers: crate::actions::ActionHandlerTable,
    pub(super) action_handlers_used: bool,
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

    pub(super) fn register_action_handler<A: crate::TypedAction>(
        &mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
    ) {
        self.action_handlers_used = true;
        let next = std::mem::take(&mut self.action_handlers).on::<A>(f);
        self.action_handlers = next;
    }

    pub(super) fn register_payload_action_handler<A: crate::actions::TypedPayloadAction>(
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

    pub(super) fn register_action_availability_handler<A: crate::TypedAction>(
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
    pub(super) fn raw_model_with<T>(&mut self, init: impl FnOnce() -> T) -> Model<T>
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
    pub(super) fn local_with<T>(&mut self, init: impl FnOnce() -> T) -> LocalState<T>
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
