pub const SOURCE: &str = include_str!("rating.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::raw::extras::Rating::uncontrolled(3)
        .count(5)
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-rating")
}
// endregion: example
