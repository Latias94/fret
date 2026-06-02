use std::sync::Arc;

use fret_core::{CursorIcon, MouseButton};
use fret_runtime::DragKindId;
use fret_ui::action::ActionCx;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::super::super::KEY_FLOAT_WINDOW_ACTIVATE;

pub(super) fn install_resize_handle_pointer_down<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    kind: DragKindId,
    cursor: CursorIcon,
    enable_activation: bool,
) {
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
}
