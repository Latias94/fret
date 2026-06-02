use std::sync::Arc;

use fret_core::CursorIcon;
use fret_interaction::runtime_drag::{DragMoveOutcome, update_immediate_move};
use fret_runtime::DragKindId;
use fret_ui::{ElementContext, UiHost};

pub(super) fn install_resize_handle_pointer_move<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    kind: DragKindId,
    cursor: CursorIcon,
) {
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
}
