fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn form_demo_header_status_uses_control_readout_role() {
    let source = include_str!("../src/form_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::text_control_readout(cx,Arc::from(format!(\"submit_count={submit_count}valid={valid}dirty={dirty}status={}\",status_text.as_ref())),)",
    ] {
        assert!(
            source.contains(needle),
            "form demo fixed header status should stay on the shared control-readout text role; missing `{needle}`"
        );
    }

    assert!(
        !source.contains(
            "cx.text(Arc::from(format!(\"submit_count={submit_count}valid={valid}dirty={dirty}status={}\",status_text.as_ref())))"
        ),
        "form demo fixed header status should not use bare wrapping text"
    );
}
