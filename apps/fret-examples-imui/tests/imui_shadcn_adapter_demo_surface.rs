#[test]
fn imui_shadcn_adapter_demo_owns_resizable_table_width_state() {
    let source = include_str!("../src/imui_shadcn_adapter_demo.rs");

    for needle in [
        "struct InspectorColumnWidths",
        "let inspector_widths_state = cx.state().local_init(InspectorColumnWidths::default);",
        "let inspector_widths = inspector_widths_state.layout_value(cx);",
        ".resizable_with_limits(Some(Px(88.0)), Some(Px(180.0)))",
        ".resizable_with_limits(Some(Px(96.0)), Some(Px(220.0)))",
        ".resizable_with_limits(Some(Px(64.0)), Some(Px(140.0)))",
        "fn apply_inspector_width_delta(",
        "header.resize.drag_delta_x()",
        "header.resize.dragging()",
        "widths_state.update_in(ui.cx_mut().app.models_mut(), |widths|",
        "clamped_width_delta(*width, delta_x, min_width, max_width)",
        "const TEST_ID_TABLE_WIDTHS: &str = \"imui-shadcn-demo.inspector.widths\";",
    ] {
        assert!(
            source.contains(needle),
            "imui_shadcn_adapter_demo should keep the app-owned resize proof marker `{needle}`"
        );
    }

    assert!(
        source.contains("\"Field###inspector-field\"")
            && source.contains("\"Value###inspector-value\"")
            && source.contains("\"Source###inspector-source\""),
        "regular inspector table should expose stable column ids for resize diagnostics"
    );
    assert!(
        source.contains("\"Signal###inspector-signal\"")
            && source.contains("\"State###inspector-state\""),
        "compact inspector table should expose stable column ids for resize diagnostics"
    );
}
