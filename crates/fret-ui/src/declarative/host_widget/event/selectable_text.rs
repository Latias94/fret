use super::ElementHostWidget;
use crate::declarative::prelude::*;

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
                    Some("text.select_all")
                }
                fret_core::KeyCode::KeyC if modifiers.ctrl || modifiers.meta => Some("text.copy"),
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
            cx.stop_propagation();
        }
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
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
                .and_then(|blob| Some(cx.services.hit_test_point(blob, local)));

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::SelectableTextState::default,
                |state| {
                    state.dragging = true;
                    if let Some(hit) = hit {
                        state.caret = hit.index;
                        state.affinity = hit.affinity;
                        if !modifiers.shift {
                            state.selection_anchor = state.caret;
                        }
                    } else {
                        state.caret = 0;
                        state.affinity = fret_core::CaretAffinity::Downstream;
                        if !modifiers.shift {
                            state.selection_anchor = 0;
                        }
                    }
                },
            );

            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
        }
        Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
            cx.set_cursor_icon(fret_core::CursorIcon::Text);
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
                .and_then(|blob| Some(cx.services.hit_test_point(blob, local)));

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
                },
            );
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
        }
        _ => {}
    }
}
