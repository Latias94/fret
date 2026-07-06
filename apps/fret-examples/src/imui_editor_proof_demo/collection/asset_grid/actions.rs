use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::Point;
use fret_runtime::Model;
use fret_ui::GlobalElementId;

use super::super::super::KernelApp;
use super::super::model_owner::ProofCollectionModelOwner;
use super::super::selection::ProofCollectionKeyboardState;
use super::ProofCollectionAssetGridModels;

pub(super) fn proof_collection_asset_grid_publish_active_focus_target(
    app: &mut KernelApp,
    active_focus_target: &Model<Option<GlobalElementId>>,
    focus_target: GlobalElementId,
) {
    ProofCollectionModelOwner::new(app.models_mut())
        .publish_active_focus_target(active_focus_target, focus_target);
}

pub(super) fn proof_collection_asset_grid_activate_clicked_asset(
    app: &mut KernelApp,
    keyboard: &Model<ProofCollectionKeyboardState>,
    asset_id: Arc<str>,
) {
    ProofCollectionModelOwner::new(app.models_mut()).activate_asset(keyboard, asset_id);
}

pub(super) fn proof_collection_asset_grid_apply_context_menu(
    app: &mut KernelApp,
    models: &ProofCollectionAssetGridModels,
    next_selection: ImUiMultiSelectState<Arc<str>>,
    next_keyboard: ProofCollectionKeyboardState,
    anchor: Option<Point>,
) {
    ProofCollectionModelOwner::new(app.models_mut()).apply_context_menu(
        &models.selection,
        &models.keyboard,
        &models.context_menu_anchor,
        next_selection,
        next_keyboard,
        anchor,
    );
}
