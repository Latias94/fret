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

#[test]
fn checkbox_table_snippet_keeps_fixed_cell_text_on_table_role() {
    let source = include_str!("../src/ui/snippets/checkbox/table.rs");

    for needle in [
        "fn table_cell_text<H: UiHost>(",
        "fret_ui_kit::declarative::text::text_table_cell(cx, text)",
        "shadcn::table_cell(table_cell_text(cx, id))",
        "shadcn::table_cell(table_cell_text(cx, role))",
    ] {
        assert!(
            source.contains(needle),
            "checkbox table snippet should route fixed cell text through the shared table-cell role; missing `{needle}`"
        );
    }

    for forbidden in [
        "shadcn::table_cell(ui::text(id))",
        "shadcn::table_cell(ui::text(role))",
    ] {
        assert!(
            !source.contains(forbidden),
            "checkbox table snippet reintroduced bare fixed table-cell text: `{forbidden}`"
        );
    }
}
