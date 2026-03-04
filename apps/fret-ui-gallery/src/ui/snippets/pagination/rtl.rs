pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

const CMD_APP_OPEN: &str = "ui_gallery.app.open";
const CMD_APP_SAVE: &str = "ui_gallery.app.save";

fn to_arabic_numerals(num: u32) -> String {
    const DIGITS: [&str; 10] = ["٠", "١", "٢", "٣", "٤", "٥", "٦", "٧", "٨", "٩"];
    num.to_string()
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| DIGITS[d as usize]))
        .collect()
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationPrevious::new()
                    .text("السابق")
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text(to_arabic_numerals(1))])
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text(to_arabic_numerals(2))])
                    .on_click(CMD_APP_SAVE)
                    .active(true)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text(to_arabic_numerals(3))])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
                .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationNext::new()
                    .text("التالي")
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        shadcn::Pagination::new([content])
            .into_element(cx)
            .test_id("ui-gallery-pagination-rtl")
    })
}
// endregion: example
