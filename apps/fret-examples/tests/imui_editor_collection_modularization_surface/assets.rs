pub(super) fn assert_assets_owner_split(assets_source: &str) {
    for needle in [
        "pub(in super::super) struct ProofCollectionAsset {",
        "pub(in super::super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
        "ProofCollectionAsset {",
        "id: Arc::from(\"stone-albedo\")",
        "path: Arc::from(\"textures/stone/albedo.ktx2\")",
    ] {
        assert!(
            assets_source.contains(needle),
            "the demo-local collection assets owner should keep asset fixtures explicit; missing `{needle}`"
        );
    }
}
