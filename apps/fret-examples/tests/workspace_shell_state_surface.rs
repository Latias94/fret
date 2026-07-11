#[test]
fn workspace_shell_demo_keeps_state_only_ownership_explicit() {
    let source = include_str!("../src/workspace_shell_demo/state.rs");

    for needle in [
        "pub(crate) fn build_file_tree_items() -> (Vec<TreeItem>, TreeState) {",
        "pub struct WorkspaceShellWindowState {",
        "pub(crate) workbench: WorkspaceWorkbench,",
        "impl fret::app::UiAppFrameStageSink for WorkspaceShellWindowState",
        "pub(crate) mod act {",
        "fret::actions!([",
        "SetActiveDirty = \"workspace.shell_demo.set_active_dirty\"",
        "ClearActiveDirty = \"workspace.shell_demo.clear_active_dirty\"",
        "SetPaneBActiveDirty = \"workspace.shell_demo.set_pane_b_active_dirty\"",
        "DebugCloseActivePaneA = \"workspace.shell_demo.debug_close_active_in_pane_a\"",
        "CloseWindow = \"window.close\"",
        "ToggleTabstripTwoRowPinned = \"workspace.shell_demo.toggle_tabstrip_two_row_pinned\"",
    ] {
        assert!(
            source.contains(needle),
            "workspace shell state should expose the new state-only owner; missing `{needle}`"
        );
    }

    for forbidden in [
        "WorkspaceDirtyClosePolicy",
        "WorkspaceShellDirtyClosePrompt",
        "DIRTY_CLOSE_PROMPT_OVERLAY_ID",
        "CMD_WORKSPACE_SHELL_DEMO_",
        "fn render_ui(",
        "fn handle_command(",
        "fn handle_command_before_ui(",
        "UiTree<App>",
        "pub struct WorkspaceShellDemoDriver;",
        "pub fn build_fn_driver()",
    ] {
        assert!(
            !source.contains(forbidden),
            "workspace shell state should not retain the obsolete owner `{forbidden}`"
        );
    }
}
