pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

const CMD_PAGE_PREVIOUS: &str = "ui_gallery.pagination.previous";
const CMD_PAGE_1: &str = "ui_gallery.pagination.page_1";
const CMD_PAGE_2: &str = "ui_gallery.pagination.page_2";
const CMD_PAGE_NEXT: &str = "ui_gallery.pagination.next";

fn page_number<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    fret_ui_kit::ui::text(label).tabular_nums().into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let content = shadcn::PaginationContent::new([
        shadcn::PaginationItem::new(
            shadcn::PaginationPrevious::new()
                .on_click(CMD_PAGE_PREVIOUS)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "1")])
                .on_click(CMD_PAGE_1)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "2")])
                .active(true)
                .on_click(CMD_PAGE_2)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
            .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationNext::new()
                .on_click(CMD_PAGE_NEXT)
                .into_element(cx),
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Pagination::new([content])
        .into_element(cx)
        .test_id("ui-gallery-pagination-usage")
}
// endregion: example
