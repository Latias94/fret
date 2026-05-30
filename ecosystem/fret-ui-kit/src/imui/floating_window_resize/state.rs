use fret_core::{Point, Size};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::FloatingWindowResizeHandleTestIds;
use super::FloatingWindowResizeSnapshot;
use drag_apply::apply_resize_drag;
use initial::initial_float_window_state;
pub(in crate::imui) use output::FloatingWindowResizeStateOutput;

mod drag_apply;
mod initial;
mod output;

pub(in crate::imui) fn prepare_resize_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    id: &str,
    area_position: Point,
    initial_size: Option<Size>,
    resize: Option<super::super::FloatingWindowResizeOptions>,
    resize_snapshot: Option<FloatingWindowResizeSnapshot>,
    collapsed: bool,
    scale_factor: f32,
) -> FloatingWindowResizeStateOutput {
    let resizing = resize_snapshot
        .map(|snapshot| snapshot.dragging)
        .unwrap_or(false)
        && !collapsed;

    let (
        position_after_resize,
        size,
        title_bar_test_id,
        close_button_test_id,
        resize_left_test_id,
        resize_right_test_id,
        resize_top_test_id,
        resize_bottom_test_id,
        resize_top_left_test_id,
        resize_top_right_test_id,
        resize_bottom_left_test_id,
        resize_corner_test_id,
    ) = cx.state_for(
        window_id,
        || initial_float_window_state(id, initial_size),
        |st| {
            let mut position = area_position;

            let resize_cfg = resize.unwrap_or_default();
            let min = resize_cfg.min_size;
            let max = resize_cfg.max_size;

            if collapsed {
                st.last_resize_position = None;
            } else if let Some(snapshot) = resize_snapshot {
                if snapshot.dragging {
                    apply_resize_drag(st, &mut position, snapshot, min, max);
                } else {
                    st.last_resize_position = None;
                }
            } else {
                st.last_resize_position = None;
            }

            st.size = super::super::snap_size_to_device_pixels(scale_factor, st.size);
            position = super::super::snap_point_to_device_pixels(scale_factor, position);

            (
                position,
                st.size,
                st.title_bar_test_id.clone(),
                st.close_button_test_id.clone(),
                st.resize_left_test_id.clone(),
                st.resize_right_test_id.clone(),
                st.resize_top_test_id.clone(),
                st.resize_bottom_test_id.clone(),
                st.resize_top_left_test_id.clone(),
                st.resize_top_right_test_id.clone(),
                st.resize_bottom_left_test_id.clone(),
                st.resize_corner_test_id.clone(),
            )
        },
    );

    FloatingWindowResizeStateOutput {
        position_after_resize,
        size,
        resizing,
        title_bar_test_id,
        close_button_test_id,
        handle_test_ids: FloatingWindowResizeHandleTestIds {
            left: resize_left_test_id,
            right: resize_right_test_id,
            top: resize_top_test_id,
            bottom: resize_bottom_test_id,
            top_left: resize_top_left_test_id,
            top_right: resize_top_right_test_id,
            bottom_left: resize_bottom_left_test_id,
            bottom_right: resize_corner_test_id,
        },
    }
}
