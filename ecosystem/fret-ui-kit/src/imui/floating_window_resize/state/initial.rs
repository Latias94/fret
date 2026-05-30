use std::sync::Arc;

use fret_core::{Px, Size};

pub(super) fn initial_float_window_state(
    id: &str,
    initial_size: Option<Size>,
) -> super::super::super::FloatWindowState {
    super::super::super::FloatWindowState {
        size: initial_size.unwrap_or_else(|| Size::new(Px(0.0), Px(0.0))),
        last_resize_position: None,
        title_bar_test_id: Arc::from(format!("imui.float_window.title_bar:{id}")),
        close_button_test_id: Arc::from(format!("imui.float_window.close:{id}")),
        resize_left_test_id: Arc::from(format!("imui.float_window.resize.left:{id}")),
        resize_right_test_id: Arc::from(format!("imui.float_window.resize.right:{id}")),
        resize_top_test_id: Arc::from(format!("imui.float_window.resize.top:{id}")),
        resize_bottom_test_id: Arc::from(format!("imui.float_window.resize.bottom:{id}")),
        resize_top_left_test_id: Arc::from(format!("imui.float_window.resize.top_left:{id}")),
        resize_top_right_test_id: Arc::from(format!("imui.float_window.resize.top_right:{id}")),
        resize_bottom_left_test_id: Arc::from(format!("imui.float_window.resize.bottom_left:{id}")),
        resize_corner_test_id: Arc::from(format!("imui.float_window.resize.corner:{id}")),
    }
}
