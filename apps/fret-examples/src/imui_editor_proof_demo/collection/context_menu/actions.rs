use super::super::super::KernelApp;
use super::super::readouts::{proof_collection_delete_status, proof_collection_duplicate_status};
use super::super::rename::{
    ProofCollectionRenameSession, proof_collection_begin_inline_rename_in_app,
};
use super::super::selection::{ProofCollectionDeleteResult, ProofCollectionDuplicateResult};
use super::ProofCollectionContextMenuModels;
use fret_ui::action::UiActionHostExt as _;

pub(super) fn proof_collection_context_menu_apply_duplicate(
    app: &mut KernelApp,
    models: &ProofCollectionContextMenuModels,
    duplicate: ProofCollectionDuplicateResult,
) {
    let command_status = proof_collection_duplicate_status(&duplicate.duplicated_assets);
    let _ = app.models_mut().update(&models.assets, |state| {
        *state = duplicate.next_assets.clone();
    });
    let _ = app.models_mut().update(&models.selection, |state| {
        *state = duplicate.next_selection.clone();
    });
    let _ = app.models_mut().update(&models.keyboard, |state| {
        *state = duplicate.next_keyboard.clone();
    });
    let _ = app.models_mut().update(&models.command_status, |status| {
        status.clear();
        status.push_str(&command_status);
    });
}

pub(super) fn proof_collection_context_menu_begin_rename(
    app: &mut KernelApp,
    models: &ProofCollectionContextMenuModels,
    session: &ProofCollectionRenameSession,
) {
    proof_collection_begin_inline_rename_in_app(
        app,
        &models.rename_session,
        &models.rename_draft,
        &models.rename_focus_pending,
        &models.rename_status,
        session,
    );
}

pub(super) fn proof_collection_context_menu_apply_delete(
    app: &mut KernelApp,
    models: &ProofCollectionContextMenuModels,
    delete: ProofCollectionDeleteResult,
) {
    let command_status = proof_collection_delete_status(&delete.deleted_assets);
    let _ = app.models_mut().update(&models.assets, |state| {
        *state = delete.remaining_assets.clone();
    });
    let _ = app.models_mut().update(&models.selection, |state| {
        *state = delete.next_selection.clone();
    });
    let _ = app.models_mut().update(&models.keyboard, |state| {
        *state = delete.next_keyboard.clone();
    });
    let _ = app.models_mut().update(&models.command_status, |status| {
        status.clear();
        status.push_str(&command_status);
    });
}
