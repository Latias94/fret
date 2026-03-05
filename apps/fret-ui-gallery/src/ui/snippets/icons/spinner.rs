pub const SOURCE: &str = include_str!("spinner.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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
