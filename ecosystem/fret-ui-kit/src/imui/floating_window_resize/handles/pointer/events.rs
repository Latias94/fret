use fret_core::CursorIcon;
use fret_runtime::DragKindId;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod down;
mod move_phase;
mod up;

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

    down::install_resize_handle_pointer_down(cx, window_id, kind, cursor, enable_activation);
    move_phase::install_resize_handle_pointer_move(cx, kind, cursor);
    up::install_resize_handle_pointer_up(cx, kind);
}
