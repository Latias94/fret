use fret_core::PointerId;

use super::{
    begin_slider_drag, clear_slider_drag, enter_slider_typing, finish_slider_drag,
    is_slider_drag_pointer, reset_slider_interaction,
};
use crate::controls::slider::model::{SliderMode, SliderState};

#[test]
fn enter_typing_clears_active_drag_state() {
    let mut state = SliderState::default();
    begin_slider_drag(&mut state, PointerId(1));

    enter_slider_typing(&mut state);

    assert_eq!(state.mode, SliderMode::Typing);
    assert!(!state.dragging);
    assert_eq!(state.pointer_id, None);
}

#[test]
fn reset_interaction_restores_slide_mode_and_clears_drag_state() {
    let mut state = SliderState::default();
    enter_slider_typing(&mut state);
    begin_slider_drag(&mut state, PointerId(1));

    reset_slider_interaction(&mut state);

    assert_eq!(state.mode, SliderMode::Slide);
    assert!(!state.dragging);
    assert_eq!(state.pointer_id, None);
}

#[test]
fn active_drag_pointer_must_match_captured_pointer() {
    let mut state = SliderState::default();
    begin_slider_drag(&mut state, PointerId(1));

    assert!(is_slider_drag_pointer(&state, PointerId(1)));
    assert!(!is_slider_drag_pointer(&state, PointerId(2)));
}

#[test]
fn finish_drag_only_clears_matching_pointer() {
    let mut state = SliderState::default();
    begin_slider_drag(&mut state, PointerId(1));

    assert!(!finish_slider_drag(&mut state, PointerId(2)));
    assert!(state.dragging);
    assert_eq!(state.pointer_id, Some(PointerId(1)));

    assert!(finish_slider_drag(&mut state, PointerId(1)));
    assert!(!state.dragging);
    assert_eq!(state.pointer_id, None);
}

#[test]
fn clear_drag_preserves_current_mode() {
    let mut state = SliderState::default();
    enter_slider_typing(&mut state);
    begin_slider_drag(&mut state, PointerId(1));

    clear_slider_drag(&mut state);

    assert_eq!(state.mode, SliderMode::Typing);
    assert!(!state.dragging);
    assert_eq!(state.pointer_id, None);
}
