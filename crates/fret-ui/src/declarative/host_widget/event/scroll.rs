use super::ElementHostWidget;
use crate::declarative::prelude::*;
use fret_core::Modifiers;

const SCROLL_CONSUMED_EPS: f32 = 0.001;
const TOUCH_PAN_SCROLL_THRESHOLD_PX: f32 = 6.0;
static DEBUG_VLIST_WHEEL_PRINTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
const DEBUG_VLIST_WHEEL_ENV: &str = "FRET_DEBUG_SCROLL_WHEEL_VLIST";

#[derive(Debug, Default, Clone, Copy)]
struct TouchPanScrollTracking {
    pointer_id: Option<fret_core::PointerId>,
    start: Option<Point>,
    last: Option<Point>,
    panning: bool,
    captured_for_pan: bool,
}

fn touch_pan_delta_for_move<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    element: crate::GlobalElementId,
    pointer_id: fret_core::PointerId,
    position: Point,
) -> Option<Point> {
    let mut out: Option<Point> = None;
    crate::elements::with_element_state(
        &mut *cx.app,
        window,
        element,
        TouchPanScrollTracking::default,
        |st| {
            if st.pointer_id != Some(pointer_id) {
                return;
            }
            let Some(prev) = st.last else {
                st.last = Some(position);
                return;
            };
            let Some(start) = st.start else {
                st.start = Some(position);
                st.last = Some(position);
                return;
            };

            let total_dx = position.x.0 - start.x.0;
            let total_dy = position.y.0 - start.y.0;
            if !st.panning {
                let dist = (total_dx * total_dx + total_dy * total_dy).sqrt();
                if dist > TOUCH_PAN_SCROLL_THRESHOLD_PX {
                    st.panning = true;
                }
            }

            st.last = Some(position);
            if st.panning {
                out = Some(Point::new(
                    Px(position.x.0 - prev.x.0),
                    Px(position.y.0 - prev.y.0),
                ));
            }
        },
    );
    out
}

fn clear_touch_pan_tracking<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    element: crate::GlobalElementId,
    pointer_id: fret_core::PointerId,
) -> bool {
    let mut captured_for_pan = false;
    crate::elements::with_element_state(
        &mut *cx.app,
        window,
        element,
        TouchPanScrollTracking::default,
        |st| {
            if st.pointer_id == Some(pointer_id) {
                captured_for_pan = st.captured_for_pan;
                *st = TouchPanScrollTracking::default();
            }
        },
    );
    captured_for_pan
}

fn mark_touch_pan_captured<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    element: crate::GlobalElementId,
    pointer_id: fret_core::PointerId,
) {
    crate::elements::with_element_state(
        &mut *cx.app,
        window,
        element,
        TouchPanScrollTracking::default,
        |st| {
            if st.pointer_id == Some(pointer_id) {
                st.captured_for_pan = true;
            }
        },
    );
}

fn clear_pressed_pressable_if_any<H: UiHost>(cx: &mut EventCx<'_, H>, window: AppWindowId) {
    if let Some(prev_node) = crate::elements::set_pressed_pressable(&mut *cx.app, window, None) {
        cx.invalidate(prev_node, Invalidation::Paint);
    }
}

fn apply_virtual_list_scroll_delta<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    element: crate::GlobalElementId,
    props: &crate::element::VirtualListProps,
    delta: Point,
    modifiers: Modifiers,
) -> (bool, bool) {
    crate::elements::with_element_state(
        &mut *cx.app,
        window,
        element,
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
            if std::env::var(DEBUG_VLIST_WHEEL_ENV)
                .ok()
                .is_some_and(|v| v == "1")
                && !DEBUG_VLIST_WHEEL_PRINTED.swap(true, std::sync::atomic::Ordering::Relaxed)
            {
                let max = props.scroll_handle.max_offset();
                let viewport_size = props.scroll_handle.viewport_size();
                let content_size = props.scroll_handle.content_size();
                eprintln!(
                    "scroll wheel vlist element={:?} handle_key={} axis={:?} delta={:.3} prev={:.3} next={:.3} viewport=({:.3},{:.3}) content=({:.3},{:.3}) max=({:.3},{:.3})",
                    element,
                    props.scroll_handle.base_handle().binding_key(),
                    axis,
                    delta.0,
                    prev_offset.0,
                    next.0,
                    viewport_size.width.0,
                    viewport_size.height.0,
                    content_size.width.0,
                    content_size.height.0,
                    max.x.0,
                    max.y.0,
                );
            }

            if (prev_offset.0 - next.0).abs() > SCROLL_CONSUMED_EPS {
                let visible_range = state.metrics.visible_range(next, viewport, 0);
                let needs_visible_range_rerender = visible_range.is_some_and(|visible| match state
                    .render_window_range
                {
                    None => visible.count > 0,
                    Some(rendered) => {
                        if rendered.count == 0 {
                            visible.count > 0
                        } else {
                            let rendered_start =
                                rendered.start_index.saturating_sub(rendered.overscan);
                            let rendered_end = (rendered.end_index + rendered.overscan)
                                .min(rendered.count.saturating_sub(1));
                            visible.start_index < rendered_start || visible.end_index > rendered_end
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
    )
}

pub(super) fn handle_virtual_list<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::VirtualListProps,
    event: &Event,
) -> bool {
    if cx.input_ctx.dispatch_phase == fret_runtime::InputDispatchPhase::Bubble
        && matches!(
            event,
            Event::Pointer(fret_core::PointerEvent::Down { .. })
                | Event::Pointer(fret_core::PointerEvent::Up { .. })
                | Event::PointerCancel(_)
        )
    {
        return true;
    }

    let mut consumed = false;
    let mut needs_visible_range_rerender = false;

    match event {
        Event::Pointer(fret_core::PointerEvent::Wheel {
            delta, modifiers, ..
        }) => {
            (consumed, needs_visible_range_rerender) = apply_virtual_list_scroll_delta(
                cx,
                window,
                this.element,
                &props,
                *delta,
                *modifiers,
            );
        }
        Event::Pointer(fret_core::PointerEvent::Down {
            pointer_type,
            button,
            pointer_id,
            position,
            ..
        }) => {
            if *button == MouseButton::Left {
                cx.request_focus(cx.node);
            }
            if *pointer_type == fret_core::PointerType::Touch && *button == MouseButton::Left {
                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    this.element,
                    TouchPanScrollTracking::default,
                    |st| {
                        st.pointer_id = Some(*pointer_id);
                        st.start = Some(*position);
                        st.last = Some(*position);
                        st.panning = false;
                        st.captured_for_pan = false;
                    },
                );
            }
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id,
            position,
            pointer_type,
            modifiers,
            ..
        }) => {
            if *pointer_type == fret_core::PointerType::Touch {
                let foreign_capture = cx.captured.is_some_and(|n| n != cx.node);
                match cx.input_ctx.dispatch_phase {
                    fret_runtime::InputDispatchPhase::Capture if !foreign_capture => return true,
                    fret_runtime::InputDispatchPhase::Bubble if foreign_capture => return true,
                    _ => {}
                }
            }

            if *pointer_type == fret_core::PointerType::Touch
                && let Some(delta) =
                    touch_pan_delta_for_move(cx, window, this.element, *pointer_id, *position)
            {
                (consumed, needs_visible_range_rerender) = apply_virtual_list_scroll_delta(
                    cx,
                    window,
                    this.element,
                    &props,
                    delta,
                    *modifiers,
                );
                if consumed && cx.captured != Some(cx.node) {
                    cx.capture_pointer(cx.node);
                    mark_touch_pan_captured(cx, window, this.element, *pointer_id);
                    clear_pressed_pressable_if_any(cx, window);
                }
            }
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id,
            pointer_type,
            ..
        }) => {
            if *pointer_type == fret_core::PointerType::Touch {
                let captured_for_pan =
                    clear_touch_pan_tracking(cx, window, this.element, *pointer_id);
                if captured_for_pan && cx.captured == Some(cx.node) {
                    cx.release_pointer_capture();
                }
            }
        }
        Event::PointerCancel(e) => {
            if e.pointer_type == fret_core::PointerType::Touch {
                let captured_for_pan =
                    clear_touch_pan_tracking(cx, window, this.element, e.pointer_id);
                if captured_for_pan && cx.captured == Some(cx.node) {
                    cx.release_pointer_capture();
                }
            }
        }
        _ => {}
    }

    if consumed {
        let inv = Invalidation::HitTestOnly;
        super::invalidate_scroll_handle_bindings(
            cx,
            window,
            props.scroll_handle.base_handle().binding_key(),
            inv,
        );
        // VirtualList scrolling is applied via a children-only render transform, so hit-testing
        // must be invalidated to refresh coordinate mapping under the updated offset. This does
        // not force a layout pass.
        cx.invalidate_self(inv);
        if needs_visible_range_rerender {
            let retained_host =
                crate::elements::with_window_state(&mut *cx.app, window, |window_state| {
                    let retained = window_state
                        .has_state::<crate::windowed_surface_host::RetainedVirtualListHostMarker>(
                        this.element,
                    );
                    if retained {
                        window_state.mark_retained_virtual_list_needs_reconcile(
                            this.element,
                            crate::tree::UiDebugRetainedVirtualListReconcileKind::Escape,
                        );
                    }
                    retained
                });

            if !retained_host {
                cx.notify();
            }
        }
        cx.request_redraw();
        cx.stop_propagation();
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
    if cx.input_ctx.dispatch_phase == fret_runtime::InputDispatchPhase::Bubble
        && matches!(
            event,
            Event::Pointer(fret_core::PointerEvent::Down { .. })
                | Event::Pointer(fret_core::PointerEvent::Up { .. })
                | Event::PointerCancel(_)
        )
    {
        return true;
    }

    if let Event::Pointer(fret_core::PointerEvent::Wheel {
        delta, modifiers, ..
    }) = event
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
        return true;
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            pointer_type,
            button,
            pointer_id,
            position,
            ..
        }) => {
            if *pointer_type == fret_core::PointerType::Touch && *button == MouseButton::Left {
                crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    this.element,
                    TouchPanScrollTracking::default,
                    |st| {
                        st.pointer_id = Some(*pointer_id);
                        st.start = Some(*position);
                        st.last = Some(*position);
                        st.panning = false;
                        st.captured_for_pan = false;
                    },
                );
            }
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id,
            position,
            pointer_type,
            ..
        }) => {
            if *pointer_type != fret_core::PointerType::Touch {
                return true;
            }
            let foreign_capture = cx.captured.is_some_and(|n| n != cx.node);
            match cx.input_ctx.dispatch_phase {
                fret_runtime::InputDispatchPhase::Capture if !foreign_capture => return true,
                fret_runtime::InputDispatchPhase::Bubble if foreign_capture => return true,
                _ => {}
            }
            let Some(delta) =
                touch_pan_delta_for_move(cx, window, this.element, *pointer_id, *position)
            else {
                return true;
            };

            let delta_x = if props.axis.scroll_x() {
                delta.x
            } else {
                Px(0.0)
            };
            let delta_y = if props.axis.scroll_y() {
                delta.y
            } else {
                Px(0.0)
            };

            let consumed = if let Some(handle) = props.scroll_handle.as_ref() {
                let prev = handle.offset();
                let desired = Point::new(Px(prev.x.0 - delta_x.0), Px(prev.y.0 - delta_y.0));
                handle.set_offset(desired);
                let next = handle.offset();
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
                        let desired =
                            Point::new(Px(prev.x.0 - delta_x.0), Px(prev.y.0 - delta_y.0));
                        state.scroll_handle.set_offset(desired);
                        let next = state.scroll_handle.offset();
                        (prev.x.0 - next.x.0).abs() > SCROLL_CONSUMED_EPS
                            || (prev.y.0 - next.y.0).abs() > SCROLL_CONSUMED_EPS
                    },
                )
            };

            if consumed {
                if cx.captured != Some(cx.node) {
                    cx.capture_pointer(cx.node);
                    mark_touch_pan_captured(cx, window, this.element, *pointer_id);
                    clear_pressed_pressable_if_any(cx, window);
                }
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
        Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id,
            pointer_type,
            ..
        }) => {
            if *pointer_type == fret_core::PointerType::Touch {
                let captured_for_pan =
                    clear_touch_pan_tracking(cx, window, this.element, *pointer_id);
                if captured_for_pan && cx.captured == Some(cx.node) {
                    cx.release_pointer_capture();
                }
            }
        }
        Event::PointerCancel(e) => {
            if e.pointer_type == fret_core::PointerType::Touch {
                let captured_for_pan =
                    clear_touch_pan_tracking(cx, window, this.element, e.pointer_id);
                if captured_for_pan && cx.captured == Some(cx.node) {
                    cx.release_pointer_capture();
                }
            }
        }
        _ => {}
    }

    true
}
