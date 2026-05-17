fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn sonner_demo_header_text_uses_fixed_chrome_roles() {
    let source = include_str!("../src/sonner_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::text_section_chrome_label(",
        "\"Sonner(shadcn/ui)demo\",",
        "decl_text::text_control_readout(cx,format!(\"promiseactive:{promise_active}|lastaction:{last_action_value}\"))",
    ] {
        assert!(
            source.contains(needle),
            "sonner demo fixed header text should stay on shared chrome/readout roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(\"Sonner(shadcn/ui)demo\")",
        "cx.text(format!(\"promiseactive:{promise_active}|lastaction:{last_action_value}\"))",
    ] {
        assert!(
            !source.contains(needle),
            "sonner demo fixed header text should not use bare wrapping text; unexpected `{needle}`"
        );
    }
}
