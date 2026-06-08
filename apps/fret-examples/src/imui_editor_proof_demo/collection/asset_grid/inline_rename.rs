use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::{ImUiFacade, UiWriterImUiFacadeExt};
use fret_ui::{GlobalElementId, UiHost};
use fret_ui_editor::controls::{
    TextField, TextFieldBlurBehavior, TextFieldOptions, TextFieldOutcome,
};
use fret_ui_editor::primitives::EditorTextSelectionBehavior;

use super::super::super::KernelApp;
use super::super::ProofCollectionAsset;
use super::super::rename::{
    proof_collection_inline_rename_focus_state, proof_collection_sync_inline_rename_focus,
};
use super::ProofCollectionAssetGridModels;

mod actions;

use actions::{
    ProofCollectionInlineRenameOutcomeModels, proof_collection_inline_rename_apply_outcome,
};

pub(super) fn render_collection_inline_rename_field(
    ui: &mut ImUiFacade<'_, '_, KernelApp>,
    models: &ProofCollectionAssetGridModels,
    asset: &ProofCollectionAsset,
    rename_focus_pending: bool,
) {
    let rename_input_id = Rc::new(Cell::new(None::<GlobalElementId>));
    let outcome_models = ProofCollectionInlineRenameOutcomeModels {
        assets: models.assets.clone(),
        rename_session: models.rename_session.clone(),
        rename_draft: models.rename_draft.clone(),
        rename_focus_pending: models.rename_focus_pending.clone(),
        rename_status: models.rename_status.clone(),
        active_focus_target: models.active_focus_target.clone(),
    };
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
                proof_collection_inline_rename_apply_outcome(
                    host,
                    action_cx,
                    &outcome_models,
                    outcome,
                );
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
