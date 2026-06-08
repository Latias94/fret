use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::{
    ImUiFacade,
    kit::{self, ImUiMultiSelectState},
    prelude::*,
};
use fret_core::Point;
use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui_kit::recipes::imui_drag_preview::drag_preview_ghost_with_options;

use super::super::{KernelApp, proof_drag_preview_card};
use super::ProofCollectionAsset;
use super::box_select::ProofCollectionRenderedItem;
use super::drag_drop::{
    proof_collection_drag_payload_for_asset, proof_collection_drag_preview_subtitle,
    proof_collection_drag_preview_title,
};
use super::geometry::{ProofCollectionLayoutMetrics, proof_collection_localize_rect};
use super::rename::ProofCollectionRenameSession;
use super::selection::{ProofCollectionKeyboardState, proof_collection_context_menu_selection};

mod actions;
mod chrome;
mod inline_rename;
mod metadata;

use actions::{
    proof_collection_asset_grid_activate_clicked_asset,
    proof_collection_asset_grid_apply_context_menu,
    proof_collection_asset_grid_publish_active_focus_target,
};
use chrome::{
    collection_asset_ghost_id, collection_asset_ghost_options, collection_asset_grid_options,
    collection_asset_selectable_options, collection_asset_tile_options,
};
use inline_rename::render_collection_inline_rename_field;
use metadata::render_collection_asset_metadata_readouts;

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

fn render_collection_asset_tile(
    ui: &mut ImUiFacade<'_, '_, KernelApp>,
    models: &ProofCollectionAssetGridModels,
    state: &ProofCollectionAssetGridState<'_>,
    asset: &ProofCollectionAsset,
) {
    let payload = proof_collection_drag_payload_for_asset(state.assets, state.selection, asset);
    let preview_title = proof_collection_drag_preview_title(&payload);
    let preview_subtitle = proof_collection_drag_preview_subtitle(&payload);
    let ghost_id = collection_asset_ghost_id(asset);

    ui.id(asset.id.clone(), |ui| {
        ui.vertical_with_options(collection_asset_tile_options(asset, state.layout), |ui| {
            let trigger = ui.multi_selectable_with_options(
                asset.label.clone(),
                &models.selection,
                state.keys,
                asset.id.clone(),
                collection_asset_selectable_options(asset),
            );
            if state
                .active_id
                .is_some_and(|active_id| active_id == &asset.id)
                && let Some(focus_target) = trigger.id()
            {
                proof_collection_asset_grid_publish_active_focus_target(
                    ui.cx_mut().app,
                    &models.active_focus_target,
                    focus_target,
                );
            }
            if trigger.clicked() {
                proof_collection_asset_grid_activate_clicked_asset(
                    ui.cx_mut().app,
                    &models.keyboard,
                    asset.id.clone(),
                );
            }
            if trigger.context_menu_requested() {
                let (next_selection, next_keyboard) =
                    proof_collection_context_menu_selection(state.selection, asset.id.clone());
                let anchor = trigger
                    .context_menu_anchor()
                    .or(trigger.rect().map(|rect| rect.origin));
                proof_collection_asset_grid_apply_context_menu(
                    ui.cx_mut().app,
                    models,
                    next_selection,
                    next_keyboard,
                    anchor,
                );
            }
            if state
                .rename_session
                .is_some_and(|session| session.target_id == asset.id)
            {
                render_collection_inline_rename_field(
                    ui,
                    models,
                    asset,
                    state.rename_focus_pending,
                );
            }
            let source = ui.drag_source_with_options(
                trigger,
                payload.clone(),
                kit::DragSourceOptions::default(),
            );
            let _ = drag_preview_ghost_with_options(
                ui,
                ghost_id.as_str(),
                source,
                collection_asset_ghost_options(asset),
                proof_drag_preview_card(preview_title.clone(), preview_subtitle.clone()),
            );

            if let Some(scope_origin) = state.scope_origin
                && let Some(bounds) = trigger
                    .id()
                    .and_then(|element_id| ui.cx_mut().last_visual_bounds_for_element(element_id))
                    .or(trigger.rect())
            {
                state
                    .rendered_items
                    .borrow_mut()
                    .push(ProofCollectionRenderedItem {
                        id: asset.id.clone(),
                        local_bounds: proof_collection_localize_rect(bounds, scope_origin),
                    });
            }

            render_collection_asset_metadata_readouts(ui, asset);
        });
    });
}
