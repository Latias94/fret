use std::sync::Arc;

use fret_core::{Point, Size};

use super::super::super::super::FloatWindowState;
use super::super::super::FloatingWindowResizeHandleTestIds;
use super::super::output::FloatingWindowResizeStateOutput;

pub(super) struct CommittedResizeState {
    position_after_resize: Point,
    size: Size,
    title_bar_test_id: Arc<str>,
    close_button_test_id: Arc<str>,
    resize_left_test_id: Arc<str>,
    resize_right_test_id: Arc<str>,
    resize_top_test_id: Arc<str>,
    resize_bottom_test_id: Arc<str>,
    resize_top_left_test_id: Arc<str>,
    resize_top_right_test_id: Arc<str>,
    resize_bottom_left_test_id: Arc<str>,
    resize_corner_test_id: Arc<str>,
}

impl CommittedResizeState {
    pub(super) fn from_window_state(position_after_resize: Point, st: &FloatWindowState) -> Self {
        Self {
            position_after_resize,
            size: st.size,
            title_bar_test_id: st.title_bar_test_id.clone(),
            close_button_test_id: st.close_button_test_id.clone(),
            resize_left_test_id: st.resize_left_test_id.clone(),
            resize_right_test_id: st.resize_right_test_id.clone(),
            resize_top_test_id: st.resize_top_test_id.clone(),
            resize_bottom_test_id: st.resize_bottom_test_id.clone(),
            resize_top_left_test_id: st.resize_top_left_test_id.clone(),
            resize_top_right_test_id: st.resize_top_right_test_id.clone(),
            resize_bottom_left_test_id: st.resize_bottom_left_test_id.clone(),
            resize_corner_test_id: st.resize_corner_test_id.clone(),
        }
    }
}

pub(super) fn output_from_committed_resize_state(
    committed: CommittedResizeState,
    resizing: bool,
) -> FloatingWindowResizeStateOutput {
    FloatingWindowResizeStateOutput {
        position_after_resize: committed.position_after_resize,
        size: committed.size,
        resizing,
        title_bar_test_id: committed.title_bar_test_id,
        close_button_test_id: committed.close_button_test_id,
        handle_test_ids: FloatingWindowResizeHandleTestIds {
            left: committed.resize_left_test_id,
            right: committed.resize_right_test_id,
            top: committed.resize_top_test_id,
            bottom: committed.resize_bottom_test_id,
            top_left: committed.resize_top_left_test_id,
            top_right: committed.resize_top_right_test_id,
            bottom_left: committed.resize_bottom_left_test_id,
            bottom_right: committed.resize_corner_test_id,
        },
    }
}
