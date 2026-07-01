use std::sync::Arc;

use fret_ui::action::{ActionCx, OnActivate, UiActionHost};

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

pub(super) fn dispatch_action_listener<A>() -> OnActivate
where
    A: crate::TypedAction,
{
    let action = A::action_id();
    Arc::new(move |host, action_cx, reason| {
        host.record_pending_command_dispatch_source(action_cx, &action, reason);
        host.dispatch_command(Some(action_cx.window), action.clone());
    })
}

pub(super) fn dispatch_payload_action_listener<A>(payload: A::Payload) -> OnActivate
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

pub(super) fn action_listener(f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> OnActivate {
    Arc::new(move |host, action_cx, _reason| f(host, action_cx))
}
