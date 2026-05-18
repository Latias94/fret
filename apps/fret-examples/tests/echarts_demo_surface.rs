fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn echarts_demo_chart_titles_use_section_chrome_role() {
    let source = include_str!("../src/echarts_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::text_section_chrome_label(",
        "std::sync::Arc::clone(&chart.title)",
    ] {
        assert!(
            source.contains(needle),
            "echarts demo chart titles should stay on the shared section chrome text role; missing `{needle}`"
        );
    }

    assert!(
        !source.contains("cx.text(std::sync::Arc::clone(&chart.title))"),
        "echarts demo chart titles should not use bare wrapping text"
    );
}
