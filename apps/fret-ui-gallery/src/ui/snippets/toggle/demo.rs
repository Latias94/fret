pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_row(|cx| {
        vec![shadcn::Toggle::uncontrolled(false)
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Sm)
            .a11y_label("Toggle bookmark")
            .leading_icon(IconId::new_static("lucide.bookmark"))
            .label("Bookmark")
            .into_element(cx)
            .test_id("ui-gallery-toggle-demo-bookmark")]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-toggle-demo")
}
// endregion: example
