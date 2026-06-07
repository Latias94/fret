use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{MouseButton, Point};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::action::{
    ActionCx, PointerCancelCx, PointerDownCx, PointerMoveCx, PointerUpCx, UiActionHostExt as _,
    UiPointerActionHost,
};

use super::super::super::KernelApp;
use super::super::super::box_select::{
    ProofCollectionBoxSelectSession, ProofCollectionBoxSelectState, ProofCollectionRenderedItem,
    proof_collection_box_select_selection,
};
use super::super::super::geometry::proof_collection_drag_threshold_met;
use super::super::super::selection::ProofCollectionKeyboardState;

type BeforeCollectionBrowserScopeBoxSelectPointerUp =
    Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, &PointerUpCx) -> bool + 'static>;

pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeModels {
    pub(super) selection: Model<ImUiMultiSelectState<Arc<str>>>,
    pub(super) keyboard: Model<ProofCollectionKeyboardState>,
    pub(super) box_select: Model<ProofCollectionBoxSelectState>,
}

pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeState {
    pub(super) collection_keys: Vec<Arc<str>>,
    pub(super) rendered_items: Rc<RefCell<Vec<ProofCollectionRenderedItem>>>,
}

pub(super) fn install_collection_browser_scope_box_select_runtime(
    cx: &mut ElementContext<'_, KernelApp>,
    models: ProofCollectionBrowserScopeBoxSelectRuntimeModels,
    state: ProofCollectionBrowserScopeBoxSelectRuntimeState,
    before_box_select_pointer_up: BeforeCollectionBrowserScopeBoxSelectPointerUp,
) {
    let rendered_items_for_move = state.rendered_items.clone();
    let rendered_items_for_up = state.rendered_items;
    let selection_model_for_down = models.selection.clone();
    let selection_model_for_move = models.selection.clone();
    let selection_model_for_up = models.selection.clone();
    let keyboard_model_for_move = models.keyboard.clone();
    let keyboard_model_for_up = models.keyboard.clone();
    let keyboard_model_for_clear = models.keyboard.clone();
    let box_select_model_for_down = models.box_select.clone();
    let box_select_model_for_move = models.box_select.clone();
    let box_select_model_for_up = models.box_select.clone();
    let box_select_model_for_cancel = models.box_select;
    let collection_keys_for_move = state.collection_keys.clone();
    let collection_keys_for_up = state.collection_keys;

    cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
        if down.button != MouseButton::Left {
            return false;
        }

        host.request_focus(acx.target);
        if !proof_collection_browser_scope_box_select_can_start_from_down(&down) {
            return false;
        }

        let baseline_selected = host
            .models_mut()
            .read(&selection_model_for_down, |state| state.selected().to_vec())
            .unwrap_or_default();
        let session =
            proof_collection_browser_scope_box_select_session_from_down(&down, baseline_selected);
        let _ = host.update_model(&box_select_model_for_down, |state| {
            state.session = Some(session);
        });
        host.capture_pointer();
        host.notify(acx);
        true
    }));

    cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
        let session = host
            .update_model(&box_select_model_for_move, |state| {
                proof_collection_browser_scope_box_select_session_for_move(state, &mv)
            })
            .flatten();

        let Some(session) = session else {
            return false;
        };

        publish_collection_browser_scope_box_select_threshold_selection(
            host,
            &selection_model_for_move,
            &keyboard_model_for_move,
            &collection_keys_for_move,
            &rendered_items_for_move,
            &session,
        );

        host.notify(acx);
        true
    }));

    cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
        if before_box_select_pointer_up(host, acx, &up) {
            return true;
        }

        let session = host
            .update_model(&box_select_model_for_up, |state| {
                proof_collection_browser_scope_box_select_session_for_up(state, &up)
            })
            .flatten();

        let Some(session) = session else {
            return false;
        };

        host.release_pointer_capture();
        if session.threshold_met {
            publish_collection_browser_scope_box_select_threshold_selection(
                host,
                &selection_model_for_up,
                &keyboard_model_for_up,
                &collection_keys_for_up,
                &rendered_items_for_up,
                &session,
            );
        } else if !session.append_mode {
            let _ = host.update_model(&selection_model_for_up, |state| {
                state.clear();
            });
            let _ = host.update_model(&keyboard_model_for_clear, |state| {
                state.active_id = None;
            });
        }

        host.notify(acx);
        true
    }));

    cx.pointer_region_on_pointer_cancel(Arc::new(move |host, _acx, cancel| {
        let cleared = host
            .update_model(&box_select_model_for_cancel, |state| {
                proof_collection_browser_scope_box_select_cancel_pointer(state, &cancel)
            })
            .unwrap_or(false);
        if cleared {
            host.release_pointer_capture();
        }
        cleared
    }));
}

fn proof_collection_browser_scope_box_select_can_start_from_down(down: &PointerDownCx) -> bool {
    down.button == MouseButton::Left && !down.hit_is_pressable
}

fn proof_collection_browser_scope_box_select_session_from_down(
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

fn proof_collection_browser_scope_box_select_session_for_move(
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

fn proof_collection_browser_scope_box_select_session_for_up(
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

fn proof_collection_browser_scope_box_select_cancel_pointer(
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

fn publish_collection_browser_scope_box_select_threshold_selection(
    host: &mut dyn UiPointerActionHost,
    selection_model: &Model<ImUiMultiSelectState<Arc<str>>>,
    keyboard_model: &Model<ProofCollectionKeyboardState>,
    collection_keys: &[Arc<str>],
    rendered_items: &Rc<RefCell<Vec<ProofCollectionRenderedItem>>>,
    session: &ProofCollectionBoxSelectSession,
) {
    if !session.threshold_met {
        return;
    }

    let next_selection =
        proof_collection_box_select_selection(collection_keys, &rendered_items.borrow(), session);
    let _ = host.update_model(selection_model, |state| {
        *state = next_selection.clone();
    });
    let _ = host.update_model(keyboard_model, |state| {
        state.active_id = next_selection.first_selected().cloned();
    });
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
