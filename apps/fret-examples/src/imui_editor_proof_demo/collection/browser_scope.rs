use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::{
    kit::{self, ImUiMultiSelectState},
    prelude::*,
};
use fret_core::{Color, Point, Px};
use fret_runtime::Model;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, GlobalElementId};

use super::asset_grid::{
    ProofCollectionAssetGridModels, ProofCollectionAssetGridState, render_collection_asset_grid,
};
use super::box_select::{
    ProofCollectionBoxSelectState, ProofCollectionRenderedItem,
    proof_collection_box_select_active_rect,
};
use super::geometry::ProofCollectionLayoutMetrics;
use super::rename::ProofCollectionRenameSession;
use super::selection::ProofCollectionKeyboardState;
use super::{KernelApp, ProofCollectionAsset};

mod input_runtime;

use input_runtime::{
    ProofCollectionBrowserScopeInputModels, ProofCollectionBrowserScopeInputState,
    install_collection_browser_scope_input_runtime, proof_collection_browser_scope_pointer_props,
};

pub(super) struct ProofCollectionBrowserScopeModels {
    pub(super) assets: Model<Vec<ProofCollectionAsset>>,
    pub(super) reverse_order: Model<bool>,
    pub(super) selection: Model<ImUiMultiSelectState<Arc<str>>>,
    pub(super) box_select: Model<ProofCollectionBoxSelectState>,
    pub(super) keyboard: Model<ProofCollectionKeyboardState>,
    pub(super) zoom: Model<Px>,
    pub(super) context_menu_anchor: Model<Option<Point>>,
    pub(super) active_focus_target: Model<Option<GlobalElementId>>,
    pub(super) rename_session: Model<Option<ProofCollectionRenameSession>>,
    pub(super) rename_draft: Model<String>,
    pub(super) rename_focus_pending: Model<bool>,
    pub(super) rename_status: Model<String>,
    pub(super) command_status: Model<String>,
    pub(super) scroll: ScrollHandle,
}

pub(super) struct ProofCollectionBrowserScopeState<'a> {
    pub(super) assets: &'a [ProofCollectionAsset],
    pub(super) keys: &'a [Arc<str>],
    pub(super) selection: &'a ImUiMultiSelectState<Arc<str>>,
    pub(super) box_select: &'a ProofCollectionBoxSelectState,
    pub(super) active_id: Option<&'a Arc<str>>,
    pub(super) rename_session: Option<&'a ProofCollectionRenameSession>,
    pub(super) rename_focus_pending: bool,
    pub(super) layout: ProofCollectionLayoutMetrics,
}

pub(super) fn render_collection_browser_scope(
    ui: &mut ImUi<'_, '_, KernelApp>,
    models: ProofCollectionBrowserScopeModels,
    state: ProofCollectionBrowserScopeState<'_>,
) {
    let collection_assets = state.assets.to_vec();
    let collection_keys = state.keys.to_vec();
    let collection_selection = state.selection.clone();
    let collection_box_select = state.box_select.clone();
    let collection_active_id = state.active_id.cloned();
    let collection_rename_session = state.rename_session.cloned();
    let collection_rename_focus_pending = state.rename_focus_pending;
    let collection_layout = state.layout;
    let collection_scroll_handle = models.scroll.clone();

    ui.child_region_with_options(
        "imui-editor-proof.authoring.imui.collection.browser",
        kit::ChildRegionOptions {
            layout: fret_ui_kit::LayoutRefinement::default()
                .w_full()
                .h_px(Px(220.0)),
            scroll: kit::ScrollOptions {
                handle: Some(collection_scroll_handle.clone()),
                viewport_test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.collection.browser.viewport",
                )),
                ..Default::default()
            },
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.browser",
            )),
            content_test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.browser.content",
            )),
            ..Default::default()
        },
        move |ui| {
            let collection_assets = collection_assets.clone();
            let collection_keys = collection_keys.clone();
            let collection_assets_model = models.assets.clone();
            let collection_reverse_order_model = models.reverse_order.clone();
            let collection_selection = collection_selection.clone();
            let collection_selection_model = models.selection.clone();
            let collection_box_select_model = models.box_select.clone();
            let collection_box_select = collection_box_select.clone();
            let collection_keyboard_model = models.keyboard.clone();
            let collection_zoom_model = models.zoom.clone();
            let collection_context_menu_anchor_model = models.context_menu_anchor.clone();
            let collection_active_focus_target_model = models.active_focus_target.clone();
            let collection_active_id = collection_active_id.clone();
            let collection_rename_session = collection_rename_session.clone();
            let collection_rename_session_model = models.rename_session.clone();
            let collection_rename_draft_model = models.rename_draft.clone();
            let collection_rename_focus_pending_model = models.rename_focus_pending.clone();
            let collection_rename_focus_pending = collection_rename_focus_pending;
            let collection_rename_status_model = models.rename_status.clone();
            let collection_command_status_model = models.command_status.clone();
            let collection_scroll_handle = collection_scroll_handle.clone();
            let collection_layout = collection_layout;

            ui.add_ui(fret_ui_kit::ui::container_build(move |cx, out| {
                let rendered_items = Rc::new(RefCell::new(Vec::<ProofCollectionRenderedItem>::new()));

                out.push(cx.pointer_region(proof_collection_browser_scope_pointer_props(), move |cx| {
                    let scope_id = cx.root_id();
                    let scope_origin = cx
                        .last_visual_bounds_for_element(scope_id)
                        .or_else(|| cx.last_bounds_for_element(scope_id))
                        .map(|rect| rect.origin);

                    install_collection_browser_scope_input_runtime(
                        cx,
                        scope_id,
                        ProofCollectionBrowserScopeInputModels {
                            assets: collection_assets_model.clone(),
                            reverse_order: collection_reverse_order_model.clone(),
                            selection: collection_selection_model.clone(),
                            box_select: collection_box_select_model.clone(),
                            keyboard: collection_keyboard_model.clone(),
                            zoom: collection_zoom_model.clone(),
                            context_menu_anchor: collection_context_menu_anchor_model.clone(),
                            rename_session: collection_rename_session_model.clone(),
                            rename_draft: collection_rename_draft_model.clone(),
                            rename_focus_pending: collection_rename_focus_pending_model.clone(),
                            rename_status: collection_rename_status_model.clone(),
                            command_status: collection_command_status_model.clone(),
                            scroll: collection_scroll_handle.clone(),
                        },
                        ProofCollectionBrowserScopeInputState {
                            keys: &collection_keys,
                            asset_count: collection_assets.len(),
                            layout: collection_layout,
                            rendered_items: rendered_items.clone(),
                        },
                    );

                    vec![fret_ui_kit::ui::stack(move |cx| {
                        let rendered_items_for_grid = rendered_items.clone();
                        let grid = fret_ui_kit::ui::container_build(
                            move |cx: &mut ElementContext<'_, KernelApp>, out| {
                                imui_build(cx, out, |ui| {
                                    render_collection_asset_grid(
                                        ui,
                                        ProofCollectionAssetGridModels {
                                            assets: collection_assets_model.clone(),
                                            selection: collection_selection_model.clone(),
                                            keyboard: collection_keyboard_model.clone(),
                                            context_menu_anchor: collection_context_menu_anchor_model
                                                .clone(),
                                            active_focus_target: collection_active_focus_target_model
                                                .clone(),
                                            rename_session: collection_rename_session_model.clone(),
                                            rename_draft: collection_rename_draft_model.clone(),
                                            rename_focus_pending:
                                                collection_rename_focus_pending_model.clone(),
                                            rename_status: collection_rename_status_model.clone(),
                                        },
                                        ProofCollectionAssetGridState {
                                            assets: &collection_assets,
                                            keys: &collection_keys,
                                            selection: &collection_selection,
                                            active_id: collection_active_id.as_ref(),
                                            rename_session: collection_rename_session.as_ref(),
                                            rename_focus_pending: collection_rename_focus_pending,
                                            layout: collection_layout,
                                            scope_origin,
                                            rendered_items: rendered_items_for_grid.clone(),
                                        },
                                    );
                                });
                            },
                        )
                        .w_full()
                        .into_element(cx);

                        let mut layers = vec![grid];
                        if let Some(drag_rect) =
                            proof_collection_box_select_active_rect(&collection_box_select)
                        {
                            let theme = fret_ui::Theme::global(&*cx.app);
                            let ring = theme.color_token("ring");
                            let fill = Color { a: 0.14, ..ring };
                            let border = Color { a: 0.88, ..ring };
                            layers.push(
                                fret_ui_kit::ui::container(
                                    |_cx| Vec::<fret_ui::element::AnyElement>::new(),
                                )
                                .absolute()
                                .left_px(drag_rect.origin.x)
                                .top_px(drag_rect.origin.y)
                                .w_px(drag_rect.size.width)
                                .h_px(drag_rect.size.height)
                                .bg(fret_ui_kit::ColorRef::Color(fill))
                                .border_1()
                                .border_color(fret_ui_kit::ColorRef::Color(border))
                                .test_id(
                                    "imui-editor-proof.authoring.imui.collection.box-select.marquee",
                                )
                                .into_element(cx),
                            );
                        }
                        layers
                    })
                    .relative()
                    .w_full()
                    .h_full()
                    .test_id("imui-editor-proof.authoring.imui.collection.box-select.scope")
                    .into_element(cx)]
                }));
            }));
        },
    );
}
