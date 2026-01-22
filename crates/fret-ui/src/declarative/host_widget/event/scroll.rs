use super::ElementHostWidget;
use crate::declarative::prelude::*;

const SCROLL_CONSUMED_EPS: f32 = 0.001;

fn range_contains_visible_range(window: crate::virtual_list::VirtualRange, visible: crate::virtual_list::VirtualRange) -> bool {
    if window.count != visible.count {
        return false;
    }
    if window.count == 0 {
        return true;
    }
    let window_start = window.start_index.saturating_sub(window.overscan);
    let window_end = (window.end_index + window.overscan).min(window.count.saturating_sub(1));
    window_start <= visible.start_index && window_end >= visible.end_index
}

pub(super) fn handle_virtual_list<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::VirtualListProps,
    event: &Event,
) -> bool {
    let Event::Pointer(pe) = event else {
        return true;
    };
    match pe {
        fret_core::PointerEvent::Wheel {
            delta, modifiers, ..
        } => {
            let scroll_invalidation = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::VirtualListState::default,
                |state| {
                    let axis = props.axis;
                    state.metrics.ensure_with_mode(
                        props.measure_mode,
                        props.len,
                        props.estimate_row_height,
                        props.gap,
                        props.scroll_margin,
                    );
                    let viewport = match axis {
                        fret_core::Axis::Vertical => Px(state.viewport_h.0.max(0.0)),
                        fret_core::Axis::Horizontal => Px(state.viewport_w.0.max(0.0)),
                    };

                    let prev = props.scroll_handle.offset();
                    let prev_offset = match axis {
                        fret_core::Axis::Vertical => prev.y,
                        fret_core::Axis::Horizontal => prev.x,
                    };
                    let offset = state.metrics.clamp_offset(prev_offset, viewport);
                    let prev_range = state.metrics.visible_range(offset, viewport, props.overscan);
                    let prev_window = state.window_range.or(prev_range);

                    let delta = match axis {
                        fret_core::Axis::Vertical => delta.y,
                        fret_core::Axis::Horizontal => {
                            if modifiers.shift {
                                delta.y
                            } else {
                                delta.x
                            }
                        }
                    };
                    let next = state.metrics.clamp_offset(Px(offset.0 - delta.0), viewport);
                    if (prev_offset.0 - next.0).abs() > SCROLL_CONSUMED_EPS {
                        match axis {
                            fret_core::Axis::Vertical => {
                                props
                                    .scroll_handle
                                    .set_offset(fret_core::Point::new(prev.x, next));
                            }
                            fret_core::Axis::Horizontal => {
                                props
                                    .scroll_handle
                                    .set_offset(fret_core::Point::new(next, prev.y));
                            }
                        }
                        let next_range =
                            state.metrics.visible_range(next, viewport, props.overscan);
                        // Keep wheel scroll transform-only while the viewport is still covered by
                        // the previous rendered window (overscan-inclusive). Only request a layout
                        // rerender once the viewport would leave the rendered window.
                        let window_changed = match (prev_window, state.metrics.visible_range(next, viewport, 0)) {
                            (Some(window), Some(visible)) => !range_contains_visible_range(window, visible),
                            _ => prev_range != next_range,
                        };
                        Some(if window_changed {
                            Invalidation::Layout
                        } else {
                            Invalidation::HitTestOnly
                        })
                    } else {
                        None
                    }
                },
            );
            if let Some(inv) = scroll_invalidation {
                super::invalidate_scroll_handle_bindings(
                    cx,
                    window,
                    props.scroll_handle.base_handle().binding_key(),
                    inv,
                );
                cx.invalidate_self(inv);
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        fret_core::PointerEvent::Down { button, .. } => {
            if *button == MouseButton::Left {
                cx.request_focus(cx.node);
            }
        }
        _ => {}
    }
    true
}

pub(super) fn handle_scroll<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::ScrollProps,
    event: &Event,
) -> bool {
    let Event::Pointer(pe) = event else {
        return true;
    };
    if let fret_core::PointerEvent::Wheel {
        delta, modifiers, ..
    } = pe
    {
        let (delta_x, delta_y) = match props.axis {
            crate::element::ScrollAxis::X => {
                // Trackpads often report diagonal deltas. If the gesture is primarily vertical and
                // Shift is not held, let the parent (typically a Y-scroll) handle it.
                if !modifiers.shift && delta.y.0.abs() > delta.x.0.abs() {
                    (Px(0.0), Px(0.0))
                } else if modifiers.shift && delta.x.0.abs() < 0.01 {
                    (delta.y, Px(0.0))
                } else {
                    (delta.x, Px(0.0))
                }
            }
            crate::element::ScrollAxis::Y => (Px(0.0), delta.y),
            crate::element::ScrollAxis::Both => (delta.x, delta.y),
        };

        let consumed = if let Some(handle) = props.scroll_handle.as_ref() {
            let prev = handle.offset();
            let desired = Point::new(Px(prev.x.0 - delta_x.0), Px(prev.y.0 - delta_y.0));
            handle.set_offset(desired);
            let next = handle.offset();
            if std::env::var("FRET_DEBUG_SCROLL_WHEEL")
                .ok()
                .is_some_and(|v| v == "1")
            {
                let max = handle.max_offset();
                eprintln!(
                    "scroll wheel element={:?} handle_key={} axis={:?} delta=({:.3},{:.3}) prev=({:.3},{:.3}) next=({:.3},{:.3}) max=({:.3},{:.3})",
                    this.element,
                    handle.binding_key(),
                    props.axis,
                    delta_x.0,
                    delta_y.0,
                    prev.x.0,
                    prev.y.0,
                    next.x.0,
                    next.y.0,
                    max.x.0,
                    max.y.0,
                );
            }
            (prev.x.0 - next.x.0).abs() > SCROLL_CONSUMED_EPS
                || (prev.y.0 - next.y.0).abs() > SCROLL_CONSUMED_EPS
        } else {
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::ScrollState::default,
                |state| {
                    let prev = state.scroll_handle.offset();
                    let desired = Point::new(Px(prev.x.0 - delta_x.0), Px(prev.y.0 - delta_y.0));
                    state.scroll_handle.set_offset(desired);
                    let next = state.scroll_handle.offset();
                    (prev.x.0 - next.x.0).abs() > SCROLL_CONSUMED_EPS
                        || (prev.y.0 - next.y.0).abs() > SCROLL_CONSUMED_EPS
                },
            )
        };

        if consumed {
            if let Some(handle) = props.scroll_handle.as_ref() {
                super::invalidate_scroll_handle_bindings(
                    cx,
                    window,
                    handle.binding_key(),
                    Invalidation::HitTestOnly,
                );
            }
            cx.invalidate_self(Invalidation::HitTestOnly);
            cx.request_redraw();
            cx.stop_propagation();
        }
    }
    true
}
