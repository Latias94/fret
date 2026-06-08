use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::Point;
use fret_runtime::Model;
use fret_ui::GlobalElementId;

use super::super::super::KernelApp;
use super::super::selection::ProofCollectionKeyboardState;
use super::ProofCollectionAssetGridModels;

pub(super) fn proof_collection_asset_grid_publish_active_focus_target(
    app: &mut KernelApp,
    active_focus_target: &Model<Option<GlobalElementId>>,
    focus_target: GlobalElementId,
) {
    let _ = app.models_mut().update(active_focus_target, |target| {
        *target = Some(focus_target);
    });
}

pub(super) fn proof_collection_asset_grid_activate_clicked_asset(
    app: &mut KernelApp,
    keyboard: &Model<ProofCollectionKeyboardState>,
    asset_id: Arc<str>,
) {
    let _ = app.models_mut().update(keyboard, |keyboard| {
        keyboard.active_id = Some(asset_id);
    });
}

pub(super) fn proof_collection_asset_grid_apply_context_menu(
    app: &mut KernelApp,
    models: &ProofCollectionAssetGridModels,
    next_selection: ImUiMultiSelectState<Arc<str>>,
    next_keyboard: ProofCollectionKeyboardState,
    anchor: Option<Point>,
) {
    let _ = app.models_mut().update(&models.selection, |selection| {
        *selection = next_selection;
    });
    let _ = app.models_mut().update(&models.keyboard, |keyboard| {
        *keyboard = next_keyboard;
    });
    let _ = app
        .models_mut()
        .update(&models.context_menu_anchor, |anchor_model| {
            *anchor_model = anchor;
        });
}
