use super::ElementHostWidget;
use crate::declarative::prelude::*;

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
        fret_core::PointerEvent::Wheel { delta, .. } => {
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::VirtualListState::default,
                |state| {
                    state.metrics.ensure(
                        props.len,
                        props.estimate_row_height,
                        props.gap,
                        props.scroll_margin,
                    );
                    let viewport_h = Px(state.viewport_h.0.max(0.0));

                    let prev = props.scroll_handle.offset();
                    let offset_y = state.metrics.clamp_offset(prev.y, viewport_h);

                    let next = state
                        .metrics
                        .clamp_offset(Px(offset_y.0 - delta.y.0), viewport_h);
                    if (prev.y.0 - next.0).abs() > 0.01 {
                        props
                            .scroll_handle
                            .set_offset(fret_core::Point::new(prev.x, next));
                    }
                },
            );
            cx.invalidate_self(Invalidation::Layout);
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
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
    if let fret_core::PointerEvent::Wheel { delta, .. } = pe {
        if let Some(handle) = props.scroll_handle.as_ref() {
            let prev = handle.offset();
            handle.set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));
        } else {
            crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::element::ScrollState::default,
                |state| {
                    let prev = state.scroll_handle.offset();
                    state
                        .scroll_handle
                        .set_offset(Point::new(prev.x, Px(prev.y.0 - delta.y.0)));
                },
            );
        }
        cx.invalidate_self(Invalidation::Layout);
        cx.invalidate_self(Invalidation::Paint);
        cx.request_redraw();
        cx.stop_propagation();
    }
    true
}
