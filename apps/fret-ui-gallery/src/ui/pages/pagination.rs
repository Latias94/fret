use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::pagination as snippets;

pub(super) fn preview_pagination(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let rtl = snippets::rtl::render(cx);
    let extras = snippets::extras::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Pagination demo (new-york-v4).",
            "Pagination primitives are intentionally small; compose them with routing/actions in your app layer.",
            "RTL examples validate icon direction + number shaping under RTL.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Pagination demo: Previous, numbered links, ellipsis, Next."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-pagination-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-pagination-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-pagination-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-pagination")]
}
