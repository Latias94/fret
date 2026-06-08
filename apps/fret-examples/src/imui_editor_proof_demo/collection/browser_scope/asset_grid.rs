use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::{kit::ImUiMultiSelectState, prelude::*};
use fret_core::Point;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId};

use super::super::asset_grid::{
    ProofCollectionAssetGridModels, ProofCollectionAssetGridState, render_collection_asset_grid,
};
use super::super::box_select::ProofCollectionRenderedItem;
use super::super::geometry::ProofCollectionLayoutMetrics;
use super::super::rename::ProofCollectionRenameSession;
use super::super::selection::ProofCollectionKeyboardState;
use super::super::{KernelApp, ProofCollectionAsset};

pub(super) struct ProofCollectionBrowserScopeAssetGridModels {
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

pub(super) struct ProofCollectionBrowserScopeAssetGridState<'a> {
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

pub(super) fn render_collection_browser_scope_asset_grid(
    cx: &mut ElementContext<'_, KernelApp>,
    models: ProofCollectionBrowserScopeAssetGridModels,
    state: ProofCollectionBrowserScopeAssetGridState<'_>,
) -> AnyElement {
    let assets = state.assets.to_vec();
    let keys = state.keys.to_vec();
    let selection = state.selection.clone();
    let active_id = state.active_id.cloned();
    let rename_session = state.rename_session.cloned();
    let rename_focus_pending = state.rename_focus_pending;
    let layout = state.layout;
    let scope_origin = state.scope_origin;
    let rendered_items = state.rendered_items;

    fret_ui_kit::ui::container_build(move |cx, out| {
        imui_build(cx, out, |ui| {
            render_collection_asset_grid(
                ui,
                ProofCollectionAssetGridModels {
                    assets: models.assets.clone(),
                    selection: models.selection.clone(),
                    keyboard: models.keyboard.clone(),
                    context_menu_anchor: models.context_menu_anchor.clone(),
                    active_focus_target: models.active_focus_target.clone(),
                    rename_session: models.rename_session.clone(),
                    rename_draft: models.rename_draft.clone(),
                    rename_focus_pending: models.rename_focus_pending.clone(),
                    rename_status: models.rename_status.clone(),
                },
                ProofCollectionAssetGridState {
                    assets: &assets,
                    keys: &keys,
                    selection: &selection,
                    active_id: active_id.as_ref(),
                    rename_session: rename_session.as_ref(),
                    rename_focus_pending,
                    layout,
                    scope_origin,
                    rendered_items: rendered_items.clone(),
                },
            );
        });
    })
    .w_full()
    .into_element(cx)
}
