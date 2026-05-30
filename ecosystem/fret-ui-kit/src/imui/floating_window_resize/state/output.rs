use std::sync::Arc;

use fret_core::{Point, Size};

use super::super::FloatingWindowResizeHandleTestIds;

pub(in crate::imui) struct FloatingWindowResizeStateOutput {
    pub(in crate::imui) position_after_resize: Point,
    pub(in crate::imui) size: Size,
    pub(in crate::imui) resizing: bool,
    pub(in crate::imui) title_bar_test_id: Arc<str>,
    pub(in crate::imui) close_button_test_id: Arc<str>,
    pub(in crate::imui) handle_test_ids: FloatingWindowResizeHandleTestIds,
}
