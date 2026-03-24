use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_prompt_input_provider_demo(
    cx: &mut UiCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::prompt_input_provider_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "AI Elements are policy-level compositions built on top of lower-level primitives.",
        ),
        vec![
            DocSection::build(cx, "Prompt Input Provider", demo)
                .description(
                    "Intentional provider seam: this example stays on `PromptInputProvider` plus the plain prompt root because the teaching goal is lifted text/attachment ownership rather than footer-tools composition.",
                )
                .test_id_prefix("ui-gallery-ai-prompt-input-provider-demo")
                .code_rust_from_file_region(
                    snippets::prompt_input_provider_demo::SOURCE,
                    "example",
                ),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
