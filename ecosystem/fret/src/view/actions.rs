use std::any::Any;
use std::sync::Arc;

use fret_ui::action::{ActionCx, OnActivate, UiActionHost};
use fret_ui::{ElementContext, UiHost};

use super::activation::action_listener;
use super::{
    AppUi, AppUiRawActionNotifyExt, LocalActionCapture, LocalState, LocalStateTxn,
    RenderContextAccess,
};

/// Grouped action/effect registration helpers for the default app authoring surface.
#[doc(hidden)]
pub struct AppUiActions<'view, 'cx, 'a, H: UiHost> {
    pub(super) cx: &'view mut AppUi<'cx, 'a, H>,
}

#[doc(hidden)]
pub struct AppUiActionLocal<'view, 'cx, 'a, H: UiHost, T> {
    cx: &'view mut AppUi<'cx, 'a, H>,
    local: LocalState<T>,
}

/// Grouped action/effect registration helpers for extracted app-render helpers on the default app
/// surface.
#[doc(hidden)]
pub struct AppRenderActions<'cx, 'a> {
    cx: &'cx mut ElementContext<'a, crate::app::App>,
}

#[doc(hidden)]
pub struct AppRenderActionLocal<'cx, 'a, T> {
    cx: &'cx mut ElementContext<'a, crate::app::App>,
    local: LocalState<T>,
}

#[doc(hidden)]
pub struct AppUiLocalsWith<'view, 'cx, 'a, H: UiHost, C> {
    cx: &'view mut AppUi<'cx, 'a, H>,
    captures: C,
}

#[doc(hidden)]
pub struct AppRenderLocalsWith<'cx, 'a, C> {
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

    pub fn availability<A>(
        self,
        f: impl for<'m> Fn(&mut LocalStateTxn<'m>, C) -> fret_ui::CommandAvailability + 'static,
    ) where
        A: crate::TypedAction,
    {
        let captures = self.captures;
        self.cx
            .register_action_availability_handler::<A>(move |host, _action_cx| {
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

impl<'cx, 'a, C> AppRenderLocalsWith<'cx, 'a, C>
where
    C: Clone + 'static,
{
    pub fn on<A>(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>, C) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        let captures = self.captures;
        app_render_on_action_notify::<A>(self.cx, move |host, _action_cx| {
            let mut tx = LocalStateTxn {
                models: host.models_mut(),
            };
            f(&mut tx, captures.clone())
        });
    }

    pub fn availability<A>(
        self,
        f: impl for<'m> Fn(&mut LocalStateTxn<'m>, C) -> fret_ui::CommandAvailability + 'static,
    ) where
        A: crate::TypedAction,
    {
        let captures = self.captures;
        app_render_on_action_availability::<A>(self.cx, move |host, _action_cx| {
            let mut tx = LocalStateTxn {
                models: host.models_mut(),
            };
            f(&mut tx, captures.clone())
        });
    }
}

impl<'cx, 'a, T> AppRenderActionLocal<'cx, 'a, T>
where
    T: Any,
{
    pub fn update<A>(self, update: impl Fn(&mut T) + 'static)
    where
        A: crate::TypedAction,
    {
        let local = self.local;
        app_render_on_action::<A>(self.cx, move |host, action_cx| {
            local.update_action(host, action_cx, |value| update(value))
        });
    }

    pub fn set<A>(self, value: T)
    where
        A: crate::TypedAction,
        T: Clone,
    {
        let local = self.local;
        app_render_on_action::<A>(self.cx, move |host, action_cx| {
            local.set_action(host, action_cx, value.clone())
        });
    }

    pub fn payload_update_if<A>(self, update: impl Fn(&mut T, A::Payload) -> bool + 'static)
    where
        A: crate::actions::TypedPayloadAction,
    {
        let local = self.local;
        app_render_on_payload_action::<A>(self.cx, move |host, action_cx, payload| {
            local.update_action_if(host, action_cx, |value| update(value, payload))
        });
    }
}

impl<'cx, 'a> AppRenderActionLocal<'cx, 'a, bool> {
    pub fn toggle_bool<A>(self)
    where
        A: crate::TypedAction,
    {
        let local = self.local;
        app_render_on_action::<A>(self.cx, move |host, action_cx| {
            local.update_action(host, action_cx, |value| *value = !*value)
        });
    }
}

#[derive(Default)]
struct AppRenderActionHooksFrameSlot {
    frame_id: Option<fret_runtime::FrameId>,
}

struct AppRenderActionHooksOwner;

fn prepare_app_render_action_hooks(cx: &mut ElementContext<'_, crate::app::App>) {
    let frame_id = cx.frame_id;
    let action_root = cx.root_id();
    let needs_reset = cx.root_state(AppRenderActionHooksFrameSlot::default, |slot| {
        if slot.frame_id == Some(frame_id) {
            return false;
        }
        slot.frame_id = Some(frame_id);
        true
    });
    if needs_reset {
        cx.action_clear_on_command_for_owner::<AppRenderActionHooksOwner>(action_root);
        cx.action_clear_on_command_availability_for_owner::<AppRenderActionHooksOwner>(action_root);
    }
}

fn app_render_on_action<A>(
    cx: &mut ElementContext<'_, crate::app::App>,
    f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, ActionCx) -> bool + 'static,
) where
    A: crate::TypedAction,
{
    prepare_app_render_action_hooks(cx);
    let action_root = cx.root_id();
    let action = A::action_id();
    cx.action_add_on_command_for_owner::<AppRenderActionHooksOwner>(
        action_root,
        Arc::new(move |host, action_cx, command| {
            if command != action {
                return false;
            }
            f(host, action_cx)
        }),
    );
}

fn app_render_on_action_notify<A>(
    cx: &mut ElementContext<'_, crate::app::App>,
    f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, ActionCx) -> bool + 'static,
) where
    A: crate::TypedAction,
{
    app_render_on_action::<A>(cx, move |host, action_cx| {
        let handled = f(host, action_cx);
        if handled {
            host.request_redraw(action_cx.window);
            host.notify(action_cx);
        }
        handled
    });
}

fn app_render_on_payload_action<A>(
    cx: &mut ElementContext<'_, crate::app::App>,
    f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, ActionCx, A::Payload) -> bool + 'static,
) where
    A: crate::actions::TypedPayloadAction,
{
    prepare_app_render_action_hooks(cx);
    let action_root = cx.root_id();
    let action = A::action_id();
    cx.action_add_on_command_for_owner::<AppRenderActionHooksOwner>(
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

fn app_render_on_action_availability<A>(
    cx: &mut ElementContext<'_, crate::app::App>,
    f: impl Fn(
        &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
        fret_ui::action::CommandAvailabilityActionCx,
    ) -> fret_ui::CommandAvailability
    + 'static,
) where
    A: crate::TypedAction,
{
    prepare_app_render_action_hooks(cx);
    let action_root = cx.root_id();
    let action = A::action_id();
    cx.action_add_on_command_availability_for_command_for_owner::<AppRenderActionHooksOwner>(
        action_root,
        action.clone(),
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

    /// Register a typed action that builds a mutation input from app-local state and submits it.
    ///
    /// The closure receives a `LocalStateTxn` so public app code can read/write ordinary
    /// `LocalState<T>` values without naming `ModelStore` or the lower-level mutation action host.
    #[cfg(feature = "state-mutation")]
    pub fn mutation_submit<A, TIn, TOut>(
        self,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        input: impl for<'m> Fn(&mut LocalStateTxn<'m>) -> Option<TIn> + 'static,
    ) where
        A: crate::TypedAction,
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
    {
        let handle = (*handle).clone();
        self.cx
            .register_action_handler::<A>(move |host, action_cx| {
                let input = {
                    let mut tx = LocalStateTxn {
                        models: host.models_mut(),
                    };
                    input(&mut tx)
                };
                let Some(input) = input else {
                    return false;
                };
                let changed = handle.submit(host.models_mut(), action_cx.window, input);
                if changed {
                    host.request_redraw(action_cx.window);
                    host.notify(action_cx);
                }
                changed
            });
    }

    /// Register a typed action that explicitly retries the last mutation input.
    ///
    /// `before_retry` is for small app-local projections such as setting a status note. Returning
    /// `false` does not cancel the retry; it only reports whether that projection changed state.
    #[cfg(feature = "state-mutation")]
    pub fn mutation_retry_last<A, TIn, TOut>(
        self,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        before_retry: impl for<'m> Fn(&mut LocalStateTxn<'m>) -> bool + 'static,
    ) where
        A: crate::TypedAction,
        TIn: Any + Clone + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
    {
        let handle = (*handle).clone();
        self.cx
            .register_action_handler::<A>(move |host, action_cx| {
                let projected = {
                    let mut tx = LocalStateTxn {
                        models: host.models_mut(),
                    };
                    before_retry(&mut tx)
                };
                let changed = handle.retry_last(host.models_mut(), action_cx.window) || projected;
                if changed {
                    host.request_redraw(action_cx.window);
                    host.notify(action_cx);
                }
                changed
            });
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

impl<'cx, 'a> AppRenderActions<'cx, 'a> {
    /// Build a widget-local activation listener without reopening the raw `Arc<dyn Fn...>` seam.
    pub fn listen(self, f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> OnActivate {
        action_listener(f)
    }

    /// Bind a typed action handler against one `LocalState<T>` slot without repeating that handle
    /// on every helper family.
    pub fn local<T>(self, local: &LocalState<T>) -> AppRenderActionLocal<'cx, 'a, T>
    where
        T: Any,
    {
        AppRenderActionLocal {
            cx: self.cx,
            local: LocalState::clone(local),
        }
    }

    pub fn models<A>(self, f: impl Fn(&mut fret_runtime::ModelStore) -> bool + 'static)
    where
        A: crate::TypedAction,
    {
        app_render_on_action_notify::<A>(self.cx, move |host, _action_cx| f(host.models_mut()));
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
        app_render_on_payload_action::<A>(self.cx, move |host, action_cx, payload| {
            let handled = f(host.models_mut(), payload);
            if handled {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
            handled
        });
    }

    /// Clone the provided `LocalState<T>` handles into a hidden builder so helper-heavy app-render
    /// code can pass `(&draft_state, &next_id_state, ...)` directly, then register the typed
    /// action via `.on::<A>(...)` without reopening a `LocalState::clone(...)` prelude.
    pub fn locals_with<C>(self, captures: C) -> AppRenderLocalsWith<'cx, 'a, C::Owned>
    where
        C: LocalActionCapture,
    {
        AppRenderLocalsWith {
            cx: self.cx,
            captures: captures.capture_owned(),
        }
    }

    pub fn transient<A>(self, transient_key: u64)
    where
        A: crate::TypedAction,
    {
        app_render_on_action_notify::<A>(self.cx, move |host, action_cx| {
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
        app_render_on_action_availability::<A>(self.cx, f);
    }
}

/// Brings the grouped `actions()` namespace to extracted app-render helper functions.
pub trait AppRenderActionsExt<'a> {
    /// Discover grouped action helpers through `cx.actions()` rather than naming the carrier type
    /// directly.
    fn actions(&mut self) -> AppRenderActions<'_, 'a>;
}

impl<'a, Cx> AppRenderActionsExt<'a> for Cx
where
    Cx: RenderContextAccess<'a, crate::app::App>,
{
    fn actions(&mut self) -> AppRenderActions<'_, 'a> {
        AppRenderActions {
            cx: self.elements(),
        }
    }
}
