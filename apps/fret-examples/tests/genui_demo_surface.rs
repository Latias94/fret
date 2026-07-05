fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn genui_demo_keeps_tool_text_on_roles() {
    let source = include_str!("../src/genui_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret_ui::{ElementContext,UiHost};",
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fngenui_code_line_text<H:UiHost>(",
        "fngenui_readout_text<H:UiHost>(",
        "fngenui_paragraph_text<H:UiHost>(",
        "decl_text::text_code_block(cx,text)",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_compact_paragraph(cx,text)",
        ".map(|line|genui_code_line_text(cx,line))",
        "items.push(genui_readout_text(cx,\"auto-apply\"));",
        "items.push(genui_readout_text(cx,\"auto-fixonapply\"));",
        "items.push(genui_readout_text(cx,count_label.clone()));",
        ".map(|s|genui_readout_text(cx,s)),",
        "vec![genui_readout_text(cx,\"Nospecissues.\")]",
        "genui_readout_text(",
        "\"patch-only:{}\",",
        "stream_children.push(genui_paragraph_text(",
        "stream_children.push(genui_readout_text(cx,summary));",
    ] {
        assert!(
            compact_source.contains(needle),
            "GenUI demo tool text should use shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(",
        "ui::rich_text(",
        ".text_sm()",
        ".font_semibold()",
        ".font_medium()",
        ".truncate()",
        "editor_children.push(ui::text(\"\").text_sm().into_element(cx));",
    ] {
        assert!(
            !compact_source.contains(needle),
            "GenUI demo should not render tool text with local ui::text policy; unexpected `{needle}`"
        );
    }
}

#[test]
fn genui_demo_uses_explicit_public_surfaces() {
    let source = include_str!("../src/genui_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret::advanced::KernelApp;",
        "usefret::advanced::driver::ViewElements;",
        "usefret::app::LocalState;",
        "usefret::app::prelude::*;",
        "usefret::style::{ColorRef,Space,ThemeSnapshot};",
        "usefret::AppComponentCx;",
        "usefret_runtime::Model;",
        "usefret_ui_kit::IntoUiElement;",
    ] {
        assert!(
            compact_source.contains(needle),
            "GenUI demo should name its required app/advanced/style surfaces explicitly; missing `{needle}`"
        );
    }

    for forbidden in [
        "LocalStateElementContextExt",
        "advanced::prelude::*",
        "component::prelude::*",
    ] {
        assert!(
            !source.contains(forbidden),
            "GenUI demo should not reintroduce broad prelude imports: `{forbidden}`",
        );
    }
}
