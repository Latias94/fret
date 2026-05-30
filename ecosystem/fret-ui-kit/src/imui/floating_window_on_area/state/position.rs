use std::sync::Arc;

use fret_core::Point;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::FloatingAreaState;

pub(super) fn sync_floating_area_position_after_resize<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    id: &str,
    initial_position: Point,
    area_position: Point,
    position_after_resize: Point,
) {
    if position_after_resize == area_position {
        return;
    }

    cx.state_for(
        window_id,
        || FloatingAreaState {
            position: initial_position,
            last_drag_position: None,
            test_id: Arc::from(format!("imui.float_window.window:{id}")),
        },
        |st| {
            st.position = position_after_resize;
        },
    );
}
