use super::{DragState, DragValueCoreResponse};
use fret_core::{Point, PointerId, Px};

fn origin() -> Point {
    Point::new(Px(0.0), Px(0.0))
}

#[test]
fn drag_state_commit_requires_a_live_value_change() {
    let mut state = DragState::<f64>::default();
    state.current_value = 10.0;
    state.begin_session(PointerId(1), origin());

    assert!(!state.apply_live_value(10.0));
    assert!(!state.commit_session());
}

#[test]
fn drag_state_commit_remembers_any_live_edit_in_the_session() {
    let mut state = DragState::<f64>::default();
    state.current_value = 10.0;
    state.begin_session(PointerId(1), origin());

    assert!(state.apply_live_value(12.0));
    assert!(state.apply_live_value(10.0));
    assert!(state.commit_session());
}

#[test]
fn drag_state_cancel_clears_live_edit_tracking() {
    let mut state = DragState::<f64>::default();
    state.current_value = 10.0;
    state.begin_session(PointerId(1), origin());
    assert!(state.apply_live_value(12.0));

    assert_eq!(state.cancel_session(), Some(10.0));

    state.begin_session(PointerId(1), origin());
    assert!(!state.commit_session());
}

#[test]
fn drag_value_core_response_exposes_read_only_signals() {
    let response = DragValueCoreResponse::new(true, true, false, true);

    assert!(response.dragging());
    assert!(response.hovered());
    assert!(!response.pressed());
    assert!(response.focused());
}
