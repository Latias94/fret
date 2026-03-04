pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Toggle::uncontrolled(false)
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Sm)
            .a11y_label("Toggle bookmark rtl")
            .leading_icon(IconId::new_static("lucide.bookmark"))
            .label("Bookmark")
            .into_element(cx)
    })
    .test_id("ui-gallery-toggle-rtl")
}
// endregion: example
