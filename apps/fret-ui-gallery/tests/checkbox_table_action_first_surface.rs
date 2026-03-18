#[test]
fn checkbox_table_snippet_keeps_action_first_select_all_surface() {
    let source = include_str!("../src/ui/snippets/checkbox/table.rs");

    assert!(
        source.contains("cx.actions().models::<act::ToggleAllRows>"),
        "checkbox table snippet should register select-all through the action-first models surface"
    );
    assert!(
        source.contains(".action(act::ToggleAllRows)"),
        "checkbox table snippet should bind the header checkbox through `.action(...)`"
    );
    assert!(
        !source.contains("command_on_command_for("),
        "checkbox table snippet should not teach a root command handler for select-all"
    );
    assert!(
        !source.contains(".on_click(CommandId::new("),
        "checkbox table snippet should not fall back to a command-id click handler for select-all"
    );
}
