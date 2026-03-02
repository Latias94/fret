pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Toggle::uncontrolled(false)
                    .variant(shadcn::ToggleVariant::Outline)
                    .size(shadcn::ToggleSize::Sm)
                    .a11y_label("Toggle bookmark")
                    .leading_icon(IconId::new_static("lucide.bookmark"))
                    .label("Bookmark")
                    .into_element(cx)
                    .test_id("ui-gallery-toggle-demo-bookmark"),
            ]
        },
    )
    .test_id("ui-gallery-toggle-demo")
}
// endregion: example
