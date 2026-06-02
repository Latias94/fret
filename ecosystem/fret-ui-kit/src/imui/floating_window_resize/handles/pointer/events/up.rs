use std::sync::Arc;

use fret_runtime::DragKindId;
use fret_ui::{ElementContext, UiHost};

pub(super) fn install_resize_handle_pointer_up<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    kind: DragKindId,
) {
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
