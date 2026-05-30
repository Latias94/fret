use std::sync::Arc;

use fret_core::MouseButton;
use fret_interaction::runtime_drag::{DragMoveOutcome, update_immediate_move};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, GlobalElementId};

use super::super::super::{
    FloatWindowResizeHandle, KEY_FLOAT_WINDOW_ACTIVATE, float_layer_bring_to_front_if_activated,
    float_window_resize_kind_for_element,
};
use super::cursor::resize_handle_cursor;
use super::layout::resize_handle_layout;

pub(super) fn resize_handle_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    handle: FloatWindowResizeHandle,
    test_id: Arc<str>,
    enable_activation: bool,
) -> AnyElement {
    let cursor = resize_handle_cursor(handle);
    let layout = resize_handle_layout(handle);

    let kind = float_window_resize_kind_for_element(window_id, handle);
    cx.pointer_region(
        PointerRegionProps {
            layout,
            ..Default::default()
        },
        move |cx| {
            let _region_id = cx.root_id();
            float_layer_bring_to_front_if_activated(cx, window_id);

            cx.pointer_region_clear_on_pointer_down();
            cx.pointer_region_clear_on_pointer_move();
            cx.pointer_region_clear_on_pointer_up();

            cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
                if down.button != MouseButton::Left {
                    return false;
                }

                host.request_focus(acx.target);
                host.capture_pointer();
                host.set_cursor_icon(cursor);
                if host.drag(down.pointer_id).is_none() {
                    host.begin_drag_with_kind(down.pointer_id, kind, acx.window, down.position);
                }
                if enable_activation {
                    host.record_transient_event(
                        fret_ui::action::ActionCx {
                            window: acx.window,
                            target: window_id,
                        },
                        KEY_FLOAT_WINDOW_ACTIVATE,
                    );
                }
                host.notify(acx);
                false
            }));

            cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
                host.set_cursor_icon(cursor);

                let Some(drag) = host.drag_mut(mv.pointer_id) else {
                    return false;
                };
                if drag.kind != kind || drag.source_window != acx.window {
                    return false;
                }

                let outcome = update_immediate_move(drag, acx.window, mv.position, mv.buttons.left);
                if outcome == DragMoveOutcome::Canceled {
                    host.cancel_drag(mv.pointer_id);
                    host.release_pointer_capture();
                    host.notify(acx);
                    return false;
                }

                host.notify(acx);
                false
            }));

            cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
                if let Some(drag) = host.drag(up.pointer_id)
                    && drag.kind == kind
                    && drag.source_window == acx.window
                {
                    host.cancel_drag(up.pointer_id);
                }
                host.release_pointer_capture();
                host.notify(acx);
                false
            }));

            Vec::new()
        },
    )
    .test_id(test_id)
}
