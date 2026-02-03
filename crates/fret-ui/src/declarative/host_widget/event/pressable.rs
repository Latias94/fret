use super::ElementHostWidget;
use crate::declarative::prelude::*;
use fret_runtime::DragHost;

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

    struct PressablePointerHookHost<'a, H: UiHost> {
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
        notify_requested: &'a mut bool,
        notify_requested_location: &'a mut Option<crate::widget::UiSourceLocation>,
        invalidations: &'a mut Vec<(NodeId, Invalidation)>,
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

        #[track_caller]
        fn notify(&mut self, _cx: action::ActionCx) {
            *self.notify_requested = true;
            if self.notify_requested_location.is_none() {
                let caller = std::panic::Location::caller();
                *self.notify_requested_location = Some(crate::widget::UiSourceLocation {
                    file: caller.file(),
                    line: caller.line(),
                    column: caller.column(),
                });
            }
        }
    }

    impl<H: UiHost> action::UiFocusActionHost for PressablePointerHookHost<'_, H> {
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

        fn prevent_default(&mut self, action: fret_runtime::DefaultAction) {
            self.prevented_default_actions.insert(action);
        }

        fn invalidate(&mut self, invalidation: Invalidation) {
            self.invalidations.push((self.node, invalidation));
        }
    }

    impl<H: UiHost> action::UiDragActionHost for PressablePointerHookHost<'_, H> {
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
        Event::Pointer(pe) => match pe {
            fret_core::PointerEvent::Move {
                pointer_id,
                position,
                buttons,
                modifiers,
                pointer_type,
            } => {
                cx.set_cursor_icon(CursorIcon::Pointer);

                let hook = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    this.element,
                    crate::action::PressableActionHooks::default,
                    |hooks| hooks.on_pointer_move.clone(),
                );

                if let Some(h) = hook {
                    let mv = action::PointerMoveCx {
                        pointer_id: *pointer_id,
                        position: *position,
                        tick_id: cx.app.tick_id(),
                        pixels_per_point,
                        buttons: *buttons,
                        modifiers: *modifiers,
                        pointer_type: *pointer_type,
                    };

                    let mut host = PressablePointerHookHost {
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
                        notify_requested: &mut cx.notify_requested,
                        notify_requested_location: &mut cx.notify_requested_location,
                        invalidations: &mut cx.invalidations,
                    };

                    if h(
                        &mut host,
                        action::ActionCx {
                            window,
                            target: this.element,
                        },
                        mv,
                    ) {
                        cx.stop_propagation();
                    }
                }
            }
            fret_core::PointerEvent::Down {
                position,
                button,
                modifiers,
                pointer_type,
                click_count,
                pointer_id,
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

                    let mut host = PressablePointerHookHost {
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
                        notify_requested: &mut cx.notify_requested,
                        notify_requested_location: &mut cx.notify_requested_location,
                        invalidations: &mut cx.invalidations,
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
                pointer_id,
                position,
                button,
                modifiers,
                is_click,
                click_count,
                pointer_type,
            } => {
                if *button != MouseButton::Left {
                    return;
                }
                let pressed =
                    crate::elements::is_pressed_pressable(&mut *cx.app, window, this.element);

                let hook = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    this.element,
                    crate::action::PressableActionHooks::default,
                    |hooks| hooks.on_pointer_up.clone(),
                );

                let mut skip_activate = false;
                if let Some(h) = hook {
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

                    let mut host = PressablePointerHookHost {
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
                        notify_requested: &mut cx.notify_requested,
                        notify_requested_location: &mut cx.notify_requested_location,
                        invalidations: &mut cx.invalidations,
                    };

                    skip_activate = matches!(
                        h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: this.element,
                            },
                            up,
                        ),
                        action::PressablePointerUpResult::SkipActivate
                    );
                }

                cx.release_pointer_capture();
                crate::elements::set_pressed_pressable(&mut *cx.app, window, None);

                // Activate based on the pointer-up position, not the cached hovered state. This
                // keeps click-through outside-press dismissal semantics (ADR 0069) robust even
                // when overlay policies update hover state in an observer pass.
                let hovered = cx.bounds.contains(*position);

                let is_touch = *pointer_type == fret_core::PointerType::Touch;
                if pressed && hovered && (!is_touch || *is_click) && !skip_activate {
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
                            notify_requested: &'a mut bool,
                            notify_requested_location:
                                &'a mut Option<crate::widget::UiSourceLocation>,
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

                            fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
                                self.app.next_clipboard_token()
                            }

                            #[track_caller]
                            fn notify(&mut self, _cx: action::ActionCx) {
                                *self.notify_requested = true;
                                if self.notify_requested_location.is_none() {
                                    let caller = std::panic::Location::caller();
                                    *self.notify_requested_location =
                                        Some(crate::widget::UiSourceLocation {
                                            file: caller.file(),
                                            line: caller.line(),
                                            column: caller.column(),
                                        });
                                }
                            }
                        }

                        let mut host = PressableActivateHookHost {
                            app: &mut *cx.app,
                            window,
                            element: this.element,
                            notify_requested: &mut cx.notify_requested,
                            notify_requested_location: &mut cx.notify_requested_location,
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
                    notify_requested: &'a mut bool,
                    notify_requested_location: &'a mut Option<crate::widget::UiSourceLocation>,
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

                    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
                        self.app.next_clipboard_token()
                    }

                    #[track_caller]
                    fn notify(&mut self, _cx: action::ActionCx) {
                        *self.notify_requested = true;
                        if self.notify_requested_location.is_none() {
                            let caller = std::panic::Location::caller();
                            *self.notify_requested_location =
                                Some(crate::widget::UiSourceLocation {
                                    file: caller.file(),
                                    line: caller.line(),
                                    column: caller.column(),
                                });
                        }
                    }
                }

                let mut host = PressableActivateHookHost {
                    app: &mut *cx.app,
                    window,
                    element: this.element,
                    notify_requested: &mut cx.notify_requested,
                    notify_requested_location: &mut cx.notify_requested_location,
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
