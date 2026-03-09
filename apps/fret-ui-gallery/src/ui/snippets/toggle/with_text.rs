pub const SOURCE: &str = include_str!("with_text.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_row(|cx| {
        vec![
            shadcn::Toggle::uncontrolled(false)
                .a11y_label("Toggle italic with text")
                .leading_icon(IconId::new_static("lucide.italic"))
                .label("Italic")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-toggle-with-text")
}
// endregion: example
