pub const SOURCE: &str = include_str!("compact_builder.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_PAGE_PREVIOUS: &str = "ui_gallery.pagination.compact.previous";
const CMD_PAGE_1: &str = "ui_gallery.pagination.compact.page_1";
const CMD_PAGE_2: &str = "ui_gallery.pagination.compact.page_2";
const CMD_PAGE_NEXT: &str = "ui_gallery.pagination.compact.next";

fn page_number<H: UiHost>(label: &'static str) -> impl IntoUiElement<H> + use<H> {
    fret_ui_kit::ui::text(label).tabular_nums()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::pagination(|cx| {
        ui::children![
            cx;
            shadcn::pagination_content(|cx| {
                ui::children![
                    cx;
                    shadcn::pagination_item(
                        shadcn::PaginationPrevious::new().action(CMD_PAGE_PREVIOUS),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("1")])
                            .action(CMD_PAGE_1),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("2")])
                            .active(true)
                            .action(CMD_PAGE_2),
                    ),
                    shadcn::pagination_item(shadcn::PaginationEllipsis::new()),
                    shadcn::pagination_item(
                        shadcn::PaginationNext::new().action(CMD_PAGE_NEXT),
                    ),
                ]
            }),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-pagination-compact-builder")
}
// endregion: example
