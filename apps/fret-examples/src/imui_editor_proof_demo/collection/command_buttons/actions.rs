use fret_runtime::Model;

use super::super::super::KernelApp;
use super::super::readouts::{proof_collection_delete_status, proof_collection_duplicate_status};
use super::super::rename::{
    ProofCollectionRenameSession, proof_collection_begin_inline_rename_in_app,
};
use super::super::selection::{ProofCollectionDeleteResult, ProofCollectionDuplicateResult};
use super::ProofCollectionCommandButtonModels;

pub(super) fn proof_collection_command_button_apply_duplicate(
    app: &mut KernelApp,
    models: &ProofCollectionCommandButtonModels,
    duplicate: ProofCollectionDuplicateResult,
) {
    let command_status = proof_collection_duplicate_status(&duplicate.duplicated_assets);
    let _ = app.models_mut().update(&models.assets, |assets| {
        *assets = duplicate.next_assets.clone();
    });
    let _ = app.models_mut().update(&models.selection, |selection| {
        *selection = duplicate.next_selection.clone();
    });
    let _ = app.models_mut().update(&models.keyboard, |keyboard| {
        *keyboard = duplicate.next_keyboard.clone();
    });
    proof_collection_set_command_status(app, &models.command_status, command_status);
}

pub(super) fn proof_collection_command_button_begin_rename(
    app: &mut KernelApp,
    models: &ProofCollectionCommandButtonModels,
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

pub(super) fn proof_collection_command_button_apply_delete(
    app: &mut KernelApp,
    models: &ProofCollectionCommandButtonModels,
    delete: ProofCollectionDeleteResult,
) {
    let command_status = proof_collection_delete_status(&delete.deleted_assets);
    let _ = app.models_mut().update(&models.assets, |assets| {
        *assets = delete.remaining_assets.clone();
    });
    let _ = app.models_mut().update(&models.selection, |selection| {
        *selection = delete.next_selection.clone();
    });
    let _ = app.models_mut().update(&models.keyboard, |keyboard| {
        *keyboard = delete.next_keyboard.clone();
    });
    proof_collection_set_command_status(app, &models.command_status, command_status);
}

fn proof_collection_set_command_status(
    app: &mut KernelApp,
    command_status_model: &Model<String>,
    next_status: String,
) {
    let _ = app.models_mut().update(command_status_model, |status| {
        status.clear();
        status.push_str(&next_status);
    });
}
