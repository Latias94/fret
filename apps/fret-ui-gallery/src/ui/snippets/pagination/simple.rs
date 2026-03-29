pub const SOURCE: &str = include_str!("simple.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

const CMD_APP_OPEN: &str = "ui_gallery.pagination.simple.open";
const CMD_APP_SAVE: &str = "ui_gallery.pagination.simple.save";

fn page_number(label: &'static str) -> impl UiChild + use<> {
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
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("1")])
                            .action(CMD_APP_OPEN),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("2")])
                            .action(CMD_APP_SAVE)
                            .active(true),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("3")])
                            .action(CMD_APP_SAVE),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("4")])
                            .action(CMD_APP_SAVE),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number("5")])
                            .action(CMD_APP_SAVE),
                    ),
                ]
            }),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-pagination-simple")
}
// endregion: example
