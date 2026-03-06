use std::sync::Arc;

use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};

/// Build an `OnActivate` handler from a closure.
#[inline]
pub fn on_activate(
    f: impl Fn(&mut dyn UiActionHost, ActionCx, ActivateReason) + 'static,
) -> OnActivate {
    Arc::new(f)
}

/// Build an `OnActivate` handler that requests a redraw after the mutation.
#[inline]
pub fn on_activate_request_redraw(f: impl Fn(&mut dyn UiActionHost) + 'static) -> OnActivate {
    Arc::new(move |host, action_cx, _reason| {
        f(host);
        host.request_redraw(action_cx.window);
    })
}

/// Build an `OnActivate` handler that notifies after the mutation.
#[inline]
pub fn on_activate_notify(f: impl Fn(&mut dyn UiActionHost) + 'static) -> OnActivate {
    Arc::new(move |host, action_cx, _reason| {
        f(host);
        host.notify(action_cx);
    })
}

/// Build an `OnActivate` handler that requests a redraw and notifies after the mutation.
#[inline]
pub fn on_activate_request_redraw_notify(
    f: impl Fn(&mut dyn UiActionHost) + 'static,
) -> OnActivate {
    Arc::new(move |host, action_cx, _reason| {
        f(host);
        host.request_redraw(action_cx.window);
        host.notify(action_cx);
    })
}
