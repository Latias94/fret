use std::sync::Arc;

use fret::advanced::view::AppRenderDataExt as _;
use fret::imui::{kit::ImUiMultiSelectState, prelude::*};
use fret_core::{Point, Px};
use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::scroll::ScrollHandle;

use super::box_select::ProofCollectionBoxSelectState;
use super::geometry::{ProofCollectionLayoutMetrics, proof_collection_layout_metrics};
use super::models::{
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
use super::rename::ProofCollectionRenameSession;
use super::selection::ProofCollectionKeyboardState;
use super::{KernelApp, ProofCollectionAsset};

pub(super) struct ProofCollectionRuntimeState {
    pub(super) models: ProofCollectionRuntimeModels,
    pub(super) snapshot: ProofCollectionRuntimeSnapshot,
}

pub(super) struct ProofCollectionRuntimeModels {
    pub(super) assets: Model<Vec<ProofCollectionAsset>>,
    pub(super) selection: Model<ImUiMultiSelectState<Arc<str>>>,
    pub(super) reverse_order: Model<bool>,
    pub(super) box_select: Model<ProofCollectionBoxSelectState>,
    pub(super) keyboard: Model<ProofCollectionKeyboardState>,
    pub(super) zoom: Model<Px>,
    pub(super) context_menu_anchor: Model<Option<Point>>,
    pub(super) rename_session: Model<Option<ProofCollectionRenameSession>>,
    pub(super) rename_draft: Model<String>,
    pub(super) rename_focus_pending: Model<bool>,
    pub(super) active_focus_target: Model<Option<GlobalElementId>>,
    pub(super) rename_status: Model<String>,
    pub(super) command_status: Model<String>,
    pub(super) scroll: ScrollHandle,
}

pub(super) struct ProofCollectionRuntimeSnapshot {
    pub(super) stored_assets: Vec<ProofCollectionAsset>,
    pub(super) selection: ImUiMultiSelectState<Arc<str>>,
    pub(super) box_select: ProofCollectionBoxSelectState,
    pub(super) keyboard: ProofCollectionKeyboardState,
    pub(super) reverse_order: bool,
    pub(super) rename_status: String,
    pub(super) command_status: String,
    pub(super) rename_session: Option<ProofCollectionRenameSession>,
    pub(super) rename_focus_pending: bool,
    pub(super) layout: ProofCollectionLayoutMetrics,
}

impl ProofCollectionRuntimeSnapshot {
    pub(super) fn rename_session(&self) -> Option<&ProofCollectionRenameSession> {
        self.rename_session.as_ref()
    }
}

pub(super) fn proof_collection_runtime_state(
    ui: &mut ImUi<'_, '_, KernelApp>,
) -> ProofCollectionRuntimeState {
    let models = ProofCollectionRuntimeModels {
        selection: authoring_parity_collection_selection_model(ui.cx_mut()),
        assets: authoring_parity_collection_assets_model(ui.cx_mut()),
        reverse_order: authoring_parity_collection_reverse_order_model(ui.cx_mut()),
        box_select: authoring_parity_collection_box_select_model(ui.cx_mut()),
        keyboard: authoring_parity_collection_keyboard_model(ui.cx_mut()),
        zoom: authoring_parity_collection_zoom_model(ui.cx_mut()),
        context_menu_anchor: authoring_parity_collection_context_menu_anchor_model(ui.cx_mut()),
        rename_session: authoring_parity_collection_rename_session_model(ui.cx_mut()),
        rename_draft: authoring_parity_collection_rename_draft_model(ui.cx_mut()),
        rename_focus_pending: authoring_parity_collection_rename_focus_pending_model(ui.cx_mut()),
        active_focus_target: authoring_parity_collection_active_focus_target_model(ui.cx_mut()),
        rename_status: authoring_parity_collection_rename_status_model(ui.cx_mut()),
        command_status: authoring_parity_collection_command_status_model(ui.cx_mut()),
        scroll: authoring_parity_collection_scroll_handle(ui.cx_mut()),
    };
    let snapshot = proof_collection_runtime_snapshot(ui, &models);

    ProofCollectionRuntimeState { models, snapshot }
}

fn proof_collection_runtime_snapshot(
    ui: &mut ImUi<'_, '_, KernelApp>,
    models: &ProofCollectionRuntimeModels,
) -> ProofCollectionRuntimeSnapshot {
    let stored_assets = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.assets, |state| state.clone());
    let selection = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.selection, |state| state);
    let box_select = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.box_select, |state| state);
    let keyboard = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.keyboard, |state| state);
    let tile_extent = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.zoom, |state| state);
    let reverse_order = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.reverse_order, |value| value);
    let rename_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.rename_status, |state| state.clone());
    let command_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.command_status, |state| state.clone());
    let rename_session = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.rename_session, |state| state.clone());
    let rename_focus_pending = ui
        .cx_mut()
        .data()
        .selector_model_paint(&models.rename_focus_pending, |state| state);
    let layout = proof_collection_layout_metrics(models.scroll.viewport_size().width, tile_extent);

    ProofCollectionRuntimeSnapshot {
        stored_assets,
        selection,
        box_select,
        keyboard,
        reverse_order,
        rename_status,
        command_status,
        rename_session,
        rename_focus_pending,
        layout,
    }
}
