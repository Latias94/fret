use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::toast as snippets;

pub(super) fn preview_toast(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let deprecated = snippets::deprecated::render(cx);

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("A succinct message that is displayed temporarily."),
        vec![
            DocSection::new("Deprecated", deprecated)
                .description("Toast is deprecated in upstream shadcn/ui docs. Prefer Sonner.")
                .test_id_prefix("ui-gallery-toast-deprecated")
                .code_rust_from_file_region(snippets::deprecated::SOURCE, "example"),
        ],
    )
    .test_id("ui-gallery-toast");

    vec![body]
}
