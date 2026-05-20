fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn simple_todo_demo_keeps_visible_text_on_roles() {
    let source = include_str!("../src/simple_todo_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::{ElementContextThemeExtas_,textasdecl_text};",
        "fnsimple_todo_readout_text<H:UiHost>(",
        "fnsimple_todo_compact_paragraph_text<H:UiHost>(",
        "fnsimple_todo_row_label_text<H:UiHost>(",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_compact_paragraph(cx,text)",
        "decl_text::text_list_row_label(cx,text).inherit_foreground(foreground)",
        "letsummary=simple_todo_readout_text(cx.elements(),status_text);",
        "letempty_text=simple_todo_compact_paragraph_text(",
        "\"Notasksyet.Addoneabove.\",",
        "letremaining=simple_todo_readout_text(cx,format!(\"{active_count}left\"));",
        "lettext=simple_todo_row_label_text(cx,row_text.clone(),row_text_foreground);",
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
    ] {
        assert!(
            !compact_source.contains(needle),
            "simple todo should not render app text with local ui::text policy; unexpected `{needle}`"
        );
    }
}
