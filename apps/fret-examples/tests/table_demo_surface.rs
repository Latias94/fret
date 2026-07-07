fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn table_demo_keeps_fixed_table_text_on_roles() {
    let source = include_str!("../src/table_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::text;",
        "text::control_readout(cx,header)",
        "|cx|text::table_cell(cx,label.clone())",
        "vec![text::table_cell(cx,text)]",
    ] {
        assert!(
            source.contains(needle),
            "table demo should keep fixed readout/header/cell text on app text facade roles; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "text_control_readout(",
        "text_table_cell(",
        "cx.text(header)",
        "|cx|cx.text(label.clone())",
        "vec![cx.text(text)]",
    ] {
        assert!(
            !source.contains(needle),
            "table demo should not render fixed table text with bare wrapping text; unexpected `{needle}`"
        );
    }
}
