use std::sync::Arc;

use fret::advanced::view::AppRenderDataExt as _;
use fret::imui::{kit::ImUiMultiSelectState, prelude::*};
use fret_core::{Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;

use super::super::KernelApp;
use super::ProofCollectionAsset;
use super::proof_collection_readout_text;
use super::readouts::{proof_collection_delete_status, proof_collection_duplicate_status};
use super::rename::{
    ProofCollectionRenameSession, proof_collection_begin_inline_rename_in_app,
    proof_collection_begin_rename_session,
};
use super::selection::{
    ProofCollectionKeyboardState, proof_collection_assets_in_visible_order,
    proof_collection_delete_selection, proof_collection_duplicate_selection,
};

mod chrome;

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
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&models.anchor, |state| *state = None);
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
            let command_status = proof_collection_duplicate_status(&duplicate.duplicated_assets);
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&models.assets, |state| {
                    *state = duplicate.next_assets.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&models.selection, |state| {
                    *state = duplicate.next_selection.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&models.keyboard, |state| {
                    *state = duplicate.next_keyboard.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&models.command_status, |status| {
                    status.clear();
                    status.push_str(&command_status);
                });
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
            proof_collection_begin_inline_rename_in_app(
                ui.cx_mut().app,
                &models.rename_session,
                &models.rename_draft,
                &models.rename_focus_pending,
                &models.rename_status,
                &session,
            );
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
            let command_status = proof_collection_delete_status(&delete.deleted_assets);
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&models.assets, |state| {
                    *state = delete.remaining_assets.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&models.selection, |state| {
                    *state = delete.next_selection.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&models.keyboard, |state| {
                    *state = delete.next_keyboard.clone();
                });
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&models.command_status, |status| {
                    status.clear();
                    status.push_str(&command_status);
                });
        }

        let _ = ui.menu_item_with_options(
            collection_context_menu_dismiss_label(),
            collection_context_menu_dismiss_options(popup_open.clone()),
        );
    });
}
