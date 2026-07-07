fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn date_picker_demo_keeps_fixed_chrome_text_on_roles() {
    let source = include_str!("../src/date_picker_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::text;",
        "text::control_readout(cx,Arc::from(format!(\"DatePicker|open={}selected={}month={}\"",
        "text::control_label(cx,\"weekstartmonday\")",
        "text::control_label(cx,\"showoutsidedays\")",
        "text::control_label(cx,\"disableoutsidedays\")",
        "text::control_label(cx,\"disableweekends\")",
        "text::control_label(cx,\"disabled\")",
        "text::paragraph(cx,Arc::from(\"Try:Tabtofocus",
    ] {
        assert!(
            source.contains(needle),
            "date picker demo should keep readouts, switch labels, and prose on app text facade roles; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "text_control_readout(",
        "text_control_label(",
        "text_paragraph(",
        "cx.text(Arc::from(format!(\"DatePicker|open={}selected={}month={}\"",
        "cx.text(\"weekstartmonday\")",
        "cx.text(\"showoutsidedays\")",
        "cx.text(\"disableoutsidedays\")",
        "cx.text(\"disableweekends\")",
        "cx.text(\"disabled\")",
        "cx.text(Arc::from(\"Try:Tabtofocus",
    ] {
        assert!(
            !source.contains(needle),
            "date picker demo should not render fixed chrome/prose examples with bare text; unexpected `{needle}`"
        );
    }
}
