use std::sync::Arc;

use fret::imui::{kit::ImUiMultiSelectState, prelude::*};

use super::geometry::{ProofCollectionLayoutMetrics, proof_collection_zoom_line};
use super::readouts::{
    proof_collection_active_line, proof_collection_assets_line,
    proof_collection_command_package_line, proof_collection_command_status_line,
    proof_collection_context_menu_line, proof_collection_rename_line,
    proof_collection_rename_status_line, proof_collection_select_all_line,
    proof_collection_selection_line, proof_collection_visible_order_line,
};
use super::selection::ProofCollectionKeyboardState;
use super::{KernelApp, ProofCollectionAsset, proof_collection_readout_text};

pub(super) struct ProofCollectionStatusReadoutState<'a> {
    pub(super) assets: &'a [ProofCollectionAsset],
    pub(super) selection: &'a ImUiMultiSelectState<Arc<str>>,
    pub(super) keyboard: &'a ProofCollectionKeyboardState,
    pub(super) layout: ProofCollectionLayoutMetrics,
    pub(super) rename_status: &'a str,
    pub(super) command_status: &'a str,
}

pub(super) fn render_collection_status_readouts(
    ui: &mut ImUi<'_, '_, KernelApp>,
    state: ProofCollectionStatusReadoutState<'_>,
) {
    proof_collection_readout_text(
        ui,
        proof_collection_assets_line(state.assets),
        "imui-editor-proof.authoring.imui.collection.assets-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_visible_order_line(state.assets),
        "imui-editor-proof.authoring.imui.collection.visible-order-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_selection_line(state.assets, state.selection),
        "imui-editor-proof.authoring.imui.collection.selection-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_active_line(state.assets, state.selection, state.keyboard),
        "imui-editor-proof.authoring.imui.collection.active-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_zoom_line(state.layout),
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
        proof_collection_rename_status_line(state.rename_status),
        "imui-editor-proof.authoring.imui.collection.rename-status-readout",
    );
    proof_collection_readout_text(
        ui,
        proof_collection_command_status_line(state.command_status),
        "imui-editor-proof.authoring.imui.collection.command-status-readout",
    );
}
