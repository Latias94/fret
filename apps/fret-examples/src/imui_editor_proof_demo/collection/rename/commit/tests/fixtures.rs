use std::sync::Arc;

use super::super::super::super::{ProofCollectionAsset, authoring_parity_collection_assets};
use super::super::super::ProofCollectionRenameSession;

pub(super) fn stored_assets() -> Vec<ProofCollectionAsset> {
    authoring_parity_collection_assets()
        .iter()
        .cloned()
        .collect::<Vec<_>>()
}

pub(super) fn rename_session() -> ProofCollectionRenameSession {
    ProofCollectionRenameSession {
        target_id: Arc::from("stone-normal"),
        original_label: Arc::from("Stone Normal"),
    }
}
