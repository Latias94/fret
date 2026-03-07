use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_conversation_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::conversation_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned AI Elements Conversation composition: root scroll container + content slot + overlay parts.",
        ),
        vec![
            DocSection::new("Conversation", demo)
                .test_id_prefix("ui-gallery-ai-conversation-demo")
                .code_rust_from_file_region(snippets::conversation_demo::SOURCE, "example"),
        ],
    );

    vec![body]
}
