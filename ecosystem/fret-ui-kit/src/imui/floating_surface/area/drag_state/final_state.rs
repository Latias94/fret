use std::sync::Arc;

use fret_core::Point;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::state::FloatingAreaState;

pub(in crate::imui::floating_surface::area) fn final_floating_area_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    area_id: GlobalElementId,
    position: Point,
    test_id: Arc<str>,
) -> (Point, Arc<str>) {
    cx.state_for(
        area_id,
        || FloatingAreaState {
            position,
            last_drag_position: None,
            test_id,
        },
        |st| (st.position, st.test_id.clone()),
    )
}
