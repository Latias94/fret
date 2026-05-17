fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn components_gallery_table_torture_uses_text_roles() {
    let source = include_str!("../src/components_gallery.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "\"id\"=>decl_text::text_table_cell(cx,row.to_string())",
        "\"status\"=>decl_text::text_table_cell(cx,ifrow%3==0",
        "\"cpu\"=>decl_text::text_table_cell(cx,format!(\"{}%\",(row*7)%100))",
        "\"mem_mb\"=>decl_text::text_table_cell(cx,format!(\"{}MB\",128+(row%4096)))",
        "_=>decl_text::text_table_cell(cx,\"?\")",
        "letheader=decl_text::text_paragraph(cx,header);",
    ] {
        assert!(
            source.contains(needle),
            "components gallery table torture should keep retained table text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "\"id\"=>cx.text(row.to_string())",
        "\"status\"=>cx.text(ifrow%3==0",
        "\"cpu\"=>cx.text(format!(\"{}%\",(row*7)%100))",
        "\"mem_mb\"=>cx.text(format!(\"{}MB\",128+(row%4096)))",
        "_=>cx.text(\"?\")",
        "letheader=cx.text(header);",
    ] {
        assert!(
            !source.contains(needle),
            "components gallery table torture should not use bare text for retained table cells/header prose; unexpected `{needle}`"
        );
    }
}
