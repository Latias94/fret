use super::ElementHostWidget;
use crate::declarative::mount::node_for_element_in_window_frame;
use crate::declarative::paint_helpers::scrollbar_thumb_rect;
use crate::declarative::paint_helpers::scrollbar_thumb_rect_horizontal;
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
            let is_horizontal = cx.bounds.size.width.0 > cx.bounds.size.height.0;
            let prev = handle.offset();
            if is_horizontal {
                let dx = if delta.x.0.abs() > 0.01 {
                    delta.x
                } else {
                    delta.y
                };
                handle.set_offset(Point::new(Px(prev.x.0 - dx.0), prev.y));
            } else {
                handle.set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));
            }

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
            let is_horizontal = bounds.size.width.0 > bounds.size.height.0;

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::ScrollbarState::default,
                |state| {
                    let (viewport, content, max_offset) = if is_horizontal {
                        let viewport = Px(handle.viewport_size().width.0.max(0.0));
                        let content = Px(handle.content_size().width.0.max(0.0));
                        (viewport, content, Px((content.0 - viewport.0).max(0.0)))
                    } else {
                        let viewport = Px(handle.viewport_size().height.0.max(0.0));
                        let content = Px(handle.content_size().height.0.max(0.0));
                        (viewport, content, Px((content.0 - viewport.0).max(0.0)))
                    };

                    let hovered = bounds.contains(position);
                    if state.hovered != hovered && !state.dragging_thumb {
                        state.hovered = hovered;
                        needs_paint = true;
                    }

                    if state.dragging_thumb
                        && max_offset.0 > 0.0
                        && let Some(thumb) = (if is_horizontal {
                            scrollbar_thumb_rect_horizontal(
                                bounds,
                                viewport,
                                content,
                                state.drag_start_offset_y,
                            )
                        } else {
                            scrollbar_thumb_rect(
                                bounds,
                                viewport,
                                content,
                                state.drag_start_offset_y,
                            )
                        })
                    {
                        if is_horizontal {
                            let max_thumb_x = (bounds.size.width.0 - thumb.size.width.0).max(0.0);
                            if max_thumb_x > 0.0 {
                                let delta_x = position.x.0 - state.drag_start_pointer_y.0;
                                let scale = max_offset.0 / max_thumb_x;
                                let next =
                                    Px((state.drag_start_offset_y.0 + delta_x * scale).max(0.0));
                                let next = Px(next.0.min(max_offset.0));
                                if (handle.offset().x.0 - next.0).abs() > 0.01 {
                                    let prev = handle.offset();
                                    handle.set_offset(Point::new(next, prev.y));
                                    needs_layout = true;
                                    needs_paint = true;
                                }
                                state.hovered = true;
                            }
                        } else {
                            let max_thumb_y = (bounds.size.height.0 - thumb.size.height.0).max(0.0);
                            if max_thumb_y > 0.0 {
                                let delta_y = position.y.0 - state.drag_start_pointer_y.0;
                                let scale = max_offset.0 / max_thumb_y;
                                let next =
                                    Px((state.drag_start_offset_y.0 + delta_y * scale).max(0.0));
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
            let is_horizontal = bounds.size.width.0 > bounds.size.height.0;

            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::ScrollbarState::default,
                |state| {
                    let (viewport, content, max_offset) = if is_horizontal {
                        let viewport = Px(handle.viewport_size().width.0.max(0.0));
                        let content = Px(handle.content_size().width.0.max(0.0));
                        (viewport, content, Px((content.0 - viewport.0).max(0.0)))
                    } else {
                        let viewport = Px(handle.viewport_size().height.0.max(0.0));
                        let content = Px(handle.content_size().height.0.max(0.0));
                        (viewport, content, Px((content.0 - viewport.0).max(0.0)))
                    };

                    if max_offset.0 <= 0.0 {
                        return;
                    }

                    let current_offset = if is_horizontal {
                        handle.offset().x
                    } else {
                        handle.offset().y
                    };
                    let thumb = if is_horizontal {
                        scrollbar_thumb_rect_horizontal(bounds, viewport, content, current_offset)
                    } else {
                        scrollbar_thumb_rect(bounds, viewport, content, current_offset)
                    };
                    let Some(thumb) = thumb else {
                        return;
                    };

                    did_handle = true;
                    state.hovered = true;

                    if thumb.contains(position) {
                        state.dragging_thumb = true;
                        state.drag_start_pointer_y = if is_horizontal {
                            position.x
                        } else {
                            position.y
                        };
                        state.drag_start_offset_y = current_offset;
                        did_start_drag = true;
                    } else if bounds.contains(position) {
                        // Page to the click position (center the thumb on the pointer).
                        if is_horizontal {
                            let max_thumb_x = (bounds.size.width.0 - thumb.size.width.0).max(0.0);
                            if max_thumb_x > 0.0 {
                                let click_x = (position.x.0 - bounds.origin.x.0)
                                    .clamp(0.0, bounds.size.width.0);
                                let thumb_left =
                                    (click_x - thumb.size.width.0 * 0.5).clamp(0.0, max_thumb_x);
                                let t = thumb_left / max_thumb_x;
                                let next = Px((max_offset.0 * t).clamp(0.0, max_offset.0));
                                let prev = handle.offset();
                                handle.set_offset(Point::new(next, prev.y));
                                did_change_offset = true;
                            }
                        } else {
                            let max_thumb_y = (bounds.size.height.0 - thumb.size.height.0).max(0.0);
                            if max_thumb_y > 0.0 {
                                let click_y = (position.y.0 - bounds.origin.y.0)
                                    .clamp(0.0, bounds.size.height.0);
                                let thumb_top =
                                    (click_y - thumb.size.height.0 * 0.5).clamp(0.0, max_thumb_y);
                                let t = thumb_top / max_thumb_y;
                                let next = Px((max_offset.0 * t).clamp(0.0, max_offset.0));
                                let prev = handle.offset();
                                handle.set_offset(Point::new(prev.x, next));
                                did_change_offset = true;
                            }
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
