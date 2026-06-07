use std::collections::HashSet;
use std::sync::Arc;

use super::super::super::super::ProofCollectionAsset;

pub(super) struct ProofCollectionDuplicateNameRegistry {
    used_ids: HashSet<String>,
    used_labels: HashSet<String>,
    used_paths: HashSet<String>,
}

impl ProofCollectionDuplicateNameRegistry {
    pub(super) fn from_assets(stored_assets: &[ProofCollectionAsset]) -> Self {
        Self {
            used_ids: stored_assets
                .iter()
                .map(|asset| asset.id.to_string())
                .collect(),
            used_labels: stored_assets
                .iter()
                .map(|asset| asset.label.to_string())
                .collect(),
            used_paths: stored_assets
                .iter()
                .map(|asset| asset.path.to_string())
                .collect(),
        }
    }

    pub(super) fn duplicate_id(&mut self, id: &str) -> Arc<str> {
        proof_collection_unique_copy_text(&mut self.used_ids, |index| {
            proof_collection_duplicate_id_candidate(id, index)
        })
    }

    pub(super) fn duplicate_label(&mut self, label: &str) -> Arc<str> {
        proof_collection_unique_copy_text(&mut self.used_labels, |index| {
            proof_collection_duplicate_label_candidate(label, index)
        })
    }

    pub(super) fn duplicate_path(&mut self, path: &str) -> Arc<str> {
        proof_collection_unique_copy_text(&mut self.used_paths, |index| {
            proof_collection_duplicate_path_candidate(path, index)
        })
    }
}

fn proof_collection_duplicate_label_candidate(label: &str, index: usize) -> String {
    if index == 1 {
        format!("{label} Copy")
    } else {
        format!("{label} Copy {index}")
    }
}

fn proof_collection_duplicate_id_candidate(id: &str, index: usize) -> String {
    if index == 1 {
        format!("{id}-copy")
    } else {
        format!("{id}-copy-{index}")
    }
}

fn proof_collection_duplicate_path_candidate(path: &str, index: usize) -> String {
    let suffix = if index == 1 {
        "-copy".to_string()
    } else {
        format!("-copy-{index}")
    };

    match path.rsplit_once('.') {
        Some((stem, ext)) if !ext.contains('/') => format!("{stem}{suffix}.{ext}"),
        _ => format!("{path}{suffix}"),
    }
}

fn proof_collection_unique_copy_text(
    used: &mut HashSet<String>,
    candidate: impl Fn(usize) -> String,
) -> Arc<str> {
    let mut index = 1;
    loop {
        let value = candidate(index);
        if used.insert(value.clone()) {
            return Arc::from(value);
        }
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
