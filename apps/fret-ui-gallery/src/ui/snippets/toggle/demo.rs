pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::h_row(|cx| {
        vec![
            shadcn::toggle_uncontrolled(cx, false, |cx| {
                ui::children![
                    cx;
                    shadcn::raw::icon::icon(cx, IconId::new_static("lucide.bookmark")),
                    ui::text("Bookmark")
                ]
            })
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Sm)
            .a11y_label("Toggle bookmark")
            .into_element(cx)
            .test_id("ui-gallery-toggle-demo-bookmark"),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-toggle-demo")
}
// endregion: example
