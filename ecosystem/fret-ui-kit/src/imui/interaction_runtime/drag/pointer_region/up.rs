use fret_core::PointerId;
use fret_runtime::DragKindId;
use fret_ui::action::{ActionCx, UiPointerActionHost};

pub(in crate::imui) fn finish_pointer_region_drag(
    host: &mut dyn UiPointerActionHost,
    acx: ActionCx,
    pointer_id: PointerId,
    drag_kind: DragKindId,
) -> bool {
    if let Some(drag) = host.drag(pointer_id)
        && drag.kind == drag_kind
        && drag.source_window == acx.window
    {
        if drag.dragging {
            host.record_transient_event(acx, crate::imui::KEY_DRAG_STOPPED);
        }
        host.cancel_drag(pointer_id);
    }
    host.release_pointer_capture();
    host.notify(acx);
    false
}
