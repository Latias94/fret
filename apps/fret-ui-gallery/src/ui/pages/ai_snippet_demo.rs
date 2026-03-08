use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_snippet_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::snippet_demo::render(cx);
    let plain = snippets::snippet_plain::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("Lightweight inline code display for terminal commands and short code references."),
        vec![
            DocSection::new("Snippet", demo)
                .descriptions([
                    "Composable add-on structure aligned with AI Elements.",
                    "Optional prefix text for terminal-style commands.",
                    "Built-in copy button with copied feedback marker.",
                    "Compact inline layout for chat and docs surfaces.",
                ])
                .test_id_prefix("ui-gallery-ai-snippet-demo")
                .code_rust_from_file_region(snippets::snippet_demo::SOURCE, "example"),
            DocSection::new("Without Prefix", plain)
                .test_id_prefix("ui-gallery-ai-snippet-plain")
                .code_rust_from_file_region(snippets::snippet_plain::SOURCE, "example"),
        ],
    );

    vec![body]
}
