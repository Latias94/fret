pub const SOURCE: &str = include_str!("routing.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::facade as shadcn;

const ROUTE_PAGE_PREVIOUS: &str = "app.router.pagination.previous";
const ROUTE_PAGE_7: &str = "app.router.pagination.page_7";
const ROUTE_PAGE_8: &str = "app.router.pagination.page_8";
const ROUTE_PAGE_9: &str = "app.router.pagination.page_9";
const ROUTE_PAGE_NEXT: &str = "app.router.pagination.next";

fn page_number(label: &'static str) -> impl UiChild + use<> {
    ui::text(label).tabular_nums()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Pagination::new([shadcn::PaginationContent::new([
        shadcn::PaginationItem::new(
            shadcn::PaginationPrevious::new()
                .action(ROUTE_PAGE_PREVIOUS)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("7").into_element(cx)])
                .action(ROUTE_PAGE_7)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("8").into_element(cx)])
                .active(true)
                .action(ROUTE_PAGE_8)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("9").into_element(cx)])
                .action(ROUTE_PAGE_9)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationNext::new()
                .action(ROUTE_PAGE_NEXT)
                .into_element(cx),
        )
        .into_element(cx),
    ])
    .into_element(cx)])
    .into_element(cx)
    .test_id("ui-gallery-pagination-routing")
}
// endregion: example
