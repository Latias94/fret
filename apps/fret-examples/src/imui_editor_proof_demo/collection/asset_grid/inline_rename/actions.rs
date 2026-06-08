use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::action::{ActionCx, UiActionHostExt as _, UiFocusActionHost};
use fret_ui_editor::controls::TextFieldOutcome;
use fret_ui_editor::primitives::EditSessionOutcome;

use super::super::super::ProofCollectionAsset;
use super::super::super::readouts::{
    proof_collection_rename_cancel_status, proof_collection_rename_commit_status,
    proof_collection_rename_invalid_status,
};
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
        let _ = host.update_model(&models.assets, |assets| {
            *assets = commit.renamed_assets.clone();
        });
        let _ = host.update_model(&models.rename_status, |status| {
            status.clear();
            status.push_str(&proof_collection_rename_commit_status(
                commit.previous_label.as_ref(),
                commit.next_label.as_ref(),
            ));
        });
        let _ = host.update_model(&models.rename_session, |state| {
            *state = None;
        });
        let _ = host.update_model(&models.rename_focus_pending, |state| {
            *state = false;
        });
        proof_collection_restore_focus_after_inline_rename(
            host,
            action_cx,
            &models.active_focus_target,
        );
    } else {
        let _ = host.update_model(&models.rename_status, |status| {
            status.clear();
            status.push_str(&proof_collection_rename_invalid_status(
                session.original_label.as_ref(),
            ));
        });
        let _ = host.update_model(&models.rename_focus_pending, |state| {
            *state = true;
        });
        host.request_redraw(action_cx.window);
    }
}

fn proof_collection_inline_rename_apply_cancel(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    models: &ProofCollectionInlineRenameOutcomeModels,
    session: ProofCollectionRenameSession,
) {
    let _ = host.update_model(&models.rename_status, |status| {
        status.clear();
        status.push_str(&proof_collection_rename_cancel_status(
            session.original_label.as_ref(),
        ));
    });
    let _ = host.update_model(&models.rename_session, |state| {
        *state = None;
    });
    let _ = host.update_model(&models.rename_focus_pending, |state| {
        *state = false;
    });
    proof_collection_restore_focus_after_inline_rename(
        host,
        action_cx,
        &models.active_focus_target,
    );
}
