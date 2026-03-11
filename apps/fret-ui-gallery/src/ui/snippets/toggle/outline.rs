pub const SOURCE: &str = include_str!("outline.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_row(|cx| {
        vec![
            shadcn::Toggle::uncontrolled(false)
                .variant(shadcn::ToggleVariant::Outline)
                .a11y_label("Toggle italic")
                .leading_icon(IconId::new_static("lucide.italic"))
                .label("Italic")
                .into_element(cx)
                .test_id("ui-gallery-toggle-outline-italic"),
            shadcn::Toggle::uncontrolled(false)
                .variant(shadcn::ToggleVariant::Outline)
                .a11y_label("Toggle bold")
                .leading_icon(IconId::new_static("lucide.bold"))
                .label("Bold")
                .into_element(cx)
                .test_id("ui-gallery-toggle-outline-bold"),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-toggle-outline")
}
// endregion: example
