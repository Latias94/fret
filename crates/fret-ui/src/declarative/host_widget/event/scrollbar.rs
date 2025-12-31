use super::ElementHostWidget;
use crate::declarative::mount::node_for_element_in_window_frame;
use crate::declarative::paint_helpers::scrollbar_thumb_rect;
use crate::declarative::prelude::*;

pub(super) fn handle_scrollbar<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::ScrollbarProps,
    event: &Event,
) -> bool {
    let Event::Pointer(pe) = event else {
        return true;
    };

    let handle = props.scroll_handle.clone();
    let scroll_target = props.scroll_target;
    match pe {
        fret_core::PointerEvent::Wheel { delta, .. } => {
            let prev = handle.offset();
            handle.set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));

            if let Some(target) = scroll_target
                && let Some(node) = node_for_element_in_window_frame(&mut *cx.app, window, target)
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
                this.element,
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
                        let max_thumb_y = (bounds.size.height.0 - thumb.size.height.0).max(0.0);
                        if max_thumb_y > 0.0 {
                            let delta_y = position.y.0 - state.drag_start_pointer_y.0;
                            let scale = max_offset.0 / max_thumb_y;
                            let next = Px((state.drag_start_offset_y.0 + delta_y * scale).max(0.0));
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
                return true;
            }

            let mut did_handle = false;
            let mut did_start_drag = false;
            let mut did_change_offset = false;
            let bounds = cx.bounds;
            let position = *position;

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::ScrollbarState::default,
                |state| {
                    let viewport_h = Px(handle.viewport_size().height.0.max(0.0));
                    let content_h = Px(handle.content_size().height.0.max(0.0));
                    let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));

                    if max_offset.0 <= 0.0 {
                        return;
                    }

                    let Some(thumb) =
                        scrollbar_thumb_rect(bounds, viewport_h, content_h, handle.offset().y)
                    else {
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
                        let max_thumb_y = (bounds.size.height.0 - thumb.size.height.0).max(0.0);
                        if max_thumb_y > 0.0 {
                            let click_y =
                                (position.y.0 - bounds.origin.y.0).clamp(0.0, bounds.size.height.0);
                            let thumb_top =
                                (click_y - thumb.size.height.0 * 0.5).clamp(0.0, max_thumb_y);
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
                return true;
            }

            let mut did_handle = false;
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
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

    true
}
