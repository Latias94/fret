use super::ElementHostWidget;
use crate::declarative::prelude::*;

fn interactive_span_at_index(
    spans: &[crate::element::SelectableTextInteractiveSpan],
    idx: usize,
) -> Option<&crate::element::SelectableTextInteractiveSpan> {
    if spans.is_empty() {
        return None;
    }
    spans
        .iter()
        .find(|s| s.range.contains(&idx) || (idx > 0 && s.range.contains(&(idx - 1))))
}

fn sync_active_text_selection<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: crate::elements::GlobalElementId,
    node: NodeId,
) {
    let (anchor, caret) = crate::elements::with_element_state(
        app,
        window,
        element,
        crate::element::SelectableTextState::default,
        |state| (state.selection_anchor, state.caret),
    );

    crate::elements::with_window_state(app, window, |window_state| {
        if anchor != caret {
            let Some(entry) = window_state.node_entry(element) else {
                return;
            };
            window_state.set_active_text_selection(Some(crate::elements::ActiveTextSelection {
                root: entry.root,
                element,
                node,
            }));
            return;
        }

        if window_state
            .active_text_selection()
            .is_some_and(|selection| selection.element == element)
        {
            window_state.set_active_text_selection(None);
        }
    });
}

pub(super) fn handle_selectable_text<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::SelectableTextProps,
    event: &Event,
) {
    let text_local_from_window_position =
        |this: &ElementHostWidget,
         cx: &mut EventCx<'_, H>,
         props: &crate::element::SelectableTextProps,
         position: fret_core::Point| {
            let element_local = fret_core::Point::new(
                fret_core::Px(position.x.0 - cx.bounds.origin.x.0),
                fret_core::Px(position.y.0 - cx.bounds.origin.y.0),
            );

            let Some(blob) = this.text_cache.blob else {
                return element_local;
            };
            let Some(metrics) = this.text_cache.metrics else {
                return element_local;
            };

            let (pad_top, pad_bottom) =
                crate::text::coords::clamp_text_ink_overflow_padding_to_bounds(
                    metrics.size.height,
                    cx.bounds.size.height,
                    this.text_cache.ink_pad_top,
                    this.text_cache.ink_pad_bottom,
                );
            let bounds = fret_core::Rect::new(
                fret_core::Point::new(
                    cx.bounds.origin.x,
                    fret_core::Px(cx.bounds.origin.y.0 + pad_top.0),
                ),
                fret_core::Size::new(
                    cx.bounds.size.width,
                    fret_core::Px((cx.bounds.size.height.0 - pad_top.0 - pad_bottom.0).max(0.0)),
                ),
            );

            let inherited_text_style =
                crate::declarative::frame::inherited_text_style_for_node(cx.app, window, cx.node);
            let vertical_placement = props
                .resolved_text_style_with_inherited(
                    cx.theme().snapshot(),
                    inherited_text_style.as_ref(),
                )
                .vertical_placement;
            let (mapping, _, _) =
                crate::text::coords::compute_text_box_mapping_for_vertical_placement(
                    cx.services.text(),
                    blob,
                    bounds,
                    metrics,
                    vertical_placement,
                );

            mapping.window_to_text_local(position)
        };

    // Keep the stored selection model stable even if the underlying text changes or external
    // surfaces (a11y, platform input) publish indices that are no longer valid.
    crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::element::SelectableTextState::default,
        |state| {
            crate::text_edit::utf8::clamp_selection_to_grapheme_boundaries(
                &props.rich.text,
                &mut state.selection_anchor,
                &mut state.caret,
            );
        },
    );

    match event {
        Event::KeyDown {
            key,
            modifiers,
            repeat: _,
        } => {
            if cx.focus != Some(cx.node) {
                return;
            }

            if matches!(
                key,
                fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter
            ) && !props.interactive_spans.is_empty()
                && !modifiers.ctrl
                && !modifiers.alt
                && !modifiers.meta
            {
                let (anchor, caret) = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    this.element,
                    crate::element::SelectableTextState::default,
                    |state| (state.selection_anchor, state.caret),
                );

                if anchor == caret
                    && let Some(span) = interactive_span_at_index(&props.interactive_spans, caret)
                {
                    let activation = crate::action::SelectableTextSpanActivation {
                        tag: span.tag.clone(),
                        range: span.range.clone(),
                    };

                    let handler = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        this.element,
                        crate::action::SelectableTextActionHooks::default,
                        |hooks| hooks.on_activate_span.clone(),
                    );

                    if let Some(handler) = handler {
                        struct SelectableTextActivateSpanHookHost<'a, H: UiHost> {
                            app: &'a mut H,
                            notify_requested: &'a mut bool,
                            notify_requested_location:
                                &'a mut Option<crate::widget::UiSourceLocation>,
                        }

                        impl<H: UiHost> crate::action::UiActionHost for SelectableTextActivateSpanHookHost<'_, H> {
                            fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                                self.app.models_mut()
                            }

                            fn push_effect(&mut self, effect: fret_runtime::Effect) {
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

                            fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
                                self.app.next_share_sheet_token()
                            }

                            fn record_transient_event(
                                &mut self,
                                cx: crate::action::ActionCx,
                                key: u64,
                            ) {
                                crate::elements::record_transient_event(
                                    &mut *self.app,
                                    cx.window,
                                    cx.target,
                                    key,
                                );
                            }

                            #[track_caller]
                            fn notify(&mut self, _cx: crate::action::ActionCx) {
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

                        let mut host = SelectableTextActivateSpanHookHost {
                            app: &mut *cx.app,
                            notify_requested: &mut cx.notify_requested,
                            notify_requested_location: &mut cx.notify_requested_location,
                        };

                        handler(
                            &mut host,
                            crate::action::ActionCx {
                                window,
                                target: this.element,
                            },
                            ActivateReason::Keyboard,
                            activation,
                        );

                        cx.stop_propagation();
                        return;
                    }
                }
            }

            let handle_visual_line_home_end =
                |this: &mut ElementHostWidget,
                 cx: &mut EventCx<'_, H>,
                 extend: bool,
                 at_line_end: bool| {
                    let Some(blob) = this.text_cache.blob else {
                        return;
                    };

                    let (caret, affinity) = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        this.element,
                        crate::element::SelectableTextState::default,
                        |state| (state.caret, state.affinity),
                    );

                    let caret =
                        crate::text_edit::utf8::clamp_to_grapheme_boundary(&props.rich.text, caret);
                    let caret_rect = cx.services.caret_rect(blob, caret, affinity);
                    let y = fret_core::Px(caret_rect.origin.y.0 + caret_rect.size.height.0 * 0.5);
                    let x = if at_line_end {
                        fret_core::Px(1.0e6)
                    } else {
                        fret_core::Px(-1.0e6)
                    };

                    let hit = cx
                        .services
                        .hit_test_point(blob, fret_core::Point::new(x, y));
                    let next = crate::text_edit::utf8::clamp_to_grapheme_boundary(
                        &props.rich.text,
                        hit.index.min(props.rich.text.len()),
                    );

                    crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        this.element,
                        crate::element::SelectableTextState::default,
                        |state| {
                            state.caret = next;
                            if !extend {
                                state.selection_anchor = next;
                            }
                            state.affinity = hit.affinity;
                            state.dragging = false;
                            state.preferred_x = None;
                        },
                    );

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    sync_active_text_selection(&mut *cx.app, window, this.element, cx.node);
                    cx.stop_propagation();
                };

            let handle_visual_move_vertical =
                |this: &mut ElementHostWidget,
                 cx: &mut EventCx<'_, H>,
                 extend: bool,
                 down: bool| {
                    let Some(blob) = this.text_cache.blob else {
                        return;
                    };

                    let (caret, anchor, affinity, preferred_x) =
                        crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            this.element,
                            crate::element::SelectableTextState::default,
                            |state| {
                                (
                                    state.caret,
                                    state.selection_anchor,
                                    state.affinity,
                                    state.preferred_x,
                                )
                            },
                        );

                    let caret =
                        crate::text_edit::utf8::clamp_to_grapheme_boundary(&props.rich.text, caret);
                    let caret_rect = cx.services.caret_rect(blob, caret, affinity);
                    let x = preferred_x.unwrap_or(caret_rect.origin.x);
                    let y = if down {
                        fret_core::Px(caret_rect.origin.y.0 + caret_rect.size.height.0 + 1.0)
                    } else {
                        fret_core::Px(caret_rect.origin.y.0 - 1.0)
                    };

                    let hit = cx
                        .services
                        .hit_test_point(blob, fret_core::Point::new(x, y));
                    let next = crate::text_edit::utf8::clamp_to_grapheme_boundary(
                        &props.rich.text,
                        hit.index.min(props.rich.text.len()),
                    );
                    let next_anchor = if extend { anchor } else { next };

                    crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        this.element,
                        crate::element::SelectableTextState::default,
                        |state| {
                            state.selection_anchor = next_anchor;
                            state.caret = next;
                            state.affinity = hit.affinity;
                            state.dragging = false;
                            state.preferred_x = Some(x);
                        },
                    );

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    sync_active_text_selection(&mut *cx.app, window, this.element, cx.node);
                    cx.stop_propagation();
                };

            match *key {
                fret_core::KeyCode::ArrowUp => {
                    handle_visual_move_vertical(this, cx, modifiers.shift, false);
                    return;
                }
                fret_core::KeyCode::ArrowDown => {
                    handle_visual_move_vertical(this, cx, modifiers.shift, true);
                    return;
                }
                fret_core::KeyCode::Home => {
                    handle_visual_line_home_end(this, cx, modifiers.shift, false);
                    return;
                }
                fret_core::KeyCode::End => {
                    handle_visual_line_home_end(this, cx, modifiers.shift, true);
                    return;
                }
                _ => {}
            }

            let command: Option<&'static str> = match *key {
                fret_core::KeyCode::KeyA if modifiers.ctrl || modifiers.meta => {
                    Some("edit.select_all")
                }
                fret_core::KeyCode::KeyC if modifiers.ctrl || modifiers.meta => Some("edit.copy"),
                fret_core::KeyCode::ArrowLeft => {
                    let word = modifiers.ctrl || modifiers.alt;
                    Some(match (modifiers.shift, word) {
                        (true, true) => "text.select_word_left",
                        (true, false) => "text.select_left",
                        (false, true) => "text.move_word_left",
                        (false, false) => "text.move_left",
                    })
                }
                fret_core::KeyCode::ArrowRight => {
                    let word = modifiers.ctrl || modifiers.alt;
                    Some(match (modifiers.shift, word) {
                        (true, true) => "text.select_word_right",
                        (true, false) => "text.select_right",
                        (false, true) => "text.move_word_right",
                        (false, false) => "text.move_right",
                    })
                }
                _ => None,
            };

            let Some(command) = command else {
                return;
            };

            if command != "edit.copy" {
                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    this.element,
                    crate::element::SelectableTextState::default,
                    |state| {
                        state.preferred_x = None;
                    },
                );
            }

            let (handled, copy_range, needs_repaint) = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| {
                    let outcome = crate::text_surface::apply_selectable_text_command(
                        &props.rich.text,
                        state,
                        command,
                        cx.input_ctx.text_boundary_mode,
                    );
                    match outcome {
                        crate::text_surface::SelectableTextCommandOutcome::Handled {
                            needs_repaint,
                            copy_range,
                        } => (true, copy_range, needs_repaint),
                        crate::text_surface::SelectableTextCommandOutcome::NotHandled => {
                            (false, None, false)
                        }
                    }
                },
            );

            if !handled {
                return;
            }

            if let Some((start, end)) = copy_range
                && end <= props.rich.text.len()
                && let Some(sel) = props.rich.text.get(start..end)
            {
                let token = cx.app.next_clipboard_token();
                cx.app.push_effect(Effect::ClipboardWriteText {
                    window,
                    token,
                    text: sel.to_string(),
                });
            }

            if needs_repaint {
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }

            sync_active_text_selection(&mut *cx.app, window, this.element, cx.node);
            cx.stop_propagation();
        }
        Event::SetTextSelection { anchor, focus } => {
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| {
                    state.selection_anchor = crate::text_edit::utf8::clamp_to_grapheme_boundary(
                        &props.rich.text,
                        *anchor as usize,
                    );
                    state.caret = crate::text_edit::utf8::clamp_to_grapheme_boundary(
                        &props.rich.text,
                        *focus as usize,
                    );
                    state.affinity = fret_core::CaretAffinity::Downstream;
                    state.dragging = false;
                    state.preferred_x = None;
                },
            );
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            sync_active_text_selection(&mut *cx.app, window, this.element, cx.node);
            cx.stop_propagation();
        }
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            click_count,
            pointer_type,
            ..
        }) => {
            match *button {
                fret_core::MouseButton::Left => {
                    cx.request_focus(cx.node);
                    cx.capture_pointer(cx.node);
                    cx.set_cursor_icon(fret_core::CursorIcon::Text);
                }
                fret_core::MouseButton::Right => {
                    if *pointer_type != fret_core::PointerType::Mouse {
                        return;
                    }
                    // Preserve selection for Copy/Cut/Paste context menus and ensure command
                    // availability hooks see this surface as focused (matching TextInput/TextArea).
                    cx.request_focus(cx.node);
                    cx.set_cursor_icon(fret_core::CursorIcon::Text);
                }
                _ => return,
            }

            let local = text_local_from_window_position(this, cx, &props, *position);

            let hit = this
                .text_cache
                .blob
                .map(|blob| cx.services.hit_test_point(blob, local));

            fn select_word(
                text: &str,
                idx: usize,
                mode: fret_runtime::TextBoundaryMode,
            ) -> (usize, usize) {
                crate::text_edit::utf8::select_word_range(text, idx, mode)
            }

            fn select_line(text: &str, idx: usize) -> (usize, usize) {
                crate::text_edit::utf8::select_line_range(text, idx)
            }

            let boundary_mode = cx.input_ctx.text_boundary_mode;
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| {
                    state.dragging = matches!(*button, fret_core::MouseButton::Left);
                    state.last_pointer_pos = Some(*position);
                    state.pointer_down_pos = Some(*position);
                    state.preferred_x = None;

                    let hit = hit.unwrap_or(fret_core::HitTestResult {
                        index: 0,
                        affinity: fret_core::CaretAffinity::Downstream,
                    });

                    let idx = hit.index.min(props.rich.text.len());
                    let selection_start = state.selection_anchor.min(state.caret);
                    let selection_end = state.selection_anchor.max(state.caret);
                    let caret_at_point =
                        crate::text_edit::utf8::clamp_to_grapheme_boundary(&props.rich.text, idx);

                    let (anchor, caret) = match *button {
                        fret_core::MouseButton::Right => {
                            // Preserve an existing selection when right-clicking inside it so
                            // upstream context menus keep Copy enabled.
                            if selection_start != selection_end
                                && caret_at_point >= selection_start
                                && caret_at_point <= selection_end
                            {
                                (state.selection_anchor, state.caret)
                            } else {
                                (caret_at_point, caret_at_point)
                            }
                        }
                        _ => match *click_count {
                            2 => select_word(&props.rich.text, idx, boundary_mode),
                            3 => select_line(&props.rich.text, idx),
                            _ => {
                                let anchor = if modifiers.shift {
                                    state.selection_anchor
                                } else {
                                    caret_at_point
                                };
                                (anchor, caret_at_point)
                            }
                        },
                    };

                    state.selection_anchor = anchor;
                    state.caret = caret;
                    state.affinity = hit.affinity;

                    state.pending_span_activation = None;
                    state.pending_span_click_count = *click_count;
                    if matches!(*button, fret_core::MouseButton::Left)
                        && *click_count == 1
                        && !modifiers.shift
                        && let Some(span) =
                            interactive_span_at_index(&props.interactive_spans, caret_at_point)
                    {
                        state.pending_span_activation =
                            Some(crate::action::SelectableTextSpanActivation {
                                tag: span.tag.clone(),
                                range: span.range.clone(),
                            });
                    }
                },
            );

            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            sync_active_text_selection(&mut *cx.app, window, this.element, cx.node);
            if matches!(*button, fret_core::MouseButton::Left) {
                cx.stop_propagation();
            }
        }
        Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
            let local = text_local_from_window_position(this, cx, &props, *position);

            let hit = this
                .text_cache
                .blob
                .map(|blob| cx.services.hit_test_point(blob, local));

            let cursor = hit
                .and_then(|hit| interactive_span_at_index(&props.interactive_spans, hit.index))
                .map(|_| fret_core::CursorIcon::Pointer)
                .unwrap_or(fret_core::CursorIcon::Text);
            cx.set_cursor_icon(cursor);

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| {
                    state.last_pointer_pos = Some(*position);
                },
            );
            if cx.captured != Some(cx.node) {
                return;
            }

            const ACTIVATION_DRAG_THRESHOLD_PX: f32 = 4.0;
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| {
                    if state.pending_span_activation.is_some()
                        && let Some(down) = state.pointer_down_pos
                    {
                        let dx = position.x.0 - down.x.0;
                        let dy = position.y.0 - down.y.0;
                        if (dx * dx + dy * dy)
                            > (ACTIVATION_DRAG_THRESHOLD_PX * ACTIVATION_DRAG_THRESHOLD_PX)
                        {
                            state.pending_span_activation = None;
                        }
                    }
                },
            );

            let dragging = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| state.dragging,
            );
            if !dragging {
                return;
            }

            if let Some(hit) = hit {
                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    this.element,
                    crate::element::SelectableTextState::default,
                    |state| {
                        state.caret = crate::text_edit::utf8::clamp_to_grapheme_boundary(
                            &props.rich.text,
                            hit.index,
                        );
                        state.affinity = hit.affinity;
                    },
                );
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                sync_active_text_selection(&mut *cx.app, window, this.element, cx.node);
            }

            cx.stop_propagation();
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            click_count,
            ..
        }) => {
            if *button != fret_core::MouseButton::Left {
                return;
            }
            if cx.captured == Some(cx.node) {
                cx.release_pointer_capture();
            }

            const ACTIVATION_DRAG_THRESHOLD_PX: f32 = 4.0;
            let (pending, anchor, caret, down_pos) = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| {
                    (
                        state.pending_span_activation.clone(),
                        state.selection_anchor,
                        state.caret,
                        state.pointer_down_pos,
                    )
                },
            );

            if let Some(pending) = pending
                && *click_count == 1
                && anchor == caret
                && let Some(down_pos) = down_pos
            {
                let dx = position.x.0 - down_pos.x.0;
                let dy = position.y.0 - down_pos.y.0;
                if (dx * dx + dy * dy)
                    <= (ACTIVATION_DRAG_THRESHOLD_PX * ACTIVATION_DRAG_THRESHOLD_PX)
                {
                    let local = text_local_from_window_position(this, cx, &props, *position);
                    let hit = this
                        .text_cache
                        .blob
                        .map(|blob| cx.services.hit_test_point(blob, local));
                    if let Some(hit) = hit
                        && let Some(span) =
                            interactive_span_at_index(&props.interactive_spans, hit.index)
                        && span.tag.as_ref() == pending.tag.as_ref()
                        && span.range == pending.range
                    {
                        let handler = crate::elements::with_element_state(
                            &mut *cx.app,
                            window,
                            this.element,
                            crate::action::SelectableTextActionHooks::default,
                            |hooks| hooks.on_activate_span.clone(),
                        );
                        if let Some(handler) = handler {
                            struct SelectableTextActivateSpanHookHost<'a, H: UiHost> {
                                app: &'a mut H,
                                notify_requested: &'a mut bool,
                                notify_requested_location:
                                    &'a mut Option<crate::widget::UiSourceLocation>,
                            }

                            impl<H: UiHost> crate::action::UiActionHost for SelectableTextActivateSpanHookHost<'_, H> {
                                fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                                    self.app.models_mut()
                                }

                                fn push_effect(&mut self, effect: fret_runtime::Effect) {
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

                                fn next_share_sheet_token(
                                    &mut self,
                                ) -> fret_runtime::ShareSheetToken {
                                    self.app.next_share_sheet_token()
                                }

                                fn record_transient_event(
                                    &mut self,
                                    cx: crate::action::ActionCx,
                                    key: u64,
                                ) {
                                    crate::elements::record_transient_event(
                                        &mut *self.app,
                                        cx.window,
                                        cx.target,
                                        key,
                                    );
                                }

                                #[track_caller]
                                fn notify(&mut self, _cx: crate::action::ActionCx) {
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

                            let mut host = SelectableTextActivateSpanHookHost {
                                app: &mut *cx.app,
                                notify_requested: &mut cx.notify_requested,
                                notify_requested_location: &mut cx.notify_requested_location,
                            };

                            handler(
                                &mut host,
                                crate::action::ActionCx {
                                    window,
                                    target: this.element,
                                },
                                crate::action::ActivateReason::Pointer,
                                pending,
                            );
                        }
                    }
                }
            }

            let settings = cx
                .app
                .global::<fret_runtime::TextInteractionSettings>()
                .copied()
                .unwrap_or_default();
            if settings.linux_primary_selection && cx.input_ctx.caps.clipboard.primary_text {
                let (anchor, caret) = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    this.element,
                    crate::element::SelectableTextState::default,
                    |state| (state.selection_anchor, state.caret),
                );
                let (start, end) = crate::text_edit::buffer::selection_range(anchor, caret);
                if start != end
                    && end <= props.rich.text.len()
                    && let Some(sel) = props.rich.text.get(start..end)
                {
                    cx.app.push_effect(Effect::PrimarySelectionSetText {
                        text: sel.to_string(),
                    });
                }
            }

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| {
                    state.dragging = false;
                    state.last_pointer_pos = None;
                    state.pointer_down_pos = None;
                    state.pending_span_activation = None;
                    state.pending_span_click_count = 0;
                },
            );
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            sync_active_text_selection(&mut *cx.app, window, this.element, cx.node);
            cx.stop_propagation();
        }
        _ => {}
    }
}
