use std::sync::Arc;

use super::super::super::super::super::super::ProofCollectionAsset;

pub(super) fn asset(id: &str, label: &str, path: &str) -> ProofCollectionAsset {
    ProofCollectionAsset {
        id: Arc::from(id),
        label: Arc::from(label),
        path: Arc::from(path),
        kind: Arc::from("Texture"),
        size_kib: 256,
    }
}
