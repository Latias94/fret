use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_selectable_text<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    _props: crate::element::SelectableTextProps,
    event: &Event,
) {
    match event {
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
