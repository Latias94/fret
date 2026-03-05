use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_prompt_input_docs_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::prompt_input_docs_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("Docs-aligned PromptInput composition (AI Elements)."),
        vec![
            DocSection::new("Prompt Input (Docs-aligned)", demo)
                .test_id_prefix("ui-gallery-ai-prompt-input-docs-demo")
                .code_rust_from_file_region(snippets::prompt_input_docs_demo::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-page-ai-prompt-input-docs-demo")]
}
