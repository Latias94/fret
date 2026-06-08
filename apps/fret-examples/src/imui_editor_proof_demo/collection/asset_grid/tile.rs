use fret::imui::{ImUiFacade, kit, prelude::*};
use fret_ui_kit::recipes::imui_drag_preview::drag_preview_ghost_with_options;

use super::super::super::{KernelApp, proof_drag_preview_card};
use super::super::ProofCollectionAsset;
use super::super::box_select::ProofCollectionRenderedItem;
use super::super::drag_drop::{
    proof_collection_drag_payload_for_asset, proof_collection_drag_preview_subtitle,
    proof_collection_drag_preview_title,
};
use super::super::geometry::proof_collection_localize_rect;
use super::super::selection::proof_collection_context_menu_selection;
use super::actions::{
    proof_collection_asset_grid_activate_clicked_asset,
    proof_collection_asset_grid_apply_context_menu,
    proof_collection_asset_grid_publish_active_focus_target,
};
use super::chrome::{
    collection_asset_ghost_id, collection_asset_ghost_options, collection_asset_selectable_options,
    collection_asset_tile_options,
};
use super::inline_rename::render_collection_inline_rename_field;
use super::metadata::render_collection_asset_metadata_readouts;
use super::{ProofCollectionAssetGridModels, ProofCollectionAssetGridState};

pub(super) fn render_collection_asset_tile(
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
