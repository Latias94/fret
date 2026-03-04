use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::navigation_menu as snippets;

pub(super) fn preview_navigation_menu(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let docs_demo = snippets::docs_demo::render(cx);
    let demo_with_toggle = snippets::demo::render(cx);
    let rtl = snippets::rtl::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Navigation Menu docs order: Demo, RTL. Container query toggle is a Fret-specific extra.",
        ),
        vec![
            DocSection::new("Demo", docs_demo)
                .code_rust_from_file_region(snippets::docs_demo::SOURCE, "example"),
            DocSection::new("Container Query Toggle", demo_with_toggle)
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-page-navigation-menu")]
}
