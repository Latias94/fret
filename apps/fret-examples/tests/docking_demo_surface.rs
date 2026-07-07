fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn docking_demo_keeps_panel_text_on_roles() {
    let source = include_str!("../src/docking_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::{AppRenderContext,text};",
        "fndocking_demo_list_row_text<'a,Cx>(",
        "fndocking_demo_readout_text<'a,Cx>(",
        "Cx:AppRenderContext<'a>,",
        "text::list_row_label(cx,text)",
        "text::control_readout(cx,text)",
        "docking_demo_list_row_text(cx,\"Scene\")",
        "docking_demo_list_row_text(cx,\"Camera\")",
        "docking_demo_list_row_text(cx,\"DirectionalLight\")",
        "docking_demo_list_row_text(cx,\"Player\")",
        "docking_demo_readout_text(cx,\"Name:Player\")",
        "docking_demo_readout_text(cx,\"Position:(12.0,3.0,-8.0)\")",
        "docking_demo_readout_text(cx,\"Rotation:(0.0,90.0,0.0)\")",
    ] {
        assert!(
            source.contains(needle),
            "docking demo should keep fixed panel text on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(\"Scene\")",
        "cx.text(\"•Camera\")",
        "cx.text(\"•DirectionalLight\")",
        "cx.text(\"•Player\")",
        "cx.text(\"Name:Player\")",
        "cx.text(\"Position:(12.0,3.0,-8.0)\")",
        "cx.text(\"Rotation:(0.0,90.0,0.0)\")",
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "text_list_row_label(",
        "text_control_readout(",
    ] {
        assert!(
            !source.contains(needle),
            "docking demo should not render panel chrome/readouts with bare wrapping text; unexpected `{needle}`"
        );
    }
}
