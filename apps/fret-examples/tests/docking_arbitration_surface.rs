fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn docking_arbitration_demo_keeps_body_and_state_text_on_roles() {
    let source = include_str!("../src/docking_arbitration_demo.rs");
    let source = compact(source);

    for needle in [
        "fndocking_arbitration_readout_text<H:fret_ui::UiHost>(",
        "fndocking_arbitration_paragraph_text<H:fret_ui::UiHost>(",
        "fret_ui_kit::declarative::text::text_control_readout(cx,text)",
        "fret_ui_kit::declarative::text::text_paragraph(cx,text)",
        "docking_arbitration_paragraph_text(cx,\"Non-modaloverlay(Popover).\",)",
        ".map(|v|docking_arbitration_readout_text(cx,v))",
        "docking_arbitration_readout_text(cx,ifpopover_is_open{\"Popover:open\"}else{\"Popover:closed\"},)",
        "docking_arbitration_readout_text(cx,ifdialog_is_open{\"Dialog:open\"}else{\"Dialog:closed\"},)",
        "docking_arbitration_readout_text(cx,ifdrop_mask_left_disallowed{\"Dropmask:leftedgedockingdisallowed\"}else{\"Dropmask:leftedgedockingallowed\"},)",
    ] {
        assert!(
            source.contains(needle),
            "docking arbitration demo should keep fixed state readouts on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "vec![cx.text(ifpopover_is_open",
        "vec![cx.text(ifdialog_is_open",
        "vec![cx.text(ifdrop_mask_left_disallowed",
        "cx.text(\"Non-modaloverlay(Popover).\")",
        ".map(|v|cx.text(v))",
    ] {
        assert!(
            !source.contains(needle),
            "docking arbitration demo should not render state readouts with bare wrapping text; unexpected `{needle}`"
        );
    }
}
