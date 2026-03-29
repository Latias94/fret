pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::facade as shadcn;

const CMD_PAGE_PREVIOUS: &str = "ui_gallery.pagination.previous";
const CMD_PAGE_1: &str = "ui_gallery.pagination.page_1";
const CMD_PAGE_2: &str = "ui_gallery.pagination.page_2";
const CMD_PAGE_3: &str = "ui_gallery.pagination.page_3";
const CMD_PAGE_NEXT: &str = "ui_gallery.pagination.next";

fn page_number(label: &'static str) -> impl UiChild + use<> {
    fret_ui_kit::ui::text(label).tabular_nums()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Pagination::new([shadcn::PaginationContent::new([
        shadcn::PaginationItem::new(
            shadcn::PaginationPrevious::new()
                .action(CMD_PAGE_PREVIOUS)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("1").into_element(cx)])
                .action(CMD_PAGE_1)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("2").into_element(cx)])
                .active(true)
                .action(CMD_PAGE_2)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("3").into_element(cx)])
                .action(CMD_PAGE_3)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
            .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationNext::new()
                .action(CMD_PAGE_NEXT)
                .into_element(cx),
        )
        .into_element(cx),
    ])
    .into_element(cx)])
    .into_element(cx)
    .test_id("ui-gallery-pagination-usage")
}
// endregion: example
