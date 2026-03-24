use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_open_in_chat_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::open_in_chat_demo::render(cx);
    let demo_section = DocSection::build(cx, "OpenIn", demo)
        .test_id_prefix("ui-gallery-ai-open-in-chat-demo")
        .code_rust_from_file_region(snippets::open_in_chat_demo::SOURCE, "example");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![demo_section],
    );

    vec![body.into_element(cx)]
}
