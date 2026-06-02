use fret_core::Point;

use super::super::super::super::{FloatWindowState, FloatingWindowResizeOptions};
use super::super::super::FloatingWindowResizeSnapshot;
use super::super::drag_apply::apply_resize_drag;

pub(super) struct ResizeStateMutationInput {
    pub(super) area_position: Point,
    pub(super) resize: Option<FloatingWindowResizeOptions>,
    pub(super) resize_snapshot: Option<FloatingWindowResizeSnapshot>,
    pub(super) collapsed: bool,
}

pub(super) fn apply_resize_state_mutation(
    st: &mut FloatWindowState,
    input: ResizeStateMutationInput,
) -> Point {
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

    position
}
