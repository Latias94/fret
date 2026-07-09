#[test]
fn workspace_shell_demo_keeps_state_and_dirty_close_owner_explicit() {
    let source = include_str!("../src/workspace_shell_demo/state.rs");

    for needle in [
        "pub(crate) fn build_file_tree_items() -> (Vec<TreeItem>, TreeState) {",
        "pub struct WorkspaceShellWindowState {",
        "pub(crate) const CMD_WORKSPACE_SHELL_DEMO_SET_ACTIVE_DIRTY: &str =",
        "pub(crate) const DIRTY_CLOSE_PROMPT_OVERLAY_ID: GlobalElementId",
        "pub(crate) struct WorkspaceShellDirtyClosePrompt {",
        "pub(crate) struct WorkspaceShellDemoDirtyClosePolicy {",
        "impl WorkspaceDirtyClosePolicy for WorkspaceShellDemoDirtyClosePolicy",
        "WorkspaceCloseReason::CloseWindow",
    ] {
        assert!(
            source.contains(needle),
            "workspace shell demo should keep the state and dirty-close owner explicit; missing `{needle}`"
        );
    }

    for needle in [
        "fn render_ui(",
        "fn handle_command(",
        "fn handle_command_before_ui(",
        "UiTree<App>",
        "pub struct WorkspaceShellDemoDriver;",
        "pub fn build_fn_driver()",
    ] {
        assert!(
            !source.contains(needle),
            "workspace shell demo state owner should not carry driver/render entrypoints; unexpected `{needle}`"
        );
    }
}
