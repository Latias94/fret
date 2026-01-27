use super::ElementHostWidget;
use crate::declarative::prelude::*;

const SCROLL_CONSUMED_EPS: f32 = 0.001;

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
            let (consumed, needs_visible_range_rerender) = crate::elements::with_element_state(
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
                    if viewport.0 <= 0.0 || props.len == 0 {
                        return (false, false);
                    }

                    let prev = props.scroll_handle.offset();
                    let prev_offset = match axis {
                        fret_core::Axis::Vertical => prev.y,
                        fret_core::Axis::Horizontal => prev.x,
                    };
                    let offset = state.metrics.clamp_offset(prev_offset, viewport);

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
                        let visible_range = state.metrics.visible_range(next, viewport, 0);
                        let needs_visible_range_rerender =
                            visible_range.is_some_and(|visible| match state.render_window_range {
                                None => visible.count > 0,
                                Some(rendered) => {
                                    if rendered.count == 0 {
                                        visible.count > 0
                                    } else {
                                        let rendered_start =
                                            rendered.start_index.saturating_sub(rendered.overscan);
                                        let rendered_end = (rendered.end_index + rendered.overscan)
                                            .min(rendered.count.saturating_sub(1));
                                        visible.start_index < rendered_start
                                            || visible.end_index > rendered_end
                                    }
                                }
                            });

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
                        (true, needs_visible_range_rerender)
                    } else {
                        (false, false)
                    }
                },
            );

            if consumed {
                let inv = Invalidation::HitTestOnly;
                super::invalidate_scroll_handle_bindings(
                    cx,
                    window,
                    props.scroll_handle.base_handle().binding_key(),
                    inv,
                );
                // VirtualList scrolling is applied via a children-only render transform, so
                // hit-testing must be invalidated to refresh coordinate mapping under the updated
                // offset. This does not force a layout pass.
                cx.invalidate_self(inv);
                if needs_visible_range_rerender {
                    let retained_host = crate::elements::with_window_state(
                        &mut *cx.app,
                        window,
                        |window_state| {
                            let retained = window_state
                                .has_state::<crate::windowed_surface_host::RetainedVirtualListHostMarker>(
                                    this.element,
                                );
                            if retained {
                                window_state
                                    .mark_retained_virtual_list_needs_reconcile(this.element);
                            }
                            retained
                        },
                    );

                    if !retained_host {
                        cx.notify();
                    }
                }
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
