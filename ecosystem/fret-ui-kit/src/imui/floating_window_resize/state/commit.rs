use fret_core::{Point, Size};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{FloatingWindowResizeHandleTestIds, FloatingWindowResizeSnapshot};
use super::drag_apply::apply_resize_drag;
use super::initial::initial_float_window_state;
use super::output::FloatingWindowResizeStateOutput;

pub(super) struct ResizeStateCommitInput<'a> {
    pub(super) window_id: GlobalElementId,
    pub(super) id: &'a str,
    pub(super) area_position: Point,
    pub(super) initial_size: Option<Size>,
    pub(super) resize: Option<super::super::super::FloatingWindowResizeOptions>,
    pub(super) resize_snapshot: Option<FloatingWindowResizeSnapshot>,
    pub(super) collapsed: bool,
    pub(super) scale_factor: f32,
    pub(super) resizing: bool,
}

pub(super) fn commit_resize_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: ResizeStateCommitInput<'_>,
) -> FloatingWindowResizeStateOutput {
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
        input.window_id,
        || initial_float_window_state(input.id, input.initial_size),
        |st| {
            let mut position = input.area_position;

            let resize_cfg = input.resize.unwrap_or_default();
            let min = resize_cfg.min_size;
            let max = resize_cfg.max_size;

            if input.collapsed {
                st.last_resize_position = None;
            } else if let Some(snapshot) = input.resize_snapshot {
                if snapshot.dragging {
                    apply_resize_drag(st, &mut position, snapshot, min, max);
                } else {
                    st.last_resize_position = None;
                }
            } else {
                st.last_resize_position = None;
            }

            st.size = super::super::super::snap_size_to_device_pixels(input.scale_factor, st.size);
            position =
                super::super::super::snap_point_to_device_pixels(input.scale_factor, position);

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
        resizing: input.resizing,
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
