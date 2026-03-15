use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_workflow_edge_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::workflow_edge_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![
            DocSection::build(cx, "Workflow Edge", demo)
                .test_id_prefix("ui-gallery-ai-workflow-edge-demo")
                .code_rust_from_file_region(snippets::workflow_edge_demo::SOURCE, "example"),
        ],
    );

    vec![body.into_element(cx)]
}
