use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::{
    kit::{self, ImUiMultiSelectState},
    prelude::*,
};
use fret_core::{Color, MouseButton, Point, Px};
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::Length;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, GlobalElementId};

use super::asset_grid::{
    ProofCollectionAssetGridModels, ProofCollectionAssetGridState, render_collection_asset_grid,
};
use super::box_select::{
    ProofCollectionBoxSelectSession, ProofCollectionBoxSelectState, ProofCollectionRenderedItem,
    proof_collection_box_select_active_rect, proof_collection_box_select_selection,
};
use super::geometry::{
    ProofCollectionLayoutMetrics, proof_collection_drag_threshold_met,
    proof_collection_zoom_request,
};
use super::keyboard::{ProofCollectionKeyboardHandlerModels, install_collection_keyboard_handler};
use super::rename::ProofCollectionRenameSession;
use super::selection::ProofCollectionKeyboardState;
use super::{KernelApp, ProofCollectionAsset};

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
                let mut props = fret_ui::element::PointerRegionProps::default();
                props.layout.size.width = Length::Fill;
                props.capture_phase_pointer_moves = true;

                out.push(cx.pointer_region(props, move |cx| {
                    let scope_id = cx.root_id();
                    let scope_origin = cx
                        .last_visual_bounds_for_element(scope_id)
                        .or_else(|| cx.last_bounds_for_element(scope_id))
                        .map(|rect| rect.origin);

                    let rendered_items_for_move = rendered_items.clone();
                    let rendered_items_for_up = rendered_items.clone();
                    let selection_model_for_down = collection_selection_model.clone();
                    let selection_model_for_move = collection_selection_model.clone();
                    let selection_model_for_up = collection_selection_model.clone();
                    let keyboard_model_for_move = collection_keyboard_model.clone();
                    let keyboard_model_for_up = collection_keyboard_model.clone();
                    let keyboard_model_for_clear = collection_keyboard_model.clone();
                    let context_menu_anchor_model_for_up =
                        collection_context_menu_anchor_model.clone();
                    let box_select_model_for_down = collection_box_select_model.clone();
                    let box_select_model_for_move = collection_box_select_model.clone();
                    let box_select_model_for_up = collection_box_select_model.clone();
                    let box_select_model_for_cancel = collection_box_select_model.clone();
                    let collection_keys_for_move = collection_keys.clone();
                    let collection_keys_for_up = collection_keys.clone();
                    let collection_zoom_model_for_wheel = collection_zoom_model.clone();
                    let collection_scroll_handle_for_wheel = collection_scroll_handle.clone();
                    let collection_asset_count_for_wheel = collection_assets.len();

                    install_collection_keyboard_handler(
                        cx,
                        scope_id,
                        collection_layout.columns,
                        ProofCollectionKeyboardHandlerModels {
                            assets: collection_assets_model.clone(),
                            reverse_order: collection_reverse_order_model.clone(),
                            selection: collection_selection_model.clone(),
                            keyboard: collection_keyboard_model.clone(),
                            rename_session: collection_rename_session_model.clone(),
                            rename_draft: collection_rename_draft_model.clone(),
                            rename_focus_pending: collection_rename_focus_pending_model.clone(),
                            rename_status: collection_rename_status_model.clone(),
                            command_status: collection_command_status_model.clone(),
                        },
                    );

                    cx.pointer_region_on_wheel(Arc::new(move |host, acx, wheel| {
                        let Some(update) = proof_collection_zoom_request(
                            collection_layout,
                            collection_scroll_handle_for_wheel.offset(),
                            wheel.position_local,
                            wheel.delta,
                            wheel.modifiers,
                            collection_asset_count_for_wheel,
                        ) else {
                            return false;
                        };

                        let _ = host.update_model(&collection_zoom_model_for_wheel, |state| {
                            *state = update.next_tile_extent;
                        });
                        collection_scroll_handle_for_wheel.set_offset(update.next_scroll_offset);
                        host.notify(acx);
                        true
                    }));

                    cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
                        if down.button != MouseButton::Left {
                            return false;
                        }

                        host.request_focus(acx.target);
                        if down.hit_is_pressable {
                            return false;
                        }
                        let baseline_selected = host
                            .models_mut()
                            .read(&selection_model_for_down, |state| state.selected().to_vec())
                            .unwrap_or_default();
                        let append_mode = down.modifiers.ctrl || down.modifiers.meta;
                        let _ = host.update_model(&box_select_model_for_down, |state| {
                            state.session = Some(ProofCollectionBoxSelectSession {
                                pointer_id: down.pointer_id,
                                origin_local: down.position_local,
                                current_local: down.position_local,
                                baseline_selected,
                                append_mode,
                                threshold_met: false,
                            });
                        });
                        host.capture_pointer();
                        host.notify(acx);
                        true
                    }));

                    cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
                        if !mv.buttons.left {
                            return false;
                        }

                        let session = host
                            .update_model(&box_select_model_for_move, |state| {
                                let Some(session) = state.session.as_mut() else {
                                    return None;
                                };
                                if session.pointer_id != mv.pointer_id {
                                    return None;
                                }

                                session.current_local = mv.position_local;
                                if !session.threshold_met {
                                    session.threshold_met = proof_collection_drag_threshold_met(
                                        session.origin_local,
                                        session.current_local,
                                    );
                                }

                                Some(session.clone())
                            })
                            .flatten();

                        let Some(session) = session else {
                            return false;
                        };

                        if session.threshold_met {
                            let next_selection = proof_collection_box_select_selection(
                                &collection_keys_for_move,
                                &rendered_items_for_move.borrow(),
                                &session,
                            );
                            let _ = host.update_model(&selection_model_for_move, |state| {
                                *state = next_selection.clone();
                            });
                            let _ = host.update_model(&keyboard_model_for_move, |state| {
                                state.active_id = next_selection.first_selected().cloned();
                            });
                        }

                        host.notify(acx);
                        true
                    }));

                    cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
                        if up.button == MouseButton::Right && up.is_click {
                            if up.down_hit_pressable_target.is_some()
                                || up.down_hit_pressable_target_in_descendant_subtree
                            {
                                return false;
                            }

                            host.request_focus(acx.target);
                            let position = up.position_window.unwrap_or(up.position);
                            let _ = host.update_model(&context_menu_anchor_model_for_up, |state| {
                                *state = Some(position);
                            });
                            host.notify(acx);
                            return true;
                        }

                        let session = host
                            .update_model(&box_select_model_for_up, |state| {
                                let Some(mut session) = state.session.take() else {
                                    return None;
                                };
                                if session.pointer_id != up.pointer_id {
                                    state.session = Some(session);
                                    return None;
                                }

                                session.current_local = up.position_local;
                                if !session.threshold_met {
                                    session.threshold_met = proof_collection_drag_threshold_met(
                                        session.origin_local,
                                        session.current_local,
                                    );
                                }

                                Some(session)
                            })
                            .flatten();

                        let Some(session) = session else {
                            return false;
                        };

                        host.release_pointer_capture();
                        if session.threshold_met {
                            let next_selection = proof_collection_box_select_selection(
                                &collection_keys_for_up,
                                &rendered_items_for_up.borrow(),
                                &session,
                            );
                            let _ = host.update_model(&selection_model_for_up, |state| {
                                *state = next_selection.clone();
                            });
                            let _ = host.update_model(&keyboard_model_for_up, |state| {
                                state.active_id = next_selection.first_selected().cloned();
                            });
                        } else if !session.append_mode {
                            let _ = host.update_model(&selection_model_for_up, |state| {
                                state.clear();
                            });
                            let _ = host.update_model(&keyboard_model_for_clear, |state| {
                                state.active_id = None;
                            });
                        }

                        host.notify(acx);
                        true
                    }));

                    cx.pointer_region_on_pointer_cancel(Arc::new(move |host, _acx, cancel| {
                        let cleared = host
                            .update_model(&box_select_model_for_cancel, |state| {
                                let matches_pointer = state
                                    .session
                                    .as_ref()
                                    .is_some_and(|session| session.pointer_id == cancel.pointer_id);
                                if matches_pointer {
                                    state.session = None;
                                }
                                matches_pointer
                            })
                            .unwrap_or(false);
                        if cleared {
                            host.release_pointer_capture();
                        }
                        cleared
                    }));

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
