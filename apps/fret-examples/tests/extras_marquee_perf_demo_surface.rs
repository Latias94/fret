fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn extras_marquee_perf_demo_keeps_title_on_chrome_role() {
    let source = include_str!("../src/extras_marquee_perf_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret::advanced::text;",
        "fnmarquee_perf_title_text(cx:&mutElementContext<'_,KernelApp>,",
        "text::section_chrome_label(cx,text)",
        "marquee_perf_title_text(cx,\"Marqueeperfprobe(extras)\")",
    ] {
        assert!(
            compact_source.contains(needle),
            "marquee perf demo title should use shared chrome text role; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "ui::text(\"Marqueeperfprobe(extras)\")",
        ".font_semibold()",
        ".text_sm()",
    ] {
        assert!(
            !compact_source.contains(needle),
            "marquee perf demo should not render fixed title with local text policy; unexpected `{needle}`"
        );
    }
}
