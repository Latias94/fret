use std::sync::Arc;

use fret_core::Point;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::super::{
    FloatingAreaOptions, point_add, point_sub, snap_point_to_device_pixels,
};
use super::super::super::state::FloatingAreaState;
use super::snapshot::FloatingAreaDragSnapshot;

pub(super) fn commit_floating_area_drag_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    area_id: GlobalElementId,
    id: &str,
    initial_position: Point,
    options: &FloatingAreaOptions,
    drag_snapshot: Option<FloatingAreaDragSnapshot>,
    scale_factor: f32,
) -> (Point, Arc<str>) {
    cx.state_for(
        area_id,
        || FloatingAreaState {
            position: initial_position,
            last_drag_position: None,
            test_id: options
                .test_id
                .clone()
                .unwrap_or_else(|| Arc::from(format!("{}{id}", options.test_id_prefix))),
        },
        |st| {
            if let Some(test_id) = options.test_id.clone() {
                st.test_id = test_id;
            }

            if let Some(snapshot) = drag_snapshot {
                if snapshot.dragging {
                    let prev = st.last_drag_position.unwrap_or(snapshot.start_position);
                    st.position = point_add(st.position, point_sub(snapshot.position, prev));
                    st.position = snap_point_to_device_pixels(scale_factor, st.position);
                    st.last_drag_position = Some(snapshot.position);
                } else {
                    st.last_drag_position = None;
                }
            } else {
                st.last_drag_position = None;
            }
            (st.position, st.test_id.clone())
        },
    )
}
