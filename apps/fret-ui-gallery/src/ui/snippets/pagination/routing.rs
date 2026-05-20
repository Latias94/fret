pub const SOURCE: &str = include_str!("routing.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{IntoUiElement, declarative::text as decl_text, ui};
use fret_ui_shadcn::facade as shadcn;

const ROUTE_PAGE_PREVIOUS: &str = "app.router.pagination.previous";
const ROUTE_PAGE_7: &str = "app.router.pagination.page_7";
const ROUTE_PAGE_8: &str = "app.router.pagination.page_8";
const ROUTE_PAGE_9: &str = "app.router.pagination.page_9";
const ROUTE_PAGE_NEXT: &str = "app.router.pagination.next";

fn page_number<H, L>(cx: &mut ElementContext<'_, H>, label: L) -> impl UiChild + use<H, L>
where
    H: UiHost,
    L: Into<std::sync::Arc<str>>,
{
    decl_text::text_button_label(cx, label)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Pagination::new([shadcn::PaginationContent::new([
        shadcn::PaginationItem::new(
            shadcn::PaginationPrevious::new()
                .action(ROUTE_PAGE_PREVIOUS)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "7").into_element(cx)])
                .action(ROUTE_PAGE_7)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "8").into_element(cx)])
                .active(true)
                .action(ROUTE_PAGE_8)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "9").into_element(cx)])
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
