fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn workspace_shell_driver_model_writes_stay_behind_owner_helpers() {
    let source = include_str!("../src/workspace_shell_demo/driver.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("workspace shell driver should have production source before tests");
    let compact_source = compact(source);
    let compact_production = compact(production_source);

    for needle in [
        "structWorkspaceShellModelOwner<'a>{",
        "models:&'amutModelStore,",
        "fnupdate<T:Any,R>(&mutself,model:&Model<T>,f:implFnOnce(&mutT)->R)->Option<R>{",
        "fnset<T:Any>(&mutself,model:&Model<T>,value:T)->bool{",
        "fnupdate_window_layout<R>(",
        "fnopen_dirty_close_prompt(",
        "fnclear_dirty_close_prompt(",
        "fntoggle_tabstrip_two_row_pinned(&mutself,model:&Model<bool>)->bool{",
        "fnworkspace_shell_update_window_layout<R>(",
        "fnworkspace_shell_open_dirty_close_prompt(",
        "fnworkspace_shell_clear_dirty_close_prompt(",
        "fnworkspace_shell_host_clear_dirty_close_prompt(",
        "workspace_shell_host_clear_dirty_close_prompt(host,&prompt_model,&open_model,);",
        "workspace_shell_open_dirty_close_prompt(app,state,WorkspaceShellDirtyClosePrompt::window_close(req),);",
        "workspace_shell_clear_dirty_close_prompt(app,state);",
        "WorkspaceShellModelOwner::new(app.models_mut()).toggle_tabstrip_two_row_pinned(&state.tabstrip_two_row_pinned);",
    ] {
        assert!(
            compact_source.contains(needle),
            "workspace shell driver should keep shared-model writes behind a named owner helper; missing `{needle}`"
        );
    }

    for forbidden in [
        "models_mut().update(",
        "models_mut().update::<",
        "models_mut().update_any(",
        "models_mut().update_any::<",
        "ModelStore::update(",
        "ModelStore::update::<",
        "ModelStore::update_any(",
        "ModelStore::update_any::<",
        "<ModelStore>::update(",
        "<ModelStore>::update::<",
        "<ModelStore>::update_any(",
        "<ModelStore>::update_any::<",
        "fnworkspace_shell_update_model",
        "fnworkspace_shell_host_update_model",
        "fnworkspace_shell_set_model",
        "fnworkspace_shell_host_set_model",
    ] {
        assert!(
            !compact_production.contains(forbidden),
            "workspace shell driver production code should not bypass the owner helper with `{forbidden}`"
        );
    }
}
