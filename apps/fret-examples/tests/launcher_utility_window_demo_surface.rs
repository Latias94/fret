fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn launcher_utility_window_demo_keeps_fixed_text_on_roles() {
    let source = include_str!("../src/launcher_utility_window_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnlauncher_utility_title_text<H:UiHost>(",
        "fnlauncher_utility_readout_text<H:UiHost>(",
        "fnlauncher_utility_code_label_text<H:UiHost>(",
        "fnlauncher_utility_glyph_text<H:UiHost>(",
        "decl_text::text_section_chrome_label(cx,text)",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_code_label(cx,text)",
        "decl_text::text_chrome_glyph(cx,text)",
        "launcher_utility_title_text(cx,\"LauncherUtilityWindow(draghere)\",)",
        "launcher_utility_code_label_text(cx,style_text).test_id(TEST_ID_STYLE_TEXT)",
        "launcher_utility_readout_text(cx,view_settings.status)",
        "launcher_utility_glyph_text(cx,\"↘\")",
    ] {
        assert!(
            source.contains(needle),
            "launcher utility window demo should keep fixed chrome/readout text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(\"LauncherUtilityWindow(draghere)\").font_semibold()",
        "ui::text(style_text).font_monospace().text_sm()",
        "ui::text(view_settings.status).text_sm()",
        "ui::text(\"↘\").font_semibold()",
    ] {
        assert!(
            !source.contains(needle),
            "launcher utility window demo should not render fixed chrome/readouts with local text policy; unexpected `{needle}`"
        );
    }
}
