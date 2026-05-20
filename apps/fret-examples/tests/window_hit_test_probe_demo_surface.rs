fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn window_hit_test_probe_demo_keeps_fixed_text_on_roles() {
    let source = include_str!("../src/window_hit_test_probe_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnwindow_hit_test_title_text<H:UiHost>(",
        "fnwindow_hit_test_readout_text<H:UiHost>(",
        "fnwindow_hit_test_code_label_text<H:UiHost>(",
        "decl_text::text_section_chrome_label(cx,text)",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_code_label(cx,text)",
        "window_hit_test_title_text(cx,\"Hit-testpassthroughprobe\",)",
        "window_hit_test_code_label_text(cx,format!(\"logical_window_id={logical}\"),)",
        "window_hit_test_readout_text(cx,status)",
    ] {
        assert!(
            source.contains(needle),
            "window hit-test probe should keep fixed chrome/readout text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(\"Hit-testpassthroughprobe\").font_semibold().text_sm()",
        "ui::text(format!(\"logical_window_id={logical}\")).font_monospace().text_sm()",
        "ui::text(status).text_sm()",
    ] {
        assert!(
            !source.contains(needle),
            "window hit-test probe should not render fixed chrome/readouts with local text policy; unexpected `{needle}`"
        );
    }
}
