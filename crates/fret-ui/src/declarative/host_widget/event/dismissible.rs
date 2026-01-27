use super::ElementHostWidget;
use crate::declarative::frame::DismissibleLayerProps;
use crate::declarative::prelude::*;

pub(super) fn handle_dismissible_layer_observer<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut crate::widget::ObserverCx<'_, H>,
    window: AppWindowId,
    props: DismissibleLayerProps,
    event: &Event,
) {
    if !props.enabled {
        return;
    }

    struct DismissibleHookHost<'a, H: UiHost> {
        app: &'a mut H,
        window: AppWindowId,
        element: crate::GlobalElementId,
        notify_requested: &'a mut bool,
    }

    impl<H: UiHost> action::UiActionHost for DismissibleHookHost<'_, H> {
        fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
            self.app.models_mut()
        }

        fn push_effect(&mut self, effect: Effect) {
            match effect {
                Effect::SetTimer {
                    window: Some(window),
                    token,
                    ..
                } if window == self.window => {
                    crate::elements::record_timer_target(
                        &mut *self.app,
                        window,
                        token,
                        self.element,
                    );
                }
                Effect::CancelTimer { token } => {
                    crate::elements::clear_timer_target(&mut *self.app, self.window, token);
                }
                _ => {}
            }
            self.app.push_effect(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.app.request_redraw(window);
        }

        fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
            self.app.next_timer_token()
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.app.next_clipboard_token()
        }

        fn notify(&mut self, _cx: action::ActionCx) {
            *self.notify_requested = true;
        }
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            button,
            modifiers,
            click_count,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::DismissibleActionHooks::default,
                |hooks| hooks.on_dismiss_request.clone(),
            );

            if let Some(h) = hook {
                let mut host = DismissibleHookHost {
                    app: &mut *cx.app,
                    window,
                    element: this.element,
                    notify_requested: &mut cx.notify_requested,
                };
                let mut req = action::DismissRequestCx::new(DismissReason::OutsidePress {
                    pointer: Some(action::OutsidePressCx {
                        pointer_id: *pointer_id,
                        pointer_type: *pointer_type,
                        button: *button,
                        modifiers: *modifiers,
                        click_count: *click_count,
                    }),
                });
                h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: this.element,
                    },
                    &mut req,
                );
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons,
            modifiers,
            pointer_type,
            pointer_id,
            ..
        }) => {
            let pixels_per_point = cx
                .app
                .global::<fret_core::WindowMetricsService>()
                .and_then(|svc| svc.scale_factor(window))
                .unwrap_or(1.0);

            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::DismissibleActionHooks::default,
                |hooks| hooks.on_pointer_move.clone(),
            );
            let Some(h) = hook else {
                return;
            };

            let mv = action::PointerMoveCx {
                pointer_id: *pointer_id,
                position: *position,
                tick_id: cx.app.tick_id(),
                pixels_per_point,
                buttons: *buttons,
                modifiers: *modifiers,
                pointer_type: *pointer_type,
            };

            let mut host = DismissibleHookHost {
                app: &mut *cx.app,
                window,
                element: this.element,
                notify_requested: &mut cx.notify_requested,
            };
            let handled = h(
                &mut host,
                action::ActionCx {
                    window,
                    target: this.element,
                },
                mv,
            );

            if handled {
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
        }
        _ => {}
    }
}

pub(super) fn handle_dismissible_layer<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: DismissibleLayerProps,
    event: &Event,
) {
    if !props.enabled {
        return;
    }

    struct DismissibleHookHost<'a, H: UiHost> {
        app: &'a mut H,
        window: AppWindowId,
        element: crate::GlobalElementId,
        notify_requested: &'a mut bool,
    }

    impl<H: UiHost> action::UiActionHost for DismissibleHookHost<'_, H> {
        fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
            self.app.models_mut()
        }

        fn push_effect(&mut self, effect: Effect) {
            match effect {
                Effect::SetTimer {
                    window: Some(window),
                    token,
                    ..
                } if window == self.window => {
                    crate::elements::record_timer_target(
                        &mut *self.app,
                        window,
                        token,
                        self.element,
                    );
                }
                Effect::CancelTimer { token } => {
                    crate::elements::clear_timer_target(&mut *self.app, self.window, token);
                }
                _ => {}
            }
            self.app.push_effect(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.app.request_redraw(window);
        }

        fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
            self.app.next_timer_token()
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.app.next_clipboard_token()
        }

        fn notify(&mut self, _cx: action::ActionCx) {
            *self.notify_requested = true;
        }
    }

    match event {
        Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            repeat: false,
            ..
        } => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::DismissibleActionHooks::default,
                |hooks| hooks.on_dismiss_request.clone(),
            );

            if let Some(h) = hook {
                let mut host = DismissibleHookHost {
                    app: &mut *cx.app,
                    window,
                    element: this.element,
                    notify_requested: &mut cx.notify_requested,
                };
                let mut req = action::DismissRequestCx::new(DismissReason::Escape);
                h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: this.element,
                    },
                    &mut req,
                );
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons,
            modifiers,
            pointer_type,
            pointer_id,
            ..
        }) => {
            let pixels_per_point = cx
                .app
                .global::<fret_core::WindowMetricsService>()
                .and_then(|svc| svc.scale_factor(window))
                .unwrap_or(1.0);

            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::DismissibleActionHooks::default,
                |hooks| hooks.on_pointer_move.clone(),
            );

            let Some(h) = hook else {
                return;
            };

            let mv = action::PointerMoveCx {
                pointer_id: *pointer_id,
                position: *position,
                tick_id: cx.app.tick_id(),
                pixels_per_point,
                buttons: *buttons,
                modifiers: *modifiers,
                pointer_type: *pointer_type,
            };

            let mut host = DismissibleHookHost {
                app: &mut *cx.app,
                window,
                element: this.element,
                notify_requested: &mut cx.notify_requested,
            };
            let handled = h(
                &mut host,
                action::ActionCx {
                    window,
                    target: this.element,
                },
                mv,
            );

            if handled {
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
        }
        _ => {}
    }
}
