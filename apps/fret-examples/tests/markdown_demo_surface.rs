fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn markdown_demo_chrome_text_uses_shared_roles() {
    let source = include_str!("../src/markdown_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::prelude::*;",
        "AppComponentCx,AppElement,AppRenderContext,LocalState,RenderContextAccessas_,text,",
        "usefret::scroll::ScrollHandle;",
        "MarkdownComponents::<App>::default()",
        "fnmarkdown_demo_readout_text<'a,Cx,T>(cx:&mutCx,text:T)->AppElement",
        "fnmarkdown_demo_title_text<'a,Cx,T>(cx:&mutCx,text:T)->AppElement",
        "fnmarkdown_demo_paragraph_text<'a,Cx,T>(cx:&mutCx,text:T)->AppElement",
        "fnmarkdown_demo_image_placeholder_text<'a,Cx,T>(",
        "text::control_readout(cx,text)",
        "text::section_chrome_label(cx,text)",
        "text::paragraph(cx,text)",
        "text::paragraph_break_words_with_foreground(cx,text,foreground)",
        "markdown_demo_readout_text(cx,format!(\"wrapcode:{}\",ifwrap_enabled{\"on\"}else{\"off\"}),)",
        "markdown_demo_readout_text(cx,format!(\"capcodeheight:{}\",ifcap_enabled{\"on\"}else{\"off\"}),)",
        "markdown_demo_readout_text(cx,format!(\"expandedcodeblocks:{expanded_count}\"))",
        "markdown_demo_title_text(cx,\"markdown_demo\")",
        "markdown_demo_paragraph_text(cx,\"Scrollablemarkdownpreview(linksopenviaplatformshell).\",)",
        "letforeground=theme.color_token(\"muted-foreground\");",
        "vec![markdown_demo_image_placeholder_text(cx,text.clone(),foreground,)]",
        "markdown_demo_image_placeholder_text(cx,text,foreground)",
        "ui_assets::rgba8_image_state(cx,*width,*height,rgba.as_ref(),ui_assets::ImageColorSpace::Srgb,)",
        "ui_assets::rgba8_image_state(cx,96,96,checker_rgba.as_ref(),ui_assets::ImageColorSpace::Srgb,)",
    ] {
        assert!(
            source.contains(needle),
            "markdown demo should keep fixed chrome/status text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(format!(\"wrapcode:{}\",ifwrap_enabled{\"on\"}else{\"off\"}))",
        "cx.text(format!(\"capcodeheight:{}\",ifcap_enabled{\"on\"}else{\"off\"}))",
        "cx.text(format!(\"expandedcodeblocks:{expanded_count}\"))",
        "cx.text(\"markdown_demo\")",
        "cx.text(\"Scrollablemarkdownpreview(linksopenviaplatformshell).\")",
        "cx.text_props(TextProps{",
        "TextProps{",
        "wrap:fret_core::TextWrap::Word",
        "overflow:fret_core::TextOverflow::Clip",
        "decl_text::",
        "fret_ui::ElementContext",
        "fret_ui::UiHost",
        "AnyElement",
        "fret_ui::scroll::ScrollHandle",
        "usefret_ui_assets::ui::use_rgba8_image_state_in;",
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
        "usefret_ui_shadcn::facadeasshadcn;",
    ] {
        assert!(
            !source.contains(needle),
            "markdown demo should not render fixed chrome/status text with bare wrapping text; unexpected `{needle}`"
        );
    }
}
