use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::shadcn_extras as snippets;

pub(super) fn preview_shadcn_extras(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let announcement = snippets::announcement::render(cx);
    let banner = snippets::banner::render(cx);
    let tags = snippets::tags::render(cx);
    let marquee = snippets::marquee::render(cx);
    let kanban = snippets::kanban::render(cx);
    let ticker = snippets::ticker::render(cx);
    let relative_time = snippets::relative_time::render(cx);
    let rating = snippets::rating::render(cx);
    let avatar_stack = snippets::avatar_stack::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "A small grab-bag of shadcn-style extras; each section is intentionally self-contained.",
        ),
        vec![
            DocSection::new("Announcement", announcement)
                .code_rust_from_file_region(snippets::announcement::SOURCE, "example"),
            DocSection::new("Banner (dismissible)", banner)
                .code_rust_from_file_region(snippets::banner::SOURCE, "example"),
            DocSection::new("Tags", tags)
                .code_rust_from_file_region(snippets::tags::SOURCE, "example"),
            DocSection::new("Marquee (pause on hover)", marquee)
                .code_rust_from_file_region(snippets::marquee::SOURCE, "example"),
            DocSection::new("Kanban (drag & drop)", kanban)
                .code_rust_from_file_region(snippets::kanban::SOURCE, "example"),
            DocSection::new("Ticker", ticker)
                .code_rust_from_file_region(snippets::ticker::SOURCE, "example"),
            DocSection::new("Relative time", relative_time)
                .code_rust_from_file_region(snippets::relative_time::SOURCE, "example"),
            DocSection::new("Rating", rating)
                .code_rust_from_file_region(snippets::rating::SOURCE, "example"),
            DocSection::new("Avatar stack", avatar_stack)
                .code_rust_from_file_region(snippets::avatar_stack::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-shadcn-extras-component")]
}
