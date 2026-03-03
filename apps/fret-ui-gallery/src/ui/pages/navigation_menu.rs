use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::navigation_menu as snippets;

pub(super) fn preview_navigation_menu(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo_with_toggle = snippets::demo::render(cx);
    let rtl = snippets::rtl::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Navigation Menu docs order: Demo, RTL."),
        vec![
            DocSection::new("Demo", demo_with_toggle)
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-navigation-menu-component")]
}
