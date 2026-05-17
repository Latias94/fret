fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn container_queries_docking_demo_keeps_fixed_panel_text_on_roles() {
    let source = include_str!("../src/container_queries_docking_demo.rs");
    let source = compact(source);

    for needle in [
        "fncontainer_query_docking_readout_text<H:fret_ui::UiHost>(",
        "fncontainer_query_docking_placeholder_text<H:fret_ui::UiHost>(",
        "fret_ui_kit::declarative::text::text_control_readout(cx,text)",
        "fret_ui_kit::declarative::text::text_button_label(cx,text)",
        "container_query_docking_readout_text(cx,Arc::clone(&mode_text),)",
        "container_query_docking_placeholder_text(cx,\"Inputstub\")",
        "container_query_docking_readout_text(cx,\"Unregisteredpanelkind\",)",
    ] {
        assert!(
            source.contains(needle),
            "container queries docking demo should keep fixed panel text on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(Arc::clone(&mode_text))",
        "cx.text(\"Inputstub\")",
        "cx.text(\"Unregisteredpanelkind\")",
    ] {
        assert!(
            !source.contains(needle),
            "container queries docking demo should not render fixed panel text with bare wrapping text; unexpected `{needle}`"
        );
    }
}
