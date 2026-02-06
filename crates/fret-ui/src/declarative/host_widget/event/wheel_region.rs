use super::ElementHostWidget;
use crate::declarative::frame::element_record_for_node;
use crate::declarative::mount::node_for_element_in_window_frame;
use crate::declarative::prelude::*;

const SCROLL_CONSUMED_EPS: f32 = 0.001;

pub(super) fn handle_wheel_region<H: UiHost>(
    _this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::WheelRegionProps,
    event: &Event,
) -> bool {
    let Event::Pointer(pe) = event else {
        return true;
    };

    let fret_core::PointerEvent::Wheel {
        delta, modifiers, ..
    } = pe
    else {
        return true;
    };

    let scroll_x = props.axis.scroll_x();
    let scroll_y = props.axis.scroll_y();
    let mut delta_x = if scroll_x { delta.x } else { Px(0.0) };
    let delta_y = if scroll_y { delta.y } else { Px(0.0) };
    if scroll_x && !scroll_y && modifiers.shift && delta_x.0.abs() <= 0.01 {
        delta_x = delta_y;
    }

    let prev = props.scroll_handle.offset();
    let desired = Point::new(Px(prev.x.0 - delta_x.0), Px(prev.y.0 - delta_y.0));
    props.scroll_handle.set_offset(desired);
    let next = props.scroll_handle.offset();
    let consumed = (prev.x.0 - next.x.0).abs() > SCROLL_CONSUMED_EPS
        || (prev.y.0 - next.y.0).abs() > SCROLL_CONSUMED_EPS;
    if !consumed {
        return true;
    }

    super::invalidate_scroll_handle_bindings(
        cx,
        window,
        props.scroll_handle.binding_key(),
        Invalidation::HitTestOnly,
    );

    if let Some(target) = props.scroll_target
        && let Some(node) = node_for_element_in_window_frame(&mut *cx.app, window, target)
    {
        let is_vlist = element_record_for_node(&mut *cx.app, window, node)
            .map(|r| {
                matches!(
                    r.instance,
                    crate::declarative::frame::ElementInstance::VirtualList(_)
                )
            })
            .unwrap_or(false);
        let inv = if is_vlist {
            Invalidation::Layout
        } else {
            Invalidation::HitTestOnly
        };
        cx.invalidate(node, inv);
    }

    cx.invalidate_self(Invalidation::HitTestOnly);
    cx.request_redraw();
    cx.stop_propagation();

    true
}
