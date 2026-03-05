use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_workflow_chrome_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::workflow_chrome_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![
            DocSection::new("Workflow Chrome", demo)
                .test_id_prefix("ui-gallery-ai-workflow-chrome-demo")
                .code_rust_from_file_region(snippets::workflow_chrome_demo::SOURCE, "example"),
        ],
    );

    vec![body]
}
