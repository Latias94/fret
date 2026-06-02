use fret_core::PointerId;

use super::model::{SliderMode, SliderState};

#[cfg(test)]
mod tests;

pub(super) fn reset_slider_interaction(state: &mut SliderState) {
    state.mode = SliderMode::Slide;
    state.dragging = false;
    state.pointer_id = None;
}

pub(super) fn enter_slider_typing(state: &mut SliderState) {
    state.mode = SliderMode::Typing;
    state.dragging = false;
    state.pointer_id = None;
}

pub(super) fn begin_slider_drag(state: &mut SliderState, pointer_id: PointerId) {
    state.dragging = true;
    state.pointer_id = Some(pointer_id);
}

pub(super) fn clear_slider_drag(state: &mut SliderState) {
    state.dragging = false;
    state.pointer_id = None;
}

pub(super) fn finish_slider_drag(state: &mut SliderState, pointer_id: PointerId) -> bool {
    if state.pointer_id == Some(pointer_id) {
        clear_slider_drag(state);
        true
    } else {
        false
    }
}

pub(super) fn is_slider_drag_pointer(state: &SliderState, pointer_id: PointerId) -> bool {
    state.dragging && state.pointer_id == Some(pointer_id)
}
