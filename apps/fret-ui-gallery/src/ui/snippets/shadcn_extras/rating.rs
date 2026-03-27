pub const SOURCE: &str = include_str!("rating.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::extras::Rating::uncontrolled(3)
        .count(5)
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-rating")
}
// endregion: example
