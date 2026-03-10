pub const SOURCE: &str = include_str!("simple.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

const CMD_APP_OPEN: &str = "ui_gallery.pagination.simple.open";
const CMD_APP_SAVE: &str = "ui_gallery.pagination.simple.save";

fn page_number<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    fret_ui_kit::ui::text(label).tabular_nums().into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let content = shadcn::PaginationContent::new([
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "1")])
                .on_click(CMD_APP_OPEN)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "2")])
                .on_click(CMD_APP_SAVE)
                .active(true)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "3")])
                .on_click(CMD_APP_SAVE)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "4")])
                .on_click(CMD_APP_SAVE)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number(cx, "5")])
                .on_click(CMD_APP_SAVE)
                .into_element(cx),
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Pagination::new([content])
        .into_element(cx)
        .test_id("ui-gallery-pagination-simple")
}
// endregion: example
