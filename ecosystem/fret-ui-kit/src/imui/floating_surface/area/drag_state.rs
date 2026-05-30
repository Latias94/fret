use std::sync::Arc;

use fret_core::Point;
use fret_core::window::WindowMetricsService;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{FloatingAreaOptions, point_add, point_sub, snap_point_to_device_pixels};
use super::super::state::FloatingAreaState;
use snapshot::floating_area_drag_snapshot;

mod snapshot;

pub(super) struct PreparedFloatingAreaState {
    pub(super) position: Point,
    pub(super) test_id: Arc<str>,
    pub(super) dragging: bool,
}

pub(super) fn prepare_floating_area_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    area_id: GlobalElementId,
    id: &str,
    initial_position: Point,
    options: &FloatingAreaOptions,
    drag_kind: fret_runtime::DragKindId,
) -> PreparedFloatingAreaState {
    let drag_snapshot = floating_area_drag_snapshot(cx, drag_kind);
    let dragging = drag_snapshot
        .map(|snapshot| snapshot.dragging)
        .unwrap_or(false);

    let scale_factor = cx
        .app
        .global::<WindowMetricsService>()
        .and_then(|svc| svc.scale_factor(cx.window))
        .unwrap_or(1.0);

    let (position, test_id) = cx.state_for(
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
    );

    PreparedFloatingAreaState {
        position,
        test_id,
        dragging,
    }
}

pub(super) fn final_floating_area_state<H: UiHost>(
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
