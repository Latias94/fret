use std::sync::Arc;

use fret_core::{CursorIcon, MouseButton};
use fret_interaction::runtime_drag::{DragMoveOutcome, update_immediate_move};
use fret_runtime::DragKindId;
use fret_ui::action::ActionCx;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::super::KEY_FLOAT_WINDOW_ACTIVATE;

pub(super) struct ResizeHandlePointerInput {
    pub(super) window_id: GlobalElementId,
    pub(super) kind: DragKindId,
    pub(super) cursor: CursorIcon,
    pub(super) enable_activation: bool,
}

pub(super) fn install_resize_handle_pointer_events<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: ResizeHandlePointerInput,
) {
    let ResizeHandlePointerInput {
        window_id,
        kind,
        cursor,
        enable_activation,
    } = input;

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
                ActionCx {
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
}
