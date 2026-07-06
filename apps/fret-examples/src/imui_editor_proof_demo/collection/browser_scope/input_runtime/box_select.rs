use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::MouseButton;
use fret_runtime::{Model, ModelStore};
use fret_ui::ElementContext;
use fret_ui::action::{ActionCx, PointerCancelCx, PointerMoveCx, PointerUpCx, UiPointerActionHost};

use super::super::super::KernelApp;
use super::super::super::box_select::{
    ProofCollectionBoxSelectSession, ProofCollectionBoxSelectState, ProofCollectionRenderedItem,
    proof_collection_box_select_selection,
};
use super::super::super::model_owner::ProofCollectionModelOwner;
use super::super::super::selection::ProofCollectionKeyboardState;

mod session;

use session::{
    proof_collection_browser_scope_box_select_can_start_from_down,
    proof_collection_browser_scope_box_select_cancel_pointer,
    proof_collection_browser_scope_box_select_session_for_move,
    proof_collection_browser_scope_box_select_session_for_up,
    proof_collection_browser_scope_box_select_session_from_down,
};

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

struct ProofCollectionBrowserScopeBoxSelectModelOwner<'a> {
    models: &'a mut ModelStore,
}

impl<'a> ProofCollectionBrowserScopeBoxSelectModelOwner<'a> {
    fn new(models: &'a mut ModelStore) -> Self {
        Self { models }
    }

    fn update<R>(
        &mut self,
        model: &Model<ProofCollectionBoxSelectState>,
        f: impl FnOnce(&mut ProofCollectionBoxSelectState) -> R,
    ) -> Option<R> {
        self.models.update(model, f).ok()
    }

    fn begin_session(
        &mut self,
        model: &Model<ProofCollectionBoxSelectState>,
        session: ProofCollectionBoxSelectSession,
    ) {
        let _ = self.update(model, |state| {
            state.session = Some(session);
        });
    }

    fn session_for_move(
        &mut self,
        model: &Model<ProofCollectionBoxSelectState>,
        mv: &PointerMoveCx,
    ) -> Option<ProofCollectionBoxSelectSession> {
        self.update(model, |state| {
            proof_collection_browser_scope_box_select_session_for_move(state, mv)
        })
        .flatten()
    }

    fn session_for_up(
        &mut self,
        model: &Model<ProofCollectionBoxSelectState>,
        up: &PointerUpCx,
    ) -> Option<ProofCollectionBoxSelectSession> {
        self.update(model, |state| {
            proof_collection_browser_scope_box_select_session_for_up(state, up)
        })
        .flatten()
    }

    fn cancel_pointer(
        &mut self,
        model: &Model<ProofCollectionBoxSelectState>,
        cancel: &PointerCancelCx,
    ) -> bool {
        self.update(model, |state| {
            proof_collection_browser_scope_box_select_cancel_pointer(state, cancel)
        })
        .unwrap_or(false)
    }
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
        ProofCollectionBrowserScopeBoxSelectModelOwner::new(host.models_mut())
            .begin_session(&box_select_model_for_down, session);
        host.capture_pointer();
        host.notify(acx);
        true
    }));

    cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
        let session = ProofCollectionBrowserScopeBoxSelectModelOwner::new(host.models_mut())
            .session_for_move(&box_select_model_for_move, &mv);

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

        let session = ProofCollectionBrowserScopeBoxSelectModelOwner::new(host.models_mut())
            .session_for_up(&box_select_model_for_up, &up);

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
            ProofCollectionModelOwner::new(host.models_mut()).apply_navigation(
                &selection_model_for_up,
                &keyboard_model_for_clear,
                ImUiMultiSelectState::default(),
                ProofCollectionKeyboardState::default(),
            );
        }

        host.notify(acx);
        true
    }));

    cx.pointer_region_on_pointer_cancel(Arc::new(move |host, _acx, cancel| {
        let cleared = ProofCollectionBrowserScopeBoxSelectModelOwner::new(host.models_mut())
            .cancel_pointer(&box_select_model_for_cancel, &cancel);
        if cleared {
            host.release_pointer_capture();
        }
        cleared
    }));
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
    let next_keyboard = ProofCollectionKeyboardState {
        active_id: next_selection.first_selected().cloned(),
    };
    ProofCollectionModelOwner::new(host.models_mut()).apply_navigation(
        selection_model,
        keyboard_model,
        next_selection,
        next_keyboard,
    );
}
