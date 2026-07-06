use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{KeyCode, Modifiers};

use super::ProofCollectionAsset;
use super::selection::{ProofCollectionKeyboardState, proof_collection_active_id};

mod commit;
mod focus;

pub(super) use commit::{ProofCollectionRenameCommit, proof_collection_commit_rename};
pub(super) use focus::{
    proof_collection_inline_rename_focus_state, proof_collection_restore_focus_after_inline_rename,
    proof_collection_sync_inline_rename_focus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProofCollectionRenameSession {
    pub(super) target_id: Arc<str>,
    pub(super) original_label: Arc<str>,
}

pub(super) fn proof_collection_rename_shortcut_matches(key: KeyCode, modifiers: Modifiers) -> bool {
    key == KeyCode::F2 && modifiers == Modifiers::default()
}

pub(super) fn proof_collection_begin_rename_session(
    visible_assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<ProofCollectionRenameSession> {
    let visible_keys = visible_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let active_id = proof_collection_active_id(&visible_keys, selection, keyboard)?;
    let asset = visible_assets.iter().find(|asset| asset.id == active_id)?;

    Some(ProofCollectionRenameSession {
        target_id: asset.id.clone(),
        original_label: asset.label.clone(),
    })
}

#[cfg(test)]
mod tests;
