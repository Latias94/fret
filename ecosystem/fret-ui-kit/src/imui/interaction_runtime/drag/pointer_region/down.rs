use fret_core::{CursorIcon, MouseButton};
use fret_runtime::DragKindId;
use fret_ui::action::{ActionCx, PointerDownCx, UiPointerActionHost};

pub(in crate::imui) fn prepare_pointer_region_drag_on_left_down(
    host: &mut dyn UiPointerActionHost,
    acx: ActionCx,
    down: PointerDownCx,
    drag_kind: Option<DragKindId>,
    cursor: Option<CursorIcon>,
) -> bool {
    if down.button != MouseButton::Left {
        return false;
    }

    host.request_focus(acx.target);
    if let Some(cursor) = cursor {
        host.set_cursor_icon(cursor);
    }
    if let Some(drag_kind) = drag_kind {
        host.capture_pointer();
        if host.drag(down.pointer_id).is_none() {
            host.begin_drag_with_kind(down.pointer_id, drag_kind, acx.window, down.position);
        }
    }
    true
}
