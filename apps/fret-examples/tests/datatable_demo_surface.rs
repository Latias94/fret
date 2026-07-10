fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn datatable_demo_keeps_fixed_table_text_on_roles() {
    let source = include_str!("../src/datatable_demo.rs");
    let source = compact(source);

    for needle in [
        "text::control_readout(cx,Arc::from(format!(\"DataTable|selected={selected}sort={sorting}\")),)",
        "\"id\"=>text::table_cell(cx,Arc::from(row.id.to_string()))",
        "\"name\"=>text::table_cell(cx,Arc::clone(&row.name))",
        "\"role\"=>text::table_cell(cx,Arc::clone(&row.role))",
        "\"score\"=>text::table_cell(cx,Arc::from(row.score.to_string()))",
        "_=>text::table_cell(cx,Arc::from(\"\"))",
    ] {
        assert!(
            source.contains(needle),
            "datatable demo should keep fixed readout/cell text on app text facade roles; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "text_control_readout(",
        "text_table_cell(",
        "cx.text(Arc::from(format!(\"DataTable|selected={selected}sort={sorting}\")))",
        "\"id\"=>cx.text(Arc::from(row.id.to_string()))",
        "\"name\"=>cx.text(Arc::clone(&row.name))",
        "\"role\"=>cx.text(Arc::clone(&row.role))",
        "\"score\"=>cx.text(Arc::from(row.score.to_string()))",
        "_=>cx.text(Arc::from(\"\"))",
    ] {
        assert!(
            !source.contains(needle),
            "datatable demo should not render fixed table text with bare wrapping text; unexpected `{needle}`"
        );
    }
}
