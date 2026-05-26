use std::sync::Arc;

use fret_core::Point;

#[derive(Debug, Clone, Copy, Default)]
pub(in crate::imui) struct FloatingWindowChromeResponse {
    pub(in crate::imui) size: Option<fret_core::Size>,
    pub(in crate::imui) resizing: bool,
    pub(in crate::imui) collapsed: bool,
}

#[derive(Debug)]
pub(in crate::imui) struct FloatingAreaState {
    pub(in crate::imui) position: Point,
    pub(in crate::imui) last_drag_position: Option<Point>,
    pub(in crate::imui) test_id: Arc<str>,
}

#[derive(Debug)]
pub(in crate::imui) struct FloatWindowState {
    pub(in crate::imui) size: fret_core::Size,
    pub(in crate::imui) last_resize_position: Option<Point>,
    pub(in crate::imui) title_bar_test_id: Arc<str>,
    pub(in crate::imui) close_button_test_id: Arc<str>,
    pub(in crate::imui) resize_left_test_id: Arc<str>,
    pub(in crate::imui) resize_right_test_id: Arc<str>,
    pub(in crate::imui) resize_top_test_id: Arc<str>,
    pub(in crate::imui) resize_bottom_test_id: Arc<str>,
    pub(in crate::imui) resize_top_left_test_id: Arc<str>,
    pub(in crate::imui) resize_top_right_test_id: Arc<str>,
    pub(in crate::imui) resize_bottom_left_test_id: Arc<str>,
    pub(in crate::imui) resize_corner_test_id: Arc<str>,
}
