use super::browser_scope::ProofCollectionBrowserScopeModels;
use super::command_buttons::ProofCollectionCommandButtonModels;
use super::context_menu::ProofCollectionContextMenuModels;
use super::runtime_state::ProofCollectionRuntimeModels;

pub(super) struct ProofCollectionChildModels {
    pub(super) command_buttons: ProofCollectionCommandButtonModels,
    pub(super) browser_scope: ProofCollectionBrowserScopeModels,
    pub(super) context_menu: ProofCollectionContextMenuModels,
}

pub(super) fn proof_collection_child_models(
    models: &ProofCollectionRuntimeModels,
) -> ProofCollectionChildModels {
    ProofCollectionChildModels {
        command_buttons: ProofCollectionCommandButtonModels {
            assets: models.assets.clone(),
            selection: models.selection.clone(),
            keyboard: models.keyboard.clone(),
            command_status: models.command_status.clone(),
            rename_session: models.rename_session.clone(),
            rename_draft: models.rename_draft.clone(),
            rename_focus_pending: models.rename_focus_pending.clone(),
            rename_status: models.rename_status.clone(),
        },
        browser_scope: ProofCollectionBrowserScopeModels {
            assets: models.assets.clone(),
            reverse_order: models.reverse_order.clone(),
            selection: models.selection.clone(),
            box_select: models.box_select.clone(),
            keyboard: models.keyboard.clone(),
            zoom: models.zoom.clone(),
            context_menu_anchor: models.context_menu_anchor.clone(),
            active_focus_target: models.active_focus_target.clone(),
            rename_session: models.rename_session.clone(),
            rename_draft: models.rename_draft.clone(),
            rename_focus_pending: models.rename_focus_pending.clone(),
            rename_status: models.rename_status.clone(),
            command_status: models.command_status.clone(),
            scroll: models.scroll.clone(),
        },
        context_menu: ProofCollectionContextMenuModels {
            anchor: models.context_menu_anchor.clone(),
            selection: models.selection.clone(),
            keyboard: models.keyboard.clone(),
            assets: models.assets.clone(),
            reverse_order: models.reverse_order.clone(),
            command_status: models.command_status.clone(),
            rename_session: models.rename_session.clone(),
            rename_draft: models.rename_draft.clone(),
            rename_focus_pending: models.rename_focus_pending.clone(),
            rename_status: models.rename_status.clone(),
        },
    }
}
