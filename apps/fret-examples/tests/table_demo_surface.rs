fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn table_demo_keeps_fixed_table_text_on_roles() {
    let source = include_str!("../src/table_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fntable_demo_readout_text<H:fret_ui::UiHost>(",
        "fntable_demo_cell_text<H:fret_ui::UiHost>(",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_table_cell(cx,text)",
        "table_demo_readout_text(cx,header)",
        "|cx|table_demo_cell_text(cx,label.clone())",
        "vec![table_demo_cell_text(cx,text)]",
    ] {
        assert!(
            source.contains(needle),
            "table demo should keep fixed readout/header/cell text on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
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
