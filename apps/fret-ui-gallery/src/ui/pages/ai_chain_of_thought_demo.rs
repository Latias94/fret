use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_chain_of_thought_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::chain_of_thought_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "A collapsible component that visualizes AI reasoning steps with support for search results, images, and step-by-step progress indicators.",
        ),
        vec![
            DocSection::new("Chain of Thought", demo)
                .test_id_prefix("ui-gallery-ai-chain-of-thought-demo")
                .code_rust_from_file_region(snippets::chain_of_thought_demo::SOURCE, "example"),
        ],
    );

    vec![body]
}
