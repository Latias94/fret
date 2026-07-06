use std::any::Any;
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{Point, Px};
use fret_runtime::{Model, ModelStore};
use fret_ui::GlobalElementId;

use super::ProofCollectionAsset;
use super::readouts::{
    proof_collection_delete_status, proof_collection_duplicate_status,
    proof_collection_rename_cancel_status, proof_collection_rename_commit_status,
    proof_collection_rename_invalid_status, proof_collection_rename_ready_status,
    proof_collection_select_all_status,
};
use super::rename::{ProofCollectionRenameCommit, ProofCollectionRenameSession};
use super::selection::{
    ProofCollectionDeleteResult, ProofCollectionDuplicateResult, ProofCollectionKeyboardState,
};

pub(super) struct ProofCollectionModelOwner<'a> {
    models: &'a mut ModelStore,
}

impl<'a> ProofCollectionModelOwner<'a> {
    pub(super) fn new(models: &'a mut ModelStore) -> Self {
        Self { models }
    }

    fn update<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.models.update(model, f).ok()
    }

    fn set<T: Any>(&mut self, model: &Model<T>, value: T) -> bool {
        self.update(model, |slot| *slot = value).is_some()
    }

    fn replace_string(&mut self, model: &Model<String>, next_value: String) -> bool {
        self.update(model, |value| {
            value.clear();
            value.push_str(&next_value);
        })
        .is_some()
    }

    pub(super) fn apply_duplicate(
        &mut self,
        assets_model: &Model<Vec<ProofCollectionAsset>>,
        selection_model: &Model<ImUiMultiSelectState<Arc<str>>>,
        keyboard_model: &Model<ProofCollectionKeyboardState>,
        command_status_model: &Model<String>,
        duplicate: ProofCollectionDuplicateResult,
    ) {
        let command_status = proof_collection_duplicate_status(&duplicate.duplicated_assets);
        let _ = self.set(assets_model, duplicate.next_assets);
        let _ = self.set(selection_model, duplicate.next_selection);
        let _ = self.set(keyboard_model, duplicate.next_keyboard);
        let _ = self.replace_string(command_status_model, command_status);
    }

    pub(super) fn apply_delete(
        &mut self,
        assets_model: &Model<Vec<ProofCollectionAsset>>,
        selection_model: &Model<ImUiMultiSelectState<Arc<str>>>,
        keyboard_model: &Model<ProofCollectionKeyboardState>,
        command_status_model: &Model<String>,
        delete: ProofCollectionDeleteResult,
    ) {
        let command_status = proof_collection_delete_status(&delete.deleted_assets);
        let _ = self.set(assets_model, delete.remaining_assets);
        let _ = self.set(selection_model, delete.next_selection);
        let _ = self.set(keyboard_model, delete.next_keyboard);
        let _ = self.replace_string(command_status_model, command_status);
    }

    pub(super) fn apply_select_all(
        &mut self,
        selection_model: &Model<ImUiMultiSelectState<Arc<str>>>,
        keyboard_model: &Model<ProofCollectionKeyboardState>,
        command_status_model: &Model<String>,
        next_selection: ImUiMultiSelectState<Arc<str>>,
        next_keyboard: ProofCollectionKeyboardState,
    ) {
        let command_status = proof_collection_select_all_status(next_selection.selected_count());
        let _ = self.set(selection_model, next_selection);
        let _ = self.set(keyboard_model, next_keyboard);
        let _ = self.replace_string(command_status_model, command_status);
    }

    pub(super) fn apply_navigation(
        &mut self,
        selection_model: &Model<ImUiMultiSelectState<Arc<str>>>,
        keyboard_model: &Model<ProofCollectionKeyboardState>,
        next_selection: ImUiMultiSelectState<Arc<str>>,
        next_keyboard: ProofCollectionKeyboardState,
    ) {
        let _ = self.set(selection_model, next_selection);
        let _ = self.set(keyboard_model, next_keyboard);
    }

    pub(super) fn begin_inline_rename(
        &mut self,
        rename_session_model: &Model<Option<ProofCollectionRenameSession>>,
        rename_draft_model: &Model<String>,
        rename_focus_pending_model: &Model<bool>,
        rename_status_model: &Model<String>,
        session: &ProofCollectionRenameSession,
    ) {
        let _ = self.set(rename_session_model, Some(session.clone()));
        let _ = self.replace_string(
            rename_draft_model,
            session.original_label.as_ref().to_string(),
        );
        let _ = self.set(rename_focus_pending_model, true);
        let _ = self.replace_string(
            rename_status_model,
            proof_collection_rename_ready_status(session.original_label.as_ref()),
        );
    }

    pub(super) fn apply_inline_rename_commit(
        &mut self,
        assets_model: &Model<Vec<ProofCollectionAsset>>,
        rename_session_model: &Model<Option<ProofCollectionRenameSession>>,
        rename_focus_pending_model: &Model<bool>,
        rename_status_model: &Model<String>,
        commit: ProofCollectionRenameCommit,
    ) {
        let _ = self.set(assets_model, commit.renamed_assets);
        let _ = self.replace_string(
            rename_status_model,
            proof_collection_rename_commit_status(
                commit.previous_label.as_ref(),
                commit.next_label.as_ref(),
            ),
        );
        let _ = self.set(rename_session_model, None);
        let _ = self.set(rename_focus_pending_model, false);
    }

    pub(super) fn reject_inline_rename(
        &mut self,
        rename_focus_pending_model: &Model<bool>,
        rename_status_model: &Model<String>,
        session: &ProofCollectionRenameSession,
    ) {
        let _ = self.replace_string(
            rename_status_model,
            proof_collection_rename_invalid_status(session.original_label.as_ref()),
        );
        let _ = self.set(rename_focus_pending_model, true);
    }

    pub(super) fn cancel_inline_rename(
        &mut self,
        rename_session_model: &Model<Option<ProofCollectionRenameSession>>,
        rename_focus_pending_model: &Model<bool>,
        rename_status_model: &Model<String>,
        session: &ProofCollectionRenameSession,
    ) {
        let _ = self.replace_string(
            rename_status_model,
            proof_collection_rename_cancel_status(session.original_label.as_ref()),
        );
        let _ = self.set(rename_session_model, None);
        let _ = self.set(rename_focus_pending_model, false);
    }

    pub(super) fn publish_active_focus_target(
        &mut self,
        active_focus_target_model: &Model<Option<GlobalElementId>>,
        focus_target: GlobalElementId,
    ) {
        let _ = self.set(active_focus_target_model, Some(focus_target));
    }

    pub(super) fn publish_context_menu_anchor(
        &mut self,
        context_menu_anchor_model: &Model<Option<Point>>,
        anchor: Point,
    ) {
        let _ = self.set(context_menu_anchor_model, Some(anchor));
    }

    pub(super) fn set_zoom_extent(&mut self, zoom_model: &Model<Px>, extent: Px) {
        let _ = self.set(zoom_model, extent);
    }

    pub(super) fn take_inline_rename_focus_pending(
        &mut self,
        rename_focus_pending_model: &Model<bool>,
    ) -> bool {
        self.update(rename_focus_pending_model, std::mem::take)
            .unwrap_or(false)
    }

    pub(super) fn activate_asset(
        &mut self,
        keyboard_model: &Model<ProofCollectionKeyboardState>,
        asset_id: Arc<str>,
    ) {
        let _ = self.update(keyboard_model, |keyboard| {
            keyboard.active_id = Some(asset_id);
        });
    }

    pub(super) fn apply_context_menu(
        &mut self,
        selection_model: &Model<ImUiMultiSelectState<Arc<str>>>,
        keyboard_model: &Model<ProofCollectionKeyboardState>,
        context_menu_anchor_model: &Model<Option<Point>>,
        next_selection: ImUiMultiSelectState<Arc<str>>,
        next_keyboard: ProofCollectionKeyboardState,
        anchor: Option<Point>,
    ) {
        let _ = self.set(selection_model, next_selection);
        let _ = self.set(keyboard_model, next_keyboard);
        let _ = self.set(context_menu_anchor_model, anchor);
    }

    pub(super) fn clear_context_menu_anchor(
        &mut self,
        context_menu_anchor_model: &Model<Option<Point>>,
    ) {
        let _ = self.set(context_menu_anchor_model, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Px;

    fn asset(id: &str, label: &str) -> ProofCollectionAsset {
        ProofCollectionAsset {
            id: Arc::from(id),
            label: Arc::from(label),
            path: Arc::from("textures/test.ktx2"),
            kind: Arc::from("Texture"),
            size_kib: 1,
        }
    }

    #[test]
    fn proof_collection_model_owner_applies_command_transactions() {
        let mut models = ModelStore::default();
        let assets = models.insert(vec![asset("stone", "Stone")]);
        let selection = models.insert(ImUiMultiSelectState::single(Arc::from("stone")));
        let keyboard = models.insert(ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone")),
        });
        let command_status = models.insert(String::from("Idle"));

        ProofCollectionModelOwner::new(&mut models).apply_duplicate(
            &assets,
            &selection,
            &keyboard,
            &command_status,
            ProofCollectionDuplicateResult {
                next_assets: vec![asset("stone", "Stone"), asset("stone-copy", "Stone Copy")],
                duplicated_assets: vec![asset("stone-copy", "Stone Copy")],
                next_selection: ImUiMultiSelectState::single(Arc::from("stone-copy")),
                next_keyboard: ProofCollectionKeyboardState {
                    active_id: Some(Arc::from("stone-copy")),
                },
            },
        );

        assert_eq!(
            models
                .read(&assets, |state| state
                    .iter()
                    .map(|asset| asset.id.clone())
                    .collect::<Vec<_>>())
                .unwrap(),
            vec![Arc::<str>::from("stone"), Arc::<str>::from("stone-copy")]
        );
        assert_eq!(
            models
                .read(&selection, |state| state.selected().to_vec())
                .unwrap(),
            vec![Arc::<str>::from("stone-copy")]
        );
        assert_eq!(
            models
                .read(&keyboard, |state| state.active_id.clone())
                .unwrap(),
            Some(Arc::<str>::from("stone-copy"))
        );
        assert_eq!(
            models.read(&command_status, Clone::clone).unwrap(),
            "Duplicated 1 asset(s): Stone Copy"
        );

        ProofCollectionModelOwner::new(&mut models).apply_delete(
            &assets,
            &selection,
            &keyboard,
            &command_status,
            ProofCollectionDeleteResult {
                remaining_assets: vec![asset("stone", "Stone")],
                next_selection: ImUiMultiSelectState::single(Arc::from("stone")),
                next_keyboard: ProofCollectionKeyboardState {
                    active_id: Some(Arc::from("stone")),
                },
                deleted_assets: vec![asset("stone-copy", "Stone Copy")],
                deleted_ids: vec![Arc::from("stone-copy")],
            },
        );

        assert_eq!(
            models
                .read(&assets, |state| state
                    .iter()
                    .map(|asset| asset.id.clone())
                    .collect::<Vec<_>>())
                .unwrap(),
            vec![Arc::<str>::from("stone")]
        );
        assert_eq!(
            models.read(&command_status, Clone::clone).unwrap(),
            "Deleted 1 asset(s): Stone Copy"
        );

        ProofCollectionModelOwner::new(&mut models).apply_select_all(
            &selection,
            &keyboard,
            &command_status,
            ImUiMultiSelectState::new(
                vec![Arc::<str>::from("stone"), Arc::<str>::from("water")],
                Some(Arc::from("stone")),
            ),
            ProofCollectionKeyboardState {
                active_id: Some(Arc::from("water")),
            },
        );

        assert_eq!(
            models
                .read(&selection, |state| state.selected().to_vec())
                .unwrap(),
            vec![Arc::<str>::from("stone"), Arc::<str>::from("water")]
        );
        assert_eq!(
            models
                .read(&keyboard, |state| state.active_id.clone())
                .unwrap(),
            Some(Arc::<str>::from("water"))
        );
        assert_eq!(
            models.read(&command_status, Clone::clone).unwrap(),
            "Selected all 2 visible asset(s)."
        );

        ProofCollectionModelOwner::new(&mut models).apply_navigation(
            &selection,
            &keyboard,
            ImUiMultiSelectState::single(Arc::from("stone")),
            ProofCollectionKeyboardState {
                active_id: Some(Arc::from("stone")),
            },
        );

        assert_eq!(
            models
                .read(&selection, |state| state.selected().to_vec())
                .unwrap(),
            vec![Arc::<str>::from("stone")]
        );
        assert_eq!(
            models
                .read(&keyboard, |state| state.active_id.clone())
                .unwrap(),
            Some(Arc::<str>::from("stone"))
        );
    }

    #[test]
    fn proof_collection_model_owner_applies_rename_and_tile_state() {
        let mut models = ModelStore::default();
        let rename_session = models.insert(None::<ProofCollectionRenameSession>);
        let rename_draft = models.insert(String::new());
        let rename_focus_pending = models.insert(false);
        let rename_status = models.insert(String::from("Idle"));
        let active_focus_target = models.insert(None::<GlobalElementId>);
        let keyboard = models.insert(ProofCollectionKeyboardState::default());
        let selection = models.insert(ImUiMultiSelectState::default());
        let context_menu_anchor = models.insert(None::<Point>);
        let zoom = models.insert(Px(96.0));

        let session = ProofCollectionRenameSession {
            target_id: Arc::from("stone"),
            original_label: Arc::from("Stone"),
        };

        ProofCollectionModelOwner::new(&mut models).begin_inline_rename(
            &rename_session,
            &rename_draft,
            &rename_focus_pending,
            &rename_status,
            &session,
        );

        assert_eq!(
            models.read(&rename_session, Clone::clone).unwrap(),
            Some(session)
        );
        assert_eq!(models.read(&rename_draft, Clone::clone).unwrap(), "Stone");
        assert_eq!(models.get_copied(&rename_focus_pending), Some(true));
        assert_eq!(
            models.read(&rename_status, Clone::clone).unwrap(),
            "Rename ready: Stone. The inline editor will focus, Enter commits, and Escape or blur cancels."
        );

        ProofCollectionModelOwner::new(&mut models)
            .publish_active_focus_target(&active_focus_target, GlobalElementId(42));
        ProofCollectionModelOwner::new(&mut models)
            .publish_context_menu_anchor(&context_menu_anchor, Point::new(Px(4.0), Px(8.0)));
        ProofCollectionModelOwner::new(&mut models).set_zoom_extent(&zoom, Px(128.0));
        ProofCollectionModelOwner::new(&mut models).activate_asset(&keyboard, Arc::from("stone"));
        ProofCollectionModelOwner::new(&mut models).apply_context_menu(
            &selection,
            &keyboard,
            &context_menu_anchor,
            ImUiMultiSelectState::single(Arc::from("stone")),
            ProofCollectionKeyboardState {
                active_id: Some(Arc::from("stone")),
            },
            Some(Point::new(Px(12.0), Px(24.0))),
        );

        assert_eq!(
            models.get_copied(&active_focus_target),
            Some(Some(GlobalElementId(42)))
        );
        assert_eq!(
            models.get_copied(&context_menu_anchor),
            Some(Some(Point::new(Px(12.0), Px(24.0))))
        );
        assert_eq!(models.get_copied(&zoom), Some(Px(128.0)));
        assert!(
            ProofCollectionModelOwner::new(&mut models)
                .take_inline_rename_focus_pending(&rename_focus_pending)
        );
        assert!(
            !ProofCollectionModelOwner::new(&mut models)
                .take_inline_rename_focus_pending(&rename_focus_pending)
        );
        assert_eq!(
            models
                .read(&selection, |state| state.selected().to_vec())
                .unwrap(),
            vec![Arc::<str>::from("stone")]
        );
        assert_eq!(
            models
                .read(&keyboard, |state| state.active_id.clone())
                .unwrap(),
            Some(Arc::<str>::from("stone"))
        );
        ProofCollectionModelOwner::new(&mut models).clear_context_menu_anchor(&context_menu_anchor);
        assert_eq!(models.get_copied(&context_menu_anchor), Some(None));
    }

    #[test]
    fn proof_collection_model_owner_applies_inline_rename_outcomes() {
        let mut models = ModelStore::default();
        let assets = models.insert(vec![asset("stone", "Stone")]);
        let rename_session = models.insert(None::<ProofCollectionRenameSession>);
        let rename_focus_pending = models.insert(true);
        let rename_status = models.insert(String::from("Idle"));

        let session = ProofCollectionRenameSession {
            target_id: Arc::from("stone"),
            original_label: Arc::from("Stone"),
        };
        let commit = ProofCollectionRenameCommit {
            target_id: Arc::from("stone"),
            previous_label: Arc::from("Stone"),
            next_label: Arc::from("Polished Stone"),
            renamed_assets: vec![asset("stone", "Polished Stone")],
        };

        ProofCollectionModelOwner::new(&mut models).apply_inline_rename_commit(
            &assets,
            &rename_session,
            &rename_focus_pending,
            &rename_status,
            commit,
        );

        assert_eq!(
            models
                .read(&assets, |state| state
                    .iter()
                    .map(|asset| asset.label.clone())
                    .collect::<Vec<_>>())
                .unwrap(),
            vec![Arc::<str>::from("Polished Stone")]
        );
        assert_eq!(models.read(&rename_session, Clone::clone).unwrap(), None);
        assert_eq!(models.get_copied(&rename_focus_pending), Some(false));
        assert_eq!(
            models.read(&rename_status, Clone::clone).unwrap(),
            "Renamed Stone -> Polished Stone."
        );

        ProofCollectionModelOwner::new(&mut models).reject_inline_rename(
            &rename_focus_pending,
            &rename_status,
            &session,
        );

        assert_eq!(models.get_copied(&rename_focus_pending), Some(true));
        assert_eq!(
            models.read(&rename_status, Clone::clone).unwrap(),
            "Rename for Stone still needs a non-empty label."
        );

        let _ = models.update(&rename_session, |state| *state = Some(session.clone()));
        ProofCollectionModelOwner::new(&mut models).cancel_inline_rename(
            &rename_session,
            &rename_focus_pending,
            &rename_status,
            &session,
        );

        assert_eq!(models.read(&rename_session, Clone::clone).unwrap(), None);
        assert_eq!(models.get_copied(&rename_focus_pending), Some(false));
        assert_eq!(
            models.read(&rename_status, Clone::clone).unwrap(),
            "Rename canceled for Stone."
        );
    }
}
