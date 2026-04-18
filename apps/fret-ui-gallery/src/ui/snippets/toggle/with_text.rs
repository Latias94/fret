pub const SOURCE: &str = include_str!("with_text.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::h_row(|cx| {
        vec![
            shadcn::toggle_uncontrolled(cx, false, |cx| {
                ui::children![
                    cx;
                    shadcn::raw::icon::icon(cx, IconId::new_static("lucide.italic")),
                    ui::text("Italic")
                ]
            })
            .a11y_label("Toggle italic with text")
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-toggle-with-text")
}
// endregion: example
