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
use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost_with_options,
};

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

mod inline_rename;
mod metadata;

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
    ui.grid_with_options(
        kit::GridOptions {
            columns: state.layout.columns,
            column_gap: fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N2),
            row_gap: fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N2),
            row_items: fret_ui_kit::Items::Stretch,
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.grid",
            )),
            ..Default::default()
        },
        |ui| {
            for asset in state.assets {
                render_collection_asset_tile(ui, &models, &state, asset);
            }
        },
    );
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
    let ghost_id = format!(
        "imui-editor-proof.authoring.imui.collection.asset.{}.ghost",
        asset.id
    );

    ui.id(asset.id.clone(), |ui| {
        ui.vertical_with_options(
            kit::VerticalOptions {
                layout: fret_ui_kit::LayoutRefinement::default()
                    .flex_1()
                    .min_h(state.layout.tile_min_height),
                gap: fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N1),
                test_id: Some(Arc::from(format!(
                    "imui-editor-proof.authoring.imui.collection.asset.{}",
                    asset.id
                ))),
                ..Default::default()
            },
            |ui| {
                let trigger = ui.multi_selectable_with_options(
                    asset.label.clone(),
                    &models.selection,
                    state.keys,
                    asset.id.clone(),
                    kit::SelectableOptions {
                        focusable: false,
                        test_id: Some(Arc::from(format!(
                            "imui-editor-proof.authoring.imui.collection.asset.{}.select",
                            asset.id
                        ))),
                        ..Default::default()
                    },
                );
                if state
                    .active_id
                    .is_some_and(|active_id| active_id == &asset.id)
                    && let Some(focus_target) = trigger.id()
                {
                    let _ = ui.cx_mut().app.models_mut().update(
                        &models.active_focus_target,
                        |target| {
                            *target = Some(focus_target);
                        },
                    );
                }
                if trigger.clicked() {
                    let _ = ui
                        .cx_mut()
                        .app
                        .models_mut()
                        .update(&models.keyboard, |keyboard| {
                            keyboard.active_id = Some(asset.id.clone());
                        });
                }
                if trigger.context_menu_requested() {
                    let (next_selection, next_keyboard) =
                        proof_collection_context_menu_selection(state.selection, asset.id.clone());
                    let anchor = trigger
                        .context_menu_anchor()
                        .or(trigger.rect().map(|rect| rect.origin));
                    let _ = ui
                        .cx_mut()
                        .app
                        .models_mut()
                        .update(&models.selection, |selection| {
                            *selection = next_selection.clone();
                        });
                    let _ = ui
                        .cx_mut()
                        .app
                        .models_mut()
                        .update(&models.keyboard, |keyboard| {
                            *keyboard = next_keyboard.clone();
                        });
                    let _ = ui.cx_mut().app.models_mut().update(
                        &models.context_menu_anchor,
                        |anchor_model| {
                            *anchor_model = anchor;
                        },
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
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from(format!(
                            "imui-editor-proof.authoring.imui.collection.asset.{}.ghost",
                            asset.id
                        ))),
                        ..Default::default()
                    },
                    proof_drag_preview_card(preview_title.clone(), preview_subtitle.clone()),
                );

                if let Some(scope_origin) = state.scope_origin
                    && let Some(bounds) = trigger
                        .id()
                        .and_then(|element_id| {
                            ui.cx_mut().last_visual_bounds_for_element(element_id)
                        })
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
            },
        );
    });
}
