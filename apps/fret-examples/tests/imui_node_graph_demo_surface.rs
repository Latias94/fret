#[test]
fn imui_node_graph_demo_keeps_compat_title_on_shared_role() {
    let source = include_str!("../src/imui_node_graph_demo.rs");

    for needle in [
        "Retained-bridge IMUI demo for `fret-node`.",
        "compatibility-oriented and should not be treated as the default downstream",
        "Prefer the declarative node-graph surfaces for normal downstream guidance.",
        "use fret_imui::prelude::UiWriter as _;",
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn compat_section_text<",
        "decl_text::text_section_chrome_label(cx, text)",
        "compat_section_text(cx, \"imui node-graph compatibility proof\")",
        "NodeGraphSurfaceCompatRetainedProps::new(",
        "node_graph_surface_compat_retained(",
    ] {
        assert!(
            source.contains(needle),
            "imui_node_graph_demo should keep the retained bridge compatibility proof explicit and route fixed title text through the shared section role; missing `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::ui::text(\"imui node-graph compatibility proof\")",
        ".font_semibold()",
    ] {
        assert!(
            !source.contains(needle),
            "imui_node_graph_demo should not hand-roll local title text styling; unexpected `{needle}`"
        );
    }
}
