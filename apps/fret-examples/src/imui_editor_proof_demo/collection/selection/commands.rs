mod delete;
mod duplicate;

pub(in super::super) use delete::{
    ProofCollectionDeleteResult, proof_collection_delete_key_matches,
    proof_collection_delete_selection,
};
pub(in super::super) use duplicate::{
    ProofCollectionDuplicateResult, proof_collection_duplicate_selection,
    proof_collection_duplicate_shortcut_matches,
};
