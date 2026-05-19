fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

fn assert_query_text_roles(source: &str, label: &str) {
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnquery_readout_text<H:UiHost>(",
        "fnquery_readout_text_with_color<H:UiHost>(",
        "fnquery_data_text<H:UiHost>(",
        "decl_text::text_control_readout(cx,text)",
        "query_readout_text(cx,text).inherit_foreground(foreground)",
        "decl_text::text_code_label(cx,text)",
        "query_readout_text(cx,info_line)",
        "query_data_text(cx,data_line)",
        "query_readout_text_with_color(cx,error_line,error_color)",
    ] {
        assert!(
            source.contains(needle),
            "{label} should keep query detail text on shared roles; missing `{needle}`",
        );
    }

    for needle in [
        "ui::raw_text(",
        "ui::text_block(",
        ".text_color(ColorRef::Color(error_color))",
        ".text_color(ColorRef::Color(theme.color_token(\"muted-foreground\")))",
    ] {
        assert!(
            !source.contains(needle),
            "{label} should not render query detail text with local raw text policy; unexpected `{needle}`",
        );
    }
}

#[test]
fn query_demos_keep_detail_text_on_roles() {
    assert_query_text_roles(include_str!("../src/query_demo.rs"), "query_demo");
    assert_query_text_roles(
        include_str!("../src/query_async_tokio_demo.rs"),
        "query_async_tokio_demo",
    );
}
