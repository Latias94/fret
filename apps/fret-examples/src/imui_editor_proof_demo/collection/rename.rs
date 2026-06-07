use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{KeyCode, Modifiers};
use fret_runtime::Model;

use super::super::KernelApp;
use super::ProofCollectionAsset;
use super::readouts::proof_collection_rename_ready_status;
use super::selection::{ProofCollectionKeyboardState, proof_collection_active_id};

mod focus;

pub(super) use focus::{
    proof_collection_inline_rename_focus_state, proof_collection_restore_focus_after_inline_rename,
    proof_collection_sync_inline_rename_focus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProofCollectionRenameSession {
    pub(super) target_id: Arc<str>,
    pub(super) original_label: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProofCollectionRenameCommit {
    pub(super) target_id: Arc<str>,
    pub(super) previous_label: Arc<str>,
    pub(super) next_label: Arc<str>,
    pub(super) renamed_assets: Vec<ProofCollectionAsset>,
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

pub(super) fn proof_collection_begin_inline_rename_in_app(
    app: &mut KernelApp,
    rename_session_model: &Model<Option<ProofCollectionRenameSession>>,
    rename_draft_model: &Model<String>,
    rename_focus_pending_model: &Model<bool>,
    rename_status_model: &Model<String>,
    session: &ProofCollectionRenameSession,
) {
    let _ = app.models_mut().update(rename_session_model, |state| {
        *state = Some(session.clone());
    });
    let _ = app.models_mut().update(rename_draft_model, |draft| {
        draft.clear();
        draft.push_str(session.original_label.as_ref());
    });
    let _ = app
        .models_mut()
        .update(rename_focus_pending_model, |state| {
            *state = true;
        });
    let _ = app.models_mut().update(rename_status_model, |status| {
        status.clear();
        status.push_str(&proof_collection_rename_ready_status(
            session.original_label.as_ref(),
        ));
    });
}

pub(super) fn proof_collection_commit_rename(
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
    use super::super::authoring_parity_collection_assets;
    use super::*;

    fn selection_state(selected: &[&str], anchor: Option<&str>) -> ImUiMultiSelectState<Arc<str>> {
        ImUiMultiSelectState::new(
            selected.iter().map(|id| Arc::from(*id)).collect(),
            anchor.map(Arc::from),
        )
    }

    #[test]
    fn proof_collection_begin_rename_session_prefers_active_visible_asset() {
        let visible_assets = authoring_parity_collection_assets();
        let selection = selection_state(&["stone-albedo", "stone-normal"], Some("stone-albedo"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-normal")),
        };

        let session = proof_collection_begin_rename_session(&visible_assets, &selection, &keyboard)
            .expect("rename should target the active visible asset");

        assert_eq!(session.target_id, Arc::from("stone-normal"));
        assert_eq!(session.original_label, Arc::from("Stone Normal"));
    }

    #[test]
    fn proof_collection_begin_rename_session_falls_back_to_first_visible_asset() {
        let visible_assets = authoring_parity_collection_assets();
        let selection = ImUiMultiSelectState::default();
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("missing")),
        };

        let session = proof_collection_begin_rename_session(&visible_assets, &selection, &keyboard)
            .expect("rename should fall back to the first visible asset");

        assert_eq!(session.target_id, Arc::from("stone-albedo"));
        assert_eq!(session.original_label, Arc::from("Stone Albedo"));
    }

    #[test]
    fn proof_collection_rename_shortcut_matches_plain_f2_only() {
        assert!(proof_collection_rename_shortcut_matches(
            KeyCode::F2,
            Modifiers::default(),
        ));
        assert!(!proof_collection_rename_shortcut_matches(
            KeyCode::F2,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_rename_shortcut_matches(
            KeyCode::F2,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_rename_shortcut_matches(
            KeyCode::KeyA,
            Modifiers::default(),
        ));
    }

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
