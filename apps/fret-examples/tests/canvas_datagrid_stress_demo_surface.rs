fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn canvas_datagrid_stress_demo_keeps_header_text_on_readout_role() {
    let source = include_str!("../src/canvas_datagrid_stress_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fncanvas_datagrid_stress_readout_text<H:fret_ui::UiHost>(",
        "decl_text::text_control_readout(cx,text)",
        "canvas_datagrid_stress_readout_text(cx,header)",
    ] {
        assert!(
            source.contains(needle),
            "canvas datagrid stress demo should keep compact header text on the shared readout role; missing `{needle}`"
        );
    }

    for needle in ["cx.text(header)"] {
        assert!(
            !source.contains(needle),
            "canvas datagrid stress demo should not render the fixed header readout with bare wrapping text; unexpected `{needle}`"
        );
    }
}
