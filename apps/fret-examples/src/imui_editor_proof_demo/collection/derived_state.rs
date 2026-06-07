use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

use super::ProofCollectionAsset;
use super::rename::{ProofCollectionRenameSession, proof_collection_begin_rename_session};
use super::selection::{
    ProofCollectionKeyboardState, proof_collection_active_id,
    proof_collection_assets_in_visible_order,
};

pub(super) struct ProofCollectionDerivedState {
    pub(super) assets: Vec<ProofCollectionAsset>,
    pub(super) keys: Vec<Arc<str>>,
    pub(super) active_id: Option<Arc<str>>,
    pub(super) rename_ready_session: Option<ProofCollectionRenameSession>,
}

pub(super) fn proof_collection_derived_state(
    stored_assets: &[ProofCollectionAsset],
    reverse_order: bool,
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> ProofCollectionDerivedState {
    let assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(stored_assets.to_vec()),
        reverse_order,
    );
    let keys = assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let active_id = proof_collection_active_id(&keys, selection, keyboard);
    let rename_ready_session = proof_collection_begin_rename_session(&assets, selection, keyboard);

    ProofCollectionDerivedState {
        assets,
        keys,
        active_id,
        rename_ready_session,
    }
}
