pub(super) fn assert_render_states_owner_split(
    collection_source: &str,
    render_states_source: &str,
) {
    for needle in [
        "pub(super) struct ProofCollectionRenderStates",
        "pub(super) fn proof_collection_render_states<'a>(",
        "runtime: &'a ProofCollectionRuntimeState",
        "state: &'a ProofCollectionDerivedState",
        "status_readouts: ProofCollectionStatusReadoutState {",
        "command_buttons: ProofCollectionCommandButtonState {",
        "browser_scope: ProofCollectionBrowserScopeState {",
        "rename_ready_session: state.rename_ready_session.as_ref()",
        "rename_session: runtime.snapshot.rename_session()",
        "rename_focus_pending: runtime.snapshot.rename_focus_pending",
    ] {
        assert!(
            render_states_source.contains(needle),
            "the demo-local collection render-state owner should keep child render-state projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionStatusReadoutState {",
        "ProofCollectionCommandButtonState {",
        "ProofCollectionBrowserScopeState {",
        "collection_runtime.snapshot.rename_status.as_str()",
        "collection_runtime.snapshot.command_status.as_str()",
        "collection_runtime.snapshot.rename_session()",
        "collection_state.rename_ready_session.as_ref()",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route child render-state projection through collection/render_states.rs; unexpected `{needle}`"
        );
    }
}
