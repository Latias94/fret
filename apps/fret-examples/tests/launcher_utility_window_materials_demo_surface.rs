fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn launcher_utility_window_materials_demo_keeps_fixed_text_on_roles() {
    let source = include_str!("../src/launcher_utility_window_materials_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::advanced::text;",
        "fnlauncher_utility_materials_title_text<H:UiHost>(",
        "fnlauncher_utility_materials_readout_text<H:UiHost>(",
        "fnlauncher_utility_materials_code_label_text<H:UiHost>(",
        "text::section_chrome_label(cx,text)",
        "text::control_readout(cx,text)",
        "text::code_label(cx,text)",
        "launcher_utility_materials_title_text(cx,\"UtilityWindowMaterials(Mica/Acrylic)\",)",
        "launcher_utility_materials_code_label_text(cx,style_text).test_id(TEST_ID_STYLE_TEXT)",
        "launcher_utility_materials_readout_text(cx,status)",
    ] {
        assert!(
            source.contains(needle),
            "launcher utility window materials demo should keep fixed chrome/readout text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "ui::text(\"UtilityWindowMaterials(Mica/Acrylic)\").font_semibold()",
        "ui::text(style_text).font_monospace().text_sm()",
        "ui::text(status).text_sm()",
    ] {
        assert!(
            !source.contains(needle),
            "launcher utility window materials demo should not render fixed chrome/readouts with local text policy; unexpected `{needle}`"
        );
    }
}
