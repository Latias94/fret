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
                fret_core::KeyCode::Home => Some(if modifiers.shift {
                    "text.select_home"
                } else {
                    "text.move_home"
                }),
                fret_core::KeyCode::End => Some(if modifiers.shift {
                    "text.select_end"
                } else {
                    "text.move_end"
                }),
                _ => None,
            };

            let Some(command) = command else {
                return;
            };

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
                    state.selection_anchor = *anchor as usize;
                    state.caret = *focus as usize;
                    state.affinity = fret_core::CaretAffinity::Downstream;
                    state.dragging = false;
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

            fn char_at(text: &str, idx: usize) -> Option<char> {
                if idx >= text.len() {
                    return None;
                }
                let next = crate::text_edit::utf8::next_char_boundary(text, idx);
                text.get(idx..next).and_then(|s| s.chars().next())
            }

            fn select_word(text: &str, idx: usize) -> (usize, usize) {
                if text.is_empty() {
                    return (0, 0);
                }
                let mut idx = crate::text_edit::utf8::clamp_to_char_boundary(text, idx);
                if idx >= text.len() {
                    idx = crate::text_edit::utf8::prev_char_boundary(text, idx);
                }

                // Hit-testing can return a caret stop at the end of the word. If the current char
                // is whitespace but the previous is a word char, prefer selecting the previous word.
                if char_at(text, idx).is_some_and(|c| c.is_whitespace()) && idx > 0 {
                    let prev = crate::text_edit::utf8::prev_char_boundary(text, idx);
                    if char_at(text, prev).is_some_and(crate::text_edit::utf8::is_word_char) {
                        idx = prev;
                    }
                }

                let Some(ch) = char_at(text, idx) else {
                    return (0, 0);
                };

                if crate::text_edit::utf8::is_word_char(ch) {
                    (
                        crate::text_edit::utf8::move_word_left(text, idx),
                        crate::text_edit::utf8::move_word_right(text, idx),
                    )
                } else if ch.is_whitespace() {
                    let mut start = idx;
                    while start > 0 {
                        let prev = crate::text_edit::utf8::prev_char_boundary(text, start);
                        if char_at(text, prev).is_some_and(|c| c.is_whitespace()) {
                            start = prev;
                        } else {
                            break;
                        }
                    }
                    let mut end = crate::text_edit::utf8::next_char_boundary(text, idx);
                    while end < text.len() {
                        if char_at(text, end).is_some_and(|c| c.is_whitespace()) {
                            end = crate::text_edit::utf8::next_char_boundary(text, end);
                        } else {
                            break;
                        }
                    }
                    (start, end)
                } else {
                    (idx, crate::text_edit::utf8::next_char_boundary(text, idx))
                }
            }

            fn select_line(text: &str, idx: usize) -> (usize, usize) {
                if text.is_empty() {
                    return (0, 0);
                }
                let idx = crate::text_edit::utf8::clamp_to_char_boundary(text, idx).min(text.len());
                let start = text[..idx]
                    .rfind('\n')
                    .map(|i| (i + 1).min(text.len()))
                    .unwrap_or(0);
                let end = text[idx..]
                    .find('\n')
                    .map(|i| (idx + i).min(text.len()))
                    .unwrap_or(text.len());
                (start, end)
            }

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| {
                    state.dragging = true;
                    state.last_pointer_pos = Some(*position);

                    let hit = hit.unwrap_or(fret_core::HitTestResult {
                        index: 0,
                        affinity: fret_core::CaretAffinity::Downstream,
                    });

                    let idx = hit.index.min(props.rich.text.len());
                    let (anchor, caret) = match *click_count {
                        2 => select_word(&props.rich.text, idx),
                        3 => select_line(&props.rich.text, idx),
                        _ => {
                            let caret = idx;
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
                        state.caret = hit.index;
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
