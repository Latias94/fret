use std::sync::Arc;

use super::super::super::super::super::ProofCollectionAsset;
use super::ProofCollectionDuplicateNameRegistry;

fn asset(id: &str, label: &str, path: &str) -> ProofCollectionAsset {
    ProofCollectionAsset {
        id: Arc::from(id),
        label: Arc::from(label),
        path: Arc::from(path),
        kind: Arc::from("Texture"),
        size_kib: 256,
    }
}

#[test]
fn proof_collection_duplicate_name_registry_uses_unique_copy_suffixes() {
    let stored_assets = vec![
        asset("stone-normal", "Stone Normal", "textures/stone/normal.ktx2"),
        asset(
            "stone-normal-copy",
            "Stone Normal Copy",
            "textures/stone/normal-copy.ktx2",
        ),
    ];
    let mut registry = ProofCollectionDuplicateNameRegistry::from_assets(&stored_assets);

    assert_eq!(
        registry.duplicate_id("stone-normal"),
        Arc::from("stone-normal-copy-2")
    );
    assert_eq!(
        registry.duplicate_label("Stone Normal"),
        Arc::from("Stone Normal Copy 2")
    );
    assert_eq!(
        registry.duplicate_path("textures/stone/normal.ktx2"),
        Arc::from("textures/stone/normal-copy-2.ktx2")
    );
}
