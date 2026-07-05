fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn simple_todo_demo_keeps_visible_text_on_roles() {
    let source = include_str!("../src/simple_todo_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "fnsimple_todo_readout_text",
        "fnsimple_todo_compact_paragraph_text",
        "fnsimple_todo_row_label_text",
        "Cx:AppRenderContext<'a>",
        "text::control_readout(cx,text)",
        "text::compact_paragraph(cx,text)",
        "text::list_row_label_with_foreground(cx,text,foreground)",
        "letsummary=simple_todo_readout_text(cx,status_text);",
        "letempty_text=simple_todo_compact_paragraph_text(",
        "\"Notasksyet.Addoneabove.\",",
        "letremaining=simple_todo_readout_text(cx,format!(\"{active_count}left\"));",
        "lettext=simple_todo_row_label_text(cx,row_text.clone(),ColorRef::Color(row_text_foreground));",
    ] {
        assert!(
            compact_source.contains(needle),
            "simple todo visible text should use shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(status_text)",
        "ui::text(\"Notasksyet.Addoneabove.\")",
        "ui::text(format!(\"{active_count}left\"))",
        "ui::text(row.text.clone())",
        "ui::text(",
        "cx.elements()",
        "usefret_ui_kit::declarative::textasdecl_text;",
    ] {
        assert!(
            !compact_source.contains(needle),
            "simple todo should not render app text through local/raw text policy; unexpected `{needle}`"
        );
    }
}
