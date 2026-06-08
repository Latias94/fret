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
mod tests {
    use super::super::super::authoring_parity_collection_assets;
    use super::*;

    #[test]
    fn proof_collection_commit_rename_updates_label_without_touching_order_or_ids() {
        let stored_assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let session = ProofCollectionRenameSession {
            target_id: Arc::from("stone-normal"),
            original_label: Arc::from("Stone Normal"),
        };

        let commit =
            proof_collection_commit_rename(&stored_assets, &session, "Stone Detail Normal")
                .expect("non-empty rename should commit");

        assert_eq!(commit.target_id, Arc::from("stone-normal"));
        assert_eq!(commit.previous_label, Arc::from("Stone Normal"));
        assert_eq!(commit.next_label, Arc::from("Stone Detail Normal"));
        assert_eq!(
            commit
                .renamed_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>(),
            stored_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            commit
                .renamed_assets
                .iter()
                .find(|asset| asset.id == Arc::from("stone-normal"))
                .map(|asset| asset.label.clone()),
            Some(Arc::from("Stone Detail Normal"))
        );
    }

    #[test]
    fn proof_collection_commit_rename_rejects_empty_trimmed_label() {
        let stored_assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let session = ProofCollectionRenameSession {
            target_id: Arc::from("stone-normal"),
            original_label: Arc::from("Stone Normal"),
        };

        assert!(
            proof_collection_commit_rename(&stored_assets, &session, "   ").is_none(),
            "inline rename should reject empty trimmed labels so the app-local editor can stay open"
        );
    }
}
