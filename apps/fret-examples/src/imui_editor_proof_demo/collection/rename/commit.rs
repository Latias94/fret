use std::sync::Arc;

use super::super::ProofCollectionAsset;
use super::ProofCollectionRenameSession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct ProofCollectionRenameCommit {
    pub(in super::super) target_id: Arc<str>,
    pub(in super::super) previous_label: Arc<str>,
    pub(in super::super) next_label: Arc<str>,
    pub(in super::super) renamed_assets: Vec<ProofCollectionAsset>,
}

pub(in super::super) fn proof_collection_commit_rename(
    stored_assets: &[ProofCollectionAsset],
    session: &ProofCollectionRenameSession,
    draft: &str,
) -> Option<ProofCollectionRenameCommit> {
    let next_label = Arc::<str>::from(draft.trim());
    if next_label.is_empty() {
        return None;
    }

    let _target = stored_assets
        .iter()
        .find(|asset| asset.id == session.target_id)?;
    let renamed_assets = stored_assets
        .iter()
        .cloned()
        .map(|mut asset| {
            if asset.id == session.target_id {
                asset.label = next_label.clone();
            }
            asset
        })
        .collect::<Vec<_>>();

    Some(ProofCollectionRenameCommit {
        target_id: session.target_id.clone(),
        previous_label: session.original_label.clone(),
        next_label,
        renamed_assets,
    })
}

#[cfg(test)]
mod tests;
