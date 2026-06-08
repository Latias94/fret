use std::sync::Arc;

use fret::imui::{kit::ImUiMultiSelectState, prelude::*};
use fret_runtime::Model;

mod actions;
mod chrome;

use actions::{
    proof_collection_command_button_apply_delete, proof_collection_command_button_apply_duplicate,
    proof_collection_command_button_begin_rename,
};
use chrome::{
    collection_delete_selected_button_options, collection_delete_selected_label,
    collection_duplicate_selected_button_options, collection_duplicate_selected_label,
    collection_rename_active_button_options, collection_rename_active_label,
};

use super::super::KernelApp;
use super::ProofCollectionAsset;
use super::rename::ProofCollectionRenameSession;
use super::selection::{
    ProofCollectionKeyboardState, proof_collection_delete_selection,
    proof_collection_duplicate_selection,
};

pub(super) struct ProofCollectionCommandButtonModels {
    pub(super) assets: Model<Vec<ProofCollectionAsset>>,
    pub(super) selection: Model<ImUiMultiSelectState<Arc<str>>>,
    pub(super) keyboard: Model<ProofCollectionKeyboardState>,
    pub(super) command_status: Model<String>,
    pub(super) rename_session: Model<Option<ProofCollectionRenameSession>>,
    pub(super) rename_draft: Model<String>,
    pub(super) rename_focus_pending: Model<bool>,
    pub(super) rename_status: Model<String>,
}

pub(super) struct ProofCollectionCommandButtonState<'a> {
    pub(super) visible_assets: &'a [ProofCollectionAsset],
    pub(super) stored_assets: &'a [ProofCollectionAsset],
    pub(super) selection: &'a ImUiMultiSelectState<Arc<str>>,
    pub(super) keyboard: &'a ProofCollectionKeyboardState,
    pub(super) reverse_order: bool,
    pub(super) rename_ready_session: Option<&'a ProofCollectionRenameSession>,
}

pub(super) fn render_collection_command_buttons(
    ui: &mut ImUi<'_, '_, KernelApp>,
    models: ProofCollectionCommandButtonModels,
    state: ProofCollectionCommandButtonState<'_>,
) {
    let duplicate_selected = ui.button_with_options(
        collection_duplicate_selected_label(),
        collection_duplicate_selected_button_options(!state.selection.is_empty()),
    );
    if duplicate_selected.clicked()
        && let Some(duplicate) = proof_collection_duplicate_selection(
            state.visible_assets,
            state.stored_assets,
            state.selection,
            state.keyboard,
            state.reverse_order,
        )
    {
        proof_collection_command_button_apply_duplicate(ui.cx_mut().app, &models, duplicate);
    }

    let rename_active = ui.button_with_options(
        collection_rename_active_label(),
        collection_rename_active_button_options(state.rename_ready_session.is_some()),
    );
    if rename_active.clicked()
        && let Some(session) = state.rename_ready_session
    {
        proof_collection_command_button_begin_rename(ui.cx_mut().app, &models, session);
    }

    let delete_selected = ui.button_with_options(
        collection_delete_selected_label(),
        collection_delete_selected_button_options(!state.selection.is_empty()),
    );
    if delete_selected.clicked()
        && let Some(delete) = proof_collection_delete_selection(
            state.visible_assets,
            state.stored_assets,
            state.selection,
            state.keyboard,
        )
    {
        proof_collection_command_button_apply_delete(ui.cx_mut().app, &models, delete);
    }
}
