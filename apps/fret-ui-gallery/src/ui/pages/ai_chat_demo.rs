use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_chat_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::chat_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("Interactive chat harness: PromptInput + transcript streaming + tool calls + sources."),
        vec![
            DocSection::new("Chat Demo", demo)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-ai-chat-demo")
                .code_rust_from_file_region(snippets::chat_demo::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-chat-demo")]
}
