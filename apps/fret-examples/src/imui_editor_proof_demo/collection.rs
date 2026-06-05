use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::advanced::view::AppRenderDataExt as _;
use fret::imui::{kit::ImUiMultiSelectState, prelude::*};
use fret_core::{Color, Px};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::Length;
use fret_ui::{ElementContext, UiHost};

use super::{
    KernelApp, named_demo_state, proof_compact_readout_element, proof_section_chrome_label,
};

mod asset_grid;
mod box_select;
mod command_buttons;
mod context_menu;
mod drag_drop;
mod geometry;
mod keyboard;
mod models;
mod readouts;
mod rename;
mod selection;

use asset_grid::{
    ProofCollectionAssetGridModels, ProofCollectionAssetGridState, render_collection_asset_grid,
};
use box_select::{
    ProofCollectionBoxSelectSession, ProofCollectionBoxSelectState, ProofCollectionRenderedItem,
    proof_collection_box_select_active_rect, proof_collection_box_select_selection,
};
use command_buttons::{
    ProofCollectionCommandButtonModels, ProofCollectionCommandButtonState,
    render_collection_command_buttons,
};
use context_menu::{ProofCollectionContextMenuModels, render_collection_context_menu};
use drag_drop::{ProofCollectionDragPayload, proof_collection_drop_status};
use geometry::{
    PROOF_COLLECTION_GRID_FALLBACK_COLUMNS, PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX,
    proof_collection_drag_threshold_met, proof_collection_layout_metrics,
    proof_collection_zoom_line, proof_collection_zoom_request,
};
use keyboard::{ProofCollectionKeyboardHandlerModels, install_collection_keyboard_handler};
use models::{
    authoring_parity_collection_active_focus_target_model,
    authoring_parity_collection_assets_model, authoring_parity_collection_box_select_model,
    authoring_parity_collection_command_status_model,
    authoring_parity_collection_context_menu_anchor_model,
    authoring_parity_collection_drop_status_model, authoring_parity_collection_keyboard_model,
    authoring_parity_collection_rename_draft_model,
    authoring_parity_collection_rename_focus_pending_model,
    authoring_parity_collection_rename_session_model,
    authoring_parity_collection_rename_status_model,
    authoring_parity_collection_reverse_order_model, authoring_parity_collection_scroll_handle,
    authoring_parity_collection_selection_model, authoring_parity_collection_zoom_model,
};
use readouts::{
    proof_collection_active_line, proof_collection_assets_line,
    proof_collection_command_package_line, proof_collection_command_status_line,
    proof_collection_context_menu_line, proof_collection_rename_line,
    proof_collection_rename_status_line, proof_collection_select_all_line,
    proof_collection_selection_line, proof_collection_visible_order_line,
};
use rename::proof_collection_begin_rename_session;
use selection::{proof_collection_active_id, proof_collection_assets_in_visible_order};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProofCollectionAsset {
    pub(super) id: Arc<str>,
    pub(super) label: Arc<str>,
    pub(super) path: Arc<str>,
    pub(super) kind: Arc<str>,
    pub(super) size_kib: u32,
}

pub(super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {
    vec![
        ProofCollectionAsset {
            id: Arc::from("stone-albedo"),
            label: Arc::from("Stone Albedo"),
            path: Arc::from("textures/stone/albedo.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 512,
        },
        ProofCollectionAsset {
            id: Arc::from("stone-normal"),
            label: Arc::from("Stone Normal"),
            path: Arc::from("textures/stone/normal.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 384,
        },
        ProofCollectionAsset {
            id: Arc::from("stone-orm"),
            label: Arc::from("Stone ORM"),
            path: Arc::from("textures/stone/orm.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 256,
        },
        ProofCollectionAsset {
            id: Arc::from("moss-overlay"),
            label: Arc::from("Moss Overlay"),
            path: Arc::from("textures/moss/overlay.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 196,
        },
        ProofCollectionAsset {
            id: Arc::from("pebble-height"),
            label: Arc::from("Pebble Height"),
            path: Arc::from("textures/pebble/height.ktx2"),
            kind: Arc::from("Height"),
            size_kib: 164,
        },
        ProofCollectionAsset {
            id: Arc::from("dust-mask"),
            label: Arc::from("Dust Mask"),
            path: Arc::from("textures/shared/dust-mask.ktx2"),
            kind: Arc::from("Mask"),
            size_kib: 72,
        },
    ]
    .into()
}

pub(super) fn proof_collection_readout_text(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: impl Into<Arc<str>>,
    test_id: &'static str,
) {
    let element =
        ui.with_cx_mut(|cx| proof_compact_readout_element(cx, text, Arc::<str>::from(test_id)));
    ui.add(element);
}

fn proof_collection_section_label(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: &'static str,
    test_id: &'static str,
) {
    let element = ui.with_cx_mut(|cx| proof_section_chrome_label(cx, text, test_id));
    ui.add(element);
}

pub(super) fn render_collection_first_asset_browser_proof(ui: &mut ImUi<'_, '_, KernelApp>) {
    proof_collection_section_label(
        ui,
        "Collection-first asset browser proof",
        "imui-editor-proof.authoring.imui.collection.title",
    );
    ui.text_wrapped(
        "Stable keys keep browser selection pinned while visible order flips and selected-set drag/drop stays app-defined.",
    );
    ui.text_wrapped(
        "Background drag now draws a marquee and updates grid selection app-locally while shared helper widening stays deferred until another first-party proof surface exists.",
    );

    let collection_selection_model = authoring_parity_collection_selection_model(ui.cx_mut());
    let collection_assets_model = authoring_parity_collection_assets_model(ui.cx_mut());
    let collection_reverse_order_model =
        authoring_parity_collection_reverse_order_model(ui.cx_mut());
    let collection_box_select_model = authoring_parity_collection_box_select_model(ui.cx_mut());
    let collection_keyboard_model = authoring_parity_collection_keyboard_model(ui.cx_mut());
    let collection_zoom_model = authoring_parity_collection_zoom_model(ui.cx_mut());
    let collection_context_menu_anchor_model =
        authoring_parity_collection_context_menu_anchor_model(ui.cx_mut());
    let collection_rename_session_model =
        authoring_parity_collection_rename_session_model(ui.cx_mut());
    let collection_rename_draft_model = authoring_parity_collection_rename_draft_model(ui.cx_mut());
    let collection_rename_focus_pending_model =
        authoring_parity_collection_rename_focus_pending_model(ui.cx_mut());
    let collection_active_focus_target_model =
        authoring_parity_collection_active_focus_target_model(ui.cx_mut());
    let collection_rename_status_model =
        authoring_parity_collection_rename_status_model(ui.cx_mut());
    let collection_command_status_model =
        authoring_parity_collection_command_status_model(ui.cx_mut());
    let collection_scroll_handle = authoring_parity_collection_scroll_handle(ui.cx_mut());
    let collection_drop_status_model = authoring_parity_collection_drop_status_model(ui.cx_mut());
    let stored_collection_assets = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_assets_model, |state| state.clone());
    let collection_selection = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_selection_model, |state| state);
    let collection_box_select = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_box_select_model, |state| state);
    let collection_keyboard = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_keyboard_model, |state| state);
    let collection_tile_extent = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_zoom_model, |state| state);
    let mut collection_reverse_order = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_reverse_order_model, |value| value);
    let collection_rename_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_rename_status_model, |state| state.clone());
    let collection_command_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_command_status_model, |state| state.clone());
    let collection_rename_session = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_rename_session_model, |state| state.clone());
    let collection_rename_focus_pending = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_rename_focus_pending_model, |state| state);
    let collection_layout = proof_collection_layout_metrics(
        collection_scroll_handle.viewport_size().width,
        collection_tile_extent,
    );

    let order_toggle = ui.button_with_options(
        if collection_reverse_order {
            "Show folder order"
        } else {
            "Reverse visible order"
        },
        kit::ButtonOptions {
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.order-toggle",
            )),
            ..Default::default()
        },
    );
    if order_toggle.clicked() {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_reverse_order_model, |value| *value = !*value);
        collection_reverse_order = !collection_reverse_order;
    }

    let collection_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(stored_collection_assets.clone()),
        collection_reverse_order,
    );
    let collection_keys = collection_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let collection_active_id = proof_collection_active_id(
        &collection_keys,
        &collection_selection,
        &collection_keyboard,
    );
    let collection_rename_ready_session = proof_collection_begin_rename_session(
        &collection_assets,
        &collection_selection,
        &collection_keyboard,
    );

    proof_collection_readout_text(
        ui,
        proof_collection_assets_line(&collection_assets),
        "imui-editor-proof.authoring.imui.collection.assets-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_visible_order_line(&collection_assets),
        "imui-editor-proof.authoring.imui.collection.visible-order-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_selection_line(&collection_assets, &collection_selection),
        "imui-editor-proof.authoring.imui.collection.selection-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_active_line(
            &collection_assets,
            &collection_selection,
            &collection_keyboard,
        ),
        "imui-editor-proof.authoring.imui.collection.active-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_zoom_line(collection_layout),
        "imui-editor-proof.authoring.imui.collection.zoom-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_select_all_line(),
        "imui-editor-proof.authoring.imui.collection.select-all-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_rename_line(),
        "imui-editor-proof.authoring.imui.collection.rename-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_context_menu_line(),
        "imui-editor-proof.authoring.imui.collection.context-menu-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_command_package_line(),
        "imui-editor-proof.authoring.imui.collection.command-package-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_rename_status_line(&collection_rename_status),
        "imui-editor-proof.authoring.imui.collection.rename-status-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_command_status_line(&collection_command_status),
        "imui-editor-proof.authoring.imui.collection.command-status-readout",
    );
    render_collection_command_buttons(
        ui,
        ProofCollectionCommandButtonModels {
            assets: collection_assets_model.clone(),
            selection: collection_selection_model.clone(),
            keyboard: collection_keyboard_model.clone(),
            command_status: collection_command_status_model.clone(),
            rename_session: collection_rename_session_model.clone(),
            rename_draft: collection_rename_draft_model.clone(),
            rename_focus_pending: collection_rename_focus_pending_model.clone(),
            rename_status: collection_rename_status_model.clone(),
        },
        ProofCollectionCommandButtonState {
            visible_assets: &collection_assets,
            stored_assets: &stored_collection_assets,
            selection: &collection_selection,
            keyboard: &collection_keyboard,
            reverse_order: collection_reverse_order,
            rename_ready_session: collection_rename_ready_session.as_ref(),
        },
    );

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
        |ui| {
            let collection_assets = collection_assets.clone();
            let collection_keys = collection_keys.clone();
            let collection_assets_model = collection_assets_model.clone();
            let collection_reverse_order_model = collection_reverse_order_model.clone();
            let collection_selection = collection_selection.clone();
            let collection_selection_model = collection_selection_model.clone();
            let collection_box_select_model = collection_box_select_model.clone();
            let collection_box_select = collection_box_select.clone();
            let collection_keyboard_model = collection_keyboard_model.clone();
            let collection_zoom_model = collection_zoom_model.clone();
            let collection_context_menu_anchor_model = collection_context_menu_anchor_model.clone();
            let collection_active_focus_target_model =
                collection_active_focus_target_model.clone();
            let collection_active_id = collection_active_id.clone();
            let collection_rename_session = collection_rename_session.clone();
            let collection_rename_session_model = collection_rename_session_model.clone();
            let collection_rename_draft_model = collection_rename_draft_model.clone();
            let collection_rename_focus_pending_model =
                collection_rename_focus_pending_model.clone();
            let collection_rename_focus_pending = collection_rename_focus_pending;
            let collection_rename_status_model = collection_rename_status_model.clone();
            let collection_command_status_model = collection_command_status_model.clone();
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
                        if down.button != fret_core::MouseButton::Left {
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
                        if up.button == fret_core::MouseButton::Right && up.is_click {
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

    render_collection_context_menu(
        ui,
        ProofCollectionContextMenuModels {
            anchor: collection_context_menu_anchor_model.clone(),
            selection: collection_selection_model.clone(),
            keyboard: collection_keyboard_model.clone(),
            assets: collection_assets_model.clone(),
            reverse_order: collection_reverse_order_model.clone(),
            command_status: collection_command_status_model.clone(),
            rename_session: collection_rename_session_model.clone(),
            rename_draft: collection_rename_draft_model.clone(),
            rename_focus_pending: collection_rename_focus_pending_model.clone(),
            rename_status: collection_rename_status_model.clone(),
        },
    );

    if let Some(session) = collection_rename_session.as_ref()
        && !collection_assets
            .iter()
            .any(|asset| asset.id == session.target_id)
    {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_rename_session_model, |state| *state = None);
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_rename_focus_pending_model, |state| {
                *state = false
            });
    }

    let import_trigger = ui.button_with_options(
        "Import selected set to bundle",
        kit::ButtonOptions {
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.import-target",
            )),
            ..Default::default()
        },
    );
    let import_drop = ui.drop_target::<ProofCollectionDragPayload>(import_trigger);
    if let Some(payload) = import_drop.delivered_payload() {
        let next_status = proof_collection_drop_status("Delivered", &payload);
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_drop_status_model, |status| {
                status.clear();
                status.push_str(&next_status);
            });
    }

    let persisted_collection_status = ui
        .cx_mut()
        .data()
        .selector_model_paint(&collection_drop_status_model, |value| value);
    let visible_collection_status = if let Some(payload) = import_drop.delivered_payload() {
        proof_collection_drop_status("Delivered", &payload)
    } else if let Some(payload) = import_drop.preview_payload() {
        proof_collection_drop_status("Preview", &payload)
    } else if import_drop.active() {
        "Compatible collection drag active".to_string()
    } else {
        persisted_collection_status
    };
    proof_collection_readout_text(
        ui,
        visible_collection_status,
        "imui-editor-proof.authoring.imui.collection.drop-status-readout",
    );
}
