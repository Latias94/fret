use std::sync::Arc;

use fret_core::{Point, Px, Size};
use fret_ui::UiHost;
use fret_ui::{ElementContext, GlobalElementId};

mod handles;

pub(super) use handles::resize_stack_element;

#[derive(Debug, Clone)]
pub(super) struct FloatingWindowResizeHandleTestIds {
    pub(super) left: Arc<str>,
    pub(super) right: Arc<str>,
    pub(super) top: Arc<str>,
    pub(super) bottom: Arc<str>,
    pub(super) top_left: Arc<str>,
    pub(super) top_right: Arc<str>,
    pub(super) bottom_left: Arc<str>,
    pub(super) bottom_right: Arc<str>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FloatingWindowResizeSnapshot {
    handle: super::FloatWindowResizeHandle,
    dragging: bool,
    position: Point,
    start_position: Point,
}

pub(super) struct FloatingWindowResizeStateOutput {
    pub(super) position_after_resize: Point,
    pub(super) size: Size,
    pub(super) resizing: bool,
    pub(super) title_bar_test_id: Arc<str>,
    pub(super) close_button_test_id: Arc<str>,
    pub(super) handle_test_ids: FloatingWindowResizeHandleTestIds,
}

pub(super) fn current_resize_snapshot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    resize_enabled: bool,
) -> Option<FloatingWindowResizeSnapshot> {
    if !resize_enabled {
        return None;
    }

    [
        super::FloatWindowResizeHandle::Left,
        super::FloatWindowResizeHandle::Right,
        super::FloatWindowResizeHandle::Top,
        super::FloatWindowResizeHandle::Bottom,
        super::FloatWindowResizeHandle::TopLeft,
        super::FloatWindowResizeHandle::TopRight,
        super::FloatWindowResizeHandle::BottomLeft,
        super::FloatWindowResizeHandle::BottomRight,
    ]
    .into_iter()
    .find_map(|handle| {
        let kind = super::float_window_resize_kind_for_element(window_id, handle);
        cx.app
            .find_drag_pointer_id(|d| {
                d.kind == kind && d.source_window == cx.window && d.current_window == cx.window
            })
            .and_then(|pointer_id| cx.app.drag(pointer_id))
            .filter(|drag| drag.kind == kind)
            .map(|drag| FloatingWindowResizeSnapshot {
                handle,
                dragging: drag.dragging,
                position: drag.position,
                start_position: drag.start_position,
            })
    })
}

fn apply_resize_drag(
    st: &mut super::FloatWindowState,
    position: &mut Point,
    snapshot: FloatingWindowResizeSnapshot,
    min: Size,
    max: Option<Size>,
) {
    let clamp_width = |value: f32| -> Px {
        let mut out = value.max(min.width.0);
        if let Some(max) = max {
            out = out.min(max.width.0);
        }
        Px(out)
    };
    let clamp_height = |value: f32| -> Px {
        let mut out = value.max(min.height.0);
        if let Some(max) = max {
            out = out.min(max.height.0);
        }
        Px(out)
    };

    let prev = st.last_resize_position.unwrap_or(snapshot.start_position);
    let delta = super::point_sub(snapshot.position, prev);

    match snapshot.handle {
        super::FloatWindowResizeHandle::Left => {
            let right = Px(position.x.0 + st.size.width.0);
            let width = clamp_width(st.size.width.0 - delta.x.0);
            st.size.width = width;
            position.x = Px(right.0 - width.0);
        }
        super::FloatWindowResizeHandle::Right => {
            st.size.width = clamp_width(st.size.width.0 + delta.x.0);
        }
        super::FloatWindowResizeHandle::Top => {
            let bottom = Px(position.y.0 + st.size.height.0);
            let height = clamp_height(st.size.height.0 - delta.y.0);
            st.size.height = height;
            position.y = Px(bottom.0 - height.0);
        }
        super::FloatWindowResizeHandle::Bottom => {
            st.size.height = clamp_height(st.size.height.0 + delta.y.0);
        }
        super::FloatWindowResizeHandle::TopLeft => {
            let right = Px(position.x.0 + st.size.width.0);
            let bottom = Px(position.y.0 + st.size.height.0);

            let width = clamp_width(st.size.width.0 - delta.x.0);
            let height = clamp_height(st.size.height.0 - delta.y.0);
            st.size.width = width;
            st.size.height = height;
            position.x = Px(right.0 - width.0);
            position.y = Px(bottom.0 - height.0);
        }
        super::FloatWindowResizeHandle::TopRight => {
            let bottom = Px(position.y.0 + st.size.height.0);
            st.size.width = clamp_width(st.size.width.0 + delta.x.0);
            let height = clamp_height(st.size.height.0 - delta.y.0);
            st.size.height = height;
            position.y = Px(bottom.0 - height.0);
        }
        super::FloatWindowResizeHandle::BottomLeft => {
            let right = Px(position.x.0 + st.size.width.0);
            let width = clamp_width(st.size.width.0 - delta.x.0);
            st.size.width = width;
            position.x = Px(right.0 - width.0);
            st.size.height = clamp_height(st.size.height.0 + delta.y.0);
        }
        super::FloatWindowResizeHandle::BottomRight => {
            st.size.width = clamp_width(st.size.width.0 + delta.x.0);
            st.size.height = clamp_height(st.size.height.0 + delta.y.0);
        }
    }

    st.last_resize_position = Some(snapshot.position);
}

pub(super) fn prepare_resize_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    id: &str,
    area_position: Point,
    initial_size: Option<Size>,
    resize: Option<super::FloatingWindowResizeOptions>,
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
        || super::FloatWindowState {
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
            resize_bottom_left_test_id: Arc::from(format!(
                "imui.float_window.resize.bottom_left:{id}"
            )),
            resize_corner_test_id: Arc::from(format!("imui.float_window.resize.corner:{id}")),
        },
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

            st.size = super::snap_size_to_device_pixels(scale_factor, st.size);
            position = super::snap_point_to_device_pixels(scale_factor, position);

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
