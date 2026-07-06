use std::sync::Arc;

use fret::advanced::view::AppRenderDataExt as _;
use fret::imui::{kit::ImUiMultiSelectState, prelude::*};
use fret_core::{Point, Px, Rect, Size};
use fret_runtime::Model;

use super::super::KernelApp;
use super::ProofCollectionAsset;
use super::model_owner::ProofCollectionModelOwner;
use super::proof_collection_readout_text;
use super::rename::{ProofCollectionRenameSession, proof_collection_begin_rename_session};
use super::selection::{
    ProofCollectionKeyboardState, proof_collection_assets_in_visible_order,
    proof_collection_delete_selection, proof_collection_duplicate_selection,
};

mod actions;
mod chrome;

use actions::{
    proof_collection_context_menu_apply_delete, proof_collection_context_menu_apply_duplicate,
    proof_collection_context_menu_begin_rename,
};
use chrome::{
    collection_context_menu_delete_selected_label, collection_context_menu_delete_selected_options,
    collection_context_menu_dismiss_label, collection_context_menu_dismiss_options,
    collection_context_menu_duplicate_selected_label,
    collection_context_menu_duplicate_selected_options, collection_context_menu_popup_id,
    collection_context_menu_rename_active_label, collection_context_menu_rename_active_options,
    collection_context_menu_selection_readout_id,
};

pub(super) struct ProofCollectionContextMenuModels {
    pub(super) anchor: Model<Option<Point>>,
    pub(super) selection: Model<ImUiMultiSelectState<Arc<str>>>,
    pub(super) keyboard: Model<ProofCollectionKeyboardState>,
    pub(super) assets: Model<Vec<ProofCollectionAsset>>,
    pub(super) reverse_order: Model<bool>,
    pub(super) command_status: Model<String>,
    pub(super) rename_session: Model<Option<ProofCollectionRenameSession>>,
    pub(super) rename_draft: Model<String>,
    pub(super) rename_focus_pending: Model<bool>,
    pub(super) rename_status: Model<String>,
}

pub(super) fn render_collection_context_menu(
    ui: &mut ImUi<'_, '_, KernelApp>,
    models: ProofCollectionContextMenuModels,
) {
    let anchor = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.anchor, |state| state);
    if let Some(anchor) = anchor {
        ui.open_popup_at(
            collection_context_menu_popup_id(),
            Rect::new(anchor, Size::new(Px(1.0), Px(1.0))),
        );
        ProofCollectionModelOwner::new(ui.cx_mut().app.models_mut())
            .clear_context_menu_anchor(&models.anchor);
    }

    let popup_open = ui.popup_open_model(collection_context_menu_popup_id());
    let popup_selection = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.selection, |state| state);
    let popup_keyboard = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.keyboard, |state| state);
    let popup_assets = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.assets, |state| state.clone());
    let popup_reverse_order = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.reverse_order, |state| state);
    let popup_visible_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(popup_assets.clone()),
        popup_reverse_order,
    );

    ui.begin_popup_menu(collection_context_menu_popup_id(), None, |ui| {
        let rename_session = proof_collection_begin_rename_session(
            &popup_visible_assets,
            &popup_selection,
            &popup_keyboard,
        );
        proof_collection_readout_text(
            ui,
            format!("Selection: {} item(s)", popup_selection.selected_count()),
            collection_context_menu_selection_readout_id(),
        );
        ui.separator();

        let duplicate_from_menu = ui.menu_item_with_options(
            collection_context_menu_duplicate_selected_label(),
            collection_context_menu_duplicate_selected_options(
                !popup_selection.is_empty(),
                popup_open.clone(),
            ),
        );
        if duplicate_from_menu.clicked()
            && let Some(duplicate) = proof_collection_duplicate_selection(
                &popup_visible_assets,
                &popup_assets,
                &popup_selection,
                &popup_keyboard,
                popup_reverse_order,
            )
        {
            proof_collection_context_menu_apply_duplicate(ui.cx_mut().app, &models, duplicate);
        }

        let rename_from_menu = ui.menu_item_with_options(
            collection_context_menu_rename_active_label(),
            collection_context_menu_rename_active_options(
                rename_session.is_some(),
                popup_open.clone(),
            ),
        );
        if rename_from_menu.clicked()
            && let Some(session) = rename_session
        {
            proof_collection_context_menu_begin_rename(ui.cx_mut().app, &models, &session);
        }

        let delete_from_menu = ui.menu_item_with_options(
            collection_context_menu_delete_selected_label(),
            collection_context_menu_delete_selected_options(
                !popup_selection.is_empty(),
                popup_open.clone(),
            ),
        );
        if delete_from_menu.clicked()
            && let Some(delete) = proof_collection_delete_selection(
                &popup_visible_assets,
                &popup_assets,
                &popup_selection,
                &popup_keyboard,
            )
        {
            proof_collection_context_menu_apply_delete(ui.cx_mut().app, &models, delete);
        }

        let _ = ui.menu_item_with_options(
            collection_context_menu_dismiss_label(),
            collection_context_menu_dismiss_options(popup_open.clone()),
        );
    });
}
