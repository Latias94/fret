fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn hello_counter_demo_keeps_status_and_help_text_on_roles() {
    let source = include_str!("../src/hello_counter_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnhello_counter_status_text(cx:&mutAppUi<'_,'_>,text:implInto<Arc<str>>)->AnyElement",
        "fnhello_counter_paragraph_text(cx:&mutAppUi<'_,'_>,text:implInto<Arc<str>>)->AnyElement",
        "decl_text::text_control_readout(cx.elements(),text)",
        "decl_text::text_paragraph(cx.elements(),text)",
        "letstatus_line=hello_counter_status_text(cx,status_text);",
        "letstep_help=hello_counter_paragraph_text(cx,ifstep_valid{",
        "ui::text(count.to_string()).text_size_px(Px(72.0))",
    ] {
        assert!(
            source.contains(needle),
            "hello counter should keep resize-sensitive status/help text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(status_text).text_color(",
        "ui::text_block(ifstep_valid{",
    ] {
        assert!(
            !source.contains(needle),
            "hello counter should not render status/help text with local text policy; unexpected `{needle}`"
        );
    }
}
