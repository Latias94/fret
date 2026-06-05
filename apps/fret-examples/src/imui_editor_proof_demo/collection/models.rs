use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{Point, Px};
use fret_runtime::Model;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::named_demo_state;
use super::geometry::PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX;
use super::rename::ProofCollectionRenameSession;
use super::selection::ProofCollectionKeyboardState;
use super::{
    ProofCollectionAsset, ProofCollectionBoxSelectState, authoring_parity_collection_assets,
};

pub(super) fn authoring_parity_collection_selection_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<ImUiMultiSelectState<Arc<str>>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_selection",
        |cx| {
            let assets = authoring_parity_collection_assets();
            let default_id = assets.first().map(|asset| asset.id.clone());
            let state = default_id
                .clone()
                .map(ImUiMultiSelectState::single)
                .unwrap_or_default();
            cx.app.models_mut().insert(state)
        },
    )
}

pub(super) fn authoring_parity_collection_assets_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<ProofCollectionAsset>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_assets",
        |cx| {
            cx.app.models_mut().insert(
                authoring_parity_collection_assets()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        },
    )
}

pub(super) fn authoring_parity_collection_reverse_order_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<bool> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_reverse_order",
        |cx| cx.app.models_mut().insert(false),
    )
}

pub(super) fn authoring_parity_collection_box_select_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<ProofCollectionBoxSelectState> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_box_select",
        |cx| {
            cx.app
                .models_mut()
                .insert(ProofCollectionBoxSelectState::default())
        },
    )
}

pub(super) fn authoring_parity_collection_keyboard_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<ProofCollectionKeyboardState> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_keyboard",
        |cx| {
            let active_id = authoring_parity_collection_assets()
                .first()
                .map(|asset| asset.id.clone());
            cx.app
                .models_mut()
                .insert(ProofCollectionKeyboardState { active_id })
        },
    )
}

pub(super) fn authoring_parity_collection_zoom_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Px> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_zoom",
        |cx| {
            cx.app
                .models_mut()
                .insert(Px(PROOF_COLLECTION_TILE_EXTENT_DEFAULT_PX))
        },
    )
}

pub(super) fn authoring_parity_collection_scroll_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> ScrollHandle {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.state.authoring_parity.collection_scroll_handle",
        |_cx| ScrollHandle::default(),
    )
}

pub(super) fn authoring_parity_collection_context_menu_anchor_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Point>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_context_menu_anchor",
        |cx| cx.app.models_mut().insert(None::<Point>),
    )
}

pub(super) fn authoring_parity_collection_rename_session_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<ProofCollectionRenameSession>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_rename_session",
        |cx| {
            cx.app
                .models_mut()
                .insert(None::<ProofCollectionRenameSession>)
        },
    )
}

pub(super) fn authoring_parity_collection_rename_draft_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_rename_draft",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

pub(super) fn authoring_parity_collection_rename_focus_pending_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<bool> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_rename_focus_pending",
        |cx| cx.app.models_mut().insert(false),
    )
}

pub(super) fn authoring_parity_collection_active_focus_target_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<GlobalElementId>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_active_focus_target",
        |cx| cx.app.models_mut().insert(None::<GlobalElementId>),
    )
}

pub(super) fn authoring_parity_collection_rename_status_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_rename_status",
        |cx| cx.app.models_mut().insert("Idle".to_string()),
    )
}

pub(super) fn authoring_parity_collection_command_status_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_command_status",
        |cx| cx.app.models_mut().insert("Idle".to_string()),
    )
}

pub(super) fn authoring_parity_collection_drop_status_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.collection_drop_status",
        |cx| cx.app.models_mut().insert("Idle".to_string()),
    )
}
