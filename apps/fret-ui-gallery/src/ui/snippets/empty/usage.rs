pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let icon = icon::icon(cx, fret_icons::IconId::new_static("lucide.folder-search"));

    shadcn::empty(|cx| {
        ui::children![
            cx;
            shadcn::empty_header(|cx| {
                ui::children![
                    cx;
                    shadcn::empty_media(|cx| ui::children![cx; icon])
                        .variant(shadcn::EmptyMediaVariant::Icon),
                    shadcn::empty_title("No data"),
                    shadcn::empty_description("No data found."),
                ]
            }),
            shadcn::empty_content(|cx| ui::children![cx; shadcn::Button::new("Add data"),]),
        ]
    })
    .into_element(cx)
}
// endregion: example
