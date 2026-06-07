use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::{ImUiFacade, UiWriterImUiFacadeExt};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{GlobalElementId, UiHost};
use fret_ui_editor::controls::{
    TextField, TextFieldBlurBehavior, TextFieldOptions, TextFieldOutcome,
};
use fret_ui_editor::primitives::{EditSessionOutcome, EditorTextSelectionBehavior};

use super::super::super::KernelApp;
use super::super::ProofCollectionAsset;
use super::super::readouts::{
    proof_collection_rename_cancel_status, proof_collection_rename_commit_status,
    proof_collection_rename_invalid_status,
};
use super::super::rename::{
    proof_collection_commit_rename, proof_collection_inline_rename_focus_state,
    proof_collection_restore_focus_after_inline_rename, proof_collection_sync_inline_rename_focus,
};
use super::ProofCollectionAssetGridModels;

pub(super) fn render_collection_inline_rename_field(
    ui: &mut ImUiFacade<'_, '_, KernelApp>,
    models: &ProofCollectionAssetGridModels,
    asset: &ProofCollectionAsset,
    rename_focus_pending: bool,
) {
    let rename_input_id = Rc::new(Cell::new(None::<GlobalElementId>));
    let rename_session_model_for_outcome = models.rename_session.clone();
    let rename_draft_model_for_outcome = models.rename_draft.clone();
    let rename_assets_model_for_outcome = models.assets.clone();
    let rename_status_model_for_outcome = models.rename_status.clone();
    let rename_focus_pending_model_for_outcome = models.rename_focus_pending.clone();
    let rename_restore_focus_target_model = models.active_focus_target.clone();
    let inline_test_id: Arc<str> = Arc::from(format!(
        "imui-editor-proof.authoring.imui.collection.asset.{}.rename.inline",
        asset.id
    ));
    let inline_id_source: Arc<str> = Arc::from(format!(
        "imui-editor-proof.authoring.imui.collection.asset.{}.rename.inline",
        asset.id
    ));
    let field = TextField::new(models.rename_draft.clone())
        .on_outcome(Some(Arc::new(
            move |host, action_cx, outcome: TextFieldOutcome| {
                let session = host
                    .models_mut()
                    .read(&rename_session_model_for_outcome, |state| state.clone())
                    .ok()
                    .flatten();
                let Some(session) = session else {
                    return;
                };

                match outcome {
                    EditSessionOutcome::Committed => {
                        let draft = host
                            .models_mut()
                            .read(&rename_draft_model_for_outcome, |state| state.clone())
                            .unwrap_or_default();
                        let stored_assets = host
                            .models_mut()
                            .read(&rename_assets_model_for_outcome, |state| state.clone())
                            .unwrap_or_default();
                        if let Some(commit) =
                            proof_collection_commit_rename(&stored_assets, &session, &draft)
                        {
                            let _ = host.update_model(&rename_assets_model_for_outcome, |assets| {
                                *assets = commit.renamed_assets.clone();
                            });
                            let _ = host.update_model(&rename_status_model_for_outcome, |status| {
                                status.clear();
                                status.push_str(&proof_collection_rename_commit_status(
                                    commit.previous_label.as_ref(),
                                    commit.next_label.as_ref(),
                                ));
                            });
                            let _ = host.update_model(&rename_session_model_for_outcome, |state| {
                                *state = None;
                            });
                            let _ = host.update_model(
                                &rename_focus_pending_model_for_outcome,
                                |state| {
                                    *state = false;
                                },
                            );
                            proof_collection_restore_focus_after_inline_rename(
                                host,
                                action_cx,
                                &rename_restore_focus_target_model,
                            );
                        } else {
                            let _ = host.update_model(&rename_status_model_for_outcome, |status| {
                                status.clear();
                                status.push_str(&proof_collection_rename_invalid_status(
                                    session.original_label.as_ref(),
                                ));
                            });
                            let _ = host.update_model(
                                &rename_focus_pending_model_for_outcome,
                                |state| {
                                    *state = true;
                                },
                            );
                            host.request_redraw(action_cx.window);
                        }
                    }
                    EditSessionOutcome::Canceled => {
                        let _ = host.update_model(&rename_status_model_for_outcome, |status| {
                            status.clear();
                            status.push_str(&proof_collection_rename_cancel_status(
                                session.original_label.as_ref(),
                            ));
                        });
                        let _ = host.update_model(&rename_session_model_for_outcome, |state| {
                            *state = None;
                        });
                        let _ =
                            host.update_model(&rename_focus_pending_model_for_outcome, |state| {
                                *state = false;
                            });
                        proof_collection_restore_focus_after_inline_rename(
                            host,
                            action_cx,
                            &rename_restore_focus_target_model,
                        );
                    }
                }
            },
        )))
        .options(TextFieldOptions {
            id_source: Some(inline_id_source),
            placeholder: Some(Arc::from("Rename active asset")),
            selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
            blur_behavior: TextFieldBlurBehavior::Cancel,
            test_id: Some(inline_test_id),
            input_id_out: Some(rename_input_id.clone()),
            ..Default::default()
        })
        .into_element(ui.cx_mut());
    ui.add(field);
    if let Some(input_id) = rename_input_id.get() {
        let focus_state = proof_collection_inline_rename_focus_state(ui.cx_mut());
        proof_collection_sync_inline_rename_focus(
            ui.cx_mut(),
            input_id,
            rename_focus_pending,
            &models.rename_focus_pending,
            &focus_state,
        );
    }
    ui.text_wrapped(
        "Inline rename stays app-owned: Enter commits; Escape or blur cancels without widening shared IMUI helpers.",
    );
}
