pub const SOURCE: &str = include_str!("spinner.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::h_row(|cx| {
        vec![
            shadcn::Spinner::new().into_element(cx),
            shadcn::Spinner::new().speed(0.0).into_element(cx),
            cx.text("Spinner (animated / static)"),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-icons-spinner-row")
}
// endregion: example
