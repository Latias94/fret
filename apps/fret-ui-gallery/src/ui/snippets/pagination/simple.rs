pub const SOURCE: &str = include_str!("simple.rs");

// region: example
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_APP_OPEN: &str = "ui_gallery.pagination.simple.open";
const CMD_APP_SAVE: &str = "ui_gallery.pagination.simple.save";

fn page_number<H: UiHost>(label: &'static str) -> impl IntoUiElement<H> + use<H> {
    fret_ui_kit::ui::text(label).tabular_nums()
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let content = shadcn::PaginationContent::new([
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("1").into_element(cx)])
                .on_click(CMD_APP_OPEN)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("2").into_element(cx)])
                .on_click(CMD_APP_SAVE)
                .active(true)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("3").into_element(cx)])
                .on_click(CMD_APP_SAVE)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("4").into_element(cx)])
                .on_click(CMD_APP_SAVE)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([page_number("5").into_element(cx)])
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
