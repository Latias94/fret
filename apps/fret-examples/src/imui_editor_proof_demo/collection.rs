use std::sync::Arc;

use fret::advanced::view::AppRenderDataExt as _;
use fret::imui::{kit::ImUiMultiSelectState, prelude::*};

use super::KernelApp;

mod asset_grid;
mod assets;
mod box_select;
mod browser_scope;
mod chrome;
mod command_buttons;
mod context_menu;
mod drag_drop;
mod geometry;
mod import_target;
mod keyboard;
mod models;
mod readouts;
mod rename;
mod selection;

pub(super) use assets::{ProofCollectionAsset, authoring_parity_collection_assets};
pub(super) use chrome::proof_collection_readout_text;

use browser_scope::{
    ProofCollectionBrowserScopeModels, ProofCollectionBrowserScopeState,
    render_collection_browser_scope,
};
use chrome::proof_collection_section_label;
use command_buttons::{
    ProofCollectionCommandButtonModels, ProofCollectionCommandButtonState,
    render_collection_command_buttons,
};
use context_menu::{ProofCollectionContextMenuModels, render_collection_context_menu};
use geometry::{
    PROOF_COLLECTION_GRID_FALLBACK_COLUMNS, PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX,
    proof_collection_layout_metrics, proof_collection_zoom_line,
};
use import_target::render_collection_import_target;
use models::{
    authoring_parity_collection_active_focus_target_model,
    authoring_parity_collection_assets_model, authoring_parity_collection_box_select_model,
    authoring_parity_collection_command_status_model,
    authoring_parity_collection_context_menu_anchor_model,
    authoring_parity_collection_keyboard_model, authoring_parity_collection_rename_draft_model,
    authoring_parity_collection_rename_focus_pending_model,
    authoring_parity_collection_rename_session_model,
    authoring_parity_collection_rename_status_model,
    authoring_parity_collection_reverse_order_model, authoring_parity_collection_scroll_handle,
    authoring_parity_collection_selection_model, authoring_parity_collection_zoom_model,
};
use readouts::{
    proof_collection_active_line, proof_collection_assets_line,
    proof_collection_command_package_line, proof_collection_command_status_line,
    proof_collection_context_menu_line, proof_collection_rename_line,
    proof_collection_rename_status_line, proof_collection_select_all_line,
    proof_collection_selection_line, proof_collection_visible_order_line,
};
use rename::proof_collection_begin_rename_session;
use selection::{proof_collection_active_id, proof_collection_assets_in_visible_order};

pub(super) fn render_collection_first_asset_browser_proof(ui: &mut ImUi<'_, '_, KernelApp>) {
    proof_collection_section_label(
        ui,
        "Collection-first asset browser proof",
        "imui-editor-proof.authoring.imui.collection.title",
    );
    ui.text_wrapped(
        "Stable keys keep browser selection pinned while visible order flips and selected-set drag/drop stays app-defined.",
    );
    ui.text_wrapped(
        "Background drag now draws a marquee and updates grid selection app-locally while shared helper widening stays deferred until another first-party proof surface exists.",
    );

    let collection_selection_model = authoring_parity_collection_selection_model(ui.cx_mut());
    let collection_assets_model = authoring_parity_collection_assets_model(ui.cx_mut());
    let collection_reverse_order_model =
        authoring_parity_collection_reverse_order_model(ui.cx_mut());
    let collection_box_select_model = authoring_parity_collection_box_select_model(ui.cx_mut());
    let collection_keyboard_model = authoring_parity_collection_keyboard_model(ui.cx_mut());
    let collection_zoom_model = authoring_parity_collection_zoom_model(ui.cx_mut());
    let collection_context_menu_anchor_model =
        authoring_parity_collection_context_menu_anchor_model(ui.cx_mut());
    let collection_rename_session_model =
        authoring_parity_collection_rename_session_model(ui.cx_mut());
    let collection_rename_draft_model = authoring_parity_collection_rename_draft_model(ui.cx_mut());
    let collection_rename_focus_pending_model =
        authoring_parity_collection_rename_focus_pending_model(ui.cx_mut());
    let collection_active_focus_target_model =
        authoring_parity_collection_active_focus_target_model(ui.cx_mut());
    let collection_rename_status_model =
        authoring_parity_collection_rename_status_model(ui.cx_mut());
    let collection_command_status_model =
        authoring_parity_collection_command_status_model(ui.cx_mut());
    let collection_scroll_handle = authoring_parity_collection_scroll_handle(ui.cx_mut());
    let stored_collection_assets = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_assets_model, |state| state.clone());
    let collection_selection = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_selection_model, |state| state);
    let collection_box_select = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_box_select_model, |state| state);
    let collection_keyboard = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_keyboard_model, |state| state);
    let collection_tile_extent = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_zoom_model, |state| state);
    let mut collection_reverse_order = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_reverse_order_model, |value| value);
    let collection_rename_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_rename_status_model, |state| state.clone());
    let collection_command_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_command_status_model, |state| state.clone());
    let collection_rename_session = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_rename_session_model, |state| state.clone());
    let collection_rename_focus_pending = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_rename_focus_pending_model, |state| state);
    let collection_layout = proof_collection_layout_metrics(
        collection_scroll_handle.viewport_size().width,
        collection_tile_extent,
    );

    let order_toggle = ui.button_with_options(
        if collection_reverse_order {
            "Show folder order"
        } else {
            "Reverse visible order"
        },
        kit::ButtonOptions {
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.order-toggle",
            )),
            ..Default::default()
        },
    );
    if order_toggle.clicked() {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_reverse_order_model, |value| *value = !*value);
        collection_reverse_order = !collection_reverse_order;
    }

    let collection_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(stored_collection_assets.clone()),
        collection_reverse_order,
    );
    let collection_keys = collection_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let collection_active_id = proof_collection_active_id(
        &collection_keys,
        &collection_selection,
        &collection_keyboard,
    );
    let collection_rename_ready_session = proof_collection_begin_rename_session(
        &collection_assets,
        &collection_selection,
        &collection_keyboard,
    );

    proof_collection_readout_text(
        ui,
        proof_collection_assets_line(&collection_assets),
        "imui-editor-proof.authoring.imui.collection.assets-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_visible_order_line(&collection_assets),
        "imui-editor-proof.authoring.imui.collection.visible-order-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_selection_line(&collection_assets, &collection_selection),
        "imui-editor-proof.authoring.imui.collection.selection-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_active_line(
            &collection_assets,
            &collection_selection,
            &collection_keyboard,
        ),
        "imui-editor-proof.authoring.imui.collection.active-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_zoom_line(collection_layout),
        "imui-editor-proof.authoring.imui.collection.zoom-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_select_all_line(),
        "imui-editor-proof.authoring.imui.collection.select-all-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_rename_line(),
        "imui-editor-proof.authoring.imui.collection.rename-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_context_menu_line(),
        "imui-editor-proof.authoring.imui.collection.context-menu-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_command_package_line(),
        "imui-editor-proof.authoring.imui.collection.command-package-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_rename_status_line(&collection_rename_status),
        "imui-editor-proof.authoring.imui.collection.rename-status-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_command_status_line(&collection_command_status),
        "imui-editor-proof.authoring.imui.collection.command-status-readout",
    );
    render_collection_command_buttons(
        ui,
        ProofCollectionCommandButtonModels {
            assets: collection_assets_model.clone(),
            selection: collection_selection_model.clone(),
            keyboard: collection_keyboard_model.clone(),
            command_status: collection_command_status_model.clone(),
            rename_session: collection_rename_session_model.clone(),
            rename_draft: collection_rename_draft_model.clone(),
            rename_focus_pending: collection_rename_focus_pending_model.clone(),
            rename_status: collection_rename_status_model.clone(),
        },
        ProofCollectionCommandButtonState {
            visible_assets: &collection_assets,
            stored_assets: &stored_collection_assets,
            selection: &collection_selection,
            keyboard: &collection_keyboard,
            reverse_order: collection_reverse_order,
            rename_ready_session: collection_rename_ready_session.as_ref(),
        },
    );

    render_collection_browser_scope(
        ui,
        ProofCollectionBrowserScopeModels {
            assets: collection_assets_model.clone(),
            reverse_order: collection_reverse_order_model.clone(),
            selection: collection_selection_model.clone(),
            box_select: collection_box_select_model.clone(),
            keyboard: collection_keyboard_model.clone(),
            zoom: collection_zoom_model.clone(),
            context_menu_anchor: collection_context_menu_anchor_model.clone(),
            active_focus_target: collection_active_focus_target_model.clone(),
            rename_session: collection_rename_session_model.clone(),
            rename_draft: collection_rename_draft_model.clone(),
            rename_focus_pending: collection_rename_focus_pending_model.clone(),
            rename_status: collection_rename_status_model.clone(),
            command_status: collection_command_status_model.clone(),
            scroll: collection_scroll_handle.clone(),
        },
        ProofCollectionBrowserScopeState {
            assets: &collection_assets,
            keys: &collection_keys,
            selection: &collection_selection,
            box_select: &collection_box_select,
            active_id: collection_active_id.as_ref(),
            rename_session: collection_rename_session.as_ref(),
            rename_focus_pending: collection_rename_focus_pending,
            layout: collection_layout,
        },
    );

    render_collection_context_menu(
        ui,
        ProofCollectionContextMenuModels {
            anchor: collection_context_menu_anchor_model.clone(),
            selection: collection_selection_model.clone(),
            keyboard: collection_keyboard_model.clone(),
            assets: collection_assets_model.clone(),
            reverse_order: collection_reverse_order_model.clone(),
            command_status: collection_command_status_model.clone(),
            rename_session: collection_rename_session_model.clone(),
            rename_draft: collection_rename_draft_model.clone(),
            rename_focus_pending: collection_rename_focus_pending_model.clone(),
            rename_status: collection_rename_status_model.clone(),
        },
    );

    if let Some(session) = collection_rename_session.as_ref()
        && !collection_assets
            .iter()
            .any(|asset| asset.id == session.target_id)
    {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_rename_session_model, |state| *state = None);
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_rename_focus_pending_model, |state| {
                *state = false
            });
    }

    render_collection_import_target(ui);
}
