fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn container_queries_docking_demo_keeps_fixed_panel_text_on_roles() {
    let source = include_str!("../src/container_queries_docking_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::{AppRenderContext,text};",
        "fncontainer_query_docking_readout_text<'a,Cx>(",
        "fncontainer_query_docking_placeholder_text<'a,Cx>(",
        "Cx:AppRenderContext<'a>,",
        "text::control_readout(cx,text)",
        "text::button_label(cx,text)",
        "fnrender_left_panel(cx:&mutElementContext<'_,App>,theme:&Theme)->Vec<AnyElement>{",
        "container_query_docking_readout_text(cx,Arc::clone(&mode_text),)",
        "container_query_docking_placeholder_text(cx,\"Inputstub\")",
        "container_query_docking_readout_text(cx,\"Unregisteredpanelkind\",)",
    ] {
        assert!(
            source.contains(needle),
            "container queries docking demo should keep fixed panel text on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(Arc::clone(&mode_text))",
        "cx.text(\"Inputstub\")",
        "cx.text(\"Unregisteredpanelkind\")",
        "fret_ui_kit::declarative::text::text_control_readout(",
        "fret_ui_kit::declarative::text::text_button_label(",
    ] {
        assert!(
            !source.contains(needle),
            "container queries docking demo should not render fixed panel text with bare wrapping text; unexpected `{needle}`"
        );
    }
}
