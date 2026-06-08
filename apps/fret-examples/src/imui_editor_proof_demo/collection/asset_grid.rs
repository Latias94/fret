use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::{kit::ImUiMultiSelectState, prelude::*};
use fret_core::Point;
use fret_runtime::Model;
use fret_ui::GlobalElementId;

use super::super::KernelApp;
use super::ProofCollectionAsset;
use super::box_select::ProofCollectionRenderedItem;
use super::geometry::ProofCollectionLayoutMetrics;
use super::rename::ProofCollectionRenameSession;
use super::selection::ProofCollectionKeyboardState;

mod actions;
mod chrome;
mod inline_rename;
mod metadata;
mod tile;

use chrome::collection_asset_grid_options;
use tile::render_collection_asset_tile;

pub(super) struct ProofCollectionAssetGridModels {
    pub(super) assets: Model<Vec<ProofCollectionAsset>>,
    pub(super) selection: Model<ImUiMultiSelectState<Arc<str>>>,
    pub(super) keyboard: Model<ProofCollectionKeyboardState>,
    pub(super) context_menu_anchor: Model<Option<Point>>,
    pub(super) active_focus_target: Model<Option<GlobalElementId>>,
    pub(super) rename_session: Model<Option<ProofCollectionRenameSession>>,
    pub(super) rename_draft: Model<String>,
    pub(super) rename_focus_pending: Model<bool>,
    pub(super) rename_status: Model<String>,
}

pub(super) struct ProofCollectionAssetGridState<'a> {
    pub(super) assets: &'a [ProofCollectionAsset],
    pub(super) keys: &'a [Arc<str>],
    pub(super) selection: &'a ImUiMultiSelectState<Arc<str>>,
    pub(super) active_id: Option<&'a Arc<str>>,
    pub(super) rename_session: Option<&'a ProofCollectionRenameSession>,
    pub(super) rename_focus_pending: bool,
    pub(super) layout: ProofCollectionLayoutMetrics,
    pub(super) scope_origin: Option<Point>,
    pub(super) rendered_items: Rc<RefCell<Vec<ProofCollectionRenderedItem>>>,
}

pub(super) fn render_collection_asset_grid(
    ui: &mut ImUi<'_, '_, KernelApp>,
    models: ProofCollectionAssetGridModels,
    state: ProofCollectionAssetGridState<'_>,
) {
    ui.grid_with_options(collection_asset_grid_options(state.layout), |ui| {
        for asset in state.assets {
            render_collection_asset_tile(ui, &models, &state, asset);
        }
    });
}
