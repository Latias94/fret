#[test]
fn imui_shadcn_adapter_demo_owns_resizable_table_width_state() {
    let source = include_str!("../src/imui_shadcn_adapter_demo.rs");
    let compact_source = source.split_whitespace().collect::<String>();

    for needle in [
        "struct InspectorColumnWidths",
        "let inspector_widths_state = cx.state().local_init(InspectorColumnWidths::default);",
        "let inspector_widths = inspector_widths_state.layout_value(cx);",
        ".resizable_with_limits(Some(Px(88.0)), Some(Px(180.0)))",
        ".resizable_with_limits(Some(Px(96.0)), Some(Px(220.0)))",
        ".resizable_with_limits(Some(Px(64.0)), Some(Px(140.0)))",
        "fn apply_inspector_width_delta(",
        "let resize = header.resize();",
        "resize.drag_delta_x()",
        "resize.dragging()",
        "tx.update(widths_state, |widths|",
        "clamped_width_delta(*width, delta_x, min_width, max_width)",
        "const TEST_ID_TABLE_WIDTHS: &str = \"imui-shadcn-demo.inspector.widths\";",
    ] {
        assert!(
            source.contains(needle),
            "imui_shadcn_adapter_demo should keep the app-owned resize proof marker `{needle}`"
        );
    }

    for needle in [
        "app::AppLocalStateTxnExt as _",
        "local_state_txn(|tx| tx.update(&count_state, |v| *v += 1))",
        "switch_model_with_options(\"Enabled (switch)\",&enabled_state,",
        "slider_f32_model_with_options(\"Value\",&value_state,",
        "combo_model_with_options(\"imui-shadcn-demo.mode.popup\",\"Mode\",&mode_state,",
        "input_text_model_with_options(&draft_state,",
    ] {
        assert!(
            compact_source.contains(&needle.split_whitespace().collect::<String>()),
            "imui_shadcn_adapter_demo should stay on the LocalState-first IMUI surface; missing `{needle}`"
        );
    }

    for forbidden in [
        "fret::advanced::raw",
        "LocalStateModelStoreExt",
        "LocalStateRawModelExt",
        ".update_in(",
        ".model()",
    ] {
        assert!(
            !source.contains(forbidden),
            "imui_shadcn_adapter_demo should not reopen the raw LocalState bridge: `{forbidden}`"
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
