fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn date_picker_demo_keeps_fixed_chrome_text_on_roles() {
    let source = include_str!("../src/date_picker_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fndate_picker_readout_text<H:fret_ui::UiHost>(",
        "fndate_picker_control_label_text<H:fret_ui::UiHost>(",
        "fndate_picker_paragraph_text<H:fret_ui::UiHost>(",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_control_label(cx,text)",
        "decl_text::text_paragraph(cx,text)",
        "date_picker_readout_text(cx,Arc::from(format!(\"DatePicker|open={}selected={}month={}\"",
        "date_picker_control_label_text(cx,\"weekstartmonday\")",
        "date_picker_control_label_text(cx,\"showoutsidedays\")",
        "date_picker_control_label_text(cx,\"disableoutsidedays\")",
        "date_picker_control_label_text(cx,\"disableweekends\")",
        "date_picker_control_label_text(cx,\"disabled\")",
        "date_picker_paragraph_text(cx,Arc::from(\"Try:Tabtofocus",
    ] {
        assert!(
            source.contains(needle),
            "date picker demo should keep readouts, switch labels, and prose on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
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
