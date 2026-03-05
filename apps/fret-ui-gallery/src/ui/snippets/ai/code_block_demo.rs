pub const SOURCE: &str = include_str!("code_block_demo.rs");

// region: example
use fret_icons_lucide::generated_ids::lucide::FILE_CODE;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let muted_fg = theme.color_token("muted-foreground");

    let code: Arc<str> = Arc::from(
        "fn hello(name: &str) {\n    println!(\"hello, {name}\");\n}\n\nhello(\"world\");\n",
    );

    let title = ui::h_row(|cx| {
        vec![
            decl_icon::icon_with(
                cx,
                FILE_CODE,
                Some(Px(14.0)),
                Some(ColorRef::Color(muted_fg)),
            ),
            ui_ai::CodeBlockFilename::new("example.rs").into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let code_block = ui_ai::CodeBlock::new(code.clone())
        .language("rust")
        .show_language(false)
        .header_left([title])
        .header_right([ui_ai::CodeBlockCopyButton::new(code.clone())
            .test_id("ui-ai-code-block-copy")
            .copied_marker_test_id("ui-ai-code-block-copied-marker")
            .into_element(cx)])
        .test_id("ui-ai-code-block-root")
        .into_element(cx);

    let snippet_code: Arc<str> = Arc::from("cargo run -p fret-ui-gallery --release");
    let snippet = ui_ai::Snippet::new([
        ui_ai::SnippetText::new("$").into_element(cx),
        ui_ai::SnippetInput::new(snippet_code.clone()).into_element(cx),
        ui_ai::SnippetCopyButton::new(snippet_code)
            .test_id("ui-ai-snippet-copy")
            .copied_marker_test_id("ui-ai-snippet-copied-marker")
            .into_element(cx),
    ])
    .test_id("ui-ai-snippet-root")
    .into_element(cx);

    ui::v_flex(|cx| {
        vec![
            cx.text("CodeBlock (AI Elements)"),
            cx.text("Backed by `fret-code-view` with a copy surface."),
            code_block,
            snippet,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
