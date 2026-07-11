fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn workspace_shell_driver_routes_workspace_transactions_through_workbench() {
    let source = include_str!("../src/workspace_shell_demo/driver.rs");
    let compact_source = compact(source);

    for needle in [
        "structWorkspaceShellModelBundle{window_layout:Model<WorkspaceWindowLayout>,workbench:WorkspaceWorkbench,",
        "fnnew(app:&mutApp,window_layout:WorkspaceWindowLayout,file_tree_items:Vec<TreeItem>,file_tree_state:TreeState,block_dirty_close:bool,)->Self{",
        "WorkspaceWorkbench::new(app.models_mut(),window_layout.clone(),block_dirty_close);",
        "implfret::workspace::WorkspaceWindowStateforWorkspaceShellWindowState{",
        "fnworkspace_workbench(&self)->&WorkspaceWorkbench{",
        "fnsave_workspace_dirty_close(",
        "typed_command_id::<shell_act::SetActiveDirty>()",
        "typed_command_id::<shell_act::SetPaneBActiveDirty>()",
        "typed_command_id::<shell_act::ClearActiveDirty>()",
        "typed_command_id::<shell_act::ToggleTabstripTwoRowPinned>()",
        "typed_command_id::<shell_act::DebugCloseActivePaneA>()",
        "letclose_window=typed_command_id::<shell_act::CloseWindow,>();",
        "WorkspaceApp::new(\"workspace-shell-demo\")",
    ] {
        assert!(
            compact_source.contains(needle),
            "workspace shell should use the app-facing Workbench owner; missing `{needle}`"
        );
    }

    for forbidden in [
        "ModelStore",
        "UiTree<App>",
        "FnDriver",
        "RenderRootContext",
        "UiFrameCx",
        "surface.driver()",
        "declarative::render_root(",
        ".propagate_model_changes(",
        ".propagate_global_changes(",
        ".layout_all(",
        ".paint_all(",
        ".build_semantics(",
        ".apply_workspace_model_commands(",
        "WorkspaceShellDirtyClosePrompt",
        "WorkspaceShellDemoDirtyClosePolicy",
        "fnapply_dirty_close_resolution(",
        "CMD_WORKSPACE_SHELL_DEMO_",
        "CommandId::new(",
        "command.as_str()",
    ] {
        assert!(
            !source.contains(forbidden),
            "workspace shell should not retain superseded transaction ownership `{forbidden}`"
        );
    }
}
