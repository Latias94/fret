fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn todo_demo_keeps_visible_text_on_roles() {
    let source = include_str!("../src/todo_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::{ElementContextThemeExtas_,textasdecl_text};",
        "fntodo_readout_text<H:UiHost>(",
        "fntodo_chrome_title_text<H:UiHost>(",
        "fntodo_compact_paragraph_text<H:UiHost>(",
        "fntodo_filter_label_text<H:UiHost>(",
        "fntodo_row_label_text<H:UiHost>(",
        "fntodo_attributed_row_label_text<H:UiHost>(",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_chrome_title(cx,text)",
        "decl_text::text_compact_paragraph(cx,text)",
        "decl_text::text_button_label(cx,text)",
        "decl_text::text_list_row_label(cx,text).inherit_foreground(foreground)",
        "decl_text::text_list_row_label_attributed(cx,rich).inherit_foreground(foreground)",
        "todo_readout_text(cx.elements(),\"Addatasktogetstarted\")",
        "letcompleted_text=todo_readout_text(cx,\"Alltaskscompleted\");",
        "todo_readout_text(cx.elements(),format!(\"{active_count}{task_label}left\"))",
        "lettitle=todo_chrome_title_text(cx,\"Mytasks\");",
        "letprogress_label=todo_readout_text(cx,\"Progress\");",
        "letprogress_value=todo_readout_text(cx,format!(\"{:.0}%\",progress_pct));",
        "letempty_text=todo_compact_paragraph_text(cx,empty_label);",
        "letlabel=todo_filter_label_text(cx.elements(),filter.label());",
        "todo_attributed_row_label_text(cx,rich,muted_foreground)",
        "todo_row_label_text(cx,row_text.clone(),foreground)",
    ] {
        assert!(
            compact_source.contains(needle),
            "todo demo visible text should use shared text roles; missing `{needle}`"
        );
    }

    for needle in ["ui::text(", "ui::rich_text(", "typography::"] {
        assert!(
            !compact_source.contains(needle),
            "todo demo should not render app text with local text policy; unexpected `{needle}`"
        );
    }
}
