use super::ElementHostWidget;
use crate::declarative::prelude::*;
use fret_runtime::DragHost;

pub(super) fn handle_pointer_region<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: PointerRegionProps,
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

    struct PointerHookHost<'a, H: UiHost> {
        app: &'a mut H,
        window: AppWindowId,
        element: crate::GlobalElementId,
        node: NodeId,
        bounds: Rect,
        input_ctx: &'a fret_runtime::InputContext,
        prevented_default_actions: &'a mut fret_runtime::DefaultActionSet,
        requested_focus: &'a mut Option<NodeId>,
        requested_capture: &'a mut Option<Option<NodeId>>,
        requested_cursor: &'a mut Option<fret_core::CursorIcon>,
    }

    impl<H: UiHost> action::UiActionHost for PointerHookHost<'_, H> {
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
    }

    impl<H: UiHost> action::UiFocusActionHost for PointerHookHost<'_, H> {
        fn request_focus(&mut self, target: crate::GlobalElementId) {
            let Some(node) =
                crate::elements::with_window_state(&mut *self.app, self.window, |window_state| {
                    window_state.node_entry(target).map(|e| e.node)
                })
            else {
                return;
            };
            *self.requested_focus = Some(node);
        }
    }

    impl<H: UiHost> action::UiPointerActionHost for PointerHookHost<'_, H> {
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

        fn prevent_default(&mut self, action: fret_runtime::DefaultAction) {
            self.prevented_default_actions.insert(action);
        }
    }

    impl<H: UiHost> action::UiDragActionHost for PointerHookHost<'_, H> {
        fn begin_drag_with_kind(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: fret_runtime::DragKindId,
            source_window: AppWindowId,
            start: Point,
        ) {
            DragHost::begin_drag_with_kind(
                &mut *self.app,
                pointer_id,
                kind,
                source_window,
                start,
                (),
            );
        }

        fn begin_cross_window_drag_with_kind(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: fret_runtime::DragKindId,
            source_window: AppWindowId,
            start: Point,
        ) {
            DragHost::begin_cross_window_drag_with_kind(
                &mut *self.app,
                pointer_id,
                kind,
                source_window,
                start,
                (),
            );
        }

        fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&fret_runtime::DragSession> {
            DragHost::drag(&*self.app, pointer_id)
        }

        fn drag_mut(
            &mut self,
            pointer_id: fret_core::PointerId,
        ) -> Option<&mut fret_runtime::DragSession> {
            DragHost::drag_mut(&mut *self.app, pointer_id)
        }

        fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
            DragHost::cancel_drag(&mut *self.app, pointer_id);
        }
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            pointer_type,
            click_count,
            pointer_id,
            ..
        }) => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::PointerActionHooks::default,
                |hooks| hooks.on_pointer_down.clone(),
            );

            let down = action::PointerDownCx {
                pointer_id: *pointer_id,
                position: *position,
                tick_id: cx.app.tick_id(),
                pixels_per_point,
                button: *button,
                modifiers: *modifiers,
                click_count: *click_count,
                pointer_type: *pointer_type,
            };

            let Some(h) = hook else {
                return;
            };

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::PointerRegionState::default,
                |state| {
                    state.last_down = Some(down);
                },
            );

            let mut host = PointerHookHost {
                app: &mut *cx.app,
                window,
                element: this.element,
                node: cx.node,
                bounds: cx.bounds,
                input_ctx: &cx.input_ctx,
                prevented_default_actions: cx.prevented_default_actions,
                requested_focus: &mut cx.requested_focus,
                requested_capture: &mut cx.requested_capture,
                requested_cursor: &mut cx.requested_cursor,
            };
            let handled = h(
                &mut host,
                action::ActionCx {
                    window,
                    target: this.element,
                },
                down,
            );

            if handled {
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
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::PointerActionHooks::default,
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

            let mut host = PointerHookHost {
                app: &mut *cx.app,
                window,
                element: this.element,
                node: cx.node,
                bounds: cx.bounds,
                input_ctx: &cx.input_ctx,
                prevented_default_actions: cx.prevented_default_actions,
                requested_focus: &mut cx.requested_focus,
                requested_capture: &mut cx.requested_capture,
                requested_cursor: &mut cx.requested_cursor,
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
                cx.stop_propagation();
            }
        }
        Event::Pointer(fret_core::PointerEvent::Wheel {
            position,
            delta,
            modifiers,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::PointerActionHooks::default,
                |hooks| hooks.on_wheel.clone(),
            );

            let Some(h) = hook else {
                return;
            };

            let wheel = action::WheelCx {
                pointer_id: *pointer_id,
                position: *position,
                tick_id: cx.app.tick_id(),
                pixels_per_point,
                delta: *delta,
                modifiers: *modifiers,
                pointer_type: *pointer_type,
            };

            let mut host = PointerHookHost {
                app: &mut *cx.app,
                window,
                element: this.element,
                node: cx.node,
                bounds: cx.bounds,
                input_ctx: &cx.input_ctx,
                prevented_default_actions: cx.prevented_default_actions,
                requested_focus: &mut cx.requested_focus,
                requested_capture: &mut cx.requested_capture,
                requested_cursor: &mut cx.requested_cursor,
            };
            let handled = h(
                &mut host,
                action::ActionCx {
                    window,
                    target: this.element,
                },
                wheel,
            );

            if handled {
                cx.stop_propagation();
            }
        }
        Event::Pointer(fret_core::PointerEvent::PinchGesture {
            position,
            delta,
            modifiers,
            pointer_type,
            pointer_id,
            ..
        }) => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::PointerActionHooks::default,
                |hooks| hooks.on_pinch_gesture.clone(),
            );

            let Some(h) = hook else {
                return;
            };

            let pinch = action::PinchGestureCx {
                pointer_id: *pointer_id,
                position: *position,
                tick_id: cx.app.tick_id(),
                pixels_per_point,
                delta: *delta,
                modifiers: *modifiers,
                pointer_type: *pointer_type,
            };

            let mut host = PointerHookHost {
                app: &mut *cx.app,
                window,
                element: this.element,
                node: cx.node,
                bounds: cx.bounds,
                input_ctx: &cx.input_ctx,
                prevented_default_actions: cx.prevented_default_actions,
                requested_focus: &mut cx.requested_focus,
                requested_capture: &mut cx.requested_capture,
                requested_cursor: &mut cx.requested_cursor,
            };
            let handled = h(
                &mut host,
                action::ActionCx {
                    window,
                    target: this.element,
                },
                pinch,
            );

            if handled {
                cx.stop_propagation();
            }
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            modifiers,
            is_click,
            pointer_type,
            click_count,
            pointer_id,
            ..
        }) => {
            let was_captured = cx.captured == Some(cx.node);

            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::PointerActionHooks::default,
                |hooks| hooks.on_pointer_up.clone(),
            );

            let up = action::PointerUpCx {
                pointer_id: *pointer_id,
                position: *position,
                tick_id: cx.app.tick_id(),
                pixels_per_point,
                button: *button,
                modifiers: *modifiers,
                is_click: *is_click,
                click_count: *click_count,
                pointer_type: *pointer_type,
            };

            if let Some(h) = hook {
                let mut host = PointerHookHost {
                    app: &mut *cx.app,
                    window,
                    element: this.element,
                    node: cx.node,
                    bounds: cx.bounds,
                    input_ctx: &cx.input_ctx,
                    prevented_default_actions: cx.prevented_default_actions,
                    requested_focus: &mut cx.requested_focus,
                    requested_capture: &mut cx.requested_capture,
                    requested_cursor: &mut cx.requested_cursor,
                };
                let handled = h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: this.element,
                    },
                    up,
                );

                if handled {
                    cx.stop_propagation();
                }
            }

            if was_captured {
                cx.release_pointer_capture();
            }
        }
        Event::PointerCancel(e) => {
            let was_captured = cx.captured == Some(cx.node);

            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::PointerActionHooks::default,
                |hooks| hooks.on_pointer_cancel.clone(),
            );

            if let Some(h) = hook {
                let cancel = action::PointerCancelCx {
                    pointer_id: e.pointer_id,
                    position: e.position,
                    tick_id: cx.app.tick_id(),
                    pixels_per_point,
                    buttons: e.buttons,
                    modifiers: e.modifiers,
                    pointer_type: e.pointer_type,
                    reason: e.reason,
                };

                let mut host = PointerHookHost {
                    app: &mut *cx.app,
                    window,
                    element: this.element,
                    node: cx.node,
                    bounds: cx.bounds,
                    input_ctx: &cx.input_ctx,
                    prevented_default_actions: cx.prevented_default_actions,
                    requested_focus: &mut cx.requested_focus,
                    requested_capture: &mut cx.requested_capture,
                    requested_cursor: &mut cx.requested_cursor,
                };
                let handled = h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: this.element,
                    },
                    cancel,
                );

                if handled {
                    cx.stop_propagation();
                }
            }

            if was_captured {
                cx.release_pointer_capture();
            }
        }
        _ => {}
    }
}
