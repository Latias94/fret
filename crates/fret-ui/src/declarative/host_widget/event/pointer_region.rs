use super::ElementHostWidget;
use crate::declarative::prelude::*;

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

    struct PointerHookHost<'a, H: UiHost> {
        app: &'a mut H,
        window: AppWindowId,
        node: NodeId,
        bounds: Rect,
        input_ctx: &'a fret_runtime::InputContext,
        requested_focus: &'a mut Option<NodeId>,
        requested_capture: &'a mut Option<Option<NodeId>>,
        requested_cursor: &'a mut Option<fret_core::CursorIcon>,
    }

    impl<H: UiHost> action::UiActionHost for PointerHookHost<'_, H> {
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
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            pointer_type,
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
                position: *position,
                button: *button,
                modifiers: *modifiers,
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
                node: cx.node,
                bounds: cx.bounds,
                input_ctx: &cx.input_ctx,
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
                position: *position,
                buttons: *buttons,
                modifiers: *modifiers,
                pointer_type: *pointer_type,
            };

            let mut host = PointerHookHost {
                app: &mut *cx.app,
                window,
                node: cx.node,
                bounds: cx.bounds,
                input_ctx: &cx.input_ctx,
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
        Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            modifiers,
            pointer_type,
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
                position: *position,
                button: *button,
                modifiers: *modifiers,
                pointer_type: *pointer_type,
            };

            if let Some(h) = hook {
                let mut host = PointerHookHost {
                    app: &mut *cx.app,
                    window,
                    node: cx.node,
                    bounds: cx.bounds,
                    input_ctx: &cx.input_ctx,
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
        _ => {}
    }
}
