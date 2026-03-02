use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_stack_trace_large_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::stack_trace_large_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![DocSection::new("StackTrace (Large)", demo)
            .max_w(Px(820.0))
            .test_id_prefix("ui-gallery-ai-stack-trace-large-demo")
            .code_rust_from_file_region(
                snippets::stack_trace_large_demo::SOURCE,
                "example",
            )],
    );

    vec![body.test_id("ui-gallery-page-ai-stack-trace-large-demo")]
}

