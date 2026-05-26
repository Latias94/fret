use fret_core::{Point, Px, Size};

use super::super::FloatingWindowResizeSnapshot;

pub(super) fn apply_resize_drag(
    st: &mut super::super::super::FloatWindowState,
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
    let delta = super::super::super::point_sub(snapshot.position, prev);

    match snapshot.handle {
        super::super::super::FloatWindowResizeHandle::Left => {
            let right = Px(position.x.0 + st.size.width.0);
            let width = clamp_width(st.size.width.0 - delta.x.0);
            st.size.width = width;
            position.x = Px(right.0 - width.0);
        }
        super::super::super::FloatWindowResizeHandle::Right => {
            st.size.width = clamp_width(st.size.width.0 + delta.x.0);
        }
        super::super::super::FloatWindowResizeHandle::Top => {
            let bottom = Px(position.y.0 + st.size.height.0);
            let height = clamp_height(st.size.height.0 - delta.y.0);
            st.size.height = height;
            position.y = Px(bottom.0 - height.0);
        }
        super::super::super::FloatWindowResizeHandle::Bottom => {
            st.size.height = clamp_height(st.size.height.0 + delta.y.0);
        }
        super::super::super::FloatWindowResizeHandle::TopLeft => {
            let right = Px(position.x.0 + st.size.width.0);
            let bottom = Px(position.y.0 + st.size.height.0);

            let width = clamp_width(st.size.width.0 - delta.x.0);
            let height = clamp_height(st.size.height.0 - delta.y.0);
            st.size.width = width;
            st.size.height = height;
            position.x = Px(right.0 - width.0);
            position.y = Px(bottom.0 - height.0);
        }
        super::super::super::FloatWindowResizeHandle::TopRight => {
            let bottom = Px(position.y.0 + st.size.height.0);
            st.size.width = clamp_width(st.size.width.0 + delta.x.0);
            let height = clamp_height(st.size.height.0 - delta.y.0);
            st.size.height = height;
            position.y = Px(bottom.0 - height.0);
        }
        super::super::super::FloatWindowResizeHandle::BottomLeft => {
            let right = Px(position.x.0 + st.size.width.0);
            let width = clamp_width(st.size.width.0 - delta.x.0);
            st.size.width = width;
            position.x = Px(right.0 - width.0);
            st.size.height = clamp_height(st.size.height.0 + delta.y.0);
        }
        super::super::super::FloatWindowResizeHandle::BottomRight => {
            st.size.width = clamp_width(st.size.width.0 + delta.x.0);
            st.size.height = clamp_height(st.size.height.0 + delta.y.0);
        }
    }

    st.last_resize_position = Some(snapshot.position);
}
