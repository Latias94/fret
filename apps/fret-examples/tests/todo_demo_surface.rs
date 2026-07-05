fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn todo_demo_keeps_visible_text_on_roles() {
    let source = include_str!("../src/todo_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "fntodo_readout_text<'a,Cx>(cx:&mutCx,text:implInto<Arc<str>>)->AnyElement",
        "fntodo_chrome_title_text<'a,Cx>(cx:&mutCx,text:implInto<Arc<str>>)->AnyElement",
        "fntodo_compact_paragraph_text<'a,Cx>(cx:&mutCx,text:implInto<Arc<str>>)->AnyElement",
        "fntodo_filter_label_text<'a,Cx>(cx:&mutCx,text:implInto<Arc<str>>)->AnyElement",
        "fntodo_row_label_text<'a,Cx>(",
        "fntodo_attributed_row_label_text<'a,Cx>(",
        "Cx:fret::app::AppRenderContext<'a>,",
        "text::control_readout(cx,text)",
        "text::chrome_title(cx,text)",
        "text::compact_paragraph(cx,text)",
        "text::button_label(cx,text)",
        "text::list_row_label_with_foreground(cx,text,foreground)",
        "text::list_row_label_attributed_with_foreground(cx,rich,foreground)",
        "todo_readout_text(cx,\"Addatasktogetstarted\")",
        "letcompleted_text=todo_readout_text(cx,\"Alltaskscompleted\");",
        "todo_readout_text(cx,format!(\"{active_count}{task_label}left\"))",
        "lettitle=todo_chrome_title_text(cx,\"Mytasks\");",
        "letprogress_label=todo_readout_text(cx,\"Progress\");",
        "letprogress_value=todo_readout_text(cx,format!(\"{:.0}%\",progress_pct));",
        "letempty_text=todo_compact_paragraph_text(cx,empty_label);",
        "letlabel=todo_filter_label_text(cx,filter.label());",
        "todo_attributed_row_label_text(cx,rich,muted_foreground)",
        "todo_row_label_text(cx,row_text.clone(),foreground)",
    ] {
        assert!(
            compact_source.contains(needle),
            "todo demo visible text should use shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(",
        "ui::rich_text(",
        "typography::",
        "textasdecl_text",
        "todo_readout_text(cx.elements()",
        "todo_filter_label_text(cx.elements()",
    ] {
        assert!(
            !compact_source.contains(needle),
            "todo demo should not render app text with local text policy; unexpected `{needle}`"
        );
    }
}
