use std::any::Any;

use fret_runtime::Model;
use fret_ui::UiHost;

use super::AppUi;

#[doc(hidden)]
pub trait AppUiComponentLaneRequiresExplicitElementsEscapeHatch {}

/// Explicit raw-model state hooks that intentionally stay off the default app authoring surface.
///
/// This trait is intentionally omitted from `fret::app::prelude::*` and reexported from
/// `fret::advanced::raw`.
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
/// `fret::advanced::raw`.
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
