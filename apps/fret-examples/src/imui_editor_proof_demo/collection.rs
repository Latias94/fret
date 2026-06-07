use fret::imui::prelude::*;

use super::KernelApp;

mod asset_grid;
mod assets;
mod box_select;
mod browser_scope;
mod chrome;
mod command_buttons;
mod context_menu;
mod derived_state;
mod drag_drop;
mod geometry;
mod import_target;
mod keyboard;
mod models;
mod order_toggle;
mod readouts;
mod rename;
mod runtime_state;
mod selection;
mod status_readouts;

pub(super) use assets::{ProofCollectionAsset, authoring_parity_collection_assets};
pub(super) use chrome::proof_collection_readout_text;

use browser_scope::{
    ProofCollectionBrowserScopeModels, ProofCollectionBrowserScopeState,
    render_collection_browser_scope,
};
use chrome::proof_collection_section_label;
use command_buttons::{
    ProofCollectionCommandButtonModels, ProofCollectionCommandButtonState,
    render_collection_command_buttons,
};
use context_menu::{ProofCollectionContextMenuModels, render_collection_context_menu};
use derived_state::proof_collection_derived_state;
use import_target::render_collection_import_target;
use order_toggle::render_collection_order_toggle;
use runtime_state::proof_collection_runtime_state;
use status_readouts::{ProofCollectionStatusReadoutState, render_collection_status_readouts};

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

    let collection_runtime = proof_collection_runtime_state(ui);

    let collection_reverse_order = render_collection_order_toggle(
        ui,
        &collection_runtime.models.reverse_order,
        collection_runtime.snapshot.reverse_order,
    );

    let collection_state = proof_collection_derived_state(
        &collection_runtime.snapshot.stored_assets,
        collection_reverse_order,
        &collection_runtime.snapshot.selection,
        &collection_runtime.snapshot.keyboard,
    );

    render_collection_status_readouts(
        ui,
        ProofCollectionStatusReadoutState {
            assets: &collection_state.assets,
            selection: &collection_runtime.snapshot.selection,
            keyboard: &collection_runtime.snapshot.keyboard,
            layout: collection_runtime.snapshot.layout,
            rename_status: collection_runtime.snapshot.rename_status.as_str(),
            command_status: collection_runtime.snapshot.command_status.as_str(),
        },
    );
    render_collection_command_buttons(
        ui,
        ProofCollectionCommandButtonModels {
            assets: collection_runtime.models.assets.clone(),
            selection: collection_runtime.models.selection.clone(),
            keyboard: collection_runtime.models.keyboard.clone(),
            command_status: collection_runtime.models.command_status.clone(),
            rename_session: collection_runtime.models.rename_session.clone(),
            rename_draft: collection_runtime.models.rename_draft.clone(),
            rename_focus_pending: collection_runtime.models.rename_focus_pending.clone(),
            rename_status: collection_runtime.models.rename_status.clone(),
        },
        ProofCollectionCommandButtonState {
            visible_assets: &collection_state.assets,
            stored_assets: &collection_runtime.snapshot.stored_assets,
            selection: &collection_runtime.snapshot.selection,
            keyboard: &collection_runtime.snapshot.keyboard,
            reverse_order: collection_reverse_order,
            rename_ready_session: collection_state.rename_ready_session.as_ref(),
        },
    );

    render_collection_browser_scope(
        ui,
        ProofCollectionBrowserScopeModels {
            assets: collection_runtime.models.assets.clone(),
            reverse_order: collection_runtime.models.reverse_order.clone(),
            selection: collection_runtime.models.selection.clone(),
            box_select: collection_runtime.models.box_select.clone(),
            keyboard: collection_runtime.models.keyboard.clone(),
            zoom: collection_runtime.models.zoom.clone(),
            context_menu_anchor: collection_runtime.models.context_menu_anchor.clone(),
            active_focus_target: collection_runtime.models.active_focus_target.clone(),
            rename_session: collection_runtime.models.rename_session.clone(),
            rename_draft: collection_runtime.models.rename_draft.clone(),
            rename_focus_pending: collection_runtime.models.rename_focus_pending.clone(),
            rename_status: collection_runtime.models.rename_status.clone(),
            command_status: collection_runtime.models.command_status.clone(),
            scroll: collection_runtime.models.scroll.clone(),
        },
        ProofCollectionBrowserScopeState {
            assets: &collection_state.assets,
            keys: &collection_state.keys,
            selection: &collection_runtime.snapshot.selection,
            box_select: &collection_runtime.snapshot.box_select,
            active_id: collection_state.active_id.as_ref(),
            rename_session: collection_runtime.snapshot.rename_session.as_ref(),
            rename_focus_pending: collection_runtime.snapshot.rename_focus_pending,
            layout: collection_runtime.snapshot.layout,
        },
    );

    render_collection_context_menu(
        ui,
        ProofCollectionContextMenuModels {
            anchor: collection_runtime.models.context_menu_anchor.clone(),
            selection: collection_runtime.models.selection.clone(),
            keyboard: collection_runtime.models.keyboard.clone(),
            assets: collection_runtime.models.assets.clone(),
            reverse_order: collection_runtime.models.reverse_order.clone(),
            command_status: collection_runtime.models.command_status.clone(),
            rename_session: collection_runtime.models.rename_session.clone(),
            rename_draft: collection_runtime.models.rename_draft.clone(),
            rename_focus_pending: collection_runtime.models.rename_focus_pending.clone(),
            rename_status: collection_runtime.models.rename_status.clone(),
        },
    );

    if let Some(session) = collection_runtime.snapshot.rename_session.as_ref()
        && !collection_state
            .assets
            .iter()
            .any(|asset| asset.id == session.target_id)
    {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_runtime.models.rename_session, |state| {
                *state = None
            });
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&collection_runtime.models.rename_focus_pending, |state| {
                *state = false
            });
    }

    render_collection_import_target(ui);
}
