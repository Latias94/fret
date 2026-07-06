use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_ui::action::{ActionCx, UiFocusActionHost};

use super::super::model_owner::ProofCollectionModelOwner;
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
    ProofCollectionModelOwner::new(host.models_mut()).apply_delete(
        &models.assets,
        &models.selection,
        &models.keyboard,
        &models.command_status,
        delete,
    );
    host.notify(acx);
}

pub(super) fn proof_collection_keyboard_begin_rename(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &ProofCollectionKeyboardHandlerModels,
    session: ProofCollectionRenameSession,
) {
    ProofCollectionModelOwner::new(host.models_mut()).begin_inline_rename(
        &models.rename_session,
        &models.rename_draft,
        &models.rename_focus_pending,
        &models.rename_status,
        &session,
    );
    host.notify(acx);
}

pub(super) fn proof_collection_keyboard_apply_select_all(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &ProofCollectionKeyboardHandlerModels,
    next_selection: ImUiMultiSelectState<Arc<str>>,
    next_keyboard: ProofCollectionKeyboardState,
) {
    ProofCollectionModelOwner::new(host.models_mut()).apply_select_all(
        &models.selection,
        &models.keyboard,
        &models.command_status,
        next_selection,
        next_keyboard,
    );
    host.notify(acx);
}

pub(super) fn proof_collection_keyboard_apply_duplicate(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &ProofCollectionKeyboardHandlerModels,
    duplicate: ProofCollectionDuplicateResult,
) {
    ProofCollectionModelOwner::new(host.models_mut()).apply_duplicate(
        &models.assets,
        &models.selection,
        &models.keyboard,
        &models.command_status,
        duplicate,
    );
    host.notify(acx);
}

pub(super) fn proof_collection_keyboard_apply_navigation(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &ProofCollectionKeyboardHandlerModels,
    next_selection: ImUiMultiSelectState<Arc<str>>,
    next_keyboard: ProofCollectionKeyboardState,
) {
    ProofCollectionModelOwner::new(host.models_mut()).apply_navigation(
        &models.selection,
        &models.keyboard,
        next_selection,
        next_keyboard,
    );
    host.notify(acx);
}
