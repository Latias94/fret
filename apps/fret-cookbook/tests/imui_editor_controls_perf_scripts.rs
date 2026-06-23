fn step_index(script: &str, needle: &str) -> usize {
    script
        .find(needle)
        .unwrap_or_else(|| panic!("missing script fragment: {needle}"))
}

#[test]
fn editor_controls_click_stress_resets_after_initial_root_settle() {
    let script = include_str!(
        "../../../tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-click-stress.json"
    );

    let resize = step_index(script, r#""type": "set_window_inner_size""#);
    let root_wait = step_index(script, r#""id": "cookbook.imui_editor_controls.root""#);
    let reset = step_index(script, r#""type": "reset_diagnostics""#);
    let first_click = step_index(script, r#""id": "cookbook.imui_editor_controls.assist""#);
    let capture = step_index(
        script,
        r#""label": "cookbook-imui-editor-controls-click-stress""#,
    );

    assert!(
        resize < root_wait && root_wait < reset,
        "click-stress should size and settle the cookbook root before resetting diagnostics"
    );
    assert!(
        reset < first_click && first_click < capture,
        "click-stress should measure the interaction window after reset"
    );
    assert!(
        script.contains("startup resize/header measurement"),
        "click-stress should document why reset_diagnostics is not the first step"
    );
}
