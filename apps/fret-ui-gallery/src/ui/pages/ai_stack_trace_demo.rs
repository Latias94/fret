use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_stack_trace_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::stack_trace_demo::render(cx);
    let collapsed = snippets::stack_trace_collapsed::render(cx);
    let no_internal = snippets::stack_trace_no_internal::render(cx);

    let notes = doc_layout::notes_block([
        "Mechanism/lifecycle looked healthy here: the main mismatch was component-layer API shape and the docs page not mirroring the official AI Elements examples.",
        "`StackTrace` now uses a closure-based compound children API so the UI Gallery example matches the upstream composition model without pushing policy into `fret-ui`.",
        "The separate large-list page still covers scroll and click seams; this page now focuses on docs parity and copyable examples.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned StackTrace examples using the same compound-parts composition model as the official AI Elements page.",
        ),
        vec![
            DocSection::build(cx, "Default", demo)
                .description("Expanded example with copy + file-open seams and compound children.")
                .test_id_prefix("ui-gallery-ai-stack-trace-demo")
                .code_rust_from_file_region(snippets::stack_trace_demo::SOURCE, "example"),
            DocSection::build(cx, "Collapsed by Default", collapsed)
                .description("Matches the official collapsed example.")
                .test_id_prefix("ui-gallery-ai-stack-trace-collapsed")
                .code_rust_from_file_region(snippets::stack_trace_collapsed::SOURCE, "example"),
            DocSection::build(cx, "Hide Internal Frames", no_internal)
                .description("Matches the official no-internal-frames example.")
                .test_id_prefix("ui-gallery-ai-stack-trace-no-internal")
                .code_rust_from_file_region(snippets::stack_trace_no_internal::SOURCE, "example"),
            DocSection::build(cx, "Notes", notes)
                .description("Layering + parity findings for StackTrace."),
        ],
    );

    vec![
        body.test_id("ui-gallery-page-ai-stack-trace-demo")
            .into_element(cx),
    ]
}
