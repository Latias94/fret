use super::browser_scope::ProofCollectionBrowserScopeState;
use super::command_buttons::ProofCollectionCommandButtonState;
use super::derived_state::ProofCollectionDerivedState;
use super::runtime_state::ProofCollectionRuntimeState;
use super::status_readouts::ProofCollectionStatusReadoutState;

pub(super) struct ProofCollectionRenderStates<'a> {
    pub(super) status_readouts: ProofCollectionStatusReadoutState<'a>,
    pub(super) command_buttons: ProofCollectionCommandButtonState<'a>,
    pub(super) browser_scope: ProofCollectionBrowserScopeState<'a>,
}

pub(super) fn proof_collection_render_states<'a>(
    runtime: &'a ProofCollectionRuntimeState,
    state: &'a ProofCollectionDerivedState,
    reverse_order: bool,
) -> ProofCollectionRenderStates<'a> {
    ProofCollectionRenderStates {
        status_readouts: ProofCollectionStatusReadoutState {
            assets: &state.assets,
            selection: &runtime.snapshot.selection,
            keyboard: &runtime.snapshot.keyboard,
            layout: runtime.snapshot.layout,
            rename_status: runtime.snapshot.rename_status.as_str(),
            command_status: runtime.snapshot.command_status.as_str(),
        },
        command_buttons: ProofCollectionCommandButtonState {
            visible_assets: &state.assets,
            stored_assets: &runtime.snapshot.stored_assets,
            selection: &runtime.snapshot.selection,
            keyboard: &runtime.snapshot.keyboard,
            reverse_order,
            rename_ready_session: state.rename_ready_session.as_ref(),
        },
        browser_scope: ProofCollectionBrowserScopeState {
            assets: &state.assets,
            keys: &state.keys,
            selection: &runtime.snapshot.selection,
            box_select: &runtime.snapshot.box_select,
            active_id: state.active_id.as_ref(),
            rename_session: runtime.snapshot.rename_session(),
            rename_focus_pending: runtime.snapshot.rename_focus_pending,
            layout: runtime.snapshot.layout,
        },
    }
}
