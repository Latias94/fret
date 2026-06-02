use fret_core::{Point, Size};

use super::super::super::{FloatWindowState, point_sub};
use super::super::FloatingWindowResizeSnapshot;
use bounds::ResizeDragBounds;
use handles::apply_resize_handle_delta;

mod bounds;
mod handles;

pub(super) fn apply_resize_drag(
    st: &mut FloatWindowState,
    position: &mut Point,
    snapshot: FloatingWindowResizeSnapshot,
    min: Size,
    max: Option<Size>,
) {
    let bounds = ResizeDragBounds::new(min, max);
    let prev = st.last_resize_position.unwrap_or(snapshot.start_position);
    let delta = point_sub(snapshot.position, prev);
    apply_resize_handle_delta(st, position, snapshot.handle, delta, &bounds);

    st.last_resize_position = Some(snapshot.position);
}
