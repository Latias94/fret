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

#[test]
fn components_gallery_chrome_and_controls_use_text_roles() {
    let source = include_str!("../src/components_gallery.rs");
    let source = compact(source);

    for needle in [
        "decl_text::text_chrome_title(cx,title)",
        "decl_text::text_control_readout(cx,subtitle)",
        "decl_text::text_control_label(cx,Arc::<str>::from(\"Theme:\"),)",
        "decl_text::text_control_readout(cx,Arc::<str>::from(format!(\"Themeconfig:{}\",theme_name)),)",
        "decl_text::text_control_label(cx,label,)",
        "decl_text::text_control_readout(cx,format!(\"checkbox:{checkbox_value}\"),)",
        "decl_text::text_control_readout(cx,format!(\"switch:{switch_value}\"),)",
        "decl_text::text_control_readout(cx,format!(\"radio:{radio_label}\"),)",
        "decl_text::text_control_readout(cx,format!(\"select:{select_label}\"),)",
    ] {
        assert!(
            source.contains(needle),
            "components gallery chrome/control text should stay on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(title)",
        "cx.text(subtitle)",
        "cx.text(Arc::<str>::from(\"Theme:\"))",
        "cx.text(Arc::<str>::from(format!(\"Themeconfig:{}\",theme_name)))",
        "cx.text(label)",
        "cx.text(format!(\"checkbox:{checkbox_value}\"))",
        "cx.text(format!(\"switch:{switch_value}\"))",
        "cx.text(format!(\"radio:{radio_label}\"))",
        "cx.text(format!(\"select:{select_label}\"))",
    ] {
        assert!(
            !source.contains(needle),
            "components gallery fixed chrome/control text should not use bare text; unexpected `{needle}`"
        );
    }
}
