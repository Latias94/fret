pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

const CMD_APP_OPEN: &str = "ui_gallery.app.open";
const CMD_APP_SAVE: &str = "ui_gallery.app.save";

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let page_number = |cx: &mut ElementContext<'_, H>, label: &'static str| {
        fret_ui_kit::ui::text(label).tabular_nums().into_element(cx)
    };

    let content = shadcn::PaginationContent::new([
        shadcn::PaginationItem::new(
            shadcn::PaginationPrevious::new()
                .on_click(CMD_APP_OPEN)
                .into_element(cx),
        )
        .into_element(cx),
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
        shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
            .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationNext::new()
                .on_click(CMD_APP_SAVE)
                .into_element(cx),
        )
        .into_element(cx),
    ])
    .into_element(cx);

    shadcn::Pagination::new([content])
        .into_element(cx)
        .test_id("ui-gallery-pagination-demo")
}
// endregion: example
