pub const SOURCE: &str = include_str!("custom_text.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

const CMD_PAGE_PREVIOUS: &str = "ui_gallery.pagination.custom_text.previous";
const CMD_PAGE_11: &str = "ui_gallery.pagination.custom_text.page_11";
const CMD_PAGE_12: &str = "ui_gallery.pagination.custom_text.page_12";
const CMD_PAGE_13: &str = "ui_gallery.pagination.custom_text.page_13";
const CMD_PAGE_NEXT: &str = "ui_gallery.pagination.custom_text.next";

fn page_number(label: &'static str) -> impl UiChild + use<> {
    ui::text(label).tabular_nums()
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::pagination(|cx| {
        ui::children![
            cx;
            shadcn::pagination_content(|cx| {
                ui::children![
                    cx;
                    shadcn::pagination_item(
                        shadcn::PaginationPrevious::new()
                            .text("Older")
                            .action(CMD_PAGE_PREVIOUS),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("11")])
                            .action(CMD_PAGE_11),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("12")])
                            .active(true)
                            .action(CMD_PAGE_12),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("13")])
                            .action(CMD_PAGE_13),
                    ),
                    shadcn::pagination_item(
                        shadcn::PaginationNext::new()
                            .text("Newer")
                            .action(CMD_PAGE_NEXT),
                    ),
                ]
            }),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-pagination-custom-text")
}
// endregion: example
