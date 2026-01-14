use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_pressable<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: PressableProps,
    event: &Event,
) {
    if !props.enabled {
        return;
    }

    let pixels_per_point = cx
        .app
        .global::<fret_core::WindowMetricsService>()
        .and_then(|svc| svc.scale_factor(window))
        .unwrap_or(1.0);
    match event {
        Event::Pointer(pe) => match pe {
            fret_core::PointerEvent::Move { .. } => {
                cx.set_cursor_icon(CursorIcon::Pointer);
            }
            fret_core::PointerEvent::Down {
                position,
                button,
                modifiers,
                pointer_type,
                click_count,
                ..
            } => {
                let hook = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    this.element,
                    crate::action::PressableActionHooks::default,
                    |hooks| hooks.on_pointer_down.clone(),
                );

                if let Some(h) = hook {
                    struct PressablePointerHookHost<'a, H: UiHost> {
                        app: &'a mut H,
                        window: AppWindowId,
                        element: crate::GlobalElementId,
                        node: NodeId,
                        bounds: Rect,
                        input_ctx: &'a fret_runtime::InputContext,
                        requested_focus: &'a mut Option<NodeId>,
                        requested_capture: &'a mut Option<Option<NodeId>>,
                        requested_cursor: &'a mut Option<fret_core::CursorIcon>,
                    }

                    impl<H: UiHost> action::UiActionHost for PressablePointerHookHost<'_, H> {
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
                                    crate::elements::clear_timer_target(
                                        &mut *self.app,
                                        self.window,
                                        token,
                                    );
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
                    }

                    impl<H: UiHost> action::UiFocusActionHost for PressablePointerHookHost<'_, H> {
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

                    impl<H: UiHost> action::UiPointerActionHost for PressablePointerHookHost<'_, H> {
                        fn bounds(&self) -> Rect {
                            self.bounds
                        }

                        fn capture_pointer(&mut self) {
                            *self.requested_capture = Some(Some(self.node));
                        }

                        fn release_pointer_capture(&mut self) {
                            *self.requested_capture = Some(None);
                        }

                        fn set_cursor_icon(&mut self, icon: fret_core::CursorIcon) {
                            if !self.input_ctx.caps.ui.cursor_icons {
                                return;
                            }
                            *self.requested_cursor = Some(icon);
                        }
                    }

                    let down = action::PointerDownCx {
                        position: *position,
                        pixels_per_point,
                        button: *button,
                        modifiers: *modifiers,
                        click_count: *click_count,
                        pointer_type: *pointer_type,
                    };

                    let mut host = PressablePointerHookHost {
                        app: &mut *cx.app,
                        window,
                        element: this.element,
                        node: cx.node,
                        bounds: cx.bounds,
                        input_ctx: &cx.input_ctx,
                        requested_focus: &mut cx.requested_focus,
                        requested_capture: &mut cx.requested_capture,
                        requested_cursor: &mut cx.requested_cursor,
                    };

                    match h(
                        &mut host,
                        action::ActionCx {
                            window,
                            target: this.element,
                        },
                        down,
                    ) {
                        action::PressablePointerDownResult::Continue => {}
                        action::PressablePointerDownResult::SkipDefault => {
                            return;
                        }
                        action::PressablePointerDownResult::SkipDefaultAndStopPropagation => {
                            cx.stop_propagation();
                            return;
                        }
                    }
                }

                if *button != MouseButton::Left {
                    return;
                }
                if props.focusable {
                    cx.request_focus(cx.node);
                }
                cx.capture_pointer(cx.node);
                crate::elements::set_pressed_pressable(&mut *cx.app, window, Some(this.element));
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            fret_core::PointerEvent::Up {
                button, position, ..
            } => {
                if *button != MouseButton::Left {
                    return;
                }
                let pressed =
                    crate::elements::is_pressed_pressable(&mut *cx.app, window, this.element);
                cx.release_pointer_capture();
                crate::elements::set_pressed_pressable(&mut *cx.app, window, None);

                // Activate based on the pointer-up position, not the cached hovered state. This
                // keeps click-through outside-press dismissal semantics (ADR 0069) robust even
                // when overlay policies update hover state in an observer pass.
                let hovered = cx.bounds.contains(*position);

                if pressed && hovered {
                    let hook = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        this.element,
                        crate::action::PressableActionHooks::default,
                        |hooks| hooks.on_activate.clone(),
                    );

                    if let Some(h) = hook {
                        struct PressableActivateHookHost<'a, H: UiHost> {
                            app: &'a mut H,
                            window: AppWindowId,
                            element: crate::GlobalElementId,
                        }

                        impl<H: UiHost> action::UiActionHost for PressableActivateHookHost<'_, H> {
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
                                        crate::elements::clear_timer_target(
                                            &mut *self.app,
                                            self.window,
                                            token,
                                        );
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
                        }

                        let mut host = PressableActivateHookHost {
                            app: &mut *cx.app,
                            window,
                            element: this.element,
                        };
                        h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: this.element,
                            },
                            ActivateReason::Pointer,
                        );
                    }
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        },
        Event::KeyDown { key, repeat, .. } => {
            if *repeat {
                return;
            }
            if cx.focus != Some(cx.node) {
                return;
            }
            if !matches!(
                key,
                fret_core::KeyCode::Enter
                    | fret_core::KeyCode::NumpadEnter
                    | fret_core::KeyCode::Space
            ) {
                return;
            }
            crate::elements::set_pressed_pressable(&mut *cx.app, window, Some(this.element));
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
        }
        Event::KeyUp { key, .. } => {
            if cx.focus != Some(cx.node) {
                return;
            }
            if !matches!(
                key,
                fret_core::KeyCode::Enter
                    | fret_core::KeyCode::NumpadEnter
                    | fret_core::KeyCode::Space
            ) {
                return;
            }
            let pressed = crate::elements::is_pressed_pressable(&mut *cx.app, window, this.element);
            if !pressed {
                return;
            }
            crate::elements::set_pressed_pressable(&mut *cx.app, window, None);
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::PressableActionHooks::default,
                |hooks| hooks.on_activate.clone(),
            );

            if let Some(h) = hook {
                struct PressableActivateHookHost<'a, H: UiHost> {
                    app: &'a mut H,
                    window: AppWindowId,
                    element: crate::GlobalElementId,
                }

                impl<H: UiHost> action::UiActionHost for PressableActivateHookHost<'_, H> {
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
                                crate::elements::clear_timer_target(
                                    &mut *self.app,
                                    self.window,
                                    token,
                                );
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
                }

                let mut host = PressableActivateHookHost {
                    app: &mut *cx.app,
                    window,
                    element: this.element,
                };
                h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: this.element,
                    },
                    ActivateReason::Keyboard,
                );
            }
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
        }
        _ => {}
    };
}
