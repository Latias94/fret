use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{MouseButton, Point, Px};
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{Length, PointerRegionProps};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, GlobalElementId};

use super::super::box_select::{
    ProofCollectionBoxSelectSession, ProofCollectionBoxSelectState, ProofCollectionRenderedItem,
    proof_collection_box_select_selection,
};
use super::super::geometry::{ProofCollectionLayoutMetrics, proof_collection_drag_threshold_met};
use super::super::keyboard::{
    ProofCollectionKeyboardHandlerModels, install_collection_keyboard_handler,
};
use super::super::rename::ProofCollectionRenameSession;
use super::super::selection::ProofCollectionKeyboardState;
use super::super::{KernelApp, ProofCollectionAsset};

mod context_menu;
mod zoom;

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
    let rendered_items_for_move = rendered_items.clone();
    let rendered_items_for_up = rendered_items;
    let selection_model_for_down = models.selection.clone();
    let selection_model_for_move = models.selection.clone();
    let selection_model_for_up = models.selection.clone();
    let keyboard_model_for_move = models.keyboard.clone();
    let keyboard_model_for_up = models.keyboard.clone();
    let keyboard_model_for_clear = models.keyboard.clone();
    let context_menu_anchor_model_for_up = models.context_menu_anchor.clone();
    let box_select_model_for_down = models.box_select.clone();
    let box_select_model_for_move = models.box_select.clone();
    let box_select_model_for_up = models.box_select.clone();
    let box_select_model_for_cancel = models.box_select.clone();
    let collection_keys_for_move = collection_keys.clone();
    let collection_keys_for_up = collection_keys;

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
        if publish_collection_browser_scope_context_menu_anchor(
            host,
            acx,
            &context_menu_anchor_model_for_up,
            &up,
        ) {
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
}
