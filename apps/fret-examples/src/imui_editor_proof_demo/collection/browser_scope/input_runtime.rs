use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{Point, Px};
use fret_runtime::Model;
use fret_ui::element::{Length, PointerRegionProps};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, GlobalElementId};

use super::super::box_select::{ProofCollectionBoxSelectState, ProofCollectionRenderedItem};
use super::super::geometry::ProofCollectionLayoutMetrics;
use super::super::keyboard::{
    ProofCollectionKeyboardHandlerModels, install_collection_keyboard_handler,
};
use super::super::rename::ProofCollectionRenameSession;
use super::super::selection::ProofCollectionKeyboardState;
use super::super::{KernelApp, ProofCollectionAsset};

mod box_select;
mod context_menu;
mod zoom;

use box_select::{
    ProofCollectionBrowserScopeBoxSelectRuntimeModels,
    ProofCollectionBrowserScopeBoxSelectRuntimeState,
    install_collection_browser_scope_box_select_runtime,
};
use context_menu::publish_collection_browser_scope_context_menu_anchor;
use zoom::install_collection_browser_scope_zoom_runtime;

pub(super) struct ProofCollectionBrowserScopeInputModels {
    pub(super) assets: Model<Vec<ProofCollectionAsset>>,
    pub(super) reverse_order: Model<bool>,
    pub(super) selection: Model<ImUiMultiSelectState<Arc<str>>>,
    pub(super) box_select: Model<ProofCollectionBoxSelectState>,
    pub(super) keyboard: Model<ProofCollectionKeyboardState>,
    pub(super) zoom: Model<Px>,
    pub(super) context_menu_anchor: Model<Option<Point>>,
    pub(super) rename_session: Model<Option<ProofCollectionRenameSession>>,
    pub(super) rename_draft: Model<String>,
    pub(super) rename_focus_pending: Model<bool>,
    pub(super) rename_status: Model<String>,
    pub(super) command_status: Model<String>,
    pub(super) scroll: ScrollHandle,
}

pub(super) struct ProofCollectionBrowserScopeInputState<'a> {
    pub(super) keys: &'a [Arc<str>],
    pub(super) asset_count: usize,
    pub(super) layout: ProofCollectionLayoutMetrics,
    pub(super) rendered_items: Rc<RefCell<Vec<ProofCollectionRenderedItem>>>,
}

pub(super) fn proof_collection_browser_scope_pointer_props() -> PointerRegionProps {
    let mut props = PointerRegionProps::default();
    props.layout.size.width = Length::Fill;
    props.capture_phase_pointer_moves = true;
    props
}

pub(super) fn install_collection_browser_scope_input_runtime(
    cx: &mut ElementContext<'_, KernelApp>,
    scope_id: GlobalElementId,
    models: ProofCollectionBrowserScopeInputModels,
    state: ProofCollectionBrowserScopeInputState<'_>,
) {
    let collection_keys = state.keys.to_vec();
    let collection_layout = state.layout;
    let rendered_items = state.rendered_items;
    let context_menu_anchor_model_for_up = models.context_menu_anchor.clone();

    install_collection_keyboard_handler(
        cx,
        scope_id,
        collection_layout.columns,
        ProofCollectionKeyboardHandlerModels {
            assets: models.assets.clone(),
            reverse_order: models.reverse_order.clone(),
            selection: models.selection.clone(),
            keyboard: models.keyboard.clone(),
            rename_session: models.rename_session.clone(),
            rename_draft: models.rename_draft.clone(),
            rename_focus_pending: models.rename_focus_pending.clone(),
            rename_status: models.rename_status.clone(),
            command_status: models.command_status.clone(),
        },
    );

    install_collection_browser_scope_zoom_runtime(
        cx,
        collection_layout,
        models.scroll.clone(),
        models.zoom.clone(),
        state.asset_count,
    );

    install_collection_browser_scope_box_select_runtime(
        cx,
        ProofCollectionBrowserScopeBoxSelectRuntimeModels {
            selection: models.selection.clone(),
            keyboard: models.keyboard.clone(),
            box_select: models.box_select.clone(),
        },
        ProofCollectionBrowserScopeBoxSelectRuntimeState {
            collection_keys,
            rendered_items,
        },
        Arc::new(move |host, acx, up| {
            publish_collection_browser_scope_context_menu_anchor(
                host,
                acx,
                &context_menu_anchor_model_for_up,
                up,
            )
        }),
    );
}
