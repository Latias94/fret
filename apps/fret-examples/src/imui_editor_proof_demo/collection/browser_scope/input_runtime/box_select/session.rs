use std::sync::Arc;

use fret_core::{MouseButton, Point};
use fret_ui::action::{PointerCancelCx, PointerDownCx, PointerMoveCx, PointerUpCx};

use super::super::super::super::box_select::{
    ProofCollectionBoxSelectSession, ProofCollectionBoxSelectState,
};
use super::super::super::super::geometry::proof_collection_drag_threshold_met;

pub(super) fn proof_collection_browser_scope_box_select_can_start_from_down(
    down: &PointerDownCx,
) -> bool {
    down.button == MouseButton::Left && !down.hit_is_pressable
}

pub(super) fn proof_collection_browser_scope_box_select_session_from_down(
    down: &PointerDownCx,
    baseline_selected: Vec<Arc<str>>,
) -> ProofCollectionBoxSelectSession {
    ProofCollectionBoxSelectSession {
        pointer_id: down.pointer_id,
        origin_local: down.position_local,
        current_local: down.position_local,
        baseline_selected,
        append_mode: down.modifiers.ctrl || down.modifiers.meta,
        threshold_met: false,
    }
}

fn proof_collection_browser_scope_box_select_update_session_position(
    session: &mut ProofCollectionBoxSelectSession,
    position_local: Point,
) {
    session.current_local = position_local;
    if !session.threshold_met {
        session.threshold_met =
            proof_collection_drag_threshold_met(session.origin_local, session.current_local);
    }
}

pub(super) fn proof_collection_browser_scope_box_select_session_for_move(
    state: &mut ProofCollectionBoxSelectState,
    mv: &PointerMoveCx,
) -> Option<ProofCollectionBoxSelectSession> {
    if !mv.buttons.left {
        return None;
    }

    let session = state.session.as_mut()?;
    if session.pointer_id != mv.pointer_id {
        return None;
    }

    proof_collection_browser_scope_box_select_update_session_position(session, mv.position_local);
    Some(session.clone())
}

pub(super) fn proof_collection_browser_scope_box_select_session_for_up(
    state: &mut ProofCollectionBoxSelectState,
    up: &PointerUpCx,
) -> Option<ProofCollectionBoxSelectSession> {
    let Some(mut session) = state.session.take() else {
        return None;
    };
    if session.pointer_id != up.pointer_id {
        state.session = Some(session);
        return None;
    }

    proof_collection_browser_scope_box_select_update_session_position(
        &mut session,
        up.position_local,
    );
    Some(session)
}

pub(super) fn proof_collection_browser_scope_box_select_cancel_pointer(
    state: &mut ProofCollectionBoxSelectState,
    cancel: &PointerCancelCx,
) -> bool {
    let matches_pointer = state
        .session
        .as_ref()
        .is_some_and(|session| session.pointer_id == cancel.pointer_id);
    if matches_pointer {
        state.session = None;
    }
    matches_pointer
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::{Modifiers, MouseButtons, PointerCancelReason, PointerId, PointerType, Px};
    use fret_runtime::TickId;

    fn point(x: f32, y: f32) -> Point {
        Point::new(Px(x), Px(y))
    }

    fn pointer_down(
        button: MouseButton,
        position: Point,
        hit_is_pressable: bool,
        modifiers: Modifiers,
    ) -> PointerDownCx {
        PointerDownCx {
            pointer_id: PointerId(7),
            position,
            position_local: position,
            position_window: Some(position),
            tick_id: TickId(0),
            pixels_per_point: 1.0,
            button,
            modifiers,
            click_count: 1,
            pointer_type: PointerType::Mouse,
            hit_is_text_input: false,
            hit_is_pressable,
            hit_pressable_target: None,
            hit_pressable_target_in_descendant_subtree: false,
        }
    }

    fn pointer_move(pointer_id: PointerId, position: Point, left: bool) -> PointerMoveCx {
        PointerMoveCx {
            pointer_id,
            position,
            position_local: position,
            position_window: Some(position),
            tick_id: TickId(0),
            pixels_per_point: 1.0,
            velocity_window: None,
            buttons: MouseButtons {
                left,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }
    }

    fn pointer_up(pointer_id: PointerId, position: Point) -> PointerUpCx {
        PointerUpCx {
            pointer_id,
            position,
            position_local: position,
            position_window: Some(position),
            tick_id: TickId(0),
            pixels_per_point: 1.0,
            velocity_window: None,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: PointerType::Mouse,
            down_hit_pressable_target: None,
            down_hit_pressable_target_in_descendant_subtree: false,
        }
    }

    fn pointer_cancel(pointer_id: PointerId) -> PointerCancelCx {
        PointerCancelCx {
            pointer_id,
            position: None,
            position_local: None,
            position_window: None,
            tick_id: TickId(0),
            pixels_per_point: 1.0,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            reason: PointerCancelReason::LeftWindow,
        }
    }

    fn session(pointer_id: PointerId) -> ProofCollectionBoxSelectSession {
        ProofCollectionBoxSelectSession {
            pointer_id,
            origin_local: point(0.0, 0.0),
            current_local: point(0.0, 0.0),
            baseline_selected: vec![Arc::from("stone-albedo")],
            append_mode: false,
            threshold_met: false,
        }
    }

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
            ))
        );
        assert!(
            !proof_collection_browser_scope_box_select_can_start_from_down(&pointer_down(
                MouseButton::Left,
                point(0.0, 0.0),
                true,
                Modifiers::default(),
            ))
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
}
