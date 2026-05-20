fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn async_playground_demo_keeps_visible_text_on_roles() {
    let source = include_str!("../src/async_playground_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnasync_chrome_title_text<H:UiHost>(",
        "fnasync_section_text<H:UiHost>(",
        "fnasync_list_row_text<H:UiHost>(",
        "fnasync_readout_text<H:UiHost>(",
        "fnasync_code_label_text<H:UiHost>(",
        "fnasync_compact_paragraph_text<H:UiHost>(",
        "decl_text::text_chrome_title(cx,text)",
        "decl_text::text_section_chrome_label(cx,text)",
        "decl_text::text_list_row_label(cx,text)",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_code_label(cx,text)",
        "decl_text::text_compact_paragraph(cx,text)",
        "lettitle=async_chrome_title_text(cx.elements(),\"AsyncPlayground\");",
        "letslow_label=async_readout_text(cx.elements(),\"Slownetwork(x2)\");",
        "letheader=async_section_text(cx.elements(),\"Catalog\");",
        "lettitle=async_list_row_text(cx,id.label());",
        "lettitle=async_section_text(cx.elements(),selected.label());",
        "out.push(async_code_label_text(cx,key.namespace()));",
        "out.push(async_readout_text(cx,format!(\"status:{status:?}\")));",
        "letpolicy_editor=policy_editor(cx,st,selected);",
        "letkeep_prev_label=async_readout_text(cx.elements(),\"keepPreviousDataWhileLoading\");",
        "letfail_label=async_readout_text(cx.elements(),\"failmode\");",
        "async_section_text(cx,\"Policy\")",
        "letinputs=query_inputs_row(cx,locals,id);",
        "letview=query_result_view(cx,id,key,&state,snap.as_ref(),&policy);",
        "children.push(async_compact_paragraph_text(",
        "letleft=async_code_label_text(cx,id.namespace());",
        "QueryStatus::Idle=>async_compact_paragraph_text(cx.elements(),\"Idle(notfetchedyet).\"),",
        "QueryStatus::Success=>async_compact_paragraph_text(",
        "lettitle=async_section_text(cx,\"Result\");",
    ] {
        assert!(
            compact_source.contains(needle),
            "async playground visible text should use shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(",
        "ui::rich_text(",
        ".font_semibold()",
        ".font_medium()",
        ".truncate()",
        "text_color(ColorRef::Color(theme.color_token(\"muted-foreground\")))",
        "policy_editor(cx,st,theme.clone(),selected)",
        "query_inputs_row(cx,locals,theme.clone(),id)",
        "query_result_view(cx,theme,id,",
    ] {
        assert!(
            !compact_source.contains(needle),
            "async playground should not render app text with local text policy; unexpected `{needle}`"
        );
    }
}
