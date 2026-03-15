pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";
const CMD_APP_SAVE: &str = "ui_gallery.app.save";

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let page_number = |cx: &mut UiCx<'_>, label: &'static str| {
        fret_ui_kit::ui::text(label).tabular_nums().into_element(cx)
    };

    shadcn::pagination(|cx| {
        ui::children![
            cx;
            shadcn::pagination_content(|cx| {
                ui::children![
                    cx;
                    shadcn::pagination_item(
                        shadcn::PaginationPrevious::new().on_click(CMD_APP_OPEN),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number(cx, "1")])
                            .on_click(CMD_APP_OPEN),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number(cx, "2")])
                            .on_click(CMD_APP_SAVE)
                            .active(true),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number(cx, "3")])
                            .on_click(CMD_APP_SAVE),
                    ),
                    shadcn::pagination_item(shadcn::PaginationEllipsis::new()),
                    shadcn::pagination_item(
                        shadcn::PaginationNext::new().on_click(CMD_APP_SAVE),
                    ),
                ]
            }),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-pagination-demo")
}
// endregion: example
