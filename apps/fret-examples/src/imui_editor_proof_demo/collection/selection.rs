use std::sync::Arc;

mod commands;
mod context_menu;
mod keyboard;
mod projection;
mod select_all;

pub(super) use commands::{
    ProofCollectionDeleteResult, ProofCollectionDuplicateResult,
    proof_collection_delete_key_matches, proof_collection_delete_selection,
    proof_collection_duplicate_selection, proof_collection_duplicate_shortcut_matches,
};
pub(super) use context_menu::proof_collection_context_menu_selection;
pub(super) use keyboard::proof_collection_keyboard_selection;
pub(super) use projection::{
    proof_collection_active_id, proof_collection_assets_in_visible_order,
    proof_collection_selected_assets,
};
pub(super) use select_all::{
    proof_collection_select_all_selection, proof_collection_select_all_shortcut_matches,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ProofCollectionKeyboardState {
    pub(super) active_id: Option<Arc<str>>,
}
