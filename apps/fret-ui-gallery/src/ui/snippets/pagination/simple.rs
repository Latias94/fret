pub const SOURCE: &str = include_str!("simple.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{declarative::text as decl_text, ui};
use fret_ui_shadcn::facade as shadcn;

const CMD_APP_OPEN: &str = "ui_gallery.pagination.simple.open";
const CMD_APP_SAVE: &str = "ui_gallery.pagination.simple.save";

fn page_number<H, L>(cx: &mut ElementContext<'_, H>, label: L) -> impl UiChild + use<H, L>
where
    H: UiHost,
    L: Into<std::sync::Arc<str>>,
{
    decl_text::text_button_label(cx, label)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::pagination(|cx| {
        ui::children![
            cx;
            shadcn::pagination_content(|cx| {
                ui::children![
                    cx;
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number(cx, "1")])
                            .action(CMD_APP_OPEN),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number(cx, "2")])
                            .action(CMD_APP_SAVE)
                            .active(true),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number(cx, "3")])
                            .action(CMD_APP_SAVE),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number(cx, "4")])
                            .action(CMD_APP_SAVE),
                    ),
                    shadcn::pagination_item(
                        shadcn::pagination_link(|cx| ui::children![cx; page_number(cx, "5")])
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
