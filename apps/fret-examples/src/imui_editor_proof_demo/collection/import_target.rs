use std::sync::Arc;

use fret::advanced::view::AppRenderDataExt as _;
use fret::imui::{kit, prelude::*};

use super::drag_drop::{ProofCollectionDragPayload, proof_collection_drop_status};
use super::models::authoring_parity_collection_drop_status_model;
use super::{KernelApp, proof_collection_readout_text};

pub(super) fn render_collection_import_target(ui: &mut ImUi<'_, '_, KernelApp>) {
    let collection_drop_status_model = authoring_parity_collection_drop_status_model(ui.cx_mut());
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
