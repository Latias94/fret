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
mod tests;
