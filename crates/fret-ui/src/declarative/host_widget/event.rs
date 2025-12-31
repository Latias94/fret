use super::ElementHostWidget;
use super::super::frame::*;
use super::super::mount::node_for_element_in_window_frame;
use super::super::paint_helpers::*;
use super::super::prelude::*;

impl ElementHostWidget {
    pub(super) fn event_impl<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        let is_text_input = matches!(
            instance,
            ElementInstance::TextInput(_) | ElementInstance::TextArea(_)
        );

        if let Event::Timer { token } = event {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                crate::action::TimerActionHooks::default,
                |hooks| hooks.on_timer.clone(),
            );

            if let Some(h) = hook {
                let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                let handled = h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: self.element,
                    },
                    *token,
                );
                if handled {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
        }

        let try_key_hook = |cx: &mut EventCx<'_, H>,
                            key: fret_core::KeyCode,
                            modifiers: fret_core::Modifiers,
                            repeat: bool| {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                self.element,
                crate::action::KeyActionHooks::default,
                |hooks| hooks.on_key_down.clone(),
            );

            if let Some(h) = hook {
                let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                let handled = h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: self.element,
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
        };

        if let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
            && cx.focus == Some(cx.node)
            && !is_text_input
            && try_key_hook(cx, *key, *modifiers, *repeat)
        {
            return;
        }

        match instance {
            ElementInstance::TextInput(props) => {
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(props.model));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != props.model.id() {
                    input.set_model(props.model);
                }
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);
                input.event(cx, event);
            }
            ElementInstance::TextArea(props) => {
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(props.model));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != props.model.id() {
                    area.set_model(props.model);
                }
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);
                area.event(cx, event);
            }
            ElementInstance::ResizablePanelGroup(props) => {
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            props.model,
                        ));
                }
                let group = self
                    .resizable_panel_group
                    .as_mut()
                    .expect("resizable panel group");
                if group.model_id() != props.model.id() {
                    group.set_model(props.model);
                }
                group.set_axis(props.axis);
                group.set_enabled(props.enabled);
                group.set_min_px(props.min_px.clone());
                group.set_style(props.chrome.clone());
                group.event(cx, event);
            }
            ElementInstance::VirtualList(props) => {
                let Event::Pointer(pe) = event else {
                    return;
                };
                match pe {
                    fret_core::PointerEvent::Wheel { delta, .. } => {
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::VirtualListState::default,
                            |state| {
                                state.metrics.ensure(
                                    props.len,
                                    props.estimate_row_height,
                                    props.gap,
                                    props.scroll_margin,
                                );
                                let viewport_h = Px(state.viewport_h.0.max(0.0));

                                let prev = props.scroll_handle.offset();
                                let offset_y = state.metrics.clamp_offset(prev.y, viewport_h);

                                let next = state
                                    .metrics
                                    .clamp_offset(Px(offset_y.0 - delta.y.0), viewport_h);
                                if (prev.y.0 - next.0).abs() > 0.01 {
                                    props
                                        .scroll_handle
                                        .set_offset(fret_core::Point::new(prev.x, next));
                                }
                            },
                        );
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    fret_core::PointerEvent::Down { button, .. } => {
                        if *button == MouseButton::Left {
                            cx.request_focus(cx.node);
                        }
                    }
                    _ => {}
                }
            }
            ElementInstance::Scroll(props) => {
                let Event::Pointer(pe) = event else {
                    return;
                };
                if let fret_core::PointerEvent::Wheel { delta, .. } = pe {
                    if let Some(handle) = props.scroll_handle.as_ref() {
                        let prev = handle.offset();
                        handle.set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));
                    } else {
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollState::default,
                            |state| {
                                let prev = state.scroll_handle.offset();
                                state
                                    .scroll_handle
                                    .set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));
                            },
                        );
                    }
                    cx.invalidate_self(Invalidation::Layout);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            ElementInstance::Scrollbar(props) => {
                let Event::Pointer(pe) = event else {
                    return;
                };

                let handle = props.scroll_handle.clone();
                let scroll_target = props.scroll_target;
                match pe {
                    fret_core::PointerEvent::Wheel { delta, .. } => {
                        let prev = handle.offset();
                        handle.set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));

                        if let Some(target) = scroll_target
                            && let Some(node) =
                                node_for_element_in_window_frame(&mut *cx.app, window, target)
                        {
                            cx.invalidate(node, Invalidation::Layout);
                            cx.invalidate(node, Invalidation::Paint);
                        }

                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    fret_core::PointerEvent::Move { position, .. } => {
                        let mut needs_layout = false;
                        let mut needs_paint = false;

                        let bounds = cx.bounds;
                        let position = *position;

                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollbarState::default,
                            |state| {
                                let viewport_h = Px(handle.viewport_size().height.0.max(0.0));
                                let content_h = Px(handle.content_size().height.0.max(0.0));
                                let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));

                                let hovered = bounds.contains(position);
                                if state.hovered != hovered && !state.dragging_thumb {
                                    state.hovered = hovered;
                                    needs_paint = true;
                                }

                                if state.dragging_thumb
                                    && max_offset.0 > 0.0
                                    && let Some(thumb) = scrollbar_thumb_rect(
                                        bounds,
                                        viewport_h,
                                        content_h,
                                        state.drag_start_offset_y,
                                    )
                                {
                                    let max_thumb_y =
                                        (bounds.size.height.0 - thumb.size.height.0).max(0.0);
                                    if max_thumb_y > 0.0 {
                                        let delta_y = position.y.0 - state.drag_start_pointer_y.0;
                                        let scale = max_offset.0 / max_thumb_y;
                                        let next = Px((state.drag_start_offset_y.0
                                            + delta_y * scale)
                                            .max(0.0));
                                        let next = Px(next.0.min(max_offset.0));
                                        if (handle.offset().y.0 - next.0).abs() > 0.01 {
                                            let prev = handle.offset();
                                            handle.set_offset(Point::new(prev.x, next));
                                            needs_layout = true;
                                            needs_paint = true;
                                        }
                                        state.hovered = true;
                                    }
                                }
                            },
                        );

                        if needs_layout {
                            cx.invalidate_self(Invalidation::Layout);
                            if let Some(target) = scroll_target
                                && let Some(node) =
                                    node_for_element_in_window_frame(&mut *cx.app, window, target)
                            {
                                cx.invalidate(node, Invalidation::Layout);
                                cx.invalidate(node, Invalidation::Paint);
                            }
                        }
                        if needs_paint {
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                    }
                    fret_core::PointerEvent::Down {
                        position, button, ..
                    } => {
                        if *button != MouseButton::Left {
                            return;
                        }

                        let bounds = cx.bounds;
                        let position = *position;

                        let mut did_handle = false;
                        let mut did_start_drag = false;
                        let mut did_change_offset = false;
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollbarState::default,
                            |state| {
                                let viewport_h = Px(handle.viewport_size().height.0.max(0.0));
                                let content_h = Px(handle.content_size().height.0.max(0.0));
                                let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
                                if max_offset.0 <= 0.0 {
                                    return;
                                }

                                let Some(thumb) = scrollbar_thumb_rect(
                                    bounds,
                                    viewport_h,
                                    content_h,
                                    handle.offset().y,
                                ) else {
                                    return;
                                };

                                did_handle = true;
                                state.hovered = true;

                                if thumb.contains(position) {
                                    state.dragging_thumb = true;
                                    state.drag_start_pointer_y = position.y;
                                    state.drag_start_offset_y = handle.offset().y;
                                    did_start_drag = true;
                                } else if bounds.contains(position) {
                                    // Page to the click position (center the thumb on the pointer).
                                    let max_thumb_y =
                                        (bounds.size.height.0 - thumb.size.height.0).max(0.0);
                                    if max_thumb_y > 0.0 {
                                        let click_y = (position.y.0 - bounds.origin.y.0)
                                            .clamp(0.0, bounds.size.height.0);
                                        let thumb_top = (click_y - thumb.size.height.0 * 0.5)
                                            .clamp(0.0, max_thumb_y);
                                        let t = thumb_top / max_thumb_y;
                                        let next = Px((max_offset.0 * t).clamp(0.0, max_offset.0));
                                        let prev = handle.offset();
                                        handle.set_offset(Point::new(prev.x, next));
                                        did_change_offset = true;
                                    }
                                } else {
                                    did_handle = false;
                                }
                            },
                        );

                        if did_handle {
                            if did_start_drag {
                                cx.capture_pointer(cx.node);
                            }
                            if did_change_offset
                                && let Some(target) = scroll_target
                                && let Some(node) =
                                    node_for_element_in_window_frame(&mut *cx.app, window, target)
                            {
                                cx.invalidate(node, Invalidation::Layout);
                                cx.invalidate(node, Invalidation::Paint);
                            }
                            cx.invalidate_self(Invalidation::Layout);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    fret_core::PointerEvent::Up { button, .. } => {
                        if *button != MouseButton::Left {
                            return;
                        }

                        let mut did_handle = false;
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::element::ScrollbarState::default,
                            |state| {
                                if state.dragging_thumb {
                                    did_handle = true;
                                    state.dragging_thumb = false;
                                }
                            },
                        );
                        if did_handle {
                            cx.release_pointer_capture();
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                }
            }
            ElementInstance::DismissibleLayer(props) => {
                if !props.enabled {
                    return;
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
                            self.element,
                            crate::action::DismissibleActionHooks::default,
                            |hooks| hooks.on_dismiss_request.clone(),
                        );

                        if let Some(h) = hook {
                            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                            h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                DismissReason::Escape,
                            );
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    Event::Pointer(fret_core::PointerEvent::Down { .. }) => {
                        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Observer
                        {
                            return;
                        }
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::DismissibleActionHooks::default,
                            |hooks| hooks.on_dismiss_request.clone(),
                        );

                        if let Some(h) = hook {
                            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                            h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
                                },
                                DismissReason::OutsidePress,
                            );
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            ElementInstance::Pressable(props) => {
                if !props.enabled {
                    return;
                }
                match event {
                    Event::Pointer(pe) => match pe {
                        fret_core::PointerEvent::Move { .. } => {
                            cx.set_cursor_icon(CursorIcon::Pointer);
                        }
                        fret_core::PointerEvent::Down { button, .. } => {
                            if *button != MouseButton::Left {
                                return;
                            }
                            if props.focusable {
                                cx.request_focus(cx.node);
                            }
                            cx.capture_pointer(cx.node);
                            crate::elements::set_pressed_pressable(
                                &mut *cx.app,
                                window,
                                Some(self.element),
                            );
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                        fret_core::PointerEvent::Up { button, .. } => {
                            if *button != MouseButton::Left {
                                return;
                            }
                            cx.release_pointer_capture();
                            crate::elements::set_pressed_pressable(&mut *cx.app, window, None);

                            let hovered = crate::elements::is_hovered_pressable(
                                &mut *cx.app,
                                window,
                                self.element,
                            );

                            if hovered {
                                let hook = crate::elements::with_element_state(
                                    &mut *cx.app,
                                    window,
                                    self.element,
                                    crate::action::PressableActionHooks::default,
                                    |hooks| hooks.on_activate.clone(),
                                );

                                if let Some(h) = hook {
                                    let mut host =
                                        action::UiActionHostAdapter { app: &mut *cx.app };
                                    h(
                                        &mut host,
                                        action::ActionCx {
                                            window,
                                            target: self.element,
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
                        crate::elements::set_pressed_pressable(
                            &mut *cx.app,
                            window,
                            Some(self.element),
                        );
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
                        let pressed = crate::elements::is_pressed_pressable(
                            &mut *cx.app,
                            window,
                            self.element,
                        );
                        if !pressed {
                            return;
                        }
                        crate::elements::set_pressed_pressable(&mut *cx.app, window, None);
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PressableActionHooks::default,
                            |hooks| hooks.on_activate.clone(),
                        );

                        if let Some(h) = hook {
                            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                            h(
                                &mut host,
                                action::ActionCx {
                                    window,
                                    target: self.element,
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
            ElementInstance::PointerRegion(props) => {
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

                    fn next_timer_token(&mut self) -> fret_core::TimerToken {
                        self.app.next_timer_token()
                    }
                }

                impl<H: UiHost> action::UiPointerActionHost for PointerHookHost<'_, H> {
                    fn bounds(&self) -> Rect {
                        self.bounds
                    }

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
                    }) => {
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PointerActionHooks::default,
                            |hooks| hooks.on_pointer_down.clone(),
                        );

                        let Some(h) = hook else {
                            return;
                        };

                        let down = action::PointerDownCx {
                            position: *position,
                            button: *button,
                            modifiers: *modifiers,
                        };

                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
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
                                target: self.element,
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
                    }) => {
                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
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
                                target: self.element,
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
                    }) => {
                        let was_captured = cx.captured == Some(cx.node);

                        let hook = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            self.element,
                            crate::action::PointerActionHooks::default,
                            |hooks| hooks.on_pointer_up.clone(),
                        );

                        let up = action::PointerUpCx {
                            position: *position,
                            button: *button,
                            modifiers: *modifiers,
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
                                    target: self.element,
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
            ElementInstance::RovingFlex(props) => {
                if !props.roving.enabled {
                    return;
                }

                let Event::KeyDown { key, repeat, .. } = event else {
                    return;
                };
                if *repeat {
                    return;
                }

                enum Nav {
                    Prev,
                    Next,
                    Home,
                    End,
                }

                let nav = match (props.flex.direction, key) {
                    (_, fret_core::KeyCode::Home) => Some(Nav::Home),
                    (_, fret_core::KeyCode::End) => Some(Nav::End),
                    (fret_core::Axis::Vertical, fret_core::KeyCode::ArrowUp) => Some(Nav::Prev),
                    (fret_core::Axis::Vertical, fret_core::KeyCode::ArrowDown) => Some(Nav::Next),
                    (fret_core::Axis::Horizontal, fret_core::KeyCode::ArrowLeft) => Some(Nav::Prev),
                    (fret_core::Axis::Horizontal, fret_core::KeyCode::ArrowRight) => {
                        Some(Nav::Next)
                    }
                    _ => None,
                };
                let len = cx.children.len();
                if len == 0 {
                    return;
                }

                let current = cx
                    .focus
                    .and_then(|focus| cx.children.iter().position(|n| *n == focus));

                let is_disabled = |idx: usize| -> bool {
                    props.roving.disabled.get(idx).copied().unwrap_or(false)
                };

                let mut target: Option<usize> = None;
                match nav {
                    Some(Nav::Home) => {
                        target = (0..len).find(|&i| !is_disabled(i));
                    }
                    Some(Nav::End) => {
                        target = (0..len).rev().find(|&i| !is_disabled(i));
                    }
                    Some(Nav::Next) if props.roving.wrap => {
                        let Some(current) = current else {
                            return;
                        };
                        for step in 1..=len {
                            let idx = (current + step) % len;
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    }
                    Some(Nav::Prev) if props.roving.wrap => {
                        let Some(current) = current else {
                            return;
                        };
                        for step in 1..=len {
                            let idx = (current + len - (step % len)) % len;
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    }
                    Some(Nav::Next) => {
                        let Some(current) = current else {
                            return;
                        };
                        target = ((current + 1)..len).find(|&i| !is_disabled(i));
                    }
                    Some(Nav::Prev) => {
                        let Some(current) = current else {
                            return;
                        };
                        if current > 0 {
                            target = (0..current).rev().find(|&i| !is_disabled(i));
                        }
                    }
                    None => {}
                }

                let key_to_ascii = |key: fret_core::KeyCode| -> Option<char> {
                    use fret_core::KeyCode;
                    Some(match key {
                        KeyCode::KeyA => 'a',
                        KeyCode::KeyB => 'b',
                        KeyCode::KeyC => 'c',
                        KeyCode::KeyD => 'd',
                        KeyCode::KeyE => 'e',
                        KeyCode::KeyF => 'f',
                        KeyCode::KeyG => 'g',
                        KeyCode::KeyH => 'h',
                        KeyCode::KeyI => 'i',
                        KeyCode::KeyJ => 'j',
                        KeyCode::KeyK => 'k',
                        KeyCode::KeyL => 'l',
                        KeyCode::KeyM => 'm',
                        KeyCode::KeyN => 'n',
                        KeyCode::KeyO => 'o',
                        KeyCode::KeyP => 'p',
                        KeyCode::KeyQ => 'q',
                        KeyCode::KeyR => 'r',
                        KeyCode::KeyS => 's',
                        KeyCode::KeyT => 't',
                        KeyCode::KeyU => 'u',
                        KeyCode::KeyV => 'v',
                        KeyCode::KeyW => 'w',
                        KeyCode::KeyX => 'x',
                        KeyCode::KeyY => 'y',
                        KeyCode::KeyZ => 'z',
                        KeyCode::Digit0 => '0',
                        KeyCode::Digit1 => '1',
                        KeyCode::Digit2 => '2',
                        KeyCode::Digit3 => '3',
                        KeyCode::Digit4 => '4',
                        KeyCode::Digit5 => '5',
                        KeyCode::Digit6 => '6',
                        KeyCode::Digit7 => '7',
                        KeyCode::Digit8 => '8',
                        KeyCode::Digit9 => '9',
                        _ => return None,
                    })
                };

                if target.is_none()
                    && let Some(ch) = key_to_ascii(*key)
                {
                    let hook = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::action::RovingActionHooks::default,
                        |hooks| hooks.on_typeahead.clone(),
                    );

                    if let Some(h) = hook {
                        let tick = cx.app.tick_id().0;
                        let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                        target = h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: self.element,
                            },
                            crate::action::RovingTypeaheadCx {
                                input: ch,
                                current,
                                len,
                                disabled: props.roving.disabled.clone(),
                                wrap: props.roving.wrap,
                                tick,
                            },
                        );
                    }
                }

                let Some(target) = target else {
                    return;
                };
                if current.is_some_and(|current| target == current) {
                    return;
                }

                cx.request_focus(cx.children[target]);

                let hook = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::action::RovingActionHooks::default,
                    |hooks| hooks.on_active_change.clone(),
                );

                if let Some(h) = hook {
                    let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                    h(
                        &mut host,
                        action::ActionCx {
                            window,
                            target: self.element,
                        },
                        target,
                    );
                }

                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }

        if is_text_input
            && !cx.stop_propagation
            && let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = event
            && cx.focus == Some(cx.node)
            && try_key_hook(cx, *key, *modifiers, *repeat)
        {}
    }
}
