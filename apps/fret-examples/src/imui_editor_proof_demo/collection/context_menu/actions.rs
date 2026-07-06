use super::super::super::KernelApp;
use super::super::model_owner::ProofCollectionModelOwner;
use super::super::rename::ProofCollectionRenameSession;
use super::super::selection::{ProofCollectionDeleteResult, ProofCollectionDuplicateResult};
use super::ProofCollectionContextMenuModels;

pub(super) fn proof_collection_context_menu_apply_duplicate(
    app: &mut KernelApp,
    models: &ProofCollectionContextMenuModels,
    duplicate: ProofCollectionDuplicateResult,
) {
    ProofCollectionModelOwner::new(app.models_mut()).apply_duplicate(
        &models.assets,
        &models.selection,
        &models.keyboard,
        &models.command_status,
        duplicate,
    );
}

pub(super) fn proof_collection_context_menu_begin_rename(
    app: &mut KernelApp,
    models: &ProofCollectionContextMenuModels,
    session: &ProofCollectionRenameSession,
) {
    ProofCollectionModelOwner::new(app.models_mut()).begin_inline_rename(
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
    ProofCollectionModelOwner::new(app.models_mut()).apply_delete(
        &models.assets,
        &models.selection,
        &models.keyboard,
        &models.command_status,
        delete,
    );
}
