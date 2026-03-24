use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_workflow_node_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::workflow_node_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![
            DocSection::build(cx, "Workflow Node", demo)
                .test_id_prefix("ui-gallery-ai-workflow-node-demo")
                .code_rust_from_file_region(snippets::workflow_node_demo::SOURCE, "example"),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
