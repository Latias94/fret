use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_code_block_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

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

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("CodeBlock + Snippet (AI Elements)"),
                cx.text(
                    "CodeBlock is backed by `fret-code-view` and exposes explicit copy surfaces.",
                ),
                code_block,
                snippet,
            ]
        },
    )]
}
