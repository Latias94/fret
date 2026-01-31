use super::ElementHostWidget;
use crate::declarative::prelude::*;

fn sync_active_text_selection<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: crate::elements::GlobalElementId,
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
    match event {
        Event::KeyDown {
            key,
            modifiers,
            repeat: _,
        } => {
            if cx.focus != Some(cx.node) {
                return;
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
                    sync_active_text_selection(&mut *cx.app, window, this.element);
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
                    sync_active_text_selection(&mut *cx.app, window, this.element);
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
                cx.app.push_effect(Effect::ClipboardSetText {
                    text: sel.to_string(),
                });
            }

            if needs_repaint {
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }

            sync_active_text_selection(&mut *cx.app, window, this.element);
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
            sync_active_text_selection(&mut *cx.app, window, this.element);
            cx.stop_propagation();
        }
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            click_count,
            ..
        }) => {
            if *button != fret_core::MouseButton::Left {
                return;
            }
            cx.request_focus(cx.node);
            cx.capture_pointer(cx.node);
            cx.set_cursor_icon(fret_core::CursorIcon::Text);

            let local = fret_core::Point::new(
                fret_core::Px(position.x.0 - cx.bounds.origin.x.0),
                fret_core::Px(position.y.0 - cx.bounds.origin.y.0),
            );

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
                    state.dragging = true;
                    state.last_pointer_pos = Some(*position);
                    state.preferred_x = None;

                    let hit = hit.unwrap_or(fret_core::HitTestResult {
                        index: 0,
                        affinity: fret_core::CaretAffinity::Downstream,
                    });

                    let idx = hit.index.min(props.rich.text.len());
                    let (anchor, caret) = match *click_count {
                        2 => select_word(&props.rich.text, idx, boundary_mode),
                        3 => select_line(&props.rich.text, idx),
                        _ => {
                            let caret = crate::text_edit::utf8::clamp_to_grapheme_boundary(
                                &props.rich.text,
                                idx,
                            );
                            let anchor = if modifiers.shift {
                                state.selection_anchor
                            } else {
                                caret
                            };
                            (anchor, caret)
                        }
                    };

                    state.selection_anchor = anchor;
                    state.caret = caret;
                    state.affinity = hit.affinity;
                },
            );

            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            sync_active_text_selection(&mut *cx.app, window, this.element);
            cx.stop_propagation();
        }
        Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
            cx.set_cursor_icon(fret_core::CursorIcon::Text);
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

            let local = fret_core::Point::new(
                fret_core::Px(position.x.0 - cx.bounds.origin.x.0),
                fret_core::Px(position.y.0 - cx.bounds.origin.y.0),
            );

            let hit = this
                .text_cache
                .blob
                .map(|blob| cx.services.hit_test_point(blob, local));

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
                sync_active_text_selection(&mut *cx.app, window, this.element);
            }

            cx.stop_propagation();
        }
        Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
            if *button != fret_core::MouseButton::Left {
                return;
            }
            if cx.captured == Some(cx.node) {
                cx.release_pointer_capture();
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
                },
            );
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            sync_active_text_selection(&mut *cx.app, window, this.element);
            cx.stop_propagation();
        }
        _ => {}
    }
}
