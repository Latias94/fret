use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::Modifiers;
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId};

use super::super::KernelApp;
use super::ProofCollectionAsset;
use super::rename::{
    ProofCollectionRenameSession, proof_collection_begin_rename_session,
    proof_collection_rename_shortcut_matches,
};
use super::selection::{
    ProofCollectionKeyboardState, proof_collection_assets_in_visible_order,
    proof_collection_delete_key_matches, proof_collection_delete_selection,
    proof_collection_duplicate_selection, proof_collection_duplicate_shortcut_matches,
    proof_collection_keyboard_selection, proof_collection_select_all_selection,
    proof_collection_select_all_shortcut_matches,
};

mod actions;

use actions::{
    proof_collection_keyboard_apply_delete, proof_collection_keyboard_apply_duplicate,
    proof_collection_keyboard_apply_navigation, proof_collection_keyboard_apply_select_all,
    proof_collection_keyboard_begin_rename,
};

pub(super) struct ProofCollectionKeyboardHandlerModels {
    pub(super) assets: Model<Vec<ProofCollectionAsset>>,
    pub(super) reverse_order: Model<bool>,
    pub(super) selection: Model<ImUiMultiSelectState<Arc<str>>>,
    pub(super) keyboard: Model<ProofCollectionKeyboardState>,
    pub(super) rename_session: Model<Option<ProofCollectionRenameSession>>,
    pub(super) rename_draft: Model<String>,
    pub(super) rename_focus_pending: Model<bool>,
    pub(super) rename_status: Model<String>,
    pub(super) command_status: Model<String>,
}

pub(super) fn install_collection_keyboard_handler(
    cx: &mut ElementContext<'_, KernelApp>,
    scope_id: GlobalElementId,
    columns: usize,
    models: ProofCollectionKeyboardHandlerModels,
) {
    cx.key_on_key_down_for(
        scope_id,
        Arc::new(move |host, acx, down| {
            if down.ime_composing {
                return false;
            }

            let selection = host
                .models_mut()
                .read(&models.selection, |state| state.clone())
                .unwrap_or_default();
            let keyboard = host
                .models_mut()
                .read(&models.keyboard, |state| state.clone())
                .unwrap_or_default();
            let stored_assets = host
                .models_mut()
                .read(&models.assets, |state| state.clone())
                .unwrap_or_default();
            let reverse_order = host
                .models_mut()
                .read(&models.reverse_order, |value| *value)
                .unwrap_or(false);
            let visible_assets = proof_collection_assets_in_visible_order(
                Arc::<[ProofCollectionAsset]>::from(stored_assets.clone()),
                reverse_order,
            );
            if host
                .models_mut()
                .read(&models.rename_session, |state| state.is_some())
                .unwrap_or(false)
            {
                return false;
            }
            let collection_keys = visible_assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect::<Vec<_>>();

            if down.modifiers == Modifiers::default()
                && proof_collection_delete_key_matches(down.key)
                && let Some(delete) = proof_collection_delete_selection(
                    &visible_assets,
                    &stored_assets,
                    &selection,
                    &keyboard,
                )
            {
                proof_collection_keyboard_apply_delete(host, acx, &models, delete);
                return true;
            }

            if proof_collection_rename_shortcut_matches(down.key, down.modifiers)
                && let Some(session) =
                    proof_collection_begin_rename_session(&visible_assets, &selection, &keyboard)
            {
                proof_collection_keyboard_begin_rename(host, acx, &models, session);
                return true;
            }

            if proof_collection_select_all_shortcut_matches(down.key, down.modifiers)
                && let Some((next_selection, next_keyboard)) =
                    proof_collection_select_all_selection(&collection_keys, &selection, &keyboard)
            {
                proof_collection_keyboard_apply_select_all(
                    host,
                    acx,
                    &models,
                    next_selection,
                    next_keyboard,
                );
                return true;
            }

            if proof_collection_duplicate_shortcut_matches(down.key, down.modifiers)
                && let Some(duplicate) = proof_collection_duplicate_selection(
                    &visible_assets,
                    &stored_assets,
                    &selection,
                    &keyboard,
                    reverse_order,
                )
            {
                proof_collection_keyboard_apply_duplicate(host, acx, &models, duplicate);
                return true;
            }

            let Some((next_selection, next_keyboard)) = proof_collection_keyboard_selection(
                &collection_keys,
                &selection,
                &keyboard,
                columns,
                down.key,
                down.modifiers,
            ) else {
                return false;
            };

            proof_collection_keyboard_apply_navigation(
                host,
                acx,
                &models,
                next_selection,
                next_keyboard,
            );
            true
        }),
    );
}
