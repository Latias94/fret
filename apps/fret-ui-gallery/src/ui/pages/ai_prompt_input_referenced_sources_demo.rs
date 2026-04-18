use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::AppComponentCx;

pub(super) fn preview_ai_prompt_input_referenced_sources_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::prompt_input_referenced_sources_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "AI Elements are policy-level compositions built on top of lower-level primitives.",
        ),
        vec![
            DocSection::build(cx, "Prompt Input Referenced Sources", demo)
                .description(
                    "Intentional header-slot seam: referenced sources currently teach `PromptInputRoot::into_element_with_slots(...)` because the example only needs the block-start chip row, not the full textarea/body children lane.",
                )
                .test_id_prefix("ui-gallery-ai-prompt-input-referenced-sources-demo")
                .code_rust_from_file_region(
                    snippets::prompt_input_referenced_sources_demo::SOURCE,
                    "example",
                ),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
