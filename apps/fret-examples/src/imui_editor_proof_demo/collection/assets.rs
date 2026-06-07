use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct ProofCollectionAsset {
    pub(in super::super) id: Arc<str>,
    pub(in super::super) label: Arc<str>,
    pub(in super::super) path: Arc<str>,
    pub(in super::super) kind: Arc<str>,
    pub(in super::super) size_kib: u32,
}

pub(in super::super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {
    vec![
        ProofCollectionAsset {
            id: Arc::from("stone-albedo"),
            label: Arc::from("Stone Albedo"),
            path: Arc::from("textures/stone/albedo.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 512,
        },
        ProofCollectionAsset {
            id: Arc::from("stone-normal"),
            label: Arc::from("Stone Normal"),
            path: Arc::from("textures/stone/normal.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 384,
        },
        ProofCollectionAsset {
            id: Arc::from("stone-orm"),
            label: Arc::from("Stone ORM"),
            path: Arc::from("textures/stone/orm.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 256,
        },
        ProofCollectionAsset {
            id: Arc::from("moss-overlay"),
            label: Arc::from("Moss Overlay"),
            path: Arc::from("textures/moss/overlay.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 196,
        },
        ProofCollectionAsset {
            id: Arc::from("pebble-height"),
            label: Arc::from("Pebble Height"),
            path: Arc::from("textures/pebble/height.ktx2"),
            kind: Arc::from("Height"),
            size_kib: 164,
        },
        ProofCollectionAsset {
            id: Arc::from("dust-mask"),
            label: Arc::from("Dust Mask"),
            path: Arc::from("textures/shared/dust-mask.ktx2"),
            kind: Arc::from("Mask"),
            size_kib: 72,
        },
    ]
    .into()
}
