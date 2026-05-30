use std::sync::Arc;

use fret_core::CursorIcon;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{
    drag_kind_for_element, drag_threshold_for, finish_pointer_region_drag,
    handle_pointer_region_drag_move_with_threshold, prepare_pointer_region_drag_on_left_down,
};

pub(super) fn install_child_region_resize_handle_pointer_events<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    region_id: GlobalElementId,
    enabled: bool,
    cursor: CursorIcon,
) {
    let drag_kind = drag_kind_for_element(region_id);
    let drag_threshold = drag_threshold_for(cx);

    cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
        prepare_pointer_region_drag_on_left_down(
            host,
            acx,
            down,
            enabled.then_some(drag_kind),
            Some(cursor),
        )
    }));
    cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
        if !enabled {
            return false;
        }
        host.set_cursor_icon(cursor);
        handle_pointer_region_drag_move_with_threshold(host, acx, mv, drag_kind, drag_threshold)
    }));
    cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
        if !enabled {
            return false;
        }
        finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
    }));
}
