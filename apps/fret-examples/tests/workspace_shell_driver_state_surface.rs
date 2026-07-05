fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn workspace_shell_driver_model_writes_stay_behind_owner_helpers() {
    let source = include_str!("../src/workspace_shell_demo/driver.rs");
    let compact_source = compact(source);

    for needle in [
        "fnworkspace_shell_update_model<T:Any,R>(",
        "fnworkspace_shell_host_update_model<T:Any>(",
        "fnworkspace_shell_set_model<T:Any>(",
        "fnworkspace_shell_host_set_model<T:Any>(",
        "fnworkspace_shell_update_window_layout<R>(",
        "fnworkspace_shell_open_dirty_close_prompt(",
        "fnworkspace_shell_clear_dirty_close_prompt(",
        "fnworkspace_shell_host_clear_dirty_close_prompt(",
        "workspace_shell_host_clear_dirty_close_prompt(host,&prompt_model,&open_model,);",
        "workspace_shell_open_dirty_close_prompt(app,state,WorkspaceShellDirtyClosePrompt::window_close(req),);",
        "workspace_shell_clear_dirty_close_prompt(app,state);",
    ] {
        assert!(
            compact_source.contains(needle),
            "workspace shell driver should keep shared-model writes behind explicit owner helpers; missing `{needle}`"
        );
    }

    assert_eq!(
        source.matches("models_mut().update(").count(),
        2,
        "workspace shell driver should not scatter raw ModelStore updates outside owner helpers"
    );
}
