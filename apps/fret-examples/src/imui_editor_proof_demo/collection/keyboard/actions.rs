use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_ui::action::{ActionCx, UiActionHostExt as _, UiFocusActionHost};

use super::super::readouts::{
    proof_collection_delete_status, proof_collection_duplicate_status,
    proof_collection_rename_ready_status, proof_collection_select_all_status,
};
use super::super::rename::ProofCollectionRenameSession;
use super::super::selection::{
    ProofCollectionDeleteResult, ProofCollectionDuplicateResult, ProofCollectionKeyboardState,
};
use super::ProofCollectionKeyboardHandlerModels;

pub(super) fn proof_collection_keyboard_apply_delete(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &ProofCollectionKeyboardHandlerModels,
    delete: ProofCollectionDeleteResult,
) {
    let next_status = proof_collection_delete_status(&delete.deleted_assets);
    let _ = host.update_model(&models.assets, |state| {
        *state = delete.remaining_assets.clone();
    });
    let _ = host.update_model(&models.selection, |state| {
        *state = delete.next_selection.clone();
    });
    let _ = host.update_model(&models.keyboard, |state| {
        *state = delete.next_keyboard.clone();
    });
    let _ = host.update_model(&models.command_status, |status| {
        status.clear();
        status.push_str(&next_status);
    });
    host.notify(acx);
}

pub(super) fn proof_collection_keyboard_begin_rename(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &ProofCollectionKeyboardHandlerModels,
    session: ProofCollectionRenameSession,
) {
    let _ = host.update_model(&models.rename_session, |state| {
        *state = Some(session.clone());
    });
    let _ = host.update_model(&models.rename_draft, |draft| {
        draft.clear();
        draft.push_str(session.original_label.as_ref());
    });
    let _ = host.update_model(&models.rename_focus_pending, |state| {
        *state = true;
    });
    let _ = host.update_model(&models.rename_status, |status| {
        status.clear();
        status.push_str(&proof_collection_rename_ready_status(
            session.original_label.as_ref(),
        ));
    });
    host.notify(acx);
}

pub(super) fn proof_collection_keyboard_apply_select_all(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &ProofCollectionKeyboardHandlerModels,
    next_selection: ImUiMultiSelectState<Arc<str>>,
    next_keyboard: ProofCollectionKeyboardState,
) {
    let next_status = proof_collection_select_all_status(next_selection.selected_count());
    let _ = host.update_model(&models.selection, |state| {
        *state = next_selection.clone();
    });
    let _ = host.update_model(&models.keyboard, |state| {
        *state = next_keyboard.clone();
    });
    let _ = host.update_model(&models.command_status, |status| {
        status.clear();
        status.push_str(&next_status);
    });
    host.notify(acx);
}

pub(super) fn proof_collection_keyboard_apply_duplicate(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &ProofCollectionKeyboardHandlerModels,
    duplicate: ProofCollectionDuplicateResult,
) {
    let next_status = proof_collection_duplicate_status(&duplicate.duplicated_assets);
    let _ = host.update_model(&models.assets, |state| {
        *state = duplicate.next_assets.clone();
    });
    let _ = host.update_model(&models.selection, |state| {
        *state = duplicate.next_selection.clone();
    });
    let _ = host.update_model(&models.keyboard, |state| {
        *state = duplicate.next_keyboard.clone();
    });
    let _ = host.update_model(&models.command_status, |status| {
        status.clear();
        status.push_str(&next_status);
    });
    host.notify(acx);
}

pub(super) fn proof_collection_keyboard_apply_navigation(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &ProofCollectionKeyboardHandlerModels,
    next_selection: ImUiMultiSelectState<Arc<str>>,
    next_keyboard: ProofCollectionKeyboardState,
) {
    let _ = host.update_model(&models.selection, |state| {
        *state = next_selection.clone();
    });
    let _ = host.update_model(&models.keyboard, |state| {
        *state = next_keyboard.clone();
    });
    host.notify(acx);
}
