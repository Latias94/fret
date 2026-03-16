use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::shadcn_extras as snippets;

pub(super) fn preview_shadcn_extras(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let announcement = snippets::announcement::render(cx);
    let banner = snippets::banner::render(cx);
    let tags = snippets::tags::render(cx);
    let marquee = snippets::marquee::render(cx);
    let kanban = snippets::kanban::render(cx);
    let ticker = snippets::ticker::render(cx);
    let relative_time = snippets::relative_time::render(cx);
    let rating = snippets::rating::render(cx);
    let avatar_stack = snippets::avatar_stack::render(cx);
    let announcement = DocSection::build(cx, "Announcement", announcement)
        .code_rust_from_file_region(snippets::announcement::SOURCE, "example");
    let banner = DocSection::build(cx, "Banner (dismissible)", banner)
        .code_rust_from_file_region(snippets::banner::SOURCE, "example");
    let tags = DocSection::build(cx, "Tags", tags)
        .code_rust_from_file_region(snippets::tags::SOURCE, "example");
    let marquee = DocSection::build(cx, "Marquee (pause on hover)", marquee)
        .code_rust_from_file_region(snippets::marquee::SOURCE, "example");
    let kanban = DocSection::build(cx, "Kanban (drag & drop)", kanban)
        .code_rust_from_file_region(snippets::kanban::SOURCE, "example");
    let ticker = DocSection::build(cx, "Ticker", ticker)
        .code_rust_from_file_region(snippets::ticker::SOURCE, "example");
    let relative_time = DocSection::build(cx, "Relative time", relative_time)
        .code_rust_from_file_region(snippets::relative_time::SOURCE, "example");
    let rating = DocSection::build(cx, "Rating", rating)
        .code_rust_from_file_region(snippets::rating::SOURCE, "example");
    let avatar_stack = DocSection::build(cx, "Avatar stack", avatar_stack)
        .code_rust_from_file_region(snippets::avatar_stack::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "A small grab-bag of shadcn-style extras; each section is intentionally self-contained.",
        ),
        vec![
            announcement,
            banner,
            tags,
            marquee,
            kanban,
            ticker,
            relative_time,
            rating,
            avatar_stack,
        ],
    );

    let body = body.test_id("ui-gallery-shadcn-extras-component");
    vec![body.into_element(cx)]
}
