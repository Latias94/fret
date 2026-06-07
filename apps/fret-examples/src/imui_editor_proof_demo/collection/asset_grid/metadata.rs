use fret::imui::ImUiFacade;

use super::super::super::KernelApp;
use super::super::{ProofCollectionAsset, proof_collection_readout_text};

pub(super) fn render_collection_asset_metadata_readouts(
    ui: &mut ImUiFacade<'_, '_, KernelApp>,
    asset: &ProofCollectionAsset,
) {
    proof_collection_readout_text(
        ui,
        format!("{} | {} KiB", asset.kind, asset.size_kib),
        "imui-editor-proof.authoring.imui.collection.asset.metadata",
    );
    proof_collection_readout_text(
        ui,
        asset.path.clone(),
        "imui-editor-proof.authoring.imui.collection.asset.path",
    );
}
