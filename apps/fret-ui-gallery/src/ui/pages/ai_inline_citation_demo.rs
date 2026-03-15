use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::UiCx;

pub(super) fn preview_ai_inline_citation_demo(
    cx: &mut UiCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::inline_citation_demo::render(cx);
    let notes = crate::ui::doc_layout::notes_block([
        "Preview follows the official AI Elements Inline Citation example more closely: inline copy + hostname badge + hover card pager.",
        "`InlineCitation::with_children(...)` keeps the root copy composable without pushing AI policy into `crates/fret-ui`.",
        "The hover card, hostname/count badge, and pager remain policy-level behavior in `fret-ui-ai`.",
    ]);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("AI Elements are policy-level compositions built on top of lower-level primitives."),
        vec![
            DocSection::build(cx, "Demo", demo)
                .test_id_prefix("ui-gallery-ai-inline-citation-demo")
                .description(
                    "Docs-style inline citation example with multi-source hover-card paging.",
                )
                .code_rust_from_file_region(snippets::inline_citation_demo::SOURCE, "example"),
            DocSection::build(cx, "Notes", notes)
                .description("Layering and API notes.")
                .no_shell()
                .test_id_prefix("ui-gallery-ai-inline-citation-notes"),
        ],
    );

    vec![body.into_element(cx)]
}
