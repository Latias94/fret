use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_chat_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::chat_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Interactive chat harness: PromptInput + transcript streaming + tool calls + sources.",
        ),
        vec![
            DocSection::build(cx, "Chat Demo", demo)
                .test_id_prefix("ui-gallery-ai-chat-demo")
                .code_rust_from_file_region(snippets::chat_demo::SOURCE, "example"),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
