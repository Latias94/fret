use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::action::{ActionCx, UiFocusActionHost};
use fret_ui_editor::controls::TextFieldOutcome;
use fret_ui_editor::primitives::EditSessionOutcome;

use super::super::super::ProofCollectionAsset;
use super::super::super::model_owner::ProofCollectionModelOwner;
use super::super::super::rename::{
    ProofCollectionRenameSession, proof_collection_commit_rename,
    proof_collection_restore_focus_after_inline_rename,
};

pub(super) struct ProofCollectionInlineRenameOutcomeModels {
    pub(super) assets: Model<Vec<ProofCollectionAsset>>,
    pub(super) rename_session: Model<Option<ProofCollectionRenameSession>>,
    pub(super) rename_draft: Model<String>,
    pub(super) rename_focus_pending: Model<bool>,
    pub(super) rename_status: Model<String>,
    pub(super) active_focus_target: Model<Option<GlobalElementId>>,
}

pub(super) fn proof_collection_inline_rename_apply_outcome(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    models: &ProofCollectionInlineRenameOutcomeModels,
    outcome: TextFieldOutcome,
) {
    let session = host
        .models_mut()
        .read(&models.rename_session, |state| state.clone())
        .ok()
        .flatten();
    let Some(session) = session else {
        return;
    };

    match outcome {
        EditSessionOutcome::Committed => {
            proof_collection_inline_rename_apply_commit(host, action_cx, models, session);
        }
        EditSessionOutcome::Canceled => {
            proof_collection_inline_rename_apply_cancel(host, action_cx, models, session);
        }
    }
}

fn proof_collection_inline_rename_apply_commit(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    models: &ProofCollectionInlineRenameOutcomeModels,
    session: ProofCollectionRenameSession,
) {
    let draft = host
        .models_mut()
        .read(&models.rename_draft, |state| state.clone())
        .unwrap_or_default();
    let stored_assets = host
        .models_mut()
        .read(&models.assets, |state| state.clone())
        .unwrap_or_default();
    if let Some(commit) = proof_collection_commit_rename(&stored_assets, &session, &draft) {
        ProofCollectionModelOwner::new(host.models_mut()).apply_inline_rename_commit(
            &models.assets,
            &models.rename_session,
            &models.rename_focus_pending,
            &models.rename_status,
            commit,
        );
        proof_collection_restore_focus_after_inline_rename(
            host,
            action_cx,
            &models.active_focus_target,
        );
    } else {
        ProofCollectionModelOwner::new(host.models_mut()).reject_inline_rename(
            &models.rename_focus_pending,
            &models.rename_status,
            &session,
        );
        host.request_redraw(action_cx.window);
    }
}

fn proof_collection_inline_rename_apply_cancel(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    models: &ProofCollectionInlineRenameOutcomeModels,
    session: ProofCollectionRenameSession,
) {
    ProofCollectionModelOwner::new(host.models_mut()).cancel_inline_rename(
        &models.rename_session,
        &models.rename_focus_pending,
        &models.rename_status,
        &session,
    );
    proof_collection_restore_focus_after_inline_rename(
        host,
        action_cx,
        &models.active_focus_target,
    );
}
