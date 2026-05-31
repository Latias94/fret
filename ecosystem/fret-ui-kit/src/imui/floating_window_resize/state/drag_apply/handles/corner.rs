use fret_core::{Point, Px};

use super::super::super::super::super::{FloatWindowResizeHandle, FloatWindowState};
use super::super::bounds::ResizeDragBounds;

pub(super) fn apply_corner_resize_handle_delta(
    st: &mut FloatWindowState,
    position: &mut Point,
    handle: FloatWindowResizeHandle,
    delta: Point,
    bounds: &ResizeDragBounds,
) {
    match handle {
        FloatWindowResizeHandle::TopLeft => {
            let right = Px(position.x.0 + st.size.width.0);
            let bottom = Px(position.y.0 + st.size.height.0);

            let width = bounds.clamp_width(st.size.width.0 - delta.x.0);
            let height = bounds.clamp_height(st.size.height.0 - delta.y.0);
            st.size.width = width;
            st.size.height = height;
            position.x = Px(right.0 - width.0);
            position.y = Px(bottom.0 - height.0);
        }
        FloatWindowResizeHandle::TopRight => {
            let bottom = Px(position.y.0 + st.size.height.0);
            st.size.width = bounds.clamp_width(st.size.width.0 + delta.x.0);
            let height = bounds.clamp_height(st.size.height.0 - delta.y.0);
            st.size.height = height;
            position.y = Px(bottom.0 - height.0);
        }
        FloatWindowResizeHandle::BottomLeft => {
            let right = Px(position.x.0 + st.size.width.0);
            let width = bounds.clamp_width(st.size.width.0 - delta.x.0);
            st.size.width = width;
            position.x = Px(right.0 - width.0);
            st.size.height = bounds.clamp_height(st.size.height.0 + delta.y.0);
        }
        FloatWindowResizeHandle::BottomRight => {
            st.size.width = bounds.clamp_width(st.size.width.0 + delta.x.0);
            st.size.height = bounds.clamp_height(st.size.height.0 + delta.y.0);
        }
        FloatWindowResizeHandle::Left
        | FloatWindowResizeHandle::Right
        | FloatWindowResizeHandle::Top
        | FloatWindowResizeHandle::Bottom => {
            unreachable!("edge resize handles are routed through edge mutation")
        }
    }
}
