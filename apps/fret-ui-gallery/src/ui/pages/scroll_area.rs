use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::scroll_area as snippets;

pub(super) fn preview_scroll_area(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let horizontal = snippets::horizontal::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "ScrollArea is for custom scrollbars + consistent styling; use native scrolling when you don't need custom chrome.",
            "Keep scroll region sizes explicit in docs to avoid layout drift.",
            "Horizontal rails are easiest to reason about when the child has a fixed width.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn ScrollArea demo: Vertical + Horizontal."),
        vec![
            DocSection::new("Demo", demo)
                .description("Vertical scroll region with tags and separators.")
                .code_rust_from_file_region(include_str!("../snippets/scroll_area/demo.rs"), "example"),
            DocSection::new("Horizontal", horizontal)
                .description("Horizontal rail (fixed-size items) inside a scroll area.")
                .code_rust_from_file_region(
                    include_str!("../snippets/scroll_area/horizontal.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("ScrollArea behavior under an RTL direction provider.")
                .code_rust_from_file_region(include_str!("../snippets/scroll_area/rtl.rs"), "example"),
            DocSection::new("Notes", notes).description("Usage notes and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-scroll-area")]
}
