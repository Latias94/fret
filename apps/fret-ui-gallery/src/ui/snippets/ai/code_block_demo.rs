pub const SOURCE: &str = include_str!("code_block_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let code: Arc<str> = Arc::from(
        "fn hello(name: &str) {\n    println!(\"hello, {name}\");\n}\n\nhello(\"world\");\n",
    );

    let code_block = ui_ai::CodeBlock::new(code.clone())
        .language("rust")
        .show_header(true)
        .show_language(true)
        .header_right([ui_ai::CodeBlockCopyButton::new(code.clone())
            .test_id("ui-ai-code-block-copy")
            .copied_marker_test_id("ui-ai-code-block-copied-marker")
            .into_element(cx)])
        .test_id("ui-ai-code-block-root")
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("CodeBlock (AI Elements)"),
                cx.text("Backed by `fret-code-view` with a copy surface."),
                code_block,
            ]
        },
    )
}
// endregion: example
