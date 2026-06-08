use std::sync::Arc;

use fret_core::{Modifiers, MouseButton, PointerId};

use super::{
    proof_collection_browser_scope_box_select_can_start_from_down,
    proof_collection_browser_scope_box_select_cancel_pointer,
    proof_collection_browser_scope_box_select_session_for_move,
    proof_collection_browser_scope_box_select_session_for_up,
    proof_collection_browser_scope_box_select_session_from_down,
};
use crate::imui_editor_proof_demo::collection::box_select::ProofCollectionBoxSelectState;

mod fixtures;

use fixtures::{point, pointer_cancel, pointer_down, pointer_move, pointer_up, session};

#[test]
fn box_select_down_arms_left_background_session() {
    let down = pointer_down(
        MouseButton::Left,
        point(12.0, 24.0),
        false,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    assert!(proof_collection_browser_scope_box_select_can_start_from_down(&down));
    let session = proof_collection_browser_scope_box_select_session_from_down(
        &down,
        vec![Arc::from("stone-normal")],
    );

    assert_eq!(session.pointer_id, PointerId(7));
    assert_eq!(session.origin_local, point(12.0, 24.0));
    assert_eq!(session.current_local, point(12.0, 24.0));
    assert_eq!(session.baseline_selected, vec![Arc::from("stone-normal")]);
    assert!(session.append_mode);
    assert!(!session.threshold_met);
}

#[test]
fn box_select_down_ignores_non_left_or_pressable_origin() {
    assert!(
        !proof_collection_browser_scope_box_select_can_start_from_down(&pointer_down(
            MouseButton::Right,
            point(0.0, 0.0),
            false,
            Modifiers::default(),
        ),)
    );
    assert!(
        !proof_collection_browser_scope_box_select_can_start_from_down(&pointer_down(
            MouseButton::Left,
            point(0.0, 0.0),
            true,
            Modifiers::default(),
        ),)
    );
}

#[test]
fn box_select_move_marks_threshold_for_matching_pointer() {
    let mut state = ProofCollectionBoxSelectState {
        session: Some(session(PointerId(7))),
    };

    assert!(
        proof_collection_browser_scope_box_select_session_for_move(
            &mut state,
            &pointer_move(PointerId(8), point(32.0, 0.0), true),
        )
        .is_none()
    );
    assert_eq!(
        state.session.as_ref().map(|session| session.current_local),
        Some(point(0.0, 0.0))
    );

    let session = proof_collection_browser_scope_box_select_session_for_move(
        &mut state,
        &pointer_move(PointerId(7), point(7.0, 0.0), true),
    )
    .expect("matching left move should update session");

    assert_eq!(session.current_local, point(7.0, 0.0));
    assert!(session.threshold_met);
}

#[test]
fn box_select_move_ignores_released_left_button() {
    let mut state = ProofCollectionBoxSelectState {
        session: Some(session(PointerId(7))),
    };

    assert!(
        proof_collection_browser_scope_box_select_session_for_move(
            &mut state,
            &pointer_move(PointerId(7), point(7.0, 0.0), false),
        )
        .is_none()
    );
    assert!(!state.session.as_ref().unwrap().threshold_met);
}

#[test]
fn box_select_up_restores_mismatched_pointer_and_takes_matching_session() {
    let mut state = ProofCollectionBoxSelectState {
        session: Some(session(PointerId(7))),
    };

    assert!(
        proof_collection_browser_scope_box_select_session_for_up(
            &mut state,
            &pointer_up(PointerId(8), point(12.0, 0.0)),
        )
        .is_none()
    );
    assert!(state.session.is_some());

    let session = proof_collection_browser_scope_box_select_session_for_up(
        &mut state,
        &pointer_up(PointerId(7), point(7.0, 0.0)),
    )
    .expect("matching pointer up should finish session");

    assert!(state.session.is_none());
    assert_eq!(session.current_local, point(7.0, 0.0));
    assert!(session.threshold_met);
}

#[test]
fn box_select_cancel_clears_matching_pointer_only() {
    let mut state = ProofCollectionBoxSelectState {
        session: Some(session(PointerId(7))),
    };

    assert!(!proof_collection_browser_scope_box_select_cancel_pointer(
        &mut state,
        &pointer_cancel(PointerId(8)),
    ));
    assert!(state.session.is_some());

    assert!(proof_collection_browser_scope_box_select_cancel_pointer(
        &mut state,
        &pointer_cancel(PointerId(7)),
    ));
    assert!(state.session.is_none());
}
