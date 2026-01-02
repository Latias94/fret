use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_timer_event<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    event: &Event,
) -> bool {
    let Event::Timer { token } = event else {
        return false;
    };

    let hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::TimerActionHooks::default,
        |hooks| hooks.on_timer.clone(),
    );

    if let Some(h) = hook {
        struct TimerHookHost<'a, H: UiHost> {
            app: &'a mut H,
            window: AppWindowId,
            requested_focus: &'a mut Option<NodeId>,
        }

        impl<H: UiHost> action::UiActionHost for TimerHookHost<'_, H> {
            fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                self.app.models_mut()
            }

            fn push_effect(&mut self, effect: Effect) {
                self.app.push_effect(effect);
            }

            fn request_redraw(&mut self, window: AppWindowId) {
                self.app.request_redraw(window);
            }

            fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
                self.app.next_timer_token()
            }
        }

        impl<H: UiHost> action::UiFocusActionHost for TimerHookHost<'_, H> {
            fn request_focus(&mut self, target: crate::GlobalElementId) {
                let Some(node) = crate::elements::with_window_state(
                    &mut *self.app,
                    self.window,
                    |window_state| window_state.node_entry(target).map(|e| e.node),
                ) else {
                    return;
                };
                *self.requested_focus = Some(node);
            }
        }

        let mut host = TimerHookHost {
            app: &mut *cx.app,
            window,
            requested_focus: &mut cx.requested_focus,
        };
        let handled = h(
            &mut host,
            action::ActionCx {
                window,
                target: this.element,
            },
            *token,
        );
        if handled {
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
            return true;
        }
    }

    false
}

pub(super) fn try_key_hook<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
    repeat: bool,
) -> bool {
    let hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::KeyActionHooks::default,
        |hooks| hooks.on_key_down.clone(),
    );

    if let Some(h) = hook {
        struct KeyHookHost<'a, H: UiHost> {
            app: &'a mut H,
            window: AppWindowId,
            requested_focus: &'a mut Option<NodeId>,
        }

        impl<H: UiHost> action::UiActionHost for KeyHookHost<'_, H> {
            fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                self.app.models_mut()
            }

            fn push_effect(&mut self, effect: Effect) {
                self.app.push_effect(effect);
            }

            fn request_redraw(&mut self, window: AppWindowId) {
                self.app.request_redraw(window);
            }

            fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
                self.app.next_timer_token()
            }
        }

        impl<H: UiHost> action::UiFocusActionHost for KeyHookHost<'_, H> {
            fn request_focus(&mut self, target: crate::GlobalElementId) {
                let Some(node) = crate::elements::with_window_state(
                    &mut *self.app,
                    self.window,
                    |window_state| window_state.node_entry(target).map(|e| e.node),
                ) else {
                    return;
                };
                *self.requested_focus = Some(node);
            }
        }

        let mut host = KeyHookHost {
            app: &mut *cx.app,
            window,
            requested_focus: &mut cx.requested_focus,
        };
        let handled = h(
            &mut host,
            action::ActionCx {
                window,
                target: this.element,
            },
            KeyDownCx {
                key,
                modifiers,
                repeat,
            },
        );
        if handled {
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
            return true;
        }
    }

    false
}
